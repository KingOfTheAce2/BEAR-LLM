use reqwest;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub model_id: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub pipeline_tag: Option<String>,
    pub license: Option<String>,
    pub downloads: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedModelCard {
    pub metadata: ModelMetadata,
    pub readme_content: String,
    pub cached_at: SystemTime,
}

pub struct ModelCardFetcher {
    #[allow(dead_code)]
    cache_dir: PathBuf,
    http_client: reqwest::Client,
    cache_ttl: Duration,
}

impl ModelCardFetcher {
    pub fn new(cache_dir: PathBuf) -> Self {
        // Ensure cache directory exists
        if let Err(e) = fs::create_dir_all(&cache_dir) {
            eprintln!("Warning: Failed to create cache directory: {}", e);
        }

        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            cache_dir,
            http_client,
            cache_ttl: Duration::from_secs(7 * 24 * 60 * 60), // 7 days
        }
    }

    /// Fetch model card from HuggingFace Hub API
    pub async fn fetch_model_card(&self, model_id: &str) -> Result<CachedModelCard, String> {
        // Check cache first
        if let Ok(cached) = self.get_cached_model_card(model_id) {
            if let Ok(elapsed) = cached.cached_at.elapsed() {
                if elapsed < self.cache_ttl {
                    return Ok(cached);
                }
            }
        }

        // Fetch from API
        match self.fetch_from_api(model_id).await {
            Ok(card) => {
                // Save to cache
                let _ = self.cache_model_card(model_id, &card);
                Ok(card)
            }
            Err(e) => {
                // Try to return stale cache if API fails
                if let Ok(cached) = self.get_cached_model_card(model_id) {
                    eprintln!("API fetch failed, using stale cache: {}", e);
                    return Ok(cached);
                }
                Err(e)
            }
        }
    }

    /// Fetch model metadata and README from HuggingFace API
    async fn fetch_from_api(&self, model_id: &str) -> Result<CachedModelCard, String> {
        // Fetch metadata
        let metadata_url = format!("https://huggingface.co/api/models/{}", model_id);
        let metadata: ModelMetadata = self.http_client
            .get(&metadata_url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch metadata: {}", e))?
            .json()
            .await
            .map_err(|e| format!("Failed to parse metadata: {}", e))?;

        // Fetch README
        let readme_url = format!("https://huggingface.co/{}/raw/main/README.md", model_id);
        let readme_content = self.http_client
            .get(&readme_url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch README: {}", e))?
            .text()
            .await
            .map_err(|e| format!("Failed to read README: {}", e))?;

        Ok(CachedModelCard {
            metadata,
            readme_content,
            cached_at: SystemTime::now(),
        })
    }

    /// Get cached model card
    fn get_cached_model_card(&self, model_id: &str) -> Result<CachedModelCard, String> {
        let cache_path = self.get_cache_path(model_id);

        let content = fs::read_to_string(&cache_path)
            .map_err(|e| format!("Failed to read cache: {}", e))?;

        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse cache: {}", e))
    }

    /// Save model card to cache
    fn cache_model_card(&self, model_id: &str, card: &CachedModelCard) -> Result<(), String> {
        let cache_path = self.get_cache_path(model_id);

        let json = serde_json::to_string_pretty(card)
            .map_err(|e| format!("Failed to serialize cache: {}", e))?;

        fs::write(&cache_path, json)
            .map_err(|e| format!("Failed to write cache: {}", e))
    }

    /// Get cache file path for a model
    fn get_cache_path(&self, model_id: &str) -> PathBuf {
        // Sanitize model_id for filesystem
        let safe_name = model_id.replace('/', "_").replace('\\', "_");
        self.cache_dir.join(format!("{}.json", safe_name))
    }

    /// Check if model card is cached and fresh
    pub fn is_cached(&self, model_id: &str) -> bool {
        if let Ok(cached) = self.get_cached_model_card(model_id) {
            if let Ok(elapsed) = cached.cached_at.elapsed() {
                return elapsed < self.cache_ttl;
            }
        }
        false
    }

    /// Clear cache for a specific model
    pub fn clear_cache(&self, model_id: &str) -> Result<(), String> {
        let cache_path = self.get_cache_path(model_id);
        fs::remove_file(&cache_path)
            .map_err(|e| format!("Failed to remove cache: {}", e))
    }

    /// Clear all cached model cards
    pub fn clear_all_cache(&self) -> Result<(), String> {
        fs::read_dir(&self.cache_dir)
            .map_err(|e| format!("Failed to read cache directory: {}", e))?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.path().extension()
                    .and_then(|s| s.to_str())
                    .map(|s| s == "json")
                    .unwrap_or(false)
            })
            .for_each(|entry| {
                let _ = fs::remove_file(entry.path());
            });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_cache_path_generation() {
        let temp_dir = tempdir().unwrap();
        let fetcher = ModelCardFetcher::new(temp_dir.path().to_path_buf());

        let path = fetcher.get_cache_path("meta-llama/Llama-2-7b-chat-hf");
        assert!(path.to_str().unwrap().contains("meta-llama_Llama-2-7b-chat-hf.json"));
    }

    #[test]
    fn test_cache_operations() {
        let temp_dir = tempdir().unwrap();
        let fetcher = ModelCardFetcher::new(temp_dir.path().to_path_buf());

        let card = CachedModelCard {
            metadata: ModelMetadata {
                model_id: "test/model".to_string(),
                author: Some("test".to_string()),
                description: Some("Test model".to_string()),
                tags: vec!["test".to_string()],
                pipeline_tag: None,
                license: Some("MIT".to_string()),
                downloads: Some(100),
            },
            readme_content: "# Test Model\n\nThis is a test.".to_string(),
            cached_at: SystemTime::now(),
        };

        // Save to cache
        assert!(fetcher.cache_model_card("test/model", &card).is_ok());

        // Read from cache
        let cached = fetcher.get_cached_model_card("test/model").unwrap();
        assert_eq!(cached.metadata.model_id, "test/model");
        assert_eq!(cached.readme_content, "# Test Model\n\nThis is a test.");

        // Clear cache
        assert!(fetcher.clear_cache("test/model").is_ok());
        assert!(fetcher.get_cached_model_card("test/model").is_err());
    }
}
