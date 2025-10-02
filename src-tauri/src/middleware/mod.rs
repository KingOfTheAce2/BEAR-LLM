// Middleware Module - Request Guards and Cross-Cutting Concerns
// Provides consent enforcement, rate limiting, and other middleware

pub mod consent_guard;

// Tests removed - all test infrastructure in compliance module

pub use consent_guard::{ConsentGuard, ConsentGuardBuilder};
