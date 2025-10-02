// SPDX-License-Identifier: MIT
// Copyright (c) 2025 BEAR AI LLM
//
// SQLCipher Database Encryption
// GDPR Article 32 - Security of Processing
//
// This module provides encrypted SQLite database connections using SQLCipher.
// All data at rest is encrypted with AES-256 encryption.

use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::key_manager::KeyManager;

/// Configuration for encrypted database
#[derive(Debug, Clone)]
pub struct EncryptionConfig {
    /// SQLCipher version compatibility
    pub cipher_version: u32,
    /// Key derivation iterations (higher = more secure but slower)
    pub kdf_iter: u32,
    /// Page size in bytes
    pub page_size: u32,
    /// HMAC algorithm
    pub hmac_algorithm: HmacAlgorithm,
}

#[derive(Debug, Clone, Copy)]
pub enum HmacAlgorithm {
    Sha1,
    Sha256,
    Sha512,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            cipher_version: 4, // SQLCipher 4.x
            kdf_iter: 256_000, // PBKDF2 iterations (SQLCipher 4 default)
            page_size: 4096,   // 4KB pages
            hmac_algorithm: HmacAlgorithm::Sha512,
        }
    }
}

impl EncryptionConfig {
    /// High security configuration with increased KDF iterations
    pub fn high_security() -> Self {
        Self {
            cipher_version: 4,
            kdf_iter: 500_000, // Increased iterations for higher security
            page_size: 4096,
            hmac_algorithm: HmacAlgorithm::Sha512,
        }
    }

    /// Balanced configuration (default)
    pub fn balanced() -> Self {
        Self::default()
    }

    /// Performance-optimized configuration
    pub fn performance() -> Self {
        Self {
            cipher_version: 4,
            kdf_iter: 64_000, // Reduced iterations for better performance
            page_size: 8192,  // Larger page size
            hmac_algorithm: HmacAlgorithm::Sha256,
        }
    }
}

impl HmacAlgorithm {
    fn to_sqlcipher_value(&self) -> &str {
        match self {
            HmacAlgorithm::Sha1 => "HMAC_SHA1",
            HmacAlgorithm::Sha256 => "HMAC_SHA256",
            HmacAlgorithm::Sha512 => "HMAC_SHA512",
        }
    }
}

/// Encrypted database connection wrapper
pub struct EncryptedDatabase {
    key_manager: Arc<KeyManager>,
    config: EncryptionConfig,
    db_path: PathBuf,
}

impl EncryptedDatabase {
    /// Create a new encrypted database instance
    pub fn new<P: AsRef<Path>>(db_path: P, config: EncryptionConfig) -> Result<Self> {
        let key_manager = Arc::new(KeyManager::new()?);

        Ok(Self {
            key_manager,
            config,
            db_path: db_path.as_ref().to_path_buf(),
        })
    }

