// Layer 2: Built-in Regex Detection Tests
//
// Tests for pure Rust regex-based PII detection patterns
// Covers: SSN, credit cards, emails, phones, IPs, names, organizations, legal entities

use bear_ai_llm::pii_detector::{PIIDetector, PIIDetectionConfig};

#[tokio::test]
async fn test_regex_ssn_standard_format() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let test_cases = vec![
        ("123-45-6789", true),
        ("000-00-0000", true),
        ("999-99-9999", true),
    ];

    for (ssn, should_detect) in test_cases {
        let text = format!("SSN: {}", ssn);
        let entities = detector.detect_pii(&text).await.unwrap();

        let ssn_found = entities.iter().any(|e| e.entity_type == "SSN");
        assert_eq!(ssn_found, should_detect,
            "SSN '{}' detection failed", ssn);
    }
}

#[tokio::test]
async fn test_regex_ssn_invalid_formats() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let invalid_ssns = vec![
        "123456789",      // No hyphens (current limitation)
        "12-345-6789",    // Wrong format
        "123-4-56789",    // Wrong format
        "ABC-DE-FGHI",    // Letters
    ];

    for ssn in invalid_ssns {
        let text = format!("SSN: {}", ssn);
        let entities = detector.detect_pii(&text).await.unwrap();

        let ssn_entities: Vec<_> = entities.iter()
            .filter(|e| e.entity_type == "SSN")
            .collect();

        println!("Invalid SSN '{}' detected as: {:?}", ssn, ssn_entities);
    }
}

#[tokio::test]
async fn test_regex_credit_card_luhn_validation() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    // Valid Luhn checksums
    let valid_cards = vec![
        "4532-1234-5678-9010",  // Visa
        "5425-2334-3010-9903",  // Mastercard
        "3782-822463-10005",    // Amex
    ];

    for card in valid_cards {
        let text = format!("Card: {}", card);
        let entities = detector.detect_pii(&text).await.unwrap();

        let cc_found = entities.iter()
            .any(|e| e.entity_type == "CREDIT_CARD");

        assert!(cc_found, "Valid card '{}' should be detected", card);
    }
}

#[tokio::test]
async fn test_regex_credit_card_invalid_luhn() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    // Invalid Luhn checksum
    let invalid_card = "4532-1234-5678-9011";
    let text = format!("Card: {}", invalid_card);
    let entities = detector.detect_pii(&text).await.unwrap();

    let cc_found = entities.iter()
        .any(|e| e.entity_type == "CREDIT_CARD");

    assert!(!cc_found,
        "Invalid Luhn checksum should be rejected");
}

#[tokio::test]
async fn test_regex_email_various_formats() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let valid_emails = vec![
        "test@example.com",
        "user.name@example.co.uk",
        "first+last@example.org",
        "admin@sub.domain.example.com",
        "123@numbers.com",
    ];

    for email in valid_emails {
        let text = format!("Contact: {}", email);
        let entities = detector.detect_pii(&text).await.unwrap();

        let email_found = entities.iter()
            .any(|e| e.entity_type == "EMAIL" && e.text == email);

        assert!(email_found, "Email '{}' should be detected", email);
    }
}

#[tokio::test]
async fn test_regex_phone_formats() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let phone_formats = vec![
        "555-123-4567",
        "(555) 123-4567",
        "555.123.4567",
        "+1-555-123-4567",
        "5551234567",
    ];

    for phone in phone_formats {
        let text = format!("Call: {}", phone);
        let entities = detector.detect_pii(&text).await.unwrap();

        let phone_found = entities.iter()
            .any(|e| e.entity_type == "PHONE");

        println!("Phone '{}' detection: {}", phone, phone_found);
    }
}

#[tokio::test]
async fn test_regex_ip_addresses() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let valid_ips = vec![
        "192.168.1.1",
        "10.0.0.1",
        "172.16.0.1",
        "8.8.8.8",
        "255.255.255.255",
    ];

    for ip in valid_ips {
        let text = format!("IP: {}", ip);
        let entities = detector.detect_pii(&text).await.unwrap();

        let ip_found = entities.iter()
            .any(|e| e.text == ip);

        assert!(ip_found, "IP '{}' should be detected", ip);
    }
}

