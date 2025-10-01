// Performance Tests for Compliance Features
//
// Tests verify that compliance features meet performance benchmarks

use crate::export_engine::ExportEngine;
use crate::pii_detector::PIIDetector;
use crate::compliance::tests::fixtures::{mock_user_large, mock_text_with_pii};
use crate::compliance::tests::test_utils::{create_test_db, create_temp_dir, cleanup_temp_dir};
use std::time::Instant;

#[test]
fn test_export_performance_1000_messages() {
    let engine = ExportEngine::new();
    let data = mock_user_large(); // Contains 100 chats with 50 messages each = 5000 messages
    let temp_dir = create_temp_dir();

    let start = Instant::now();
    let result = engine.export_to_markdown(&data, &temp_dir.join("perf_test.md"));
    let duration = start.elapsed();

    assert!(result.is_ok(), "Export should succeed");
    assert!(
        duration.as_secs() < 5,
        "Export of 5000 messages should complete within 5 seconds, took {:?}",
        duration
    );

    cleanup_temp_dir(&temp_dir);
}

#[tokio::test]
async fn test_pii_detection_performance_100kb_text() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    // Generate large text (~100KB)
    let large_text = mock_text_with_pii().repeat(100);
    assert!(large_text.len() > 100_000, "Text should be over 100KB");

    let start = Instant::now();
    let result = detector.detect_pii(&large_text).await;
    let duration = start.elapsed();

    assert!(result.is_ok(), "PII detection should succeed");
    assert!(
        duration.as_millis() < 1000,
        "PII detection on 100KB should complete within 1s, took {:?}",
        duration
    );
}

#[test]
fn test_audit_log_insertion_performance() {
    let conn = create_test_db();

    let start = Instant::now();

    // Insert 1000 audit log entries
    for i in 0..1000 {
        conn.execute(
            "INSERT INTO audit_log (event_type, user_id, action, result) VALUES (?1, ?2, ?3, ?4)",
            [
                "test_event",
                &format!("user-{}", i),
                "test_action",
                "success"
            ],
        ).unwrap();
    }

    let duration = start.elapsed();

    assert!(
        duration.as_millis() < 5000,
        "1000 audit log insertions should complete within 5s, took {:?}",
        duration
    );
}

#[test]
fn test_database_query_performance() {
    let conn = create_test_db();

    // Insert test data
    for i in 0..10000 {
        conn.execute(
            "INSERT INTO documents (filename, content, file_type) VALUES (?1, ?2, ?3)",
            [&format!("doc-{}.pdf", i), "Test content", "pdf"],
        ).unwrap();
    }

    // Test query performance
    let start = Instant::now();

    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM documents WHERE file_type = ?1",
        ["pdf"],
        |row| row.get(0)
    ).unwrap();

    let duration = start.elapsed();

    assert_eq!(count, 10000);
    assert!(
        duration.as_millis() < 100,
        "Query on 10K documents should complete within 100ms, took {:?}",
        duration
    );
}

#[test]
fn test_data_deletion_cascade_performance() {
    let conn = create_test_db();

    // Setup: Create linked data
    for i in 0..100 {
        let chat_id = format!("chat-{}", i);

        conn.execute(
            "INSERT INTO chat_sessions (id, title, model_used) VALUES (?1, ?2, ?3)",
            [&chat_id, &format!("Chat {}", i), "gpt-4"],
        ).unwrap();

        // 100 messages per chat = 10,000 total messages
        for j in 0..100 {
            conn.execute(
                "INSERT INTO chat_messages (chat_id, role, content) VALUES (?1, ?2, ?3)",
                [&chat_id, "user", &format!("Message {}", j)],
            ).unwrap();
        }
    }

    // Test cascading deletion performance
    let start = Instant::now();

    // Delete all messages first (foreign key constraint)
    conn.execute("DELETE FROM chat_messages", []).unwrap();
    // Then delete chats
    conn.execute("DELETE FROM chat_sessions", []).unwrap();

    let duration = start.elapsed();

    assert!(
        duration.as_secs() < 2,
        "Cascading deletion of 10K messages should complete within 2s, took {:?}",
        duration
    );

    // Verify deletion
    let remaining: i64 = conn.query_row("SELECT COUNT(*) FROM chat_messages", [], |row| row.get(0)).unwrap();
    assert_eq!(remaining, 0);
}

#[test]
fn test_encryption_performance_1mb() {
    use age::secrecy::Secret;

    // Generate 1MB of data
    let data = "x".repeat(1024 * 1024);
    let passphrase = Secret::new("test-passphrase".to_string());

    let start = Instant::now();

    // Encrypt
    let encryptor = age::Encryptor::with_user_passphrase(passphrase.clone());
    let mut encrypted = vec![];
    let mut writer = encryptor.wrap_output(&mut encrypted).unwrap();
    std::io::Write::write_all(&mut writer, data.as_bytes()).unwrap();
    writer.finish().unwrap();

    let encrypt_duration = start.elapsed();

    assert!(
        encrypt_duration.as_millis() < 500,
        "Encryption of 1MB should complete within 500ms, took {:?}",
        encrypt_duration
    );

    assert!(!encrypted.is_empty(), "Should produce encrypted data");
}

