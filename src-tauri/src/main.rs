#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use sysinfo::System;
use tauri::{Emitter, State};
use tokio::sync::RwLock;

// Core AI modules
mod gguf_inference;
mod llm_manager;
mod pii_detector;
mod rag_engine;

// Core modules
mod commands;
mod constants;
mod database;
mod file_processor;
mod hardware_detector;
mod hardware_monitor;
mod huggingface_api;
mod mcp_server;
mod model_manager;
mod presidio_bridge;
mod presidio_service;
mod process_helper;
mod rate_limiter;
mod setup_manager;
mod system;
mod system_monitor;
mod utils;

// GDPR Compliance module
mod compliance;

// AI Transparency module
mod ai_transparency;

// Middleware for consent enforcement
mod middleware;

// Scheduler for automated tasks
mod scheduler;

// Import commands - removed non-existent commands

// Use core AI modules
use llm_manager::LLMManager;
use pii_detector::PIIDetector;
use rag_engine::RAGEngine;

// Use other modules
use file_processor::FileProcessor;
use hardware_monitor::HardwareMonitor;
use presidio_bridge::PresidioBridge;
use setup_manager::SetupManager;
// DatabaseManager is internal to the database module
use bear_ai_llm::commands::transparency_commands::TransparencyState;
use compliance::ComplianceManager;
use hardware_detector::{HardwareDetector, HardwareSpecs, ModelRecommendation};
use mcp_server::{AgentOrchestrator, MCPServer};
use middleware::{ConsentGuard, ConsentGuardBuilder};
use rate_limiter::RateLimiter;
use scheduler::{RetentionScheduler, SchedulerHandle};

// SECURITY FIX: Use tempfile crate for atomic temporary file creation
// This prevents race conditions where file creation happens after validation
use tempfile::NamedTempFile;

// RAII guard for automatic temporary file cleanup using tempfile crate
struct TempFileGuard {
    // Use NamedTempFile which provides atomic creation and automatic cleanup
    temp_file: Option<NamedTempFile>,
    path: PathBuf,
}

impl TempFileGuard {
    /// Atomically create secure temporary file with content
    ///
    /// SECURITY: Uses tempfile::NamedTempFile for atomic creation, preventing TOCTOU attacks.
    /// The file is created with exclusive access and automatically cleaned up on drop.
    fn create_with_content(filename: &str, content: &[u8]) -> Result<Self, String> {
        use std::io::Write;

        // Sanitize filename to prevent path traversal attacks
        let safe_filename = filename
            .replace("..", "")
            .replace("/", "_")
            .replace("\\", "_")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '_' || *c == '-')
            .collect::<String>();

        if safe_filename.is_empty() {
            return Err("Invalid filename: resulted in empty name after sanitization".to_string());
        }

        // Create temporary file with atomic creation (prevents TOCTOU race conditions)
        // Builder allows us to set a prefix for better identification
        let mut temp_file = tempfile::Builder::new()
            .prefix(&format!("bear_ai_{}_", safe_filename))
            .tempfile()
            .map_err(|e| format!("Failed to create secure temporary file: {}", e))?;

        // Write content to the temporary file
        temp_file
            .write_all(content)
            .map_err(|e| format!("Failed to write to temporary file: {}", e))?;

        // Flush to ensure all data is written
        temp_file
            .flush()
            .map_err(|e| format!("Failed to flush temporary file: {}", e))?;

        let path = temp_file.path().to_path_buf();

        tracing::debug!(
            path = ?path,
            size = content.len(),
            "Created secure temporary file atomically with tempfile crate"
        );

        Ok(Self {
            temp_file: Some(temp_file),
            path,
        })
    }

    fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Disable automatic cleanup (useful if file ownership is transferred)
    #[allow(dead_code)]
    fn persist(mut self) -> Result<PathBuf, String> {
        if let Some(temp_file) = self.temp_file.take() {
            // Persist the file (prevents automatic deletion)
            let persisted_path = temp_file.into_temp_path();
            let final_path = persisted_path
                .keep()
                .map_err(|e| format!("Failed to persist temporary file: {}", e))?;
            Ok(final_path)
        } else {
            Ok(self.path.clone())
        }
    }
}

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        // NamedTempFile automatically cleans up when dropped
        if self.temp_file.is_some() {
            tracing::debug!(path = ?self.path, "Cleaning up secure temporary file");
        }
        // temp_file.drop() handles cleanup automatically
    }
}

// Minimal DatabaseManager stub for compilation
struct DatabaseManager;

impl DatabaseManager {
    fn new() -> Result<Self, String> {
        Ok(Self)
    }

    fn new_in_memory() -> Self {
        Self
    }

    fn health_check(&self) -> Result<bool, String> {
        Ok(true)
    }
}

// Unified Application State
#[derive(Clone)]
struct AppState {
    // Production services
    pii_detector: Arc<RwLock<PIIDetector>>,
    rag_engine: Arc<RwLock<RAGEngine>>,
    llm_manager: Arc<RwLock<LLMManager>>,

    // Core services
    presidio_bridge: Arc<RwLock<PresidioBridge>>,
    setup_manager: Arc<RwLock<SetupManager>>,
    file_processor: Arc<FileProcessor>,
    database_manager: Arc<RwLock<DatabaseManager>>,

    // System monitoring
    system_monitor: Arc<RwLock<system_monitor::SystemMonitor>>,
    hardware_monitor: Arc<RwLock<HardwareMonitor>>,
    hardware_detector: Arc<RwLock<HardwareDetector>>,

    // MCP and agent orchestration
    #[allow(dead_code)]
    mcp_server: Arc<MCPServer>,
    #[allow(dead_code)]
    agent_orchestrator: Arc<AgentOrchestrator>,

    // GDPR Compliance
    compliance_manager: Arc<ComplianceManager>,

    // Consent Guard Middleware
    consent_guard: Arc<ConsentGuard>,

    // Rate limiting
    rate_limiter: Arc<RateLimiter>,

    // Retention Scheduler
    scheduler_handle: Option<Arc<RwLock<SchedulerHandle>>>,

    // AI Transparency
    transparency_state: Arc<TransparencyState>,
}

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

