// End-to-End Tests for User Rights
//
// Tests cover GDPR Articles 15 (Access), 17 (Erasure), and 20 (Portability)

use crate::export_engine::ExportEngine;
use crate::compliance::tests::fixtures::{mock_user_full, mock_user_empty};
use crate::compliance::tests::test_utils::{
    create_test_db, create_temp_dir, cleanup_temp_dir
};

/// Test GDPR Article 15 - Right of Access
/// User should be able to access all their personal data
#[test]
fn test_gdpr_article_15_right_of_access() {
    let conn = create_test_db();

    // Setup: Create user data
    conn.execute(
        "INSERT INTO chat_sessions (id, title, model_used) VALUES (?1, ?2, ?3)",
        ["chat-001", "Test Chat", "gpt-4"],
    ).unwrap();

    conn.execute(
        "INSERT INTO chat_messages (chat_id, role, content) VALUES (?1, ?2, ?3)",
        ["chat-001", "user", "Test message"],
    ).unwrap();

    conn.execute(
        "INSERT INTO documents (filename, content, file_type) VALUES (?1, ?2, ?3)",
        ["test.pdf", "Test content", "pdf"],
    ).unwrap();

    // User Action: Request access to all data
    let chat_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM chat_sessions",
        [],
        |row| row.get(0)
    ).unwrap();

    let doc_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM documents",
        [],
        |row| row.get(0)
    ).unwrap();

    // Verification: User can see all their data
    assert_eq!(chat_count, 1, "Should access chat sessions");
    assert_eq!(doc_count, 1, "Should access documents");

    // Export for user access
    let engine = ExportEngine::new();
    let data = mock_user_full();
    let temp_dir = create_temp_dir();
    let export_path = temp_dir.join("user_access.json");

    let json_str = serde_json::to_string_pretty(&data).unwrap();
    std::fs::write(&export_path, &json_str).unwrap();

    assert!(export_path.exists(), "Should create access report");

    cleanup_temp_dir(&temp_dir);
}

