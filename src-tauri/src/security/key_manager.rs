// SPDX-License-Identifier: MIT
// Copyright (c) 2025 BEAR AI LLM
//
// Secure Key Management for Database Encryption
// GDPR Article 32 - Security of Processing
//
// This module manages encryption keys using the OS credential store
// for secure, persistent key storage across application restarts.

use anyhow::{Context, Result};
use keyring::Entry;
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};
use zeroize::Zeroize;

const SERVICE_NAME: &str = "bear-ai-llm";
const KEY_NAME: &str = "database-encryption-key";
const KEY_LENGTH: usize = 32; // 256-bit key for AES-256

/// Secure key manager that stores encryption keys in OS keychain
pub struct KeyManager {
    entry: Entry,
    cached_key: Arc<Mutex<Option<Vec<u8>>>>,
}

impl KeyManager {
    /// Create a new key manager instance
    pub fn new() -> Result<Self> {
        let entry = Entry::new(SERVICE_NAME, KEY_NAME)
            .context("Failed to create keyring entry")?;

        Ok(Self {
            entry,
            cached_key: Arc::new(Mutex::new(None)),
        })
    }

    /// Get or create the database encryption key
    ///
    /// This method:
    /// 1. Checks if key exists in OS keychain
    /// 2. If not, generates a new secure random key
    /// 3. Stores the key in OS keychain
    /// 4. Caches the key in memory for performance
    pub fn get_or_create_key(&self) -> Result<Vec<u8>> {
        // Check memory cache first
        {
            let cached = self.cached_key.lock().unwrap();
            if let Some(key) = &*cached {
                return Ok(key.clone());
            }
        }

        // Try to retrieve from keychain
        match self.entry.get_password() {
            Ok(stored_key) => {
                let key = hex::decode(&stored_key)
                    .context("Failed to decode stored key")?;

                if key.len() != KEY_LENGTH {
                    anyhow::bail!("Invalid key length in keychain");
                }

                // Cache the key
                let mut cached = self.cached_key.lock().unwrap();
                *cached = Some(key.clone());

                Ok(key)
            }
            Err(keyring::Error::NoEntry) => {
                // Key doesn't exist, generate new one
                let key = self.generate_key()?;
                self.store_key(&key)?;

                // Cache the key
                let mut cached = self.cached_key.lock().unwrap();
                *cached = Some(key.clone());

                Ok(key)
            }
            Err(e) => {
                anyhow::bail!("Failed to access keychain: {}", e);
            }
        }
    }

    /// Generate a cryptographically secure random key
    fn generate_key(&self) -> Result<Vec<u8>> {
        use rand::RngCore;
        let mut key = vec![0u8; KEY_LENGTH];

        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut key);

        Ok(key)
    }

    /// Store key securely in OS keychain
    fn store_key(&self, key: &[u8]) -> Result<()> {
        let encoded_key = hex::encode(key);
        self.entry
            .set_password(&encoded_key)
            .context("Failed to store key in keychain")?;
        Ok(())
    }

    /// Rotate the encryption key
    ///
    /// WARNING: This will require re-encrypting all existing databases
    /// Use with caution and ensure proper migration procedures
    pub fn rotate_key(&self) -> Result<Vec<u8>> {
        // Generate new key
        let new_key = self.generate_key()?;

        // Store in keychain
        self.store_key(&new_key)?;

        // Update cache
        let mut cached = self.cached_key.lock().unwrap();
        *cached = Some(new_key.clone());

        Ok(new_key)
    }

    /// Delete the encryption key from keychain
    ///
    /// WARNING: This will make encrypted databases inaccessible
    /// Only use for testing or complete data removal
    pub fn delete_key(&self) -> Result<()> {
        self.entry
            .delete_credential()
            .context("Failed to delete key from keychain")?;

        // Clear cache
        let mut cached = self.cached_key.lock().unwrap();
        if let Some(ref mut key) = *cached {
            key.zeroize();
        }
        *cached = None;

        Ok(())
    }

    /// Derive a database-specific key from the master key
    ///
    /// This allows using a single master key to encrypt multiple databases
    /// with different derived keys for additional security
    pub fn derive_key(&self, context: &str) -> Result<Vec<u8>> {
        let master_key = self.get_or_create_key()?;

        let mut hasher = Sha256::new();
        hasher.update(&master_key);
        hasher.update(context.as_bytes());

        Ok(hasher.finalize().to_vec())
    }

    /// Get the hex-encoded key for SQLCipher PRAGMA key command
    pub fn get_sqlcipher_key(&self, context: Option<&str>) -> Result<String> {
        let key = match context {
            Some(ctx) => self.derive_key(ctx)?,
            None => self.get_or_create_key()?,
        };

        // SQLCipher expects hex-encoded key with 'x' prefix
        Ok(format!("x'{}'", hex::encode(key)))
    }

    /// Clear the in-memory key cache
    ///
    /// Useful for security-critical operations where you want to
    /// minimize the time keys are held in memory
    pub fn clear_cache(&self) {
        let mut cached = self.cached_key.lock().unwrap();
        if let Some(ref mut key) = *cached {
            key.zeroize();
        }
        *cached = None;
    }
}

