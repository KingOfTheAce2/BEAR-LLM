#![allow(dead_code)]
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, Duration};
use tracing::{debug, error, info};

pub mod retention_tasks;

use retention_tasks::RetentionCleanupTask;

/// Schedule configuration for cleanup tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleConfig {
    /// Cleanup interval in hours
    pub interval_hours: u64,
    /// Enable automatic cleanup
    pub enabled: bool,
    /// Next scheduled run time
    pub next_run: Option<DateTime<Utc>>,
}

impl Default for ScheduleConfig {
    fn default() -> Self {
        Self {
            interval_hours: 24, // Daily by default
            enabled: true,
            next_run: None,
        }
    }
}

/// Scheduler command messages
#[derive(Debug, Clone)]
pub enum SchedulerCommand {
    /// Trigger manual cleanup
    RunCleanup,
    /// Update schedule configuration
    UpdateConfig(ScheduleConfig),
    /// Get current status
    GetStatus,
    /// Shutdown scheduler
    Shutdown,
}

/// Scheduler status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerStatus {
    pub is_running: bool,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
    pub total_cleanups: u64,
    pub last_cleanup_result: Option<CleanupResult>,
}

/// Result of a cleanup operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupResult {
    pub timestamp: DateTime<Utc>,
    pub documents_deleted: usize,
    pub chat_sessions_deleted: usize,
    pub chat_messages_deleted: usize,
    pub query_history_deleted: usize,
    pub errors: Vec<String>,
    pub success: bool,
}

/// Background scheduler for automated data retention cleanup
pub struct RetentionScheduler {
    db_path: PathBuf,
    config: Arc<RwLock<ScheduleConfig>>,
    status: Arc<RwLock<SchedulerStatus>>,
    command_tx: mpsc::UnboundedSender<SchedulerCommand>,
    command_rx: Option<mpsc::UnboundedReceiver<SchedulerCommand>>,
}

impl RetentionScheduler {
    /// Create a new retention scheduler
    pub fn new(db_path: PathBuf) -> Self {
        let (command_tx, command_rx) = mpsc::unbounded_channel();

        Self {
            db_path,
            config: Arc::new(RwLock::new(ScheduleConfig::default())),
            status: Arc::new(RwLock::new(SchedulerStatus {
                is_running: false,
                last_run: None,
                next_run: None,
                total_cleanups: 0,
                last_cleanup_result: None,
            })),
            command_tx,
            command_rx: Some(command_rx),
        }
    }

    /// Get a handle to send commands to the scheduler
    pub fn get_handle(&self) -> SchedulerHandle {
        SchedulerHandle {
            command_tx: self.command_tx.clone(),
            status: Arc::clone(&self.status),
        }
    }

    /// Start the scheduler background task
    pub async fn start(mut self) -> Result<()> {
        info!("Starting retention cleanup scheduler");

        let mut command_rx = self
            .command_rx
            .take()
            .context("Scheduler already started")?;

        // Update initial status
        {
            let mut status = self.status.write().await;
            status.is_running = true;

            let config = self.config.read().await;
            if config.enabled {
                status.next_run = Some(Self::calculate_next_run(config.interval_hours));
            }
        }

        // Clone Arc references for the task
        let db_path = self.db_path.clone();
        let config = Arc::clone(&self.config);
        let status = Arc::clone(&self.status);

        // Spawn background task
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(60)); // Check every minute

            loop {
                tokio::select! {
                    // Handle periodic checks
                    _ = ticker.tick() => {
                        let cfg = config.read().await;
                        if !cfg.enabled {
                            continue;
                        }

                        let should_run = {
                            let stat = status.read().await;
                            if let Some(next_run) = stat.next_run {
                                Utc::now() >= next_run
                            } else {
                                true // First run
                            }
                        };

                        if should_run {
                            debug!("Scheduled cleanup triggered");
                            Self::execute_cleanup(&db_path, &status).await;

                            // Update next run time
                            let mut stat = status.write().await;
                            stat.next_run = Some(Self::calculate_next_run(cfg.interval_hours));
                        }
                    }

                    // Handle commands
                    Some(cmd) = command_rx.recv() => {
                        match cmd {
                            SchedulerCommand::RunCleanup => {
                                info!("Manual cleanup triggered");
                                Self::execute_cleanup(&db_path, &status).await;
                            }
                            SchedulerCommand::UpdateConfig(new_config) => {
                                info!("Updating scheduler configuration");
                                let mut cfg = config.write().await;
                                *cfg = new_config;

                                // Update next run time
                                if cfg.enabled {
                                    let mut stat = status.write().await;
                                    stat.next_run = Some(Self::calculate_next_run(cfg.interval_hours));
                                }
                            }
                            SchedulerCommand::GetStatus => {
                                // Status is retrieved via handle, nothing to do
                                debug!("Status requested");
                            }
                            SchedulerCommand::Shutdown => {
                                info!("Scheduler shutdown requested");
                                let mut stat = status.write().await;
                                stat.is_running = false;
                                break;
                            }
                        }
                    }
                }
            }

