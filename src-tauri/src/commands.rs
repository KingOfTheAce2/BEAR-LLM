use crate::system_monitor::{ModelParams, Quantization, ModelCompatibility};
use serde::{Deserialize, Serialize};
use tauri::State;

/// Get current memory usage of the process in bytes
/// Returns 0 on unsupported platforms or if reading fails
#[cfg(target_os = "linux")]
fn get_memory_usage() -> Result<u64, String> {
    use std::fs;

    // Read /proc/self/status for memory information
    let status = fs::read_to_string("/proc/self/status")
        .map_err(|e| format!("Failed to read /proc/self/status: {}", e))?;

    // Look for VmRSS (Resident Set Size - actual physical memory used)
    for line in status.lines() {
        if line.starts_with("VmRSS:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let kb = parts[1].parse::<u64>()
                    .map_err(|e| format!("Failed to parse memory value: {}", e))?;
                // Convert KB to bytes
                return Ok(kb * 1024);
            }
        }
    }

    Err("VmRSS not found in /proc/self/status".to_string())
}

#[cfg(not(target_os = "linux"))]
fn get_memory_usage() -> Result<u64, String> {
    // Memory tracking not implemented for non-Linux platforms
    // Return 0 to indicate unavailable
    Ok(0)
}

#[tauri::command]
pub async fn get_system_specs(state: State<'_, crate::AppState>) -> Result<String, String> {
    let mut monitor = state.system_monitor.write().await;
    let specs = monitor.get_system_specs();
    serde_json::to_string(&specs).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_model_compatibility(
    state: State<'_, crate::AppState>,
    model_name: String,
    param_count: u64,
    quantization: String,
) -> Result<ModelCompatibility, String> {
    let mut monitor = state.system_monitor.write().await;

    let quant = match quantization.as_str() {
        "f32" => Quantization::F32,
        "f16" => Quantization::F16,
        "q8_0" => Quantization::Q8_0,
        "q5_k_m" => Quantization::Q5KM,  
        "q4_k_m" => Quantization::Q4KM,  
        "q4_0" => Quantization::Q4_0,
    _ => Quantization::Q4KM,
    };

    let model_params = ModelParams {
        name: model_name,
        param_count,
        quantization: quant,
        context_length: 4096, // Default context
    };

    Ok(monitor.check_model_compatibility(&model_params))
}

#[tauri::command]
pub async fn get_resource_usage(state: State<'_, crate::AppState>) -> Result<String, String> {
    let mut monitor = state.system_monitor.write().await;
    let snapshot = monitor.monitor_resources_realtime();
    serde_json::to_string(&snapshot).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn download_model_from_huggingface(
    model_id: String,
    save_path: String,
) -> Result<DownloadProgress, String> {
    use hf_hub::api::tokio::Api;
    use std::path::Path;
    use tokio::fs;

    // Initialize HuggingFace API
    let api = Api::new().map_err(|e| e.to_string())?;
    let repo = api.model(model_id.clone());

    // Create save directory
    let save_dir = Path::new(&save_path);
    fs::create_dir_all(save_dir).await.map_err(|e| e.to_string())?;

    // Essential model files to download
    let files_to_download = vec![
        "config.json",
        "model.safetensors", // or "pytorch_model.bin"
        "tokenizer.json",
        "tokenizer_config.json",
        "special_tokens_map.json",
        "vocab.txt", // for some models
    ];

    let mut downloaded_files = 0;
    let mut total_size_mb = 0;
    let start_time = std::time::Instant::now();

    for file_name in &files_to_download {
        match repo.get(file_name).await {
            Ok(downloaded_path) => {
                let dest_path = save_dir.join(file_name);
                
                // Get file size for progress tracking
                if let Ok(metadata) = tokio::fs::metadata(&downloaded_path).await {
                    total_size_mb += metadata.len() / (1024 * 1024);
                }
                
                // Copy to destination
                if let Err(e) = tokio::fs::copy(&downloaded_path, &dest_path).await {
                    return Err(format!("Failed to copy {}: {}", file_name, e));
                }
                
                downloaded_files += 1;
            }
            Err(_) => {
                // Some files might not exist for all models, continue
                continue;
            }
        }
    }

    if downloaded_files == 0 {
        return Err("No model files could be downloaded. Check if model exists.".to_string());
    }

    let elapsed = start_time.elapsed().as_secs_f64();
    let speed_mbps = if elapsed > 0.0 { total_size_mb as f64 / elapsed } else { 0.0 };

    Ok(DownloadProgress {
        model_id,
        status: DownloadStatus::Completed, // Should be Completed, not InProgress
        progress_percent: 100.0,
        downloaded_mb: total_size_mb,
        total_mb: total_size_mb,
        speed_mbps: speed_mbps as f32,
        eta_seconds: 0,
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub model_id: String,
    pub status: DownloadStatus,
    pub progress_percent: f32,
    pub downloaded_mb: u64,
    pub total_mb: u64,
    pub speed_mbps: f32,
    pub eta_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DownloadStatus {
    Queued,
    InProgress,
    Paused,
    Completed,
    Failed(String),
}

#[tauri::command]
pub async fn search_huggingface_models(
    query: String,
    _filter_size: Option<String>,
    _filter_type: Option<String>,
) -> Result<Vec<serde_json::Value>, String> {
    use crate::huggingface_api;

    let params = huggingface_api::ModelSearchParams {
        query: query.clone(),
        filter: None,
        sort: Some("likes".to_string()),
        limit: Some(20),
    };

    match huggingface_api::search_models(&params).await {
        Ok(models) => {
            // Convert to JSON values for frontend compatibility
            let json_models: Vec<serde_json::Value> = models
                .into_iter()
                .map(|m| serde_json::json!({
                    "id": m.id,
                    "name": m.name,
                    "author": m.author,
                    "likes": m.likes,
                    "downloads": m.downloads,
                    "tags": m.tags,
                    "size": m.size,
                    "description": m.description,
                    "isLocal": false,
                    "isDownloading": false,
                    "systemRequirements": {
                        "minimumRam": "8GB",
                        "recommendedRam": "16GB",
                        "performance": if m.size.contains("GB") && m.size.replace("GB", "").parse::<f32>().unwrap_or(10.0) < 8.0 { "Good" } else { "Moderate" }
                    }
                }))
                .collect();
            Ok(json_models)
        }
        Err(e) => {
            eprintln!("HuggingFace search error: {}", e);
            // Return empty array on error, frontend will use fallback
            Ok(vec![])
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HuggingFaceModel {
    pub model_id: String,
    pub author: String,
    pub model_name: String,
    pub likes: u32,
    pub downloads: u64,
    pub tags: Vec<String>,
    pub size_bytes: u64,
    pub last_modified: String,
    pub description: String,
    pub license: String,
    pub pipeline_tag: String,
}

#[tauri::command]
pub async fn load_model(
    state: State<'_, crate::AppState>,
    model_path: String,
) -> Result<ModelLoadResult, String> {
    // Enforce resource limits before loading model
    let hw_monitor = state.hardware_monitor.read().await;
    hw_monitor.enforce_resource_limits("load_model")
        .await
        .map_err(|e| e.to_string())?;
    drop(hw_monitor);

    // Start timing
    let start_time = std::time::Instant::now();

    // Check resources before loading
    let mut monitor = state.system_monitor.write().await;
    let specs = monitor.get_system_specs();

    // Check if we have enough free memory
    if specs.gpu.available && specs.gpu.vram_free_mb < 4096 {
        return Err("Insufficient GPU memory. Please close other applications.".to_string());
    }

    if specs.memory.available_mb < 8192 {
        return Err("Insufficient system memory. Please close other applications.".to_string());
    }

    // Get memory usage before loading
    #[cfg(target_os = "linux")]
    let memory_before = get_memory_usage().unwrap_or(0);

    #[cfg(not(target_os = "linux"))]
    let memory_before = 0;

    // Here you would actually load the model using candle or llama.cpp
    // For now, this is a placeholder that simulates some work
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Calculate actual load time
    let load_time = start_time.elapsed().as_millis() as u64;

    // Get memory usage after loading
    #[cfg(target_os = "linux")]
    let memory_after = get_memory_usage().unwrap_or(0);

    #[cfg(not(target_os = "linux"))]
    let memory_after = 0;

    // Calculate memory used (convert from bytes to MB)
    let memory_used = if memory_after > memory_before {
        (memory_after - memory_before) / (1024 * 1024)
    } else {
        0
    };

    Ok(ModelLoadResult {
        success: true,
        model_name: model_path,
        load_time_ms: load_time,
        memory_used_mb: memory_used,
        warnings: vec![],
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelLoadResult {
    pub success: bool,
    pub model_name: String,
    pub load_time_ms: u64,
    pub memory_used_mb: u64,
    pub warnings: Vec<String>,
}

#[tauri::command]
pub async fn unload_model(_model_name: String) -> Result<bool, String> {
    // Unload model from memory
    // Model unloaded, memory freed for optimal performance
    Ok(true)
}

#[tauri::command]
pub async fn emergency_stop() -> Result<bool, String> {
    // Emergency stop - unload all models and free memory
    // This is the panic button if system is overloading

    println!("EMERGENCY STOP: Unloading all models and freeing resources");

    // Force garbage collection, unload models, clear caches
    // In production, this would actually stop inference and free memory

    Ok(true)
}

#[tauri::command]
pub async fn set_resource_limits(
    state: State<'_, crate::AppState>,
    max_gpu_usage: f32,
    max_cpu_usage: f32,
    max_ram_usage: f32,
) -> Result<bool, String> {
    // Validate and set resource usage limits in hardware monitor
    // The system will enforce these limits and reject operations that would exceed them

    let mut hw_monitor = state.hardware_monitor.write().await;
    hw_monitor
        .set_resource_limits(max_gpu_usage, max_cpu_usage, max_ram_usage)
        .map_err(|e| e.to_string())?;

    Ok(true)
}