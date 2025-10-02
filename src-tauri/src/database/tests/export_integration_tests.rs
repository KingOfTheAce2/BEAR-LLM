// Comprehensive tests for database export integration

#[cfg(test)]
mod export_integration_tests {
    use rusqlite::Connection;
    use std::path::PathBuf;
    use std::env;
    use anyhow::Result;

    use crate::database::export_integration::ExportIntegration;

    fn get_test_db() -> PathBuf {
        let mut path = env::temp_dir();
        path.push(format!("test_export_integration_{}.db", uuid::Uuid::new_v4()));
        path
    }

    fn setup_comprehensive_test_database(db_path: &PathBuf) -> Result<()> {
        let conn = Connection::open(db_path)?;

        // Create all necessary tables
        conn.execute(
            "CREATE TABLE chat_sessions (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                model_used TEXT NOT NULL,
                tags TEXT DEFAULT '[]'
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE chat_messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                chat_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                metadata TEXT,
                FOREIGN KEY (chat_id) REFERENCES chat_sessions (id)
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE documents (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                filename TEXT NOT NULL,
                content TEXT NOT NULL,
                file_type TEXT NOT NULL,
                upload_date DATETIME DEFAULT CURRENT_TIMESTAMP,
                chunk_count INTEGER DEFAULT 0
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE pii_detections (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                document_id INTEGER NOT NULL,
                pii_type TEXT NOT NULL,
                replacement_text TEXT NOT NULL,
                confidence REAL NOT NULL,
                position_start INTEGER NOT NULL,
                position_end INTEGER NOT NULL,
                FOREIGN KEY (document_id) REFERENCES documents (id)
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE user_settings (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                setting_key TEXT UNIQUE NOT NULL,
                setting_value TEXT NOT NULL,
                last_updated DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE user_consent (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id TEXT NOT NULL,
                consent_type TEXT NOT NULL,
                granted BOOLEAN NOT NULL,
                granted_at DATETIME,
                revoked_at DATETIME,
                version INTEGER NOT NULL,
                consent_text TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE audit_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                user_id TEXT NOT NULL,
                action_type TEXT NOT NULL,
                entity_type TEXT NOT NULL,
                entity_id TEXT,
                details TEXT,
                success BOOLEAN NOT NULL,
                error_message TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE processing_records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                user_id TEXT NOT NULL,
                processing_purpose TEXT NOT NULL,
                retention_period INTEGER
            )",
            [],
        )?;

        Ok(())
    }

    #[test]
    fn test_export_multiple_chats() {
        let db_path = get_test_db();
        setup_comprehensive_test_database(&db_path).unwrap();

        let conn = Connection::open(&db_path).unwrap();

        // Insert multiple chat sessions
        for i in 1..=5 {
            conn.execute(
                "INSERT INTO chat_sessions (id, title, model_used) VALUES (?1, ?2, 'claude-3')",
                [&format!("chat{}", i), &format!("Test Chat {}", i)],
            ).unwrap();

            // Add messages to each chat
            for j in 1..=3 {
                conn.execute(
                    "INSERT INTO chat_messages (chat_id, role, content) VALUES (?1, 'user', ?2)",
                    [&format!("chat{}", i), &format!("Message {} in chat {}", j, i)],
                ).unwrap();
            }
        }

        drop(conn);

        let exporter = ExportIntegration::new(db_path.clone());
        let user_data = exporter.fetch_user_data("test_user").unwrap();

        assert_eq!(user_data.chats.len(), 5);
        for chat in &user_data.chats {
            assert_eq!(chat.messages.len(), 3);
        }

        let _ = std::fs::remove_file(db_path);
    }

    #[test]
    fn test_export_documents_with_pii() {
        let db_path = get_test_db();
        setup_comprehensive_test_database(&db_path).unwrap();

        let conn = Connection::open(&db_path).unwrap();

        // Insert documents with various PII types
        for i in 1..=3 {
            conn.execute(
                "INSERT INTO documents (filename, content, file_type, chunk_count)
                 VALUES (?1, 'Sample content', 'pdf', 10)",
                [&format!("document{}.pdf", i)],
            ).unwrap();

            let doc_id = conn.last_insert_rowid();

            // Add different PII types
            let pii_types = vec![
                ("EMAIL", "[EMAIL]", 0.95),
                ("PHONE", "[PHONE]", 0.89),
                ("SSN", "[SSN]", 0.97),
            ];

            for (idx, (pii_type, replacement, confidence)) in pii_types.iter().enumerate() {
                conn.execute(
                    "INSERT INTO pii_detections
                     (document_id, pii_type, replacement_text, confidence, position_start, position_end)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![doc_id, pii_type, replacement, confidence, idx * 20, (idx + 1) * 20],
                ).unwrap();
            }
        }

        drop(conn);

        let exporter = ExportIntegration::new(db_path.clone());
        let user_data = exporter.fetch_user_data("test_user").unwrap();

        assert_eq!(user_data.documents.len(), 3);
        for doc in &user_data.documents {
            assert_eq!(doc.pii_detections.len(), 3);
            assert_eq!(doc.chunk_count, 10);
        }

        let _ = std::fs::remove_file(db_path);
    }

    #[test]
    fn test_export_user_settings() {
        let db_path = get_test_db();
        setup_comprehensive_test_database(&db_path).unwrap();

        let conn = Connection::open(&db_path).unwrap();

        // Insert various settings
        conn.execute(
            "INSERT INTO user_settings (setting_key, setting_value) VALUES ('theme', '\"dark\"')",
            [],
        ).unwrap();

        conn.execute(
            "INSERT INTO user_settings (setting_key, setting_value) VALUES ('notifications', 'true')",
            [],
        ).unwrap();

        conn.execute(
            "INSERT INTO user_settings (setting_key, setting_value)
             VALUES ('preferences', '{\"language\": \"en\", \"timezone\": \"UTC\"}')",
            [],
        ).unwrap();

        drop(conn);

        let exporter = ExportIntegration::new(db_path.clone());
        let user_data = exporter.fetch_user_data("test_user").unwrap();

        let settings = &user_data.settings.preferences;
        assert!(settings.is_object());
        assert!(settings.get("theme").is_some());
        assert!(settings.get("notifications").is_some());

        let _ = std::fs::remove_file(db_path);
    }

    #[test]
    fn test_export_consent_history() {
        let db_path = get_test_db();
        setup_comprehensive_test_database(&db_path).unwrap();

        let conn = Connection::open(&db_path).unwrap();

        // Insert consent records with history
        let consent_types = vec!["chat_storage", "pii_detection", "document_processing"];

        for consent_type in consent_types {
            // Version 1 - granted
            conn.execute(
                "INSERT INTO user_consent
                 (user_id, consent_type, granted, granted_at, version, consent_text)
                 VALUES ('test_user', ?1, 1, datetime('now'), 1, 'Version 1 consent text')",
                [consent_type],
            ).unwrap();

            // Version 2 - updated
            conn.execute(
                "INSERT INTO user_consent
                 (user_id, consent_type, granted, granted_at, version, consent_text)
                 VALUES ('test_user', ?1, 1, datetime('now'), 2, 'Version 2 consent text')",
                [consent_type],
            ).unwrap();
        }

        drop(conn);

        let exporter = ExportIntegration::new(db_path.clone());
        let consent_data = exporter.fetch_consent_data("test_user").unwrap();

        assert_eq!(consent_data["total_count"], 6); // 3 types × 2 versions
        assert!(consent_data["consents"].is_array());

        let _ = std::fs::remove_file(db_path);
    }

    #[test]
    fn test_export_audit_trail() {
        let db_path = get_test_db();
        setup_comprehensive_test_database(&db_path).unwrap();

        let conn = Connection::open(&db_path).unwrap();

        // Insert audit log entries
        let actions = vec![
            ("consent_granted", "consent", true, None),
            ("data_accessed", "document", true, None),
            ("data_modified", "document", true, None),
            ("data_exported", "user_setting", true, None),
            ("data_deleted", "document", false, Some("Permission denied")),
        ];

        for (action_type, entity_type, success, error_msg) in actions {
            conn.execute(
                "INSERT INTO audit_log
                 (user_id, action_type, entity_type, entity_id, success, error_message, details)
                 VALUES ('test_user', ?1, ?2, 'entity123', ?3, ?4, '{\"test\": \"data\"}')",
                rusqlite::params![action_type, entity_type, success, error_msg],
            ).unwrap();
        }

        drop(conn);

        let exporter = ExportIntegration::new(db_path.clone());
        let audit_logs = exporter.fetch_audit_logs("test_user", 100).unwrap();

        assert_eq!(audit_logs["total_count"], 5);
        let logs = audit_logs["audit_trail"].as_array().unwrap();
        assert_eq!(logs.len(), 5);

        // Verify failed action is included
        let failed_log = logs.iter().find(|l| !l["success"].as_bool().unwrap());
        assert!(failed_log.is_some());
        assert_eq!(failed_log.unwrap()["error_message"], "Permission denied");

        let _ = std::fs::remove_file(db_path);
    }

    #[test]
    fn test_complete_export_integration() {
        let db_path = get_test_db();
        setup_comprehensive_test_database(&db_path).unwrap();

        let conn = Connection::open(&db_path).unwrap();

        // Set up comprehensive test data
        // 1. Chat sessions
        conn.execute(
            "INSERT INTO chat_sessions (id, title, model_used, tags)
             VALUES ('chat1', 'Legal Research', 'claude-3', '[\"legal\", \"research\"]')",
            [],
        ).unwrap();

        conn.execute(
            "INSERT INTO chat_messages (chat_id, role, content)
             VALUES ('chat1', 'user', 'What is GDPR Article 20?')",
            [],
        ).unwrap();

        conn.execute(
            "INSERT INTO chat_messages (chat_id, role, content)
             VALUES ('chat1', 'assistant', 'GDPR Article 20 is about data portability...')",
            [],
        ).unwrap();

        // 2. Documents
        conn.execute(
            "INSERT INTO documents (filename, content, file_type, chunk_count)
             VALUES ('contract.pdf', 'Sample contract content', 'pdf', 15)",
            [],
        ).unwrap();

        let doc_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO pii_detections
             (document_id, pii_type, replacement_text, confidence, position_start, position_end)
             VALUES (?1, 'EMAIL', '[EMAIL]', 0.95, 100, 120)",
            [doc_id],
        ).unwrap();

        // 3. Settings
        conn.execute(
            "INSERT INTO user_settings (setting_key, setting_value)
             VALUES ('data_retention', '\"2_years\"')",
            [],
        ).unwrap();

        // 4. Consent
        conn.execute(
            "INSERT INTO user_consent
             (user_id, consent_type, granted, granted_at, version, consent_text)
             VALUES ('test_user', 'chat_storage', 1, datetime('now'), 1, 'I consent to chat storage')",
            [],
        ).unwrap();

        // 5. Audit log
        conn.execute(
            "INSERT INTO audit_log
             (user_id, action_type, entity_type, success)
             VALUES ('test_user', 'data_accessed', 'document', 1)",
            [],
        ).unwrap();

        drop(conn);

        // Test complete export
        let exporter = ExportIntegration::new(db_path.clone());
        let complete_data = exporter.fetch_complete_user_data("test_user").unwrap();

        // Verify all sections are present
        assert!(complete_data["export_metadata"].is_object());
        assert!(complete_data["user_data"].is_object());
        assert!(complete_data["compliance_data"].is_object());
        assert!(complete_data["statistics"].is_object());

        // Verify statistics
        let stats = &complete_data["statistics"];
        assert_eq!(stats["total_chats"], 1);
        assert_eq!(stats["total_documents"], 1);
        assert_eq!(stats["total_messages"], 2);
        assert_eq!(stats["total_pii_detections"], 1);

        // Verify metadata
        let metadata = &complete_data["export_metadata"];
        assert!(metadata["hash"].as_str().unwrap().len() > 0);
        assert_eq!(metadata["format"], "GDPR Article 20 Compliant");

        let _ = std::fs::remove_file(db_path);
    }

    #[test]
    fn test_export_empty_database() {
        let db_path = get_test_db();
        setup_comprehensive_test_database(&db_path).unwrap();

        let exporter = ExportIntegration::new(db_path.clone());
        let user_data = exporter.fetch_user_data("nonexistent_user").unwrap();

        // Should return empty but valid export
        assert_eq!(user_data.chats.len(), 0);
        assert_eq!(user_data.documents.len(), 0);
        assert!(user_data.metadata.compliance_info.gdpr_article_20);

        let _ = std::fs::remove_file(db_path);
    }

    #[test]
    fn test_export_metadata_generation() {
        let db_path = get_test_db();
        setup_comprehensive_test_database(&db_path).unwrap();

        let conn = Connection::open(&db_path).unwrap();

        conn.execute(
            "INSERT INTO chat_sessions (id, title, model_used) VALUES ('chat1', 'Test', 'claude-3')",
            [],
        ).unwrap();

        drop(conn);

        let exporter = ExportIntegration::new(db_path.clone());
        let user_data = exporter.fetch_user_data("test_user").unwrap();

        // Verify metadata
        assert!(user_data.metadata.export_hash.len() == 64); // SHA-256 is 64 hex chars
        assert_eq!(user_data.metadata.format_version, "1.0.0");
        assert!(user_data.metadata.compliance_info.gdpr_article_20);
        assert!(user_data.metadata.compliance_info.integrity_verified);

        let _ = std::fs::remove_file(db_path);
    }

    #[test]
    fn test_export_with_special_characters() {
        let db_path = get_test_db();
        setup_comprehensive_test_database(&db_path).unwrap();

        let conn = Connection::open(&db_path).unwrap();

        // Insert data with special characters
        conn.execute(
            "INSERT INTO chat_sessions (id, title, model_used)
             VALUES ('chat1', 'Test with émojis 🚀 and spëcial chars', 'claude-3')",
            [],
        ).unwrap();

        conn.execute(
            "INSERT INTO chat_messages (chat_id, role, content)
             VALUES ('chat1', 'user', 'Content with \"quotes\" and <tags> & symbols')",
            [],
        ).unwrap();

        drop(conn);

        let exporter = ExportIntegration::new(db_path.clone());
        let user_data = exporter.fetch_user_data("test_user").unwrap();

        assert_eq!(user_data.chats.len(), 1);
        assert!(user_data.chats[0].title.contains("émojis"));
        assert!(user_data.chats[0].messages[0].content.contains("quotes"));

        let _ = std::fs::remove_file(db_path);
    }
}
