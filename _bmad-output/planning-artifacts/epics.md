---
stepsCompleted: [step-01-validate-prerequisites, step-02-design-epics, step-03-create-stories, step-04-final-validation]
workflowType: 'epics'
lastStep: 4
status: 'complete'
completedAt: '2026-02-05'
inputDocuments:
  - /home/tiziano/Scaricati/OKComputer_Rust D&D Scheduler App/_bmad-output/planning-artifacts/prd.md
  - /home/tiziano/Scaricati/OKComputer_Rust D&D Scheduler App/_bmad-output/planning-artifacts/architecture.md
project_name: 'OKComputer_Rust D&D Scheduler App'
---

# OKComputer_Rust D&D Scheduler App - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for OKComputer_Rust D&D Scheduler App, decomposing the requirements from the PRD, UX Design if it exists, and Architecture requirements into implementable stories.

## Requirements Inventory

### Functional Requirements

FR1: Users can create a new poll without an account (Anonymous DM).
FR2: Users can define a date range for the poll using a visual calendar picker.
FR3: Users can set a poll title and optional description.
FR4: Users represent the "Timezone Authority" (Poll defaults to their TZ).
FR5: Users can delete a poll they created (via Admin Key/Token).
FR6: Users can access a poll via a shared link without login (Anonymous Player).
FR7: Users can view availability times in their auto-detected local timezone.
FR8: Users can select availability slots via "Drag-to-Paint" interaction (Desktop & Mobile).
FR9: Users can edit their previous vote (identified by LocalStorage/Session Token).
FR10: Users can toggle between "Yes" (Green) and "Maybe" (Yellow) states.
FR11: Users can view an aggregate "Heatmap" of all participant availability.
FR12: Users can hover/tap a slot to see specifically *who* voted for it.
FR13: Users can visually identify the "Best Time" (highest overlap).
FR14: System generates a unique, obscure URL for each poll.
FR15: System renders OpenGraph metadata (Title, Date Range) for Discord/WhatsApp previews.
FR16: System rate-limits poll creation and voting by IP address.
FR17: Admin users can delete specific votes from a poll (Troll mitigation).
FR18: System purges poll data automatically after a configurable expiration (default 30 days).
FR19: Users can "Finalize" a poll to lock voting.
FR20: Users can export the finalized event as an ICS file.

### NonFunctional Requirements

NFR1: The Availability Grid must render interactive states in < 50ms (responsive to touch/click).
NFR2: Creating a poll must complete in < 200ms (Database Write -> Redirect).
NFR3: No user data (votes/names) persists beyond the poll expiration date (Auto-purge).
NFR4: API must reject requests exceeding rate limits (10 req/min per IP) with 429 Too Many Requests.
NFR5: Admin "Delete" actions must be protected by a server-side Token.
NFR6: Heatmap colors must pass WCAG AA contrast ratio for "Yes" vs "No" states.
NFR7: All interactive elements must be focusable/actionable via Keyboard.
NFR8: The application binary must start with zero external configuration (defaults to `./data.db`).
NFR9: The application must function 100% offline (no external CDNs).

### Additional Requirements

- **Starter Template**: Initialize with custom Axum Modular Monolith structure (`src/api`, `src/core`, `src/db`).
- **Data Model**: Use SQLite with Normalized `participants` table and Hard Cap of 25 participants.
- **Frontend State**: Implement "Optimistic UI Store" pattern (Vanilla JS Class).
- **Authentication**: Implement Server-Signed HttpOnly Cookies for anonymous session tracking.
- **Security**: Implement Token Exchange Flow for Admin Access (`X-Admin-Token` -> Session Cookie).
- **API**: Use `camelCase` for JSON serialization (via `serde` rename).
- **Validation**: Enforce STRICT Rate Limiting on `POST /poll/:id/vote` (5 req/min).

### FR Coverage Map

### FR Coverage Map

FR1: Epic 1 - Poll Creation
FR2: Epic 1 - Date Range Selection
FR3: Epic 1 - Poll Metadata (Title/Desc)
FR4: Epic 1 - Timezone Authority
FR5: Epic 3 - Delete Poll (Admin)
FR6: Epic 2 - Access Poll (Link)
FR7: Epic 2 - Local Timezone View
FR8: Epic 2 - Drag-to-Paint Voting
FR9: Epic 3 - Edit Vote (Session)
FR10: Epic 2 - Maybe State
FR11: Epic 4 - Heatmap Visualization
FR12: Epic 4 - Voter Transparency
FR13: Epic 4 - Best Time Identification
FR14: Epic 1 - Unique URL Generation
FR15: Epic 1 - OpenGraph Metadata
FR16: Epic 1 - Creation Rate Limiting
FR17: Epic 3 - Admin Delete Vote
FR18: Epic 1 - Auto-Purging (System Job)
FR19: Epic 4 - Finalize Poll
FR20: Epic 4 - Export ICS

