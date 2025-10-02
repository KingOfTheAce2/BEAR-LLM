// Database module for BEAR AI
// Provides database management, export integration, and data access

pub mod chat_encryption_integration;
pub mod export_integration;

#[cfg(test)]
mod tests;

pub use chat_encryption_integration::ChatEncryptionLayer;
pub use export_integration::ExportIntegration;