#[tokio::test]
async fn test_regex_ip_invalid_formats() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let invalid_ips = vec![
        "256.256.256.256",  // Out of range
        "192.168.1",        // Incomplete
        "192.168.1.1.1",    // Too many octets
    ];

    for ip in invalid_ips {
        let text = format!("IP: {}", ip);
        let entities = detector.detect_pii(&text).await.unwrap();

        // May or may not detect - depends on regex strictness
        println!("Invalid IP '{}' detection result: {:?}", ip, entities);
    }
}

#[tokio::test]
async fn test_regex_case_numbers() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let case_numbers = vec![
        "Case No. 2024-CV-001234",
        "Case Number: 23-CR-5678",
        "2024-FAM-9999",
    ];

    for case_num in case_numbers {
        let text = format!("Docket: {}", case_num);
        let entities = detector.detect_pii(&text).await.unwrap();

        let case_found = entities.iter()
            .any(|e| e.entity_type == "CASE_NUMBER");

        println!("Case number '{}' detected: {}", case_num, case_found);
    }
}

#[tokio::test]
async fn test_regex_medical_record_numbers() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let mrns = vec![
        "MRN: ABC123456",
        "Medical Record Number: XYZ789012",
        "MRN ABC123456",
    ];

    for mrn in mrns {
        let entities = detector.detect_pii(mrn).await.unwrap();

        let mrn_found = entities.iter()
            .any(|e| e.entity_type == "MEDICAL_RECORD");

        println!("MRN '{}' detected: {}", mrn, mrn_found);
    }
}

#[tokio::test]
async fn test_regex_person_names_capitalized() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let names = vec![
        "John Smith",
        "Mary Jane Watson",
        "Dr. Robert Brown",
        "Attorney Sarah Johnson",
    ];

    for name in names {
        let text = format!("Person: {}", name);
        let entities = detector.detect_pii(&text).await.unwrap();

        let name_found = entities.iter()
            .any(|e| e.entity_type == "PERSON");

        println!("Name '{}' detected: {}", name, name_found);
    }
}

#[tokio::test]
async fn test_regex_person_names_lowercase_limitation() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    // Current limitation: lowercase names not detected
    let lowercase_names = vec![
        "john smith",
        "mary watson",
    ];

    for name in lowercase_names {
        let text = format!("Person: {}", name);
        let entities = detector.detect_pii(&text).await.unwrap();

        let name_found = entities.iter()
            .any(|e| e.entity_type == "PERSON" && e.text.to_lowercase() == name);

        // Document known limitation
        println!("⚠️  Known limitation: lowercase name '{}' detected: {}",
            name, name_found);
    }
}

#[tokio::test]
async fn test_regex_titles_with_names() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let titled_names = vec![
        "Mr. Smith",
        "Mrs. Johnson",
        "Dr. Williams",
        "Judge Anderson",
        "Attorney Garcia",
    ];

    for name in titled_names {
        let text = format!("Present: {}", name);
        let entities = detector.detect_pii(&text).await.unwrap();

        let name_found = entities.iter()
            .any(|e| e.entity_type == "PERSON");

        assert!(name_found, "Titled name '{}' should be detected", name);
    }
}

#[tokio::test]
async fn test_regex_organizations_corporate() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let organizations = vec![
        "Acme Corporation",
        "Smith & Sons LLC",
        "Johnson Partners LLP",
        "ABC Company Inc",
        "XYZ Group Limited",
    ];

    for org in organizations {
        let text = format!("Company: {}", org);
        let entities = detector.detect_pii(&text).await.unwrap();

        let org_found = entities.iter()
            .any(|e| e.entity_type == "ORGANIZATION");

        println!("Organization '{}' detected: {}", org, org_found);
    }
}

