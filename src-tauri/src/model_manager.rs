use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalModel {
    pub id: String,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    pub metadata: serde_json::Value,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub model_id: String,
    pub progress_percent: f32,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub speed_mbps: f32,
    pub eta_seconds: u32,
    pub status: String,
}

#[allow(dead_code)]
pub struct ModelManager {
    models_dir: PathBuf,
    local_models: Arc<RwLock<HashMap<String, LocalModel>>>,
    downloads: Arc<RwLock<HashMap<String, DownloadProgress>>>,
}

#[allow(dead_code)]
impl ModelManager {
    pub fn new() -> Result<Self> {
        let models_dir = dirs::data_local_dir()
            .ok_or_else(|| anyhow!("Failed to get local data directory"))?
            .join("bear-ai-llm")
            .join("models");

        // Create models directory if it doesn't exist
        std::fs::create_dir_all(&models_dir)?;

        let mut manager = Self {
            models_dir,
            local_models: Arc::new(RwLock::new(HashMap::new())),
            downloads: Arc::new(RwLock::new(HashMap::new())),
        };

        // Scan for existing models
        manager.scan_local_models()?;

        Ok(manager)
    }

    pub fn scan_local_models(&mut self) -> Result<()> {
        let models = self.local_models.clone();
        let models_dir = self.models_dir.clone();

        tokio::spawn(async move {
            let mut models_map = models.write().await;
            models_map.clear();

            if let Ok(entries) = std::fs::read_dir(&models_dir) {
                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_dir() {
                            let model_id = entry.file_name().to_string_lossy().to_string();
                            let model_path = entry.path();

                            // Check if model files exist
                            let config_path = model_path.join("config.json");
                            if config_path.exists() {
                                let size = get_directory_size(&model_path).unwrap_or(0);

                                let local_model = LocalModel {
                                    id: model_id.clone(),
                                    path: model_path,
                                    size_bytes: size,
                                    last_accessed: chrono::Utc::now(),
                                    metadata: serde_json::json!({}),
                                };

                                models_map.insert(model_id, local_model);
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn get_local_models(&self) -> Vec<LocalModel> {
        let models = self.local_models.read().await;
        models.values().cloned().collect()
    }

    pub async fn is_model_local(&self, model_id: &str) -> bool {
        let models = self.local_models.read().await;
        models.contains_key(model_id)
    }

    pub async fn download_model(&self, model_id: &str) -> Result<()> {
        // Check if already downloading
        {
            let downloads = self.downloads.read().await;
            if downloads.contains_key(model_id) {
                return Err(anyhow!("Model is already downloading"));
            }
        }

        // Create model directory
        let model_dir = self.models_dir.join(model_id.replace('/', "_"));
        std::fs::create_dir_all(&model_dir)?;

        // Initialize download progress
        {
            let mut downloads = self.downloads.write().await;
            downloads.insert(
                model_id.to_string(),
                DownloadProgress {
                    model_id: model_id.to_string(),
                    progress_percent: 0.0,
                    downloaded_bytes: 0,
                    total_bytes: 0,
                    speed_mbps: 0.0,
                    eta_seconds: 0,
                    status: "Starting".to_string(),
                },
            );
        }

        let downloads = self.downloads.clone();
        let model_id_clone = model_id.to_string();
        let model_dir_str = model_dir.to_string_lossy().to_string();

        // Start async download
        tokio::spawn(async move {
            let result = crate::huggingface_api::download_model_with_progress(
                &model_id_clone,
                &model_dir_str,
                |progress| {
                    let downloads = downloads.clone();
                    let model_id = model_id_clone.clone();

                    tokio::spawn(async move {
                        let mut downloads_map = downloads.write().await;
                        if let Some(progress_entry) = downloads_map.get_mut(&model_id) {
                            progress_entry.progress_percent = progress;
                            progress_entry.status = if progress >= 100.0 {
                                "Completed".to_string()
                            } else {
                                "Downloading".to_string()
                            };
                        }
                    });
                },
            )
            .await;

            // Update final status
            let mut downloads_map = downloads.write().await;
            match result {
                Ok(_) => {
                    if let Some(progress) = downloads_map.get_mut(&model_id_clone) {
                        progress.status = "Completed".to_string();
                        progress.progress_percent = 100.0;
                    }
                }
                Err(e) => {
                    if let Some(progress) = downloads_map.get_mut(&model_id_clone) {
                        progress.status = format!("Failed: {}", e);
                    }
                }
            }

            // Remove from active downloads after a delay
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            downloads_map.remove(&model_id_clone);
        });

        Ok(())
    }

    pub async fn get_download_progress(&self, model_id: &str) -> Option<DownloadProgress> {
        let downloads = self.downloads.read().await;
        downloads.get(model_id).cloned()
    }

    pub async fn get_all_downloads(&self) -> Vec<DownloadProgress> {
        let downloads = self.downloads.read().await;
        downloads.values().cloned().collect()
    }

    pub async fn load_model(&self, model_id: &str) -> Result<()> {
        let models = self.local_models.read().await;

        if let Some(_model) = models.get(model_id) {
            // Update last accessed time
            drop(models);
            let mut models = self.local_models.write().await;
            if let Some(model) = models.get_mut(model_id) {
                model.last_accessed = chrono::Utc::now();
            }

            // Here you would actually load the model into memory
            // Verify model file exists
            Ok(())
        } else {
            Err(anyhow!("Model not found locally"))
        }
    }

    pub async fn delete_model(&self, model_id: &str) -> Result<()> {
        let model_dir = self.models_dir.join(model_id.replace('/', "_"));

        if model_dir.exists() {
            std::fs::remove_dir_all(&model_dir)?;

            let mut models = self.local_models.write().await;
            models.remove(model_id);

            Ok(())
        } else {
            Err(anyhow!("Model not found"))
        }
    }

    pub fn get_models_directory(&self) -> &Path {
        &self.models_dir
    }
}

#[allow(dead_code)]
fn get_directory_size(path: &Path) -> Result<u64> {
    let mut size = 0;

    for entry in walkdir::WalkDir::new(path) {
        if let Ok(entry) = entry {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    size += metadata.len();
                }
            }
        }
    }

    Ok(size)
}

// Global model manager instance
lazy_static::lazy_static! {
    pub static ref MODEL_MANAGER: Arc<RwLock<Option<ModelManager>>> = Arc::new(RwLock::new(None));
}

#[allow(dead_code)]
pub async fn init_model_manager() -> Result<()> {
    let manager = ModelManager::new()?;
    let mut global = MODEL_MANAGER.write().await;
    *global = Some(manager);
    Ok(())
}

#[allow(dead_code)]
pub async fn get_model_manager() -> Result<Arc<RwLock<Option<ModelManager>>>> {
    Ok(MODEL_MANAGER.clone())
}
