# Story: Integration Test Refactoring to Redb & TestServer

## Status
- **Status**: done
- **Priority**: High
- **Owner**: Tiziano_di_gennaro

## Context
The project is transitioning from SQLx/SQLite to Redb for better performance and simpler deployment (embedded DB). The integration tests need to be refactored to use `axum_test::TestServer` and the new Redb-based `DbPool`.

## Acceptance Criteria
1. [x] All integration tests in `tests/integration/` should use `axum_test::TestServer`.
2. [x] Redb should be used for the test database instead of SQLx.
3. [x] `helpers.rs` should provide a clean `setup_test_app()` function that returns a `TestServer` compatible router.
4. [x] Anonymous user flows (joining, voting) should be correctly tested using cookies and session tokens.
5. [x] RBAC (Role-Based Access Control) tests should verify that roles are correctly enforced.
6. [x] Compilation warnings related to unused imports or deprecated patterns should be resolved.
7. [x] Test database files should be cleaned up or managed to avoid disk clutter (using unique names).

## Tasks
- [x] Refactor `helpers.rs` to use Redb and `axum_test`.
- [x] Refactor `test_anonymous.rs` to use `TestServer`.
- [x] Refactor `rbac_tests.rs` to use `TestServer`.
- [x] Ensure proper cookie handling in anonymous tests.
- [x] Fix failing tests in `test_anonymous.rs`.
- [x] Address compilation warnings and unused variables.
- [x] Implement automatic cleanup of test DB files using `tempfile`.

## Dev Agent Record
### File List
- `tests/integration/helpers.rs`
- `tests/integration/test_anonymous.rs`
- `tests/integration/rbac_tests.rs`
- `tests/integration/admin_polls_tests.rs`
- `tests/integration/auth_tests.rs`
- `tests/integration/authelia_tests.rs`
- `tests/integration/test_availability.rs`
- `tests/integration/test_finalization.rs`
- `tests/event_store_tests.rs`
- `src/db/queries/poll_repo.rs`
- `src/api/handlers/poll.rs`
- `src/api/handlers/general.rs`
- `src/api/handlers/activity.rs`
- `src/api/handlers/export.rs`
- `src/security/auth.rs`
- `src/db/mod.rs`
- `src/lib.rs`

### Completion Notes

- **Review Follow-up [High]**: Fixed `create_test_poll` in `helpers.rs` — hardcoded past date `2023-11-01` replaced with dynamic `Utc::now() + 7 days`.
- **Review Follow-up [Medium]**: `test_admin_login_flow` — env var now set *before* creating the app, with `unsafe` block documented for future awareness of thread safety concerns.
- **Review Follow-up [Medium]**: `test_availability_rate_limiting` — test now explicitly skips when `DND_DISABLE_RATE_LIMIT` is set, avoiding silent false-positive pass; real assertion added.
- **Review Follow-up [Low]**: Removed `test_authelia_header_parsing_remotes` from integration suite — identical coverage already exists in `src/security/authelia.rs` unit tests.

## Change Log

- Code review pass: fixed past-date helper, env mutation race, silent rate-limit skip, and redundant integration test. 43/43 tests pass. (Date: 2026-02-23)
