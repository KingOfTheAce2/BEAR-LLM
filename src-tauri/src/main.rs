#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use sysinfo::{System, SystemExt, CpuExt};
use tauri::State;
use tokio::sync::RwLock;
use regex::Regex;
use lazy_static::lazy_static;

mod pii_detector;
mod hardware_monitor;
mod llm_manager;
mod file_processor;
mod rag_engine;
mod system_monitor;
mod commands;

use pii_detector::PIIDetector;
use hardware_monitor::HardwareMonitor;
use llm_manager::LLMManager;
use file_processor::FileProcessor;
use rag_engine::RAGEngine;

#[derive(Clone)]
struct AppState {
    pii_detector: Arc<PIIDetector>,
    hardware_monitor: Arc<RwLock<HardwareMonitor>>,
    llm_manager: Arc<RwLock<LLMManager>>,
    file_processor: Arc<FileProcessor>,
    rag_engine: Arc<RwLock<RAGEngine>>,
}

// Add the new AppState for commands
use commands::AppState as CommandState;

#[derive(Debug, Serialize, Deserialize)]
struct SystemStatus {
    cpu_usage: f32,
    memory_usage: f32,
    gpu_usage: Option<f32>,
    temperature: Option<f32>,
    is_safe: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
    timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProcessedDocument {
    id: String,
    filename: String,
    content: String,
    pii_removed: bool,
    metadata: serde_json::Value,
}

#[tauri::command]
async fn check_system_status(state: State<'_, AppState>) -> Result<SystemStatus, String> {
    let monitor = state.hardware_monitor.read().await;
    monitor.get_status().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn process_document(
    state: State<'_, AppState>,
    file_path: String,
    file_type: String,
) -> Result<ProcessedDocument, String> {
    let content = state.file_processor
        .process_file(&file_path, &file_type)
        .await
        .map_err(|e| e.to_string())?;

    let cleaned_content = state.pii_detector
        .remove_pii(&content)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ProcessedDocument {
        id: uuid::Uuid::new_v4().to_string(),
        filename: file_path,
        content: cleaned_content,
        pii_removed: true,
        metadata: serde_json::json!({"type": file_type}),
    })
}

#[tauri::command]
async fn send_message(
    state: State<'_, AppState>,
    message: String,
    model_name: String,
) -> Result<String, String> {
    let mut hw_monitor = state.hardware_monitor.write().await;
    if !hw_monitor.check_safety().await.map_err(|e| e.to_string())? {
        return Err("System resources are critically high. Please wait before sending another message.".to_string());
    }

    let cleaned_message = state.pii_detector
        .remove_pii(&message)
        .await
        .map_err(|e| e.to_string())?;

    let mut llm = state.llm_manager.write().await;
    let response = llm.generate_response(&cleaned_message, &model_name)
        .await
        .map_err(|e| e.to_string())?;

    Ok(response)
}

#[tauri::command]
async fn search_knowledge_base(
    state: State<'_, AppState>,
    query: String,
    limit: usize,
) -> Result<Vec<serde_json::Value>, String> {
    let cleaned_query = state.pii_detector
        .remove_pii(&query)
        .await
        .map_err(|e| e.to_string())?;

    let rag = state.rag_engine.read().await;
    rag.search(&cleaned_query, limit)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_to_knowledge_base(
    state: State<'_, AppState>,
    content: String,
    metadata: serde_json::Value,
) -> Result<String, String> {
    let cleaned_content = state.pii_detector
        .remove_pii(&content)
        .await
        .map_err(|e| e.to_string())?;

    let mut rag = state.rag_engine.write().await;
    rag.add_document(&cleaned_content, metadata)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_available_models(
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let llm = state.llm_manager.read().await;
    Ok(llm.list_models().await)
}

#[tauri::command]
async fn download_model(
    state: State<'_, AppState>,
    model_name: String,
) -> Result<String, String> {
    let mut llm = state.llm_manager.write().await;
    llm.download_model(&model_name)
        .await
        .map_err(|e| e.to_string())?;
    Ok(format!("Model {} downloaded successfully", model_name))
}

fn main() {
    let app_state = AppState {
        pii_detector: Arc::new(PIIDetector::new()),
        hardware_monitor: Arc::new(RwLock::new(HardwareMonitor::new())),
        llm_manager: Arc::new(RwLock::new(LLMManager::new())),
        file_processor: Arc::new(FileProcessor::new()),
        rag_engine: Arc::new(RwLock::new(RAGEngine::new())),
    };

    // Initialize the system monitor state
    let command_state = CommandState {
        system_monitor: std::sync::Mutex::new(system_monitor::SystemMonitor::new()),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_os::init())
        .manage(app_state.clone())
        .manage(command_state)
        .setup(move |app| {
            let state = app_state.clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    let mut monitor = state.hardware_monitor.write().await;
                    if let Err(e) = monitor.update_metrics().await {
                        eprintln!("Failed to update hardware metrics: {}", e);
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            check_system_status,
            process_document,
            send_message,
            search_knowledge_base,
            add_to_knowledge_base,
            list_available_models,
            download_model,
            commands::get_system_specs,
            commands::check_model_compatibility,
            commands::get_resource_usage,
            commands::download_model_from_huggingface,
            commands::search_huggingface_models,
            commands::load_model,
            commands::unload_model,
            commands::emergency_stop,
            commands::set_resource_limits,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}