    /// Create with default security configuration
    pub fn with_default_config<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        Self::new(db_path, EncryptionConfig::default())
    }

    /// Open or create an encrypted database connection
    pub fn connect(&self) -> Result<Connection> {
        self.connect_with_context(None)
    }

    /// Open or create an encrypted database connection with a specific key context
    ///
    /// This allows multiple databases to use different derived keys from the same master key
    pub fn connect_with_context(&self, context: Option<&str>) -> Result<Connection> {
        // Get encryption key
        let key = self.key_manager.get_sqlcipher_key(context)?;

        // Open connection
        let conn = Connection::open(&self.db_path)
            .with_context(|| format!("Failed to open database at {:?}", self.db_path))?;

        // Configure SQLCipher encryption
        self.configure_encryption(&conn, &key)?;

        // Verify encryption is working
        self.verify_encryption(&conn)?;

        Ok(conn)
    }

    /// Configure SQLCipher encryption parameters
    fn configure_encryption(&self, conn: &Connection, key: &str) -> Result<()> {
        // Set encryption key
        conn.execute(&format!("PRAGMA key = {}", key), [])
            .context("Failed to set encryption key")?;

        // Configure cipher compatibility version
        conn.execute(
            &format!(
                "PRAGMA cipher_compatibility = {}",
                self.config.cipher_version
            ),
            [],
        )
        .context("Failed to set cipher version")?;

        // Configure KDF iterations
        conn.execute(&format!("PRAGMA kdf_iter = {}", self.config.kdf_iter), [])
            .context("Failed to set KDF iterations")?;

        // Configure page size
        conn.execute(
            &format!("PRAGMA cipher_page_size = {}", self.config.page_size),
            [],
        )
        .context("Failed to set page size")?;

        // Configure HMAC algorithm
        conn.execute(
            &format!(
                "PRAGMA cipher_hmac_algorithm = {}",
                self.config.hmac_algorithm.to_sqlcipher_value()
            ),
            [],
        )
        .context("Failed to set HMAC algorithm")?;

        Ok(())
    }

    /// Verify that encryption is properly configured
    fn verify_encryption(&self, conn: &Connection) -> Result<()> {
        // Try to query cipher_version to ensure encryption is active
        let cipher_version: String = conn
            .query_row("PRAGMA cipher_version", [], |row| row.get(0))
            .context("Failed to verify encryption - SQLCipher may not be properly configured")?;

        if !cipher_version.contains("sqlcipher") {
            anyhow::bail!("Database is not encrypted with SQLCipher");
        }

        Ok(())
    }

    /// Create a new encrypted database from scratch
    pub fn create_new(&self) -> Result<Connection> {
        // Ensure database doesn't exist
        if self.db_path.exists() {
            anyhow::bail!("Database already exists at {:?}", self.db_path);
        }

        // Create encrypted database
        let conn = self.connect()?;

        // Initialize with a simple query to ensure file is created
        conn.execute(
            "CREATE TABLE IF NOT EXISTS _encryption_meta (
                id INTEGER PRIMARY KEY,
                created_at TEXT NOT NULL,
                encryption_version INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "INSERT INTO _encryption_meta (id, created_at, encryption_version) VALUES (1, datetime('now'), ?)",
            [self.config.cipher_version],
        )?;

        Ok(conn)
    }

    /// Change the encryption key (re-key the database)
    ///
    /// This operation re-encrypts the entire database with a new key
    pub fn rekey(&self, new_key_context: Option<&str>) -> Result<()> {
        let conn = self.connect()?;

        // Get new encryption key
        let new_key = self.key_manager.get_sqlcipher_key(new_key_context)?;

        // Rekey the database
        conn.execute(&format!("PRAGMA rekey = {}", new_key), [])
            .context("Failed to rekey database")?;

        Ok(())
    }

    /// Export encrypted database to an unencrypted database
    ///
    /// WARNING: This removes encryption protection. Use only for authorized exports
    /// under GDPR data portability requirements (Article 20)
    pub fn export_unencrypted<P: AsRef<Path>>(&self, output_path: P) -> Result<()> {
        let conn = self.connect()?;

        // Attach unencrypted database
        conn.execute(
            &format!(
                "ATTACH DATABASE '{}' AS plaintext KEY ''",
                output_path.as_ref().display()
            ),
            [],
        )?;

        // Export schema and data
        conn.execute("SELECT sqlcipher_export('plaintext')", [])?;

        // Detach database
        conn.execute("DETACH DATABASE plaintext", [])?;

        Ok(())
    }

    /// Get the database path
    pub fn path(&self) -> &Path {
        &self.db_path
    }

    /// Get the encryption configuration
    pub fn config(&self) -> &EncryptionConfig {
        &self.config
    }
}

/// Migrate an unencrypted database to encrypted format
pub struct DatabaseMigration {
    source_path: PathBuf,
    target_path: PathBuf,
    config: EncryptionConfig,
}

impl DatabaseMigration {
    /// Create a new migration instance
    pub fn new<P: AsRef<Path>>(source_path: P, target_path: P, config: EncryptionConfig) -> Self {
        Self {
            source_path: source_path.as_ref().to_path_buf(),
            target_path: target_path.as_ref().to_path_buf(),
            config,
        }
    }

    /// Execute the migration from unencrypted to encrypted database
    pub fn migrate(&self) -> Result<()> {
        // Verify source exists
        if !self.source_path.exists() {
            anyhow::bail!("Source database does not exist: {:?}", self.source_path);
        }

        // Verify target doesn't exist
        if self.target_path.exists() {
            anyhow::bail!("Target database already exists: {:?}", self.target_path);
        }

        // Open source (unencrypted) database
        let source_conn =
            Connection::open(&self.source_path).context("Failed to open source database")?;

        // Create encrypted target database
        let encrypted_db = EncryptedDatabase::new(&self.target_path, self.config.clone())?;
        let key = encrypted_db.key_manager.get_sqlcipher_key(None)?;

        // Attach encrypted target database
        source_conn.execute(
            &format!(
                "ATTACH DATABASE '{}' AS encrypted KEY {}",
                self.target_path.display(),
                key
            ),
            [],
        )?;

        // Configure encryption on attached database
        encrypted_db.configure_encryption(&source_conn, &key)?;

        // Export to encrypted database
        source_conn
            .execute("SELECT sqlcipher_export('encrypted')", [])
            .context("Failed to export data to encrypted database")?;

        // Detach
        source_conn.execute("DETACH DATABASE encrypted", [])?;

        // Verify encrypted database
        let verify_conn = encrypted_db.connect()?;
        encrypted_db.verify_encryption(&verify_conn)?;

        Ok(())
    }

