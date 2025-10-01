// Compliance Testing Module
//
// This module contains comprehensive tests for GDPR/AI Act compliance features.
// Tests are organized by category: unit, integration, e2e, and security.

#[cfg(test)]
pub mod fixtures;

#[cfg(test)]
pub mod unit;

#[cfg(test)]
pub mod integration;

#[cfg(test)]
pub mod e2e;

#[cfg(test)]
pub mod security;

// Test utilities and helpers
#[cfg(test)]
pub mod test_utils {
    use rusqlite::Connection;
    use std::path::PathBuf;

    /// Create an in-memory test database with schema
    pub fn create_test_db() -> Connection {
        let conn = Connection::open_in_memory()
            .expect("Failed to create in-memory database");

        // Initialize schema
        conn.execute(
            "CREATE TABLE IF NOT EXISTS chat_sessions (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                model_used TEXT NOT NULL,
                tags TEXT DEFAULT '[]'
            )",
            [],
        ).expect("Failed to create chat_sessions table");

        conn.execute(
            "CREATE TABLE IF NOT EXISTS chat_messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                chat_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                metadata TEXT,
                FOREIGN KEY (chat_id) REFERENCES chat_sessions (id)
            )",
            [],
        ).expect("Failed to create chat_messages table");

        conn.execute(
            "CREATE TABLE IF NOT EXISTS documents (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                filename TEXT NOT NULL,
                content TEXT NOT NULL,
                file_type TEXT NOT NULL,
                upload_date DATETIME DEFAULT CURRENT_TIMESTAMP,
                vector_embedding BLOB,
                chunk_count INTEGER DEFAULT 0
            )",
            [],
        ).expect("Failed to create documents table");

        conn.execute(
            "CREATE TABLE IF NOT EXISTS pii_detections (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                document_id INTEGER NOT NULL,
                pii_type TEXT NOT NULL,
                replacement_text TEXT NOT NULL,
                confidence REAL NOT NULL,
                position_start INTEGER NOT NULL,
                position_end INTEGER NOT NULL,
                detection_date DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (document_id) REFERENCES documents (id)
            )",
            [],
        ).expect("Failed to create pii_detections table");

        conn.execute(
            "CREATE TABLE IF NOT EXISTS user_settings (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                setting_key TEXT UNIQUE NOT NULL,
                setting_value TEXT NOT NULL,
                last_updated DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        ).expect("Failed to create user_settings table");

        conn.execute(
            "CREATE TABLE IF NOT EXISTS audit_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                event_type TEXT NOT NULL,
                user_id TEXT,
                resource_type TEXT,
                resource_id TEXT,
                action TEXT NOT NULL,
                result TEXT NOT NULL,
                metadata TEXT
            )",
            [],
        ).expect("Failed to create audit_log table");

        conn
    }

    /// Clean up test database and files
    pub fn cleanup_test_db(conn: Connection) {
        drop(conn);
    }

    /// Generate temporary test directory
    pub fn create_temp_dir() -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("bear_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&path).expect("Failed to create temp directory");
        path
    }

    /// Remove temporary test directory
    pub fn cleanup_temp_dir(path: &PathBuf) {
        let _ = std::fs::remove_dir_all(path);
    }
}