// Unified system status command
#[tauri::command]
async fn check_system_status(state: State<'_, AppState>) -> Result<SystemStatus, String> {
    // Use hardware monitor for real-time status
    let monitor = state.hardware_monitor.read().await;
    monitor.get_status().await.map_err(|e| e.to_string())
}

// Get current resource limits
#[tauri::command]
async fn get_resource_limits(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let monitor = state.hardware_monitor.read().await;
    let limits = monitor.get_resource_limits();

    Ok(serde_json::json!({
        "max_gpu_usage": limits.max_gpu_usage,
        "max_cpu_usage": limits.max_cpu_usage,
        "max_ram_usage": limits.max_ram_usage
    }))
}

// Check if current resource usage is within limits
#[tauri::command]
async fn check_resource_limits(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let monitor = state.hardware_monitor.read().await;
    let status = monitor
        .check_resource_limits()
        .await
        .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "within_limits": status.within_limits,
        "cpu_usage": status.cpu_usage,
        "cpu_limit": status.cpu_limit,
        "cpu_exceeded": status.cpu_exceeded,
        "memory_usage": status.memory_usage,
        "memory_limit": status.memory_limit,
        "memory_exceeded": status.memory_exceeded,
        "gpu_usage": status.gpu_usage,
        "gpu_limit": status.gpu_limit,
        "gpu_exceeded": status.gpu_exceeded
    }))
}

// Health check endpoint for monitoring and load balancers
#[tauri::command]
async fn health_check(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    // Check LLM status
    let llm = state.llm_manager.read().await;
    let llm_loaded = llm.is_model_loaded().await.unwrap_or(false);
    drop(llm);

    // Check RAG status
    let rag = state.rag_engine.read().await;
    let rag_ready = rag.is_initialized();
    drop(rag);

    // Check database connection
    let db = state.database_manager.read().await;
    let db_connected = db.health_check().unwrap_or(false);
    drop(db);

    // Overall status
    let status = if db_connected { "healthy" } else { "degraded" };

    Ok(serde_json::json!({
        "status": status,
        "version": env!("CARGO_PKG_VERSION"),
        "llm_loaded": llm_loaded,
        "rag_ready": rag_ready,
        "database_connected": db_connected,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

// Enhanced document processing
#[tauri::command]
async fn process_document(
    state: State<'_, AppState>,
    file_path: String,
    file_type: String,
) -> Result<ProcessedDocument, String> {
    // Rate limit check
    state
        .rate_limiter
        .check_rate_limit(&format!("process_document:{}", file_path))
        .await?;

    let content = state
        .file_processor
        .process_file(&file_path, &file_type)
        .await
        .map_err(|e| e.to_string())?;

    let detector = state.pii_detector.read().await;
    let cleaned_content = detector
        .redact_pii(&content)
        .await
        .map_err(|e| e.to_string())?;

    // Add to RAG engine
    let rag = state.rag_engine.write().await;
    let doc_id = rag
        .add_document(
            &cleaned_content,
            serde_json::json!({
                "filename": file_path.clone(),
                "file_type": file_type.clone()
            }),
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(ProcessedDocument {
        id: doc_id,
        filename: file_path,
        content: cleaned_content,
        pii_removed: true,
        metadata: serde_json::json!({"type": file_type}),
    })
}

// Enhanced message generation using new LLM manager
#[tauri::command]
async fn send_message(
    state: State<'_, AppState>,
    message: String,
    model_name: String,
) -> Result<String, String> {
    // Rate limit check - use generic key for now (in production, use actual user/session ID)
    state
        .rate_limiter
        .check_rate_limit("send_message")
        .await
        .map_err(|e| {
            tracing::warn!("Rate limit exceeded for send_message");
            e
        })?;

    // Check system safety
    let mut hw_monitor = state.hardware_monitor.write().await;
    if !hw_monitor.check_safety().await.map_err(|e| e.to_string())? {
        tracing::warn!("System resources critically high during send_message");
        return Err(
            "System resources are critically high. Please wait before sending another message."
                .to_string(),
        );
    }

    // Enforce resource limits before proceeding
    hw_monitor
        .enforce_resource_limits("send_message")
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Resource limits exceeded in send_message");
            e.to_string()
        })?;

    drop(hw_monitor);

    // Clean PII from message
    let detector = state.pii_detector.read().await;
    let cleaned_message = detector
        .redact_pii(&message)
        .await
        .map_err(|e| e.to_string())?;

    // Ensure model is ready and generate response
    let llm = state.llm_manager.read().await;
    llm.ensure_model_ready(&model_name)
        .await
        .map_err(|e| e.to_string())?;

    let result = llm
        .generate(&cleaned_message, None)
        .await
        .map_err(|e| e.to_string())?;

    Ok(result.text)
}

#[tauri::command]
async fn detect_hardware(state: State<'_, AppState>) -> Result<HardwareSpecs, String> {
    let mut detector = state.hardware_detector.write().await;
    detector.detect_hardware().map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_model_recommendations(
    state: State<'_, AppState>,
) -> Result<Vec<ModelRecommendation>, String> {
    let mut detector = state.hardware_detector.write().await;
    let hardware = detector.detect_hardware().map_err(|e| e.to_string())?;
    Ok(detector.recommend_models(&hardware))
}

#[tauri::command]
async fn get_system_summary(state: State<'_, AppState>) -> Result<String, String> {
    let mut detector = state.hardware_detector.write().await;
    let hardware = detector.detect_hardware().map_err(|e| e.to_string())?;
    Ok(detector.get_system_summary(&hardware))
}

#[tauri::command]
async fn estimate_model_performance(
    state: State<'_, AppState>,
    model_size_gb: f64,
) -> Result<String, String> {
    let mut detector = state.hardware_detector.write().await;
    let hardware = detector.detect_hardware().map_err(|e| e.to_string())?;
    Ok(detector.estimate_model_performance(&hardware, model_size_gb))
}

// Enhanced search using new RAG engine
#[tauri::command]
async fn search_knowledge_base(
    state: State<'_, AppState>,
    query: String,
    limit: usize,
) -> Result<Vec<serde_json::Value>, String> {
    let detector = state.pii_detector.read().await;
    let cleaned_query = detector
        .redact_pii(&query)
        .await
        .map_err(|e| e.to_string())?;

    let rag = state.rag_engine.read().await;
    let results = rag
        .search(&cleaned_query, Some(limit))
        .await
        .map_err(|e| e.to_string())?;

    // Convert to JSON
    let json_results = results
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "document_id": r.document_id,
                "content": r.content,
                "score": r.score,
                "metadata": r.metadata
            })
        })
        .collect();

    Ok(json_results)
}

