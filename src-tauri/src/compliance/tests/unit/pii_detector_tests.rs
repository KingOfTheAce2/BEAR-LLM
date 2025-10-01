// Unit Tests for PII Detector
//
// Tests cover PII detection accuracy, redaction, and GDPR privacy compliance

use crate::pii_detector::{PIIDetector, PIIDetectionConfig, PIIEntity};
use crate::compliance::tests::fixtures::{mock_text_with_pii, mock_text_without_pii};

#[tokio::test]
async fn test_pii_detector_initialization() {
    let detector = PIIDetector::new();
    let result = detector.initialize().await;

    assert!(result.is_ok(), "PII detector should initialize successfully");
}

#[tokio::test]
async fn test_detect_ssn() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let text = "Patient SSN: 123-45-6789";
    let entities = detector.detect_pii(text).await.unwrap();

    let ssn_entities: Vec<&PIIEntity> = entities.iter()
        .filter(|e| e.entity_type == "SSN")
        .collect();

    assert!(!ssn_entities.is_empty(), "Should detect SSN");
    assert_eq!(ssn_entities[0].text, "123-45-6789");
    assert!(ssn_entities[0].confidence >= 0.9, "SSN confidence should be high");
}

#[tokio::test]
async fn test_detect_email() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let text = "Contact: john.doe@example.com";
    let entities = detector.detect_pii(text).await.unwrap();

    let email_entities: Vec<&PIIEntity> = entities.iter()
        .filter(|e| e.entity_type == "EMAIL")
        .collect();

    assert!(!email_entities.is_empty(), "Should detect email");
    assert_eq!(email_entities[0].text, "john.doe@example.com");
}

#[tokio::test]
async fn test_detect_phone() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let text = "Call me at +1 (555) 123-4567";
    let entities = detector.detect_pii(text).await.unwrap();

    let phone_entities: Vec<&PIIEntity> = entities.iter()
        .filter(|e| e.entity_type == "PHONE")
        .collect();

    assert!(!phone_entities.is_empty(), "Should detect phone number");
    assert!(phone_entities[0].text.contains("555"));
}

#[tokio::test]
async fn test_detect_credit_card() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let text = "Card: 4532-1234-5678-9010"; // Valid Luhn
    let entities = detector.detect_pii(text).await.unwrap();

    let cc_entities: Vec<&PIIEntity> = entities.iter()
        .filter(|e| e.entity_type == "CREDIT_CARD")
        .collect();

    assert!(!cc_entities.is_empty(), "Should detect valid credit card");
    assert!(cc_entities[0].confidence >= 0.9);
}

#[tokio::test]
async fn test_detect_medical_record() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let text = "Medical Record Number: ABC123456";
    let entities = detector.detect_pii(text).await.unwrap();

    let mrn_entities: Vec<&PIIEntity> = entities.iter()
        .filter(|e| e.entity_type == "MEDICAL_RECORD")
        .collect();

    assert!(!mrn_entities.is_empty(), "Should detect medical record number");
}

#[tokio::test]
async fn test_detect_case_number() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let text = "Case Number: 2024-CV-001234";
    let entities = detector.detect_pii(text).await.unwrap();

    let case_entities: Vec<&PIIEntity> = entities.iter()
        .filter(|e| e.entity_type == "CASE_NUMBER")
        .collect();

    assert!(!case_entities.is_empty(), "Should detect legal case number");
}

#[tokio::test]
async fn test_detect_multiple_pii_types() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let text = mock_text_with_pii();
    let entities = detector.detect_pii(&text).await.unwrap();

    // Should detect various PII types
    let pii_types: std::collections::HashSet<_> = entities.iter()
        .map(|e| e.entity_type.clone())
        .collect();

    assert!(pii_types.len() >= 4, "Should detect multiple PII types, found: {:?}", pii_types);
    assert!(pii_types.contains("EMAIL"));
    assert!(pii_types.contains("SSN"));
}

#[tokio::test]
async fn test_no_false_positives() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let text = mock_text_without_pii();
    let entities = detector.detect_pii(&text).await.unwrap();

    // Should not detect locations and legal terms as PII
    let false_positives: Vec<&PIIEntity> = entities.iter()
        .filter(|e| {
            e.text.contains("United States") ||
            e.text.contains("New York") ||
            e.text.contains("Supreme Court") ||
            e.text.contains("First Amendment")
        })
        .collect();

    assert!(
        false_positives.is_empty(),
        "Should not detect common legal terms as PII: {:?}",
        false_positives
    );
}

#[tokio::test]
async fn test_redact_pii() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let text = "Email: test@example.com, SSN: 123-45-6789";
    let redacted = detector.redact_pii(text).await.unwrap();

    assert!(!redacted.contains("test@example.com"), "Email should be redacted");
    assert!(!redacted.contains("123-45-6789"), "SSN should be redacted");
    assert!(redacted.contains("[EMAIL]") || redacted.contains("REDACTED"),
           "Should contain redaction markers");
}

#[tokio::test]
async fn test_anonymize_pii() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let text = "Contact: john@example.com and jane@example.com";
    let (anonymized, mappings) = detector.anonymize_pii(text).await.unwrap();

    assert!(!anonymized.contains("john@example.com"));
    assert!(!anonymized.contains("jane@example.com"));
    assert!(!mappings.is_empty(), "Should create anonymization mappings");
}

