---
stepsCompleted: [1, 2, 3, 4, 5]
inputDocuments: []
workflowType: 'research'
lastStep: 5
research_type: 'technical'
research_topic: 'LettuceMeet-style Calendar Implementation'
research_goals: 'Identify technical requirements, architectural patterns, and libraries for implementing a high-performance, interactive availability heat map similar to LettuceMeet using Vanilla JS and Rust.'
user_name: 'Tiziano'
date: '2026-02-05'
web_research_enabled: true
source_verification: true
---

# Research Report: technical

**Date:** 2026-02-05
**Author:** Tiziano
**Research Type:** technical

---

## Research Overview

This document outlines the technical strategy for implementing a "LettuceMeet-style" collaborative availability calendar within the D&D Scheduler application. It covers tech stack selection, system architecture, integration patterns, and a phased implementation roadmap.

---

## Technical Research Scope Confirmation

**Research Topic:** LettuceMeet-style Calendar Implementation
**Research Goals:** Identify technical requirements, architectural patterns, and libraries for implementing a high-performance, interactive availability heat map similar to LettuceMeet using Vanilla JS and Rust.

**Technical Research Scope:**

- Architecture Analysis - design patterns, frameworks, system architecture
- Implementation Approaches - development methodologies, coding patterns
- Technology Stack - languages, frameworks, tools, platforms
- Integration Patterns - APIs, protocols, interoperability
- Performance Considerations - scalability, optimization, patterns

**Research Methodology:**

- Current web data with rigorous source verification
- Multi-source validation for critical technical claims
- Confidence level framework for uncertain information
- Comprehensive technical coverage with architecture-specific insights

**Scope Confirmed:** 2026-02-05

---

## Technology Stack Analysis

### Programming Languages

*   **Frontend**: **Vanilla JavaScript (ES6+)**.
    *   *Rationale*: The user's goal is a "mystical" and highly customized experience. Frameworks (React/Vue) might impose rigid DOM structures that conflict with granular `Anime.js` animations. Vanilla JS offers direct DOM control for high-performance grid interactions (hover, drag-select) and seamless integration with the existing `static/js/app.js` architecture.
    *   *Performance*: Direct DOM manipulation via `PointerEvents` is optimal for a 24x7 grid (168+ cells).
*   **Backend**: **Rust (Axum)**.
    *   *Rationale*: Continues the existing high-performance backend. Rust's strict type system ensures safe handling of time zones and availability intervals.

### Development Frameworks and Libraries

