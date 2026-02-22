// Main Integration Tests Entry Point

// Moduli helper
pub mod helpers;

// Test modules using Redb
mod test_finalization;

// The following tests are tightly coupled to SQLite and sqlx, disabled temporarily.
mod admin_polls_tests;
mod auth_tests;
mod authelia_tests;
mod email_tests;
mod rbac_tests;
mod test_availability;
mod test_anonymous;
mod test_poll_dates;

// Re-export helper functions for use in test modules
pub use helpers::*;
