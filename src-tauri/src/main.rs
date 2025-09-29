#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
// Remove: use sysinfo::System; (unused)
use tauri::State;
use tokio::sync::RwLock;
// Remove: use regex::Regex; (unused)
// Remove: use lazy_static::lazy_static; (unused)

mod pii_detector;
mod hardware_monitor;
mod llm_manager;
mod file_processor;
mod rag_engine;
mod system_monitor;
mod commands;
mod database;
mod mcp_server;
mod hardware_detector;
mod inference_engine;

use pii_detector::PIIDetector;
use hardware_monitor::HardwareMonitor;
use llm_manager::LLMManager;
use file_processor::FileProcessor;
use rag_engine::RAGEngine;
use database::DatabaseManager;
use mcp_server::{MCPServer, AgentOrchestrator};
use hardware_detector::{HardwareDetector, HardwareSpecs, ModelRecommendation};

#[derive(Clone)]
struct AppState {
    pii_detector: Arc<PIIDetector>,
    hardware_monitor: Arc<RwLock<HardwareMonitor>>,
    llm_manager: Arc<RwLock<LLMManager>>,
    file_processor: Arc<FileProcessor>,
    rag_engine: Arc<RwLock<RAGEngine>>,
    database_manager: Arc<RwLock<DatabaseManager>>,
    #[allow(dead_code)]
    mcp_server: Arc<MCPServer>,
    #[allow(dead_code)]
    agent_orchestrator: Arc<AgentOrchestrator>,
    hardware_detector: Arc<RwLock<HardwareDetector>>,
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
#[allow(dead_code)]
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
async fn detect_hardware(
    state: State<'_, AppState>
) -> Result<HardwareSpecs, String> {
    let mut detector = state.hardware_detector.write().await;
    detector.detect_hardware().map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_model_recommendations(
    state: State<'_, AppState>
) -> Result<Vec<ModelRecommendation>, String> {
    let mut detector = state.hardware_detector.write().await;
    let hardware = detector.detect_hardware().map_err(|e| e.to_string())?;
    Ok(detector.recommend_models(&hardware))
}

#[tauri::command]
async fn get_system_summary(
    state: State<'_, AppState>
) -> Result<String, String> {
    let mut detector = state.hardware_detector.write().await;
    let hardware = detector.detect_hardware().map_err(|e| e.to_string())?;
    Ok(detector.get_system_summary(&hardware))
}

#[tauri::command]
async fn estimate_model_performance(
    state: State<'_, AppState>,
    model_size_gb: f64
) -> Result<String, String> {
    let mut detector = state.hardware_detector.write().await;
    let hardware = detector.detect_hardware().map_err(|e| e.to_string())?;
    Ok(detector.estimate_model_performance(&hardware, model_size_gb))
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

// New database commands
#[tauri::command]
async fn execute_sql_query(
    state: State<'_, AppState>,
    query: String,
) -> Result<serde_json::Value, String> {
    let db = state.database_manager.read().await;
    db.execute_sql_query(&query).map_err(|e| e.to_string())
}

#[tauri::command]
async fn rag_search(
    state: State<'_, AppState>,
    query: String,
    use_agentic: bool,
    max_results: usize,
) -> Result<serde_json::Value, String> {
    let cleaned_query = state.pii_detector
        .remove_pii(&query)
        .await
        .map_err(|e| e.to_string())?;

    let rag = state.rag_engine.read().await;

    let results = if use_agentic {
        rag.agentic_search(&cleaned_query, "")
            .await
            .map_err(|e| e.to_string())?
    } else {
        rag.search(&cleaned_query, max_results)
            .await
            .map_err(|e| e.to_string())?
    };

    // Calculate confidence and create structured response
    let confidence = if results.len() > 0 { 0.85 } else { 0.0 };

    Ok(serde_json::json!({
        "answer": format!("Found {} relevant documents for your query.", results.len()),
        "sources": results.iter().map(|r| serde_json::json!({
            "title": r.get("title").unwrap_or(&serde_json::Value::String("Document".to_string())),
            "snippet": r.get("content").unwrap_or(&serde_json::Value::String("No content".to_string())),
            "relevance": 0.8,
            "source": "Knowledge Base"
        })).collect::<Vec<_>>(),
        "reasoning": if use_agentic { Some("Used advanced reasoning and query rewriting for enhanced accuracy") } else { None },
        "confidence": confidence
    }))
}

#[tauri::command]
async fn upload_document(
    state: State<'_, AppState>,
    filename: String,
    content: Vec<u8>,
) -> Result<serde_json::Value, String> {
    // Convert bytes to string (simplified)
    let content_str = String::from_utf8_lossy(&content);

    // Process file with PII detection
    let cleaned_content = state.pii_detector
        .remove_pii(&content_str)
        .await
        .map_err(|e| e.to_string())?;

    // Store in database
    let db = state.database_manager.read().await;
    let file_type = filename.split('.').last().unwrap_or("txt");
    let doc_id = db.store_document(&filename, &cleaned_content, file_type)
        .map_err(|e| e.to_string())?;

    // Add to RAG engine
    let mut rag = state.rag_engine.write().await;
    rag.add_document(&cleaned_content, serde_json::json!({
        "filename": filename,
        "document_id": doc_id
    })).await.map_err(|e| e.to_string())?;

    // Estimate chunks (simplified)
    let chunk_count = (cleaned_content.len() / 512).max(1);

    Ok(serde_json::json!({
        "chunks": chunk_count,
        "document_id": doc_id
    }))
}

#[tauri::command]
async fn analyze_document_pii(
    state: State<'_, AppState>,
    filename: String,
    content: Vec<u8>,
) -> Result<serde_json::Value, String> {
    let start_time = std::time::Instant::now();

    // Process file first
    let file_type = filename.split('.').last().unwrap_or("unknown");
    let original_text = if state.file_processor.is_supported(file_type) {
        // Create temporary file for processing
        let temp_path = format!("/tmp/{}", filename);
        std::fs::write(&temp_path, &content).map_err(|e| e.to_string())?;

        state.file_processor
            .process_file(&temp_path, file_type)
            .await
            .unwrap_or_else(|_| String::from_utf8_lossy(&content).to_string())
    } else {
        return Ok(serde_json::json!({
            "filename": filename,
            "fileType": file_type,
            "originalText": "",
            "cleanedText": "",
            "piiDetections": [],
            "processingTime": 0,
            "supported": false,
            "error": format!("Unsupported file type: {}", file_type)
        }));
    };

    // Detect PII
    let detections = state.pii_detector
        .detect_pii(&original_text)
        .await
        .map_err(|e| e.to_string())?;

    // Clean the text
    let cleaned_text = state.pii_detector
        .remove_pii(&original_text)
        .await
        .map_err(|e| e.to_string())?;

    let processing_time = start_time.elapsed().as_millis();

    Ok(serde_json::json!({
        "filename": filename,
        "fileType": file_type,
        "originalText": original_text,
        "cleanedText": cleaned_text,
        "piiDetections": detections.iter().map(|d| serde_json::json!({
            "type": d.pii_type,
            "text": d.text,
            "startIndex": d.start,
            "endIndex": d.end,
            "confidence": 0.95, // Default confidence
            "replacement": format!("[REDACTED_{}]", d.pii_type.to_uppercase())
        })).collect::<Vec<_>>(),
        "processingTime": processing_time,
        "supported": true
    }))
}

#[tauri::command]
async fn get_database_stats(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let db = state.database_manager.read().await;
    db.get_document_statistics().map_err(|e| e.to_string())
}

fn main() {
    let database_manager = Arc::new(RwLock::new(
        DatabaseManager::new().expect("Failed to initialize database")
    ));

    let app_state = AppState {
        pii_detector: Arc::new(PIIDetector::new()),
        hardware_monitor: Arc::new(RwLock::new(HardwareMonitor::new())),
        llm_manager: Arc::new(RwLock::new(LLMManager::new())),
        file_processor: Arc::new(FileProcessor::new()),
        rag_engine: Arc::new(RwLock::new(RAGEngine::new())),
        database_manager,
        mcp_server: Arc::new(MCPServer::new(true)), // sandboxed mode
        agent_orchestrator: Arc::new(AgentOrchestrator::new(true)),
        hardware_detector: Arc::new(RwLock::new(HardwareDetector::new())),
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
        .setup(move |_app| {
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
            // New database and RAG commands
            execute_sql_query,
            rag_search,
            upload_document,
            analyze_document_pii,
            get_database_stats,
            // Hardware detection commands
            detect_hardware,
            get_model_recommendations,
            get_system_summary,
            estimate_model_performance,
            // Existing commands
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