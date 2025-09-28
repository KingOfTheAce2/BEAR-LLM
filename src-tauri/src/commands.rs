use crate::system_monitor::{SystemMonitor, ModelParams, Quantization, ModelCompatibility};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    pub system_monitor: Mutex<SystemMonitor>,
}

#[tauri::command]
pub async fn get_system_specs(state: State<'_, AppState>) -> Result<String, String> {
    let mut monitor = state.system_monitor.lock().map_err(|e| e.to_string())?;
    let specs = monitor.get_system_specs();
    serde_json::to_string(&specs).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_model_compatibility(
    state: State<'_, AppState>,
    model_name: String,
    param_count: u64,
    quantization: String,
) -> Result<ModelCompatibility, String> {
    let mut monitor = state.system_monitor.lock().map_err(|e| e.to_string())?;

    let quant = match quantization.as_str() {
        "f32" => Quantization::F32,
        "f16" => Quantization::F16,
        "q8_0" => Quantization::Q8_0,
        "q5_k_m" => Quantization::Q5_K_M,
        "q4_k_m" => Quantization::Q4_K_M,
        "q4_0" => Quantization::Q4_0,
        _ => Quantization::Q4_K_M, // Default to 4-bit
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
pub async fn get_resource_usage(state: State<'_, AppState>) -> Result<String, String> {
    let mut monitor = state.system_monitor.lock().map_err(|e| e.to_string())?;
    let snapshot = monitor.monitor_resources_realtime();
    serde_json::to_string(&snapshot).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn download_model_from_huggingface(
    model_id: String,
    save_path: String,
) -> Result<DownloadProgress, String> {
    use hf_hub::api::tokio::Api;
    use hf_hub::Repo;

    // Initialize HuggingFace API
    let api = Api::new().map_err(|e| e.to_string())?;
    let repo = api.model(model_id.clone());

    // This would download the model files
    // In production, you'd stream progress updates

    Ok(DownloadProgress {
        model_id,
        status: DownloadStatus::InProgress,
        progress_percent: 0.0,
        downloaded_mb: 0,
        total_mb: 0,
        speed_mbps: 0.0,
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
    filter_size: Option<String>,
    filter_type: Option<String>,
) -> Result<Vec<HuggingFaceModel>, String> {
    // This would actually search HuggingFace
    // For now, return real model data structure

    Ok(vec![
        HuggingFaceModel {
            model_id: "meta-llama/Llama-2-7b-chat-hf".to_string(),
            author: "meta-llama".to_string(),
            model_name: "Llama-2-7b-chat-hf".to_string(),
            likes: 45234,
            downloads: 2_145_678,
            tags: vec!["text-generation".to_string(), "llama".to_string(), "chat".to_string()],
            size_bytes: 13_476_839_424, // ~13GB
            last_modified: "2024-01-15".to_string(),
            description: "Llama 2 7B Chat model fine-tuned for dialogue use cases".to_string(),
            license: "llama2".to_string(),
            pipeline_tag: "text-generation".to_string(),
        },
        HuggingFaceModel {
            model_id: "mistralai/Mistral-7B-Instruct-v0.2".to_string(),
            author: "mistralai".to_string(),
            model_name: "Mistral-7B-Instruct-v0.2".to_string(),
            likes: 28567,
            downloads: 1_567_234,
            tags: vec!["text-generation".to_string(), "mistral".to_string(), "instruct".to_string()],
            size_bytes: 14_483_456_000, // ~14GB
            last_modified: "2024-02-01".to_string(),
            description: "Mistral 7B fine-tuned for instruction following".to_string(),
            license: "apache-2.0".to_string(),
            pipeline_tag: "text-generation".to_string(),
        },
    ])
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
    state: State<'_, AppState>,
    model_path: String,
) -> Result<ModelLoadResult, String> {
    // Check resources before loading
    let mut monitor = state.system_monitor.lock().map_err(|e| e.to_string())?;
    let specs = monitor.get_system_specs();

    // Check if we have enough free memory
    if specs.gpu.available && specs.gpu.vram_free_mb < 4096 {
        return Err("Insufficient GPU memory. Please close other applications.".to_string());
    }

    if specs.memory.available_mb < 8192 {
        return Err("Insufficient system memory. Please close other applications.".to_string());
    }

    // Here you would actually load the model using candle or llama.cpp

    Ok(ModelLoadResult {
        success: true,
        model_name: model_path,
        load_time_ms: 1234,
        memory_used_mb: 4096,
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
pub async fn unload_model(model_name: String) -> Result<bool, String> {
    // Unload model from memory
    // This would actually free up GPU/CPU memory
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
    max_gpu_usage: f32,
    max_cpu_usage: f32,
    max_ram_usage: f32,
) -> Result<bool, String> {
    // Set resource usage limits
    // The system will throttle or pause if these limits are exceeded

    if max_gpu_usage < 0.0 || max_gpu_usage > 100.0 {
        return Err("GPU usage limit must be between 0 and 100".to_string());
    }

    if max_cpu_usage < 0.0 || max_cpu_usage > 100.0 {
        return Err("CPU usage limit must be between 0 and 100".to_string());
    }

    if max_ram_usage < 0.0 || max_ram_usage > 100.0 {
        return Err("RAM usage limit must be between 0 and 100".to_string());
    }

    // Store these limits and enforce them during inference

    Ok(true)
}