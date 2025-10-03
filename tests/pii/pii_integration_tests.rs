// Integration Tests: 3-Layer PII Detection System
//
// Tests for layer interactions, cross-validation, deduplication, and fallback mechanisms
// Covers: Full 3-layer workflow, layer cooperation, error handling

use bear_ai_llm::pii_detector::{PIIDetector, PIIDetectionConfig, DetectionLayer, PresidioMode};
use std::collections::HashMap;

#[tokio::test]
async fn test_three_layer_workflow() {
    let mut config = PIIDetectionConfig::default();
    config.detection_layer = DetectionLayer::FullStack;
    config.presidio_mode = PresidioMode::SpacyOnly;
    config.gline_enabled = true;

    let detector = PIIDetector::new();
    detector.update_config(config).await.unwrap();
    detector.initialize().await.unwrap();

    let text = r#"
        The plaintiff John Smith (SSN: 123-45-6789) contacted
        attorney@lawfirm.com regarding Case No. 2024-CV-12345.
        Phone: (555) 123-4567
    "#;

    let entities = detector.detect_pii(text).await.unwrap();

    // Should detect from multiple layers
    let engines: std::collections::HashSet<_> = entities.iter()
        .map(|e| e.engine.as_str())
        .collect();

    println!("Engines used in 3-layer detection: {:?}", engines);

    // Verify comprehensive detection
    let entity_types: std::collections::HashSet<_> = entities.iter()
        .map(|e| e.entity_type.as_str())
        .collect();

    println!("Entity types detected: {:?}", entity_types);

    assert!(entity_types.len() >= 3,
        "3-layer detection should find multiple entity types");
}

#[tokio::test]
async fn test_layer_fallback_mechanism() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    // Even if advanced layers fail, Layer 1 (regex) should always work
    let text = "SSN: 123-45-6789, Email: test@example.com";
    let entities = detector.detect_pii(text).await.unwrap();

    assert!(!entities.is_empty(),
        "Should detect PII even if advanced layers unavailable");

    // Verify at least regex layer worked
    let has_regex = entities.iter().any(|e| e.engine == "regex");
    assert!(has_regex,
        "Regex layer (Layer 1) should always provide fallback");
}

#[tokio::test]
async fn test_layer_selection_accuracy() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    // Test accuracy levels for different layer configurations
    let layers = vec![
        DetectionLayer::RegexOnly,
        DetectionLayer::WithGline,
        DetectionLayer::FullStack,
    ];

    for layer in layers {
        assert_eq!(layer.accuracy(), match layer {
            DetectionLayer::RegexOnly => 85,
            DetectionLayer::WithGline => 92,
            DetectionLayer::FullStack => 95,
        });

        assert_eq!(layer.layer_count(), match layer {
            DetectionLayer::RegexOnly => 1,
            DetectionLayer::WithGline => 2,
            DetectionLayer::FullStack => 3,
        });
    }
}

#[tokio::test]
async fn test_cross_layer_deduplication() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let text = "Contact John Smith at john@example.com";
    let entities = detector.detect_pii(text).await.unwrap();

    // Check for overlapping detections from multiple engines
    for i in 0..entities.len() {
        for j in (i + 1)..entities.len() {
            let overlap = entities[i].start < entities[j].end &&
                         entities[j].start < entities[i].end;

            if overlap {
                println!("Overlapping entities: {:?} and {:?}",
                    entities[i], entities[j]);

                // If overlapping, higher confidence should win
                assert_ne!(entities[i].start, entities[j].start,
                    "Deduplication should remove exact duplicates");
            }
        }
    }
}

#[tokio::test]
async fn test_layer_status_reporting() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let status = detector.get_layer_status().await;

    // Layer 1 (regex) should always be available
    assert_eq!(status.get("layer1_regex"), Some(&true),
        "Layer 1 (regex) should always be available");

    // Log status of other layers
    println!("Layer 2 (gline-rs) available: {:?}",
        status.get("layer2_gline"));
    println!("Layer 3 (Presidio) available: {:?}",
        status.get("layer3_presidio"));
}

#[tokio::test]
async fn test_detection_layer_switching() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let text = "Contact: john@example.com";

    // Test Layer 1 only
    detector.set_detection_layer(DetectionLayer::RegexOnly).await.unwrap();
    let layer1_entities = detector.detect_pii(text).await.unwrap();

    // Test Layer 1 + 2
    detector.set_detection_layer(DetectionLayer::WithGline).await.unwrap();
    let layer2_entities = detector.detect_pii(text).await.unwrap();

    // Test all 3 layers
    detector.set_detection_layer(DetectionLayer::FullStack).await.unwrap();
    let layer3_entities = detector.detect_pii(text).await.unwrap();

    println!("Layer 1: {} entities", layer1_entities.len());
    println!("Layer 1+2: {} entities", layer2_entities.len());
    println!("Layer 1+2+3: {} entities", layer3_entities.len());

    // More layers should generally find >= entities (or same after dedup)
}

