use anyhow::Result;
use std::path::PathBuf;
use tracing::{info, warn};
use crate::compliance::retention::RetentionManager;

/// Task for executing retention cleanup operations
pub struct RetentionCleanupTask {
    db_path: PathBuf,
}

impl RetentionCleanupTask {
    /// Create a new retention cleanup task
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }

    /// Execute the cleanup task
    /// Returns tuple of (documents, sessions, messages, queries) deleted
    pub async fn execute(&self) -> Result<(usize, usize, usize, usize)> {
        info!("Starting automated retention cleanup");

        // Create retention manager
        let manager = RetentionManager::new(self.db_path.clone());

        // Execute cleanup for each entity type
        let documents_deleted = self.cleanup_entity_type(&manager, "document").await?;
        let sessions_deleted = self.cleanup_entity_type(&manager, "chat_session").await?;
        let messages_deleted = self.cleanup_entity_type(&manager, "chat_message").await?;
        let queries_deleted = self.cleanup_entity_type(&manager, "query_history").await?;

        let total = documents_deleted + sessions_deleted + messages_deleted + queries_deleted;
        info!(
            "Retention cleanup completed: {} total entities deleted",
            total
        );

        Ok((documents_deleted, sessions_deleted, messages_deleted, queries_deleted))
    }

    /// Cleanup expired entities of a specific type
    async fn cleanup_entity_type(
        &self,
        manager: &RetentionManager,
        entity_type: &str,
    ) -> Result<usize> {
        // Get expired entities first (for logging)
        let expired_ids = manager.get_expired_entities(entity_type)?;

        if expired_ids.is_empty() {
            info!("No expired {} entities found", entity_type);
            return Ok(0);
        }

        info!(
            "Found {} expired {} entities, deleting...",
            expired_ids.len(),
            entity_type
        );

        // Execute deletion
        let deleted_count = manager.delete_expired_entities(entity_type)?;

        if deleted_count != expired_ids.len() {
            warn!(
                "Mismatch in deletion count for {}: expected {}, deleted {}",
                entity_type,
                expired_ids.len(),
                deleted_count
            );
        }

        Ok(deleted_count)
    }

    /// Get cleanup preview without deleting
    pub async fn preview_cleanup(&self) -> Result<CleanupPreview> {
        let manager = RetentionManager::new(self.db_path.clone());

        let documents = manager.get_expired_entities("document")?;
        let sessions = manager.get_expired_entities("chat_session")?;
        let messages = manager.get_expired_entities("chat_message")?;
        let queries = manager.get_expired_entities("query_history")?;

        Ok(CleanupPreview {
            documents_to_delete: documents.len(),
            sessions_to_delete: sessions.len(),
            messages_to_delete: messages.len(),
            queries_to_delete: queries.len(),
            total: documents.len() + sessions.len() + messages.len() + queries.len(),
        })
    }

    /// Apply default retention policies to all entities
    pub async fn apply_default_policies(&self) -> Result<PolicyApplicationResult> {
        info!("Applying default retention policies");

        let manager = RetentionManager::new(self.db_path.clone());
        let result = manager.apply_default_policies()?;

        info!("Default policies applied: {:?}", result);

        // Parse the result into structured data
        let result_map = result.as_object().ok_or_else(|| {
            anyhow::anyhow!("Invalid result format from apply_default_policies")
        })?;

        let mut policies_applied = 0;
        for value in result_map.values() {
            if let Some(obj) = value.as_object() {
                if let Some(count) = obj.get("entities_updated") {
                    if let Some(num) = count.as_i64() {
                        policies_applied += num as usize;
                    }
                }
            }
        }

        Ok(PolicyApplicationResult {
            policies_applied,
            details: result,
        })
    }
}

/// Preview of what would be deleted
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CleanupPreview {
    pub documents_to_delete: usize,
    pub sessions_to_delete: usize,
    pub messages_to_delete: usize,
    pub queries_to_delete: usize,
    pub total: usize,
}

/// Result of applying retention policies
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PolicyApplicationResult {
    pub policies_applied: usize,
    pub details: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn get_test_db() -> PathBuf {
        let mut path = env::temp_dir();
        path.push(format!("test_retention_tasks_{}.db", uuid::Uuid::new_v4()));
        path
    }

    #[tokio::test]
    async fn test_cleanup_task_creation() {
        let db_path = get_test_db();
        let task = RetentionCleanupTask::new(db_path);

        // Task should be created successfully
        assert!(task.db_path.to_str().is_some());
    }

    #[tokio::test]
    async fn test_preview_cleanup_empty() {
        let db_path = get_test_db();

        // Initialize empty database
        {
            use rusqlite::Connection;
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
            ).unwrap();
            conn.execute(
                "CREATE TABLE chat_sessions (
                    id INTEGER PRIMARY KEY,
                    name TEXT,
                    retention_until DATETIME
                )",
                [],
            ).unwrap();
            conn.execute(
                "CREATE TABLE chat_messages (
                    id INTEGER PRIMARY KEY,
                    chat_id INTEGER,
                    content TEXT,
                    retention_until DATETIME
                )",
                [],
            ).unwrap();
            conn.execute(
                "CREATE TABLE query_history (
                    id INTEGER PRIMARY KEY,
                    query TEXT,
                    retention_until DATETIME
                )",
                [],
            ).unwrap();
        }

        let task = RetentionCleanupTask::new(db_path.clone());
        let preview = task.preview_cleanup().await.unwrap();

        assert_eq!(preview.total, 0);
        assert_eq!(preview.documents_to_delete, 0);

        // Cleanup
        let _ = std::fs::remove_file(db_path);
    }
}
