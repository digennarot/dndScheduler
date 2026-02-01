---
stepsCompleted: [step-01-init, step-02-context, step-03-starter, step-04-decisions, step-05-patterns, step-06-structure, step-07-validation, step-08-complete]
workflowType: 'architecture'
lastStep: 8
status: 'complete'
completedAt: '2026-02-05'
inputDocuments:
  - /home/tiziano/Scaricati/OKComputer_Rust D&D Scheduler App/_bmad-output/planning-artifacts/prd.md
  - /home/tiziano/Scaricati/OKComputer_Rust D&D Scheduler App/_bmad-output/planning-artifacts/product-brief-OKComputer_Rust D&D Scheduler App-2026-02-05.md
  - /home/tiziano/Scaricati/OKComputer_Rust D&D Scheduler App/_bmad-output/planning-artifacts/research/technical-LettuceMeet-Calendar-research-2026-02-05.md
  - /home/tiziano/Scaricati/OKComputer_Rust D&D Scheduler App/docs/project-overview.md
workflowType: 'architecture'
project_name: 'OKComputer_Rust D&D Scheduler App'
user_name: 'Tiziano'
date: '2026-02-05'
---

# Architecture Decision Document

_This document builds collaboratively through step-by-step discovery. Sections are appended as we work through each architectural decision together._

## Project Context Analysis

### Requirements Overview
**Functional Requirements:**
20 FRs focusing on a rigid "Poll -> Vote -> Finalize" workflow.
*   **Critical Architectural Driver**: The "No Login" vote requiring robust session tracking without a `users` table relationship.

**Non-Functional Requirements:**
*   **Performance**: < 50ms interaction response (Requires client-side state management).
*   **Deployment**: Zero-Config, Single Binary (Requires strict asset embedding).

**Scale & Complexity:**
*   Primary domain: Web Application (Rust/Axum + Vanilla JS)
*   Complexity level: Low-Medium
*   Estimated architectural components: ~6 (PollService, VoteService, AnonSessionManager, StaticHandler, SchedulerDb, FrontendStore)

### Technical Constraints & Dependencies
*   **SQLite-Only**: No Redis for session caching (Must use in-memory `DashMap` or DB).
*   **No External CDNs**: All JS/CSS must be bundled.
*   **Mobile Safari**: Pointer events must support touch-drag.

### Cross-Cutting Concerns Identified
*   **Anonymous Session Management**: Tracking users without accounts.
*   **Rate Limiting**: IP-based protection for writing to the DB.
*   **Data Pruning**: Auto-cleanup of expired polls.

## Starter Template Evaluation

### Primary Technology Domain
**Web Application (Backend-Driven)**: Rust/Axum serving HTML/JSON + Vanilla JS Client.

### Selected Starter: Custom Axum Modular Monolith

**Rationale for Selection:**
Standard CLI starters (like `create-t3-app`) enforce frontend frameworks (React/Next.js) which violate our "Single Binary / Zero Build Step" goal. A custom structure allows us to optimize for the specific "Self-Hosted" NFRs.

**Initialization Command:**

```bash
# Manual setup required for custom structure
cargo new dnd_scheduler
cd dnd_scheduler
cargo add axum tokio tower-http sqlx serde serde_json tracing tracing-subscriber rust-embed
mkdir -p src/{api,core,db,security} static/{js,css,img}
```

**Architectural Decisions Provided by Starter:**

**Language & Runtime:**
*   **Language**: Rust (2026 Edition)
*   **Runtime**: Tokio (Multi-threaded async)

**Styling Solution:**
*   **CSS**: Vanilla CSS with CSS Variables (No pre-processor)

**Build Tooling:**
*   **Asset Embedding**: `rust-embed` to compile `static/` into binary
*   **Dev Server**: `cargo-watch -x run`

**Testing Framework:**
*   **Unit/Integration**: Standard `cargo test`

**Code Organization:**
*   **Layered Architecture**: `api` (Controllers) -> `core` (Business Logic) -> `db` (Data Access)
*   **Modular Monolith**: Separation by technical concern, scalable to domain modules later.

**Development Experience:**
*   **Hot Reload**: Supported via `cargo-watch`
*   **Type Safety**: Full Rust compiler guarantees + `sqlx` query verification.

**Note:** Project initialization using this command should be the first implementation story.

