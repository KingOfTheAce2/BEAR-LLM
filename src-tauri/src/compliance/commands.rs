// Tauri Commands for GDPR Compliance Frontend Integration

use tauri::State;
use serde_json::Value as JsonValue;
use crate::compliance::{ComplianceManager, ConsentType, AuditAction, EntityType, AuditQuery};

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

    let consent_mgr = compliance.consent().read().await;
    consent_mgr.has_consent(&user_id, &consent_type_enum)
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
        let consent_mgr = compliance.consent().write().await;
        consent_mgr.grant_consent(&user_id, &consent_type_enum)
            .map_err(|e| e.to_string())?
    };

    // Log consent grant
    let consent_type_clone = consent_type.clone();
    {
        let audit = compliance.audit().write().await;
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
        let consent_mgr = compliance.consent().write().await;
        consent_mgr.revoke_consent(&user_id, &consent_type_enum)
            .map_err(|e| e.to_string())?;
    }

    // Log consent revocation
    let consent_type_clone = consent_type.clone();
    {
        let audit = compliance.audit().write().await;
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
    let consent_mgr = compliance.consent().read().await;
    let consents = consent_mgr.get_user_consents(&user_id)
        .map_err(|e| e.to_string())?;

    Ok(serde_json::to_value(consents).unwrap())
}

/// Get consent audit trail
#[tauri::command]
pub async fn get_consent_audit_trail(
    compliance: State<'_, ComplianceManager>,
    user_id: String,
) -> Result<JsonValue, String> {
    let consent_mgr = compliance.consent().read().await;
    consent_mgr.get_consent_audit_trail(&user_id)
        .map_err(|e| e.to_string())
}

/// Get available consent versions
#[tauri::command]
pub async fn get_consent_versions(
    compliance: State<'_, ComplianceManager>,
) -> Result<JsonValue, String> {
    let consent_mgr = compliance.consent().read().await;
    let versions = consent_mgr.get_consent_versions()
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
    let retention_mgr = compliance.retention().write().await;
    retention_mgr.set_retention(&entity_type, entity_id, retention_days)
        .map_err(|e| e.to_string())?;

    Ok(true)
}

/// Get retention statistics
#[tauri::command]
pub async fn get_retention_stats(
    compliance: State<'_, ComplianceManager>,
) -> Result<JsonValue, String> {
    let retention_mgr = compliance.retention().read().await;
    let stats = retention_mgr.get_retention_stats()
        .map_err(|e| e.to_string())?;

    Ok(serde_json::to_value(stats).unwrap())
}

/// Apply default retention policies
#[tauri::command]
pub async fn apply_default_retention_policies(
    compliance: State<'_, ComplianceManager>,
) -> Result<JsonValue, String> {
    let retention_mgr = compliance.retention().write().await;
    retention_mgr.apply_default_policies()
        .map_err(|e| e.to_string())
}

/// Delete expired data
#[tauri::command]
pub async fn delete_expired_data(
    compliance: State<'_, ComplianceManager>,
    entity_type: String,
) -> Result<usize, String> {
    let deleted = {
        let retention_mgr = compliance.retention().write().await;
        retention_mgr.delete_expired_entities(&entity_type)
            .map_err(|e| e.to_string())?
    };

    // Log deletion
    let entity_type_clone = entity_type.clone();
    {
        let audit = compliance.audit().write().await;
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
    let audit = compliance.audit().read().await;

    let query = AuditQuery {
        user_id,
        action_type,
        entity_type: None,
        start_date: None,
        end_date: None,
        limit: limit.unwrap_or(100),
    };

    let logs = audit.query_logs(&query)
        .map_err(|e| e.to_string())?;

    Ok(serde_json::to_value(logs).unwrap())
}

/// Get audit statistics
#[tauri::command]
pub async fn get_audit_stats(
    compliance: State<'_, ComplianceManager>,
) -> Result<JsonValue, String> {
    let audit = compliance.audit().read().await;
    audit.get_audit_stats()
        .map_err(|e| e.to_string())
}

/// Export user data (GDPR Right to Data Portability)
#[tauri::command]
pub async fn export_user_data(
    compliance: State<'_, ComplianceManager>,
    user_id: String,
) -> Result<JsonValue, String> {
    compliance.export_user_data(&user_id)
        .await
        .map_err(|e| e.to_string())
}

/// Delete user data (GDPR Right to Erasure)
#[tauri::command]
pub async fn delete_user_data(
    compliance: State<'_, ComplianceManager>,
    user_id: String,
) -> Result<JsonValue, String> {
    compliance.delete_user_data(&user_id)
        .await
        .map_err(|e| e.to_string())
}

/// Generate compliance report
#[tauri::command]
pub async fn generate_compliance_report(
    compliance: State<'_, ComplianceManager>,
    user_id: String,
) -> Result<JsonValue, String> {
    compliance.generate_compliance_report(&user_id)
        .await
        .map_err(|e| e.to_string())
}

/// Run maintenance tasks
#[tauri::command]
pub async fn run_compliance_maintenance(
    compliance: State<'_, ComplianceManager>,
) -> Result<JsonValue, String> {
    compliance.run_maintenance()
        .await
        .map_err(|e| e.to_string())
}