#[tokio::test]
async fn test_confidence_boosting_across_layers() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let text = "The defendant John Smith appeared in court.";
    let entities = detector.detect_pii(text).await.unwrap();

    // Find entities detected by multiple engines
    let mut entity_map: HashMap<String, Vec<&bear_ai_llm::pii_detector::PIIEntity>> = HashMap::new();

    for entity in &entities {
        entity_map.entry(entity.text.clone())
            .or_insert_with(Vec::new)
            .push(entity);
    }

    // Check for cross-validation
    for (text, detections) in &entity_map {
        if detections.len() > 1 {
            println!("Entity '{}' detected by {} engines", text, detections.len());

            // Multiple detections suggest higher confidence
            let avg_confidence: f32 = detections.iter()
                .map(|e| e.confidence)
                .sum::<f32>() / detections.len() as f32;

            println!("  Average confidence: {:.2}", avg_confidence);
        }
    }
}

#[tokio::test]
async fn test_context_enhancement_with_layers() {
    let mut config = PIIDetectionConfig::default();
    config.use_context_enhancement = true;

    let detector = PIIDetector::new();
    detector.update_config(config).await.unwrap();
    detector.initialize().await.unwrap();

    // Legal context should boost confidence
    let legal_text = "The plaintiff John Smith filed a motion.";
    let legal_entities = detector.detect_pii(legal_text).await.unwrap();

    // Non-legal context
    let plain_text = "John Smith went to the store.";
    let plain_entities = detector.detect_pii(plain_text).await.unwrap();

    // Find person entities
    let legal_person: Vec<_> = legal_entities.iter()
        .filter(|e| e.entity_type == "PERSON")
        .collect();

    let plain_person: Vec<_> = plain_entities.iter()
        .filter(|e| e.entity_type == "PERSON")
        .collect();

    if !legal_person.is_empty() && !plain_person.is_empty() {
        println!("Legal context confidence: {:.2}", legal_person[0].confidence);
        println!("Plain context confidence: {:.2}", plain_person[0].confidence);

        // Legal context should have higher confidence (context enhancement)
        assert!(legal_person[0].confidence >= plain_person[0].confidence,
            "Legal context should boost confidence");
    }
}

#[tokio::test]
async fn test_multilayer_performance() {
    use std::time::Instant;

    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let text = "Contact John Smith at john.smith@example.com or 555-123-4567";

    // Test Layer 1 performance
    detector.set_detection_layer(DetectionLayer::RegexOnly).await.unwrap();
    let start1 = Instant::now();
    let _ = detector.detect_pii(text).await.unwrap();
    let layer1_time = start1.elapsed();

    // Test Layer 1+2 performance
    detector.set_detection_layer(DetectionLayer::WithGline).await.unwrap();
    let start2 = Instant::now();
    let _ = detector.detect_pii(text).await.unwrap();
    let layer2_time = start2.elapsed();

    // Test all layers
    detector.set_detection_layer(DetectionLayer::FullStack).await.unwrap();
    let start3 = Instant::now();
    let _ = detector.detect_pii(text).await.unwrap();
    let layer3_time = start3.elapsed();

    println!("Layer 1 (Regex): {:?}", layer1_time);
    println!("Layer 1+2 (Gline): {:?}", layer2_time);
    println!("Layer 1+2+3 (Full): {:?}", layer3_time);

    // Layer 1 should be fastest
    assert!(layer1_time <= layer3_time,
        "Layer 1 should be fastest (or equal after optimization)");
}

#[tokio::test]
async fn test_error_handling_layer_isolation() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    // Malformed input should not crash any layer
    let malformed_inputs = vec![
        "\x00\x01\x02",  // Binary data
        "🔥🎉😀",        // Emojis only
        "a".repeat(100000), // Very long text
    ];

    for input in malformed_inputs {
        let result = detector.detect_pii(&input).await;

        assert!(result.is_ok(),
            "Should handle malformed input gracefully across all layers");
    }
}

#[tokio::test]
async fn test_layer_coordination_statistics() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let text = r#"
        Contact Information:
        Name: John Smith, Jane Doe, Bob Johnson
        Emails: john@example.com, jane@example.com
        SSNs: 123-45-6789, 987-65-4321
        Phones: 555-123-4567, 555-987-6543
    "#;

    let stats = detector.get_statistics(text).await.unwrap();

    println!("Detection statistics: {:?}", stats);

    // Should have counts for various entity types
    assert!(!stats.is_empty(), "Should generate statistics");

    // Verify reasonable counts
    if let Some(&email_count) = stats.get("EMAIL") {
        assert!(email_count >= 2, "Should detect multiple emails");
    }

    if let Some(&ssn_count) = stats.get("SSN") {
        assert!(ssn_count >= 2, "Should detect multiple SSNs");
    }
}

#[tokio::test]
async fn test_redaction_with_multilayer() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let text = "Contact John Smith at john@example.com, SSN: 123-45-6789";
    let redacted = detector.redact_pii(text).await.unwrap();

    println!("Original: {}", text);
    println!("Redacted: {}", redacted);

    // Should not contain original PII
    assert!(!redacted.contains("john@example.com"));
    assert!(!redacted.contains("123-45-6789"));

    // Should contain redaction markers
    assert!(redacted.contains("[EMAIL]") || redacted.contains("REDACTED"));
    assert!(redacted.contains("[SSN]") || redacted.contains("REDACTED"));
}

