use super::super::*;
use std::env;
use std::path::PathBuf;
use tokio::time::{sleep, Duration};

fn get_test_db() -> PathBuf {
    let mut path = env::temp_dir();
    path.push(format!("test_scheduler_{}.db", uuid::Uuid::new_v4()));
    path
}

#[tokio::test]
async fn test_scheduler_lifecycle() {
    let db_path = get_test_db();
    let scheduler = RetentionScheduler::new(db_path.clone());
    let handle = scheduler.get_handle();

    // Start scheduler
    let start_result = scheduler.start().await;
    assert!(start_result.is_ok());

    // Give it time to start
    sleep(Duration::from_millis(100)).await;

    // Check status
    let status = handle.get_status().await;
    assert!(status.is_running);

    // Shutdown
    assert!(handle.shutdown().is_ok());

    // Give it time to shutdown
    sleep(Duration::from_millis(100)).await;

    // Cleanup
    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn test_manual_cleanup_trigger() {
    let db_path = get_test_db();

    // Initialize test database
    {
        use rusqlite::Connection;
        let conn = Connection::open(&db_path).unwrap();
        conn.execute(
            "CREATE TABLE documents (
                id INTEGER PRIMARY KEY,
                filename TEXT,
                retention_until DATETIME
            )",
            [],
        ).unwrap();
        conn.execute(
            "CREATE TABLE chat_sessions (
                id INTEGER PRIMARY KEY,
                retention_until DATETIME
            )",
            [],
        ).unwrap();
        conn.execute(
            "CREATE TABLE chat_messages (
                id INTEGER PRIMARY KEY,
                retention_until DATETIME
            )",
            [],
        ).unwrap();
        conn.execute(
            "CREATE TABLE query_history (
                id INTEGER PRIMARY KEY,
                retention_until DATETIME
            )",
            [],
        ).unwrap();
    }

    let scheduler = RetentionScheduler::new(db_path.clone());
    let handle = scheduler.get_handle();

    // Start scheduler
    let _ = scheduler.start().await;
    sleep(Duration::from_millis(100)).await;

    // Get initial status
    let initial_status = handle.get_status().await;
    let initial_cleanups = initial_status.total_cleanups;

    // Trigger manual cleanup
    assert!(handle.trigger_cleanup().is_ok());

    // Wait for cleanup to complete
    sleep(Duration::from_millis(500)).await;

    // Check that cleanup was executed
    let final_status = handle.get_status().await;
    assert!(final_status.total_cleanups > initial_cleanups);
    assert!(final_status.last_run.is_some());

    // Shutdown
    let _ = handle.shutdown();
    sleep(Duration::from_millis(100)).await;

    // Cleanup
    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn test_config_update() {
    let db_path = get_test_db();
    let scheduler = RetentionScheduler::new(db_path.clone());
    let handle = scheduler.get_handle();

    // Start scheduler
    let _ = scheduler.start().await;
    sleep(Duration::from_millis(100)).await;

    // Update configuration
    let new_config = ScheduleConfig {
        interval_hours: 48,
        enabled: false,
        next_run: None,
    };

    assert!(handle.update_config(new_config).is_ok());

    // Wait for config to be applied
    sleep(Duration::from_millis(100)).await;

    // Note: We can't directly verify the config change from here,
    // but the command should be processed without error

    // Shutdown
    let _ = handle.shutdown();
    sleep(Duration::from_millis(100)).await;

    // Cleanup
    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn test_schedule_calculation() {
    let now = Utc::now();
    let next_run_24h = RetentionScheduler::calculate_next_run(24);
    let next_run_48h = RetentionScheduler::calculate_next_run(48);

    // Verify 24 hour schedule
    let diff_24h = (next_run_24h - now).num_hours();
    assert!(diff_24h >= 23 && diff_24h <= 24);

    // Verify 48 hour schedule
    let diff_48h = (next_run_48h - now).num_hours();
    assert!(diff_48h >= 47 && diff_48h <= 48);

    // Verify ordering
    assert!(next_run_48h > next_run_24h);
}

#[tokio::test]
async fn test_concurrent_commands() {
    let db_path = get_test_db();

    // Initialize database
    {
        use rusqlite::Connection;
        let conn = Connection::open(&db_path).unwrap();
        conn.execute(
            "CREATE TABLE documents (
                id INTEGER PRIMARY KEY,
                retention_until DATETIME
            )",
            [],
        ).unwrap();
        conn.execute(
            "CREATE TABLE chat_sessions (
                id INTEGER PRIMARY KEY,
                retention_until DATETIME
            )",
            [],
        ).unwrap();
        conn.execute(
            "CREATE TABLE chat_messages (
                id INTEGER PRIMARY KEY,
                retention_until DATETIME
            )",
            [],
        ).unwrap();
        conn.execute(
            "CREATE TABLE query_history (
                id INTEGER PRIMARY KEY,
                retention_until DATETIME
            )",
            [],
        ).unwrap();
    }

    let scheduler = RetentionScheduler::new(db_path.clone());
    let handle = scheduler.get_handle();

    // Start scheduler
    let _ = scheduler.start().await;
    sleep(Duration::from_millis(100)).await;

    // Send multiple commands concurrently
    let handle1 = handle.clone();
    let handle2 = handle.clone();
    let handle3 = handle.clone();

    let task1 = tokio::spawn(async move {
        handle1.trigger_cleanup()
    });

    let task2 = tokio::spawn(async move {
        handle2.get_status().await
    });

    let task3 = tokio::spawn(async move {
        let config = ScheduleConfig {
            interval_hours: 12,
            enabled: true,
            next_run: None,
        };
        handle3.update_config(config)
    });

    // All should complete without error
    let result1 = task1.await.unwrap();
    let status = task2.await.unwrap();
    let result3 = task3.await.unwrap();

    assert!(result1.is_ok());
    assert!(status.is_running);
    assert!(result3.is_ok());

    // Shutdown
    let _ = handle.shutdown();
    sleep(Duration::from_millis(100)).await;

    // Cleanup
    let _ = std::fs::remove_file(db_path);
}

#[test]
fn test_schedule_config_serialization() {
    let config = ScheduleConfig {
        interval_hours: 24,
        enabled: true,
        next_run: Some(Utc::now()),
    };

    // Test serialization
    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("interval_hours"));
    assert!(json.contains("enabled"));

    // Test deserialization
    let deserialized: ScheduleConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.interval_hours, 24);
    assert!(deserialized.enabled);
}

#[test]
fn test_cleanup_result_serialization() {
    let result = CleanupResult {
        timestamp: Utc::now(),
        documents_deleted: 10,
        chat_sessions_deleted: 5,
        chat_messages_deleted: 20,
        query_history_deleted: 15,
        errors: vec!["Test error".to_string()],
        success: true,
    };

    // Test serialization
    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("documents_deleted"));
    assert!(json.contains("success"));

    // Test deserialization
    let deserialized: CleanupResult = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.documents_deleted, 10);
    assert_eq!(deserialized.errors.len(), 1);
    assert!(deserialized.success);
}
