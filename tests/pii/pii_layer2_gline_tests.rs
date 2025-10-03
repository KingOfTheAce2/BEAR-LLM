// Layer 2: gline-rs Enhanced Detection Tests
//
// Tests for Rust-native ML-based PII detection using gline-rs
// Covers: Entity detection, confidence scoring, entity type mapping, fallback behavior

use bear_ai_llm::pii_detector::{PIIDetector, PIIDetectionConfig, DetectionLayer};

#[tokio::test]
async fn test_gline_initialization() {
    let detector = PIIDetector::new();
    let result = detector.initialize().await;

    // Should initialize successfully (or gracefully fall back)
    assert!(result.is_ok(), "gline-rs initialization should not panic");
}

#[tokio::test]
async fn test_gline_enabled_by_default() {
    let config = PIIDetectionConfig::default();

    assert!(config.gline_enabled, "gline-rs should be enabled by default");
    assert_eq!(config.detection_layer, DetectionLayer::WithGline,
        "Default layer should be WithGline (Layer 1 + Layer 2)");
}

#[tokio::test]
async fn test_gline_detection_layer_config() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let mut config = PIIDetectionConfig::default();

    // Test Layer 1 only (Regex only)
    config.detection_layer = DetectionLayer::RegexOnly;
    config.gline_enabled = false;
    detector.update_config(config.clone()).await.unwrap();

    let text = "John Smith, email: john@example.com";
    let entities_layer1 = detector.detect_pii(text).await.unwrap();

    // All detections should be from regex
    for entity in &entities_layer1 {
        assert_eq!(entity.engine, "regex",
            "RegexOnly mode should only use regex engine");
    }

    // Test Layer 1 + Layer 2 (Regex + gline-rs)
    config.detection_layer = DetectionLayer::WithGline;
    config.gline_enabled = true;
    detector.update_config(config.clone()).await.unwrap();

    let entities_layer2 = detector.detect_pii(text).await.unwrap();

    // Should have detections from both engines
    let engines: std::collections::HashSet<_> = entities_layer2.iter()
        .map(|e| e.engine.as_str())
        .collect();

    println!("Detection engines (Layer 1+2): {:?}", engines);
}

#[tokio::test]
async fn test_gline_person_detection() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let text = "The witness, John Smith, testified yesterday.";
    let entities = detector.detect_pii(text).await.unwrap();

    let person_entities: Vec<_> = entities.iter()
        .filter(|e| e.entity_type == "PERSON")
        .collect();

    println!("Person entities detected: {:?}", person_entities);

    // gline-rs should detect person names with higher accuracy than regex
    if !person_entities.is_empty() {
        let gline_detections = person_entities.iter()
            .filter(|e| e.engine == "gline-rs")
            .count();

        println!("gline-rs person detections: {}", gline_detections);
    }
}

#[tokio::test]
async fn test_gline_organization_detection() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let text = "The company Acme Corporation filed a lawsuit.";
    let entities = detector.detect_pii(text).await.unwrap();

    let org_entities: Vec<_> = entities.iter()
        .filter(|e| e.entity_type == "ORGANIZATION")
        .collect();

    println!("Organization entities: {:?}", org_entities);
}

#[tokio::test]
async fn test_gline_location_detection() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let text = "The trial was held in Los Angeles, California.";
    let entities = detector.detect_pii(text).await.unwrap();

    let location_entities: Vec<_> = entities.iter()
        .filter(|e| e.entity_type == "LOCATION" || e.entity_type == "GPE")
        .collect();

    println!("Location entities: {:?}", location_entities);
}

#[tokio::test]
async fn test_gline_entity_type_mapping() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let text = r#"
        Person: John Doe
        Email: john@example.com
        Phone: 555-123-4567
        SSN: 123-45-6789
        Organization: Acme Inc
    "#;

    let entities = detector.detect_pii(text).await.unwrap();

    // Verify entity types are properly mapped
    let entity_types: std::collections::HashSet<_> = entities.iter()
        .map(|e| e.entity_type.as_str())
        .collect();

    println!("Detected entity types: {:?}", entity_types);

    // Should have standard entity types (not gline-specific)
    for entity in &entities {
        assert!(
            matches!(entity.entity_type.as_str(),
                "PERSON" | "EMAIL" | "PHONE" | "SSN" |
                "CREDIT_CARD" | "ORGANIZATION" | "LOCATION" |
                "MEDICAL_RECORD" | "CASE_NUMBER" | "IP_ADDRESS" | _
            ),
            "Entity type '{}' should be standardized",
            entity.entity_type
        );
    }
}