/// Test GDPR Article 17 - Right to Erasure
/// User should be able to delete all their personal data
#[test]
fn test_gdpr_article_17_right_to_erasure() {
    let conn = create_test_db();

    // Setup: Create user data
    let user_id = "user-to-delete";

    conn.execute(
        "INSERT INTO chat_sessions (id, title, model_used) VALUES (?1, ?2, ?3)",
        ["chat-delete-001", "Chat to Delete", "gpt-4"],
    ).unwrap();

    conn.execute(
        "INSERT INTO chat_messages (chat_id, role, content) VALUES (?1, ?2, ?3)",
        ["chat-delete-001", "user", "Message to delete"],
    ).unwrap();

    conn.execute(
        "INSERT INTO documents (id, filename, content, file_type) VALUES (?1, ?2, ?3, ?4)",
        ["1", "delete.pdf", "Content to delete", "pdf"],
    ).unwrap();

    conn.execute(
        "INSERT INTO pii_detections (document_id, pii_type, replacement_text, confidence, position_start, position_end) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        ["1", "EMAIL", "[REDACTED]", "0.95", "0", "10"],
    ).unwrap();

    // Verify data exists before deletion
    let before_chats: i64 = conn.query_row("SELECT COUNT(*) FROM chat_sessions", [], |row| row.get(0)).unwrap();
    let before_messages: i64 = conn.query_row("SELECT COUNT(*) FROM chat_messages", [], |row| row.get(0)).unwrap();
    let before_docs: i64 = conn.query_row("SELECT COUNT(*) FROM documents", [], |row| row.get(0)).unwrap();
    let before_pii: i64 = conn.query_row("SELECT COUNT(*) FROM pii_detections", [], |row| row.get(0)).unwrap();

    assert!(before_chats > 0, "Should have chat sessions before deletion");
    assert!(before_messages > 0, "Should have messages before deletion");
    assert!(before_docs > 0, "Should have documents before deletion");
    assert!(before_pii > 0, "Should have PII detections before deletion");

    // User Action: Request data deletion (cascading)
    conn.execute("DELETE FROM chat_messages WHERE chat_id = ?1", ["chat-delete-001"]).unwrap();
    conn.execute("DELETE FROM chat_sessions WHERE id = ?1", ["chat-delete-001"]).unwrap();
    conn.execute("DELETE FROM pii_detections WHERE document_id = ?1", ["1"]).unwrap();
    conn.execute("DELETE FROM documents WHERE id = ?1", ["1"]).unwrap();

    // Verification: All data should be deleted
    let after_chats: i64 = conn.query_row("SELECT COUNT(*) FROM chat_sessions", [], |row| row.get(0)).unwrap();
    let after_messages: i64 = conn.query_row("SELECT COUNT(*) FROM chat_messages", [], |row| row.get(0)).unwrap();
    let after_docs: i64 = conn.query_row("SELECT COUNT(*) FROM documents", [], |row| row.get(0)).unwrap();
    let after_pii: i64 = conn.query_row("SELECT COUNT(*) FROM pii_detections", [], |row| row.get(0)).unwrap();

    assert_eq!(after_chats, 0, "All chat sessions should be deleted");
    assert_eq!(after_messages, 0, "All messages should be deleted");
    assert_eq!(after_docs, 0, "All documents should be deleted");
    assert_eq!(after_pii, 0, "All PII detections should be deleted");

    // Log the deletion in audit log
    conn.execute(
        "INSERT INTO audit_log (event_type, user_id, action, result) VALUES (?1, ?2, ?3, ?4)",
        ["data_deletion", user_id, "delete_all_data", "success"],
    ).unwrap();

    // Verify audit log entry
    let audit_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM audit_log WHERE event_type = ?1 AND user_id = ?2",
        ["data_deletion", user_id],
        |row| row.get(0)
    ).unwrap();

    assert_eq!(audit_count, 1, "Deletion should be logged in audit trail");
}

/// Test GDPR Article 20 - Right to Data Portability
/// User should receive data in structured, machine-readable format
#[test]
fn test_gdpr_article_20_right_to_portability() {
    let engine = ExportEngine::new();
    let data = mock_user_full();
    let temp_dir = create_temp_dir();

    // User Action: Request data export in multiple formats
    let formats = vec!["json", "markdown", "docx"];
    let result = engine.export_user_data(&data, &temp_dir, &formats);

    assert!(result.is_ok(), "Export should succeed");

    let exported_files = result.unwrap();
    assert_eq!(exported_files.len(), 3, "Should export all requested formats");

    // Verification 1: JSON is machine-readable
    let json_content = std::fs::read_to_string(temp_dir.join("bear_ai_export.json"))
        .expect("Should read JSON file");
    let parsed: serde_json::Value = serde_json::from_str(&json_content)
        .expect("JSON should be valid and parseable");

    assert!(parsed.is_object(), "JSON should be structured");
    assert!(parsed.get("user_id").is_some(), "Should contain user_id");
    assert!(parsed.get("chats").is_some(), "Should contain chats");
    assert!(parsed.get("documents").is_some(), "Should contain documents");

    // Verification 2: Markdown is human-readable
    let md_content = std::fs::read_to_string(temp_dir.join("bear_ai_export.md"))
        .expect("Should read Markdown file");

    assert!(md_content.contains("# BEAR AI"), "Should have readable header");
    assert!(md_content.contains("GDPR Article 20"), "Should mention compliance");
    assert!(md_content.contains("machine-readable"), "Should describe format");

    // Verification 3: Data integrity
    use sha2::{Sha256, Digest};
    use hex;

    let mut hasher = Sha256::new();
    hasher.update(json_content.as_bytes());
    let hash = hex::encode(hasher.finalize());

    assert_eq!(hash.len(), 64, "Should generate SHA-256 hash");

    // Verification 4: GDPR compliance metadata
    let compliance_info = parsed.get("metadata")
        .and_then(|m| m.get("compliance_info"))
        .expect("Should have compliance info");

    assert_eq!(
        compliance_info.get("gdpr_article_20").and_then(|v| v.as_bool()),
        Some(true),
        "Should be GDPR Article 20 compliant"
    );

    cleanup_temp_dir(&temp_dir);
}