## Core Architectural Decisions

### Decision Priority Analysis

**Critical Decisions (Block Implementation):**
*   **Anonymous Authentication Strategy**: Switching to Server-Signed Cookies (Hardened).
*   **Data Modeling**: Normalized `participants` table.
*   **Admin Security**: Session Exchange pattern.

**Important Decisions (Shape Architecture):**
*   **Frontend State**: Optimistic UI Store.
*   **Rate Limiting**: Granular Route-Based limits.

### Data Architecture

**1. Participant Modeling**
*   **Decision**: Normalized `participants` table.
*   **Schema**: `votes` -> `participant_id` <- `poll_id`.
*   **Constraint**: **Hard Cap of 25 participants per poll** (DoS prevention).
*   **Rationale**: Enables "Edit Vote" functionality and prevents array-flooding attacks.

**2. Data Pruning**
*   **Decision**: `DELETE FROM polls WHERE created_at < now() - 30 days`.
*   **Mechanism**: Background `tokio::spawn` task running daily.

### Authentication & Security (Hardened)

**3. Anonymous Identity**
*   **Decision**: **Server-Signed HttpOnly Cookies**.
*   **Change from Initial Idea**: We rejected "Client-Generated UUIDs" based on Red Team analysis (Tampering/Spoofing risk).
*   **Mechanism**: Server generates UUID, signs it with a secret, sets `Set-Cookie`.

**4. Admin Access**
*   **Decision**: **Token Exchange Flow**.
*   **Mechanism**: Admin sends `X-Admin-Token` ONCE to `/api/admin/login`. Server validates and sets a secure `admin_session` cookie.
*   **Rationale**: Prevents Replay Attacks on LAN (HTTP) by strictly limiting the window where the raw secret is on the wire.

### API & Communication Patterns

**5. Rate Limiting**
*   **Technology**: `axum-governor` (GCRA algorithm).
*   **Policies**:
    *   Global: 60 req/min (Gentle).
    *   **Write Hardening**: `POST /poll/:id/vote` limited to 5 req/min per IP (Anti-Spam).

**6. Error Handling**
*   **Pattern**: Custom `AppError` enum implementing `IntoResponse`.
*   **Response**: JSON `{ "error": "code", "message": "human readable" }`.

### Frontend Architecture

**7. State Management**
*   **Pattern**: **Optimistic UI Store** (Vanilla JS Class).
*   **Behavior**:
    1.  User clicks -> Update DOM & Heap immediately (Green).
    2.  Async Request -> Send to API.
    3.  Error? -> Revert DOM & Heap (Red toast).

**8. Asset Delivery**
*   **Decision**: `rust-embed` with `tower-http`.
*   **Rationale**: Single Binary requirement. All assets mapped to `/assets/*` with long cache headers.

### Decision Impact Analysis

**Implementation Sequence:**
1.  **Core**: `axum` scaffolding + `rust-embed`.
2.  **Auth**: Signed Cookie middleware.
3.  **Data**: SQLite schema (`polls`, `participants`, `votes`).
4.  **API**: Poll creation and Voting endpoints (with Rate Limits).
5.  **Frontend**: JS Store and Heatmap visualization.

**Cross-Component Dependencies:**
*   **Cookie Secret**: Must be generated on first run if not provided in `.env` to ensure security.

## Implementation Patterns & Consistency Rules

### Pattern Categories Defined

**Critical Conflict Points Identified:**
3 key areas where Rust/JS paradigms clash (Case, API Structure, State).

### Naming Patterns

**Database Naming Conventions:**
*   **Tables**: `snake_case`, plural (e.g., `polls`, `participants`, `votes`).
*   **Columns**: `snake_case` (e.g., `created_at`, `poll_id`).
*   **Foreign Keys**: `singular_noun_id` (e.g., `poll_id` references `polls.id`).

**API Naming Conventions:**
*   **JSON Fields**: `camelCase` (Enforced by `#[serde(rename_all = "camelCase")]`).
    *   *Example*: Rust `created_at` -> JSON `createdAt`.
*   **Endpoints**: Standard REST `kebab-case`.
    *   `GET /api/polls/:id`
    *   `POST /api/polls/:id/vote`

