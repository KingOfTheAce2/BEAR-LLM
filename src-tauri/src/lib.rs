// Library exports for BEAR AI LLM
// This allows tests and external crates to use the modules

pub mod ai_transparency;
pub mod candle_inference; // Pure Rust inference (Candle-based GGUF)
pub mod commands;
pub mod compliance;
pub mod constants;
pub mod database;
pub mod export_engine;
pub mod hardware_monitor;
pub mod llm_manager;
pub mod middleware;
pub mod pii_detector;
pub mod process_helper;
pub mod rag_engine;
pub mod risk_assessment;
pub mod scheduler;
pub mod security;
pub mod system;
pub mod utils;

// Re-export commonly used types
pub use ai_transparency::{RiskLevel, TransparencyContext, TransparencyPreferences};
pub use database::export_integration::ExportIntegration;
pub use export_engine::ExportEngine;
pub use llm_manager::LLMManager;
pub use pii_detector::PIIDetector;
pub use rag_engine::RAGEngine;
pub use risk_assessment::{
    Disclaimer, DisclaimerCategory, LegalSuitability, RiskAssessment, RiskAssessor, SeverityLevel,
};

// SystemStatus - temporary stub for hardware_monitor.rs until main.rs types are refactored
#[derive(Debug, Clone)]
pub struct SystemStatus {
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub gpu_usage: Option<f32>,
    pub temperature: Option<f32>,
    pub is_safe: bool,
}
