// Consent Guard Middleware - GDPR & AI Act Compliance
// Enforces consent requirements before data processing operations

use crate::compliance::{ConsentManager, ConsentType};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Consent enforcement result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentCheckResult {
    pub allowed: bool,
    pub consent_type: String,
    pub user_id: String,
    pub reason: Option<String>,
    pub requires_reconsent: bool,
}

/// Consent guard for data processing operations
pub struct ConsentGuard {
    consent_manager: Arc<RwLock<ConsentManager>>,
    strict_mode: bool,
}

impl ConsentGuard {
    /// Create new consent guard with database path
    pub fn new(db_path: PathBuf, strict_mode: bool) -> Self {
        Self {
            consent_manager: Arc::new(RwLock::new(ConsentManager::new(db_path))),
            strict_mode,
        }
    }

    /// Create from existing consent manager
    pub fn from_manager(consent_manager: Arc<RwLock<ConsentManager>>, strict_mode: bool) -> Self {
        Self {
            consent_manager,
            strict_mode,
        }
    }

    /// Initialize consent guard
    pub async fn initialize(&self) -> Result<()> {
        let manager = self.consent_manager.write().await;
        manager.initialize()
    }

    /// Check consent before chat storage operation
    pub async fn check_chat_storage(&self, user_id: &str) -> Result<ConsentCheckResult> {
        self.check_consent(user_id, &ConsentType::ChatStorage).await
    }

    /// Check consent before document processing
    pub async fn check_document_processing(&self, user_id: &str) -> Result<ConsentCheckResult> {
        self.check_consent(user_id, &ConsentType::DocumentProcessing)
            .await
    }

    /// Check consent before PII detection
    pub async fn check_pii_detection(&self, user_id: &str) -> Result<ConsentCheckResult> {
        self.check_consent(user_id, &ConsentType::PiiDetection)
            .await
    }

    /// Check consent before analytics/telemetry
    pub async fn check_analytics(&self, user_id: &str) -> Result<ConsentCheckResult> {
        self.check_consent(user_id, &ConsentType::Analytics).await
    }

    /// Check consent before AI processing
    pub async fn check_ai_processing(&self, user_id: &str) -> Result<ConsentCheckResult> {
        self.check_consent(user_id, &ConsentType::AiProcessing)
            .await
    }

    /// Check consent before data retention operations
    pub async fn check_data_retention(&self, user_id: &str) -> Result<ConsentCheckResult> {
        self.check_consent(user_id, &ConsentType::DataRetention)
            .await
    }

    /// Generic consent check with re-consent detection
    async fn check_consent(
        &self,
        user_id: &str,
        consent_type: &ConsentType,
    ) -> Result<ConsentCheckResult> {
        let manager = self.consent_manager.read().await;

        // Check if consent exists
        let has_consent = manager.has_consent(user_id, consent_type)?;

        // Check if re-consent is needed (version update)
        let needs_reconsent = manager.needs_reconsent(user_id, consent_type)?;

        let allowed = if self.strict_mode {
            // Strict mode: require valid, up-to-date consent
            has_consent && !needs_reconsent
        } else {
            // Lenient mode: allow if consent exists, even if outdated
            has_consent
        };

        let reason = if !has_consent {
            Some("No consent granted".to_string())
        } else if needs_reconsent {
            Some("Consent version outdated, re-consent required".to_string())
        } else {
            None
        };

        Ok(ConsentCheckResult {
            allowed,
            consent_type: consent_type.as_str().to_string(),
            user_id: user_id.to_string(),
            reason,
            requires_reconsent: needs_reconsent,
        })
    }

    /// Enforce consent - returns error if consent not granted
    pub async fn enforce_consent(&self, user_id: &str, consent_type: &ConsentType) -> Result<()> {
        let result = self.check_consent(user_id, consent_type).await?;

        if !result.allowed {
            return Err(anyhow!(
                "Operation denied: {} - {}",
                result.consent_type,
                result
                    .reason
                    .unwrap_or_else(|| "Unknown reason".to_string())
            ));
        }

        if result.requires_reconsent && self.strict_mode {
            return Err(anyhow!(
                "Re-consent required for {} due to version update",
                result.consent_type
            ));
        }

        Ok(())
    }

