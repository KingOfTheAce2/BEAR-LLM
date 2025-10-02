// SPDX-License-Identifier: MIT
// Copyright (c) 2025 BEAR AI LLM
//
// Chat Message Encryption
// GDPR Article 32 - Security of Processing
//
// This module provides application-level encryption for chat messages
// using AES-256-GCM for authenticated encryption. This is an additional
// security layer on top of SQLCipher database encryption.
//
// Messages are encrypted BEFORE being stored in the database to ensure
// maximum security for sensitive legal conversations.

use anyhow::{Context, Result};
use ring::aead::{
    Aad, BoundKey, Nonce, NonceSequence, OpeningKey, SealingKey, UnboundKey, AES_256_GCM,
};
use ring::error::Unspecified;
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
// NonZeroU32 removed - no longer used in this module
use zeroize::Zeroize;

/// Encrypted message structure stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMessage {
    /// Encrypted content (ciphertext)
    pub ciphertext: Vec<u8>,
    /// Nonce/IV used for encryption (12 bytes for GCM)
    pub nonce: Vec<u8>,
    /// Authentication tag (included in ciphertext by ring)
    /// Encryption version for key rotation support
    pub version: u32,
    /// User ID for key derivation
    pub user_id: String,
}

/// Chat message encryptor using AES-256-GCM
pub struct ChatEncryptor {
    rng: SystemRandom,
}

impl ChatEncryptor {
    /// Create a new chat encryptor
    pub fn new() -> Self {
        Self {
            rng: SystemRandom::new(),
        }
    }

    /// Encrypt a chat message
    ///
    /// # Arguments
    /// * `plaintext` - The message content to encrypt
    /// * `user_key` - The user-specific encryption key (32 bytes for AES-256)
    /// * `user_id` - User identifier for key derivation tracking
    ///
    /// # Returns
    /// An `EncryptedMessage` containing ciphertext, nonce, and metadata
    pub fn encrypt(
        &self,
        plaintext: &str,
        user_key: &[u8],
        user_id: &str,
    ) -> Result<EncryptedMessage> {
        if user_key.len() != 32 {
            anyhow::bail!("Invalid key length: expected 32 bytes for AES-256");
        }

        // Generate random nonce (12 bytes for GCM)
        let mut nonce_bytes = [0u8; 12];
        self.rng
            .fill(&mut nonce_bytes)
            .map_err(|_| anyhow::anyhow!("Failed to generate random nonce"))?;

        // Create unbound key
        let unbound_key = UnboundKey::new(&AES_256_GCM, user_key)
            .map_err(|_| anyhow::anyhow!("Failed to create encryption key"))?;

        // Create sealing key with nonce
        let nonce_sequence = CounterNonceSequence::new(nonce_bytes);
        let mut sealing_key = SealingKey::new(unbound_key, nonce_sequence);

        // Prepare data to encrypt
        let mut in_out = plaintext.as_bytes().to_vec();

        // Encrypt in place (appends authentication tag)
        sealing_key
            .seal_in_place_append_tag(Aad::empty(), &mut in_out)
            .map_err(|_| anyhow::anyhow!("Encryption failed"))?;

        Ok(EncryptedMessage {
            ciphertext: in_out,
            nonce: nonce_bytes.to_vec(),
            version: 1, // Current encryption version
            user_id: user_id.to_string(),
        })
    }

    /// Decrypt a chat message
    ///
    /// # Arguments
    /// * `encrypted_message` - The encrypted message to decrypt
    /// * `user_key` - The user-specific decryption key (32 bytes for AES-256)
    ///
    /// # Returns
    /// The decrypted plaintext message
    pub fn decrypt(&self, encrypted_message: &EncryptedMessage, user_key: &[u8]) -> Result<String> {
        if user_key.len() != 32 {
            anyhow::bail!("Invalid key length: expected 32 bytes for AES-256");
        }

        if encrypted_message.nonce.len() != 12 {
            anyhow::bail!("Invalid nonce length: expected 12 bytes for GCM");
        }

        // Create nonce from stored bytes
        let mut nonce_bytes = [0u8; 12];
        nonce_bytes.copy_from_slice(&encrypted_message.nonce);

        // Create unbound key
        let unbound_key = UnboundKey::new(&AES_256_GCM, user_key)
            .map_err(|_| anyhow::anyhow!("Failed to create decryption key"))?;

        // Create opening key with nonce
        let nonce_sequence = CounterNonceSequence::new(nonce_bytes);
        let mut opening_key = OpeningKey::new(unbound_key, nonce_sequence);

        // Prepare data to decrypt (must be mutable)
        let mut in_out = encrypted_message.ciphertext.clone();

        // Decrypt in place (verifies authentication tag)
        let decrypted = opening_key
            .open_in_place(Aad::empty(), &mut in_out)
            .map_err(|_| anyhow::anyhow!("Decryption failed - invalid key or corrupted data"))?;

        // Convert decrypted bytes to string
        String::from_utf8(decrypted.to_vec()).context("Decrypted data is not valid UTF-8")
    }

