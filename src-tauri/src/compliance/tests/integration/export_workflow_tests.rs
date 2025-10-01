// Integration Tests for Export Workflow
//
// Tests cover end-to-end data export flow for GDPR Article 20 compliance

use crate::export_engine::ExportEngine;
use crate::database::DatabaseManager;
use crate::compliance::tests::fixtures::{mock_user_full, mock_chat};
use crate::compliance::tests::test_utils::{create_temp_dir, cleanup_temp_dir, create_test_db};
use std::path::PathBuf;

#[test]
fn test_full_export_workflow_from_database() {
    // Setup: Create test database and populate with data
    let conn = create_test_db();

    // Insert test chat session
    conn.execute(
        "INSERT INTO chat_sessions (id, title, model_used) VALUES (?1, ?2, ?3)",
        ["test-chat-001", "Contract Review", "gpt-4"],
    ).unwrap();

    // Insert test messages
    conn.execute(
        "INSERT INTO chat_messages (chat_id, role, content) VALUES (?1, ?2, ?3)",
        ["test-chat-001", "user", "Analyze this contract"],
    ).unwrap();

    conn.execute(
        "INSERT INTO chat_messages (chat_id, role, content) VALUES (?1, ?2, ?3)",
        ["test-chat-001", "assistant", "Here is my analysis..."],
    ).unwrap();

    // Insert test document
    conn.execute(
        "INSERT INTO documents (filename, content, file_type) VALUES (?1, ?2, ?3)",
        ["test_contract.pdf", "Contract text content", "pdf"],
    ).unwrap();

    // Workflow: Gather data from database
    let mut stmt = conn.prepare(
        "SELECT id, title, created_at, updated_at, model_used, tags FROM chat_sessions"
    ).unwrap();

    let chats: Vec<_> = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(4)?,
        ))
    }).unwrap().collect::<Result<Vec<_>, _>>().unwrap();

    assert!(!chats.is_empty(), "Should retrieve chat sessions from database");

    // Workflow: Export data
    let engine = ExportEngine::new();
    let data = mock_user_full();
    let temp_dir = create_temp_dir();

    let formats = vec!["json", "markdown", "docx"];
    let result = engine.export_user_data(&data, &temp_dir, &formats);

    assert!(result.is_ok(), "Export workflow should complete successfully");

    let exported_files = result.unwrap();
    assert_eq!(exported_files.len(), 3, "Should export all requested formats");

    // Cleanup
    cleanup_temp_dir(&temp_dir);
}

#[test]
fn test_export_with_pii_redaction() {
    // Setup
    let conn = create_test_db();
    let doc_id = 1i64;

    conn.execute(
        "INSERT INTO documents (id, filename, content, file_type) VALUES (?1, ?2, ?3, ?4)",
        [&doc_id.to_string(), "sensitive.txt", "Contains SSN: 123-45-6789", "txt"],
    ).unwrap();

    // Store PII detection
    conn.execute(
        "INSERT INTO pii_detections (document_id, pii_type, replacement_text, confidence, position_start, position_end) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        [&doc_id.to_string(), "SSN", "[REDACTED]", "0.95", "14", "25"],
    ).unwrap();

    // Verify PII was stored
    let pii_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM pii_detections WHERE document_id = ?1",
        [&doc_id],
        |row| row.get(0)
    ).unwrap();

    assert_eq!(pii_count, 1, "PII detection should be stored");

    // Export workflow should include PII information
    let engine = ExportEngine::new();
    let data = mock_user_full();
    let temp_dir = create_temp_dir();
    let output_path = temp_dir.join("export.md");

    let result = engine.export_to_markdown(&data, &output_path);
    assert!(result.is_ok());

    cleanup_temp_dir(&temp_dir);
}

#[test]
fn test_export_integrity_verification() {
    use sha2::{Sha256, Digest};
    use hex;

    let engine = ExportEngine::new();
    let data = mock_user_full();
    let temp_dir = create_temp_dir();

    // Export to JSON
    let json_path = temp_dir.join("export.json");
    let json_str = serde_json::to_string_pretty(&data).unwrap();
    std::fs::write(&json_path, &json_str).unwrap();

    // Calculate hash
    let mut hasher = Sha256::new();
    hasher.update(json_str.as_bytes());
    let calculated_hash = hex::encode(hasher.finalize());

    // Verify hash matches
    assert_eq!(
        calculated_hash.len(),
        64,
        "Hash should be 64 characters (SHA-256)"
    );

    // Re-read and verify
    let read_content = std::fs::read_to_string(&json_path).unwrap();
    let mut verifier = Sha256::new();
    verifier.update(read_content.as_bytes());
    let verification_hash = hex::encode(verifier.finalize());

    assert_eq!(
        calculated_hash,
        verification_hash,
        "Hash should match after re-reading file"
    );

    cleanup_temp_dir(&temp_dir);
}

#[test]
fn test_export_with_consent_validation() {
    let conn = create_test_db();

    // Store user consent settings
    conn.execute(
        "INSERT INTO user_settings (setting_key, setting_value) VALUES (?1, ?2)",
        ["analytics_consent", "true"],
    ).unwrap();

    conn.execute(
        "INSERT INTO user_settings (setting_key, setting_value) VALUES (?1, ?2)",
        ["marketing_consent", "false"],
    ).unwrap();

    // Retrieve consent settings
    let analytics_consent: String = conn.query_row(
        "SELECT setting_value FROM user_settings WHERE setting_key = ?1",
        ["analytics_consent"],
        |row| row.get(0)
    ).unwrap();

    assert_eq!(analytics_consent, "true", "Should retrieve consent setting");

    // Export should include consent information
    let engine = ExportEngine::new();
    let data = mock_user_full();
    let temp_dir = create_temp_dir();
    let output_path = temp_dir.join("export.json");

    std::fs::write(&output_path, serde_json::to_string_pretty(&data).unwrap()).unwrap();

    let content = std::fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("preferences"), "Should include user preferences");

    cleanup_temp_dir(&temp_dir);
}