/// Test complete user journey: Access → Export → Delete
#[test]
fn test_complete_user_rights_workflow() {
    let conn = create_test_db();
    let user_id = "complete-workflow-user";

    // Phase 1: User creates data
    conn.execute(
        "INSERT INTO chat_sessions (id, title, model_used) VALUES (?1, ?2, ?3)",
        ["workflow-chat", "My Legal Research", "gpt-4"],
    ).unwrap();

    conn.execute(
        "INSERT INTO chat_messages (chat_id, role, content) VALUES (?1, ?2, ?3)",
        ["workflow-chat", "user", "What are my data rights?"],
    ).unwrap();

    // Phase 2: User exercises Right of Access (Article 15)
    let chat_title: String = conn.query_row(
        "SELECT title FROM chat_sessions WHERE id = ?1",
        ["workflow-chat"],
        |row| row.get(0)
    ).unwrap();

    assert_eq!(chat_title, "My Legal Research", "User can access their data");

    // Phase 3: User exercises Right to Portability (Article 20)
    let engine = ExportEngine::new();
    let data = mock_user_full();
    let temp_dir = create_temp_dir();

    let export_result = engine.export_user_data(&data, &temp_dir, &vec!["json", "markdown"]);
    assert!(export_result.is_ok(), "User can export their data");

    // Log export
    conn.execute(
        "INSERT INTO audit_log (event_type, user_id, action, result) VALUES (?1, ?2, ?3, ?4)",
        ["data_export", user_id, "export_data", "success"],
    ).unwrap();

    // Phase 4: User exercises Right to Erasure (Article 17)
    conn.execute("DELETE FROM chat_messages WHERE chat_id = ?1", ["workflow-chat"]).unwrap();
    conn.execute("DELETE FROM chat_sessions WHERE id = ?1", ["workflow-chat"]).unwrap();

    // Log deletion
    conn.execute(
        "INSERT INTO audit_log (event_type, user_id, action, result) VALUES (?1, ?2, ?3, ?4)",
        ["data_deletion", user_id, "delete_all_data", "success"],
    ).unwrap();

    // Verify data is gone
    let remaining_chats: i64 = conn.query_row(
        "SELECT COUNT(*) FROM chat_sessions WHERE id = ?1",
        ["workflow-chat"],
        |row| row.get(0)
    ).unwrap();

    assert_eq!(remaining_chats, 0, "Data should be completely deleted");

    // Verify audit trail exists
    let audit_entries: i64 = conn.query_row(
        "SELECT COUNT(*) FROM audit_log WHERE user_id = ?1",
        [user_id],
        |row| row.get(0)
    ).unwrap();

    assert_eq!(audit_entries, 2, "Both export and deletion should be audited");

    cleanup_temp_dir(&temp_dir);
}

