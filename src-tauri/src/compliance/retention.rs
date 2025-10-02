use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Data retention policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub entity_type: String, // 'document', 'chat_message', 'query_history'
    pub retention_days: i64, // How many days to retain
    pub auto_delete: bool,   // Enable automatic deletion
}

/// Default retention policies (GDPR recommends minimal retention)
impl RetentionPolicy {
    pub fn default_policies() -> Vec<Self> {
        vec![
            RetentionPolicy {
                entity_type: "document".to_string(),
                retention_days: 365 * 2, // 2 years for documents
                auto_delete: false,      // Don't auto-delete documents by default
            },
            RetentionPolicy {
                entity_type: "chat_message".to_string(),
                retention_days: 90, // 90 days for chat history
                auto_delete: true,  // Auto-delete old chats
            },
            RetentionPolicy {
                entity_type: "query_history".to_string(),
                retention_days: 30, // 30 days for query logs
                auto_delete: true,
            },
        ]
    }
}

/// Retention statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct RetentionStats {
    pub entity_type: String,
    pub total_count: i64,
    pub expired_count: i64,
    pub pending_deletion: Vec<i64>,
}

/// Data Retention Manager
pub struct RetentionManager {
    db_path: PathBuf,
}

impl RetentionManager {
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }

    /// Initialize retention columns
    pub fn initialize(&self) -> Result<()> {
        let conn = Connection::open(&self.db_path)?;

        // Run retention migration
        let migration = include_str!("../../migrations/003_add_retention_columns.sql");

        for statement in migration.split(';') {
            let trimmed = statement.trim();
            if !trimmed.is_empty() && !trimmed.starts_with("--") {
                conn.execute(trimmed, [])?;
            }
        }

        Ok(())
    }

    /// Set retention period for an entity
    pub fn set_retention(
        &self,
        entity_type: &str,
        entity_id: i64,
        retention_days: i64,
    ) -> Result<()> {
        let conn = Connection::open(&self.db_path)?;
        let retention_until = Utc::now() + ChronoDuration::days(retention_days);

        let table = match entity_type {
            "document" => "documents",
            "chat_session" => "chat_sessions",
            "chat_message" => "chat_messages",
            "query_history" => "query_history",
            _ => return Err(anyhow!("Unknown entity type: {}", entity_type)),
        };

        let query = format!("UPDATE {} SET retention_until = ?1 WHERE id = ?2", table);

        conn.execute(&query, params![retention_until.to_rfc3339(), entity_id])?;

        Ok(())
    }

    /// Set retention for all entities of a type
    pub fn set_retention_policy(&self, entity_type: &str, retention_days: i64) -> Result<usize> {
        let conn = Connection::open(&self.db_path)?;
        let retention_until = Utc::now() + ChronoDuration::days(retention_days);

        let table = match entity_type {
            "document" => "documents",
            "chat_session" => "chat_sessions",
            "chat_message" => "chat_messages",
            "query_history" => "query_history",
            _ => return Err(anyhow!("Unknown entity type: {}", entity_type)),
        };

        let query = format!(
            "UPDATE {} SET retention_until = ?1 WHERE retention_until IS NULL",
            table
        );

        let count = conn.execute(&query, params![retention_until.to_rfc3339()])?;

        Ok(count)
    }

    /// Get entities pending deletion
    pub fn get_expired_entities(&self, entity_type: &str) -> Result<Vec<i64>> {
        let conn = Connection::open(&self.db_path)?;
        let now = Utc::now().to_rfc3339();

        let table = match entity_type {
            "document" => "documents",
            "chat_session" => "chat_sessions",
            "chat_message" => "chat_messages",
            "query_history" => "query_history",
            _ => return Err(anyhow!("Unknown entity type: {}", entity_type)),
        };

        let query = format!(
            "SELECT id FROM {} WHERE retention_until IS NOT NULL AND retention_until < ?1",
            table
        );

        let mut stmt = conn.prepare(&query)?;
        let ids: Vec<i64> = stmt
            .query_map(params![now], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ids)
    }

    /// Delete expired entities (GDPR automated deletion)
    pub fn delete_expired_entities(&self, entity_type: &str) -> Result<usize> {
        let expired_ids = self.get_expired_entities(entity_type)?;

        if expired_ids.is_empty() {
            return Ok(0);
        }

        let conn = Connection::open(&self.db_path)?;
        let now = Utc::now().to_rfc3339();

        let table = match entity_type {
            "document" => {
                // Also delete related data
                conn.execute(
                    "DELETE FROM document_chunks WHERE document_id IN (
                        SELECT id FROM documents WHERE retention_until < ?1
                    )",
                    params![now],
                )?;
                conn.execute(
                    "DELETE FROM pii_detections WHERE document_id IN (
                        SELECT id FROM documents WHERE retention_until < ?1
                    )",
                    params![now],
                )?;
                "documents"
            }
            "chat_session" => {
                // Delete related messages
                conn.execute(
                    "DELETE FROM chat_messages WHERE chat_id IN (
                        SELECT id FROM chat_sessions WHERE retention_until < ?1
                    )",
                    params![now],
                )?;
                "chat_sessions"
            }
            "chat_message" => "chat_messages",
            "query_history" => "query_history",
            _ => return Err(anyhow!("Unknown entity type: {}", entity_type)),
        };

        let query = format!(
            "DELETE FROM {} WHERE retention_until IS NOT NULL AND retention_until < ?1",
            table
        );

        let count = conn.execute(&query, params![now])?;

        Ok(count)
    }

    /// Get retention statistics for all entity types
    pub fn get_retention_stats(&self) -> Result<Vec<RetentionStats>> {
        let entity_types = vec!["document", "chat_session", "chat_message", "query_history"];
        let mut stats = Vec::new();

        for entity_type in entity_types {
            let stat = self.get_entity_retention_stats(entity_type)?;
            stats.push(stat);
        }

        Ok(stats)
    }

    /// Get retention statistics for specific entity type
    fn get_entity_retention_stats(&self, entity_type: &str) -> Result<RetentionStats> {
        let conn = Connection::open(&self.db_path)?;
        let now = Utc::now().to_rfc3339();

        let table = match entity_type {
            "document" => "documents",
            "chat_session" => "chat_sessions",
            "chat_message" => "chat_messages",
            "query_history" => "query_history",
            _ => return Err(anyhow!("Unknown entity type: {}", entity_type)),
        };

        // Total count
        let total_count: i64 =
            conn.query_row(&format!("SELECT COUNT(*) FROM {}", table), [], |row| {
                row.get(0)
            })?;

        // Expired count
        let expired_count: i64 = conn.query_row(
            &format!(
                "SELECT COUNT(*) FROM {} WHERE retention_until IS NOT NULL AND retention_until < ?1",
                table
            ),
            params![now],
            |row| row.get(0),
        )?;

        // Pending deletion IDs
        let pending_deletion = self.get_expired_entities(entity_type)?;

        Ok(RetentionStats {
            entity_type: entity_type.to_string(),
            total_count,
            expired_count,
            pending_deletion,
        })
    }

    /// Clear retention period (keep indefinitely)
    pub fn clear_retention(&self, entity_type: &str, entity_id: i64) -> Result<()> {
        let conn = Connection::open(&self.db_path)?;

        let table = match entity_type {
            "document" => "documents",
            "chat_session" => "chat_sessions",
            "chat_message" => "chat_messages",
            "query_history" => "query_history",
            _ => return Err(anyhow!("Unknown entity type: {}", entity_type)),
        };

        let query = format!("UPDATE {} SET retention_until = NULL WHERE id = ?1", table);

        conn.execute(&query, params![entity_id])?;

        Ok(())
    }

    /// Run automated cleanup (should be called periodically)
    pub fn run_automated_cleanup(&self) -> Result<serde_json::Value> {
        let mut results = serde_json::Map::new();
        let entity_types = vec!["document", "chat_session", "chat_message", "query_history"];

        for entity_type in entity_types {
            let deleted_count = self.delete_expired_entities(entity_type)?;
            results.insert(entity_type.to_string(), serde_json::json!(deleted_count));
        }

        Ok(serde_json::Value::Object(results))
    }

    /// Apply default retention policies
    pub fn apply_default_policies(&self) -> Result<serde_json::Value> {
        let policies = RetentionPolicy::default_policies();
        let mut results = serde_json::Map::new();

        for policy in policies {
            if policy.auto_delete {
                let count =
                    self.set_retention_policy(&policy.entity_type, policy.retention_days)?;
                results.insert(
                    policy.entity_type.clone(),
                    serde_json::json!({
                        "entities_updated": count,
                        "retention_days": policy.retention_days
                    }),
                );
            }
        }

        Ok(serde_json::Value::Object(results))
    }

    /// Extend retention for specific entity (user requested extension)
    pub fn extend_retention(
        &self,
        entity_type: &str,
        entity_id: i64,
        additional_days: i64,
    ) -> Result<()> {
        let conn = Connection::open(&self.db_path)?;

        let table = match entity_type {
            "document" => "documents",
            "chat_session" => "chat_sessions",
            "chat_message" => "chat_messages",
            "query_history" => "query_history",
            _ => return Err(anyhow!("Unknown entity type: {}", entity_type)),
        };

        // Get current retention date or use now
        let query = format!("SELECT retention_until FROM {} WHERE id = ?1", table);
        let current: Option<String> = conn
            .query_row(&query, params![entity_id], |row| row.get(0))
            .ok();

        let new_retention = if let Some(current_str) = current {
            let current_date: DateTime<Utc> = current_str.parse()?;
            current_date + ChronoDuration::days(additional_days)
        } else {
            Utc::now() + ChronoDuration::days(additional_days)
        };

        let update_query = format!("UPDATE {} SET retention_until = ?1 WHERE id = ?2", table);

        conn.execute(
            &update_query,
            params![new_retention.to_rfc3339(), entity_id],
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn get_test_db() -> PathBuf {
        let mut path = env::temp_dir();
        path.push(format!("test_retention_{}.db", uuid::Uuid::new_v4()));
        path
    }

    #[test]
    fn test_retention_lifecycle() {
        let db_path = get_test_db();

        // Initialize database with documents table
        {
            let conn = Connection::open(&db_path).unwrap();
            conn.execute(
                "CREATE TABLE documents (
                    id INTEGER PRIMARY KEY,
                    filename TEXT,
                    content TEXT,
                    file_type TEXT,
                    retention_until DATETIME
                )",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO documents (filename, content, file_type) VALUES ('test.txt', 'content', 'txt')",
                [],
            ).unwrap();
        }

        let manager = RetentionManager::new(db_path.clone());

        // Set retention
        manager.set_retention("document", 1, 30).unwrap();

        // Get stats
        let stats = manager.get_entity_retention_stats("document").unwrap();
        assert_eq!(stats.total_count, 1);

        // Cleanup
        let _ = std::fs::remove_file(db_path);
    }
}
