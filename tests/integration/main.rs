// Main Integration Tests Entry Point

// Moduli helper
mod helpers;

// Test modules
mod admin_polls_tests;
mod auth_tests;
mod authelia_tests;
mod email_tests;
mod rbac_tests;
mod test_anonymous;
mod test_availability;

// Re-export helper functions for use in test modules
pub use helpers::*;
