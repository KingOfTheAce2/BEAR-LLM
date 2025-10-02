use crate::scheduler::retention_tasks::{CleanupPreview, RetentionCleanupTask};
use crate::scheduler::{CleanupResult, ScheduleConfig, SchedulerHandle, SchedulerStatus};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

/// Trigger manual retention cleanup
#[tauri::command]
pub async fn trigger_retention_cleanup(
    scheduler_handle: State<'_, Option<Arc<RwLock<SchedulerHandle>>>>,
) -> Result<String, String> {
    if let Some(handle_arc) = scheduler_handle.inner() {
        let handle = handle_arc.read().await;
        handle
            .trigger_cleanup()
            .map(|_| "Cleanup triggered successfully".to_string())
            .map_err(|e| format!("Failed to trigger cleanup: {}", e))
    } else {
        Err("Scheduler not initialized".to_string())
    }
}

/// Get scheduler status
#[tauri::command]
pub async fn get_scheduler_status(
    scheduler_handle: State<'_, Option<Arc<RwLock<SchedulerHandle>>>>,
) -> Result<SchedulerStatus, String> {
    if let Some(handle_arc) = scheduler_handle.inner() {
        let handle = handle_arc.read().await;
        Ok(handle.get_status().await)
    } else {
        Err("Scheduler not initialized".to_string())
    }
}

/// Update scheduler configuration
#[tauri::command]
pub async fn update_scheduler_config(
    scheduler_handle: State<'_, Option<Arc<RwLock<SchedulerHandle>>>>,
    config: ScheduleConfig,
) -> Result<String, String> {
    if let Some(handle_arc) = scheduler_handle.inner() {
        let handle = handle_arc.read().await;
        handle
            .update_config(config)
            .map(|_| "Configuration updated successfully".to_string())
            .map_err(|e| format!("Failed to update configuration: {}", e))
    } else {
        Err("Scheduler not initialized".to_string())
    }
}

/// Preview what would be deleted in next cleanup
#[tauri::command]
pub async fn preview_retention_cleanup(
    db_path: State<'_, PathBuf>,
) -> Result<CleanupPreview, String> {
    let task = RetentionCleanupTask::new(db_path.inner().clone());
    task.preview_cleanup()
        .await
        .map_err(|e| format!("Failed to preview cleanup: {}", e))
}

/// Get last cleanup result
#[tauri::command]
pub async fn get_last_cleanup_result(
    scheduler_handle: State<'_, Option<Arc<RwLock<SchedulerHandle>>>>,
) -> Result<Option<CleanupResult>, String> {
    if let Some(handle_arc) = scheduler_handle.inner() {
        let handle = handle_arc.read().await;
        let status = handle.get_status().await;
        Ok(status.last_cleanup_result)
    } else {
        Err("Scheduler not initialized".to_string())
    }
}

/// Enable or disable automatic cleanup
#[tauri::command]
pub async fn set_automatic_cleanup(
    scheduler_handle: State<'_, Option<Arc<RwLock<SchedulerHandle>>>>,
    enabled: bool,
    interval_hours: Option<u64>,
) -> Result<String, String> {
    if let Some(handle_arc) = scheduler_handle.inner() {
        let handle = handle_arc.read().await;
        let config = ScheduleConfig {
            interval_hours: interval_hours.unwrap_or(24),
            enabled,
            next_run: None,
        };

        handle
            .update_config(config)
            .map(|_| {
                if enabled {
                    format!(
                        "Automatic cleanup enabled (every {} hours)",
                        interval_hours.unwrap_or(24)
                    )
                } else {
                    "Automatic cleanup disabled".to_string()
                }
            })
            .map_err(|e| format!("Failed to set automatic cleanup: {}", e))
    } else {
        Err("Scheduler not initialized".to_string())
    }
}

// Removed: Duplicate command - use compliance::commands::apply_default_retention_policies instead
