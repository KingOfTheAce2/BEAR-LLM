// Middleware Module - Request Guards and Cross-Cutting Concerns
// Provides consent enforcement, rate limiting, and other middleware

pub mod consent_guard;

#[cfg(test)]
mod tests;

pub use consent_guard::{ConsentGuard, ConsentGuardBuilder};
