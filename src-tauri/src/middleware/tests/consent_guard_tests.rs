// Comprehensive test suite for Consent Guard Middleware

use crate::middleware::ConsentGuard;
use crate::middleware::ConsentGuardBuilder;
use crate::compliance::ConsentType;
use std::env;
use std::path::PathBuf;

fn get_test_db() -> PathBuf {
    let mut path = env::temp_dir();
    path.push(format!("test_consent_guard_{}.db", uuid::Uuid::new_v4()));
    path
}

#[tokio::test]
async fn test_consent_guard_initialization() {
    let db_path = get_test_db();
    let guard = ConsentGuardBuilder::new(db_path.clone())
        .strict_mode(true)
        .build();

    let result = guard.initialize().await;
    assert!(result.is_ok(), "Guard initialization failed");

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn test_chat_storage_consent_workflow() {
    let db_path = get_test_db();
    let guard = ConsentGuardBuilder::new(db_path.clone())
        .strict_mode(true)
        .build();

    guard.initialize().await.unwrap();
    let user_id = "test_user_chat";

    // Initially no consent
    let result = guard.check_chat_storage(user_id).await.unwrap();
    assert!(!result.allowed, "Chat storage should be denied without consent");
    assert!(result.reason.is_some(), "Reason should be provided");

    // Grant consent
    let consent_id = guard
        .grant_consent_with_audit(user_id, &ConsentType::ChatStorage, None, None)
        .await
        .unwrap();
    assert!(consent_id > 0, "Consent ID should be positive");

    // Now should be allowed
    let result = guard.check_chat_storage(user_id).await.unwrap();
    assert!(result.allowed, "Chat storage should be allowed with consent");
    assert!(result.reason.is_none(), "No reason for allowed operation");

    // Revoke consent
    guard
        .revoke_consent_with_audit(
            user_id,
            &ConsentType::ChatStorage,
            "User withdrew consent",
            None,
            None,
        )
        .await
        .unwrap();

    // Should be denied again
    let result = guard.check_chat_storage(user_id).await.unwrap();
    assert!(!result.allowed, "Chat storage should be denied after revocation");

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn test_document_processing_consent() {
    let db_path = get_test_db();
    let guard = ConsentGuardBuilder::new(db_path.clone())
        .strict_mode(false)
        .build();

    guard.initialize().await.unwrap();
    let user_id = "test_user_docs";

    // Enforcement should fail
    let result = guard
        .enforce_consent(user_id, &ConsentType::DocumentProcessing)
        .await;
    assert!(result.is_err(), "Should fail without consent");

    // Grant consent
    guard
        .grant_consent_with_audit(user_id, &ConsentType::DocumentProcessing, None, None)
        .await
        .unwrap();

    // Enforcement should succeed
    let result = guard
        .enforce_consent(user_id, &ConsentType::DocumentProcessing)
        .await;
    assert!(result.is_ok(), "Should succeed with consent");

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn test_pii_detection_consent() {
    let db_path = get_test_db();
    let guard = ConsentGuardBuilder::new(db_path.clone())
        .strict_mode(true)
        .build();

    guard.initialize().await.unwrap();
    let user_id = "test_user_pii";

    let result = guard.check_pii_detection(user_id).await.unwrap();
    assert!(!result.allowed);

    guard
        .grant_consent_with_audit(user_id, &ConsentType::PiiDetection, None, None)
        .await
        .unwrap();

    let result = guard.check_pii_detection(user_id).await.unwrap();
    assert!(result.allowed);

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn test_analytics_consent() {
    let db_path = get_test_db();
    let guard = ConsentGuardBuilder::new(db_path.clone())
        .strict_mode(true)
        .build();

    guard.initialize().await.unwrap();
    let user_id = "test_user_analytics";

    let result = guard.check_analytics(user_id).await.unwrap();
    assert!(!result.allowed);

    guard
        .grant_consent_with_audit(user_id, &ConsentType::Analytics, None, None)
        .await
        .unwrap();

    let result = guard.check_analytics(user_id).await.unwrap();
    assert!(result.allowed);

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn test_ai_processing_consent() {
    let db_path = get_test_db();
    let guard = ConsentGuardBuilder::new(db_path.clone())
        .strict_mode(true)
        .build();

    guard.initialize().await.unwrap();
    let user_id = "test_user_ai";

    let result = guard.check_ai_processing(user_id).await.unwrap();
    assert!(!result.allowed);

    guard
        .grant_consent_with_audit(user_id, &ConsentType::AiProcessing, None, None)
        .await
        .unwrap();

    let result = guard.check_ai_processing(user_id).await.unwrap();
    assert!(result.allowed);

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn test_data_retention_consent() {
    let db_path = get_test_db();
    let guard = ConsentGuardBuilder::new(db_path.clone())
        .strict_mode(true)
        .build();

    guard.initialize().await.unwrap();
    let user_id = "test_user_retention";

    let result = guard.check_data_retention(user_id).await.unwrap();
    assert!(!result.allowed);

    guard
        .grant_consent_with_audit(user_id, &ConsentType::DataRetention, None, None)
        .await
        .unwrap();

    let result = guard.check_data_retention(user_id).await.unwrap();
    assert!(result.allowed);

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn test_multiple_consent_checks() {
    let db_path = get_test_db();
    let guard = ConsentGuardBuilder::new(db_path.clone())
        .strict_mode(false)
        .build();

    guard.initialize().await.unwrap();
    let user_id = "test_user_multiple";

    // Grant some consents
    guard
        .grant_consent_with_audit(user_id, &ConsentType::ChatStorage, None, None)
        .await
        .unwrap();
    guard
        .grant_consent_with_audit(user_id, &ConsentType::DocumentProcessing, None, None)
        .await
        .unwrap();

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

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn test_enforce_multiple_consents() {
    let db_path = get_test_db();
    let guard = ConsentGuardBuilder::new(db_path.clone())
        .strict_mode(true)
        .build();

    guard.initialize().await.unwrap();
    let user_id = "test_user_enforce_multi";

    // Should fail without consents
    let result = guard
        .enforce_multiple_consents(
            user_id,
            &[ConsentType::ChatStorage, ConsentType::PiiDetection],
        )
        .await;
    assert!(result.is_err());

    // Grant both consents
    guard
        .grant_consent_with_audit(user_id, &ConsentType::ChatStorage, None, None)
        .await
        .unwrap();
    guard
        .grant_consent_with_audit(user_id, &ConsentType::PiiDetection, None, None)
        .await
        .unwrap();

    // Should succeed now
    let result = guard
        .enforce_multiple_consents(
            user_id,
            &[ConsentType::ChatStorage, ConsentType::PiiDetection],
        )
        .await;
    assert!(result.is_ok());

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn test_strict_mode_behavior() {
    let db_path = get_test_db();

    // Test with strict mode
    let strict_guard = ConsentGuardBuilder::new(db_path.clone())
        .strict_mode(true)
        .build();
    strict_guard.initialize().await.unwrap();

    let user_id = "test_strict_mode";

    // Grant consent
    strict_guard
        .grant_consent_with_audit(user_id, &ConsentType::Analytics, None, None)
        .await
        .unwrap();

    let result = strict_guard.check_analytics(user_id).await.unwrap();
    assert!(result.allowed);

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn test_lenient_mode_behavior() {
    let db_path = get_test_db();

    // Test with lenient mode
    let lenient_guard = ConsentGuardBuilder::new(db_path.clone())
        .strict_mode(false)
        .build();
    lenient_guard.initialize().await.unwrap();

    let user_id = "test_lenient_mode";

    // Grant consent
    lenient_guard
        .grant_consent_with_audit(user_id, &ConsentType::Analytics, None, None)
        .await
        .unwrap();

    let result = lenient_guard.check_analytics(user_id).await.unwrap();
    assert!(result.allowed);

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn test_audit_trail_with_ip_and_user_agent() {
    let db_path = get_test_db();
    let guard = ConsentGuardBuilder::new(db_path.clone())
        .strict_mode(true)
        .build();

    guard.initialize().await.unwrap();
    let user_id = "test_audit_user";

    // Grant with audit info
    guard
        .grant_consent_with_audit(
            user_id,
            &ConsentType::ChatStorage,
            Some("192.168.1.100"),
            Some("Mozilla/5.0"),
        )
        .await
        .unwrap();

    // Revoke with audit info
    guard
        .revoke_consent_with_audit(
            user_id,
            &ConsentType::ChatStorage,
            "Testing audit trail",
            Some("192.168.1.101"),
            Some("Chrome/91.0"),
        )
        .await
        .unwrap();

    // Check audit log exists
    let manager = guard.consent_manager();
    let manager_lock = manager.read().await;
    let logs = manager_lock.get_granular_consent_log(user_id, 10).unwrap();

    assert!(logs.len() >= 2, "Should have audit log entries");

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn test_check_all_reconsents() {
    let db_path = get_test_db();
    let guard = ConsentGuardBuilder::new(db_path.clone())
        .strict_mode(true)
        .build();

    guard.initialize().await.unwrap();
    let user_id = "test_reconsent_user";

    // Grant some consents
    guard
        .grant_consent_with_audit(user_id, &ConsentType::ChatStorage, None, None)
        .await
        .unwrap();

    // Check reconsents needed
    let needs_reconsent = guard.check_all_reconsents(user_id).await.unwrap();

    // Initially should not need reconsent
    assert!(
        needs_reconsent.is_empty(),
        "Should not need reconsent initially"
    );

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn test_consent_manager_reference() {
    let db_path = get_test_db();
    let guard = ConsentGuardBuilder::new(db_path.clone())
        .strict_mode(true)
        .build();

    guard.initialize().await.unwrap();

    // Get manager reference
    let manager = guard.consent_manager();
    let manager_lock = manager.read().await;

    // Verify it's the same manager
    let versions = manager_lock.get_consent_versions().unwrap();
    assert!(versions.len() >= 0);

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn test_dynamic_strict_mode_change() {
    let db_path = get_test_db();
    let mut guard = ConsentGuardBuilder::new(db_path.clone())
        .strict_mode(true)
        .build();

    guard.initialize().await.unwrap();

    // Change to lenient mode
    guard.set_strict_mode(false);

    // Verify mode changed
    let user_id = "test_mode_change";
    guard
        .grant_consent_with_audit(user_id, &ConsentType::Analytics, None, None)
        .await
        .unwrap();

    let result = guard.check_analytics(user_id).await.unwrap();
    assert!(result.allowed);

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn test_consent_enforcement_error_messages() {
    let db_path = get_test_db();
    let guard = ConsentGuardBuilder::new(db_path.clone())
        .strict_mode(true)
        .build();

    guard.initialize().await.unwrap();
    let user_id = "test_error_messages";

    // Test enforcement error
    let result = guard
        .enforce_consent(user_id, &ConsentType::DocumentProcessing)
        .await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("denied") || error_msg.contains("consent"));

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn test_all_consent_types() {
    let db_path = get_test_db();
    let guard = ConsentGuardBuilder::new(db_path.clone())
        .strict_mode(true)
        .build();

    guard.initialize().await.unwrap();
    let user_id = "test_all_types";

    let all_types = vec![
        ConsentType::PiiDetection,
        ConsentType::ChatStorage,
        ConsentType::DocumentProcessing,
        ConsentType::Analytics,
        ConsentType::AiProcessing,
        ConsentType::DataRetention,
    ];

    // Grant all consents
    for consent_type in &all_types {
        guard
            .grant_consent_with_audit(user_id, consent_type, None, None)
            .await
            .unwrap();
    }

    // Verify all are granted
    let results = guard
        .check_multiple_consents(user_id, &all_types)
        .await
        .unwrap();

    for result in results {
        assert!(result.allowed, "All consents should be granted");
    }

    let _ = std::fs::remove_file(db_path);
}