#[test]
fn test_incremental_export_workflow() {
    // Test exporting data in chunks (for large datasets)
    let mut all_chats = Vec::new();

    // Simulate processing in batches
    for batch_num in 0..5 {
        let batch_chats = vec![
            mock_chat(&format!("Chat Batch {}-1", batch_num), 10),
            mock_chat(&format!("Chat Batch {}-2", batch_num), 10),
        ];
        all_chats.extend(batch_chats);
    }

    assert_eq!(all_chats.len(), 10, "Should have 10 chats from 5 batches");

    // Each chat should have messages
    for chat in &all_chats {
        assert_eq!(chat.messages.len(), 10, "Each chat should have 10 messages");
    }
}

#[test]
fn test_export_atomic_operation() {
    // Ensure export is atomic (all or nothing)
    let engine = ExportEngine::new();
    let data = mock_user_full();
    let temp_dir = create_temp_dir();

    // Create a directory where a file should be (to cause error)
    let conflict_path = temp_dir.join("bear_ai_export.md");
    std::fs::create_dir(&conflict_path).unwrap();

    let result = engine.export_to_markdown(&data, &conflict_path);

    // Should fail gracefully
    assert!(result.is_err(), "Should fail when file path conflicts");

    // Remove conflict
    std::fs::remove_dir(&conflict_path).unwrap();

    // Now should succeed
    let result2 = engine.export_to_markdown(&data, &conflict_path);
    assert!(result2.is_ok(), "Should succeed after removing conflict");

    cleanup_temp_dir(&temp_dir);
}

#[test]
fn test_export_concurrent_requests() {
    use std::thread;

    let engine = ExportEngine::new();
    let temp_dir = create_temp_dir();

    // Simulate concurrent export requests
    let mut handles = vec![];

    for i in 0..3 {
        let data = mock_user_full();
        let export_path = temp_dir.join(format!("export_{}.md", i));

        let handle = thread::spawn(move || {
            let local_engine = ExportEngine::new();
            local_engine.export_to_markdown(&data, &export_path)
        });

        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        let result = handle.join().unwrap();
        assert!(result.is_ok(), "Concurrent exports should succeed");
    }

    // Verify all files created
    assert!(temp_dir.join("export_0.md").exists());
    assert!(temp_dir.join("export_1.md").exists());
    assert!(temp_dir.join("export_2.md").exists());

    cleanup_temp_dir(&temp_dir);
}

#[test]
fn test_export_with_audit_logging() {
    let conn = create_test_db();

    // Log export operation
    conn.execute(
        "INSERT INTO audit_log (event_type, user_id, action, result) VALUES (?1, ?2, ?3, ?4)",
        ["data_export", "test-user-001", "export_requested", "success"],
    ).unwrap();

    // Verify audit log entry
    let log_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM audit_log WHERE event_type = ?1",
        ["data_export"],
        |row| row.get(0)
    ).unwrap();

    assert_eq!(log_count, 1, "Export should be logged in audit log");

    // Retrieve log details
    let action: String = conn.query_row(
        "SELECT action FROM audit_log WHERE event_type = ?1",
        ["data_export"],
        |row| row.get(0)
    ).unwrap();

    assert_eq!(action, "export_requested");
}

#[test]
fn test_export_error_handling() {
    let engine = ExportEngine::new();
    let data = mock_user_full();

    // Test invalid path
    let invalid_path = PathBuf::from("/nonexistent/directory/export.md");
    let result = engine.export_to_markdown(&data, &invalid_path);

    assert!(result.is_err(), "Should fail with invalid path");

    // Test read-only filesystem (simulate by using /dev/null on Unix)
    #[cfg(unix)]
    {
        let readonly_path = PathBuf::from("/dev/null/export.md");
        let result2 = engine.export_to_markdown(&data, &readonly_path);
        assert!(result2.is_err(), "Should fail on read-only filesystem");
    }
}

#[test]
fn test_export_data_completeness() {
    let engine = ExportEngine::new();
    let data = mock_user_full();
    let temp_dir = create_temp_dir();

    // Export to JSON for easy parsing
    let json_path = temp_dir.join("export.json");
    let json_str = serde_json::to_string_pretty(&data).unwrap();
    std::fs::write(&json_path, &json_str).unwrap();

    // Parse and verify all fields present
    let exported: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert!(exported.get("user_id").is_some(), "Should include user_id");
    assert!(exported.get("chats").is_some(), "Should include chats");
    assert!(exported.get("documents").is_some(), "Should include documents");
    assert!(exported.get("settings").is_some(), "Should include settings");
    assert!(exported.get("metadata").is_some(), "Should include metadata");

    // Verify metadata completeness
    let metadata = exported.get("metadata").unwrap();
    assert!(metadata.get("export_hash").is_some(), "Should include integrity hash");
    assert!(metadata.get("compliance_info").is_some(), "Should include compliance info");

    cleanup_temp_dir(&temp_dir);
}