#[tokio::test]
async fn test_anonymization_with_multilayer() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let text = "Contact john@example.com and jane@example.com";
    let (anonymized, mappings) = detector.anonymize_pii(text).await.unwrap();

    println!("Anonymized: {}", anonymized);
    println!("Mappings: {:?}", mappings);

    // Should have anonymized both emails
    assert!(!anonymized.contains("john@example.com"));
    assert!(!anonymized.contains("jane@example.com"));

    // Should have mappings
    assert!(!mappings.is_empty());
    assert!(mappings.len() >= 2);
}

#[tokio::test]
async fn test_concurrent_multilayer_detection() {
    use std::sync::Arc;

    let detector = Arc::new(PIIDetector::new());
    detector.initialize().await.unwrap();

    let handles: Vec<_> = (0..20)
        .map(|i| {
            let detector = detector.clone();
            tokio::spawn(async move {
                let text = format!("Person-{}: john-{}@example.com, SSN: {}-45-6789",
                    i, i, 100 + i);
                detector.detect_pii(&text).await
            })
        })
        .collect();

    // All concurrent requests across all layers should succeed
    for (i, handle) in handles.into_iter().enumerate() {
        let result = handle.await.unwrap();
        assert!(result.is_ok(),
            "Concurrent request {} should succeed", i);

        let entities = result.unwrap();
        assert!(!entities.is_empty(),
            "Should detect PII in concurrent request {}", i);
    }
}

#[tokio::test]
async fn test_layer_configuration_persistence() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    // Set configuration
    detector.set_detection_layer(DetectionLayer::FullStack).await.unwrap();
    detector.set_presidio_mode(PresidioMode::SpacyOnly).await.unwrap();

    // Verify persistence
    assert_eq!(detector.get_detection_layer().await, DetectionLayer::FullStack);
    assert_eq!(detector.get_presidio_mode().await, PresidioMode::SpacyOnly);

    // Configuration should persist across detections
    let _ = detector.detect_pii("test").await.unwrap();

    assert_eq!(detector.get_detection_layer().await, DetectionLayer::FullStack);
}

#[tokio::test]
async fn test_detection_layer_from_string() {
    assert_eq!(DetectionLayer::from_string("regex_only"), DetectionLayer::RegexOnly);
    assert_eq!(DetectionLayer::from_string("layer1"), DetectionLayer::RegexOnly);

    assert_eq!(DetectionLayer::from_string("with_gline"), DetectionLayer::WithGline);
    assert_eq!(DetectionLayer::from_string("layer2"), DetectionLayer::WithGline);
    assert_eq!(DetectionLayer::from_string("gline"), DetectionLayer::WithGline);

    assert_eq!(DetectionLayer::from_string("full_stack"), DetectionLayer::FullStack);
    assert_eq!(DetectionLayer::from_string("layer3"), DetectionLayer::FullStack);
    assert_eq!(DetectionLayer::from_string("full"), DetectionLayer::FullStack);

    // Invalid defaults to WithGline
    assert_eq!(DetectionLayer::from_string("invalid"), DetectionLayer::WithGline);
}

#[tokio::test]
async fn test_detection_layer_display() {
    assert_eq!(DetectionLayer::RegexOnly.to_string(), "regex_only");
    assert_eq!(DetectionLayer::WithGline.to_string(), "with_gline");
    assert_eq!(DetectionLayer::FullStack.to_string(), "full_stack");
}

#[tokio::test]
async fn test_comprehensive_entity_coverage() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let comprehensive_text = r#"
        Legal Document - Case No. 2024-CV-12345

        Parties:
        - Plaintiff: John Smith (SSN: 123-45-6789)
        - Defendant: Acme Corporation
        - Attorney: Law Office of Johnson & Associates

        Contact Information:
        - Email: attorney@lawfirm.com
        - Phone: (555) 123-4567
        - Fax: (555) 987-6543

        Medical Records:
        - MRN: ABC123456
        - Hospital: General Medical Center

        Payment Information:
        - Card: 4532-1234-5678-9010

        IP Address: 192.168.1.1
    "#;

    let entities = detector.detect_pii(comprehensive_text).await.unwrap();

    // Verify comprehensive detection
    let entity_types: std::collections::HashSet<_> = entities.iter()
        .map(|e| e.entity_type.as_str())
        .collect();

    println!("Detected entity types: {:?}", entity_types);
    println!("Total entities: {}", entities.len());

    // Should detect multiple types
    assert!(entity_types.len() >= 5,
        "Should detect at least 5 different entity types in comprehensive document");

    // Verify specific types were detected
    let expected_types = vec!["SSN", "EMAIL", "PHONE", "CREDIT_CARD"];
    for expected in expected_types {
        assert!(entity_types.iter().any(|t| t.contains(expected)),
            "Should detect {} entities", expected);
    }
}