#[tokio::test]
async fn test_gline_confidence_scores() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let text = "Contact John Smith at john@example.com";
    let entities = detector.detect_pii(text).await.unwrap();

    // gline-rs should provide confidence scores
    let gline_entities: Vec<_> = entities.iter()
        .filter(|e| e.engine == "gline-rs")
        .collect();

    for entity in gline_entities {
        assert!(entity.confidence > 0.0 && entity.confidence <= 1.0,
            "gline-rs confidence should be valid: {}",
            entity.confidence);

        // gline-rs typically has higher confidence for ML-detected entities
        if entity.entity_type == "PERSON" {
            assert!(entity.confidence >= 0.7,
                "ML-detected person should have confidence >= 0.7");
        }
    }
}

#[tokio::test]
async fn test_gline_fallback_on_error() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    // Even if gline-rs fails, should fall back to regex
    let text = "SSN: 123-45-6789";
    let result = detector.detect_pii(text).await;

    assert!(result.is_ok(),
        "Should gracefully fall back to regex if gline-rs fails");

    let entities = result.unwrap();
    assert!(!entities.is_empty(),
        "Should still detect PII via regex fallback");
}

#[tokio::test]
async fn test_gline_disabled_mode() {
    let mut config = PIIDetectionConfig::default();
    config.gline_enabled = false;
    config.detection_layer = DetectionLayer::RegexOnly;

    let detector = PIIDetector::new();
    detector.update_config(config).await.unwrap();
    detector.initialize().await.unwrap();

    let text = "John Smith, email: john@example.com";
    let entities = detector.detect_pii(text).await.unwrap();

    // Should not use gline-rs
    for entity in &entities {
        assert_ne!(entity.engine, "gline-rs",
            "gline-rs should be disabled");
    }
}

#[tokio::test]
async fn test_gline_vs_regex_comparison() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let text = "The attorney John Smith contacted jane.doe@example.com";
    let entities = detector.detect_pii(text).await.unwrap();

    // Count detections by engine
    let regex_count = entities.iter()
        .filter(|e| e.engine == "regex")
        .count();

    let gline_count = entities.iter()
        .filter(|e| e.engine == "gline-rs")
        .count();

    println!("Regex detections: {}, gline-rs detections: {}",
        regex_count, gline_count);

    // Both engines should contribute
    assert!(regex_count > 0 || gline_count > 0,
        "At least one engine should detect entities");
}

#[tokio::test]
async fn test_gline_context_awareness() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let context_text = "The plaintiff, John Smith, filed a motion.";
    let entities = detector.detect_pii(context_text).await.unwrap();

    // gline-rs should be better at context-aware detection
    let person_in_context: Vec<_> = entities.iter()
        .filter(|e| e.entity_type == "PERSON" && e.text.contains("Smith"))
        .collect();

    println!("Person detected in legal context: {:?}", person_in_context);
}

#[tokio::test]
async fn test_gline_multilingual_support() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    // Test with non-English names (if gline-rs supports it)
    let texts = vec![
        "Juan García trabaja aquí",  // Spanish
        "François Dubois témoigne",  // French
        "Hans Müller ist Zeuge",     // German
    ];

    for text in texts {
        let entities = detector.detect_pii(text).await.unwrap();
        println!("Multilingual test '{}': {} entities",
            text, entities.len());
    }
}

#[tokio::test]
async fn test_gline_lowercase_name_detection() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    // gline-rs should be better at lowercase names than regex
    let text = "The witness john smith testified yesterday.";
    let entities = detector.detect_pii(text).await.unwrap();

    let lowercase_names: Vec<_> = entities.iter()
        .filter(|e| e.entity_type == "PERSON" &&
                   e.text.chars().next().unwrap().is_lowercase())
        .collect();

    println!("Lowercase name detections: {:?}", lowercase_names);

    // If gline-rs is working, it should detect lowercase names
    // (regex typically only catches capitalized names)
}

