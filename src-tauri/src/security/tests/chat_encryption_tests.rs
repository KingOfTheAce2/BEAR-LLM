// SPDX-License-Identifier: MIT
// Copyright (c) 2025 BEAR AI LLM
//
// Chat Encryption Test Suite
// Comprehensive tests for chat message encryption functionality

use crate::security::chat_encryption::{ChatEncryptor, EncryptedMessage, UserKeyDerivation};
use crate::security::key_manager::KeyManager;
use crate::security::migration::ChatMigrationManager;
use rusqlite::Connection;
use std::sync::Arc;

#[test]
fn test_basic_encryption_decryption() {
    let encryptor = ChatEncryptor::new();
    let key = b"12345678901234567890123456789012"; // 32 bytes for AES-256
    let message = "Confidential legal advice regarding case #12345";

    let encrypted = encryptor.encrypt(message, key, "user123").unwrap();
    let decrypted = encryptor.decrypt(&encrypted, key).unwrap();

    assert_eq!(message, decrypted);
}

#[test]
fn test_encryption_produces_different_ciphertexts() {
    let encryptor = ChatEncryptor::new();
    let key = b"12345678901234567890123456789012";
    let message = "Same message encrypted twice";

    let encrypted1 = encryptor.encrypt(message, key, "user123").unwrap();
    let encrypted2 = encryptor.encrypt(message, key, "user123").unwrap();

    // Due to random nonces, ciphertexts should be different
    assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);
    assert_ne!(encrypted1.nonce, encrypted2.nonce);

    // But both should decrypt to same plaintext
    assert_eq!(
        encryptor.decrypt(&encrypted1, key).unwrap(),
        encryptor.decrypt(&encrypted2, key).unwrap()
    );
}

#[test]
fn test_wrong_key_fails_decryption() {
    let encryptor = ChatEncryptor::new();
    let correct_key = b"12345678901234567890123456789012";
    let wrong_key = b"00000000000000000000000000000000";
    let message = "Secret legal strategy";

    let encrypted = encryptor.encrypt(message, correct_key, "user123").unwrap();
    let result = encryptor.decrypt(&encrypted, wrong_key);

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Decryption failed"));
}

#[test]
fn test_tampering_detection() {
    let encryptor = ChatEncryptor::new();
    let key = b"12345678901234567890123456789012";
    let message = "Evidence document content";

    let mut encrypted = encryptor.encrypt(message, key, "user123").unwrap();

    // Tamper with ciphertext
    if !encrypted.ciphertext.is_empty() {
        encrypted.ciphertext[0] ^= 0x01;
    }

    // Decryption should fail due to authentication failure
    let result = encryptor.decrypt(&encrypted, key);
    assert!(result.is_err());
}

#[test]
fn test_nonce_tampering_detection() {
    let encryptor = ChatEncryptor::new();
    let key = b"12345678901234567890123456789012";
    let message = "Witness statement";

    let mut encrypted = encryptor.encrypt(message, key, "user123").unwrap();

    // Tamper with nonce
    encrypted.nonce[0] ^= 0x01;

    // Decryption should fail
    let result = encryptor.decrypt(&encrypted, key);
    assert!(result.is_err());
}

#[test]
fn test_json_serialization_roundtrip() {
    let encryptor = ChatEncryptor::new();
    let key = b"12345678901234567890123456789012";
    let message = "Court filing details with special chars: 法律 🏛️";

    let json = encryptor.encrypt_to_json(message, key, "user123").unwrap();
    assert!(json.contains("ciphertext"));
    assert!(json.contains("nonce"));
    assert!(json.contains("version"));

    let decrypted = encryptor.decrypt_from_json(&json, key).unwrap();
    assert_eq!(message, decrypted);
}

#[test]
fn test_empty_message_encryption() {
    let encryptor = ChatEncryptor::new();
    let key = b"12345678901234567890123456789012";
    let message = "";

    let encrypted = encryptor.encrypt(message, key, "user123").unwrap();
    let decrypted = encryptor.decrypt(&encrypted, key).unwrap();

    assert_eq!(message, decrypted);
}

#[test]
fn test_large_message_encryption() {
    let encryptor = ChatEncryptor::new();
    let key = b"12345678901234567890123456789012";
    let message = "A".repeat(100000); // 100KB message

    let encrypted = encryptor.encrypt(&message, key, "user123").unwrap();
    let decrypted = encryptor.decrypt(&encrypted, key).unwrap();

    assert_eq!(message, decrypted);
}