**Code Naming Conventions:**
*   **Rust**: Standard `snake_case` for variables/functions, `PascalCase` for structs.
*   **JS**: `camelCase` for variables/functions, `PascalCase` for Classes (`Store`, `PollManager`).
*   **CSS Classes**: `kebab-case` (e.g., `.poll-grid`).

### Structure Patterns

**Project Organization:**
*   **Tests**: Co-located in Rust (`#[cfg(test)] mod tests { ... }`) for unit tests. Integration tests in `tests/`.
*   **Frontend**: `static/js/modules/*.js` for ES6 modules.

### Format Patterns

**API Response Formats:**
*   **Success**: Direct data object.
    *   `200 OK` -> `{ "id": "...", "title": "..." }`
*   **Error**: Standard Error Object.
    *   `4xx/5xx` -> `{ "error": "NOT_FOUND", "message": "Poll not found" }`

**Data Exchange Formats:**
*   **Dates**: ISO 8601 Strings (`2026-02-05T12:00:00Z`).
*   **Booleans**: JSON `true`/`false`.

### Communication Patterns

**State Management Patterns (Frontend):**
*   **Pattern**: **Method-Driven Store** (Observer).
*   **Rules**:
    *   State is private inside `Store` class.
    *   UI components subscribe to changes: `store.subscribe(renderer.render)`.
    *   Mutations happen via methods: `store.addVote(...)`.

### Process Patterns

**Error Handling Patterns:**
*   **Backend**: Map all errors to `AppError` enum -> convert to JSON response.
*   **Frontend**: `try/catch` around API calls. On error, `alert()` or Toast (if time permits), and **revert optimistic updates**.

### Enforcement Guidelines

**All AI Agents MUST:**
1.  **Always** add the serde rename macro to structs serializing to JSON.
2.  **Never** write business logic in the API handlers (Keep `api` thin, move to `core`).
3.  **Always** use parameterized queries in `sqlx` (No format strings!).

**Pattern Examples:**

**Good Rust Struct:**
```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePollRequest {
    pub title: String,
    pub date_range: Vec<String>, // ISO strings
}
```

**Good JS Store:**
```javascript
class Store {
    #state = { votes: [] };
    
    addVote(vote) {
        this.#state.votes.push(vote); // Optimistic
        this.notify();
        api.postVote(vote).catch(() => {
            this.#state.votes.pop(); // Revert
            this.notify();
        });
    }
}
```

## Project Structure & Boundaries

### Complete Project Directory Structure

```bash
OKComputer_Rust_DnD_Scheduler/
├── Cargo.toml            # Workspace definition / Dependencies
├── Dockerfile            # Multi-stage build for single binary
├── justfile              # Task runner (e.g., `just dev`, `just test`)
├── .env.example          # Template for secrets
├── src/
│   ├── main.rs           # Application entry point (Composition Root)
│   ├── api/              # Interface Layer (HTTP Helpers)
│   │   ├── mod.rs        # Router assembly
│   │   ├── router.rs     # Main Router loop
│   │   ├── middleware.rs # Rate Limit, Auth, Cors
│   │   └── handlers/     # Request/Response logic
│   │       ├── mod.rs
│   │       ├── admin.rs  # Admin Login
│   │       ├── poll.rs   # CRUD Polls
│   │       └── vote.rs   # Voting Logic
│   ├── core/             # Business Layer (Pure Rust)
│   │   ├── mod.rs
│   │   ├── models.rs     # Domain Structs (Poll, Vote, Participant)
│   │   └── services.rs   # Business Logic (create_poll, tally_votes)
│   ├── db/               # Infrastructure Layer (SQLx)
│   │   ├── mod.rs
│   │   ├── schema.rs     # Table definitions
│   │   └── queries/      # Encapsulated SQL
│   │       ├── poll_repo.rs
│   │       └── vote_repo.rs
│   └── security/         # Cross-Cutting Concerns
│       ├── mod.rs
│       └── token.rs      # Signed Cookie Logic
└── static/               # Frontend Assets (Vanilla JS + ES Modules)
    ├── index.html        # Main Entry (Dashboard)
    ├── create.html       # Wizard Entry
    ├── css/
    │   ├── main.css      # Variables & Reset
    │   └── components.css# Grid & Utility classes
    ├── js/
    │   ├── app.js        # Bootstrapper
    │   ├── api.js        # Fetch Wrapper & Error Handling
    │   ├── store.js      # Optimistic UI Store
    │   └── modules/
    │       ├── heatmap.js# Anime.js Grid Visualization
    │       └── wizard.js # Poll Creation Logic
    └── assets/           # Images/Fonts
```

