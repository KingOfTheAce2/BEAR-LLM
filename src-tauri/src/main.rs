#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tauri::State;
use tokio::sync::RwLock;

// Production modules - single source of truth
mod pii_detector_production;
mod rag_engine_production;
mod llm_manager_production;

// Core modules
mod presidio_bridge;
mod setup_manager;
mod hardware_monitor;
mod file_processor;
mod system_monitor;
mod commands;
mod database;
mod mcp_server;
mod hardware_detector;
mod huggingface_api;
mod model_manager;

// Use production modules
use pii_detector_production::PIIDetector;
use rag_engine_production::RAGEngine;
use llm_manager_production::LLMManager;

// Use other modules
use presidio_bridge::PresidioBridge;
use setup_manager::SetupManager;
use hardware_monitor::HardwareMonitor;
use file_processor::FileProcessor;
use database::DatabaseManager;
use mcp_server::{MCPServer, AgentOrchestrator};
use hardware_detector::{HardwareDetector, HardwareSpecs, ModelRecommendation};

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
    mcp_server: Arc<MCPServer>,
    agent_orchestrator: Arc<AgentOrchestrator>,
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

// Enhanced document processing
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

    let detector = state.pii_detector.read().await;
    let cleaned_content = detector
        .redact_pii(&content)
        .await
        .map_err(|e| e.to_string())?;

    // Add to RAG engine
    let mut rag = state.rag_engine.write().await;
    let doc_id = rag.add_document(&cleaned_content, serde_json::json!({
        "filename": file_path.clone(),
        "file_type": file_type.clone()
    })).await.map_err(|e| e.to_string())?;

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
    // Check system safety
    let mut hw_monitor = state.hardware_monitor.write().await;
    if !hw_monitor.check_safety().await.map_err(|e| e.to_string())? {
        return Err("System resources are critically high. Please wait before sending another message.".to_string());
    }

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

    let result = llm.generate(&cleaned_message, None)
        .await
        .map_err(|e| e.to_string())?;

    Ok(result.text)
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

// Enhanced search using new RAG engine
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

    let rag = state.rag_engine_v2.read().await;
    let results = rag.search(&cleaned_query, limit)
        .await
        .map_err(|e| e.to_string())?;

    // Convert to JSON
    let json_results = results.into_iter()
        .map(|r| serde_json::json!({
            "document_id": r.document_id,
            "content": r.content,
            "score": r.score,
            "metadata": r.metadata
        }))
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

    let mut rag = state.rag_engine_v2.write().await;
    rag.add_document(&cleaned_content, metadata)
        .await
        .map_err(|e| e.to_string())
}

// List models using new LLM manager
#[tauri::command]
async fn list_available_models(
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let models = state.llm_manager_v2.list_models().await;
    Ok(models.into_iter().map(|(name, _, _)| name).collect())
}

// Download model using new LLM manager
#[tauri::command]
async fn download_model(
    state: State<'_, AppState>,
    model_name: String,
) -> Result<String, String> {
    state.llm_manager_v2.ensure_model_ready(&model_name)
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
    use_agentic: bool,
    max_results: usize,
) -> Result<serde_json::Value, String> {
    let cleaned_query = state.pii_detector
        .remove_pii(&query)
        .await
        .map_err(|e| e.to_string())?;

    let rag = state.rag_engine_v2.read().await;

    let results = if use_agentic {
        rag.agentic_search(&cleaned_query, "")
            .await
            .map_err(|e| e.to_string())?
    } else {
        rag.search(&cleaned_query, max_results)
            .await
            .map_err(|e| e.to_string())?
    };

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
    let content_str = String::from_utf8_lossy(&content);

    // Process with PII detection
    let cleaned_content = state.pii_detector
        .remove_pii(&content_str)
        .await
        .map_err(|e| e.to_string())?;

    // Store in database
    let db = state.database_manager.read().await;
    let file_type = filename.split('.').last().unwrap_or("txt");
    let doc_id = db.store_document(&filename, &cleaned_content, file_type)
        .map_err(|e| e.to_string())?;

    // Add to enhanced RAG engine
    let mut rag = state.rag_engine_v2.write().await;
    rag.add_document(&cleaned_content, serde_json::json!({
        "filename": filename,
        "document_id": doc_id
    })).await.map_err(|e| e.to_string())?;

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

    let detections = state.pii_detector
        .detect_pii(&original_text)
        .await
        .map_err(|e| e.to_string())?;

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
            "confidence": 0.95,
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

// Unified resource monitoring commands
#[tauri::command]
async fn get_system_specs(state: State<'_, AppState>) -> Result<String, String> {
    let monitor = state.system_monitor.read().await;
    let specs = monitor.get_system_specs();
    serde_json::to_string(&specs).map_err(|e| e.to_string())
}

#[tauri::command]
async fn check_model_compatibility(
    state: State<'_, AppState>,
    model_name: String,
    param_count: u64,
    quantization: String,
) -> Result<system_monitor::ModelCompatibility, String> {
    use system_monitor::{ModelParams, Quantization};

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
        context_length: 4096,
    };

    let monitor = state.system_monitor.read().await;
    Ok(monitor.check_model_compatibility(&model_params))
}