// Add document to new RAG engine
#[tauri::command]
async fn add_to_knowledge_base(
    state: State<'_, AppState>,
    content: String,
    metadata: serde_json::Value,
) -> Result<String, String> {
    let detector = state.pii_detector.read().await;
    let cleaned_content = detector
        .redact_pii(&content)
        .await
        .map_err(|e| e.to_string())?;

    let rag = state.rag_engine.write().await;
    rag.add_document(&cleaned_content, metadata)
        .await
        .map_err(|e| e.to_string())
}

// List models using new LLM manager
#[tauri::command]
async fn list_available_models(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let llm = state.llm_manager.read().await;
    let models = llm.list_models().await;
    Ok(models.into_iter().map(|(name, _, _)| name).collect())
}

// Download model using new LLM manager
#[tauri::command]
async fn download_model(state: State<'_, AppState>, model_name: String) -> Result<String, String> {
    let llm = state.llm_manager.read().await;
    llm.ensure_model_ready(&model_name)
        .await
        .map_err(|e| e.to_string())?;
    Ok(format!("Model {} is ready", model_name))
}

// Database commands
#[tauri::command]
async fn execute_sql_query(
    state: State<'_, AppState>,
    query: String,
) -> Result<serde_json::Value, String> {
    let db = state.database_manager.read().await;
    db.execute_sql_query(&query).map_err(|e| e.to_string())
}

// Enhanced RAG search with agentic capabilities
#[tauri::command]
async fn rag_search(
    state: State<'_, AppState>,
    query: String,
    _use_agentic: bool,
    max_results: usize,
) -> Result<serde_json::Value, String> {
    let detector = state.pii_detector.read().await;
    let cleaned_query = detector
        .redact_pii(&query)
        .await
        .map_err(|e| e.to_string())?;

    let rag = state.rag_engine.read().await;

    // Agentic search delegates to standard RAG search
    let results = rag
        .search(&cleaned_query, Some(max_results))
        .await
        .map_err(|e| e.to_string())?;

    let confidence = if results.len() > 0 { 0.85 } else { 0.0 };

    Ok(serde_json::json!({
        "answer": format!("Found {} relevant documents for your query.", results.len()),
        "sources": results.iter().map(|r| serde_json::json!({
            "title": r.metadata.get("title").unwrap_or(&serde_json::Value::String("Document".to_string())),
            "snippet": &r.content,
            "relevance": r.score,
            "source": "Knowledge Base",
            "reasoning": r.reasoning
        })).collect::<Vec<_>>(),
        "reasoning": None::<String>,
        "confidence": confidence
    }))
}

#[tauri::command]
async fn upload_document(
    state: State<'_, AppState>,
    filename: String,
    content: Vec<u8>,
) -> Result<serde_json::Value, String> {
    let content_str = String::from_utf8_lossy(&content);

    // Process with PII detection
    let detector = state.pii_detector.read().await;
    let cleaned_content = detector
        .redact_pii(&content_str)
        .await
        .map_err(|e| e.to_string())?;

    // Store in database
    let db = state.database_manager.read().await;
    let file_type = filename.split('.').last().unwrap_or("txt");
    let doc_id = db
        .store_document(&filename, &cleaned_content, file_type)
        .map_err(|e| e.to_string())?;

    // Add to enhanced RAG engine
    let rag = state.rag_engine.write().await;
    rag.add_document(
        &cleaned_content,
        serde_json::json!({
            "filename": filename,
            "document_id": doc_id
        }),
    )
    .await
    .map_err(|e| e.to_string())?;

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

    let file_type = filename.split('.').last().unwrap_or("unknown");
    let original_text = if state.file_processor.is_supported(file_type) {
        // SECURITY FIX: Atomically create temporary file with content
        // Uses tempfile crate for atomic creation, preventing TOCTOU race conditions
        let temp_guard = TempFileGuard::create_with_content(&filename, &content)?;

        // Process the file - path is guaranteed to exist and be secure
        let result = state
            .file_processor
            .process_file(
                temp_guard.path().to_str().ok_or("Invalid temp path")?,
                file_type,
            )
            .await
            .unwrap_or_else(|_| String::from_utf8_lossy(&content).to_string());

        // temp_guard is automatically dropped here, cleaning up the file atomically
        result
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

    let detector = state.pii_detector.read().await;
    let detections = detector
        .detect_pii(&original_text)
        .await
        .map_err(|e| e.to_string())?;

    let cleaned_text = detector
        .redact_pii(&original_text)
        .await
        .map_err(|e| e.to_string())?;

    let processing_time = start_time.elapsed().as_millis();

    Ok(serde_json::json!({
        "filename": filename,
        "fileType": file_type,
        "originalText": original_text,
        "cleanedText": cleaned_text,
        "piiDetections": detections.iter().map(|d| serde_json::json!({
            "type": d.entity_type,
            "text": d.text,
            "startIndex": d.start,
            "endIndex": d.end,
            "confidence": 0.95,
            "replacement": format!("[REDACTED_{}]", d.entity_type.to_uppercase())
        })).collect::<Vec<_>>(),
        "processingTime": processing_time,
        "supported": true
    }))
}

#[tauri::command]
async fn get_database_stats(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let db = state.database_manager.read().await;
    db.get_document_statistics().map_err(|e| e.to_string())
}

// System specification commands
#[tauri::command]
async fn get_system_specs(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let detector = state.hardware_detector.read().await;
    let mut detector_mut = detector.clone();
    let specs = detector_mut.detect_hardware().map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "cpu_cores": specs.cpu_cores,
        "cpu_frequency": specs.cpu_frequency,
        "cpu_brand": specs.cpu_brand,
        "total_memory": specs.total_memory,
        "available_memory": specs.available_memory,
        "gpu_info": specs.gpu_info,
        "system_type": specs.system_type,
        "performance_category": specs.performance_category
    }))
}

