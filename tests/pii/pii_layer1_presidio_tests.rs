// Layer 1: Presidio Integration Tests
//
// Tests for Microsoft Presidio NER engine integration
// Covers: PresidioMode (Disabled/SpacyOnly/FullML), fallback behavior, Python detection

use bear_ai_llm::pii_detector::{PIIDetector, PresidioMode, PIIDetectionConfig};

#[tokio::test]
async fn test_presidio_mode_disabled() {
    let detector = PIIDetector::new();
    detector.set_presidio_mode(PresidioMode::Disabled).await.unwrap();
    detector.initialize().await.unwrap();

    let mode = detector.get_presidio_mode().await;
    assert_eq!(mode, PresidioMode::Disabled);

    // Should not use Presidio even if available
    let available = detector.is_presidio_available().await;
    if available {
        let text = "John Smith lives at 123 Main St";
        let entities = detector.detect_pii(text).await.unwrap();

        // All detections should be from regex engine
        for entity in entities {
            assert_ne!(entity.engine, "presidio",
                "PresidioMode::Disabled should not use Presidio engine");
        }
    }
}

#[tokio::test]
async fn test_presidio_mode_spacy_only_memory_overhead() {
    let mode = PresidioMode::SpacyOnly;

    // Test expected memory overhead
    assert_eq!(mode.memory_overhead_mb(), 500);
    assert_eq!(mode.accuracy(), 90);
    assert_eq!(mode.to_string(), "spacy_only");
}

#[tokio::test]
async fn test_presidio_mode_full_ml_memory_overhead() {
    let mode = PresidioMode::FullML;

    // Test expected memory overhead
    assert_eq!(mode.memory_overhead_mb(), 2048);
    assert_eq!(mode.accuracy(), 95);
    assert_eq!(mode.to_string(), "full_ml");
}

#[tokio::test]
async fn test_presidio_mode_from_string() {
    assert_eq!(PresidioMode::from_string("disabled"), PresidioMode::Disabled);
    assert_eq!(PresidioMode::from_string("spacy_only"), PresidioMode::SpacyOnly);
    assert_eq!(PresidioMode::from_string("full_ml"), PresidioMode::FullML);
    assert_eq!(PresidioMode::from_string("SPACY_ONLY"), PresidioMode::SpacyOnly);
    assert_eq!(PresidioMode::from_string("invalid"), PresidioMode::Disabled);
}

#[tokio::test]
async fn test_presidio_availability_check() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let available = detector.is_presidio_available().await;

    // Should complete without panicking
    println!("Presidio available: {}", available);

    if !available {
        println!("⚠️  Presidio not installed - testing fallback behavior");
    }
}

#[tokio::test]
async fn test_presidio_graceful_degradation() {
    let detector = PIIDetector::new();
    detector.set_presidio_mode(PresidioMode::SpacyOnly).await.unwrap();
    detector.initialize().await.unwrap();

    let text = "SSN: 123-45-6789, Email: test@example.com";
    let entities = detector.detect_pii(text).await.unwrap();

    // Should still detect PII even if Presidio unavailable
    assert!(!entities.is_empty(),
        "Should detect PII via regex fallback even without Presidio");

    // Verify at least SSN and EMAIL detected
    let entity_types: Vec<_> = entities.iter()
        .map(|e| e.entity_type.as_str())
        .collect();

    assert!(entity_types.contains(&"SSN") || entity_types.contains(&"EMAIL"),
        "Should detect at least SSN or EMAIL");
}

#[tokio::test]
async fn test_presidio_and_regex_hybrid() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let text = "Contact John Smith at john.smith@example.com or 555-123-4567";
    let entities = detector.detect_pii(text).await.unwrap();

    // Check for multiple detection engines
    let engines: std::collections::HashSet<_> = entities.iter()
        .map(|e| e.engine.as_str())
        .collect();

    println!("Detection engines used: {:?}", engines);

    // Should use at least regex engine
    assert!(engines.contains("regex"),
        "Regex engine should always run as fallback");
}

#[tokio::test]
async fn test_presidio_confidence_scores() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let text = "Patient SSN: 123-45-6789";
    let entities = detector.detect_pii(text).await.unwrap();

    // All entities should have valid confidence scores
    for entity in entities {
        assert!(entity.confidence > 0.0 && entity.confidence <= 1.0,
            "Confidence should be between 0.0 and 1.0, got {}",
            entity.confidence);
    }
}

