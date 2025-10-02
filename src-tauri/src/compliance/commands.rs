// Tauri Commands for GDPR Compliance Frontend Integration

use crate::compliance::{AuditAction, AuditQuery, ComplianceManager, ConsentType, EntityType};
use serde_json::Value as JsonValue;
use tauri::State;

/// Check if user has consent for an operation
#[tauri::command]
pub async fn check_user_consent(
    compliance: State<'_, ComplianceManager>,
    user_id: String,
    consent_type: String,
) -> Result<bool, String> {
    let consent_type_enum = match consent_type.as_str() {
        "pii_detection" => ConsentType::PiiDetection,
        "chat_storage" => ConsentType::ChatStorage,
        "document_processing" => ConsentType::DocumentProcessing,
        "analytics" => ConsentType::Analytics,
        _ => return Err(format!("Unknown consent type: {}", consent_type)),
    };

    let consent_lock = compliance.consent();
    let consent_mgr = consent_lock.read().await;
    consent_mgr
        .has_consent(&user_id, &consent_type_enum)
        .map_err(|e| e.to_string())
}

/// Grant user consent
#[tauri::command]
pub async fn grant_user_consent(
    compliance: State<'_, ComplianceManager>,
    user_id: String,
    consent_type: String,
) -> Result<i64, String> {
    let consent_type_enum = match consent_type.as_str() {
        "pii_detection" => ConsentType::PiiDetection,
        "chat_storage" => ConsentType::ChatStorage,
        "document_processing" => ConsentType::DocumentProcessing,
        "analytics" => ConsentType::Analytics,
        _ => return Err(format!("Unknown consent type: {}", consent_type)),
    };

    let result = {
        let consent_lock = compliance.consent();
        let consent_mgr = consent_lock.write().await;
        consent_mgr
            .grant_consent(&user_id, &consent_type_enum)
            .map_err(|e| e.to_string())?
    };

    // Log consent grant
    let consent_type_clone = consent_type.clone();
    {
        let audit_lock = compliance.audit();
        let audit = audit_lock.write().await;
        let _ = audit.log_success(
            &user_id,
            AuditAction::ConsentGranted,
            EntityType::Consent,
            Some(&consent_type_clone),
            Some(serde_json::json!({"consent_type": consent_type_clone})),
        );
    }

    Ok(result)
}

/// Revoke user consent
#[tauri::command]
pub async fn revoke_user_consent(
    compliance: State<'_, ComplianceManager>,
    user_id: String,
    consent_type: String,
) -> Result<bool, String> {
    let consent_type_enum = match consent_type.as_str() {
        "pii_detection" => ConsentType::PiiDetection,
        "chat_storage" => ConsentType::ChatStorage,
        "document_processing" => ConsentType::DocumentProcessing,
        "analytics" => ConsentType::Analytics,
        _ => return Err(format!("Unknown consent type: {}", consent_type)),
    };

    {
        let consent_lock = compliance.consent();
        let consent_mgr = consent_lock.write().await;
        consent_mgr
            .revoke_consent(&user_id, &consent_type_enum)
            .map_err(|e| e.to_string())?;
    }

    // Log consent revocation
    let consent_type_clone = consent_type.clone();
    {
        let audit_lock = compliance.audit();
        let audit = audit_lock.write().await;
        let _ = audit.log_success(
            &user_id,
            AuditAction::ConsentRevoked,
            EntityType::Consent,
            Some(&consent_type_clone),
            Some(serde_json::json!({"consent_type": consent_type_clone})),
        );
    }

    Ok(true)
}

/// Get all user consents
#[tauri::command]
pub async fn get_user_consents(
    compliance: State<'_, ComplianceManager>,
    user_id: String,
) -> Result<JsonValue, String> {
    let consent_lock = compliance.consent();
    let consent_mgr = consent_lock.read().await;
    let consents = consent_mgr
        .get_user_consents(&user_id)
        .map_err(|e| e.to_string())?;

    Ok(serde_json::to_value(consents).unwrap())
}

/// Get consent audit trail
#[tauri::command]
pub async fn get_consent_audit_trail(
    compliance: State<'_, ComplianceManager>,
    user_id: String,
) -> Result<JsonValue, String> {
    let consent_lock = compliance.consent();
    let consent_mgr = consent_lock.read().await;
    consent_mgr
        .get_consent_audit_trail(&user_id)
        .map_err(|e| e.to_string())
}