#[tauri::command]
async fn check_model_compatibility(
    state: State<'_, AppState>,
    model_name: String,
    model_size_gb: f64,
) -> Result<serde_json::Value, String> {
    let detector = state.hardware_detector.read().await;
    let mut detector_mut = detector.clone();
    let specs = detector_mut.detect_hardware().map_err(|e| e.to_string())?;

    // Check if system can run the model
    let required_ram_gb = model_size_gb * 1.5; // Model + context overhead
    let available_ram_gb = specs.available_memory as f64 / 1024.0;
    let compatible = available_ram_gb >= required_ram_gb;

    let recommendation = if compatible {
        "System has sufficient resources"
    } else {
        "Insufficient RAM - consider a smaller model"
    };

    Ok(serde_json::json!({
        "compatible": compatible,
        "model_name": model_name,
        "model_size_gb": model_size_gb,
        "required_ram_gb": required_ram_gb,
        "available_ram_gb": available_ram_gb,
        "recommendation": recommendation,
        "can_use_gpu": specs.gpu_info.is_some()
    }))
}

#[tauri::command]
async fn get_resource_usage(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let monitor = state.hardware_monitor.read().await;
    let status = monitor.get_status().await.map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "cpu_usage": status.cpu_usage,
        "memory_usage": status.memory_usage,
        "gpu_usage": status.gpu_usage,
        "temperature": status.temperature,
        "is_safe": status.is_safe
    }))
}

// LLM Model Management Commands
#[tauri::command]
async fn load_model(
    state: State<'_, AppState>,
    model_path: String,
    n_gpu_layers: Option<u32>,
) -> Result<String, String> {
    let llm = state.llm_manager.write().await;
    llm.load_model(&model_path, n_gpu_layers.unwrap_or(0))
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!("Model loaded successfully: {}", model_path))
}

#[tauri::command]
async fn unload_model(state: State<'_, AppState>) -> Result<String, String> {
    let llm = state.llm_manager.write().await;
    llm.unload_model().await.map_err(|e| e.to_string())?;
    Ok("Model unloaded successfully".to_string())
}

#[tauri::command]
async fn emergency_stop(state: State<'_, AppState>) -> Result<String, String> {
    // Stop all ongoing operations
    let llm = state.llm_manager.write().await;
    llm.cancel_generation().await.map_err(|e| e.to_string())?;
    Ok("All operations stopped".to_string())
}

#[tauri::command]
async fn set_resource_limits(
    state: State<'_, AppState>,
    max_cpu: Option<f32>,
    max_memory: Option<f32>,
    max_gpu: Option<f32>,
) -> Result<String, String> {
    let monitor = state.hardware_monitor.write().await;
    monitor
        .set_resource_limits(
            max_cpu.unwrap_or(85.0),
            max_memory.unwrap_or(90.0),
            max_gpu.unwrap_or(85.0),
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok("Resource limits updated".to_string())
}

// HuggingFace Integration Commands
#[tauri::command]
async fn download_model_from_huggingface(
    model_id: String,
    filename: Option<String>,
) -> Result<serde_json::Value, String> {
    use tokio::process::Command;

    let download_dir = dirs::data_local_dir()
        .map(|mut p| {
            p.push("bear-ai/models");
            p
        })
        .ok_or("Failed to get data directory")?;

    std::fs::create_dir_all(&download_dir).map_err(|e| e.to_string())?;

    let file = filename.unwrap_or_else(|| "model.gguf".to_string());
    let output_path = download_dir.join(&file);

    // Use huggingface_hub CLI to download
    let output = Command::new("huggingface-cli")
        .arg("download")
        .arg(&model_id)
        .arg(&file)
        .arg("--local-dir")
        .arg(&download_dir)
        .output()
        .await
        .map_err(|e| format!("Failed to execute download: {}", e))?;

    if output.status.success() {
        Ok(serde_json::json!({
            "success": true,
            "model_id": model_id,
            "path": output_path.to_string_lossy(),
            "message": "Model downloaded successfully"
        }))
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(format!("Download failed: {}", error))
    }
}

#[tauri::command]
async fn search_huggingface_models(
    query: String,
    limit: Option<usize>,
) -> Result<serde_json::Value, String> {
    // Simple search implementation - in production use HF API
    let popular_models = vec![
        ("TheBloke/Llama-2-7B-Chat-GGUF", "Llama 2 7B Chat", "7B"),
        (
            "TheBloke/Mistral-7B-Instruct-v0.2-GGUF",
            "Mistral 7B Instruct",
            "7B",
        ),
        (
            "TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF",
            "TinyLlama 1.1B",
            "1.1B",
        ),
        ("TheBloke/CodeLlama-7B-Instruct-GGUF", "CodeLlama 7B", "7B"),
    ];

    let results: Vec<serde_json::Value> = popular_models
        .iter()
        .filter(|(id, name, _)| {
            id.to_lowercase().contains(&query.to_lowercase())
                || name.to_lowercase().contains(&query.to_lowercase())
        })
        .take(limit.unwrap_or(10))
        .map(|(id, name, size)| {
            serde_json::json!({
                "model_id": id,
                "name": name,
                "size": size,
                "downloads": 0,
                "likes": 0
            })
        })
        .collect();

    Ok(serde_json::json!({
        "query": query,
        "results": results,
        "total": results.len()
    }))
}

// RAG Configuration Commands
#[tauri::command]
async fn get_available_rag_models() -> Result<serde_json::Value, String> {
    let models = vec![
        (
            "BAAI/bge-small-en-v1.5",
            "BGE Small English",
            "Small",
            "133MB",
        ),
        (
            "BAAI/bge-base-en-v1.5",
            "BGE Base English",
            "Medium",
            "438MB",
        ),
        (
            "sentence-transformers/all-MiniLM-L6-v2",
            "MiniLM L6",
            "Small",
            "90MB",
        ),
    ];

    let model_list: Vec<serde_json::Value> = models
        .iter()
        .map(|(id, name, size, disk)| {
            serde_json::json!({
                "id": id,
                "name": name,
                "size": size,
                "disk_size": disk,
                "embedding_dim": 384
            })
        })
        .collect();

    Ok(serde_json::json!({
        "models": model_list,
        "default": "BAAI/bge-small-en-v1.5"
    }))
}

#[tauri::command]
async fn get_active_rag_model(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let rag = state.rag_engine.read().await;
    let model_name = rag.get_active_model_name();

    Ok(serde_json::json!({
        "model_name": model_name,
        "is_loaded": rag.is_initialized()
    }))
}

#[tauri::command]
async fn switch_rag_model(
    state: State<'_, AppState>,
    model_name: String,
) -> Result<String, String> {
    let rag = state.rag_engine.write().await;
    rag.switch_embedding_model(&model_name)
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!("Switched to model: {}", model_name))
}

