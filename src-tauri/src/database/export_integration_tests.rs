// Integration tests for export_integration.rs
// Verifies critical data isolation between users

#[cfg(test)]
mod tests {
    use super::super::export_integration::ExportIntegration;
    use rusqlite::Connection;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn setup_test_db() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = Connection::open(&db_path).unwrap();

        // Create tables with user_id columns (post-migration schema)
        conn.execute_batch(
            "
            CREATE TABLE chat_sessions (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                title TEXT,
                created_at TEXT,
                updated_at TEXT,
                model_used TEXT,
                tags TEXT
            );

            CREATE TABLE chat_messages (
                id INTEGER PRIMARY KEY,
                chat_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                role TEXT,
                content TEXT,
                timestamp TEXT,
                metadata TEXT
            );

            CREATE TABLE documents (
                id INTEGER PRIMARY KEY,
                user_id TEXT NOT NULL,
                filename TEXT,
                file_type TEXT,
                upload_date TEXT,
                chunk_count INTEGER
            );

            CREATE TABLE pii_detections (
                id INTEGER PRIMARY KEY,
                document_id INTEGER,
                pii_type TEXT,
                replacement_text TEXT,
                confidence REAL,
                position_start INTEGER,
                position_end INTEGER
            );

            CREATE TABLE user_settings (
                id INTEGER PRIMARY KEY,
                user_id TEXT NOT NULL,
                setting_key TEXT,
                setting_value TEXT
            );

            -- Insert test data for user1
            INSERT INTO chat_sessions VALUES
                ('chat1', 'user1', 'User 1 Chat', '2025-01-01', '2025-01-01', 'gpt-4', '[]');

            INSERT INTO chat_messages VALUES
                (1, 'chat1', 'user1', 'user', 'User 1 message', '2025-01-01', NULL);

            INSERT INTO documents VALUES
                (1, 'user1', 'user1_doc.pdf', 'pdf', '2025-01-01', 5);

            INSERT INTO user_settings VALUES
                (1, 'user1', 'theme', 'dark');

            -- Insert test data for user2
            INSERT INTO chat_sessions VALUES
                ('chat2', 'user2', 'User 2 Chat', '2025-01-02', '2025-01-02', 'gpt-3.5', '[]');

            INSERT INTO chat_messages VALUES
                (2, 'chat2', 'user2', 'user', 'User 2 message', '2025-01-02', NULL);

            INSERT INTO documents VALUES
                (2, 'user2', 'user2_doc.pdf', 'pdf', '2025-01-02', 3);

            INSERT INTO user_settings VALUES
                (2, 'user2', 'theme', 'light');
            ",
        )
        .unwrap();

        (temp_dir, db_path)
    }

    #[test]
    fn test_user_data_isolation_chats() {
        let (_temp_dir, db_path) = setup_test_db();
        let exporter = ExportIntegration::new(db_path);

        // Fetch user1's data
        let user1_data = exporter.fetch_user_data("user1").unwrap();

        // Verify user1 only sees their own chat
        assert_eq!(user1_data.chats.len(), 1);
        assert_eq!(user1_data.chats[0].id, "chat1");
        assert_eq!(user1_data.chats[0].title, "User 1 Chat");

        // Verify user1 does NOT see user2's chat
        assert!(!user1_data
            .chats
            .iter()
            .any(|c| c.id == "chat2" || c.title.contains("User 2")));

        // Fetch user2's data
        let user2_data = exporter.fetch_user_data("user2").unwrap();

        // Verify user2 only sees their own chat
        assert_eq!(user2_data.chats.len(), 1);
        assert_eq!(user2_data.chats[0].id, "chat2");
        assert_eq!(user2_data.chats[0].title, "User 2 Chat");

        // Verify user2 does NOT see user1's chat
        assert!(!user2_data
            .chats
            .iter()
            .any(|c| c.id == "chat1" || c.title.contains("User 1")));
    }

    #[test]
    fn test_user_data_isolation_documents() {
        let (_temp_dir, db_path) = setup_test_db();
        let exporter = ExportIntegration::new(db_path);

        // Fetch user1's data
        let user1_data = exporter.fetch_user_data("user1").unwrap();

        // Verify user1 only sees their own document
        assert_eq!(user1_data.documents.len(), 1);
        assert_eq!(user1_data.documents[0].filename, "user1_doc.pdf");

        // Verify user1 does NOT see user2's document
        assert!(!user1_data
            .documents
            .iter()
            .any(|d| d.filename.contains("user2")));

        // Fetch user2's data
        let user2_data = exporter.fetch_user_data("user2").unwrap();

        // Verify user2 only sees their own document
        assert_eq!(user2_data.documents.len(), 1);
        assert_eq!(user2_data.documents[0].filename, "user2_doc.pdf");

        // Verify user2 does NOT see user1's document
        assert!(!user2_data
            .documents
            .iter()
            .any(|d| d.filename.contains("user1")));
    }

    #[test]
    fn test_user_data_isolation_settings() {
        let (_temp_dir, db_path) = setup_test_db();
        let exporter = ExportIntegration::new(db_path);

        // Fetch user1's data
        let user1_data = exporter.fetch_user_data("user1").unwrap();

        // Verify user1 sees their theme setting
        assert!(user1_data.settings.preferences.contains_key("theme"));
        assert_eq!(
            user1_data.settings.preferences.get("theme").unwrap(),
            &serde_json::json!("dark")
        );

        // Fetch user2's data
        let user2_data = exporter.fetch_user_data("user2").unwrap();

        // Verify user2 sees THEIR theme setting (not user1's)
        assert!(user2_data.settings.preferences.contains_key("theme"));
        assert_eq!(
            user2_data.settings.preferences.get("theme").unwrap(),
            &serde_json::json!("light")
        );
    }

    #[test]
    fn test_no_data_for_nonexistent_user() {
        let (_temp_dir, db_path) = setup_test_db();
        let exporter = ExportIntegration::new(db_path);

        // Fetch data for user that doesn't exist
        let user3_data = exporter.fetch_user_data("user3").unwrap();

        // Should return empty data, not other users' data
        assert_eq!(user3_data.chats.len(), 0);
        assert_eq!(user3_data.documents.len(), 0);
        assert_eq!(user3_data.settings.preferences.len(), 0);
    }

    #[test]
    fn test_chat_messages_isolation() {
        let (_temp_dir, db_path) = setup_test_db();
        let exporter = ExportIntegration::new(db_path);

        let user1_data = exporter.fetch_user_data("user1").unwrap();

        // Verify user1's chat contains only their message
        assert_eq!(user1_data.chats[0].messages.len(), 1);
        assert!(user1_data.chats[0].messages[0]
            .content
            .contains("User 1 message"));
        assert!(!user1_data.chats[0].messages[0]
            .content
            .contains("User 2"));

        let user2_data = exporter.fetch_user_data("user2").unwrap();

        // Verify user2's chat contains only their message
        assert_eq!(user2_data.chats[0].messages.len(), 1);
        assert!(user2_data.chats[0].messages[0]
            .content
            .contains("User 2 message"));
        assert!(!user2_data.chats[0].messages[0]
            .content
            .contains("User 1"));
    }
}
