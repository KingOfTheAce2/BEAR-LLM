// SPDX-License-Identifier: MIT
// Copyright (c) 2025 BEAR AI LLM
//
// Encrypted Database Connection Pool
// GDPR Article 32 - Security of Processing
//
// This module provides connection pooling for encrypted SQLite databases
// using r2d2 for efficient resource management.

use anyhow::{Context, Result};
use r2d2::{Pool, PooledConnection};
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use super::database_encryption::{EncryptedDatabase, EncryptionConfig};
use super::key_manager::KeyManager;

/// Custom error wrapper that implements std::error::Error for r2d2
#[derive(Debug)]
pub struct PoolError(String);

impl std::fmt::Display for PoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for PoolError {}

impl From<anyhow::Error> for PoolError {
    fn from(err: anyhow::Error) -> Self {
        PoolError(err.to_string())
    }
}

impl From<rusqlite::Error> for PoolError {
    fn from(err: rusqlite::Error) -> Self {
        PoolError(err.to_string())
    }
}

/// Connection manager for encrypted databases
pub struct EncryptedConnectionManager {
    key_manager: Arc<KeyManager>,
    db_path: PathBuf,
    config: EncryptionConfig,
    key_context: Option<String>,
}

impl EncryptedConnectionManager {
    /// Create a new connection manager
    pub fn new<P: AsRef<Path>>(
        db_path: P,
        config: EncryptionConfig,
        key_context: Option<String>,
    ) -> Result<Self> {
        let key_manager = Arc::new(KeyManager::new()?);

        Ok(Self {
            key_manager,
            db_path: db_path.as_ref().to_path_buf(),
            config,
            key_context,
        })
    }
}

impl r2d2::ManageConnection for EncryptedConnectionManager {
    type Connection = Connection;
    type Error = PoolError;

    fn connect(&self) -> Result<Self::Connection, Self::Error> {
        // Get encryption key
        let key = self
            .key_manager
            .get_sqlcipher_key(self.key_context.as_deref())
            .map_err(|e| PoolError(format!("Failed to get encryption key: {}", e)))?;

        // Open connection
        let conn = Connection::open(&self.db_path).map_err(|e| {
            PoolError(format!(
                "Failed to open database at {:?}: {}",
                self.db_path, e
            ))
        })?;

        // Configure SQLCipher encryption
        self.configure_encryption(&conn, &key)
            .map_err(|e| PoolError(format!("Failed to configure encryption: {}", e)))?;

        // Enable foreign keys
        conn.execute("PRAGMA foreign_keys = ON", [])
            .map_err(|e| PoolError(format!("Failed to enable foreign keys: {}", e)))?;

        // Set journal mode to WAL for better concurrency
        conn.execute("PRAGMA journal_mode = WAL", [])
            .map_err(|e| PoolError(format!("Failed to set WAL mode: {}", e)))?;

        Ok(conn)
    }

    fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        // Verify connection is still valid by executing a simple query
        conn.query_row("SELECT 1", [], |_| Ok(()))
            .map_err(|e| PoolError(format!("Connection validation failed: {}", e)))?;
        Ok(())
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        // SQLite connections don't really "break" in the traditional sense
        // The is_valid check is sufficient
        false
    }
}

impl EncryptedConnectionManager {
    fn configure_encryption(&self, conn: &Connection, key: &str) -> Result<()> {
        // Set encryption key - must be done before any other operations
        conn.execute_batch(&format!("PRAGMA key = \"{}\";", key))
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

        Ok(())
    }
}

/// Encrypted database connection pool
pub struct EncryptedPool {
    pool: Pool<EncryptedConnectionManager>,
    db_path: PathBuf,
}

impl EncryptedPool {
    /// Create a new encrypted connection pool
    pub fn new<P: AsRef<Path>>(
        db_path: P,
        config: EncryptionConfig,
        max_connections: u32,
    ) -> Result<Self> {
        Self::new_with_context(db_path, config, max_connections, None)
    }

    /// Create a new encrypted connection pool with key context
    pub fn new_with_context<P: AsRef<Path>>(
        db_path: P,
        config: EncryptionConfig,
        max_connections: u32,
        key_context: Option<String>,
    ) -> Result<Self> {
        let db_path = db_path.as_ref().to_path_buf();

        // Ensure database exists and is properly encrypted
        let encrypted_db = EncryptedDatabase::new(&db_path, config.clone())?;
        if !db_path.exists() {
            encrypted_db.create_new()?;
        } else {
            // Verify existing database is encrypted
            let _conn = encrypted_db.connect()?;
        }

        // Create connection manager
        let manager = EncryptedConnectionManager::new(&db_path, config, key_context)?;

        // Build pool
        let pool = Pool::builder()
            .max_size(max_connections)
            .min_idle(Some(1))
            .connection_timeout(Duration::from_secs(30))
            .idle_timeout(Some(Duration::from_secs(300)))
            .max_lifetime(Some(Duration::from_secs(1800)))
            .build(manager)
            .context("Failed to create connection pool")?;

        Ok(Self { pool, db_path })
    }

    /// Get a connection from the pool
    pub fn get(&self) -> Result<PooledConnection<EncryptedConnectionManager>> {
        self.pool
            .get()
            .context("Failed to get connection from pool")
    }

    /// Get the current pool state
    pub fn state(&self) -> r2d2::State {
        self.pool.state()
    }

    /// Get the database path
    pub fn path(&self) -> &Path {
        &self.db_path
    }

    /// Execute a query with a pooled connection
    pub fn with_connection<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T>,
    {
        let conn = self.get()?;
        f(&conn)
    }