### Architectural Boundaries

**API Boundaries (Interface Layer):**
*   **Responsibility**: JSON serialization/deserialization, HTTP Status Codes, Cookie Management.
*   **Constraint**: NO Business Logic here. Just converting HTTP <-> Domain.
*   **Endpoints**:
    *   `POST /api/polls` (Public, Rate Limited)
    *   `POST /api/polls/:id/vote` (Public, STRICT Rate Limit)
    *   `POST /api/admin/login` (Admin Token Exchange)

**Core Boundaries (Business Layer):**
*   **Responsibility**: Validation rules (e.g., "Date must be in future"), Data transformation, Tallying logic.
*   **Constraint**: NO SQL types (use Domain Structs), NO HTTP types (use generic Results).

**Infrastructure Boundaries (Data Layer):**
*   **Responsibility**: Translating Domain Structs <-> SQL Rows.
*   **Constraint**: All SQL lives here. No `query!` macros in `api/`.

### Requirements to Structure Mapping

**Epic: Poll Management**
*   **Handlers**: `src/api/handlers/poll.rs`
*   **Service**: `src/core/services.rs` (create methods)
*   **DB**: `src/db/queries/poll_repo.rs`

**Epic: Participation & Voting**
*   **Handlers**: `src/api/handlers/vote.rs`
*   **Logic**: `src/core/models.rs` (Date parsing/matching)
*   **Frontend**: `static/js/modules/heatmap.js` (Interaction)

**Cross-Cutting: Authentication**
*   **Middleware**: `src/api/middleware.rs` (Checks Cookie)
*   **Logic**: `src/security/token.rs` (Signs/Verifies Config)

### Integration Points

**Internal Communication:**
*   **API -> Core**: Direct Function Calls (In-process).
*   **Core -> DB**: Dependency Injection via Traits (or direct calls for MVP).

**Frontend Integration:**
*   **Protocol**: Strict REST via `fetch()`.

**Frontend Integration:**
*   **Protocol**: Strict REST via `fetch()`.
*   **State Sync**: Optimistic UI updates in `store.js` before API return.

## Architecture Validation Results

### Coherence Validation ✅

**Decision Compatibility:**
*   **Decisions vs Structure**: The **Modular Monolith** structure perfectly supports the "Custom Rust Backend" decision. The `static/` directory separation supports the "No Bundler" frontend decision.
*   **Patterns vs Code**: The `Result<T, AppError>` pattern in Rust is supported by the `src/api/middleware.rs` and `handler` separation. The "Optimistic UI" pattern is supported by the `store.js` module.

### Requirements Coverage Validation ✅

**Epic/Feature Coverage:**
*   **Epic: Poll Management**: Covered by `src/api/handlers/poll.rs` + `src/core/services.rs`.
*   **Epic: Voting**: Covered by `src/api/handlers/vote.rs` + `src/db/queries/vote_repo.rs`.

**Non-Functional Requirements Coverage:**
*   **NFR: Performance (<50ms)**: Covered by "Optimistic UI" pattern (immediate DOM update) + Axum's speed.
*   **NFR: Anonymity**: Covered by `src/security/token.rs` (Signed Cookies).

### Gap Analysis Results

*   **Metric Collection**: No explicit metrics strategy (e.g., Prometheus) defined, but `AppError` provides a hook. (Priority: Low - Post-MVP).
*   **CI/CD**: `README.md` and `justfile` are strictly placeholders. (Priority: Medium - Needs definition in Implementation Plan).

### Architecture Readiness Assessment

**Overall Status:** READY FOR IMPLEMENTATION

**Confidence Level:** High

**Key Strengths:**
*   Clear Separation of Concerns (Core vs API vs DB).
*   Hardened Security decisions (Signed Cookies, Admin Token Exchange).
*   Optimistic UI pattern defined for "Magical" feel.

### Implementation Handoff

**First Implementation Priority:**
Initialize the project structure:
`cargo new OKComputer_Rust_DnD_Scheduler` and setup `src/api`, `src/core`, `src/db` directories.
