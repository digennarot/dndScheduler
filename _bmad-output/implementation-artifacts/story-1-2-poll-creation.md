# Story 1.2: Poll Creation API Endpoint

**Epic:** 1: Core Polling & Creation
**Status:** Done
**Epic Link:** [Epics](../planning-artifacts/epics.md)

## Description
As an Anonymous DM,
I want to POST configuration to `/api/polls`,
So that I can persist a new poll and receive a unique ID.

## Acceptance Criteria
- [x] **Given** The generic application server is running
- [x] **When** I POST `{ "title": "Game Night", "timezone": "UTC" }` to `/api/polls`
- [x] **Then** It should return a 201 Created with `{ "id": "nano_id_string", "adminToken": "..." }`
- [x] **And** The poll should be visible in the SQLite `polls` table
- [x] **And** The ID should be a URL-safe NanoID (10-12 chars)

## Tasks
- [x] Initialize Poll Structs & Models <!-- id: 0 -->
- [x] Create API Handler `create_poll` <!-- id: 1 -->
- [x] Implement Service Logic for Poll Creation <!-- id: 2 -->
- [x] Integrate SQLite Repository for Polls <!-- id: 3 -->
- [x] Add Route to Router <!-- id: 4 -->

## Dev Agent Record
- **Files Changed:**
    - `src/api/handlers/poll.rs`
    - `src/core/models.rs`
    - `src/core/services.rs`
    - `src/db/queries/poll_repo.rs`
    - `src/lib.rs`
    - `src/api/handlers/mod.rs`
    - `src/db/queries/mod.rs`
    - `Cargo.toml`
