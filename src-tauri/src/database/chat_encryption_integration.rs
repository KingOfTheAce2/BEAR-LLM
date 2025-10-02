// SPDX-License-Identifier: MIT
// Copyright (c) 2025 BEAR AI LLM
//
// Database Integration for Chat Encryption
// GDPR Article 32 - Security of Processing
//
// This module provides seamless integration between the database layer
// and chat message encryption, enabling transparent encryption/decryption.

use anyhow::{Context, Result};
use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use serde_json::Value as JsonValue;
use std::sync::Arc;
use tracing;

use bear_ai_llm::security::{ChatEncryptor, KeyManager, UserKeyDerivation};

/// Chat message encryption wrapper for database operations
pub struct ChatEncryptionLayer {
    key_manager: Arc<KeyManager>,
    encryptor: ChatEncryptor,
}

impl ChatEncryptionLayer {
    /// Create a new encryption layer
    pub fn new(key_manager: Arc<KeyManager>) -> Result<Self> {
        Ok(Self {
            key_manager,
            encryptor: ChatEncryptor::new(),
        })
    }

    /// Store an encrypted chat message
    ///
    /// # Arguments
    /// * `conn` - Database connection
    /// * `chat_id` - Chat session ID
    /// * `role` - Message role (user/assistant)
    /// * `content` - Plaintext message content
    /// * `user_id` - User identifier for key derivation
    /// * `metadata` - Optional metadata (also encrypted if contains sensitive data)
    pub fn store_encrypted_message(
        &self,
        conn: &PooledConnection<SqliteConnectionManager>,
        chat_id: &str,
        role: &str,
        content: &str,
        user_id: &str,
        metadata: Option<&str>,
    ) -> Result<i64> {
        // Derive user-specific encryption key
        let master_key = self.key_manager.get_or_create_key()?;
        let key_derivation = UserKeyDerivation::new(master_key)?;
        let user_key = key_derivation.derive_default_key(user_id)?;

        // Encrypt the message content
        let encrypted = self.encryptor.encrypt(content, &user_key, user_id)?;
        let encrypted_json =
            serde_json::to_string(&encrypted).context("Failed to serialize encrypted message")?;

        // Encrypt metadata if provided
        let encrypted_metadata = if let Some(meta) = metadata {
            let meta_encrypted = self.encryptor.encrypt(meta, &user_key, user_id)?;
            Some(
                serde_json::to_string(&meta_encrypted)
                    .context("Failed to serialize encrypted metadata")?,
            )
        } else {
            None
        };

        // Insert into database
        conn.execute(
            "INSERT INTO chat_messages (chat_id, role, content, user_id, encrypted, encryption_version, metadata)
             VALUES (?1, ?2, ?3, ?4, 1, ?5, ?6)",
            [
                chat_id,
                role,
                &encrypted_json,
                user_id,
                &encrypted.version.to_string(),
                encrypted_metadata.as_deref().unwrap_or(""),
            ],
        )
        .context("Failed to insert encrypted message")?;

        Ok(conn.last_insert_rowid())
    }

