// GDPR Compliance Module for BEAR AI
// Implements consent management, data retention, and audit logging

pub mod consent;
pub mod retention;
pub mod audit;
pub mod commands;

pub use consent::{ConsentManager, ConsentType};
pub use retention::RetentionManager;
pub use audit::{AuditLogger, AuditAction, EntityType, AuditQuery};

use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Unified compliance manager that coordinates all GDPR features
pub struct ComplianceManager {
    consent_manager: Arc<RwLock<ConsentManager>>,
    retention_manager: Arc<RwLock<RetentionManager>>,
    audit_logger: Arc<RwLock<AuditLogger>>,
}

impl ComplianceManager {
    pub fn new(db_path: PathBuf) -> Self {
        Self {
            consent_manager: Arc::new(RwLock::new(ConsentManager::new(db_path.clone()))),
            retention_manager: Arc::new(RwLock::new(RetentionManager::new(db_path.clone()))),
            audit_logger: Arc::new(RwLock::new(AuditLogger::new(db_path))),
        }
    }

    /// Initialize all compliance modules
    pub async fn initialize(&self) -> Result<()> {
        // Initialize consent management
        let consent = self.consent_manager.write().await;
        consent.initialize()?;
        drop(consent);

        // Initialize retention management
        let retention = self.retention_manager.write().await;
        retention.initialize()?;
        drop(retention);

        // Initialize audit logging
        let audit = self.audit_logger.write().await;
        audit.initialize()?;
        drop(audit);

        // Log initialization
        let audit = self.audit_logger.write().await;
        audit.log_success(
            "system",
            AuditAction::SettingChanged,
            EntityType::UserSetting,
            None,
            Some(serde_json::json!({"action": "compliance_initialized"})),
        )?;

        Ok(())
    }

    /// Get consent manager
    pub fn consent(&self) -> Arc<RwLock<ConsentManager>> {
        self.consent_manager.clone()
    }

    /// Get retention manager
    pub fn retention(&self) -> Arc<RwLock<RetentionManager>> {
        self.retention_manager.clone()
    }

    /// Get audit logger
    pub fn audit(&self) -> Arc<RwLock<AuditLogger>> {
        self.audit_logger.clone()
    }

    /// Check if operation is allowed based on consent
    pub async fn check_operation_consent(
        &self,
        user_id: &str,
        operation: &str,
    ) -> Result<bool> {
        let consent_type = match operation {
            "pii_detection" => ConsentType::PiiDetection,
            "chat_storage" => ConsentType::ChatStorage,
            "document_processing" => ConsentType::DocumentProcessing,
            "analytics" => ConsentType::Analytics,
            _ => return Ok(true), // Unknown operations allowed by default
        };

        let consent = self.consent_manager.read().await;
        consent.has_consent(user_id, &consent_type)
    }

    /// Run periodic maintenance tasks
    pub async fn run_maintenance(&self) -> Result<serde_json::Value> {
        let mut results = serde_json::Map::new();

        // Run retention cleanup
        let retention = self.retention_manager.write().await;
        let cleanup_results = retention.run_automated_cleanup()?;
        results.insert("retention_cleanup".to_string(), cleanup_results);
        drop(retention);

        // Clean old audit logs (keep 2 years)
        let audit = self.audit_logger.write().await;
        let deleted_logs = audit.delete_old_logs(730)?;
        results.insert("audit_logs_cleaned".to_string(), serde_json::json!(deleted_logs));
        drop(audit);

        // Log maintenance completion
        let audit = self.audit_logger.write().await;
        audit.log_success(
            "system",
            AuditAction::SettingChanged,
            EntityType::UserSetting,
            None,
            Some(serde_json::json!({"action": "maintenance_completed", "results": results.clone()})),
        )?;

        Ok(serde_json::Value::Object(results))
    }

    /// Generate GDPR compliance report
    pub async fn generate_compliance_report(
        &self,
        user_id: &str,
    ) -> Result<serde_json::Value> {
        let consent = self.consent_manager.read().await;
        let consents = consent.get_user_consents(user_id)?;
        let consent_audit = consent.get_consent_audit_trail(user_id)?;
        drop(consent);

        let retention = self.retention_manager.read().await;
        let retention_stats = retention.get_retention_stats()?;
        drop(retention);

        let audit = self.audit_logger.read().await;
        let audit_logs = audit.get_user_logs(user_id, 100)?;
        let audit_stats = audit.get_audit_stats()?;
        drop(audit);

        Ok(serde_json::json!({
            "user_id": user_id,
            "report_date": chrono::Utc::now().to_rfc3339(),
            "consents": {
                "current": consents,
                "audit_trail": consent_audit
            },
            "data_retention": retention_stats,
            "audit_trail": {
                "recent_logs": audit_logs,
                "statistics": audit_stats
            }
        }))
    }

    /// Export all user data (GDPR "Right to Data Portability")
    pub async fn export_user_data(
        &self,
        user_id: &str,
    ) -> Result<serde_json::Value> {
        let consent = self.consent_manager.read().await;
        let consent_data = consent.export_consent_data(user_id)?;
        drop(consent);

        let audit_data = {
            let audit = self.audit_logger.read().await;
            audit.export_user_audit_trail(user_id)?
        };

        // Log export action
        let audit = self.audit_logger.write().await;
        audit.log_success(
            user_id,
            AuditAction::DataExported,
            EntityType::UserSetting,
            None,
            Some(serde_json::json!({"export_type": "full_user_data"})),
        )?;

        Ok(serde_json::json!({
            "user_id": user_id,
            "export_date": chrono::Utc::now().to_rfc3339(),
            "format": "JSON",
            "data": {
                "consents": consent_data,
                "audit_trail": audit_data
            }
        }))
    }

    /// Delete all user data (GDPR "Right to Erasure")
    pub async fn delete_user_data(
        &self,
        user_id: &str,
    ) -> Result<serde_json::Value> {
        let mut results = serde_json::Map::new();

        // Withdraw all consents
        let consent = self.consent_manager.write().await;
        let consents_withdrawn = consent.withdraw_all_consents(user_id)?;
        results.insert("consents_withdrawn".to_string(), serde_json::json!(consents_withdrawn));
        drop(consent);

        // Note: Actual data deletion (documents, chats, etc.) should be handled
        // by the application layer with appropriate cascading

        // Log deletion
        let audit = self.audit_logger.write().await;
        audit.log_success(
            user_id,
            AuditAction::DataDeleted,
            EntityType::UserSetting,
            None,
            Some(serde_json::json!({"deletion_type": "full_user_data"})),
        )?;

        Ok(serde_json::json!({
            "user_id": user_id,
            "deletion_date": chrono::Utc::now().to_rfc3339(),
            "results": results
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_compliance_manager() {
        let mut db_path = env::temp_dir();
        db_path.push(format!("test_compliance_{}.db", uuid::Uuid::new_v4()));

        let manager = ComplianceManager::new(db_path.clone());
        manager.initialize().await.unwrap();

        let user_id = "test_user";

        // Grant consent
        {
            let mut consent = manager.consent().write().await;
            consent.grant_consent(user_id, &ConsentType::ChatStorage).unwrap();
        }

        // Check consent
        let has_consent = manager.check_operation_consent(user_id, "chat_storage").await.unwrap();
        assert!(has_consent);

        // Generate report
        let report = manager.generate_compliance_report(user_id).await.unwrap();
        assert!(report["consents"]["current"].as_array().unwrap().len() > 0);

        // Cleanup
        let _ = std::fs::remove_file(db_path);
    }
}
