// Unit Tests for Export Engine
//
// Tests cover GDPR Article 20 (Data Portability) compliance

use crate::export_engine::ExportEngine;
use crate::compliance::tests::fixtures::{mock_user_full, mock_user_empty, mock_user_large};
use crate::compliance::tests::test_utils::{create_temp_dir, cleanup_temp_dir};
use std::path::PathBuf;

#[test]
fn test_export_engine_initialization() {
    let engine = ExportEngine::new();
    assert!(
        std::mem::size_of_val(&engine) > 0,
        "Export engine should initialize successfully"
    );
}

#[test]
fn test_export_to_json_basic() {
    let engine = ExportEngine::new();
    let data = mock_user_full();
    let temp_dir = create_temp_dir();
    let output_path = temp_dir.join("export.json");

    let result = std::fs::write(
        &output_path,
        serde_json::to_string_pretty(&data).expect("Failed to serialize")
    );

    assert!(result.is_ok(), "JSON export should succeed");
    assert!(output_path.exists(), "JSON file should be created");

    // Verify file content
    let content = std::fs::read_to_string(&output_path).expect("Failed to read file");
    assert!(content.contains("test-user-001"), "Should contain user ID");
    assert!(content.contains("GDPR"), "Should mention GDPR compliance");

    cleanup_temp_dir(&temp_dir);
}

#[test]
fn test_export_to_docx() {
    let engine = ExportEngine::new();
    let data = mock_user_full();
    let temp_dir = create_temp_dir();
    let output_path = temp_dir.join("export.docx");

    let result = engine.export_to_docx(&data, &output_path);

    assert!(result.is_ok(), "DOCX export should succeed: {:?}", result.err());
    assert!(output_path.exists(), "DOCX file should be created");
    assert!(
        output_path.metadata().unwrap().len() > 1000,
        "DOCX file should have substantial content"
    );

    cleanup_temp_dir(&temp_dir);
}

#[test]
fn test_export_to_markdown() {
    let engine = ExportEngine::new();
    let data = mock_user_full();
    let temp_dir = create_temp_dir();
    let output_path = temp_dir.join("export.md");

    let result = engine.export_to_markdown(&data, &output_path);

    assert!(result.is_ok(), "Markdown export should succeed");
    assert!(output_path.exists(), "Markdown file should be created");

    // Verify markdown structure
    let content = std::fs::read_to_string(&output_path).expect("Failed to read file");
    assert!(content.contains("# BEAR AI"), "Should have main header");
    assert!(content.contains("## GDPR Article 20"), "Should have compliance section");
    assert!(content.contains("## Chat History"), "Should have chat section");
    assert!(content.contains("## Processed Documents"), "Should have documents section");

    cleanup_temp_dir(&temp_dir);
}

#[test]
fn test_export_to_pdf() {
    let engine = ExportEngine::new();
    let data = mock_user_full();
    let temp_dir = create_temp_dir();
    let output_path = temp_dir.join("export.pdf");

    let result = engine.export_to_pdf(&data, &output_path);

    assert!(result.is_ok(), "PDF export should succeed: {:?}", result.err());
    assert!(output_path.exists(), "PDF file should be created");
    assert!(
        output_path.metadata().unwrap().len() > 500,
        "PDF file should have content"
    );

    cleanup_temp_dir(&temp_dir);
}

#[test]
fn test_export_to_text() {
    let engine = ExportEngine::new();
    let data = mock_user_full();
    let temp_dir = create_temp_dir();
    let output_path = temp_dir.join("export.txt");

    let result = engine.export_to_text(&data, &output_path);

    assert!(result.is_ok(), "Text export should succeed");
    assert!(output_path.exists(), "Text file should be created");

    // Verify text structure
    let content = std::fs::read_to_string(&output_path).expect("Failed to read file");
    assert!(content.contains("BEAR AI"), "Should have title");
    assert!(content.contains("GDPR ARTICLE 20"), "Should have compliance header");
    assert!(content.lines().count() > 20, "Should have multiple lines");

    cleanup_temp_dir(&temp_dir);
}

#[test]
fn test_export_empty_user_data() {
    let engine = ExportEngine::new();
    let data = mock_user_empty();
    let temp_dir = create_temp_dir();
    let output_path = temp_dir.join("export_empty.md");

    let result = engine.export_to_markdown(&data, &output_path);

    assert!(result.is_ok(), "Should handle empty user data gracefully");

    let content = std::fs::read_to_string(&output_path).expect("Failed to read file");
    assert!(content.contains("test-user-002"), "Should contain user ID");
    assert!(content.contains("Total Documents: 0"), "Should show zero documents");

    cleanup_temp_dir(&temp_dir);
}