#[tokio::test]
async fn test_concurrent_pii_detection_performance() {
    use tokio::task;

    let detector = std::sync::Arc::new(PIIDetector::new());
    detector.initialize().await.unwrap();

    let texts: Vec<String> = (0..10)
        .map(|i| format!("{} - Email: user{}@example.com", mock_text_with_pii(), i))
        .collect();

    let start = Instant::now();

    // Process 10 texts concurrently
    let mut handles = vec![];

    for text in texts {
        let detector_clone = detector.clone();
        let handle = task::spawn(async move {
            detector_clone.detect_pii(&text).await
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok(), "Concurrent PII detection should succeed");
    }

    let duration = start.elapsed();

    assert!(
        duration.as_secs() < 3,
        "10 concurrent PII detections should complete within 3s, took {:?}",
        duration
    );
}

#[test]
fn test_export_format_generation_performance() {
    let engine = ExportEngine::new();
    let data = mock_user_large();
    let temp_dir = create_temp_dir();

    // Test each format individually for performance
    let formats = vec!["markdown", "docx", "pdf", "txt"];

    for format in formats {
        let start = Instant::now();

        let result = engine.export_user_data(&data, &temp_dir, &vec![format]);

        let duration = start.elapsed();

        assert!(result.is_ok(), "Export format {} should succeed", format);
        assert!(
            duration.as_secs() < 5,
            "Export to {} should complete within 5s, took {:?}",
            format,
            duration
        );
    }

    cleanup_temp_dir(&temp_dir);
}

#[test]
fn test_memory_usage_during_large_export() {
    use sysinfo::{System, SystemExt, ProcessExt};

    let mut sys = System::new_all();
    sys.refresh_all();

    let pid = sysinfo::get_current_pid().unwrap();
    sys.refresh_process(pid);

    let initial_memory = sys.process(pid).map(|p| p.memory()).unwrap_or(0);

    // Perform large export
    let engine = ExportEngine::new();
    let data = mock_user_large();
    let temp_dir = create_temp_dir();

    let _ = engine.export_user_data(&data, &temp_dir, &vec!["markdown", "docx", "pdf"]);

    sys.refresh_process(pid);
    let final_memory = sys.process(pid).map(|p| p.memory()).unwrap_or(0);

    let memory_increase = final_memory.saturating_sub(initial_memory);

    assert!(
        memory_increase < 200 * 1024 * 1024, // Less than 200MB increase
        "Memory increase should be under 200MB, was {} MB",
        memory_increase / (1024 * 1024)
    );

    cleanup_temp_dir(&temp_dir);
}

#[test]
fn test_audit_log_query_performance_100k_entries() {
    let conn = create_test_db();

    // Insert 100K audit log entries
    for i in 0..100_000 {
        conn.execute(
            "INSERT INTO audit_log (event_type, user_id, action, result) VALUES (?1, ?2, ?3, ?4)",
            [
                if i % 3 == 0 { "data_export" } else if i % 3 == 1 { "data_deletion" } else { "consent_change" },
                &format!("user-{}", i % 1000),
                "test_action",
                if i % 10 == 0 { "failure" } else { "success" }
            ],
        ).unwrap();
    }

    // Query performance test
    let start = Instant::now();

    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM audit_log WHERE event_type = ?1 AND result = ?2",
        ["data_export", "success"],
        |row| row.get(0)
    ).unwrap();

    let duration = start.elapsed();

    assert!(count > 0, "Should find matching entries");
    assert!(
        duration.as_millis() < 500,
        "Query on 100K entries should complete within 500ms, took {:?}",
        duration
    );
}

#[tokio::test]
async fn test_pii_redaction_performance() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let text = mock_text_with_pii().repeat(50); // ~50x PII occurrences

    let start = Instant::now();
    let result = detector.redact_pii(&text).await;
    let duration = start.elapsed();

    assert!(result.is_ok(), "Redaction should succeed");
    assert!(
        duration.as_millis() < 800,
        "PII redaction should complete within 800ms, took {:?}",
        duration
    );
}

#[test]
fn test_json_serialization_performance() {
    let data = mock_user_large();

    let start = Instant::now();
    let json_str = serde_json::to_string_pretty(&data);
    let duration = start.elapsed();

    assert!(json_str.is_ok(), "JSON serialization should succeed");
    assert!(
        duration.as_millis() < 200,
        "JSON serialization should complete within 200ms, took {:?}",
        duration
    );

    let json_content = json_str.unwrap();
    assert!(json_content.len() > 10000, "Should produce substantial JSON output");
}

#[test]
fn test_hash_generation_performance() {
    use sha2::{Sha256, Digest};
    use hex;

    let data = "x".repeat(10 * 1024 * 1024); // 10MB

    let start = Instant::now();

    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    let hash = hex::encode(hasher.finalize());

    let duration = start.elapsed();

    assert_eq!(hash.len(), 64, "Should generate 64-char hash");
    assert!(
        duration.as_millis() < 300,
        "SHA-256 hash of 10MB should complete within 300ms, took {:?}",
        duration
    );
}