    /// Encrypt a message and serialize to JSON
    pub fn encrypt_to_json(
        &self,
        plaintext: &str,
        user_key: &[u8],
        user_id: &str,
    ) -> Result<String> {
        let encrypted = self.encrypt(plaintext, user_key, user_id)?;
        serde_json::to_string(&encrypted).context("Failed to serialize encrypted message")
    }

    /// Decrypt a message from JSON
    pub fn decrypt_from_json(&self, json: &str, user_key: &[u8]) -> Result<String> {
        let encrypted: EncryptedMessage =
            serde_json::from_str(json).context("Failed to deserialize encrypted message")?;
        self.decrypt(&encrypted, user_key)
    }
}

impl Default for ChatEncryptor {
    fn default() -> Self {
        Self::new()
    }
}

/// Counter-based nonce sequence for AEAD operations
struct CounterNonceSequence {
    nonce: [u8; 12],
    used: bool,
}

impl CounterNonceSequence {
    fn new(nonce: [u8; 12]) -> Self {
        Self { nonce, used: false }
    }
}

impl NonceSequence for CounterNonceSequence {
    fn advance(&mut self) -> Result<Nonce, Unspecified> {
        if self.used {
            return Err(Unspecified);
        }
        self.used = true;
        Nonce::try_assume_unique_for_key(&self.nonce)
    }
}

/// Per-user key derivation from master key
pub struct UserKeyDerivation {
    master_key: Vec<u8>,
}

impl UserKeyDerivation {
    /// Create a new key derivation instance
    ///
    /// # Arguments
    /// * `master_key` - The master encryption key from KeyManager (32 bytes)
    pub fn new(master_key: Vec<u8>) -> Result<Self> {
        if master_key.len() != 32 {
            anyhow::bail!("Invalid master key length: expected 32 bytes");
        }
        Ok(Self { master_key })
    }

    /// Derive a user-specific key using Argon2id
    ///
    /// # Arguments
    /// * `user_id` - Unique user identifier
    /// * `password` - User password or PIN (optional, can be empty for default)
    ///
    /// # Returns
    /// A 32-byte user-specific encryption key
    pub fn derive_user_key(&self, user_id: &str, password: &str) -> Result<Vec<u8>> {
        use argon2::{Argon2, ParamsBuilder, Version};

        // CRITICAL SECURITY WARNING: Using predictable salt is cryptographically weak
        //
        // TODO: Implement proper random salt storage
        // Required changes:
        // 1. Add user_encryption_keys table with columns: user_id, salt (BLOB), created_at, version
        // 2. Generate cryptographically random 32-byte salt using ring::rand::SystemRandom
        // 3. Store salt in database on first key derivation
        // 4. Retrieve stored salt for subsequent derivations
        // 5. Implement salt rotation mechanism for key rotation
        //
        // Current implementation uses user_id as salt - NOT PRODUCTION READY
        // This allows rainbow table attacks and violates GDPR Article 32
        let salt = format!("bear-ai-chat-{}", user_id);
        let salt_bytes = salt.as_bytes();

        // Combine master key and password for input
        let mut input = self.master_key.clone();
        if !password.is_empty() {
            input.extend_from_slice(password.as_bytes());
        }

        // Configure Argon2id parameters
        // Using moderate parameters for balance between security and performance
        let params = ParamsBuilder::new()
            .m_cost(65536) // 64 MB memory
            .t_cost(3) // 3 iterations
            .p_cost(4) // 4 parallel threads
            .output_len(32) // 256-bit output
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to build Argon2 parameters: {}", e))?;

        let argon2 = Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params);

        let mut output_key = vec![0u8; 32];
        argon2
            .hash_password_into(&input, salt_bytes, &mut output_key)
            .map_err(|e| anyhow::anyhow!("Key derivation failed: {}", e))?;

        // Zeroize sensitive data
        input.zeroize();

        Ok(output_key)
    }

    /// Derive key with empty password (default)
    pub fn derive_default_key(&self, user_id: &str) -> Result<Vec<u8>> {
        self.derive_user_key(user_id, "")
    }
}

