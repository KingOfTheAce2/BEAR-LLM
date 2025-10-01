use anyhow::{Result, anyhow};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::path::PathBuf;

/// Consent types for GDPR compliance
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConsentType {
    PiiDetection,
    ChatStorage,
    DocumentProcessing,
    Analytics,
}

impl ConsentType {
    pub fn as_str(&self) -> &str {
        match self {
            ConsentType::PiiDetection => "pii_detection",
            ConsentType::ChatStorage => "chat_storage",
            ConsentType::DocumentProcessing => "document_processing",
            ConsentType::Analytics => "analytics",
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "pii_detection" => Ok(ConsentType::PiiDetection),
            "chat_storage" => Ok(ConsentType::ChatStorage),
            "document_processing" => Ok(ConsentType::DocumentProcessing),
            "analytics" => Ok(ConsentType::Analytics),
            _ => Err(anyhow!("Unknown consent type: {}", s)),
        }
    }
}

/// Consent record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRecord {
    pub id: Option<i64>,
    pub user_id: String,
    pub consent_type: ConsentType,
    pub granted: bool,
    pub granted_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub version: i32,
    pub consent_text: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Consent version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentVersion {
    pub id: Option<i64>,
    pub consent_type: ConsentType,
    pub version: i32,
    pub consent_text: String,
    pub effective_date: DateTime<Utc>,
    pub deprecated_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Consent Manager - handles all consent operations
pub struct ConsentManager {
    db_path: PathBuf,
}

impl ConsentManager {
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }

    /// Initialize consent tables from migrations
    pub fn initialize(&self) -> Result<()> {
        let conn = Connection::open(&self.db_path)?;

        // Run all migration files
        let migrations = vec![
            include_str!("../../migrations/001_create_user_consent.sql"),
            include_str!("../../migrations/002_create_consent_versions.sql"),
            include_str!("../../migrations/004_create_audit_log.sql"),
        ];

        for migration in migrations {
            // Split by semicolon and execute each statement
            for statement in migration.split(';') {
                let trimmed = statement.trim();
                if !trimmed.is_empty() && !trimmed.starts_with("--") {
                    conn.execute(trimmed, [])?;
                }
            }
        }

        Ok(())
    }

    /// Check if user has granted consent for a specific type
    pub fn has_consent(&self, user_id: &str, consent_type: &ConsentType) -> Result<bool> {
        let conn = Connection::open(&self.db_path)?;

        let granted: Option<bool> = conn.query_row(
            "SELECT granted FROM user_consent
             WHERE user_id = ?1 AND consent_type = ?2
             AND revoked_at IS NULL
             ORDER BY version DESC LIMIT 1",
            params![user_id, consent_type.as_str()],
            |row| row.get(0),
        ).ok();

        Ok(granted.unwrap_or(false))
    }

    /// Grant consent for a specific type
    pub fn grant_consent(
        &self,
        user_id: &str,
        consent_type: &ConsentType
    ) -> Result<i64> {
        let conn = Connection::open(&self.db_path)?;

        // Get current version
        let version = self.get_current_version(consent_type)?;
        let consent_text = self.get_consent_text(consent_type, version)?;

        // Check if consent already exists
        let existing: Option<i64> = conn.query_row(
            "SELECT id FROM user_consent
             WHERE user_id = ?1 AND consent_type = ?2 AND version = ?3",
            params![user_id, consent_type.as_str(), version],
            |row| row.get(0),
        ).ok();

        if let Some(id) = existing {
            // Update existing consent
            conn.execute(
                "UPDATE user_consent
                 SET granted = 1, granted_at = datetime('now'), revoked_at = NULL, updated_at = datetime('now')
                 WHERE id = ?1",
                params![id],
            )?;
            Ok(id)
        } else {
            // Insert new consent
            conn.execute(
                "INSERT INTO user_consent (user_id, consent_type, granted, granted_at, version, consent_text)
                 VALUES (?1, ?2, 1, datetime('now'), ?3, ?4)",
                params![user_id, consent_type.as_str(), version, consent_text],
            )?;
            Ok(conn.last_insert_rowid())
        }
    }

    /// Revoke consent for a specific type
    pub fn revoke_consent(
        &self,
        user_id: &str,
        consent_type: &ConsentType
    ) -> Result<()> {
        let conn = Connection::open(&self.db_path)?;

        conn.execute(
            "UPDATE user_consent
             SET granted = 0, revoked_at = datetime('now'), updated_at = datetime('now')
             WHERE user_id = ?1 AND consent_type = ?2 AND granted = 1 AND revoked_at IS NULL",
            params![user_id, consent_type.as_str()],
        )?;

        Ok(())
    }

    /// Get all consent records for a user
    pub fn get_user_consents(&self, user_id: &str) -> Result<Vec<ConsentRecord>> {
        let conn = Connection::open(&self.db_path)?;

        let mut stmt = conn.prepare(
            "SELECT id, user_id, consent_type, granted, granted_at, revoked_at,
                    version, consent_text, created_at, updated_at
             FROM user_consent
             WHERE user_id = ?1
             ORDER BY consent_type, version DESC"
        )?;

        let records = stmt.query_map(params![user_id], |row| {
            Ok(ConsentRecord {
                id: row.get(0)?,
                user_id: row.get(1)?,
                consent_type: ConsentType::from_str(&row.get::<_, String>(2)?).unwrap(),
                granted: row.get(3)?,
                granted_at: row.get::<_, Option<String>>(4)?.map(|s| s.parse().unwrap()),
                revoked_at: row.get::<_, Option<String>>(5)?.map(|s| s.parse().unwrap()),
                version: row.get(6)?,
                consent_text: row.get(7)?,
                created_at: row.get::<_, String>(8)?.parse().unwrap(),
                updated_at: row.get::<_, String>(9)?.parse().unwrap(),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(records)
    }

    /// Get consent audit trail for a user
    pub fn get_consent_audit_trail(&self, user_id: &str) -> Result<serde_json::Value> {
        let conn = Connection::open(&self.db_path)?;

        let mut stmt = conn.prepare(
            "SELECT consent_type, granted, granted_at, revoked_at, version, consent_text
             FROM user_consent
             WHERE user_id = ?1
             ORDER BY updated_at DESC"
        )?;

        let trail: Vec<serde_json::Value> = stmt.query_map(params![user_id], |row| {
            Ok(serde_json::json!({
                "consent_type": row.get::<_, String>(1)?,
                "granted": row.get::<_, bool>(2)?,
                "granted_at": row.get::<_, Option<String>>(3)?,
                "revoked_at": row.get::<_, Option<String>>(4)?,
                "version": row.get::<_, i32>(5)?,
                "consent_text": row.get::<_, String>(6)?
            }))
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(serde_json::json!({
            "user_id": user_id,
            "audit_trail": trail,
            "total_records": trail.len()
        }))
    }

    /// Get current version for consent type
    fn get_current_version(&self, consent_type: &ConsentType) -> Result<i32> {
        let conn = Connection::open(&self.db_path)?;

        let version: i32 = conn.query_row(
            "SELECT version FROM consent_versions
             WHERE consent_type = ?1 AND deprecated_date IS NULL
             ORDER BY effective_date DESC LIMIT 1",
            params![consent_type.as_str()],
            |row| row.get(0),
        )?;

        Ok(version)
    }

    /// Get consent text for specific type and version
    fn get_consent_text(&self, consent_type: &ConsentType, version: i32) -> Result<String> {
        let conn = Connection::open(&self.db_path)?;

        let text: String = conn.query_row(
            "SELECT consent_text FROM consent_versions
             WHERE consent_type = ?1 AND version = ?2",
            params![consent_type.as_str(), version],
            |row| row.get(0),
        )?;

        Ok(text)
    }

    /// Get all available consent versions
    pub fn get_consent_versions(&self) -> Result<Vec<ConsentVersion>> {
        let conn = Connection::open(&self.db_path)?;

        let mut stmt = conn.prepare(
            "SELECT id, consent_type, version, consent_text, effective_date, deprecated_date, created_at
             FROM consent_versions
             WHERE deprecated_date IS NULL
             ORDER BY consent_type, version DESC"
        )?;

        let versions = stmt.query_map([], |row| {
            Ok(ConsentVersion {
                id: row.get(0)?,
                consent_type: ConsentType::from_str(&row.get::<_, String>(1)?).unwrap(),
                version: row.get(2)?,
                consent_text: row.get(3)?,
                effective_date: row.get::<_, String>(4)?.parse().unwrap(),
                deprecated_date: row.get::<_, Option<String>>(5)?.map(|s| s.parse().unwrap()),
                created_at: row.get::<_, String>(6)?.parse().unwrap(),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(versions)
    }

    /// Check if user must re-consent due to version update
    pub fn needs_reconsent(&self, user_id: &str, consent_type: &ConsentType) -> Result<bool> {
        let current_version = self.get_current_version(consent_type)?;

        let conn = Connection::open(&self.db_path)?;
        let user_version: Option<i32> = conn.query_row(
            "SELECT version FROM user_consent
             WHERE user_id = ?1 AND consent_type = ?2 AND granted = 1 AND revoked_at IS NULL
             ORDER BY version DESC LIMIT 1",
            params![user_id, consent_type.as_str()],
            |row| row.get(0),
        ).ok();

        Ok(user_version.map(|v| v < current_version).unwrap_or(true))
    }

    /// Withdraw all consents for a user (GDPR "right to withdraw consent")
    pub fn withdraw_all_consents(&self, user_id: &str) -> Result<usize> {
        let conn = Connection::open(&self.db_path)?;

        let count = conn.execute(
            "UPDATE user_consent
             SET granted = 0, revoked_at = datetime('now'), updated_at = datetime('now')
             WHERE user_id = ?1 AND granted = 1 AND revoked_at IS NULL",
            params![user_id],
        )?;

        Ok(count)
    }

    /// Export consent data for a user (GDPR "right to data portability")
    pub fn export_consent_data(&self, user_id: &str) -> Result<serde_json::Value> {
        let consents = self.get_user_consents(user_id)?;

        Ok(serde_json::json!({
            "user_id": user_id,
            "export_date": Utc::now().to_rfc3339(),
            "consents": consents,
            "total_consents": consents.len()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn get_test_db() -> PathBuf {
        let mut path = env::temp_dir();
        path.push(format!("test_consent_{}.db", uuid::Uuid::new_v4()));
        path
    }

    #[test]
    fn test_consent_lifecycle() {
        let db_path = get_test_db();
        let manager = ConsentManager::new(db_path.clone());
        manager.initialize().unwrap();

        let user_id = "test_user";
        let consent_type = ConsentType::ChatStorage;

        // Initially no consent
        assert!(!manager.has_consent(user_id, &consent_type).unwrap());

        // Grant consent
        manager.grant_consent(user_id, &consent_type).unwrap();
        assert!(manager.has_consent(user_id, &consent_type).unwrap());

        // Revoke consent
        manager.revoke_consent(user_id, &consent_type).unwrap();
        assert!(!manager.has_consent(user_id, &consent_type).unwrap());

        // Cleanup
        let _ = std::fs::remove_file(db_path);
    }

    #[test]
    fn test_consent_audit_trail() {
        let db_path = get_test_db();
        let manager = ConsentManager::new(db_path.clone());
        manager.initialize().unwrap();

        let user_id = "test_user";
        let consent_type = ConsentType::DocumentProcessing;

        manager.grant_consent(user_id, &consent_type).unwrap();
        manager.revoke_consent(user_id, &consent_type).unwrap();
        manager.grant_consent(user_id, &consent_type).unwrap();

        let trail = manager.get_consent_audit_trail(user_id).unwrap();
        let records = trail["audit_trail"].as_array().unwrap();
        assert!(records.len() >= 2);

        // Cleanup
        let _ = std::fs::remove_file(db_path);
    }
}