#[tokio::test]
async fn test_presidio_mode_switch_runtime() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    // Start with disabled
    detector.set_presidio_mode(PresidioMode::Disabled).await.unwrap();
    assert_eq!(detector.get_presidio_mode().await, PresidioMode::Disabled);

    // Switch to SpacyOnly
    detector.set_presidio_mode(PresidioMode::SpacyOnly).await.unwrap();
    assert_eq!(detector.get_presidio_mode().await, PresidioMode::SpacyOnly);

    // Switch to FullML
    detector.set_presidio_mode(PresidioMode::FullML).await.unwrap();
    assert_eq!(detector.get_presidio_mode().await, PresidioMode::FullML);
}

#[tokio::test]
async fn test_presidio_entity_types() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let text = r#"
        Name: John Doe
        SSN: 123-45-6789
        Email: john.doe@example.com
        Phone: (555) 123-4567
        Address: 123 Main St, New York, NY 10001
    "#;

    let entities = detector.detect_pii(text).await.unwrap();

    // Should detect multiple entity types
    let entity_types: std::collections::HashSet<_> = entities.iter()
        .map(|e| e.entity_type.as_str())
        .collect();

    println!("Detected entity types: {:?}", entity_types);

    // Verify comprehensive detection
    assert!(entity_types.len() >= 3,
        "Should detect at least 3 different entity types");
}

#[tokio::test]
async fn test_presidio_python_path_detection() {
    let detector = PIIDetector::new();

    // Initialize should check for Python
    detector.initialize().await.unwrap();

    // Should not panic even if Python not found
    let available = detector.is_presidio_available().await;
    println!("Presidio check completed: available={}", available);
}

#[tokio::test]
async fn test_presidio_error_handling() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    // Even with malformed input, should not panic
    let malformed_inputs = vec![
        "",
        "   ",
        "\n\n\n",
        "12345678901234567890123456789012345678901234567890".repeat(1000),
    ];

    for input in malformed_inputs {
        let result = detector.detect_pii(&input).await;
        assert!(result.is_ok(),
            "Should handle malformed input gracefully: {:?}", input);
    }
}

#[tokio::test]
async fn test_presidio_concurrent_requests() {
    let detector = std::sync::Arc::new(PIIDetector::new());
    detector.initialize().await.unwrap();

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let detector = detector.clone();
            tokio::spawn(async move {
                let text = format!("SSN-{}: 123-45-678{}", i, i);
                detector.detect_pii(&text).await
            })
        })
        .collect();

    // All concurrent requests should succeed
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok(), "Concurrent request should succeed");
    }
}

#[tokio::test]
async fn test_presidio_mode_default() {
    let config = PIIDetectionConfig::default();

    // Default should be Disabled (use built-in detection)
    assert_eq!(config.presidio_mode, PresidioMode::Disabled);
    assert_eq!(config.use_presidio, false); // Deprecated field
}

#[tokio::test]
async fn test_presidio_with_context_enhancement() {
    let mut config = PIIDetectionConfig::default();
    config.use_context_enhancement = true;
    config.presidio_mode = PresidioMode::SpacyOnly;

    let detector = PIIDetector::new();
    detector.update_config(config).await.unwrap();
    detector.initialize().await.unwrap();

    let text = "The plaintiff John Smith testified in court";
    let entities = detector.detect_pii(text).await.unwrap();

    // Names detected in legal context should have boosted confidence
    let person_entities: Vec<_> = entities.iter()
        .filter(|e| e.entity_type == "PERSON")
        .collect();

    if !person_entities.is_empty() {
        // Context enhancement should boost confidence
        assert!(person_entities[0].confidence >= 0.75,
            "Legal context should boost name confidence");
    }
}

#[tokio::test]
async fn test_presidio_performance_small_document() {
    use std::time::Instant;

    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let text = "John Smith, SSN: 123-45-6789, john@example.com";

    let start = Instant::now();
    let _ = detector.detect_pii(text).await.unwrap();
    let duration = start.elapsed();

    // Small documents should be fast (even with Presidio)
    assert!(duration.as_millis() < 1000,
        "Small document detection should complete <1s, took {:?}",
        duration);
}
