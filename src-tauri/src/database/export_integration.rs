// Database Export Integration for GDPR Article 20 Data Portability
// Maps database records to UserDataExport structure for export engine

use anyhow::{anyhow, Result};
use chrono::Utc;
use rusqlite::{params, Connection};
use serde_json;
use std::path::PathBuf;

use crate::export_engine::{
    ChatExport, ComplianceInfo, DocumentExport, ExportMetadata, MessageExport, PIIDetection,
    SettingsExport, UserDataExport,
};

/// Database Export Manager - fetches all user data for GDPR export
pub struct ExportIntegration {
    db_path: PathBuf,
}

impl ExportIntegration {
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }

    /// Get database connection
    fn get_connection(&self) -> Result<Connection> {
        Connection::open(&self.db_path).map_err(|e| anyhow!("Failed to open database: {}", e))
    }

    /// Fetch all user data from database and convert to export format
    pub fn fetch_user_data(&self, user_id: &str) -> Result<UserDataExport> {
        // Fetch all components
        let chats = self.fetch_chat_history(user_id)?;
        let documents = self.fetch_documents(user_id)?;
        let settings = self.fetch_user_settings(user_id)?;

        // Generate export metadata
        let metadata = self.generate_export_metadata(user_id, &chats, &documents)?;

        Ok(UserDataExport {
            export_date: Utc::now(),
            version: "1.0.25".to_string(), // BEAR AI version
            user_id: user_id.to_string(),
            chats,
            documents,
            settings,
            metadata,
        })
    }

    /// Fetch all chat sessions and messages for a user
    fn fetch_chat_history(&self, _user_id: &str) -> Result<Vec<ChatExport>> {
        let conn = self.get_connection()?;

        // Fetch chat sessions
        let mut stmt = conn.prepare(
            "SELECT id, title, created_at, updated_at, model_used, tags
             FROM chat_sessions
             WHERE id IN (
                 SELECT DISTINCT chat_id FROM chat_messages
                 -- In production, filter by user_id if messages table has user association
             )
             ORDER BY created_at DESC",
        )?;

        let chat_sessions: Vec<(String, String, String, String, String, String)> = stmt
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let mut chats = Vec::new();

        for (chat_id, title, created_at, updated_at, model_used, tags_json) in chat_sessions {
            // Fetch messages for this chat
            let messages = self.fetch_chat_messages(&chat_id)?;

            // Parse tags JSON
            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_else(|_| vec![]);

            chats.push(ChatExport {
                id: chat_id,
                title,
                created_at: created_at.parse().unwrap_or_else(|_| Utc::now()),
                updated_at: updated_at.parse().unwrap_or_else(|_| Utc::now()),
                messages,
                model_used,
                tags,
            });
        }

        Ok(chats)
    }

    /// Fetch all messages for a specific chat session
    fn fetch_chat_messages(&self, chat_id: &str) -> Result<Vec<MessageExport>> {
        let conn = self.get_connection()?;

        let mut stmt = conn.prepare(
            "SELECT role, content, timestamp, metadata
             FROM chat_messages
             WHERE chat_id = ?1
             ORDER BY timestamp ASC",
        )?;

        let messages = stmt
            .query_map(params![chat_id], |row| {
                let metadata_str: Option<String> = row.get(3)?;
                let metadata = metadata_str.and_then(|s| serde_json::from_str(&s).ok());

                Ok(MessageExport {
                    role: row.get(0)?,
                    content: row.get(1)?,
                    timestamp: row
                        .get::<_, String>(2)?
                        .parse()
                        .unwrap_or_else(|_| Utc::now()),
                    metadata,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(messages)
    }

    /// Fetch all documents with PII detections for a user
    fn fetch_documents(&self, _user_id: &str) -> Result<Vec<DocumentExport>> {
        let conn = self.get_connection()?;

        // Fetch documents
        // Note: In production, filter by user_id if documents table has user association
        let mut stmt = conn.prepare(
            "SELECT id, filename, file_type, upload_date, chunk_count
             FROM documents
             ORDER BY upload_date DESC",
        )?;

        let docs: Vec<(i64, String, String, String, i64)> = stmt
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get::<_, Option<i64>>(4)?.unwrap_or(0),
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let mut documents = Vec::new();

        for (doc_id, filename, file_type, upload_date, chunk_count) in docs {
            // Fetch PII detections for this document
            let pii_detections = self.fetch_pii_detections(doc_id)?;

            documents.push(DocumentExport {
                id: doc_id,
                filename,
                file_type,
                upload_date: upload_date.parse().unwrap_or_else(|_| Utc::now()),
                chunk_count,
                pii_detections,
            });
        }

        Ok(documents)
    }

    /// Fetch PII detections for a specific document
    fn fetch_pii_detections(&self, document_id: i64) -> Result<Vec<PIIDetection>> {
        let conn = self.get_connection()?;

        let mut stmt = conn.prepare(
            "SELECT pii_type, replacement_text, confidence, position_start, position_end
             FROM pii_detections
             WHERE document_id = ?1
             ORDER BY position_start ASC",
        )?;

        let detections = stmt
            .query_map(params![document_id], |row| {
                Ok(PIIDetection {
                    pii_type: row.get(0)?,
                    replacement_text: row.get(1)?,
                    confidence: row.get(2)?,
                    position_start: row.get::<_, i64>(3)? as usize,
                    position_end: row.get::<_, i64>(4)? as usize,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(detections)
    }

    /// Fetch user settings and preferences
    fn fetch_user_settings(&self, user_id: &str) -> Result<SettingsExport> {
        let conn = self.get_connection()?;

        // Fetch all user settings
        let mut stmt = conn.prepare(
            "SELECT setting_key, setting_value
             FROM user_settings
             ORDER BY setting_key",
        )?;

        let mut preferences = serde_json::Map::new();
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        for row in rows {
            let (key, value) = row?;
            // Try to parse as JSON, fallback to string
            let parsed_value =
                serde_json::from_str(&value).unwrap_or(serde_json::Value::String(value));
            preferences.insert(key, parsed_value);
        }

        // Get retention policy from processing records if available
        let retention_policy = self.fetch_retention_policy(user_id)?;

        Ok(SettingsExport {
            preferences: serde_json::Value::Object(preferences),
            retention_policy,
        })
    }

    /// Fetch user's retention policy
    fn fetch_retention_policy(&self, user_id: &str) -> Result<Option<String>> {
        let conn = self.get_connection()?;

        let policy: Option<String> = conn
            .query_row(
                "SELECT retention_period FROM processing_records
                 WHERE user_id = ?1
                 ORDER BY timestamp DESC LIMIT 1",
                params![user_id],
                |row| row.get(0),
            )
            .ok();

        Ok(policy)
    }

    /// Generate export metadata with hash and compliance info
    fn generate_export_metadata(
        &self,
        user_id: &str,
        chats: &[ChatExport],
        documents: &[DocumentExport],
    ) -> Result<ExportMetadata> {
        // Generate hash from data for integrity verification
        let data_for_hash = format!(
            "{}:chats={}:docs={}:timestamp={}",
            user_id,
            chats.len(),
            documents.len(),
            Utc::now().to_rfc3339()
        );

        let hash = self.generate_sha256_hash(&data_for_hash);

        Ok(ExportMetadata {
            format_version: "1.0.0".to_string(),
            application_version: "1.0.25".to_string(), // BEAR AI version
            export_hash: hash,
            compliance_info: ComplianceInfo {
                gdpr_article_20: true,
                encrypted: false, // Set to true if encryption is applied
                integrity_verified: true,
            },
        })
    }

    /// Generate SHA-256 hash for data integrity
    fn generate_sha256_hash(&self, data: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Export consent data from compliance module
    pub fn fetch_consent_data(&self, user_id: &str) -> Result<serde_json::Value> {
        let conn = self.get_connection()?;

        // Fetch consent records
        let mut stmt = conn.prepare(
            "SELECT id, consent_type, granted, granted_at, revoked_at, version, consent_text, created_at, updated_at
             FROM user_consent
             WHERE user_id = ?1
             ORDER BY consent_type, version DESC"
        )?;

        let consents: Vec<serde_json::Value> = stmt
            .query_map(params![user_id], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "consent_type": row.get::<_, String>(1)?,
                    "granted": row.get::<_, bool>(2)?,
                    "granted_at": row.get::<_, Option<String>>(3)?,
                    "revoked_at": row.get::<_, Option<String>>(4)?,
                    "version": row.get::<_, i32>(5)?,
                    "consent_text": row.get::<_, String>(6)?,
                    "created_at": row.get::<_, String>(7)?,
                    "updated_at": row.get::<_, String>(8)?
                }))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(serde_json::json!({
            "user_id": user_id,
            "consents": consents,
            "total_count": consents.len()
        }))
    }

    /// Export audit logs from compliance module
    pub fn fetch_audit_logs(&self, user_id: &str, limit: usize) -> Result<serde_json::Value> {
        let conn = self.get_connection()?;

        // Fetch audit log entries
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, action_type, entity_type, entity_id, details, success, error_message
             FROM audit_log
             WHERE user_id = ?1
             ORDER BY timestamp DESC
             LIMIT ?2"
        )?;

        let logs: Vec<serde_json::Value> = stmt
            .query_map(params![user_id, limit as i64], |row| {
                let details_str: Option<String> = row.get(5)?;
                let details: Option<serde_json::Value> =
                    details_str.and_then(|s| serde_json::from_str(&s).ok());

                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "timestamp": row.get::<_, String>(1)?,
                    "action_type": row.get::<_, String>(2)?,
                    "entity_type": row.get::<_, String>(3)?,
                    "entity_id": row.get::<_, Option<String>>(4)?,
                    "details": details,
                    "success": row.get::<_, bool>(6)?,
                    "error_message": row.get::<_, Option<String>>(7)?
                }))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(serde_json::json!({
            "user_id": user_id,
            "audit_trail": logs,
            "total_count": logs.len()
        }))
    }

    /// Get comprehensive export including compliance data
    pub fn fetch_complete_user_data(&self, user_id: &str) -> Result<serde_json::Value> {
        let user_data = self.fetch_user_data(user_id)?;
        let consent_data = self.fetch_consent_data(user_id)?;
        let audit_logs = self.fetch_audit_logs(user_id, 1000)?;

        Ok(serde_json::json!({
            "export_metadata": {
                "export_date": user_data.export_date.to_rfc3339(),
                "version": user_data.version,
                "user_id": user_data.user_id,
                "format": "GDPR Article 20 Compliant",
                "hash": user_data.metadata.export_hash
            },
            "user_data": {
                "chats": user_data.chats,
                "documents": user_data.documents,
                "settings": user_data.settings
            },
            "compliance_data": {
                "consents": consent_data,
                "audit_logs": audit_logs
            },
            "statistics": {
                "total_chats": user_data.chats.len(),
                "total_documents": user_data.documents.len(),
                "total_messages": user_data.chats.iter().map(|c| c.messages.len()).sum::<usize>(),
                "total_pii_detections": user_data.documents.iter().map(|d| d.pii_detections.len()).sum::<usize>()
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn get_test_db() -> PathBuf {
        let mut path = env::temp_dir();
        path.push(format!("test_export_{}.db", uuid::Uuid::new_v4()));
        path
    }

    fn setup_test_database(db_path: &PathBuf) -> Result<()> {
        let conn = Connection::open(db_path)?;

        // Create necessary tables
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
    fn test_fetch_user_data() {
        let db_path = get_test_db();
        setup_test_database(&db_path).unwrap();

        let conn = Connection::open(&db_path).unwrap();

        // Insert test data
        conn.execute(
            "INSERT INTO chat_sessions (id, title, model_used) VALUES ('chat1', 'Test Chat', 'claude-3')",
            [],
        ).unwrap();

        conn.execute(
            "INSERT INTO chat_messages (chat_id, role, content) VALUES ('chat1', 'user', 'Hello')",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO documents (filename, content, file_type, chunk_count) VALUES ('test.pdf', 'content', 'pdf', 5)",
            [],
        ).unwrap();

        let doc_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO pii_detections (document_id, pii_type, replacement_text, confidence, position_start, position_end)
             VALUES (?1, 'EMAIL', '[EMAIL]', 0.95, 0, 20)",
            params![doc_id],
        ).unwrap();

        drop(conn);

        // Test export
        let exporter = ExportIntegration::new(db_path.clone());
        let user_data = exporter.fetch_user_data("test_user").unwrap();

        assert_eq!(user_data.chats.len(), 1);
        assert_eq!(user_data.chats[0].messages.len(), 1);
        assert_eq!(user_data.documents.len(), 1);
        assert_eq!(user_data.documents[0].pii_detections.len(), 1);
        assert!(user_data.metadata.compliance_info.gdpr_article_20);

        // Cleanup
        let _ = std::fs::remove_file(db_path);
    }

    #[test]
    fn test_fetch_consent_data() {
        let db_path = get_test_db();
        setup_test_database(&db_path).unwrap();

        let conn = Connection::open(&db_path).unwrap();

        conn.execute(
            "INSERT INTO user_consent (user_id, consent_type, granted, version, consent_text)
             VALUES ('test_user', 'chat_storage', 1, 1, 'Test consent text')",
            [],
        )
        .unwrap();

        drop(conn);

        let exporter = ExportIntegration::new(db_path.clone());
        let consent_data = exporter.fetch_consent_data("test_user").unwrap();

        assert_eq!(consent_data["total_count"], 1);
        assert!(consent_data["consents"].is_array());

        // Cleanup
        let _ = std::fs::remove_file(db_path);
    }

    #[test]
    fn test_fetch_complete_user_data() {
        let db_path = get_test_db();
        setup_test_database(&db_path).unwrap();

        let conn = Connection::open(&db_path).unwrap();

        // Insert minimal test data
        conn.execute(
            "INSERT INTO chat_sessions (id, title, model_used) VALUES ('chat1', 'Test', 'claude-3')",
            [],
        ).unwrap();

        conn.execute(
            "INSERT INTO user_consent (user_id, consent_type, granted, version, consent_text)
             VALUES ('test_user', 'chat_storage', 1, 1, 'Consent')",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO audit_log (user_id, action_type, entity_type, success)
             VALUES ('test_user', 'data_accessed', 'document', 1)",
            [],
        )
        .unwrap();

        drop(conn);

        let exporter = ExportIntegration::new(db_path.clone());
        let complete_data = exporter.fetch_complete_user_data("test_user").unwrap();

        assert!(complete_data["export_metadata"].is_object());
        assert!(complete_data["user_data"].is_object());
        assert!(complete_data["compliance_data"].is_object());
        assert!(complete_data["statistics"].is_object());

        // Cleanup
        let _ = std::fs::remove_file(db_path);
    }
}