    /// Batch consent check for multiple operations
    pub async fn check_multiple_consents(
        &self,
        user_id: &str,
        consent_types: &[ConsentType],
    ) -> Result<Vec<ConsentCheckResult>> {
        let mut results = Vec::new();

        for consent_type in consent_types {
            let result = self.check_consent(user_id, consent_type).await?;
            results.push(result);
        }

        Ok(results)
    }

    /// Enforce multiple consents - all must be granted
    pub async fn enforce_multiple_consents(
        &self,
        user_id: &str,
        consent_types: &[ConsentType],
    ) -> Result<()> {
        let results = self.check_multiple_consents(user_id, consent_types).await?;

        let denied: Vec<_> = results.iter().filter(|r| !r.allowed).collect();

        if !denied.is_empty() {
            let reasons = denied
                .iter()
                .map(|r| {
                    format!(
                        "{}: {}",
                        r.consent_type,
                        r.reason.as_ref().unwrap_or(&"Unknown".to_string())
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");

            return Err(anyhow!("Multiple consent violations: {}", reasons));
        }

        Ok(())
    }

    /// Get consent manager reference for advanced operations
    pub fn consent_manager(&self) -> Arc<RwLock<ConsentManager>> {
        self.consent_manager.clone()
    }

    /// Set strict mode (requires up-to-date consent)
    pub fn set_strict_mode(&mut self, strict: bool) {
        self.strict_mode = strict;
    }

    /// Check if user needs to re-consent for any active consents
    pub async fn check_all_reconsents(&self, user_id: &str) -> Result<Vec<ConsentType>> {
        let manager = self.consent_manager.read().await;
        let mut needs_reconsent = Vec::new();

        let all_types = vec![
            ConsentType::PiiDetection,
            ConsentType::ChatStorage,
            ConsentType::DocumentProcessing,
            ConsentType::Analytics,
            ConsentType::AiProcessing,
            ConsentType::DataRetention,
        ];

        for consent_type in all_types {
            if manager.has_consent(user_id, &consent_type)? {
                if manager.needs_reconsent(user_id, &consent_type)? {
                    needs_reconsent.push(consent_type);
                }
            }
        }

        Ok(needs_reconsent)
    }

    /// Grant consent with audit logging
    pub async fn grant_consent_with_audit(
        &self,
        user_id: &str,
        consent_type: &ConsentType,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<i64> {
        let manager = self.consent_manager.write().await;

        // Grant consent
        let consent_id = manager.grant_consent(user_id, consent_type)?;

        // Log granular consent
        let version = manager
            .has_consent(user_id, consent_type)?
            .then(|| "current".to_string())
            .unwrap_or_else(|| "unknown".to_string());

        manager.log_granular_consent(
            user_id,
            consent_type,
            &version,
            true,
            ip_address,
            user_agent,
            None,
        )?;

        Ok(consent_id)
    }

    /// Revoke consent with audit logging and reason
    pub async fn revoke_consent_with_audit(
        &self,
        user_id: &str,
        consent_type: &ConsentType,
        reason: &str,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<()> {
        let manager = self.consent_manager.write().await;

        manager.withdraw_consent_with_reason(user_id, consent_type, reason, ip_address, user_agent)
    }
}

/// Consent middleware builder for easy configuration
pub struct ConsentGuardBuilder {
    db_path: PathBuf,
    strict_mode: bool,
}

impl ConsentGuardBuilder {
    pub fn new(db_path: PathBuf) -> Self {
        Self {
            db_path,
            strict_mode: true, // Default to strict mode for compliance
        }
    }

    pub fn strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }

    pub fn build(self) -> ConsentGuard {
        ConsentGuard::new(self.db_path, self.strict_mode)
    }
}

/// Helper macro for consent enforcement in functions
#[macro_export]
macro_rules! require_consent {
    ($guard:expr, $user_id:expr, $consent_type:expr) => {
        $guard.enforce_consent($user_id, &$consent_type).await?
    };
}

/// Helper macro for checking multiple consents
#[macro_export]
macro_rules! require_consents {
    ($guard:expr, $user_id:expr, [$($consent_type:expr),+]) => {
        $guard.enforce_multiple_consents($user_id, &[$($consent_type),+]).await?
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn get_test_db() -> PathBuf {
        let mut path = env::temp_dir();
        path.push(format!("test_consent_guard_{}.db", uuid::Uuid::new_v4()));
        path
    }

    #[tokio::test]
    async fn test_consent_guard_basic() {
        let db_path = get_test_db();
        let guard = ConsentGuardBuilder::new(db_path.clone())
            .strict_mode(true)
            .build();

        guard.initialize().await.unwrap();

        let user_id = "test_user";

        // Initially no consent
        let result = guard.check_chat_storage(user_id).await.unwrap();
        assert!(!result.allowed);

        // Grant consent
        let manager = guard.consent_manager.write().await;
        manager
            .grant_consent(user_id, &ConsentType::ChatStorage)
            .unwrap();
        drop(manager);

        // Now consent should be granted
        let result = guard.check_chat_storage(user_id).await.unwrap();
        assert!(result.allowed);

        // Cleanup
        let _ = std::fs::remove_file(db_path);
    }

    #[tokio::test]
    async fn test_consent_enforcement() {
        let db_path = get_test_db();
        let guard = ConsentGuardBuilder::new(db_path.clone())
            .strict_mode(true)
            .build();

        guard.initialize().await.unwrap();

        let user_id = "test_user";

        // Should fail without consent
        let result = guard
            .enforce_consent(user_id, &ConsentType::PiiDetection)
            .await;
        assert!(result.is_err());

        // Grant consent
        guard
            .grant_consent_with_audit(user_id, &ConsentType::PiiDetection, None, None)
            .await
            .unwrap();

        // Should succeed with consent
        let result = guard
            .enforce_consent(user_id, &ConsentType::PiiDetection)
            .await;
        assert!(result.is_ok());

        // Cleanup
        let _ = std::fs::remove_file(db_path);
    }

    #[tokio::test]
    async fn test_multiple_consent_checks() {
        let db_path = get_test_db();
        let guard = ConsentGuardBuilder::new(db_path.clone())
            .strict_mode(false)
            .build();

        guard.initialize().await.unwrap();

        let user_id = "test_user";

        // Grant some consents
        let manager = guard.consent_manager.write().await;
        manager
            .grant_consent(user_id, &ConsentType::ChatStorage)
            .unwrap();
        manager
            .grant_consent(user_id, &ConsentType::DocumentProcessing)
            .unwrap();
        drop(manager);

        // Check multiple
        let results = guard
            .check_multiple_consents(
                user_id,
                &[
                    ConsentType::ChatStorage,
                    ConsentType::DocumentProcessing,
                    ConsentType::Analytics,
                ],
            )
            .await
            .unwrap();

        assert_eq!(results.len(), 3);
        assert!(results[0].allowed);
        assert!(results[1].allowed);
        assert!(!results[2].allowed);

        // Cleanup
        let _ = std::fs::remove_file(db_path);
    }

    #[tokio::test]
    async fn test_strict_vs_lenient_mode() {
        let db_path = get_test_db();

        // Strict mode guard
        let strict_guard = ConsentGuardBuilder::new(db_path.clone())
            .strict_mode(true)
            .build();
        strict_guard.initialize().await.unwrap();

        let user_id = "test_user";

        // Grant consent
        let manager = strict_guard.consent_manager.write().await;
        manager
            .grant_consent(user_id, &ConsentType::ChatStorage)
            .unwrap();
        drop(manager);

        // Check with strict mode
        let result = strict_guard.check_chat_storage(user_id).await.unwrap();
        assert!(result.allowed);

        // Cleanup
        let _ = std::fs::remove_file(db_path);
    }

    #[tokio::test]
    async fn test_consent_revocation() {
        let db_path = get_test_db();
        let guard = ConsentGuardBuilder::new(db_path.clone())
            .strict_mode(true)
            .build();

        guard.initialize().await.unwrap();

        let user_id = "test_user";

        // Grant then revoke
        guard
            .grant_consent_with_audit(user_id, &ConsentType::Analytics, None, None)
            .await
            .unwrap();

        let result = guard.check_analytics(user_id).await.unwrap();
        assert!(result.allowed);

        guard
            .revoke_consent_with_audit(
                user_id,
                &ConsentType::Analytics,
                "User requested withdrawal",
                None,
                None,
            )
            .await
            .unwrap();

        let result = guard.check_analytics(user_id).await.unwrap();
        assert!(!result.allowed);

        // Cleanup
        let _ = std::fs::remove_file(db_path);
    }
}