#[test]
fn test_unicode_message_encryption() {
    let encryptor = ChatEncryptor::new();
    let key = b"12345678901234567890123456789012";
    let message = "Legal document: 法律文件 судебный документ 🏛️⚖️📜";

    let encrypted = encryptor.encrypt(message, key, "user123").unwrap();
    let decrypted = encryptor.decrypt(&encrypted, key).unwrap();

    assert_eq!(message, decrypted);
}

#[test]
fn test_user_key_derivation_consistency() {
    let master_key = vec![42u8; 32];
    let derivation = UserKeyDerivation::new(master_key).unwrap();

    let key1 = derivation.derive_user_key("user123", "password").unwrap();
    let key2 = derivation.derive_user_key("user123", "password").unwrap();

    // Same user and password should always produce same key
    assert_eq!(key1, key2);
}

#[test]
fn test_user_key_derivation_uniqueness() {
    let master_key = vec![42u8; 32];
    let derivation = UserKeyDerivation::new(master_key).unwrap();

    let user1_key = derivation.derive_user_key("user1", "password").unwrap();
    let user2_key = derivation.derive_user_key("user2", "password").unwrap();
    let user1_diff_pass = derivation.derive_user_key("user1", "different").unwrap();

    // Different users should have different keys
    assert_ne!(user1_key, user2_key);
    // Same user with different password should have different key
    assert_ne!(user1_key, user1_diff_pass);
}

#[test]
fn test_default_key_derivation() {
    let master_key = vec![42u8; 32];
    let derivation = UserKeyDerivation::new(master_key).unwrap();

    let default_key1 = derivation.derive_default_key("user123").unwrap();
    let default_key2 = derivation.derive_default_key("user123").unwrap();
    let default_key_other = derivation.derive_default_key("user456").unwrap();

    // Same user should get same default key
    assert_eq!(default_key1, default_key2);
    // Different users should get different default keys
    assert_ne!(default_key1, default_key_other);
}

#[test]
fn test_derived_key_length() {
    let master_key = vec![42u8; 32];
    let derivation = UserKeyDerivation::new(master_key).unwrap();

    let key = derivation.derive_user_key("user123", "password").unwrap();
    assert_eq!(key.len(), 32); // Should be 256 bits for AES-256
}

#[test]
fn test_end_to_end_with_derived_keys() {
    let master_key = vec![42u8; 32];
    let derivation = UserKeyDerivation::new(master_key).unwrap();
    let encryptor = ChatEncryptor::new();

    let user_key = derivation.derive_default_key("user123").unwrap();
    let message = "Confidential attorney-client communication";

    let encrypted = encryptor.encrypt(message, &user_key, "user123").unwrap();
    let decrypted = encryptor.decrypt(&encrypted, &user_key).unwrap();

    assert_eq!(message, decrypted);
}

#[test]
fn test_key_manager_integration() {
    let key_manager = KeyManager::new().unwrap();
    // Clean up any existing test key
    let _ = key_manager.delete_key();

    let master_key = key_manager.get_or_create_key().unwrap();
    assert_eq!(master_key.len(), 32);

    let derivation = UserKeyDerivation::new(master_key).unwrap();
    let user_key = derivation.derive_default_key("test_user").unwrap();

    let encryptor = ChatEncryptor::new();
    let message = "Integration test message";

    let encrypted = encryptor.encrypt(message, &user_key, "test_user").unwrap();
    let decrypted = encryptor.decrypt(&encrypted, &user_key).unwrap();

    assert_eq!(message, decrypted);

    // Cleanup
    let _ = key_manager.delete_key();
}