    /// Execute a transaction with a pooled connection
    pub fn with_transaction<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&rusqlite::Transaction) -> Result<T>,
    {
        let mut conn = self.get()?;
        let tx = conn.transaction().context("Failed to start transaction")?;

        let result = f(&tx)?;

        tx.commit().context("Failed to commit transaction")?;

        Ok(result)
    }
}

/// Builder for creating encrypted connection pools with custom configuration
pub struct EncryptedPoolBuilder {
    db_path: PathBuf,
    config: EncryptionConfig,
    max_connections: u32,
    min_idle: Option<u32>,
    connection_timeout: Duration,
    idle_timeout: Option<Duration>,
    max_lifetime: Option<Duration>,
    key_context: Option<String>,
}

impl EncryptedPoolBuilder {
    /// Create a new pool builder
    pub fn new<P: AsRef<Path>>(db_path: P, config: EncryptionConfig) -> Self {
        Self {
            db_path: db_path.as_ref().to_path_buf(),
            config,
            max_connections: 10,
            min_idle: Some(1),
            connection_timeout: Duration::from_secs(30),
            idle_timeout: Some(Duration::from_secs(300)),
            max_lifetime: Some(Duration::from_secs(1800)),
            key_context: None,
        }
    }

    /// Set maximum number of connections
    pub fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }

    /// Set minimum idle connections
    pub fn min_idle(mut self, min: u32) -> Self {
        self.min_idle = Some(min);
        self
    }

    /// Set connection timeout
    pub fn connection_timeout(mut self, timeout: Duration) -> Self {
        self.connection_timeout = timeout;
        self
    }

    /// Set idle timeout
    pub fn idle_timeout(mut self, timeout: Duration) -> Self {
        self.idle_timeout = Some(timeout);
        self
    }

    /// Set maximum connection lifetime
    pub fn max_lifetime(mut self, lifetime: Duration) -> Self {
        self.max_lifetime = Some(lifetime);
        self
    }

    /// Set key derivation context
    pub fn key_context(mut self, context: String) -> Self {
        self.key_context = Some(context);
        self
    }

    /// Build the connection pool
    pub fn build(self) -> Result<EncryptedPool> {
        let db_path = self.db_path;

        // Ensure database exists and is properly encrypted
        let encrypted_db = EncryptedDatabase::new(&db_path, self.config.clone())?;
        if !db_path.exists() {
            encrypted_db.create_new()?;
        }

        // Create connection manager
        let manager = EncryptedConnectionManager::new(&db_path, self.config, self.key_context)?;

        // Build pool with custom configuration
        let mut builder = Pool::builder()
            .max_size(self.max_connections)
            .connection_timeout(self.connection_timeout);

        if let Some(min) = self.min_idle {
            builder = builder.min_idle(Some(min));
        }

        if let Some(timeout) = self.idle_timeout {
            builder = builder.idle_timeout(Some(timeout));
        }

        if let Some(lifetime) = self.max_lifetime {
            builder = builder.max_lifetime(Some(lifetime));
        }

        let pool = builder
            .build(manager)
            .context("Failed to create connection pool")?;

        Ok(EncryptedPool { pool, db_path })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_encrypted_pool_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let pool = EncryptedPool::new(&db_path, EncryptionConfig::default(), 5).unwrap();

        // Verify we can get a connection
        let conn = pool.get().unwrap();

        // Verify connection works
        let version: String = conn
            .query_row("SELECT sqlite_version()", [], |row| row.get(0))
            .unwrap();
        assert!(!version.is_empty());
    }

    #[test]
    fn test_pool_multiple_connections() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let pool = EncryptedPool::new(&db_path, EncryptionConfig::default(), 5).unwrap();

        // Get multiple connections
        let conn1 = pool.get().unwrap();
        let conn2 = pool.get().unwrap();
        let conn3 = pool.get().unwrap();

        // Verify all connections work
        for conn in [&conn1, &conn2, &conn3] {
            let result: i32 = conn.query_row("SELECT 1", [], |row| row.get(0)).unwrap();
            assert_eq!(result, 1);
        }
    }

    #[test]
    fn test_pool_with_transaction() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let pool = EncryptedPool::new(&db_path, EncryptionConfig::default(), 5).unwrap();

        // Create table
        pool.with_connection(|conn| {
            conn.execute("CREATE TABLE test (id INTEGER PRIMARY KEY, value TEXT)", [])?;
            Ok(())
        })
        .unwrap();

        // Insert data in transaction
        pool.with_transaction(|tx| {
            tx.execute("INSERT INTO test (value) VALUES (?)", ["test1"])?;
            tx.execute("INSERT INTO test (value) VALUES (?)", ["test2"])?;
            Ok(())
        })
        .unwrap();

        // Verify data
        pool.with_connection(|conn| {
            let count: i64 = conn.query_row("SELECT COUNT(*) FROM test", [], |row| row.get(0))?;
            assert_eq!(count, 2);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_pool_builder() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let pool = EncryptedPoolBuilder::new(&db_path, EncryptionConfig::default())
            .max_connections(10)
            .min_idle(2)
            .connection_timeout(Duration::from_secs(15))
            .build()
            .unwrap();

        let conn = pool.get().unwrap();

        let result: i32 = conn.query_row("SELECT 1", [], |row| row.get(0)).unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    fn test_pool_state() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let pool = EncryptedPool::new(&db_path, EncryptionConfig::default(), 5).unwrap();

        let state = pool.state();
        assert_eq!(state.connections, 0);
        assert_eq!(state.idle_connections, 0);

        // Get a connection
        let _conn = pool.get().unwrap();

        let state = pool.state();
        assert!(state.connections > 0);
    }
}