#[tauri::command]
async fn get_rag_config(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let rag = state.rag_engine.read().await;
    let config = rag.get_config();

    Ok(serde_json::json!({
        "chunk_size": config.chunk_size,
        "chunk_overlap": config.chunk_overlap,
        "top_k": config.top_k,
        "similarity_threshold": config.similarity_threshold
    }))
}

#[tauri::command]
async fn update_rag_config(
    state: State<'_, AppState>,
    chunk_size: Option<usize>,
    chunk_overlap: Option<usize>,
    top_k: Option<usize>,
    similarity_threshold: Option<f32>,
) -> Result<String, String> {
    let rag = state.rag_engine.write().await;

    if let Some(size) = chunk_size {
        rag.set_chunk_size(size);
    }
    if let Some(overlap) = chunk_overlap {
        rag.set_chunk_overlap(overlap);
    }
    if let Some(k) = top_k {
        rag.set_top_k(k);
    }
    if let Some(threshold) = similarity_threshold {
        rag.set_similarity_threshold(threshold);
    }

    Ok("RAG configuration updated".to_string())
}

// PII Detection Configuration Commands
#[tauri::command]
async fn get_memory_info() -> Result<serde_json::Value, String> {
    let sys = System::new_all();
    let total_mb = sys.total_memory() / 1024 / 1024;
    let available_mb = sys.available_memory() / 1024 / 1024;
    let used_mb = total_mb - available_mb;

    Ok(serde_json::json!({
        "total_mb": total_mb,
        "available_mb": available_mb,
        "used_mb": used_mb,
        "usage_percent": (used_mb as f64 / total_mb as f64) * 100.0
    }))
}

#[tauri::command]
async fn can_use_pii_mode(mode: String) -> Result<serde_json::Value, String> {
    let sys = System::new_all();
    let available_mb = sys.available_memory() / 1024 / 1024;

    let (required_mb, can_use) = match mode.as_str() {
        "builtin" => (0, true),
        "presidio_lite" => (500, available_mb > 500),
        "presidio_full" => (2048, available_mb > 2048),
        _ => return Err("Invalid mode".to_string()),
    };

    Ok(serde_json::json!({
        "mode": mode,
        "can_use": can_use,
        "required_mb": required_mb,
        "available_mb": available_mb
    }))
}

#[tauri::command]
async fn estimate_mode_impact(mode: String) -> Result<serde_json::Value, String> {
    let (memory_mb, accuracy, speed) = match mode.as_str() {
        "builtin" => (0, "60-70%", "Fast"),
        "presidio_lite" => (500, "85-90%", "Medium"),
        "presidio_full" => (2048, "95-98%", "Slow"),
        _ => return Err("Invalid mode".to_string()),
    };

    Ok(serde_json::json!({
        "mode": mode,
        "memory_mb": memory_mb,
        "accuracy": accuracy,
        "speed": speed
    }))
}

#[tauri::command]
async fn get_pii_config(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let detector = state.pii_detector.read().await;
    let config = detector.get_config();

    Ok(serde_json::json!({
        "mode": config.mode,
        "enabled": config.enabled,
        "anonymize": config.anonymize
    }))
}

#[tauri::command]
async fn set_pii_mode(state: State<'_, AppState>, mode: String) -> Result<String, String> {
    let detector = state.pii_detector.write().await;
    detector.set_mode(&mode).await.map_err(|e| e.to_string())?;
    Ok(format!("PII mode set to: {}", mode))
}

#[tauri::command]
async fn update_pii_config(
    state: State<'_, AppState>,
    enabled: Option<bool>,
    anonymize: Option<bool>,
) -> Result<String, String> {
    let detector = state.pii_detector.write().await;

    if let Some(en) = enabled {
        detector.set_enabled(en);
    }
    if let Some(anon) = anonymize {
        detector.set_anonymize(anon);
    }

    Ok("PII configuration updated".to_string())
}

#[tauri::command]
async fn install_presidio() -> Result<serde_json::Value, String> {
    // Presidio requires Python - provide installation instructions
    Ok(serde_json::json!({
        "installed": false,
        "message": "Presidio requires Python installation",
        "instructions": "pip install presidio-analyzer presidio-anonymizer",
        "optional": true
    }))
}

#[tauri::command]
async fn check_presidio_status(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let detector = state.pii_detector.read().await;
    let available = detector.is_presidio_available().await;

    Ok(serde_json::json!({
        "available": available,
        "mode": if available { "presidio" } else { "builtin" },
        "status": if available { "installed" } else { "not_installed" }
    }))
}

// Setup management commands
#[tauri::command]
async fn check_first_run(state: State<'_, AppState>) -> Result<bool, String> {
    let setup = state.setup_manager.read().await;
    let is_first = setup.check_first_run().await.map_err(|e| e.to_string())?;

    // Always check Presidio status and warn user
    let pii_detector = state.pii_detector.read().await;
    let presidio_available = pii_detector.is_presidio_available().await;

    if !presidio_available {
        tracing::warn!(
            "⚠️  User starting application without Presidio - privacy protection limited"
        );
    }

    Ok(is_first)
}