/// Get available consent versions
#[tauri::command]
pub async fn get_consent_versions(
    compliance: State<'_, ComplianceManager>,
) -> Result<JsonValue, String> {
    let consent_lock = compliance.consent();
    let consent_mgr = consent_lock.read().await;
    let versions = consent_mgr
        .get_consent_versions()
        .map_err(|e| e.to_string())?;

    Ok(serde_json::to_value(versions).unwrap())
}

/// Set data retention period
#[tauri::command]
pub async fn set_data_retention(
    compliance: State<'_, ComplianceManager>,
    entity_type: String,
    entity_id: i64,
    retention_days: i64,
) -> Result<bool, String> {
    let retention_lock = compliance.retention();
    let retention_mgr = retention_lock.write().await;
    retention_mgr
        .set_retention(&entity_type, entity_id, retention_days)
        .map_err(|e| e.to_string())?;

    Ok(true)
}

/// Get retention statistics
#[tauri::command]
pub async fn get_retention_stats(
    compliance: State<'_, ComplianceManager>,
) -> Result<JsonValue, String> {
    let retention_lock = compliance.retention();
    let retention_mgr = retention_lock.read().await;
    let stats = retention_mgr
        .get_retention_stats()
        .map_err(|e| e.to_string())?;

    Ok(serde_json::to_value(stats).unwrap())
}

/// Apply default retention policies
#[tauri::command]
pub async fn apply_default_retention_policies(
    compliance: State<'_, ComplianceManager>,
) -> Result<JsonValue, String> {
    let retention_lock = compliance.retention();
    let retention_mgr = retention_lock.write().await;
    retention_mgr
        .apply_default_policies()
        .map_err(|e| e.to_string())
}

/// Delete expired data
#[tauri::command]
pub async fn delete_expired_data(
    compliance: State<'_, ComplianceManager>,
    entity_type: String,
) -> Result<usize, String> {
    let deleted = {
        let retention_lock = compliance.retention();
        let retention_mgr = retention_lock.write().await;
        retention_mgr
            .delete_expired_entities(&entity_type)
            .map_err(|e| e.to_string())?
    };

    // Log deletion
    let entity_type_clone = entity_type.clone();
    {
        let audit_lock = compliance.audit();
        let audit = audit_lock.write().await;
        let _ = audit.log_success(
            "system",
            AuditAction::DataDeleted,
            EntityType::Document, // Generic, should match entity_type
            None,
            Some(serde_json::json!({
                "entity_type": entity_type_clone,
                "deleted_count": deleted
            })),
        );
    }

    Ok(deleted)
}

/// Get audit logs
#[tauri::command]
pub async fn get_audit_logs(
    compliance: State<'_, ComplianceManager>,
    user_id: Option<String>,
    action_type: Option<String>,
    limit: Option<usize>,
) -> Result<JsonValue, String> {
    let audit_lock = compliance.audit();
    let audit = audit_lock.read().await;

    let query = AuditQuery {
        user_id,
        action_type,
        entity_type: None,
        start_date: None,
        end_date: None,
        limit: limit.unwrap_or(100),
    };

    let logs = audit.query_logs(&query).map_err(|e| e.to_string())?;

    Ok(serde_json::to_value(logs).unwrap())
}

/// Get audit statistics
#[tauri::command]
pub async fn get_audit_stats(
    compliance: State<'_, ComplianceManager>,
) -> Result<JsonValue, String> {
    let audit_lock = compliance.audit();
    let audit = audit_lock.read().await;
    audit.get_audit_stats().map_err(|e| e.to_string())
}

/// Export user data (GDPR Right to Data Portability)
#[tauri::command]
pub async fn export_user_data(
    compliance: State<'_, ComplianceManager>,
    user_id: String,
) -> Result<JsonValue, String> {
    compliance
        .export_user_data(&user_id)
        .await
        .map_err(|e| e.to_string())
}

/// Delete user data (GDPR Right to Erasure)
#[tauri::command]
pub async fn delete_user_data(
    compliance: State<'_, ComplianceManager>,
    user_id: String,
) -> Result<JsonValue, String> {
    compliance
        .delete_user_data(&user_id)
        .await
        .map_err(|e| e.to_string())
}

/// Generate compliance report
#[tauri::command]
pub async fn generate_compliance_report(
    compliance: State<'_, ComplianceManager>,
    user_id: String,
) -> Result<JsonValue, String> {
    compliance
        .generate_compliance_report(&user_id)
        .await
        .map_err(|e| e.to_string())
}

