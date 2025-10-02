// Database module for BEAR AI
// Provides database management, export integration, and data access

pub mod chat_encryption_integration;
pub mod export_integration;

#[cfg(test)]
mod export_integration_tests;

// ChatEncryptionLayer and ExportIntegration are internal to database module