            info!("Retention scheduler stopped");
        });

        Ok(())
    }

    /// Execute cleanup task
    async fn execute_cleanup(db_path: &Path, status: &Arc<RwLock<SchedulerStatus>>) {
        let start_time = Utc::now();

        let task = RetentionCleanupTask::new(db_path.to_path_buf());
        let result = match task.execute().await {
            Ok(counts) => {
                info!(
                    "Cleanup completed: documents={}, sessions={}, messages={}, queries={}",
                    counts.0, counts.1, counts.2, counts.3
                );
                CleanupResult {
                    timestamp: start_time,
                    documents_deleted: counts.0,
                    chat_sessions_deleted: counts.1,
                    chat_messages_deleted: counts.2,
                    query_history_deleted: counts.3,
                    errors: Vec::new(),
                    success: true,
                }
            }
            Err(e) => {
                error!("Cleanup failed: {}", e);
                CleanupResult {
                    timestamp: start_time,
                    documents_deleted: 0,
                    chat_sessions_deleted: 0,
                    chat_messages_deleted: 0,
                    query_history_deleted: 0,
                    errors: vec![e.to_string()],
                    success: false,
                }
            }
        };

        // Update status
        let mut stat = status.write().await;
        stat.last_run = Some(start_time);
        stat.total_cleanups += 1;
        stat.last_cleanup_result = Some(result);
    }

    /// Calculate next run time based on interval
    fn calculate_next_run(interval_hours: u64) -> DateTime<Utc> {
        Utc::now() + chrono::Duration::hours(interval_hours as i64)
    }
}

/// Handle to interact with the scheduler
#[derive(Clone)]
pub struct SchedulerHandle {
    command_tx: mpsc::UnboundedSender<SchedulerCommand>,
    status: Arc<RwLock<SchedulerStatus>>,
}

impl SchedulerHandle {
    /// Trigger a manual cleanup
    pub fn trigger_cleanup(&self) -> Result<()> {
        self.command_tx
            .send(SchedulerCommand::RunCleanup)
            .context("Failed to send cleanup command")
    }

    /// Update scheduler configuration
    pub fn update_config(&self, config: ScheduleConfig) -> Result<()> {
        self.command_tx
            .send(SchedulerCommand::UpdateConfig(config))
            .context("Failed to send config update")
    }

    /// Get current scheduler status
    pub async fn get_status(&self) -> SchedulerStatus {
        self.status.read().await.clone()
    }

    /// Shutdown the scheduler
    pub fn shutdown(&self) -> Result<()> {
        self.command_tx
            .send(SchedulerCommand::Shutdown)
            .context("Failed to send shutdown command")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduler_creation() {
        let db_path = PathBuf::from("test.db");
        let scheduler = RetentionScheduler::new(db_path);
        let handle = scheduler.get_handle();

        let status = handle.get_status().await;
        assert!(!status.is_running);
        assert_eq!(status.total_cleanups, 0);
    }

    #[tokio::test]
    async fn test_handle_operations() {
        let db_path = PathBuf::from("test.db");
        let scheduler = RetentionScheduler::new(db_path);
        let handle = scheduler.get_handle();

        // Test config update
        let config = ScheduleConfig {
            interval_hours: 48,
            enabled: true,
            next_run: None,
        };
        assert!(handle.update_config(config).is_ok());

        // Test shutdown
        assert!(handle.shutdown().is_ok());
    }

    #[test]
    fn test_next_run_calculation() {
        let next_run = RetentionScheduler::calculate_next_run(24);
        let expected = Utc::now() + chrono::Duration::hours(24);

        // Allow 1 second tolerance for test execution time
        let diff = (next_run - expected).num_seconds().abs();
        assert!(diff < 2);
    }
}