## Epic List

## Epic List

### Epic 1: Core Polling & Creation
**Goal:** Enable a user to create a new poll, define dates, and get a secure shareable link without friction.
**FRs covered:** FR1, FR2, FR3, FR4, FR14, FR15, FR16, FR18.
**Value:** Establishes the "Creation" funnel and the core "Poll" data entity.

### Epic 2: Anonymous Participation
**Goal:** Enable users to access a poll link and intuitively cast their availability votes without login.
**FRs covered:** FR6, FR7, FR8, FR10.

## Epic 2: Anonymous Participation

**Goal:** Enable users to access a poll link and intuitively cast their availability votes without login.

### Story 2.1: Anonymous Session Initialization

As a First-Time Visitor,
I want the server to assign me a secure identity cookie transparently,
So that I can eventually vote and be recognized if I return.

**Acceptance Criteria:**

**Given** I access any page (e.g., `/poll/:id`) without a cookie
**When** The `EnsureSession` middleware runs
**Then** It should generate a UUIDv4 signed with the Server Secret
**And** Set it as an `HttpOnly`, `SameSite=Lax` cookie named `dnd_session`
**And** Future requests should include this cookie

### Story 2.2: Poll Fetching Logic

As a Participant,
I want to see the poll details and my previous vote (if any),
So that I can understand the schedule context and update my availability.

**Acceptance Criteria:**

**Given** I have a valid session cookie
**When** I GET `/api/polls/:id`
**Then** Payload should include `title`, `description`, `dates`, `participants` (list of names)
**And** Payload should include `my_vote` (null or array of slots) derived from my cookie ID
**And** I should receive 404 if the poll does not exist

### Story 2.3: Availability Grid Component (Anime.js)

As a User,
I want to see the dates rendered as a responsive grid,
So that I can visually identify the time slots.

**Acceptance Criteria:**

**Given** Poll data with a 3-day range
**When** The `Heatmap` class renders
**Then** It should generate columns for each day and rows for hours
**And** It should automatically adjust cell size for mobile screens (CSS Grid)
**And** It should be empty/grey initially (or green for my existing slots)

### Story 2.4: Drag-to-Paint Interaction

As a Mobile User,
I want to drag my finger across slots to mark them green,
So that I can vote quickly without tapping 50 times.

**Acceptance Criteria:**

**Given** I am on the Grid
**When** I touch a slot and drag across 4 others
**Then** All 5 slots should visually toggle to "Selected" (Green)
**And** Pointer Events should handle both Mouse and Touch interfaces
**And** Scrolling should be disabled while painting (if possible, or handled gracefully)

### Story 2.5: Voting API Endpoint (Strict Rate Limit)

As a Backend System,
I want to accept votes only from valid sessions and rate-limit them heavily,
So that the results remain trustworthy.

**Acceptance Criteria:**

**Given** A POST to `/api/polls/:id/vote` with `{ "slots": [...], "name": "Dave" }`
**When** The handler processes the request
**Then** It must validate the `dnd_session` cookie signature
**And** It must upsert (Insert or Update) the vote in `votes` table
**And** It must reject if > 5 requests/min from this IP (Strict Mode)

### Story 2.6: Optimistic UI & State Manager

As an Impatient User,
I want the grid to turn green *instantly* when I touch it,
So that the app feels "Magical" and responsive.

**Acceptance Criteria:**

**Given** I drag-select network slots
**When** The `Store` receives the input
**Then** It should update the local state and trigger a render ( < 16ms)
**And** It should *then* trigger the API call in background (debounced)
**And** If the API returns 500/429, it should revert the state and show a Red Error Toast

### Epic 3: Session Management & Security
**Goal:** Enable identity persistence for editing votes and secure administrative control.
**FRs covered:** FR5, FR9, FR17.

## Epic 3: Session Management & Security

**Goal:** Enable identity persistence for editing votes and secure administrative control.

### Story 3.1: Admin Token Exchange Flow

As an Admin,
I want to log in using my secret token once to get a session cookie,
So that I don't expose my secret key on every subsequent HTTP request.

**Acceptance Criteria:**

**Given** I have the `ADMIN_TOKEN` from server env
**When** I POST `{ "token": "my-secret" }` to `/api/admin/login`
**Then** The server should validate the token
**And** Set a secure `admin_session` cookie
**And** Return 200 OK

### Story 3.2: Admin Authorization Middleware

As a System,
I want to protect sensitive routes (`DELETE`) with middleware,
So that only authenticated admins can trigger destructive actions.

**Acceptance Criteria:**

**Given** An unauthenticated user
**When** They try to DELETE a poll
**Then** They should receive `403 Forbidden`
**Given** A user with a valid `admin_session`
**When** They try the same action
**Then** The request should be allowed through to the handler

### Story 3.3: Delete APIs (Poll & Vote)

