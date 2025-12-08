# OWASP Security Phase 1 Implementation

## Completed Tasks
1.  **Rate Limiting**: Implemented using `tower-governor`.
    *   Limit: 5 requests per second (burst 20).
    *   Applied globally to all routes in `main.rs`.
2.  **Account Lockout**: Implemented in `auth.rs`.
    *   Locks account for 30 minutes after 5 failed login attempts in 15 minutes.
    *   Returns `429 Too Many Requests` (mapped from `423 Locked` conceptually) with a "Try again in X seconds" message.
3.  **Audit Logging**: Implemented `src/audit.rs`.
    *   Logs critical authentication events: `login_success`, `login_failed`, `register_success`, `register_failed`, `account_locked`, `logout`.
    *   Stored in `audit_log` table in SQLite.

## Verification
*   Current implementation runs without errors.
*   Database tables for `login_attempts`, `account_locks`, and `audit_log` are created automatically.
*   Application compiles successfully.