    /// Retrieve and decrypt a chat message
    ///
    /// # Arguments
    /// * `conn` - Database connection
    /// * `message_id` - Message ID to retrieve
    ///
    /// # Returns
    /// Tuple of (chat_id, role, decrypted_content, metadata)
    pub fn retrieve_decrypted_message(
        &self,
        conn: &PooledConnection<SqliteConnectionManager>,
        message_id: i64,
    ) -> Result<(String, String, String, Option<String>)> {
        let (chat_id, role, content, user_id, encrypted, metadata): (
            String,
            String,
            String,
            String,
            bool,
            Option<String>,
        ) = conn
            .query_row(
                "SELECT chat_id, role, content, COALESCE(user_id, ''), COALESCE(encrypted, 0), metadata
                 FROM chat_messages WHERE id = ?1",
                [message_id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                    ))
                },
            )
            .context("Failed to retrieve message from database")?;

        // If message is encrypted, decrypt it
        let decrypted_content = if encrypted {
            let master_key = self.key_manager.get_or_create_key()?;
            let key_derivation = UserKeyDerivation::new(master_key)?;
            let user_key = key_derivation.derive_default_key(&user_id)?;

            match self.encryptor.decrypt_from_json(&content, &user_key) {
                Ok(plaintext) => plaintext,
                Err(e) => {
                    tracing::error!(
                        "Failed to decrypt message {}: {}. Returning error placeholder.",
                        message_id,
                        e
                    );
                    // Return error message instead of crashing
                    format!("[DECRYPTION FAILED: {}]", e)
                }
            }
        } else {
            // Legacy plaintext message
            tracing::warn!("Message {} is not encrypted (legacy format)", message_id);
            content
        };

        // Decrypt metadata if present and encrypted
        let decrypted_metadata = if let Some(meta) = metadata {
            if encrypted && !meta.is_empty() {
                let master_key = self.key_manager.get_or_create_key()?;
                let key_derivation = UserKeyDerivation::new(master_key)?;
                let user_key = key_derivation.derive_default_key(&user_id)?;

                match self.encryptor.decrypt_from_json(&meta, &user_key) {
                    Ok(plaintext) => Some(plaintext),
                    Err(e) => {
                        tracing::error!(
                            "Failed to decrypt metadata for message {}: {}",
                            message_id,
                            e
                        );
                        None
                    }
                }
            } else {
                Some(meta)
            }
        } else {
            None
        };

        Ok((chat_id, role, decrypted_content, decrypted_metadata))
    }

    /// Retrieve all messages for a chat session (decrypted)
    pub fn retrieve_chat_session_messages(
        &self,
        conn: &PooledConnection<SqliteConnectionManager>,
        chat_id: &str,
    ) -> Result<Vec<JsonValue>> {
        let mut stmt = conn.prepare(
            "SELECT id, role, content, user_id, COALESCE(encrypted, 0), timestamp, metadata
             FROM chat_messages
             WHERE chat_id = ?1
             ORDER BY timestamp ASC",
        )?;

        let master_key = self.key_manager.get_or_create_key()?;
        let key_derivation = UserKeyDerivation::new(master_key)?;

        let messages = stmt
            .query_map([chat_id], |row| {
                let id: i64 = row.get(0)?;
                let role: String = row.get(1)?;
                let content: String = row.get(2)?;
                let user_id: String = row.get(3)?;
                let encrypted: bool = row.get(4)?;
                let timestamp: String = row.get(5)?;
                let metadata: Option<String> = row.get(6)?;

                Ok((id, role, content, user_id, encrypted, timestamp, metadata))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let mut decrypted_messages = Vec::new();

        for (id, role, content, user_id, encrypted, timestamp, metadata) in messages {
            // Decrypt content if encrypted
            let decrypted_content = if encrypted {
                let user_key = key_derivation
                    .derive_default_key(&user_id)
                    .unwrap_or_else(|e| {
                        tracing::error!("Failed to derive key for user {}: {}", user_id, e);
                        vec![0u8; 32] // Fallback (will fail decryption)
                    });

                match self.encryptor.decrypt_from_json(&content, &user_key) {
                    Ok(plaintext) => plaintext,
                    Err(e) => {
                        tracing::error!("Failed to decrypt message {}: {}", id, e);
                        format!("[DECRYPTION FAILED: Message ID {}]", id)
                    }
                }
            } else {
                content
            };

            // Decrypt metadata if present
            let decrypted_metadata = if let Some(meta) = metadata {
                if encrypted && !meta.is_empty() {
                    let user_key = key_derivation
                        .derive_default_key(&user_id)
                        .unwrap_or_else(|_| vec![0u8; 32]);

                    self.encryptor.decrypt_from_json(&meta, &user_key).ok()
                } else {
                    Some(meta)
                }
            } else {
                None
            };

            decrypted_messages.push(serde_json::json!({
                "id": id,
                "role": role,
                "content": decrypted_content,
                "timestamp": timestamp,
                "encrypted": encrypted,
                "metadata": decrypted_metadata
            }));
        }

        Ok(decrypted_messages)
    }

    /// Get encryption statistics for monitoring
    pub fn get_encryption_stats(
        &self,
        conn: &PooledConnection<SqliteConnectionManager>,
    ) -> Result<JsonValue> {
        let total: i64 = conn
            .query_row("SELECT COUNT(*) FROM chat_messages", [], |row| row.get(0))
            .unwrap_or(0);

        let encrypted: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM chat_messages WHERE encrypted = 1",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let plaintext: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM chat_messages WHERE encrypted IS NULL OR encrypted = 0",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        Ok(serde_json::json!({
            "total_messages": total,
            "encrypted_messages": encrypted,
            "plaintext_messages": plaintext,
            "encryption_percentage": if total > 0 {
                (encrypted as f64 / total as f64) * 100.0
            } else {
                0.0
            },
            "encryption_enabled": true,
            "encryption_algorithm": "AES-256-GCM",
            "key_derivation": "Argon2id"
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use r2d2::Pool;
    use std::sync::Arc;

    fn setup_test_db() -> Pool<SqliteConnectionManager> {
        let manager = SqliteConnectionManager::memory();
        let pool = Pool::builder().max_size(1).build(manager).unwrap();

        let conn = pool.get().unwrap();
        conn.execute(
            "CREATE TABLE chat_messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                chat_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                metadata TEXT,
                user_id TEXT DEFAULT '',
                encrypted INTEGER DEFAULT 0,
                encryption_version INTEGER
            )",
            [],
        )
        .unwrap();

        pool
    }

    #[test]
    fn test_store_and_retrieve_encrypted_message() {
        let pool = setup_test_db();
        let conn = pool.get().unwrap();

        let key_manager = Arc::new(KeyManager::new().unwrap());
        let encryption_layer = ChatEncryptionLayer::new(key_manager).unwrap();

        let message_id = encryption_layer
            .store_encrypted_message(
                &conn,
                "chat123",
                "user",
                "Sensitive legal advice",
                "user123",
                None,
            )
            .unwrap();

        let (chat_id, role, content, _) = encryption_layer
            .retrieve_decrypted_message(&conn, message_id)
            .unwrap();

        assert_eq!(chat_id, "chat123");
        assert_eq!(role, "user");
        assert_eq!(content, "Sensitive legal advice");
    }

    #[test]
    fn test_retrieve_chat_session() {
        let pool = setup_test_db();
        let conn = pool.get().unwrap();

        let key_manager = Arc::new(KeyManager::new().unwrap());
        let encryption_layer = ChatEncryptionLayer::new(key_manager).unwrap();

        // Store multiple messages
        encryption_layer
            .store_encrypted_message(&conn, "chat1", "user", "Message 1", "user1", None)
            .unwrap();
        encryption_layer
            .store_encrypted_message(&conn, "chat1", "assistant", "Response 1", "user1", None)
            .unwrap();
        encryption_layer
            .store_encrypted_message(&conn, "chat1", "user", "Message 2", "user1", None)
            .unwrap();

        let messages = encryption_layer
            .retrieve_chat_session_messages(&conn, "chat1")
            .unwrap();

        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0]["content"], "Message 1");
        assert_eq!(messages[1]["content"], "Response 1");
        assert_eq!(messages[2]["content"], "Message 2");
    }

    #[test]
    fn test_encryption_stats() {
        let pool = setup_test_db();
        let conn = pool.get().unwrap();

        let key_manager = Arc::new(KeyManager::new().unwrap());
        let encryption_layer = ChatEncryptionLayer::new(key_manager).unwrap();

        // Store encrypted messages
        encryption_layer
            .store_encrypted_message(&conn, "chat1", "user", "Message 1", "user1", None)
            .unwrap();
        encryption_layer
            .store_encrypted_message(&conn, "chat1", "user", "Message 2", "user1", None)
            .unwrap();

        let stats = encryption_layer.get_encryption_stats(&conn).unwrap();

        assert_eq!(stats["total_messages"], 2);
        assert_eq!(stats["encrypted_messages"], 2);
        assert_eq!(stats["plaintext_messages"], 0);
        assert_eq!(stats["encryption_percentage"], 100.0);
    }
}
