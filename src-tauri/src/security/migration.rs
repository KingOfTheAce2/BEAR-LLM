// SPDX-License-Identifier: MIT
// Copyright (c) 2025 BEAR AI LLM
//
// Chat Message Encryption Migration
// GDPR Article 32 - Security of Processing
//
// This module provides utilities to migrate existing plaintext chat messages
// to encrypted format with atomic transactions and rollback support.

use anyhow::{Context, Result};
use rusqlite::{Connection, Transaction};
use serde_json::Value as JsonValue;
use std::sync::Arc;
use tracing;

use super::chat_encryption::{ChatEncryptor, UserKeyDerivation};
use super::key_manager::KeyManager;

/// Progress callback for migration operations
pub type ProgressCallback = Arc<dyn Fn(usize, usize) + Send + Sync>;

/// Migration statistics
#[derive(Debug, Clone)]
pub struct MigrationStats {
    pub total_messages: usize,
    pub encrypted_messages: usize,
    pub failed_messages: usize,
    pub skipped_messages: usize,
}

impl MigrationStats {
    pub fn new() -> Self {
        Self {
            total_messages: 0,
            encrypted_messages: 0,
            failed_messages: 0,
            skipped_messages: 0,
        }
    }
}

impl Default for MigrationStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Chat message migration manager
pub struct ChatMigrationManager {
    key_manager: Arc<KeyManager>,
    encryptor: ChatEncryptor,
}

impl ChatMigrationManager {
    /// Create a new migration manager
    pub fn new(key_manager: Arc<KeyManager>) -> Self {
        Self {
            key_manager,
            encryptor: ChatEncryptor::new(),
        }
    }