#[tokio::test]
async fn test_pii_detection_confidence_threshold() {
    let mut config = PIIDetectionConfig::default();
    config.confidence_threshold = 0.95;

    let detector = PIIDetector::new();
    detector.update_config(config).await.unwrap();
    detector.initialize().await.unwrap();

    let text = mock_text_with_pii();
    let entities = detector.detect_pii(&text).await.unwrap();

    // All returned entities should meet threshold
    for entity in &entities {
        assert!(
            entity.confidence >= 0.95,
            "Entity {} has confidence {} below threshold",
            entity.entity_type,
            entity.confidence
        );
    }
}

#[tokio::test]
async fn test_pii_detection_custom_pattern() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    // Add custom pattern for employee IDs
    detector.add_custom_pattern(
        "EMPLOYEE_ID".to_string(),
        r"EMP-\d{6}".to_string()
    ).await.unwrap();

    let text = "Employee ID: EMP-123456";
    let entities = detector.detect_pii(text).await.unwrap();

    let emp_entities: Vec<&PIIEntity> = entities.iter()
        .filter(|e| e.entity_type == "EMPLOYEE_ID")
        .collect();

    assert!(!emp_entities.is_empty(), "Should detect custom pattern");
    assert_eq!(emp_entities[0].text, "EMP-123456");
}

#[tokio::test]
async fn test_pii_statistics() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let text = mock_text_with_pii();
    let stats = detector.get_statistics(&text).await.unwrap();

    assert!(!stats.is_empty(), "Should return statistics");
    assert!(stats.values().sum::<usize>() > 0, "Should find some PII");
}

#[tokio::test]
async fn test_context_enhancement() {
    let mut config = PIIDetectionConfig::default();
    config.use_context_enhancement = true;

    let detector = PIIDetector::new();
    detector.update_config(config).await.unwrap();
    detector.initialize().await.unwrap();

    let text = "The plaintiff John Smith testified...";
    let entities = detector.detect_pii(text).await.unwrap();

    let name_entities: Vec<&PIIEntity> = entities.iter()
        .filter(|e| e.entity_type == "PERSON")
        .collect();

    if !name_entities.is_empty() {
        // Context should boost confidence when legal terms are nearby
        assert!(name_entities[0].confidence > 0.7, "Legal context should boost confidence");
    }
}

#[tokio::test]
async fn test_deduplication() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    // Text with overlapping patterns
    let text = "john.smith@example.com is john.smith@example.com";
    let entities = detector.detect_pii(text).await.unwrap();

    // Should deduplicate identical detections
    let email_entities: Vec<&PIIEntity> = entities.iter()
        .filter(|e| e.entity_type == "EMAIL")
        .collect();

    // Count unique positions
    let unique_positions: std::collections::HashSet<_> = email_entities.iter()
        .map(|e| (e.start, e.end))
        .collect();

    assert_eq!(
        unique_positions.len(),
        2,
        "Should detect both instances as separate entities"
    );
}

#[tokio::test]
async fn test_empty_text() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let entities = detector.detect_pii("").await.unwrap();
    assert!(entities.is_empty(), "Should handle empty text gracefully");
}

#[tokio::test]
async fn test_very_long_text() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let long_text = "Some text. ".repeat(10000) + "Email: test@example.com";
    let entities = detector.detect_pii(&long_text).await.unwrap();

    assert!(!entities.is_empty(), "Should detect PII in long text");
}

#[tokio::test]
async fn test_special_characters_in_pii() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let text = "Email: test+tag@example.com, Phone: +1-555-123-4567";
    let entities = detector.detect_pii(text).await.unwrap();

    let email_count = entities.iter().filter(|e| e.entity_type == "EMAIL").count();
    let phone_count = entities.iter().filter(|e| e.entity_type == "PHONE").count();

    assert!(email_count > 0, "Should detect email with special chars");
    assert!(phone_count > 0, "Should detect phone with formatting");
}

#[tokio::test]
async fn test_luhn_validation() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    // Valid Luhn checksum
    let valid = "4532-1234-5678-9010";
    let invalid = "4532-1234-5678-9011";

    let entities_valid = detector.detect_pii(&format!("Card: {}", valid)).await.unwrap();
    let entities_invalid = detector.detect_pii(&format!("Card: {}", invalid)).await.unwrap();

    let valid_cc = entities_valid.iter().filter(|e| e.entity_type == "CREDIT_CARD").count();
    let invalid_cc = entities_invalid.iter().filter(|e| e.entity_type == "CREDIT_CARD").count();

    assert!(valid_cc > 0, "Should detect valid credit card");
    assert_eq!(invalid_cc, 0, "Should reject invalid Luhn checksum");
}

#[tokio::test]
async fn test_presidio_fallback() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let presidio_available = detector.is_presidio_available().await;

    if !presidio_available {
        // Should still work with built-in detection
        let text = "SSN: 123-45-6789";
        let entities = detector.detect_pii(text).await.unwrap();

        assert!(!entities.is_empty(), "Built-in detection should work without Presidio");
    }
}

#[tokio::test]
async fn test_performance_benchmark() {
    use std::time::Instant;

    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let text = mock_text_with_pii();

    let start = Instant::now();
    let _ = detector.detect_pii(&text).await.unwrap();
    let duration = start.elapsed();

    assert!(
        duration.as_millis() < 500,
        "PII detection should complete within 500ms, took {:?}",
        duration
    );
}
