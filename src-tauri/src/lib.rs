// Library exports for BEAR AI LLM
// This allows tests and external crates to use the modules

pub mod compliance;
pub mod database;
pub mod export_engine;
pub mod hardware_detection;
pub mod llm_manager;
pub mod pii_detector;
pub mod rag_engine;

// Re-export commonly used types
pub use database::DatabaseManager;
pub use export_engine::ExportEngine;
pub use llm_manager::LLMManager;
pub use pii_detector::PIIDetector;
pub use rag_engine::RAGEngine;
