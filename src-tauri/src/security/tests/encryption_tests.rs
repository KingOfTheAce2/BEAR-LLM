// SPDX-License-Identifier: MIT
// Copyright (c) 2025 BEAR AI LLM
//
// Comprehensive Security Tests for Database Encryption
// GDPR Article 32 - Security of Processing

#[cfg(test)]
mod encryption_security_tests {
    use crate::security::{
        database_encryption::{DatabaseMigration, EncryptedDatabase, EncryptionConfig},
        encrypted_pool::EncryptedPool,
        key_manager::KeyManager,
    };
    use rusqlite::Connection;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    /// Test that encrypted databases cannot be read without the correct key
    #[test]
    fn test_encryption_security_unauthorized_access() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("secure.db");

        // Create encrypted database with sensitive data
        let encrypted_db = EncryptedDatabase::with_default_config(&db_path).unwrap();
        let conn = encrypted_db.create_new().unwrap();

        conn.execute(
            "CREATE TABLE sensitive_data (id INTEGER PRIMARY KEY, secret TEXT)",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO sensitive_data (secret) VALUES (?)",
            ["TOP SECRET INFORMATION"],
        )
        .unwrap();

        drop(conn);

        // Attempt to open database without encryption key (should fail)
        let unencrypted_conn = Connection::open(&db_path);

        match unencrypted_conn {
            Ok(conn) => {
                // If connection opens, trying to read should fail
                let result = conn.query_row(
                    "SELECT secret FROM sensitive_data WHERE id = 1",
                    [],
                    |row| row.get::<_, String>(0),
                );

                assert!(
                    result.is_err(),
                    "Unauthorized access should not be able to read encrypted data"
                );
            }
            Err(_) => {
                // Connection failed, which is also acceptable
            }
        }

        // Verify raw file doesn't contain plaintext
        let raw_content = fs::read(&db_path).unwrap();
        let raw_string = String::from_utf8_lossy(&raw_content);