impl Drop for UserKeyDerivation {
    fn drop(&mut self) {
        // Zeroize master key on drop
        self.master_key.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let encryptor = ChatEncryptor::new();
        let user_key = b"12345678901234567890123456789012"; // 32 bytes
        let plaintext = "This is a sensitive legal conversation about Case #12345";

        let encrypted = encryptor.encrypt(plaintext, user_key, "user123").unwrap();
        let decrypted = encryptor.decrypt(&encrypted, user_key).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_encrypted_message_structure() {
        let encryptor = ChatEncryptor::new();
        let user_key = b"12345678901234567890123456789012";
        let plaintext = "Test message";

        let encrypted = encryptor.encrypt(plaintext, user_key, "user123").unwrap();

        assert_eq!(encrypted.nonce.len(), 12);
        assert_eq!(encrypted.version, 1);
        assert_eq!(encrypted.user_id, "user123");
        assert!(encrypted.ciphertext.len() > plaintext.len()); // Includes auth tag
    }

    #[test]
    fn test_wrong_key_fails() {
        let encryptor = ChatEncryptor::new();
        let correct_key = b"12345678901234567890123456789012";
        let wrong_key = b"99999999999999999999999999999999";
        let plaintext = "Secret message";

        let encrypted = encryptor
            .encrypt(plaintext, correct_key, "user123")
            .unwrap();

        // Decryption with wrong key should fail
        let result = encryptor.decrypt(&encrypted, wrong_key);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Decryption failed"));
    }

    #[test]
    fn test_tampered_ciphertext_fails() {
        let encryptor = ChatEncryptor::new();
        let user_key = b"12345678901234567890123456789012";
        let plaintext = "Important message";

        let mut encrypted = encryptor.encrypt(plaintext, user_key, "user123").unwrap();

        // Tamper with ciphertext
        if !encrypted.ciphertext.is_empty() {
            encrypted.ciphertext[0] ^= 0xFF;
        }

        // Decryption should fail due to authentication tag mismatch
        let result = encryptor.decrypt(&encrypted, user_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_json_serialization() {
        let encryptor = ChatEncryptor::new();
        let user_key = b"12345678901234567890123456789012";
        let plaintext = "JSON test message";

        let json = encryptor
            .encrypt_to_json(plaintext, user_key, "user123")
            .unwrap();
        let decrypted = encryptor.decrypt_from_json(&json, user_key).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_user_key_derivation() {
        let master_key = vec![1u8; 32];
        let derivation = UserKeyDerivation::new(master_key).unwrap();

        let key1 = derivation.derive_user_key("user1", "password123").unwrap();
        let key2 = derivation.derive_user_key("user2", "password123").unwrap();
        let key3 = derivation.derive_user_key("user1", "password123").unwrap();

        // Different users should have different keys
        assert_ne!(key1, key2);
        // Same user and password should produce same key
        assert_eq!(key1, key3);
        // All keys should be 32 bytes
        assert_eq!(key1.len(), 32);
        assert_eq!(key2.len(), 32);
    }

    #[test]
    fn test_default_key_derivation() {
        let master_key = vec![1u8; 32];
        let derivation = UserKeyDerivation::new(master_key).unwrap();

        let key1 = derivation.derive_default_key("user1").unwrap();
        let key2 = derivation.derive_default_key("user2").unwrap();

        // Different users should have different keys even with default (empty) password
        assert_ne!(key1, key2);
        assert_eq!(key1.len(), 32);
    }

    #[test]
    fn test_multiple_messages_different_nonces() {
        let encryptor = ChatEncryptor::new();
        let user_key = b"12345678901234567890123456789012";
        let plaintext = "Same message";

        let encrypted1 = encryptor.encrypt(plaintext, user_key, "user123").unwrap();
        let encrypted2 = encryptor.encrypt(plaintext, user_key, "user123").unwrap();

        // Same plaintext should produce different ciphertexts due to random nonces
        assert_ne!(encrypted1.nonce, encrypted2.nonce);
        assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);

        // Both should decrypt correctly
        assert_eq!(encryptor.decrypt(&encrypted1, user_key).unwrap(), plaintext);
        assert_eq!(encryptor.decrypt(&encrypted2, user_key).unwrap(), plaintext);
    }

    #[test]
    fn test_empty_message() {
        let encryptor = ChatEncryptor::new();
        let user_key = b"12345678901234567890123456789012";
        let plaintext = "";

        let encrypted = encryptor.encrypt(plaintext, user_key, "user123").unwrap();
        let decrypted = encryptor.decrypt(&encrypted, user_key).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_unicode_message() {
        let encryptor = ChatEncryptor::new();
        let user_key = b"12345678901234567890123456789012";
        let plaintext = "Legal case 法律案件 дело 🏛️⚖️";

        let encrypted = encryptor.encrypt(plaintext, user_key, "user123").unwrap();
        let decrypted = encryptor.decrypt(&encrypted, user_key).unwrap();

        assert_eq!(plaintext, decrypted);
    }
}
