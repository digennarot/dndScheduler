# Story 1.3: Date Range & Timezone Logic

Status: done

## Story

As an Anonymous DM,
I want to define specific dates and my timezone context,
so that users vote on relevant slots in their own local time.

## Acceptance Criteria

- [x] **Given** I am creating a poll
- [x] **When** I providing a date range (e.g., "2026-02-10" to "2026-02-12")
- [x] **Then** The system should validate the start date is not in the past
- [x] **And** The system should store the `date_range` as JSON text in SQLite
- [x] **And** The system should reject ranges longer than 14 days (Soft limit NFR)

## Tasks / Subtasks

- [x] Add date validation logic in `PollService::create_poll` logic to reject past dates (AC: 1, 3)
- [x] Add duration validation logic to reject date ranges exceeding 14 days (AC: 5)
- [x] Verify dates are serialized to JSON string before DB persistence (AC: 4)
- [x] Add integration tests for date validation rules in poll creation

## Dev Notes

- **Architecture Constraints**: Refer to `src/core/services/poll.rs` and `PollService::create_poll` for current implementation logic.
- **Note**: Some logic (`date < today`, `duration > 14 days`) seems to already exist in `poll.rs`. The focus for `dev-story` should be ensuring thorough test coverage and verifying all 1.3 Acceptance Criteria are 100% satisfied.

### Project Structure Notes

- Relevant files:
  - `src/core/services/poll.rs`
  - `tests/integration/test_polls.rs` (or similar, where poll creation tests belong)

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 1.3]

## Dev Agent Record

### Agent Model Used

Antigravity 

### Debug Log References

### Completion Notes List

- Verified that `PollService::create_poll` correctly validates that `date >= today` and rejects past dates (AC: 1, 3).
- Verified duration validation rejecting date ranges exceeding 14 days is correctly enforced (AC: 5).
- Verified dates are serialized to a JSON string using `serde_json::to_string(&payload.dates)` before DB persistence (AC: 4).
- Covered the above rules with comprehensive integration tests in `test_poll_dates.rs`.
- **Review Follow-up (AI)**: Fixed Timezone Context AC by adding `chrono-tz` and parsing timezone string to offset "today" properly.
- **Review Follow-up (AI)**: Fixed HTTP API error serialization to return `{"error": ...}` instead of passing unwrapped plain text.
- **Review Follow-up (AI)**: Corrected off-by-one error checking bounds (14 span, 13 duration limits) matching exactly 14 day boundaries.

### File List

- `tests/integration/test_poll_dates.rs` (Added)
- `tests/integration/main.rs` (Modified)
- `src/core/models.rs` (Modified)
- `src/core/services/poll.rs` (Modified)
- `src/api/handlers/poll.rs` (Modified)
- `Cargo.toml` (Modified)

## Change Log

- Addressed business logic validations for polls with integration tests enforcing 14-days duration maximum, past date rejection, and JSON serialization. (Date: 2026-02-23)
- Addressed code review feedback - Fixed Timezone discrepancy, implemented JSON error formatting, corrected duration limits, and included missing files. (Date: 2026-02-23)
