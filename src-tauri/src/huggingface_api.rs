use crate::utils::{estimate_model_size_mb, parse_model_params_from_id};
use anyhow::{anyhow, Result};
use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HuggingFaceModel {
    pub id: String,
    pub author: String,
    pub name: String,
    pub likes: u32,
    pub downloads: u64,
    pub tags: Vec<String>,
    pub size: String,
    pub last_modified: String,
    pub description: Option<String>,
    pub license: Option<String>,
    pub pipeline_tag: Option<String>,
    pub is_local: bool,
    pub is_downloading: bool,
}

#[derive(Debug, Deserialize)]
struct HFApiModel {
    id: String,
    author: Option<String>,
    likes: Option<u32>,
    downloads: Option<u32>,
    tags: Option<Vec<String>>,
    #[serde(rename = "lastModified")]
    last_modified: Option<String>,
    private: Option<bool>,
    pipeline_tag: Option<String>,
    #[allow(dead_code)]
    library_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelSearchParams {
    pub query: String,
    pub filter: Option<String>,
    pub sort: Option<String>,
    pub limit: Option<usize>,
}

pub async fn search_models(params: &ModelSearchParams) -> Result<Vec<HuggingFaceModel>> {
    // Try to use the real HuggingFace API
    match search_hf_api(params).await {
        Ok(models) => Ok(models),
        Err(_) => {
            // Fallback to curated list if API fails
            Ok(get_curated_models(&params.query))
        }
    }
}

async fn search_hf_api(params: &ModelSearchParams) -> Result<Vec<HuggingFaceModel>> {
    let client = reqwest::Client::new();

    let mut url = format!(
        "https://huggingface.co/api/models?search={}&limit={}",
        urlencoding::encode(&params.query),
        params.limit.unwrap_or(20)
    );

    // Add filters for text generation models
    url.push_str("&filter=text-generation");

    if let Some(sort) = &params.sort {
        url.push_str(&format!("&sort={}", sort));
    }

    let response = client
        .get(&url)
        .header("User-Agent", "BEAR-AI-LLM/1.0")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "HuggingFace API returned error: {}",
            response.status()
        ));
    }

    let hf_models: Vec<HFApiModel> = response.json().await?;

    let models = hf_models
        .into_iter()
        .filter(|m| !m.private.unwrap_or(false))
        .map(|m| {
            let author = m
                .author
                .unwrap_or_else(|| m.id.split('/').next().unwrap_or("unknown").to_string());
            let name = m.id.split('/').last().unwrap_or(&m.id).to_string();

            HuggingFaceModel {
                id: m.id.clone(),
                author,
                name,
                likes: m.likes.unwrap_or(0),
                downloads: m.downloads.unwrap_or(0) as u64,
                tags: m.tags.unwrap_or_default(),
                size: estimate_model_size(&m.id),
                last_modified: m.last_modified.unwrap_or_else(|| "Unknown".to_string()),
                description: Some(format!("Model: {}", m.id)),
                license: None,
                pipeline_tag: m.pipeline_tag,
                is_local: false,
                is_downloading: false,
            }
        })
        .collect();

    Ok(models)
}

/// Estimate model size based on model ID
///
/// Uses improved heuristics to determine model size based on:
/// 1. Parameter count extracted from model ID
/// 2. Assumed quantization (Q4_K_M for GGUF models)
///
/// This is more accurate than hardcoded sizes and handles various model sizes.
fn estimate_model_size(model_id: &str) -> String {
    // Try to parse parameter count from model ID
    if let Some(params_billions) = parse_model_params_from_id(model_id) {
        // Check if this is a GGUF model (commonly Q4_K_M quantization)
        let quantization = if model_id.to_lowercase().contains("gguf") {
            // Check for quantization in model ID
            if model_id.contains("Q2") {
                "Q2_K"
            } else if model_id.contains("Q8") {
                "Q8_0"
            } else if model_id.contains("Q6") {
                "Q6_K"
            } else if model_id.contains("Q5") {
                "Q5_K_M"
            } else {
                "Q4_K_M" // Most common
            }
        } else {
            // Assume FP16 for non-GGUF models
            "FP16"
        };

        // Calculate size
        let size_mb = estimate_model_size_mb(params_billions, quantization);
        let size_gb = (size_mb as f32 / 1024.0).round() as u32;

        format!("{}GB", size_gb)
    } else {
        // Fallback for unknown models
        tracing::warn!("Could not estimate size for model: {}", model_id);
        "Unknown".to_string()
    }
}