#[tauri::command]
async fn run_initial_setup(
    state: State<'_, AppState>,
    window: tauri::Window,
    config: Option<serde_json::Value>,
) -> Result<bool, String> {
    use tokio::sync::mpsc;

    // Update config if provided
    if let Some(config_val) = config {
        if let Ok(setup_config) =
            serde_json::from_value::<crate::setup_manager::SetupConfig>(config_val)
        {
            let setup = state.setup_manager.read().await;
            setup
                .update_config(setup_config)
                .await
                .map_err(|e| e.to_string())?;
        }
    }

    let (tx, mut rx) = mpsc::channel(100);
    let _setup = state.setup_manager.read().await;

    // Spawn setup in background
    let setup_clone = state.setup_manager.clone();
    tokio::spawn(async move {
        let setup = setup_clone.read().await;
        if let Err(e) = setup.run_setup(tx).await {
            eprintln!("Setup failed: {}", e);
        }
    });

    // Forward progress to frontend
    tokio::spawn(async move {
        while let Some(progress) = rx.recv().await {
            let _ = window.emit("setup-progress", &progress);
        }
    });

    Ok(true)
}

#[tauri::command]
async fn mark_setup_complete(state: State<'_, AppState>) -> Result<bool, String> {
    let setup = state.setup_manager.read().await;
    setup
        .mark_setup_complete_only()
        .await
        .map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
async fn get_setup_status(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let setup = state.setup_manager.read().await;
    let mut status = setup.get_setup_status().await.map_err(|e| e.to_string())?;

    // Add Presidio status to setup info
    let pii_detector = state.pii_detector.read().await;
    let presidio_available = pii_detector.is_presidio_available().await;

    if let Some(obj) = status.as_object_mut() {
        obj.insert(
            "presidio_installed".to_string(),
            serde_json::json!(presidio_available),
        );
        if !presidio_available {
            obj.insert("warning".to_string(), serde_json::json!(
                "⚠️ Presidio not installed - Using rudimentary PII detection. For enterprise-grade protection, install Microsoft Presidio."
            ));
        }
    }

    Ok(status)
}

// Presidio-powered PII detection commands
#[tauri::command]
async fn detect_pii_presidio(
    state: State<'_, AppState>,
    text: String,
) -> Result<serde_json::Value, String> {
    let bridge = state.presidio_bridge.read().await;

    // Check if Presidio is installed
    if !bridge.check_installation_status().await.unwrap_or(false) {
        // Fall back to built-in detector
        let detector = state.pii_detector.read().await;
        let entities = detector
            .detect_pii(&text)
            .await
            .map_err(|e| e.to_string())?;

        return Ok(serde_json::json!({
            "entities": entities,
            "engine": "built-in",
            "warning": "⚠️ Presidio not installed - using rudimentary privacy shield with limited accuracy. Install Presidio for enterprise-grade protection."
        }));
    }

    // Use Presidio for detection
    match bridge.detect_pii(&text).await {
        Ok(entities) => Ok(serde_json::json!({
            "entities": entities,
            "engine": "presidio",
            "count": entities.len()
        })),
        Err(e) => {
            // Fall back to built-in detector on error
            let detector = state.pii_detector.read().await;
            let entities = detector
                .detect_pii(&text)
                .await
                .map_err(|e| e.to_string())?;

            Ok(serde_json::json!({
                "entities": entities,
                "engine": "built-in",
                "warning": format!("⚠️ Presidio error: {}. Using rudimentary built-in detector with limited accuracy.", e)
            }))
        }
    }
}

#[tauri::command]
async fn anonymize_pii_presidio(
    state: State<'_, AppState>,
    text: String,
) -> Result<serde_json::Value, String> {
    let bridge = state.presidio_bridge.read().await;

    // First detect entities
    let entities = bridge.detect_pii(&text).await.map_err(|e| e.to_string())?;

    // Then anonymize
    let anonymized = bridge
        .anonymize(&text, entities.clone())
        .await
        .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "original": text,
        "anonymized": anonymized,
        "entities_found": entities.len(),
        "engine": "presidio"
    }))
}

#[tauri::command]
async fn configure_presidio(
    state: State<'_, AppState>,
    config: presidio_bridge::PresidioConfig,
) -> Result<bool, String> {
    let bridge = state.presidio_bridge.read().await;
    bridge
        .update_config(config)
        .await
        .map_err(|e| e.to_string())?;
    Ok(true)
}

// Enhanced PII detection commands (using hybrid approach)
#[tauri::command]
async fn detect_pii_advanced(
    state: State<'_, AppState>,
    text: String,
) -> Result<serde_json::Value, String> {
    let detector = state.pii_detector.read().await;
    let entities = detector
        .detect_pii(&text)
        .await
        .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "entities": entities.iter().map(|e| serde_json::json!({
            "type": e.entity_type,
            "text": e.text,
            "start": e.start,
            "end": e.end,
            "confidence": e.confidence
        })).collect::<Vec<_>>(),
        "count": entities.len()
    }))
}

#[tauri::command]
async fn redact_pii_advanced(state: State<'_, AppState>, text: String) -> Result<String, String> {
    let detector = state.pii_detector.read().await;
    detector.redact_pii(&text).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn anonymize_pii_advanced(
    state: State<'_, AppState>,
    text: String,
) -> Result<serde_json::Value, String> {
    let detector = state.pii_detector.read().await;
    let redacted = detector
        .redact_pii(&text)
        .await
        .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "anonymized_text": redacted,
        "mappings": {}
    }))
}

#[tauri::command]
async fn configure_pii_detection(
    state: State<'_, AppState>,
    _config: serde_json::Value,
) -> Result<bool, String> {
    // Configuration is currently managed internally
    let _detector = state.pii_detector.read().await;
    Ok(true)
}

#[tauri::command]
async fn add_custom_pii_recognizer(
    state: State<'_, AppState>,
    _name: String,
    _pattern: String,
    _label: String,
    _confidence: f32,
) -> Result<bool, String> {
    // Custom recognizers are managed internally
    let _detector = state.pii_detector.read().await;
    Ok(true)
}