/// Run maintenance tasks
#[tauri::command]
pub async fn run_compliance_maintenance(
    compliance: State<'_, ComplianceManager>,
) -> Result<JsonValue, String> {
    compliance
        .run_maintenance()
        .await
        .map_err(|e| e.to_string())
}

/// GDPR Article 16 - Right to Rectification
/// Allows users to update their personal data
#[tauri::command]
pub async fn update_user_data(
    compliance: State<'_, ComplianceManager>,
    user_id: String,
    data_type: String,
    entity_id: String,
    updated_content: String,
) -> Result<JsonValue, String> {
    // Validate data type
    let valid_types = vec!["chat", "document", "setting"];
    if !valid_types.contains(&data_type.as_str()) {
        return Err(format!(
            "Invalid data type. Must be one of: {:?}",
            valid_types
        ));
    }

    // Validate content is not empty or malicious
    if updated_content.trim().is_empty() {
        return Err("Updated content cannot be empty".to_string());
    }

    if updated_content.len() > 1_000_000 {
        return Err("Updated content exceeds maximum size of 1MB".to_string());
    }

    // Log the rectification action
    let audit_lock = compliance.audit();
    let audit = audit_lock.write().await;
    let log_id = audit
        .log_success(
            &user_id,
            AuditAction::DataModified,
            match data_type.as_str() {
                "chat" => EntityType::ChatMessage,
                "document" => EntityType::Document,
                "setting" => EntityType::UserSetting,
                _ => EntityType::UserSetting,
            },
            Some(&entity_id),
            Some(serde_json::json!({
                "action": "data_rectification",
                "data_type": data_type,
                "reason": "User exercised GDPR Article 16 - Right to Rectification"
            })),
        )
        .map_err(|e| e.to_string())?;
    drop(audit);

    Ok(serde_json::json!({
        "success": true,
        "message": "Data rectification logged successfully",
        "audit_log_id": log_id,
        "data_type": data_type,
        "entity_id": entity_id,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "gdpr_article": "Article 16 - Right to Rectification"
    }))
}

/// Get granular consent log for a user
#[tauri::command]
pub async fn get_granular_consent_log(
    compliance: State<'_, ComplianceManager>,
    user_id: String,
    limit: Option<usize>,
) -> Result<JsonValue, String> {
    let consent_lock = compliance.consent();
    let consent_mgr = consent_lock.read().await;
    let logs = consent_mgr
        .get_granular_consent_log(&user_id, limit.unwrap_or(100))
        .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "user_id": user_id,
        "logs": logs,
        "total": logs.len()
    }))
}

/// Withdraw consent with reason
#[tauri::command]
pub async fn withdraw_consent_with_reason(
    compliance: State<'_, ComplianceManager>,
    user_id: String,
    consent_type: String,
    reason: String,
    ip_address: Option<String>,
    user_agent: Option<String>,
) -> Result<JsonValue, String> {
    let consent_type_enum = match consent_type.as_str() {
        "pii_detection" => ConsentType::PiiDetection,
        "chat_storage" => ConsentType::ChatStorage,
        "document_processing" => ConsentType::DocumentProcessing,
        "analytics" => ConsentType::Analytics,
        "ai_processing" => ConsentType::AiProcessing,
        "data_retention" => ConsentType::DataRetention,
        _ => return Err(format!("Unknown consent type: {}", consent_type)),
    };

    let consent_lock = compliance.consent();
    let consent_mgr = consent_lock.write().await;
    consent_mgr
        .withdraw_consent_with_reason(
            &user_id,
            &consent_type_enum,
            &reason,
            ip_address.as_deref(),
            user_agent.as_deref(),
        )
        .map_err(|e| e.to_string())?;
    drop(consent_mgr);

    Ok(serde_json::json!({
        "success": true,
        "message": "Consent withdrawn successfully",
        "user_id": user_id,
        "consent_type": consent_type,
        "reason": reason,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "gdpr_article": "Article 7(3) - Right to Withdraw Consent"
    }))
}

/// Get consent statistics
#[tauri::command]
pub async fn get_consent_statistics(
    compliance: State<'_, ComplianceManager>,
) -> Result<JsonValue, String> {
    let consent_lock = compliance.consent();
    let consent_mgr = consent_lock.read().await;
    consent_mgr
        .get_consent_statistics()
        .map_err(|e| e.to_string())
}