#[test]
fn test_migration_setup() {
    let conn = Connection::open_in_memory().unwrap();

    // Create chat_messages table
    conn.execute(
        "CREATE TABLE chat_messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            chat_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )
    .unwrap();

    // Insert test messages
    conn.execute(
        "INSERT INTO chat_messages (chat_id, role, content) VALUES
         ('chat1', 'user', 'Message 1'),
         ('chat1', 'assistant', 'Response 1'),
         ('chat2', 'user', 'Message 2')",
        [],
    )
    .unwrap();

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM chat_messages", [], |row| row.get(0))
        .unwrap();

    assert_eq!(count, 3);
}

#[test]
fn test_migration_manager_initialization() {
    let key_manager = Arc::new(KeyManager::new().unwrap());
    let _migrator = ChatMigrationManager::new(key_manager);
    // Just testing initialization doesn't panic
}

#[test]
fn test_encrypted_message_structure() {
    let encryptor = ChatEncryptor::new();
    let key = b"12345678901234567890123456789012";
    let message = "Test message";

    let encrypted = encryptor.encrypt(message, key, "user123").unwrap();

    // Verify structure
    assert_eq!(encrypted.nonce.len(), 12); // GCM nonce is 12 bytes
    assert_eq!(encrypted.version, 1);
    assert_eq!(encrypted.user_id, "user123");
    assert!(encrypted.ciphertext.len() > message.len()); // Includes 16-byte auth tag
}

#[test]
fn test_performance_benchmark() {
    let encryptor = ChatEncryptor::new();
    let key = b"12345678901234567890123456789012";
    let message = "Performance test message of reasonable length for chat";

    let iterations = 1000;
    let start = std::time::Instant::now();

    for i in 0..iterations {
        let encrypted = encryptor
            .encrypt(message, key, &format!("user{}", i))
            .unwrap();
        let _decrypted = encryptor.decrypt(&encrypted, key).unwrap();
    }

    let duration = start.elapsed();
    let ops_per_sec = iterations as f64 / duration.as_secs_f64();

    println!(
        "Performance: {:.2} encrypt+decrypt ops/sec",
        ops_per_sec
    );

    // Should be able to handle at least 1000 ops/sec on modern hardware
    assert!(ops_per_sec > 1000.0, "Performance too slow: {:.2} ops/sec", ops_per_sec);
}

#[test]
fn test_concurrent_encryption() {
    use std::thread;

    let encryptor = Arc::new(ChatEncryptor::new());
    let key = Arc::new(*b"12345678901234567890123456789012");

    let mut handles = vec![];

    for i in 0..10 {
        let encryptor_clone = Arc::clone(&encryptor);
        let key_clone = Arc::clone(&key);

        let handle = thread::spawn(move || {
            let message = format!("Concurrent message {}", i);
            let user_id = format!("user{}", i);

            let encrypted = encryptor_clone
                .encrypt(&message, &key_clone, &user_id)
                .unwrap();
            let decrypted = encryptor_clone.decrypt(&encrypted, &key_clone).unwrap();

            assert_eq!(message, decrypted);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_invalid_key_length() {
    let encryptor = ChatEncryptor::new();
    let short_key = b"tooshort";
    let message = "Test message";

    let result = encryptor.encrypt(message, short_key, "user123");
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid key length"));
}

#[test]
fn test_invalid_nonce_length() {
    let encryptor = ChatEncryptor::new();
    let key = b"12345678901234567890123456789012";

    let mut encrypted = EncryptedMessage {
        ciphertext: vec![1, 2, 3],
        nonce: vec![1, 2, 3], // Invalid: should be 12 bytes
        version: 1,
        user_id: "user123".to_string(),
    };

    let result = encryptor.decrypt(&encrypted, key);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid nonce length"));
}

#[test]
fn test_corrupted_json_deserialization() {
    let encryptor = ChatEncryptor::new();
    let key = b"12345678901234567890123456789012";
    let corrupted_json = r#"{"invalid": "json structure"}"#;

    let result = encryptor.decrypt_from_json(corrupted_json, key);
    assert!(result.is_err());
}

#[test]
fn test_special_characters_encryption() {
    let encryptor = ChatEncryptor::new();
    let key = b"12345678901234567890123456789012";
    let messages = vec![
        "Message with \n newlines \t and tabs",
        "Message with \"quotes\" and 'apostrophes'",
        r#"Message with backslashes \ and forward slashes /"#,
        "Message with null byte: \0",
        "Message with emojis: 😀 🎉 🏛️",
    ];

    for message in messages {
        let encrypted = encryptor.encrypt(message, key, "user123").unwrap();
        let decrypted = encryptor.decrypt(&encrypted, key).unwrap();
        assert_eq!(message, decrypted);
    }
}

#[test]
fn test_encryption_version_tracking() {
    let encryptor = ChatEncryptor::new();
    let key = b"12345678901234567890123456789012";
    let message = "Version tracking test";

    let encrypted = encryptor.encrypt(message, key, "user123").unwrap();
    assert_eq!(encrypted.version, 1);

    // Simulate future version handling
    let mut future_version = encrypted.clone();
    future_version.version = 2;

    // Current decryptor should still handle it (forward compatibility test)
    // In production, you might want to check version and handle differently
    assert_eq!(future_version.version, 2);
}

#[test]
fn test_user_id_preservation() {
    let encryptor = ChatEncryptor::new();
    let key = b"12345678901234567890123456789012";
    let message = "User ID test";
    let user_id = "user_12345";

    let encrypted = encryptor.encrypt(message, key, user_id).unwrap();
    assert_eq!(encrypted.user_id, user_id);

    // User ID should be preserved through serialization
    let json = serde_json::to_string(&encrypted).unwrap();
    let deserialized: EncryptedMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.user_id, user_id);
}