impl Drop for KeyManager {
    fn drop(&mut self) {
        // Ensure keys are zeroized when KeyManager is dropped
        self.clear_cache();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let manager = KeyManager::new().unwrap();

        // Clean up any existing test key
        let _ = manager.delete_key();

        // Generate first key
        let key1 = manager.get_or_create_key().unwrap();
        assert_eq!(key1.len(), KEY_LENGTH);

        // Retrieving again should return same key
        let key2 = manager.get_or_create_key().unwrap();
        assert_eq!(key1, key2);

        // Clean up
        manager.delete_key().unwrap();
    }

    #[test]
    fn test_key_derivation() {
        let manager = KeyManager::new().unwrap();

        // Clean up any existing test key
        let _ = manager.delete_key();

        // Derive keys with different contexts
        let key1 = manager.derive_key("context1").unwrap();
        let key2 = manager.derive_key("context2").unwrap();

        // Keys should be different
        assert_ne!(key1, key2);

        // Same context should produce same key
        let key1_again = manager.derive_key("context1").unwrap();
        assert_eq!(key1, key1_again);

        // Clean up
        manager.delete_key().unwrap();
    }

    #[test]
    fn test_sqlcipher_key_format() {
        let manager = KeyManager::new().unwrap();

        // Clean up any existing test key
        let _ = manager.delete_key();

        let sqlcipher_key = manager.get_sqlcipher_key(None).unwrap();

        // Should start with x'
        assert!(sqlcipher_key.starts_with("x'"));
        // Should end with '
        assert!(sqlcipher_key.ends_with('\''));
        // Should be hex-encoded (2 chars per byte + 3 for x' and ')
        assert_eq!(sqlcipher_key.len(), KEY_LENGTH * 2 + 3);

        // Clean up
        manager.delete_key().unwrap();
    }

    #[test]
    fn test_key_rotation() {
        let manager = KeyManager::new().unwrap();

        // Clean up any existing test key
        let _ = manager.delete_key();

        let original_key = manager.get_or_create_key().unwrap();
        let rotated_key = manager.rotate_key().unwrap();

        // Keys should be different
        assert_ne!(original_key, rotated_key);

        // New key should be retrievable
        let retrieved_key = manager.get_or_create_key().unwrap();
        assert_eq!(rotated_key, retrieved_key);

        // Clean up
        manager.delete_key().unwrap();
    }

    #[test]
    fn test_cache_clearing() {
        let manager = KeyManager::new().unwrap();

        // Clean up any existing test key
        let _ = manager.delete_key();

        // Get key (should be cached)
        let _key = manager.get_or_create_key().unwrap();

        // Clear cache
        manager.clear_cache();

        // Should still be able to retrieve from keychain
        let key2 = manager.get_or_create_key().unwrap();
        assert_eq!(key2.len(), KEY_LENGTH);

        // Clean up
        manager.delete_key().unwrap();
    }
}