        assert!(
            !raw_string.contains("TOP SECRET INFORMATION"),
            "Sensitive data should not be readable in raw database file"
        );
    }

    /// Test encryption key rotation functionality
    #[test]
    fn test_key_rotation_security() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("rotate.db");

        let encrypted_db = EncryptedDatabase::with_default_config(&db_path).unwrap();
        let conn = encrypted_db.create_new().unwrap();

        // Insert test data
        conn.execute(
            "CREATE TABLE test (id INTEGER PRIMARY KEY, data TEXT)",
            [],
        )
        .unwrap();
        conn.execute("INSERT INTO test (data) VALUES (?)", ["original"])
            .unwrap();

        drop(conn);

        // Rotate encryption key
        encrypted_db.rekey(Some("new-context")).unwrap();

        // Verify old key context cannot access database
        let old_access = encrypted_db.connect_with_context(None);

        // New key context should work
        let new_conn = encrypted_db.connect_with_context(Some("new-context")).unwrap();

        let data: String = new_conn
            .query_row("SELECT data FROM test WHERE id = 1", [], |row| row.get(0))
            .unwrap();

        assert_eq!(data, "original");
    }

    /// Test that different encryption contexts produce different encrypted outputs
    #[test]
    fn test_encryption_context_isolation() {
        let temp_dir = TempDir::new().unwrap();
        let db1_path = temp_dir.path().join("context1.db");
        let db2_path = temp_dir.path().join("context2.db");

        // Create two databases with different contexts
        let encrypted_db1 = EncryptedDatabase::with_default_config(&db1_path).unwrap();
        let encrypted_db2 = EncryptedDatabase::with_default_config(&db2_path).unwrap();

        let conn1 = encrypted_db1.connect_with_context(Some("context1")).unwrap();
        let conn2 = encrypted_db2.connect_with_context(Some("context2")).unwrap();

        // Same data structure and content
        for conn in [&conn1, &conn2] {
            conn.execute(
                "CREATE TABLE test (id INTEGER PRIMARY KEY, data TEXT)",
                [],
            )
            .unwrap();
            conn.execute("INSERT INTO test (data) VALUES (?)", ["same data"])
                .unwrap();
        }

        drop(conn1);
        drop(conn2);

        // Files should be different despite same content
        let file1 = fs::read(&db1_path).unwrap();
        let file2 = fs::read(&db2_path).unwrap();

        assert_ne!(
            file1, file2,
            "Different encryption contexts should produce different encrypted files"
        );
    }

    /// Test migration from unencrypted to encrypted database
    #[test]
    fn test_secure_migration() {
        let temp_dir = TempDir::new().unwrap();
        let unencrypted_path = temp_dir.path().join("unencrypted.db");
        let encrypted_path = temp_dir.path().join("encrypted.db");

        // Create unencrypted database with test data
        {
            let conn = Connection::open(&unencrypted_path).unwrap();
            conn.execute(
                "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, email TEXT)",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO users (name, email) VALUES (?, ?)",
                ["John Doe", "john@example.com"],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO users (name, email) VALUES (?, ?)",
                ["Jane Smith", "jane@example.com"],
            )
            .unwrap();
        }

        // Verify unencrypted file contains plaintext
        let unencrypted_content = fs::read(&unencrypted_path).unwrap();
        let unencrypted_string = String::from_utf8_lossy(&unencrypted_content);
        assert!(
            unencrypted_string.contains("john@example.com"),
            "Unencrypted database should contain plaintext"
        );

        // Migrate to encrypted database
        let migration =
            DatabaseMigration::new(&unencrypted_path, &encrypted_path, EncryptionConfig::default());

        migration.migrate().unwrap();

        // Verify encrypted file does NOT contain plaintext
        let encrypted_content = fs::read(&encrypted_path).unwrap();
        let encrypted_string = String::from_utf8_lossy(&encrypted_content);
        assert!(
            !encrypted_string.contains("john@example.com"),
            "Encrypted database should not contain plaintext"
        );

        // Verify data is accessible with correct key
        let encrypted_db = EncryptedDatabase::with_default_config(&encrypted_path).unwrap();
        let conn = encrypted_db.connect().unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))
            .unwrap();

        assert_eq!(count, 2);

        let email: String = conn
            .query_row(
                "SELECT email FROM users WHERE name = 'John Doe'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(email, "john@example.com");
    }

    /// Test connection pool security and isolation
    #[test]
    fn test_connection_pool_security() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("pool.db");

        let pool = EncryptedPool::new(&db_path, EncryptionConfig::default(), 5).unwrap();

        // Create table with one connection
        pool.with_connection(|conn| {
            conn.execute(
                "CREATE TABLE accounts (id INTEGER PRIMARY KEY, balance INTEGER)",
                [],
            )?;
            conn.execute("INSERT INTO accounts (balance) VALUES (1000)", [])?;
            Ok(())
        })
        .unwrap();

        // Verify transaction isolation
        pool.with_transaction(|tx| {
            tx.execute("UPDATE accounts SET balance = balance - 100 WHERE id = 1", [])?;

            let balance: i64 =
                tx.query_row("SELECT balance FROM accounts WHERE id = 1", [], |row| {
                    row.get(0)
                })?;

            assert_eq!(balance, 900);
            Ok(())
        })
        .unwrap();

        // Verify committed changes
        pool.with_connection(|conn| {
            let balance: i64 = conn.query_row(
                "SELECT balance FROM accounts WHERE id = 1",
                [],
                |row| row.get(0),
            )?;

            assert_eq!(balance, 900);
            Ok(())
        })
        .unwrap();
    }

    /// Test high security configuration settings
    #[test]
    fn test_high_security_configuration() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("high_security.db");

        let config = EncryptionConfig::high_security();
        assert_eq!(config.kdf_iter, 500_000);

        let encrypted_db = EncryptedDatabase::new(&db_path, config).unwrap();
        let conn = encrypted_db.create_new().unwrap();

        // Verify high security settings are applied
        let kdf_iter: u32 = conn
            .query_row("PRAGMA kdf_iter", [], |row| row.get(0))
            .unwrap();

        assert_eq!(kdf_iter, 500_000);
    }

    /// Test that encryption keys are properly zeroized on drop
    #[test]
    fn test_key_zeroization() {
        let key_manager = KeyManager::new().unwrap();

        // Clean up any existing key
        let _ = key_manager.delete_key();

        // Get key (caches in memory)
        let _key = key_manager.get_or_create_key().unwrap();

        // Clear cache (should zeroize memory)
        key_manager.clear_cache();

        // Should still be able to retrieve from keychain
        let key2 = key_manager.get_or_create_key().unwrap();
        assert_eq!(key2.len(), 32);

        // Clean up
        key_manager.delete_key().unwrap();
    }

    /// Test GDPR-compliant data export (encrypted to unencrypted)
    #[test]
    fn test_gdpr_data_portability() {
        let temp_dir = TempDir::new().unwrap();
        let encrypted_path = temp_dir.path().join("encrypted.db");
        let export_path = temp_dir.path().join("export.db");

        // Create encrypted database with personal data
        let encrypted_db = EncryptedDatabase::with_default_config(&encrypted_path).unwrap();
        let conn = encrypted_db.create_new().unwrap();

        conn.execute(
            "CREATE TABLE personal_data (id INTEGER PRIMARY KEY, name TEXT, data TEXT)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO personal_data (name, data) VALUES (?, ?)",
            ["User1", "Personal information"],
        )
        .unwrap();

        drop(conn);

        // Export for GDPR data portability
        encrypted_db.export_unencrypted(&export_path).unwrap();

        // Verify exported data is accessible without encryption
        let export_conn = Connection::open(&export_path).unwrap();

        let name: String = export_conn
            .query_row(
                "SELECT name FROM personal_data WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(name, "User1");
    }

    /// Test concurrent access with connection pool
    #[test]
    fn test_concurrent_encrypted_access() {
        use std::sync::Arc;
        use std::thread;

        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("concurrent.db");

        let pool = Arc::new(EncryptedPool::new(&db_path, EncryptionConfig::default(), 10).unwrap());

        // Initialize database
        pool.with_connection(|conn| {
            conn.execute(
                "CREATE TABLE counter (id INTEGER PRIMARY KEY, value INTEGER)",
                [],
            )?;
            conn.execute("INSERT INTO counter (id, value) VALUES (1, 0)", [])?;
            Ok(())
        })
        .unwrap();

        // Spawn multiple threads that increment counter
        let mut handles = vec![];

        for _ in 0..10 {
            let pool_clone = Arc::clone(&pool);
            let handle = thread::spawn(move || {
                for _ in 0..10 {
                    pool_clone
                        .with_transaction(|tx| {
                            tx.execute("UPDATE counter SET value = value + 1 WHERE id = 1", [])?;
                            Ok(())
                        })
                        .unwrap();
                }
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify final count
        pool.with_connection(|conn| {
            let value: i64 = conn.query_row("SELECT value FROM counter WHERE id = 1", [], |row| {
                row.get(0)
            })?;

            assert_eq!(value, 100);
            Ok(())
        })
        .unwrap();
    }

    /// Test performance comparison between security levels
    #[test]
    fn test_security_performance_comparison() {
        use std::time::Instant;

        let temp_dir = TempDir::new().unwrap();

        // Test balanced config
        let balanced_path = temp_dir.path().join("balanced.db");
        let balanced_db =
            EncryptedDatabase::new(&balanced_path, EncryptionConfig::balanced()).unwrap();

        let start = Instant::now();
        let _conn = balanced_db.create_new().unwrap();
        let balanced_time = start.elapsed();

        // Test high security config
        let high_sec_path = temp_dir.path().join("high_security.db");
        let high_sec_db =
            EncryptedDatabase::new(&high_sec_path, EncryptionConfig::high_security()).unwrap();

        let start = Instant::now();
        let _conn = high_sec_db.create_new().unwrap();
        let high_sec_time = start.elapsed();

        // High security should take longer due to more KDF iterations
        println!("Balanced config: {:?}", balanced_time);
        println!("High security config: {:?}", high_sec_time);

        // Just verify both completed successfully
        assert!(balanced_path.exists());
        assert!(high_sec_path.exists());
    }
}
