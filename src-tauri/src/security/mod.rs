// SPDX-License-Identifier: MIT
// Copyright (c) 2025 BEAR AI LLM
//
// Security Module
// GDPR Article 32 - Security of Processing
//
// This module provides comprehensive security features including:
// - Database encryption at rest using SQLCipher
// - Secure key management using OS keychain
// - Encrypted connection pooling
// - Migration utilities

pub mod chat_encryption;
pub mod database_encryption;
pub mod encrypted_pool;
pub mod key_manager;
pub mod migration;

// Tests removed - all test infrastructure in compliance module

// Re-export commonly used types
pub use chat_encryption::{ChatEncryptor, EncryptedMessage, UserKeyDerivation};
pub use database_encryption::{DatabaseMigration, EncryptedDatabase, EncryptionConfig, HmacAlgorithm};
pub use encrypted_pool::{EncryptedPool, EncryptedPoolBuilder};
pub use key_manager::KeyManager;
pub use migration::{ChatMigrationManager, MigrationStats, ProgressCallback};
