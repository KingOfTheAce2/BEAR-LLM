use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Audit action types for GDPR compliance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    ConsentGranted,
    ConsentRevoked,
    DataExported,
    DataDeleted,
    DataAccessed,
    DataModified,
    UserLogin,
    UserLogout,
    SettingChanged,
}

impl AuditAction {
    pub fn as_str(&self) -> &str {
        match self {
            AuditAction::ConsentGranted => "consent_granted",
            AuditAction::ConsentRevoked => "consent_revoked",
            AuditAction::DataExported => "data_exported",
            AuditAction::DataDeleted => "data_deleted",
            AuditAction::DataAccessed => "data_accessed",
            AuditAction::DataModified => "data_modified",
            AuditAction::UserLogin => "user_login",
            AuditAction::UserLogout => "user_logout",
            AuditAction::SettingChanged => "setting_changed",
        }
    }
}

/// Entity types for audit logging
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Document,
    ChatMessage,
    Consent,
    UserSetting,
    QueryHistory,
}

impl EntityType {
    pub fn as_str(&self) -> &str {
        match self {
            EntityType::Document => "document",
            EntityType::ChatMessage => "chat_message",
            EntityType::Consent => "consent",
            EntityType::UserSetting => "user_setting",
            EntityType::QueryHistory => "query_history",
        }
    }
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: Option<i64>,
    pub timestamp: DateTime<Utc>,
    pub user_id: String,
    pub action_type: AuditAction,
    pub entity_type: EntityType,
    pub entity_id: Option<String>,
    pub details: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Audit query filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditQuery {
    pub user_id: Option<String>,
    pub action_type: Option<String>,
    pub entity_type: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub limit: usize,
}

impl Default for AuditQuery {
    fn default() -> Self {
        Self {
            user_id: None,
            action_type: None,
            entity_type: None,
            start_date: None,
            end_date: None,
            limit: 100,
        }
    }
}

/// Audit Log Manager
pub struct AuditLogger {
    db_path: PathBuf,
}

impl AuditLogger {
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }

    /// Initialize audit log table
    pub fn initialize(&self) -> Result<()> {
        let conn = Connection::open(&self.db_path)?;

        // Run audit log migration
        let migration = include_str!("../../migrations/004_create_audit_log.sql");

        for statement in migration.split(';') {
            let trimmed = statement.trim();
            if !trimmed.is_empty() && !trimmed.starts_with("--") {
                conn.execute(trimmed, [])?;
            }
        }

        Ok(())
    }