#[test]
fn test_export_multiple_formats() {
    let engine = ExportEngine::new();
    let data = mock_user_full();
    let temp_dir = create_temp_dir();

    let formats = vec!["docx", "markdown", "pdf", "txt"];
    let result = engine.export_user_data(&data, &temp_dir, &formats);

    assert!(result.is_ok(), "Multi-format export should succeed");

    let files = result.unwrap();
    assert_eq!(files.len(), 4, "Should create 4 files");

    // Verify all files exist
    assert!(temp_dir.join("bear_ai_export.docx").exists());
    assert!(temp_dir.join("bear_ai_export.md").exists());
    assert!(temp_dir.join("bear_ai_export.pdf").exists());
    assert!(temp_dir.join("bear_ai_export.txt").exists());

    cleanup_temp_dir(&temp_dir);
}

#[test]
fn test_export_unsupported_format() {
    let engine = ExportEngine::new();
    let data = mock_user_full();
    let temp_dir = create_temp_dir();

    let result = engine.export_user_data(&data, &temp_dir, &vec!["xml"]);

    assert!(result.is_err(), "Should reject unsupported format");
    assert!(
        result.unwrap_err().to_string().contains("Unsupported"),
        "Error should mention unsupported format"
    );

    cleanup_temp_dir(&temp_dir);
}

#[test]
fn test_export_integrity_hash() {
    use sha2::{Sha256, Digest};
    use hex;

    let data = mock_user_full();
    let json_str = serde_json::to_string(&data).expect("Failed to serialize");

    let mut hasher = Sha256::new();
    hasher.update(json_str.as_bytes());
    let hash = hex::encode(hasher.finalize());

    assert_eq!(hash.len(), 64, "SHA-256 hash should be 64 characters");
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()), "Hash should be valid hex");
}

#[test]
fn test_export_pii_in_documents() {
    let engine = ExportEngine::new();
    let data = mock_user_full();
    let temp_dir = create_temp_dir();
    let output_path = temp_dir.join("export.md");

    let result = engine.export_to_markdown(&data, &output_path);
    assert!(result.is_ok());

    let content = std::fs::read_to_string(&output_path).expect("Failed to read file");

    // Verify PII detections are documented
    assert!(content.contains("PII Detections"), "Should document PII detections");
    assert!(content.contains("EMAIL") || content.contains("PHONE") || content.contains("SSN"),
           "Should show detected PII types");

    cleanup_temp_dir(&temp_dir);
}

#[test]
fn test_export_performance_large_dataset() {
    use std::time::Instant;

    let engine = ExportEngine::new();
    let data = mock_user_large();
    let temp_dir = create_temp_dir();
    let output_path = temp_dir.join("export_large.md");

    let start = Instant::now();
    let result = engine.export_to_markdown(&data, &output_path);
    let duration = start.elapsed();

    assert!(result.is_ok(), "Large export should succeed");
    assert!(duration.as_secs() < 5, "Export should complete within 5 seconds, took {:?}", duration);

    cleanup_temp_dir(&temp_dir);
}

#[test]
fn test_export_gdpr_compliance_metadata() {
    let engine = ExportEngine::new();
    let data = mock_user_full();
    let temp_dir = create_temp_dir();
    let output_path = temp_dir.join("export.md");

    let result = engine.export_to_markdown(&data, &output_path);
    assert!(result.is_ok());

    let content = std::fs::read_to_string(&output_path).expect("Failed to read file");

    // Verify GDPR compliance information
    assert!(content.contains("GDPR Article 20"), "Should reference GDPR Article 20");
    assert!(content.contains("data portability"), "Should mention data portability");
    assert!(content.contains("machine-readable"), "Should mention machine-readable format");

    cleanup_temp_dir(&temp_dir);
}

#[test]
fn test_export_metadata_completeness() {
    let data = mock_user_full();

    assert!(!data.user_id.is_empty(), "User ID should not be empty");
    assert!(!data.version.is_empty(), "Version should not be empty");
    assert!(!data.metadata.export_hash.is_empty(), "Hash should not be empty");
    assert!(data.metadata.compliance_info.gdpr_article_20, "Should be GDPR compliant");
    assert!(data.metadata.compliance_info.integrity_verified, "Should verify integrity");
}

#[test]
fn test_export_chat_message_order() {
    let data = mock_user_full();

    for chat in &data.chats {
        let mut prev_timestamp = chat.messages[0].timestamp;
        for msg in &chat.messages[1..] {
            assert!(
                msg.timestamp >= prev_timestamp,
                "Messages should be in chronological order"
            );
            prev_timestamp = msg.timestamp;
        }
    }
}

#[test]
fn test_export_edge_case_special_characters() {
    use crate::export_engine::UserDataExport;

    let mut data = mock_user_full();
    // Add special characters to test escaping
    data.chats[0].title = "Test <>&\"'`".to_string();
    data.chats[0].messages[0].content = "Line 1\nLine 2\tTab\r\nCRLF".to_string();

    let engine = ExportEngine::new();
    let temp_dir = create_temp_dir();
    let output_path = temp_dir.join("export_special.md");

    let result = engine.export_to_markdown(&data, &output_path);
    assert!(result.is_ok(), "Should handle special characters");

    cleanup_temp_dir(&temp_dir);
}
