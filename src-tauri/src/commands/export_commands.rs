// Tauri Commands for GDPR Data Export
// Provides frontend interface to trigger data exports
//
// Single-user desktop app: No user_id parameters needed

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;
use dirs;

use crate::export_engine::ExportEngine;
use crate::database::export_integration::ExportIntegration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequest {
    pub formats: Vec<String>, // ["docx", "pdf", "markdown", "json"]
    pub include_compliance_data: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResponse {
    pub success: bool,
    pub message: String,
    pub exported_files: Vec<String>,
    pub export_directory: String,
    pub metadata: Option<serde_json::Value>,
}

/// Export user data to specified formats
/// Single-user app: No user_id parameter needed
#[tauri::command]
pub async fn export_user_data(request: ExportRequest) -> Result<ExportResponse, String> {
    // Get database path
    let mut db_path = dirs::data_local_dir()
        .ok_or_else(|| "Could not find local data directory".to_string())?;
    db_path.push("bear-ai");
    db_path.push("bear_ai.db");

    // Create export integration
    let export_integration = ExportIntegration::new(db_path);

    // Fetch user data from database (single-user, no user_id parameter)
    let user_data = export_integration
        .fetch_user_data()
        .map_err(|e| format!("Failed to fetch user data: {}", e))?;

    // Create export engine
    let export_engine = ExportEngine::new();

    // Determine output directory
    let mut output_dir = dirs::document_dir()
        .ok_or_else(|| "Could not find documents directory".to_string())?;
    output_dir.push("BEAR_AI_Exports");
    output_dir.push(format!("export_{}", chrono::Utc::now().format("%Y%m%d_%H%M%S")));

    // Convert format strings to slices
    let formats: Vec<&str> = request.formats.iter().map(|s| s.as_str()).collect();

    // Export to requested formats
    let exported_files = export_engine
        .export_user_data(&user_data, &output_dir, &formats)
        .map_err(|e| format!("Failed to export data: {}", e))?;

    // If compliance data requested, also export JSON with full details
    let mut all_files = exported_files.clone();
    let metadata = if request.include_compliance_data {
        let complete_data = export_integration
            .fetch_complete_user_data()
            .map_err(|e| format!("Failed to fetch compliance data: {}", e))?;

        // Write complete JSON export
        let json_path = output_dir.join("bear_ai_complete_export.json");
        std::fs::write(
            &json_path,
            serde_json::to_string_pretty(&complete_data)
                .map_err(|e| format!("Failed to serialize JSON: {}", e))?,
        )
        .map_err(|e| format!("Failed to write JSON file: {}", e))?;

        all_files.push(json_path.to_string_lossy().to_string());
        Some(complete_data)
    } else {
        None
    };

    Ok(ExportResponse {
        success: true,
        message: format!("Successfully exported data to {} file(s)", all_files.len()),
        exported_files: all_files,
        export_directory: output_dir.to_string_lossy().to_string(),
        metadata,
    })
}

/// Export user data to JSON only (lightweight)
/// Single-user app: No user_id parameter needed
#[tauri::command]
pub async fn export_user_data_json() -> Result<String, String> {
    let mut db_path = dirs::data_local_dir()
        .ok_or_else(|| "Could not find local data directory".to_string())?;
    db_path.push("bear-ai");
    db_path.push("bear_ai.db");

    let export_integration = ExportIntegration::new(db_path);
    let user_data = export_integration
        .fetch_user_data()
        .map_err(|e| format!("Failed to fetch user data: {}", e))?;

    serde_json::to_string_pretty(&user_data)
        .map_err(|e| format!("Failed to serialize data: {}", e))
}

/// Export consent data only
/// Single-user app: No user_id parameter needed
#[tauri::command]
pub async fn export_consent_data() -> Result<String, String> {
    let mut db_path = dirs::data_local_dir()
        .ok_or_else(|| "Could not find local data directory".to_string())?;
    db_path.push("bear-ai");
    db_path.push("bear_ai.db");

    let export_integration = ExportIntegration::new(db_path);
    let consent_data = export_integration
        .fetch_consent_data()
        .map_err(|e| format!("Failed to fetch consent data: {}", e))?;

    serde_json::to_string_pretty(&consent_data)
        .map_err(|e| format!("Failed to serialize consent data: {}", e))
}

/// Export audit logs only
/// Single-user app: No user_id parameter needed
#[tauri::command]
pub async fn export_audit_logs(limit: Option<usize>) -> Result<String, String> {
    let mut db_path = dirs::data_local_dir()
        .ok_or_else(|| "Could not find local data directory".to_string())?;
    db_path.push("bear-ai");
    db_path.push("bear_ai.db");

    let export_integration = ExportIntegration::new(db_path);
    let audit_logs = export_integration
        .fetch_audit_logs(limit.unwrap_or(1000))
        .map_err(|e| format!("Failed to fetch audit logs: {}", e))?;

    serde_json::to_string_pretty(&audit_logs)
        .map_err(|e| format!("Failed to serialize audit logs: {}", e))
}

/// Get export preview (metadata only, no actual export)
/// Single-user app: No user_id parameter needed
#[tauri::command]
pub async fn get_export_preview() -> Result<serde_json::Value, String> {
    let mut db_path = dirs::data_local_dir()
        .ok_or_else(|| "Could not find local data directory".to_string())?;
    db_path.push("bear-ai");
    db_path.push("bear_ai.db");

    let export_integration = ExportIntegration::new(db_path);
    let user_data = export_integration
        .fetch_user_data()
        .map_err(|e| format!("Failed to fetch user data: {}", e))?;

    // Return summary without full content
    Ok(serde_json::json!({
        "user_id": user_data.user_id,
        "export_date": user_data.export_date.to_rfc3339(),
        "version": user_data.version,
        "statistics": {
            "total_chats": user_data.chats.len(),
            "total_messages": user_data.chats.iter().map(|c| c.messages.len()).sum::<usize>(),
            "total_documents": user_data.documents.len(),
            "total_pii_detections": user_data.documents.iter().map(|d| d.pii_detections.len()).sum::<usize>()
        },
        "compliance": {
            "gdpr_article_20_compliant": user_data.metadata.compliance_info.gdpr_article_20,
            "data_integrity_verified": user_data.metadata.compliance_info.integrity_verified,
            "export_hash": user_data.metadata.export_hash
        },
        "available_formats": ["docx", "pdf", "markdown", "txt", "json"]
    }))
}

/// Verify export integrity by comparing hash
/// Single-user app: No user_id parameter needed
#[tauri::command]
pub async fn verify_export_integrity(export_hash: String) -> Result<bool, String> {
    let mut db_path = dirs::data_local_dir()
        .ok_or_else(|| "Could not find local data directory".to_string())?;
    db_path.push("bear-ai");
    db_path.push("bear_ai.db");

    let export_integration = ExportIntegration::new(db_path);
    let user_data = export_integration
        .fetch_user_data()
        .map_err(|e| format!("Failed to fetch user data: {}", e))?;

    Ok(user_data.metadata.export_hash == export_hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_export_request_serialization() {
        let request = ExportRequest {
            formats: vec!["docx".to_string(), "pdf".to_string()],
            include_compliance_data: true,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("docx"));
    }

    #[tokio::test]
    async fn test_export_response_serialization() {
        let response = ExportResponse {
            success: true,
            message: "Test message".to_string(),
            exported_files: vec!["file1.docx".to_string()],
            export_directory: "/tmp/exports".to_string(),
            metadata: Some(serde_json::json!({"test": "data"})),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("Test message"));
    }
}