    /// Log an audit event
    pub fn log(
        &self,
        user_id: &str,
        action: AuditAction,
        entity_type: EntityType,
        entity_id: Option<&str>,
        details: Option<serde_json::Value>,
        success: bool,
        error_message: Option<&str>,
    ) -> Result<i64> {
        let conn = Connection::open(&self.db_path)?;

        let details_json = details.map(|d| d.to_string());

        conn.execute(
            "INSERT INTO audit_log (user_id, action_type, entity_type, entity_id, details, success, error_message)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                user_id,
                action.as_str(),
                entity_type.as_str(),
                entity_id,
                details_json,
                success,
                error_message
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    /// Log successful action (convenience method)
    pub fn log_success(
        &self,
        user_id: &str,
        action: AuditAction,
        entity_type: EntityType,
        entity_id: Option<&str>,
        details: Option<serde_json::Value>,
    ) -> Result<i64> {
        self.log(user_id, action, entity_type, entity_id, details, true, None)
    }

    /// Log failed action (convenience method)
    pub fn log_failure(
        &self,
        user_id: &str,
        action: AuditAction,
        entity_type: EntityType,
        entity_id: Option<&str>,
        error: &str,
    ) -> Result<i64> {
        self.log(
            user_id,
            action,
            entity_type,
            entity_id,
            None,
            false,
            Some(error),
        )
    }

    /// Query audit logs with filters
    pub fn query_logs(&self, query: &AuditQuery) -> Result<Vec<AuditLogEntry>> {
        let conn = Connection::open(&self.db_path)?;

        let mut sql = String::from(
            "SELECT id, timestamp, user_id, action_type, entity_type, entity_id,
                    details, ip_address, user_agent, success, error_message
             FROM audit_log WHERE 1=1",
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref user_id) = query.user_id {
            sql.push_str(" AND user_id = ?");
            params.push(Box::new(user_id.clone()));
        }

        if let Some(ref action_type) = query.action_type {
            sql.push_str(" AND action_type = ?");
            params.push(Box::new(action_type.clone()));
        }

        if let Some(ref entity_type) = query.entity_type {
            sql.push_str(" AND entity_type = ?");
            params.push(Box::new(entity_type.clone()));
        }

        if let Some(ref start_date) = query.start_date {
            sql.push_str(" AND timestamp >= ?");
            params.push(Box::new(start_date.to_rfc3339()));
        }

        if let Some(ref end_date) = query.end_date {
            sql.push_str(" AND timestamp <= ?");
            params.push(Box::new(end_date.to_rfc3339()));
        }

        sql.push_str(" ORDER BY timestamp DESC LIMIT ?");
        params.push(Box::new(query.limit as i64));

        let mut stmt = conn.prepare(&sql)?;

        let param_refs: Vec<&dyn rusqlite::ToSql> = params
            .iter()
            .map(|p| p.as_ref() as &dyn rusqlite::ToSql)
            .collect();

        let entries = stmt
            .query_map(&param_refs[..], |row| {
                let details_str: Option<String> = row.get(6)?;
                let details = details_str.and_then(|s| serde_json::from_str(&s).ok());

                Ok(AuditLogEntry {
                    id: row.get(0)?,
                    timestamp: row.get::<_, String>(1)?.parse().unwrap(),
                    user_id: row.get(2)?,
                    action_type: serde_json::from_str(&format!("\"{}\"", row.get::<_, String>(3)?))
                        .unwrap(),
                    entity_type: serde_json::from_str(&format!("\"{}\"", row.get::<_, String>(4)?))
                        .unwrap(),
                    entity_id: row.get(5)?,
                    details,
                    ip_address: row.get(7)?,
                    user_agent: row.get(8)?,
                    success: row.get(9)?,
                    error_message: row.get(10)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    /// Get audit logs for specific user
    pub fn get_user_logs(&self, user_id: &str, limit: usize) -> Result<Vec<AuditLogEntry>> {
        self.query_logs(&AuditQuery {
            user_id: Some(user_id.to_string()),
            limit,
            ..Default::default()
        })
    }

    /// Get audit logs for specific entity
    pub fn get_entity_logs(
        &self,
        entity_type: EntityType,
        _entity_id: &str,
        limit: usize,
    ) -> Result<Vec<AuditLogEntry>> {
        self.query_logs(&AuditQuery {
            entity_type: Some(entity_type.as_str().to_string()),
            limit,
            ..Default::default()
        })
    }

    /// Export audit logs for a user (GDPR compliance)
    pub fn export_user_audit_trail(&self, user_id: &str) -> Result<serde_json::Value> {
        let logs = self.get_user_logs(user_id, 10000)?; // Get all logs

        Ok(serde_json::json!({
            "user_id": user_id,
            "export_date": Utc::now().to_rfc3339(),
            "total_entries": logs.len(),
            "audit_trail": logs
        }))
    }

    /// Get audit statistics
    pub fn get_audit_stats(&self) -> Result<serde_json::Value> {
        let conn = Connection::open(&self.db_path)?;

        let total_entries: i64 =
            conn.query_row("SELECT COUNT(*) FROM audit_log", [], |row| row.get(0))?;

        let success_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM audit_log WHERE success = 1",
            [],
            |row| row.get(0),
        )?;

        let failure_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM audit_log WHERE success = 0",
            [],
            |row| row.get(0),
        )?;

        // Action type distribution
        let mut stmt = conn.prepare(
            "SELECT action_type, COUNT(*) as count
             FROM audit_log
             GROUP BY action_type
             ORDER BY count DESC",
        )?;

        let action_distribution: Vec<serde_json::Value> = stmt
            .query_map([], |row| {
                Ok(serde_json::json!({
                    "action": row.get::<_, String>(0)?,
                    "count": row.get::<_, i64>(1)?
                }))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(serde_json::json!({
            "total_entries": total_entries,
            "success_count": success_count,
            "failure_count": failure_count,
            "success_rate": if total_entries > 0 {
                (success_count as f64 / total_entries as f64) * 100.0
            } else {
                0.0
            },
            "action_distribution": action_distribution
        }))
    }

    /// Delete old audit logs (for retention)
    pub fn delete_old_logs(&self, days_to_keep: i64) -> Result<usize> {
        let conn = Connection::open(&self.db_path)?;
        let cutoff_date = Utc::now() - chrono::Duration::days(days_to_keep);

        let count = conn.execute(
            "DELETE FROM audit_log WHERE timestamp < ?1",
            params![cutoff_date.to_rfc3339()],
        )?;

        Ok(count)
    }

    /// Search audit logs by keyword in details
    pub fn search_logs(&self, keyword: &str, limit: usize) -> Result<Vec<AuditLogEntry>> {
        let conn = Connection::open(&self.db_path)?;

        let search_term = format!("%{}%", keyword);

        let mut stmt = conn.prepare(
            "SELECT id, timestamp, user_id, action_type, entity_type, entity_id,
                    details, ip_address, user_agent, success, error_message
             FROM audit_log
             WHERE details LIKE ?1 OR error_message LIKE ?1
             ORDER BY timestamp DESC
             LIMIT ?2",
        )?;

        let entries = stmt
            .query_map(params![search_term, limit as i64], |row| {
                let details_str: Option<String> = row.get(6)?;
                let details = details_str.and_then(|s| serde_json::from_str(&s).ok());

                Ok(AuditLogEntry {
                    id: row.get(0)?,
                    timestamp: row.get::<_, String>(1)?.parse().unwrap(),
                    user_id: row.get(2)?,
                    action_type: serde_json::from_str(&format!("\"{}\"", row.get::<_, String>(3)?))
                        .unwrap(),
                    entity_type: serde_json::from_str(&format!("\"{}\"", row.get::<_, String>(4)?))
                        .unwrap(),
                    entity_id: row.get(5)?,
                    details,
                    ip_address: row.get(7)?,
                    user_agent: row.get(8)?,
                    success: row.get(9)?,
                    error_message: row.get(10)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn get_test_db() -> PathBuf {
        let mut path = env::temp_dir();
        path.push(format!("test_audit_{}.db", uuid::Uuid::new_v4()));
        path
    }

    #[test]
    fn test_audit_logging() {
        let db_path = get_test_db();
        let logger = AuditLogger::new(db_path.clone());
        logger.initialize().unwrap();

        // Log successful action
        logger
            .log_success(
                "test_user",
                AuditAction::ConsentGranted,
                EntityType::Consent,
                Some("consent_123"),
                Some(serde_json::json!({"consent_type": "chat_storage"})),
            )
            .unwrap();

        // Log failed action
        logger
            .log_failure(
                "test_user",
                AuditAction::DataExported,
                EntityType::Document,
                Some("doc_456"),
                "Export failed: disk full",
            )
            .unwrap();

        // Query logs
        let logs = logger.get_user_logs("test_user", 10).unwrap();
        assert_eq!(logs.len(), 2);

        // Get stats
        let stats = logger.get_audit_stats().unwrap();
        assert_eq!(stats["total_entries"], 2);

        // Cleanup
        let _ = std::fs::remove_file(db_path);
    }
}