*   **Animation & Visuals**:
    *   **[Anime.js](https://animejs.com/)**: **CONFIRMED**. Essential for the "mystical" feel. Staggering effects (`anime.stagger`) will be used for revealing grid cells and "heat" levels.
    *   **SVG Masking**: Use SVG masks for organic/magical reveal effects of the grid, rather than standard CSS boxes.
*   **Interaction**:
    *   **Custom Vanilla JS (Pointer Events)**: Recommended over heavy data-grid libraries (like AG Grid) or drag-drop libraries (Draggable.js). A custom "drag-to-select" implementation (utilizing `mousedown`, `mousemove`, `mouseup`) provides the specific behavior needed for a calendar (toggle selection, click-drag painting).
    *   *Alternative*: **[Interact.js](https://interactjs.io/)** could be used for the lower-level drag mechanics if custom implementation becomes edge-case heavy (touch support, etc.), but likely overkill for a strict grid range.
*   **Date/Time Handling**:
    *   **Frontend**: **`Intl.DateTimeFormat`** (Native) or **[Day.js](https://day.js.org/)** (Lightweight, 2KB). Day.js is recommended for easier manipulation of time slots and timezone conversions compared to the verbose native API.
    *   **Backend**: **`chrono`** crate. The standard for Rust date/time.

### Database and Storage Technologies (SQLite)

*   **Availability Storage**:
    *   **Normalized Table (Recommended)**: A flexible schema using `start_time` and `end_time` (Unix timestamps or ISO8601 strings in UTC).
        ```sql
        CREATE TABLE availability (
            id INTEGER PRIMARY KEY,
            user_id INTEGER NOT NULL,
            poll_id INTEGER NOT NULL,
            start_time INTEGER NOT NULL, -- Unix Timestamp (UTC)
            end_time INTEGER NOT NULL,   -- Unix Timestamp (UTC)
            FOREIGN KEY(user_id) REFERENCES users(id)
        );
        ```
    *   **Bitmasking (Alternative)**: Storing availability as a `BLOB` or `INTEGER` bitmask (e.g., 1 bit per 15-min slot).
        *   *Pros*: Extremely compact storage.
        *   *Cons*: Complex to query specific overlaps in SQL; requires application-layer decoding. Harder to handle timezone shifts if the "grid" aligns differently for users.
    *   **Decision**: **Normalized Table**. D&D sessions are distinct events, not infinite recurring patterns. Querying "Who is free between X and Y?" is standard SQL range interaction (`start < ? AND end > ?`), which allows `sqlx` to shine.

### Development Tools and Platforms

*   **Canvas vs DOM**:
    *   For the "Heatmap": **DOM Elements (`div` cells)** are preferred over `<canvas>`.
    *   *Reasoning*: Accessibility (screen readers), easier styling with CSS/Tailwind, and simpler integration with `Anime.js` targeting. `<canvas>` is faster for 10,000+ points, but a weekly grid (even with 15-min slots) is <1000 nodes, which the DOM handles fine.

### Technology Adoption Trends

*   **"Micro-interactions"**: heavily trending in specialized UX. Moving away from generic Material/Bootstrap date pickers to custom, fluid interfaces (like LettuceMeet or Calendly).
*   **Server-Side Rendering (SSR) vs SPAs**: The current architecture is "Hybrid" (Rust serves HTML, JS hydrates it). We will stick to this. The Scheduler grid will be a "Client-Side Island" initialized by `app.js`.

**Analysis of LettuceMeet's Tech**:
*   LettuceMeet likely uses React/Next.js. We are emulating this interactivity with Vanilla JS + Rust, which requires recreating the *state management* (selected slots) manually in JS variables (`Set<Timestamp>`).

---

## Integration Patterns Analysis

### API Design Patterns & Data Formats

*   **Endpoint Structure**: `RESTful` design is standard and sufficient.
    *   `GET /api/poll/{id}`: Returns poll details + current user's availability.
    *   `POST /api/poll/{id}/availability`: Upsert user's availability for this poll.
*   **JSON Data Structure**:
    *   *Request/Response*: Use a normalized array of time slots.
        ```json
        {
          "poll_id": "uuid",
          "user_id": "uuid",
          "availability": [
            { "start": "2026-02-05T10:00:00Z", "end": "2026-02-05T12:00:00Z" },
            { "start": "2026-02-05T14:00:00Z", "end": "2026-02-05T16:00:00Z" }
          ]
        }
        ```
    *   *Validation*: Backend (Axum) must validate that `start < end` and slots do not overlap (or merge them automatically).

### System Interoperability (Frontend <-> Backend)

*   **Optimistic UI Updates**:
    *   *Pattern*: "Immediate feedback, background sync".
    *   *Implementation*: 
        1.  User drags to paint slots -> Frontend state updates immediately (visually green).
        2.  Debounce logic (wait 500ms after last drag event).
        3.  `fetch` POST to backend.
        4.  *Error Handling*: If `fetch` fails, show toast and revert visual state (red flash?).
    *   *Benefit*: crucial for the "LettuceMeet" feel where you paint quickly without waiting for spinners.
*   **Data Serialization**:
    *   **Frontend**: `JSON.stringify()` custom `Set` of selected intervals.
    *   **Backend**: `serde_json` with `axum::Json` extractor. Rust structs will derive `Deserialize`.
*   **Timezone Handling**:
    *   **Frontend**: Detect browser timezone (`Intl.DateTimeFormat().resolvedOptions().timeZone`) and send it in the header or payload for context, but **ALWAYS** send `start`/`end` in UTC ISO strings.
    *   **Backend**: Store as `UTC`. When rendering the "Best Time" heatmap, the backend can aggregate in UTC, and the frontend converts back to local time for display.

### Microservices / Hybrid Integration

*   **Auth Integration**:
    *   The API must respect the existing `session` based auth. The `POST` endpoint will be protected by `LoggedInUser` extractor.
*   **Performance Optimization**:
    *   **Payload Size**: A week of 15-min slots is ~672 slots max. Even sending the full array is minimal (<10KB JSON). No need for complex bitmask compression over the wire unless strictly necessary.

---

## Architectural Patterns and Design

### System Architecture Patterns

*   **Backend Architecture**: **Modular Monolith (Clean Architecture)**.
    *   *Rationale*: Simplicity of deployment (single binary) with the flexibility of decoupled modules (`auth`, `scheduling`, `notifications`) inside.
    *   *Layers*:
        1.  **Transport Layer (`src/handlers`)**: Axum routes, request extraction, response formatting.
        2.  **Service Layer (Business Logic)**: `SchedulingService` struct managing rules (e.g., "no overlap", "user is participant").
        3.  **Repository Layer (`src/db`)**: `AvailabilityRepository` for raw SQLx queries.
*   **Frontend Architecture**: **Island Architecture (Micro-Frontends Lite)**.
    *   *Rationale*: The D&D Scheduler is a classic Multi-Page App (MPA). The new Calendar will be a rich, interactive "Island" embedded in `create-poll.html` and `participate.html`.
    *   *Isolation*: The Calendar logic will be encapsulated in a pure JS module (`CalendarWidget.js`) that initializes on a specific DOM container ID.

### Design Principles (Frontend State)

*   **State Management**: **Store Pattern (Vanilla JS)**.
    *   *Pattern*: A simple `Store` class with `subscribe()` method (Observer Pattern).
    *   *State*:
        ```javascript
        {
          slots: Set<string>, // "2026-02-05T10:00:00Z"
          isDragging: boolean,
          mode: 'add' | 'remove'
        }
        ```
    *   *Flow*:
        1.  User MouseDown -> Dispatch `START_DRAG`.
        2.  Store updates `isDragging = true`.
        3.  Grid Component subscribes to store -> Updates visual class `.dragging`.
        4.  User MouseUp -> Dispatch `COMMIT_SELECTION` -> Triggers API sync.

### Integration Patterns (Conflict Resolution)

*   **Resolution Strategy**: **Last Write Wins (LWW)** with **Optimistic UI**.
    *   *Rationale*: Collaborative editing *of the same user's availability* by multiple devices is rare. Simple LWW is sufficient.
    *   *Conflict*: If a user is deleted/banned while editing, the API will return 401/403. The frontend will catch this and redirect to login, discarding changes.

### Scalability Factors

*   **Database**: SQLite `WAL` mode is already performant for thousands of concurrent reads. Normalized schema allows efficient indexing on `(poll_id, start_time)`.
*   **Rendering**:
     *   *Virtualization*: Not needed for a 7-day grid (header + 96 slots * 7 columns = ~700 divs). DOM is fast enough.
     *   *CSS Paint API*: Overkill. CSS Grid + Anime.js is sufficient.

---

## Implementation Approaches and Technology Adoption

### Technology Adoption Strategies

*   **Incremental Refactoring**:
    1.  **Phase 1 (Backend)**: Add new DB tables (`availability`) and API endpoints alongside existing ones. No breaking changes.
    2.  **Phase 2 (Frontend Alpha)**: Create a hidden/admin page to test the `CalendarWidget.js`.
    3.  **Phase 3 (Rollout)**: Replace the old "wizard" step in `create-poll.html` with the new Widget.
    4.  **Phase 4 (Cleanup)**: Remove old poll logic tables.

### Development Workflows and Tooling

*   **Tooling**:
    *   **Backend**: `cargo watch -x run` for hot reloading. `sqlx migrate` for schema changes.
    *   **Frontend**: No internal build step currently? RECOMMEND adding a lightweight bundler like **Vite** or just **ES Modules** (if target browsers allow). Given the requirement for "Vanilla JS", standard ES Modules (`<script type="module">`) are perfect and require no build step.
*   **Linting/Formatting**:
    *   Rust: `cargo clippy`, `rustfmt`.
    *   JS: `ESLint` + `Prettier` (configured for vanilla ES6).

### Testing and Quality Assurance

*   **Unit Testing (Rust)**:
    *   Test `SchedulingService` logic (overlap detection) using `mockall` or in-memory SQLite.
*   **Component Testing (JS)**:
    *   Use **Vitest** + **JSDOM** to unit test the `CalendarStore` logic (state transitions).
    *   *RATIONALE*: Logic is complex (drag ranges); manual testing is error-prone.
*   **E2E Testing**:
    *   **Cypress** or **Playwright**. Script a user logging in, dragging a range, and verifying the `POST` request.

### Security and Operations

*   **Security check**:
    *   Ensure `POST /availability` checks that `user_id` matches the session (prevent tampering).
    *   Rate limiting (Governor) on the `POST` endpoint to prevent "availability spam" (DoS).
*   **Deployment**:
    *   Existing Caddy + Docker setup works fine. The new JS files are just static assets.

---

## Technical Research Recommendations

### Implementation Roadmap

1.  **Week 1: Foundations**
    *   Create `availability` table (SQLx migration).
    *   Implement `POST / GET` endpoints in Axum.
    *   Setup `CalendarStore.js` (State Management) with unit tests.
2.  **Week 2: The Grid**
    *   Build HTML/CSS Grid structure.
    *   Implement "Drag-to-Select" logic using Pointer Events.
    *   Connect Store to Grid (Observer).
3.  **Week 3: The Magic**
    *   Add `Anime.js` staggered animations for load/reveal.
    *   Integrate `fetch` sync with Optimistic UI/Debouncing.
    *   Handle Timezone conversions (Day.js).
4.  **Week 4: Polish**
    *   "Best Time" Overlay (Heatmap calculation on frontend).
    *   Mobile touch support adjustments.

### Success Metrics

*   **Performance**: Grid renders < 100ms.
*   **UX**: Dragging feels 60fps smooth.
*   **Accuracy**: Timezones display correctly for cross-border users.
