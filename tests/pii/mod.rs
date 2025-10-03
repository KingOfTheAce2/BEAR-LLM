// PII Detection Test Suite
//
// Comprehensive tests for 3-layer PII detection system:
// - Layer 1: Regex patterns (baseline, always-on)
// - Layer 2: gline-rs (Rust-native ML detection)
// - Layer 3: Microsoft Presidio (optional post-install)
//
// Test Coverage Goals:
// - 90%+ code coverage across all layers
// - Layer interactions and fallback mechanisms
// - Multi-regional exclusions (3,474+ patterns)
// - Performance benchmarks and concurrent operations

mod pii_layer1_presidio_tests;
mod pii_layer2_regex_tests;
mod pii_layer2_gline_tests;
mod pii_integration_tests;
mod pii_exclusions_tests;

// Re-export commonly used test utilities
pub use bear_ai_llm::pii_detector::{
    PIIDetector,
    PIIDetectionConfig,
    PIIEntity,
    PresidioMode,
    DetectionLayer,
};