As a Moderator,
I want to delete a specific troll vote or an entire poll,
So that I can maintain the quality of the service.

**Acceptance Criteria:**

**Given** I am an Admin
**When** I DELETE `/api/polls/:id/votes/:vote_id`
**Then** That specific vote row should be removed
**When** I DELETE `/api/polls/:id`
**Then** The poll and ALL its votes should be cascaded deleted

### Story 3.4: "Edit My Vote" Logic

As a User who made a mistake,
I want to update my availability map,
So that I don't give the host wrong information.

**Acceptance Criteria:**

**Given** I have already voted and possess a `dnd_session` cookie
**When** I POST to `/api/polls/:id/vote` again
**Then** The system should identify me by my session ID
**And** UPDATE my existing rows instead of inserting new ones
**And** The UI should reflect my updated choices

### Story 3.5: Admin Dashboard UI

As an Admin,
I want a simple page to view raw data and delete buttons,
So that I don't have to use `curl` to manage polls.

**Acceptance Criteria:**

**Given** I am logged in as Admin
**When** I visit `/manage/:id`
**Then** I should see a list of all participants
**And** Each participant should have a "Delete" button next to their name

### Epic 4: Visualization & Finalization
**Goal:** Provide the aggregated "Heatmap" view and tools to close the loop (Finalize/Export).
**FRs covered:** FR11, FR12, FR13, FR19, FR20.

## Epic 1: Core Polling & Creation

**Goal:** Enable a user to create a new poll, define dates, and get a secure shareable link without friction.

### Story 1.1: Project Skeleton & Database Architecture

As a System,
I want a verified Rust/Axum project structure with SQLite integration,
So that I have a solid foundation for building features without circular dependencies.

**Acceptance Criteria:**

**Given** A clean environment
**When** I run `cargo new` and initialize the folder structure
**Then** I should see `src/api`, `src/core`, `src/db`, `src/security` folders
**And** The application binary should compile relative to `src/main.rs`
**And** The SQLite database should initialize automatically using `sqlx` inline migrations if it doesn't exist

### Story 1.2: Poll Creation API Endpoint

As an Anonymous DM,
I want to POST configuration to `/api/polls`,
So that I can persist a new poll and receive a unique ID.

**Acceptance Criteria:**

**Given** The generic application server is running
**When** I POST `{ "title": "Game Night", "timezone": "UTC" }` to `/api/polls`
**Then** It should return a 201 Created with `{ "id": "nano_id_string", "adminToken": "..." }`
**And** The poll should be visible in the SQLite `polls` table
**And** The ID should be a URL-safe NanoID (10-12 chars)

### Story 1.3: Date Range & Timezone Logic

As an Anonymous DM,
I want to define specific dates and my timezone context,
So that users vote on relevant slots in their own local time.

**Acceptance Criteria:**

**Given** I am creating a poll
**When** I providing a date range (e.g., "2026-02-10" to "2026-02-12")
**Then** The system should validate the start date is not in the past
**And** The system should store the `date_range` as JSON text in SQLite
**And** The system should reject ranges longer than 14 days (Soft limit NFR)

### Story 1.4: Rate Limiting Middleware

As a System Administrator,
I want to limit poll creation to 10 requests per minute per IP,
So that I can prevent malicious actors from flooding the database.

**Acceptance Criteria:**

**Given** A malicious script sending requests
**When** It exceeds 10 POST requests to `/api/polls` in 1 minute
**Then** The 11th request should receive a `429 Too Many Requests` response
**And** The application log should record a warning event

### Story 1.5: Poll Wizard UI (Frontend)

As an Anonymous DM,
I want a simple visible wizard page (`create.html`),
So that I can input my poll details without using a command line.

**Acceptance Criteria:**

**Given** I navigate to `/create`
**When** I fill in "Title", select dates, and click "Create"
**Then** The page should Fetch POST to the API
**And** On success, redirect to the `/poll/:id` management view
**And** Show a toast notification if the API fails

### Story 1.6: OpenGraph Metadata Injection

As a Discord User,
I want the shared link to show the Poll Title and Dates in the chat preview,
So that my players know exactly what they are clicking.

**Acceptance Criteria:**

**Given** A finalized Poll ID
**When** A crawler (Discordbot/WhatsApp) requests `GET /poll/:id`
**Then** The HTML `<head>` should contain `og:title` matching the Poll Title
**And** `og:description` should contain the Date Range
**And** The response should be server-rendered (or injected) before JS loads

### Story 1.7: Background Cleanup Job

As a Privacy-Conscious System,
I want to auto-delete polls that haven't been modified in 30 days,
So that I don't hoard user data indefinitely.

**Acceptance Criteria:**

**Given** A poll created 31 days ago
**When** The daily background maintenance task runs
**Then** The poll and all associated votes should be hard-deleted from SQLite
**And** An audit log entry should record "Pruned N expired polls"