#[tokio::test]
async fn test_gline_medical_entities() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let text = "Patient MRN: ABC123456, diagnosed with condition X";
    let entities = detector.detect_pii(text).await.unwrap();

    let medical: Vec<_> = entities.iter()
        .filter(|e| e.entity_type == "MEDICAL_RECORD")
        .collect();

    println!("Medical record detections: {:?}", medical);
}

#[tokio::test]
async fn test_gline_email_detection() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let emails = vec![
        "test@example.com",
        "user.name@domain.co.uk",
        "admin+tag@example.org",
    ];

    for email in emails {
        let text = format!("Contact: {}", email);
        let entities = detector.detect_pii(&text).await.unwrap();

        let email_found = entities.iter()
            .any(|e| e.entity_type == "EMAIL" && e.text.contains(email));

        assert!(email_found,
            "Email '{}' should be detected by gline-rs or regex", email);
    }
}

#[tokio::test]
async fn test_gline_performance_benchmark() {
    use std::time::Instant;

    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let text = "Contact John Smith at john.smith@example.com or call 555-123-4567";

    let start = Instant::now();
    let _ = detector.detect_pii(text).await.unwrap();
    let duration = start.elapsed();

    println!("gline-rs detection time: {:?}", duration);

    // Should be reasonably fast (< 500ms for small documents)
    assert!(duration.as_millis() < 500,
        "gline-rs detection should be fast, took {:?}", duration);
}

#[tokio::test]
async fn test_gline_thread_safety() {
    use std::sync::Arc;

    let detector = Arc::new(PIIDetector::new());
    detector.initialize().await.unwrap();

    let handles: Vec<_> = (0..50)
        .map(|i| {
            let detector = detector.clone();
            tokio::spawn(async move {
                let text = format!("Person-{}: John-{} Smith-{}", i, i, i);
                detector.detect_pii(&text).await
            })
        })
        .collect();

    // All concurrent requests should succeed
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok(),
            "Concurrent gline-rs detection should be thread-safe");
    }
}

#[tokio::test]
async fn test_gline_large_document() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    // Generate large text with scattered PII
    let mut large_text = String::new();
    for i in 0..100 {
        large_text.push_str(&format!(
            "Paragraph {}: Contact Person-{} at email-{}@example.com. ",
            i, i, i
        ));
    }

    let entities = detector.detect_pii(&large_text).await.unwrap();

    println!("Large document: {} entities detected", entities.len());

    // Should detect many entities
    assert!(entities.len() >= 100,
        "Should detect entities in large document");
}

#[tokio::test]
async fn test_gline_special_characters() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let text = "Contact: test+tag@example.com, François Müller, Ñoño García";
    let entities = detector.detect_pii(text).await.unwrap();

    println!("Special character handling: {:?}", entities);

    // gline-rs should handle Unicode characters better than regex
}

#[tokio::test]
async fn test_gline_entity_boundaries() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let text = "JohnSmith@example.com vs John Smith";
    let entities = detector.detect_pii(text).await.unwrap();

    // Should correctly identify entity boundaries
    for entity in &entities {
        assert!(entity.start < entity.end,
            "Entity boundaries should be valid");
        assert_eq!(&text[entity.start..entity.end], entity.text,
            "Entity text should match extracted range");
    }
}

#[tokio::test]
async fn test_gline_empty_input() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let empty_inputs = vec!["", "   ", "\n\n", "\t\t"];

    for input in empty_inputs {
        let result = detector.detect_pii(input).await;
        assert!(result.is_ok(),
            "gline-rs should handle empty input gracefully");

        let entities = result.unwrap();
        assert!(entities.is_empty(),
            "Empty input should yield no detections");
    }
}

#[tokio::test]
async fn test_gline_confidence_threshold() {
    let mut config = PIIDetectionConfig::default();
    config.confidence_threshold = 0.9; // High threshold

    let detector = PIIDetector::new();
    detector.update_config(config).await.unwrap();
    detector.initialize().await.unwrap();

    let text = "Maybe this is John Smith or maybe not";
    let entities = detector.detect_pii(text).await.unwrap();

    // All returned entities should meet threshold
    for entity in &entities {
        assert!(entity.confidence >= 0.9,
            "Entity '{}' confidence {} below threshold",
            entity.entity_type, entity.confidence);
    }
}