    /// Execute migration and optionally backup the original
    pub fn migrate_with_backup(&self) -> Result<()> {
        use std::fs;

        // Create backup
        let backup_path = self.source_path.with_extension("db.backup");
        fs::copy(&self.source_path, &backup_path).context("Failed to create backup")?;

        // Attempt migration
        match self.migrate() {
            Ok(()) => {
                tracing::info!(
                    "Successfully migrated database. Backup saved at {:?}",
                    backup_path
                );
                Ok(())
            }
            Err(e) => {
                // Migration failed, backup is preserved
                tracing::error!(
                    "Migration failed: {}. Original database backed up at {:?}",
                    e,
                    backup_path
                );
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_encrypted_database() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let encrypted_db = EncryptedDatabase::with_default_config(&db_path).unwrap();
        let conn = encrypted_db.create_new().unwrap();

        // Verify database was created
        assert!(db_path.exists());

        // Verify we can query it
        let version: String = conn
            .query_row("SELECT sqlite_version()", [], |row| row.get(0))
            .unwrap();
        assert!(!version.is_empty());

        // Verify encryption metadata
        let enc_version: u32 = conn
            .query_row(
                "SELECT encryption_version FROM _encryption_meta WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(enc_version, 4);
    }

    #[test]
    fn test_encrypted_database_operations() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let encrypted_db = EncryptedDatabase::with_default_config(&db_path).unwrap();
        let conn = encrypted_db.create_new().unwrap();

        // Create test table
        conn.execute(
            "CREATE TABLE test_data (id INTEGER PRIMARY KEY, value TEXT)",
            [],
        )
        .unwrap();

        // Insert data
        conn.execute(
            "INSERT INTO test_data (value) VALUES (?)",
            ["sensitive data"],
        )
        .unwrap();

        // Query data
        let value: String = conn
            .query_row("SELECT value FROM test_data WHERE id = 1", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(value, "sensitive data");

        // Close connection
        drop(conn);

        // Reopen and verify data persists
        let conn2 = encrypted_db.connect().unwrap();
        let value2: String = conn2
            .query_row("SELECT value FROM test_data WHERE id = 1", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(value2, "sensitive data");
    }

    #[test]
    fn test_wrong_key_fails() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create encrypted database
        let encrypted_db = EncryptedDatabase::with_default_config(&db_path).unwrap();
        let _conn = encrypted_db.create_new().unwrap();
        drop(_conn);

        // Try to open with wrong key (different context)
        let result = encrypted_db.connect_with_context(Some("wrong-context"));

        // Should fail to verify encryption or read data
        assert!(
            result.is_err() || {
                let conn = result.unwrap();
                conn.query_row("SELECT COUNT(*) FROM _encryption_meta", [], |row| {
                    row.get::<_, i64>(0)
                })
                .is_err()
            }
        );
    }

    #[test]
    fn test_database_migration() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("source.db");
        let target_path = temp_dir.path().join("target.db");

        // Create unencrypted source database
        {
            let conn = Connection::open(&source_path).unwrap();
            conn.execute(
                "CREATE TABLE test_data (id INTEGER PRIMARY KEY, value TEXT)",
                [],
            )
            .unwrap();
            conn.execute("INSERT INTO test_data (value) VALUES ('test data')", [])
                .unwrap();
        }

        // Migrate to encrypted database
        let migration =
            DatabaseMigration::new(&source_path, &target_path, EncryptionConfig::default());
        migration.migrate().unwrap();

        // Verify encrypted database
        let encrypted_db = EncryptedDatabase::with_default_config(&target_path).unwrap();
        let conn = encrypted_db.connect().unwrap();

        let value: String = conn
            .query_row("SELECT value FROM test_data WHERE id = 1", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(value, "test data");
    }

    #[test]
    fn test_export_unencrypted() {
        let temp_dir = TempDir::new().unwrap();
        let encrypted_path = temp_dir.path().join("encrypted.db");
        let unencrypted_path = temp_dir.path().join("unencrypted.db");

        // Create encrypted database
        let encrypted_db = EncryptedDatabase::with_default_config(&encrypted_path).unwrap();
        let conn = encrypted_db.create_new().unwrap();

        conn.execute(
            "CREATE TABLE test_data (id INTEGER PRIMARY KEY, value TEXT)",
            [],
        )
        .unwrap();
        conn.execute("INSERT INTO test_data (value) VALUES ('test')", [])
            .unwrap();
        drop(conn);

        // Export to unencrypted
        encrypted_db.export_unencrypted(&unencrypted_path).unwrap();

        // Verify unencrypted database can be opened without key
        let plain_conn = Connection::open(&unencrypted_path).unwrap();
        let value: String = plain_conn
            .query_row("SELECT value FROM test_data WHERE id = 1", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(value, "test");
    }

    #[test]
    fn test_different_security_configs() {
        let temp_dir = TempDir::new().unwrap();

        // Test high security config
        let high_security_path = temp_dir.path().join("high_security.db");
        let high_security_db =
            EncryptedDatabase::new(&high_security_path, EncryptionConfig::high_security()).unwrap();
        let conn = high_security_db.create_new().unwrap();
        drop(conn);

        // Test performance config
        let performance_path = temp_dir.path().join("performance.db");
        let performance_db =
            EncryptedDatabase::new(&performance_path, EncryptionConfig::performance()).unwrap();
        let conn = performance_db.create_new().unwrap();
        drop(conn);

        // Both should work
        assert!(high_security_path.exists());
        assert!(performance_path.exists());
    }
}