fn get_curated_models(query: &str) -> Vec<HuggingFaceModel> {
    let all_models = vec![
        HuggingFaceModel {
            id: "meta-llama/Llama-2-7b-chat-hf".to_string(),
            author: "meta-llama".to_string(),
            name: "Llama-2-7b-chat".to_string(),
            likes: 45234,
            downloads: 2145678,
            tags: vec![
                "text-generation".to_string(),
                "llama".to_string(),
                "chat".to_string(),
            ],
            size: "13GB".to_string(),
            last_modified: "2024-01-15".to_string(),
            description: Some("Llama 2 7B Chat model for dialogue".to_string()),
            license: Some("llama2".to_string()),
            pipeline_tag: Some("text-generation".to_string()),
            is_local: false,
            is_downloading: false,
        },
        HuggingFaceModel {
            id: "mistralai/Mistral-7B-Instruct-v0.2".to_string(),
            author: "mistralai".to_string(),
            name: "Mistral-7B-Instruct".to_string(),
            likes: 28567,
            downloads: 1567234,
            tags: vec!["text-generation".to_string(), "mistral".to_string()],
            size: "14GB".to_string(),
            last_modified: "2024-02-01".to_string(),
            description: Some("Mistral 7B instruction model".to_string()),
            license: Some("apache-2.0".to_string()),
            pipeline_tag: Some("text-generation".to_string()),
            is_local: false,
            is_downloading: false,
        },
        HuggingFaceModel {
            id: "microsoft/phi-2".to_string(),
            author: "microsoft".to_string(),
            name: "Phi-2".to_string(),
            likes: 15432,
            downloads: 987654,
            tags: vec!["text-generation".to_string(), "phi".to_string()],
            size: "5GB".to_string(),
            last_modified: "2024-01-20".to_string(),
            description: Some("Small but capable language model".to_string()),
            license: Some("mit".to_string()),
            pipeline_tag: Some("text-generation".to_string()),
            is_local: false,
            is_downloading: false,
        },
        HuggingFaceModel {
            id: "google/gemma-2b".to_string(),
            author: "google".to_string(),
            name: "Gemma-2B".to_string(),
            likes: 12789,
            downloads: 765432,
            tags: vec!["text-generation".to_string(), "gemma".to_string()],
            size: "4GB".to_string(),
            last_modified: "2024-02-10".to_string(),
            description: Some("Google's efficient 2B parameter model".to_string()),
            license: Some("gemma".to_string()),
            pipeline_tag: Some("text-generation".to_string()),
            is_local: false,
            is_downloading: false,
        },
    ];

    // Filter based on query
    let query_lower = query.to_lowercase();
    all_models
        .into_iter()
        .filter(|m| {
            query.is_empty()
                || m.name.to_lowercase().contains(&query_lower)
                || m.author.to_lowercase().contains(&query_lower)
                || m.tags
                    .iter()
                    .any(|t| t.to_lowercase().contains(&query_lower))
        })
        .collect()
}

#[allow(dead_code)]
pub async fn get_model_info(model_id: &str) -> Result<serde_json::Value> {
    let client = reqwest::Client::new();
    let url = format!("https://huggingface.co/api/models/{}", model_id);

    let response = client
        .get(&url)
        .header("User-Agent", "BEAR-AI-LLM/1.0")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow!("Failed to get model info: {}", response.status()));
    }

    let model_info: serde_json::Value = response.json().await?;
    Ok(model_info)
}

#[allow(dead_code)]
pub async fn download_model_with_progress<F>(
    model_id: &str,
    save_path: &str,
    progress_callback: F,
) -> Result<()>
where
    F: Fn(f32) + Send + Sync,
{
    use hf_hub::api::tokio::Api;
    use std::path::Path;
    use tokio::fs;

    // Initialize HuggingFace API
    let api = Api::new()?;
    let repo = api.model(model_id.to_string());

    // Create save directory
    let save_dir = Path::new(save_path);
    fs::create_dir_all(save_dir).await?;

    // Essential model files to download
    let files_to_download = vec![
        "config.json",
        "model.safetensors",
        "pytorch_model.bin",
        "tokenizer.json",
        "tokenizer_config.json",
        "special_tokens_map.json",
        "vocab.txt",
    ];

    let total_files = files_to_download.len();
    let mut downloaded_files = 0;

    for file_name in &files_to_download {
        match repo.get(file_name).await {
            Ok(downloaded_path) => {
                let dest_path = save_dir.join(file_name);

                // Copy to destination
                if let Ok(_) = tokio::fs::copy(&downloaded_path, &dest_path).await {
                    downloaded_files += 1;
                    let progress = (downloaded_files as f32 / total_files as f32) * 100.0;
                    progress_callback(progress);
                }
            }
            Err(_) => {
                // Some files might not exist for all models, continue
                continue;
            }
        }
    }

    if downloaded_files == 0 {
        return Err(anyhow!("No model files could be downloaded"));
    }

    Ok(())
}
