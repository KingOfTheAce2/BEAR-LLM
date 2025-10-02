// Tauri Commands for Consent Management
// Provides frontend integration for GDPR consent operations

use crate::compliance::ConsentType;
use crate::middleware::ConsentGuard;
use serde_json::Value as JsonValue;
use tauri::State;

/// Check consent status for a specific operation
#[tauri::command]
pub async fn check_consent_status(
    consent_guard: State<'_, ConsentGuard>,
    user_id: String,
    consent_type: String,
) -> Result<JsonValue, String> {
    let _consent_type_enum = parse_consent_type(&consent_type)?;

    let result = match consent_type.as_str() {
        "chat_storage" => consent_guard.check_chat_storage(&user_id).await,
        "document_processing" => consent_guard.check_document_processing(&user_id).await,
        "pii_detection" => consent_guard.check_pii_detection(&user_id).await,
        "analytics" => consent_guard.check_analytics(&user_id).await,
        "ai_processing" => consent_guard.check_ai_processing(&user_id).await,
        "data_retention" => consent_guard.check_data_retention(&user_id).await,
        _ => return Err(format!("Unknown consent type: {}", consent_type)),
    };

    result
        .map(|r| serde_json::to_value(r).unwrap())
        .map_err(|e| e.to_string())
}

/// Grant consent for a specific operation
#[tauri::command]
pub async fn grant_consent(
    consent_guard: State<'_, ConsentGuard>,
    user_id: String,
    consent_type: String,
    ip_address: Option<String>,
    user_agent: Option<String>,
) -> Result<JsonValue, String> {
    let consent_type_enum = parse_consent_type(&consent_type)?;

    let consent_id = consent_guard
        .grant_consent_with_audit(
            &user_id,
            &consent_type_enum,
            ip_address.as_deref(),
            user_agent.as_deref(),
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "success": true,
        "consent_id": consent_id,
        "user_id": user_id,
        "consent_type": consent_type,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "gdpr_article": "Article 7 - Conditions for consent"
    }))
}

/// Revoke consent for a specific operation
#[tauri::command]
pub async fn revoke_consent(
    consent_guard: State<'_, ConsentGuard>,
    user_id: String,
    consent_type: String,
    reason: String,
    ip_address: Option<String>,
    user_agent: Option<String>,
) -> Result<JsonValue, String> {
    let consent_type_enum = parse_consent_type(&consent_type)?;

    consent_guard
        .revoke_consent_with_audit(
            &user_id,
            &consent_type_enum,
            &reason,
            ip_address.as_deref(),
            user_agent.as_deref(),
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "success": true,
        "user_id": user_id,
        "consent_type": consent_type,
        "reason": reason,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "gdpr_article": "Article 7(3) - Right to withdraw consent"
    }))
}

/// Check multiple consent statuses at once
#[tauri::command]
pub async fn check_multiple_consents(
    consent_guard: State<'_, ConsentGuard>,
    user_id: String,
    consent_types: Vec<String>,
) -> Result<JsonValue, String> {
    let consent_enums: Result<Vec<ConsentType>, String> = consent_types
        .iter()
        .map(|ct| parse_consent_type(ct))
        .collect();

    let consent_enums = consent_enums?;

    let results = consent_guard
        .check_multiple_consents(&user_id, &consent_enums)
        .await
        .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "user_id": user_id,
        "results": results,
        "total_checked": results.len(),
        "all_granted": results.iter().all(|r| r.allowed),
    }))
}

/// Get consent history for a user
#[tauri::command]
pub async fn get_consent_history(
    consent_guard: State<'_, ConsentGuard>,
    user_id: String,
    limit: Option<usize>,
) -> Result<JsonValue, String> {
    let manager = consent_guard.consent_manager();
    let manager_lock = manager.read().await;

    let logs = manager_lock
        .get_granular_consent_log(&user_id, limit.unwrap_or(50))
        .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "user_id": user_id,
        "history": logs,
        "total": logs.len()
    }))
}

/// Check if user needs to re-consent for any operations
#[tauri::command]
pub async fn check_reconsent_needed(
    consent_guard: State<'_, ConsentGuard>,
    user_id: String,
) -> Result<JsonValue, String> {
    let needs_reconsent = consent_guard
        .check_all_reconsents(&user_id)
        .await
        .map_err(|e| e.to_string())?;

    let consent_types: Vec<String> = needs_reconsent
        .iter()
        .map(|ct| ct.as_str().to_string())
        .collect();

    Ok(serde_json::json!({
        "user_id": user_id,
        "needs_reconsent": !needs_reconsent.is_empty(),
        "consent_types": consent_types,
        "count": consent_types.len(),
        "reason": if !consent_types.is_empty() {
            "Consent version updated, re-consent required"
        } else {
            "All consents up to date"
        }
    }))
}

/// Batch grant all required consents
#[tauri::command]
pub async fn grant_all_consents(
    consent_guard: State<'_, ConsentGuard>,
    user_id: String,
    ip_address: Option<String>,
    user_agent: Option<String>,
) -> Result<JsonValue, String> {
    let all_types = vec![
        ConsentType::PiiDetection,
        ConsentType::ChatStorage,
        ConsentType::DocumentProcessing,
        ConsentType::Analytics,
        ConsentType::AiProcessing,
        ConsentType::DataRetention,
    ];

    let mut granted = Vec::new();
    let mut failed = Vec::new();

    for consent_type in all_types {
        match consent_guard
            .grant_consent_with_audit(
                &user_id,
                &consent_type,
                ip_address.as_deref(),
                user_agent.as_deref(),
            )
            .await
        {
            Ok(consent_id) => granted.push(serde_json::json!({
                "consent_type": consent_type.as_str(),
                "consent_id": consent_id,
            })),
            Err(e) => failed.push(serde_json::json!({
                "consent_type": consent_type.as_str(),
                "error": e.to_string(),
            })),
        }
    }

    Ok(serde_json::json!({
        "success": failed.is_empty(),
        "user_id": user_id,
        "granted": granted,
        "failed": failed,
        "total_granted": granted.len(),
        "total_failed": failed.len(),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

/// Batch revoke all consents (user withdrawal)
#[tauri::command]
pub async fn revoke_all_consents(
    consent_guard: State<'_, ConsentGuard>,
    user_id: String,
    reason: String,
    _ip_address: Option<String>,
    _user_agent: Option<String>,
) -> Result<JsonValue, String> {
    let manager = consent_guard.consent_manager();
    let manager_lock = manager.write().await;

    let count = manager_lock
        .withdraw_all_consents(&user_id)
        .map_err(|e| e.to_string())?;

    drop(manager_lock);

    Ok(serde_json::json!({
        "success": true,
        "user_id": user_id,
        "consents_revoked": count,
        "reason": reason,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "gdpr_article": "Article 7(3) - Right to withdraw consent"
    }))
}

/// Helper function to parse consent type string to enum
fn parse_consent_type(consent_type: &str) -> Result<ConsentType, String> {
    match consent_type {
        "pii_detection" => Ok(ConsentType::PiiDetection),
        "chat_storage" => Ok(ConsentType::ChatStorage),
        "document_processing" => Ok(ConsentType::DocumentProcessing),
        "analytics" => Ok(ConsentType::Analytics),
        "ai_processing" => Ok(ConsentType::AiProcessing),
        "data_retention" => Ok(ConsentType::DataRetention),
        _ => Err(format!("Unknown consent type: {}", consent_type)),
    }
}