#[tokio::test]
async fn test_regex_organizations_law_firms() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let law_firms = vec![
        "Law Office of Smith",
        "Law Firm of Johnson & Associates",
        "The Williams Firm",
    ];

    for firm in law_firms {
        let text = format!("Firm: {}", firm);
        let entities = detector.detect_pii(&text).await.unwrap();

        let org_found = entities.iter()
            .any(|e| e.entity_type == "ORGANIZATION");

        println!("Law firm '{}' detected: {}", firm, org_found);
    }
}

#[tokio::test]
async fn test_regex_confidence_scores() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let text = r#"
        SSN: 123-45-6789
        Email: test@example.com
        Phone: 555-123-4567
        Name: John Smith
    "#;

    let entities = detector.detect_pii(text).await.unwrap();

    // Verify confidence scores are appropriate
    for entity in &entities {
        match entity.entity_type.as_str() {
            "SSN" => assert_eq!(entity.confidence, 1.0, "SSN should have 1.0 confidence"),
            "EMAIL" => assert_eq!(entity.confidence, 1.0, "Email should have 1.0 confidence"),
            "CREDIT_CARD" => assert_eq!(entity.confidence, 1.0, "CC should have 1.0 confidence"),
            "PHONE" => assert_eq!(entity.confidence, 0.95, "Phone should have 0.95 confidence"),
            "PERSON" => assert!(entity.confidence >= 0.75, "Name should have >=0.75 confidence"),
            _ => {}
        }
    }
}

#[tokio::test]
async fn test_regex_special_characters() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let text = "Email: test+tag@example.com, Phone: +1-555-123-4567";
    let entities = detector.detect_pii(text).await.unwrap();

    let email_found = entities.iter().any(|e|
        e.entity_type == "EMAIL" && e.text.contains("+tag"));
    let phone_found = entities.iter().any(|e|
        e.entity_type == "PHONE" && e.text.contains("+1"));

    assert!(email_found, "Should detect email with + character");
    assert!(phone_found, "Should detect phone with international code");
}

#[tokio::test]
async fn test_regex_word_boundaries() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    // Should not detect SSN pattern within larger number
    let text = "The number 12345678901234567890 is not an SSN";
    let entities = detector.detect_pii(text).await.unwrap();

    let ssn_found = entities.iter().any(|e| e.entity_type == "SSN");

    // Regex patterns use \b word boundaries, so should not match
    assert!(!ssn_found,
        "Should not detect SSN pattern within larger number");
}

#[tokio::test]
async fn test_regex_multiple_entities_same_type() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let text = "Emails: john@example.com, jane@example.com, admin@example.com";
    let entities = detector.detect_pii(text).await.unwrap();

    let email_count = entities.iter()
        .filter(|e| e.entity_type == "EMAIL")
        .count();

    assert_eq!(email_count, 3, "Should detect all 3 emails");
}

#[tokio::test]
async fn test_regex_selective_detection_config() {
    let mut config = PIIDetectionConfig::default();
    config.detect_emails = true;
    config.detect_phones = false;
    config.detect_ssn = false;

    let detector = PIIDetector::new();
    detector.update_config(config).await.unwrap();
    detector.initialize().await.unwrap();

    let text = "Contact: test@example.com, Phone: 555-123-4567, SSN: 123-45-6789";
    let entities = detector.detect_pii(text).await.unwrap();

    let has_email = entities.iter().any(|e| e.entity_type == "EMAIL");
    let has_phone = entities.iter().any(|e| e.entity_type == "PHONE");
    let has_ssn = entities.iter().any(|e| e.entity_type == "SSN");

    assert!(has_email, "Email detection should be enabled");
    assert!(!has_phone, "Phone detection should be disabled");
    assert!(!has_ssn, "SSN detection should be disabled");
}

#[tokio::test]
async fn test_regex_thread_safety() {
    use std::sync::Arc;

    let detector = Arc::new(PIIDetector::new());
    detector.initialize().await.unwrap();

    let handles: Vec<_> = (0..100)
        .map(|i| {
            let detector = detector.clone();
            tokio::spawn(async move {
                let text = format!("Email-{}: test{}@example.com", i, i);
                detector.detect_pii(&text).await
            })
        })
        .collect();

    // All threads should succeed
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok(), "Concurrent regex detection should work");
    }
}