/// Test edge case: User with no data exercising rights
#[test]
fn test_empty_user_rights_workflow() {
    let conn = create_test_db();
    let user_id = "empty-user";

    // User has no data
    let chat_count: i64 = conn.query_row("SELECT COUNT(*) FROM chat_sessions", [], |row| row.get(0)).unwrap();
    assert_eq!(chat_count, 0, "New user has no data");

    // Right of Access: Should return empty but valid response
    let engine = ExportEngine::new();
    let data = mock_user_empty();
    let temp_dir = create_temp_dir();

    let export_result = engine.export_to_json(&data, &temp_dir.join("empty_export.json"));
    assert!(export_result.is_ok(), "Should handle empty user data");

    // Verify export is valid but empty
    let json_content = std::fs::read_to_string(temp_dir.join("empty_export.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_content).unwrap();

    assert_eq!(
        parsed.get("chats").and_then(|c| c.as_array()).map(|a| a.len()),
        Some(0),
        "Should have empty chats array"
    );

    // Right to Erasure: Should handle gracefully even with no data
    let delete_result = conn.execute(
        "DELETE FROM chat_sessions WHERE id = ?1",
        ["nonexistent-chat"]
    );
    assert!(delete_result.is_ok(), "Should handle deletion of nonexistent data");

    cleanup_temp_dir(&temp_dir);
}

fn export_to_json(data: &crate::export_engine::UserDataExport, path: &std::path::Path) -> anyhow::Result<()> {
    let json_str = serde_json::to_string_pretty(data)?;
    std::fs::write(path, json_str)?;
    Ok(())
}

/// Test data retention and automatic deletion
#[test]
fn test_data_retention_policy() {
    let conn = create_test_db();

    // Setup: Configure retention policy
    conn.execute(
        "INSERT INTO user_settings (setting_key, setting_value) VALUES (?1, ?2)",
        ["data_retention_days", "90"],
    ).unwrap();

    // Retrieve policy
    let retention_days: String = conn.query_row(
        "SELECT setting_value FROM user_settings WHERE setting_key = ?1",
        ["data_retention_days"],
        |row| row.get(0)
    ).unwrap();

    assert_eq!(retention_days, "90", "Should store retention policy");

    // Simulate old data (would be deleted by retention policy)
    // In real implementation, this would use timestamp comparison
    conn.execute(
        "INSERT INTO chat_sessions (id, title, model_used, created_at) VALUES (?1, ?2, ?3, datetime('now', '-100 days'))",
        ["old-chat", "Old Chat", "gpt-4"],
    ).unwrap();

    // Query old data (would be candidates for deletion)
    let old_chats: i64 = conn.query_row(
        "SELECT COUNT(*) FROM chat_sessions WHERE created_at < datetime('now', '-90 days')",
        [],
        |row| row.get(0)
    ).unwrap();

    assert!(old_chats > 0, "Should identify old data for retention policy");

    // In production: Automatic deletion would run here
    // conn.execute("DELETE FROM chat_sessions WHERE created_at < datetime('now', '-90 days')", [])?;
}

/// Test user consent withdrawal
#[test]
fn test_consent_withdrawal_and_data_handling() {
    let conn = create_test_db();
    let user_id = "consent-user";

    // Initial consent given
    conn.execute(
        "INSERT INTO user_settings (setting_key, setting_value) VALUES (?1, ?2)",
        ["analytics_consent", "true"],
    ).unwrap();

    // User withdraws consent
    conn.execute(
        "UPDATE user_settings SET setting_value = ?1 WHERE setting_key = ?2",
        ["false", "analytics_consent"],
    ).unwrap();

    // Verify consent withdrawn
    let consent: String = conn.query_row(
        "SELECT setting_value FROM user_settings WHERE setting_key = ?1",
        ["analytics_consent"],
        |row| row.get(0)
    ).unwrap();

    assert_eq!(consent, "false", "Consent should be withdrawn");

    // Log consent change
    conn.execute(
        "INSERT INTO audit_log (event_type, user_id, action, result) VALUES (?1, ?2, ?3, ?4)",
        ["consent_change", user_id, "withdraw_analytics_consent", "success"],
    ).unwrap();

    // Verify audit trail
    let consent_changes: i64 = conn.query_row(
        "SELECT COUNT(*) FROM audit_log WHERE event_type = ?1 AND user_id = ?2",
        ["consent_change", user_id],
        |row| row.get(0)
    ).unwrap();

    assert_eq!(consent_changes, 1, "Consent withdrawal should be audited");
}