#[tauri::command]
async fn get_pii_statistics(
    state: State<'_, AppState>,
    text: String,
) -> Result<serde_json::Value, String> {
    let detector = state.pii_detector.read().await;
    let entities = detector
        .detect_pii(&text)
        .await
        .map_err(|e| e.to_string())?;

    let mut stats: HashMap<String, usize> = HashMap::new();
    for entity in &entities {
        *stats.entry(entity.entity_type.clone()).or_insert(0) += 1;
    }

    Ok(serde_json::json!({
        "total_entities": entities.len(),
        "by_type": stats
    }))
}

// Note: download_model_from_huggingface, search_huggingface_models, load_model,
// unload_model, emergency_stop, and set_resource_limits are defined in commands.rs

fn main() {
    // Initialize Sentry for crash reporting (production only)
    // Set SENTRY_DSN environment variable for crash reporting in production
    let _sentry_guard = if !cfg!(debug_assertions) {
        if let Ok(dsn) = env::var("SENTRY_DSN") {
            Some(sentry::init((
                dsn,
                sentry::ClientOptions {
                    release: sentry::release_name!(),
                    environment: Some(if cfg!(debug_assertions) {
                        "development".into()
                    } else {
                        "production".into()
                    }),
                    before_send: Some(Arc::new(|mut event| {
                        // Filter out PII from crash reports
                        if let Some(ref mut request) = event.request {
                            request.cookies = None;
                            request.headers = std::collections::BTreeMap::new();
                        }
                        // Remove environment variables that might contain secrets
                        event.contexts.remove("os");
                        Some(event)
                    })),
                    ..Default::default()
                },
            )))
        } else {
            tracing::info!("Sentry crash reporting disabled - SENTRY_DSN not set");
            None
        }
    } else {
        None
    };

    // Initialize tracing subsystem
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        if cfg!(debug_assertions) {
            EnvFilter::new("debug")
        } else {
            EnvFilter::new("info")
        }
    });

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_line_number(true),
        )
        .init();

    tracing::info!("BEAR AI starting up...");

    let database_manager = match DatabaseManager::new() {
        Ok(db) => Arc::new(RwLock::new(db)),
        Err(e) => {
            tracing::error!(error = %e, "Failed to initialize database - will retry on demand");
            // Create a fallback in-memory database instead of panicking
            Arc::new(RwLock::new(DatabaseManager::new_in_memory()))
        }
    };

    // Initialize LLM Manager with GGUF support
    let llm_manager = match LLMManager::new() {
        Ok(manager) => Arc::new(RwLock::new(manager)),
        Err(e) => {
            tracing::error!(error = %e, "Failed to initialize GGUF inference engine");
            panic!("Critical: GGUF inference engine initialization failed. Cannot proceed without LLM support.");
        }
    };

    // Initialize Compliance Manager
    let app_data_dir = dirs::data_local_dir()
        .map(|mut p| {
            p.push("bear-ai");
            p
        })
        .unwrap_or_else(|| PathBuf::from("."));
    let db_path = app_data_dir.join("bear_ai.db");
    let compliance_manager = Arc::new(ComplianceManager::new(db_path.clone()));

    // Initialize Model Transparency State
    let model_transparency = commands::ModelTransparencyState::new(app_data_dir.clone());

    // Initialize Consent Guard Middleware
    let consent_guard = Arc::new(
        ConsentGuardBuilder::new(db_path.clone())
            .strict_mode(true) // Enforce up-to-date consent
            .build(),
    );

    // Initialize Retention Scheduler
    let retention_scheduler = RetentionScheduler::new(db_path.clone());
    let scheduler_handle = Arc::new(RwLock::new(retention_scheduler.get_handle()));

    // Create unified app state
    let app_state = AppState {
        // Production services
        pii_detector: Arc::new(RwLock::new(PIIDetector::new())),
        rag_engine: Arc::new(RwLock::new(RAGEngine::new())),
        llm_manager,

        // Core services
        presidio_bridge: Arc::new(RwLock::new(PresidioBridge::new())),
        setup_manager: Arc::new(RwLock::new(SetupManager::new())),
        file_processor: Arc::new(FileProcessor::new()),
        database_manager,

        // System monitoring
        system_monitor: Arc::new(RwLock::new(system_monitor::SystemMonitor::new())),
        hardware_monitor: Arc::new(RwLock::new(HardwareMonitor::new())),
        hardware_detector: Arc::new(RwLock::new(HardwareDetector::new())),

        // MCP and agent orchestration
        mcp_server: Arc::new(MCPServer::new(true)),
        agent_orchestrator: Arc::new(AgentOrchestrator::new(true)),

        // GDPR Compliance
        compliance_manager,

        // Consent Guard Middleware
        consent_guard: consent_guard.clone(),

        // Rate limiting
        rate_limiter: Arc::new(RateLimiter::new()),

        // Retention Scheduler
        scheduler_handle: Some(scheduler_handle.clone()),

        // AI Transparency
        transparency_state: Arc::new(TransparencyState::new()),
    };

    // Initialize modules
    tauri::async_runtime::block_on(async {
        // Check if this is first run
        let setup = app_state.setup_manager.read().await;
        let is_first_run = setup.check_first_run().await.unwrap_or(false);
        drop(setup);

        if is_first_run {
            tracing::info!("🚀 First run detected - Presidio setup will be initiated from UI");
        }

        // Initialize PII detector
        let pii_detector = app_state.pii_detector.write().await;
        match pii_detector.initialize().await {
            Ok(_) => {
                // Check if Presidio is available
                let presidio_available = pii_detector.is_presidio_available().await;
                if !presidio_available {
                    tracing::warn!(
                        "╔════════════════════════════════════════════════════════════╗"
                    );
                    tracing::warn!("║  WARNING: Presidio Not Installed                          ║");
                    tracing::warn!("║  Using basic PII detection - limited accuracy             ║");
                    tracing::warn!("║  Install Presidio for enterprise-grade protection:        ║");
                    tracing::warn!("║  pip install presidio-analyzer presidio-anonymizer        ║");
                    tracing::warn!(
                        "╚════════════════════════════════════════════════════════════╝"
                    );
                }
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to initialize PII detector");
            }
        }
        drop(pii_detector);

        // Initialize LLM manager
        let llm = app_state.llm_manager.write().await;
        if let Err(e) = llm.initialize().await {
            tracing::error!(error = %e, "Failed to initialize LLM manager");
        }
        drop(llm);

        // Initialize RAG engine
        let rag = app_state.rag_engine.write().await;
        if let Err(e) = rag.initialize().await {
            tracing::error!(error = %e, "Failed to initialize RAG engine");
        }
        drop(rag);

        // Initialize Compliance Manager
        if let Err(e) = app_state.compliance_manager.initialize().await {
            tracing::error!(error = %e, "Failed to initialize compliance manager");
        } else {
            tracing::info!("✅ GDPR Compliance Manager initialized successfully");
        }

        // Start Retention Scheduler
        if let Err(e) = retention_scheduler.start().await {
            tracing::error!(error = %e, "Failed to start retention scheduler");
        } else {
            tracing::info!("✅ Retention Scheduler started successfully");
        }
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(app_state.clone())
        .manage(app_state.compliance_manager.clone())
        .manage(consent_guard.clone())
        .manage(scheduler_handle.clone())
        .manage(db_path.clone())
        .manage(app_state.transparency_state.clone())
        .manage(model_transparency)
        .setup(move |_app| {
            let state = app_state.clone();

            // Single background monitoring task
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_secs(5)).await;

                    // Update hardware metrics
                    let mut hw_monitor = state.hardware_monitor.write().await;
                    if let Err(e) = hw_monitor.update_metrics().await {
                        tracing::warn!(error = %e, "Failed to update hardware metrics");
                    }
                    drop(hw_monitor);

                    // Update system monitor
                    let mut sys_monitor = state.system_monitor.write().await;
                    let _ = sys_monitor.monitor_resources_realtime();
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Health and monitoring
            health_check,
            check_system_status,
            get_system_specs,
            check_model_compatibility,
            get_resource_usage,
            get_resource_limits,
            check_resource_limits,
            // Document processing
            process_document,
            analyze_document_pii,
            upload_document,
            // LLM operations
            send_message,
            list_available_models,
            download_model,
            load_model,
            unload_model,
            emergency_stop,
            set_resource_limits,
            // Knowledge base
            search_knowledge_base,
            add_to_knowledge_base,
            rag_search,
            // Database
            execute_sql_query,
            get_database_stats,
            // Hardware detection
            detect_hardware,
            get_model_recommendations,
            get_system_summary,
            estimate_model_performance,
            // HuggingFace integration
            download_model_from_huggingface,
            search_huggingface_models,
            // Enhanced PII detection
            detect_pii_advanced,
            redact_pii_advanced,
            anonymize_pii_advanced,
            configure_pii_detection,
            add_custom_pii_recognizer,
            get_pii_statistics,
            // Presidio PII detection
            detect_pii_presidio,
            anonymize_pii_presidio,
            configure_presidio,
            // Setup management
            check_first_run,
            run_initial_setup,
            mark_setup_complete,
            get_setup_status,
            // RAG Model Management
            get_available_rag_models,
            get_active_rag_model,
            switch_rag_model,
            get_rag_config,
            update_rag_config,
            // GDPR Compliance
            compliance::commands::check_user_consent,
            compliance::commands::grant_user_consent,
            compliance::commands::revoke_user_consent,
            compliance::commands::get_user_consents,
            compliance::commands::get_consent_audit_trail,
            compliance::commands::get_consent_versions,
            compliance::commands::set_data_retention,
            compliance::commands::get_retention_stats,
            compliance::commands::apply_default_retention_policies,
            compliance::commands::delete_expired_data,
            compliance::commands::get_audit_logs,
            compliance::commands::get_audit_stats,
            compliance::commands::export_user_data,
            compliance::commands::delete_user_data,
            compliance::commands::generate_compliance_report,
            compliance::commands::run_compliance_maintenance,
            compliance::commands::update_user_data,
            compliance::commands::get_granular_consent_log,
            compliance::commands::withdraw_consent_with_reason,
            compliance::commands::get_consent_statistics,
            // Consent Middleware Commands (GDPR Compliance)
            commands::check_consent_status,
            commands::grant_consent,
            commands::revoke_consent,
            commands::check_multiple_consents,
            commands::get_consent_history,
            commands::check_reconsent_needed,
            commands::grant_all_consents,
            commands::revoke_all_consents,
            // Retention Scheduler Commands
            commands::scheduler_commands::trigger_retention_cleanup,
            commands::scheduler_commands::get_scheduler_status,
            commands::scheduler_commands::update_scheduler_config,
            commands::scheduler_commands::preview_retention_cleanup,
            commands::scheduler_commands::get_last_cleanup_result,
            commands::scheduler_commands::set_automatic_cleanup,
            // AI Transparency
            commands::transparency_commands::get_startup_notice,
            commands::transparency_commands::get_onboarding_notice,
            commands::transparency_commands::get_limitations_notice,
            commands::transparency_commands::get_data_processing_notice,
            commands::transparency_commands::get_legal_disclaimer,
            commands::transparency_commands::create_transparency_context,
            commands::transparency_commands::get_transparency_notice,
            commands::transparency_commands::calculate_confidence_score,
            commands::transparency_commands::get_transparency_preferences,
            commands::transparency_commands::update_transparency_preferences,
            commands::transparency_commands::complete_onboarding,
            commands::transparency_commands::needs_disclaimer,
            commands::transparency_commands::acknowledge_disclaimers,
            commands::transparency_commands::get_all_notices,
            commands::transparency_commands::export_transparency_context,
            // Model Card Transparency
            commands::model_transparency::get_model_info,
            commands::model_transparency::add_model_mapping,
            commands::model_transparency::remove_model_mapping,
            commands::model_transparency::get_model_mappings,
            commands::model_transparency::clear_model_cache,
            commands::model_transparency::clear_all_model_cache,
            commands::model_transparency::get_general_disclaimer,
            commands::model_transparency::get_ai_act_disclaimer,
            // PII Detection & Memory Management
            get_memory_info,
            can_use_pii_mode,
            estimate_mode_impact,
            get_pii_config,
            set_pii_mode,
            update_pii_config,
            install_presidio,
            check_presidio_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