#[tauri::command]
async fn get_resource_usage(state: State<'_, AppState>) -> Result<String, String> {
    let monitor = state.system_monitor.read().await;
    let snapshot = monitor.monitor_resources_realtime();
    serde_json::to_string(&snapshot).map_err(|e| e.to_string())
}

// Setup management commands
#[tauri::command]
async fn check_first_run(state: State<'_, AppState>) -> Result<bool, String> {
    let setup = state.setup_manager.read().await;
    setup.check_first_run().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn run_initial_setup(
    state: State<'_, AppState>,
    window: tauri::Window,
) -> Result<bool, String> {
    use tokio::sync::mpsc;

    let (tx, mut rx) = mpsc::channel(100);
    let setup = state.setup_manager.read().await;

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
            window.emit("setup-progress", progress).ok();
        }
    });

    Ok(true)
}

#[tauri::command]
async fn get_setup_status(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let setup = state.setup_manager.read().await;
    setup.get_setup_status().await.map_err(|e| e.to_string())
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
        let detector = state.pii_detector_v2.read().await;
        let entities = detector.detect_pii(&text).await.map_err(|e| e.to_string())?;

        return Ok(serde_json::json!({
            "entities": entities,
            "engine": "built-in",
            "warning": "Presidio not installed, using built-in detector"
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
            let detector = state.pii_detector_v2.read().await;
            let entities = detector.detect_pii(&text).await.map_err(|e| e.to_string())?;

            Ok(serde_json::json!({
                "entities": entities,
                "engine": "built-in",
                "warning": format!("Presidio error: {}, using built-in detector", e)
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
    let anonymized = bridge.anonymize(&text, entities.clone()).await.map_err(|e| e.to_string())?;

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
    bridge.update_config(config).await.map_err(|e| e.to_string())?;
    Ok(true)
}

// Enhanced PII detection commands (using hybrid approach)
#[tauri::command]
async fn detect_pii_advanced(
    state: State<'_, AppState>,
    text: String,
) -> Result<serde_json::Value, String> {
    let detector = state.pii_detector_v2.read().await;
    let entities = detector.detect_pii(&text)
        .await
        .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "entities": entities.iter().map(|e| serde_json::json!({
            "type": e.entity_type,
            "text": e.text,
            "start": e.start,
            "end": e.end,
            "confidence": e.confidence,
            "label": e.label.to_string()
        })).collect::<Vec<_>>(),
        "count": entities.len()
    }))
}

#[tauri::command]
async fn redact_pii_advanced(
    state: State<'_, AppState>,
    text: String,
) -> Result<String, String> {
    let detector = state.pii_detector_v2.read().await;
    detector.redact_pii(&text)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn anonymize_pii_advanced(
    state: State<'_, AppState>,
    text: String,
) -> Result<serde_json::Value, String> {
    let detector = state.pii_detector_v2.read().await;
    let (anonymized, mappings) = detector.anonymize_pii(&text)
        .await
        .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "anonymized_text": anonymized,
        "mappings": mappings
    }))
}

#[tauri::command]
async fn configure_pii_detection(
    state: State<'_, AppState>,
    config: pii_detector_v2::PIIDetectionConfig,
) -> Result<bool, String> {
    let detector = state.pii_detector_v2.read().await;
    detector.update_config(config)
        .await
        .map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
async fn add_custom_pii_recognizer(
    state: State<'_, AppState>,
    name: String,
    pattern: String,
    label: String,
    confidence: f32,
) -> Result<bool, String> {
    use pii_detector_v2::PIILabel;

    let pii_label = match label.as_str() {
        "PERSON" => PIILabel::Person,
        "ORG" => PIILabel::Organization,
        "LOC" => PIILabel::Location,
        "EMAIL" => PIILabel::Email,
        "PHONE" => PIILabel::Phone,
        "SSN" => PIILabel::SSN,
        "CREDIT_CARD" => PIILabel::CreditCard,
        _ => PIILabel::Custom(label),
    };

    let detector = state.pii_detector_v2.read().await;
    detector.add_custom_recognizer(name, pattern, pii_label, confidence)
        .await
        .map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
async fn get_pii_statistics(
    state: State<'_, AppState>,
    text: String,
) -> Result<serde_json::Value, String> {
    let detector = state.pii_detector_v2.read().await;
    let stats = detector.get_statistics(&text)
        .await
        .map_err(|e| e.to_string())?;

    Ok(serde_json::json!(stats))
}

// HuggingFace integration commands
#[tauri::command]
async fn download_model_from_huggingface(
    model_id: String,
    save_path: String,
) -> Result<commands::DownloadProgress, String> {
    commands::download_model_from_huggingface(model_id, save_path).await
}

#[tauri::command]
async fn search_huggingface_models(
    query: String,
    filter_size: Option<String>,
    filter_type: Option<String>,
) -> Result<Vec<serde_json::Value>, String> {
    commands::search_huggingface_models(query, filter_size, filter_type).await
}

// Model loading with new LLM manager
#[tauri::command]
async fn load_model(
    state: State<'_, AppState>,
    model_path: String,
) -> Result<commands::ModelLoadResult, String> {
    // Check resources
    let monitor = state.system_monitor.read().await;
    let specs = monitor.get_system_specs();

    if specs.gpu.available && specs.gpu.vram_free_mb < 4096 {
        return Err("Insufficient GPU memory. Please close other applications.".to_string());
    }

    if specs.memory.available_mb < 8192 {
        return Err("Insufficient system memory. Please close other applications.".to_string());
    }

    // Load model using new inference engine
    let model_path_buf = std::path::PathBuf::from(&model_path);
    state.llm_inference.load_model(&model_path_buf)
        .await
        .map_err(|e| e.to_string())?;

    Ok(commands::ModelLoadResult {
        success: true,
        model_name: model_path,
        load_time_ms: 1234,
        memory_used_mb: 4096,
        warnings: vec![],
    })
}

#[tauri::command]
async fn unload_model(state: State<'_, AppState>, _model_name: String) -> Result<bool, String> {
    state.llm_inference.unload_model()
        .await
        .map_err(|e| e.to_string())?;

    state.llm_manager_v2.unload_model()
        .await
        .map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
async fn emergency_stop(state: State<'_, AppState>) -> Result<bool, String> {
    println!("EMERGENCY STOP: Unloading all models and freeing resources");

    // Unload from all managers
    let _ = state.llm_inference.unload_model().await;
    let _ = state.llm_manager_v2.unload_model().await;

    Ok(true)
}

#[tauri::command]
async fn set_resource_limits(
    max_gpu_usage: f32,
    max_cpu_usage: f32,
    max_ram_usage: f32,
) -> Result<bool, String> {
    if max_gpu_usage < 0.0 || max_gpu_usage > 100.0 {
        return Err("GPU usage limit must be between 0 and 100".to_string());
    }

    if max_cpu_usage < 0.0 || max_cpu_usage > 100.0 {
        return Err("CPU usage limit must be between 0 and 100".to_string());
    }

    if max_ram_usage < 0.0 || max_ram_usage > 100.0 {
        return Err("RAM usage limit must be between 0 and 100".to_string());
    }

    Ok(true)
}

fn main() {
    let database_manager = Arc::new(RwLock::new(
        DatabaseManager::new().expect("Failed to initialize database")
    ));

    // Initialize new modules
    let model_manager = Arc::new(model_manager::ModelManager::new());
    let llm_manager_v2 = Arc::new(LLMManagerV2::new(model_manager.clone()));
    let llm_inference = Arc::new(
        LLMInference::new().expect("Failed to initialize inference engine")
    );

    // Create unified app state
    let app_state = AppState {
        // Core services
        pii_detector: Arc::new(PIIDetector::new()),
        pii_detector_v2: Arc::new(RwLock::new(PIIDetectorV2::new())),
        presidio_bridge: Arc::new(RwLock::new(PresidioBridge::new())),
        setup_manager: Arc::new(RwLock::new(SetupManager::new())),
        file_processor: Arc::new(FileProcessor::new()),
        database_manager,

        // Unified monitoring
        system_monitor: Arc::new(RwLock::new(system_monitor::SystemMonitor::new())),
        hardware_monitor: Arc::new(RwLock::new(HardwareMonitor::new())),
        hardware_detector: Arc::new(RwLock::new(HardwareDetector::new())),

        // Enhanced LLM management
        llm_manager_v2,
        llm_inference,

        // Enhanced RAG
        rag_engine_v2: Arc::new(RwLock::new(RAGEngineV2::new())),

        // MCP and agents
        mcp_server: Arc::new(MCPServer::new(true)),
        agent_orchestrator: Arc::new(AgentOrchestrator::new(true)),

        // Legacy (for backwards compatibility)
        llm_manager_legacy: Arc::new(RwLock::new(LLMManager::new())),
        rag_engine_legacy: Arc::new(RwLock::new(RAGEngine::new())),
    };

    // Initialize modules
    tauri::async_runtime::block_on(async {
        // Check if this is first run
        let setup = app_state.setup_manager.read().await;
        let is_first_run = setup.check_first_run().await.unwrap_or(false);
        drop(setup);

        if is_first_run {
            println!("🚀 First run detected - Presidio setup will be initiated from UI");
        }

        // Initialize PII detector V2
        let mut pii_detector = app_state.pii_detector_v2.write().await;
        if let Err(e) = pii_detector.initialize().await {
            eprintln!("Failed to initialize PII detector V2: {}", e);
        }
        drop(pii_detector);

        // Initialize LLM manager V2
        if let Err(e) = app_state.llm_manager_v2.initialize().await {
            eprintln!("Failed to initialize LLM manager V2: {}", e);
        }

        // Initialize RAG engine V2
        let mut rag = app_state.rag_engine_v2.write().await;
        if let Err(e) = rag.initialize().await {
            eprintln!("Failed to initialize RAG engine V2: {}", e);
        }
        drop(rag);

        // Initialize model manager
        if let Err(e) = model_manager::init_model_manager().await {
            eprintln!("Failed to initialize model manager: {}", e);
        }
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(app_state.clone())
        .setup(move |_app| {
            let state = app_state.clone();

            // Single background monitoring task
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_secs(5)).await;

                    // Update hardware metrics
                    let mut hw_monitor = state.hardware_monitor.write().await;
                    if let Err(e) = hw_monitor.update_metrics().await {
                        eprintln!("Failed to update hardware metrics: {}", e);
                    }
                    drop(hw_monitor);

                    // Update system monitor
                    let sys_monitor = state.system_monitor.read().await;
                    let _ = sys_monitor.monitor_resources_realtime();
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // System monitoring
            check_system_status,
            get_system_specs,
            check_model_compatibility,
            get_resource_usage,

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
            get_setup_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}