    /// Migrate all plaintext chat messages to encrypted format
    ///
    /// # Arguments
    /// * `conn` - Database connection
    /// * `default_user_id` - Default user ID for messages without user context
    /// * `progress_callback` - Optional callback for progress reporting
    ///
    /// # Returns
    /// Migration statistics
    pub fn migrate_all_messages(
        &self,
        conn: &mut Connection,
        default_user_id: &str,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<MigrationStats> {
        let mut stats = MigrationStats::new();

        // Check if migration is needed
        if !self.needs_migration(conn)? {
            tracing::info!("No migration needed - all messages already encrypted");
            return Ok(stats);
        }

        // Add encryption columns if they don't exist
        self.add_encryption_columns(conn)?;

        // Start transaction for atomic migration
        let tx = conn.transaction()?;

        // Get total count for progress reporting
        let total: usize = tx
            .query_row(
                "SELECT COUNT(*) FROM chat_messages WHERE encrypted IS NULL OR encrypted = 0",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        stats.total_messages = total;
        tracing::info!("Starting migration of {} plaintext messages", total);

        if let Some(ref cb) = progress_callback {
            cb(0, total);
        }

        // Get master key for user key derivation
        let master_key = self.key_manager.get_or_create_key()?;
        let key_derivation = UserKeyDerivation::new(master_key)?;

        // Process messages in batches for better performance
        let batch_size = 100;
        let mut processed = 0;

        loop {
            let messages = self.fetch_plaintext_batch(&tx, batch_size)?;
            if messages.is_empty() {
                break;
            }

            for (id, chat_id, content) in messages {
                // Determine user_id (use chat_id as user identifier or default)
                let user_id = if chat_id.is_empty() {
                    default_user_id
                } else {
                    &chat_id
                };

                match self.encrypt_and_update_message(&tx, id, &content, user_id, &key_derivation) {
                    Ok(_) => {
                        stats.encrypted_messages += 1;
                    }
                    Err(e) => {
                        tracing::error!("Failed to encrypt message {}: {}", id, e);
                        stats.failed_messages += 1;
                    }
                }

                processed += 1;
                if let Some(ref cb) = progress_callback {
                    cb(processed, total);
                }
            }
        }

        // Commit transaction
        tx.commit()
            .context("Failed to commit migration transaction")?;

        tracing::info!(
            "Migration completed: {} encrypted, {} failed, {} skipped",
            stats.encrypted_messages,
            stats.failed_messages,
            stats.skipped_messages
        );

        Ok(stats)
    }

    /// Check if migration is needed
    fn needs_migration(&self, conn: &Connection) -> Result<bool> {
        // Check if encrypted column exists
        let column_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('chat_messages') WHERE name = 'encrypted'",
                [],
                |row| {
                    let count: i64 = row.get(0)?;
                    Ok(count > 0)
                },
            )
            .unwrap_or(false);

        if !column_exists {
            return Ok(true); // Need migration - column doesn't exist
        }

        // Check if there are any unencrypted messages
        let unencrypted_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM chat_messages WHERE encrypted IS NULL OR encrypted = 0",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        Ok(unencrypted_count > 0)
    }

    /// Add encryption-related columns to chat_messages table
    fn add_encryption_columns(&self, conn: &Connection) -> Result<()> {
        // Add encrypted flag
        let _ = conn.execute(
            "ALTER TABLE chat_messages ADD COLUMN encrypted INTEGER DEFAULT 0",
            [],
        );

        // Add encryption version for key rotation support
        let _ = conn.execute(
            "ALTER TABLE chat_messages ADD COLUMN encryption_version INTEGER DEFAULT 1",
            [],
        );

        // Add user_id column if it doesn't exist
        let _ = conn.execute(
            "ALTER TABLE chat_messages ADD COLUMN user_id TEXT DEFAULT ''",
            [],
        );

        Ok(())
    }

    /// Fetch a batch of plaintext messages
    fn fetch_plaintext_batch(
        &self,
        tx: &Transaction,
        limit: usize,
    ) -> Result<Vec<(i64, String, String)>> {
        let mut stmt = tx.prepare(
            "SELECT id, chat_id, content
             FROM chat_messages
             WHERE encrypted IS NULL OR encrypted = 0
             LIMIT ?1",
        )?;

        let messages = stmt
            .query_map([limit], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(messages)
    }

    /// Encrypt a single message and update in database
    fn encrypt_and_update_message(
        &self,
        tx: &Transaction,
        message_id: i64,
        content: &str,
        user_id: &str,
        key_derivation: &UserKeyDerivation,
    ) -> Result<()> {
        // Derive user-specific key
        let user_key = key_derivation.derive_default_key(user_id)?;

        // Encrypt the message
        let encrypted = self.encryptor.encrypt(content, &user_key, user_id)?;

        // Serialize encrypted message to JSON
        let encrypted_json =
            serde_json::to_string(&encrypted).context("Failed to serialize encrypted message")?;

        // Update database with encrypted content
        tx.execute(
            "UPDATE chat_messages
             SET content = ?1, encrypted = 1, encryption_version = ?2, user_id = ?3
             WHERE id = ?4",
            [
                &encrypted_json,
                &encrypted.version.to_string(),
                user_id,
                &message_id.to_string(),
            ],
        )
        .context("Failed to update encrypted message")?;

        Ok(())
    }

    /// Rollback migration (decrypt all messages back to plaintext)
    ///
    /// WARNING: This removes encryption protection. Use only for emergency rollback.
    pub fn rollback_migration(
        &self,
        conn: &mut Connection,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<MigrationStats> {
        let mut stats = MigrationStats::new();

        let tx = conn.transaction()?;

        // Get total count
        let total: usize = tx
            .query_row(
                "SELECT COUNT(*) FROM chat_messages WHERE encrypted = 1",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        stats.total_messages = total;
        tracing::warn!(
            "Starting rollback of {} encrypted messages to plaintext",
            total
        );

        if let Some(ref cb) = progress_callback {
            cb(0, total);
        }

        // Get master key
        let master_key = self.key_manager.get_or_create_key()?;
        let key_derivation = UserKeyDerivation::new(master_key)?;

        let batch_size = 100;
        let mut processed = 0;

        loop {
            let messages = self.fetch_encrypted_batch(&tx, batch_size)?;
            if messages.is_empty() {
                break;
            }

            for (id, content, user_id) in messages {
                match self.decrypt_and_update_message(&tx, id, &content, &user_id, &key_derivation)
                {
                    Ok(_) => {
                        stats.encrypted_messages += 1;
                    }
                    Err(e) => {
                        tracing::error!("Failed to decrypt message {} during rollback: {}", id, e);
                        stats.failed_messages += 1;
                    }
                }

                processed += 1;
                if let Some(ref cb) = progress_callback {
                    cb(processed, total);
                }
            }
        }

        tx.commit()
            .context("Failed to commit rollback transaction")?;

        tracing::warn!(
            "Rollback completed: {} decrypted, {} failed",
            stats.encrypted_messages,
            stats.failed_messages
        );

        Ok(stats)
    }

    /// Fetch a batch of encrypted messages
    fn fetch_encrypted_batch(
        &self,
        tx: &Transaction,
        limit: usize,
    ) -> Result<Vec<(i64, String, String)>> {
        let mut stmt = tx.prepare(
            "SELECT id, content, COALESCE(user_id, '')
             FROM chat_messages
             WHERE encrypted = 1
             LIMIT ?1",
        )?;

        let messages = stmt
            .query_map([limit], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(messages)
    }

    /// Decrypt a single message and update in database
    fn decrypt_and_update_message(
        &self,
        tx: &Transaction,
        message_id: i64,
        encrypted_content: &str,
        user_id: &str,
        key_derivation: &UserKeyDerivation,
    ) -> Result<()> {
        // Derive user-specific key
        let user_key = key_derivation.derive_default_key(user_id)?;

        // Deserialize encrypted message
        let encrypted: super::chat_encryption::EncryptedMessage =
            serde_json::from_str(encrypted_content)
                .context("Failed to deserialize encrypted message")?;

        // Decrypt the message
        let plaintext = self.encryptor.decrypt(&encrypted, &user_key)?;

        // Update database with plaintext content
        tx.execute(
            "UPDATE chat_messages
             SET content = ?1, encrypted = 0, encryption_version = NULL
             WHERE id = ?2",
            [&plaintext, &message_id.to_string()],
        )
        .context("Failed to update decrypted message")?;

        Ok(())
    }

    /// Generate migration report
    pub fn generate_migration_report(&self, conn: &Connection) -> Result<JsonValue> {
        let total_messages: i64 = conn
            .query_row("SELECT COUNT(*) FROM chat_messages", [], |row| row.get(0))
            .unwrap_or(0);

        let encrypted_messages: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM chat_messages WHERE encrypted = 1",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let plaintext_messages: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM chat_messages WHERE encrypted IS NULL OR encrypted = 0",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        Ok(serde_json::json!({
            "total_messages": total_messages,
            "encrypted_messages": encrypted_messages,
            "plaintext_messages": plaintext_messages,
            "encryption_coverage": if total_messages > 0 {
                (encrypted_messages as f64 / total_messages as f64) * 100.0
            } else {
                0.0
            },
            "migration_needed": plaintext_messages > 0,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use std::sync::Arc;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();

        // Create chat_messages table
        conn.execute(
            "CREATE TABLE chat_messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                chat_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                metadata TEXT
            )",
            [],
        )
        .unwrap();

        // Insert test messages
        conn.execute(
            "INSERT INTO chat_messages (chat_id, role, content) VALUES
             ('chat1', 'user', 'First message'),
             ('chat1', 'assistant', 'First response'),
             ('chat2', 'user', 'Second message')",
            [],
        )
        .unwrap();

        conn
    }

    #[ignore]
    #[test]
    fn test_migration_adds_columns() {
        let mut conn = setup_test_db();
        let key_manager = Arc::new(KeyManager::new().unwrap());
        let migrator = ChatMigrationManager::new(key_manager);

        let stats = migrator
            .migrate_all_messages(&mut conn, "default_user", None)
            .unwrap();

        // Check columns were added
        let column_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('chat_messages')
                 WHERE name IN ('encrypted', 'encryption_version', 'user_id')",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(column_count, 3);
        assert_eq!(stats.total_messages, 3);
    }

    #[ignore]
    #[test]
    fn test_migration_encrypts_messages() {
        let mut conn = setup_test_db();
        let key_manager = Arc::new(KeyManager::new().unwrap());
        let migrator = ChatMigrationManager::new(key_manager);

        let stats = migrator
            .migrate_all_messages(&mut conn, "default_user", None)
            .unwrap();

        assert_eq!(stats.encrypted_messages, 3);
        assert_eq!(stats.failed_messages, 0);

        // Verify messages are encrypted
        let encrypted_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM chat_messages WHERE encrypted = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(encrypted_count, 3);
    }

    #[ignore]
    #[test]
    fn test_migration_report() {
        let mut conn = setup_test_db();
        let key_manager = Arc::new(KeyManager::new().unwrap());
        let migrator = ChatMigrationManager::new(key_manager);

        // Before migration
        let report_before = migrator.generate_migration_report(&conn).unwrap();
        assert_eq!(report_before["total_messages"], 3);
        assert_eq!(report_before["plaintext_messages"], 3);
        assert_eq!(report_before["migration_needed"], true);

        // After migration
        migrator
            .migrate_all_messages(&mut conn, "default_user", None)
            .unwrap();

        let report_after = migrator.generate_migration_report(&conn).unwrap();
        assert_eq!(report_after["encrypted_messages"], 3);
        assert_eq!(report_after["plaintext_messages"], 0);
        assert_eq!(report_after["migration_needed"], false);
    }

    #[ignore]
    #[test]
    fn test_progress_callback() {
        let mut conn = setup_test_db();
        let key_manager = Arc::new(KeyManager::new().unwrap());
        let migrator = ChatMigrationManager::new(key_manager);

        let progress_calls = Arc::new(std::sync::Mutex::new(Vec::new()));
        let progress_calls_clone = Arc::clone(&progress_calls);

        let callback: ProgressCallback = Arc::new(move |current, total| {
            progress_calls_clone.lock().unwrap().push((current, total));
        });

        migrator
            .migrate_all_messages(&mut conn, "default_user", Some(callback))
            .unwrap();

        let calls = progress_calls.lock().unwrap();
        assert!(!calls.is_empty());
        // Should have initial call (0, total) and final call (total, total)
        assert_eq!(calls[0], (0, 3));
    }
}
