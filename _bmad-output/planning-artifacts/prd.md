---
stepsCompleted: [step-01-init, step-02-discovery, step-03-success, step-04-journeys, step-05-domain, step-06-innovation, step-07-project-type, step-08-scoping, step-09-functional, step-10-nonfunctional, step-11-polish]
classification:
  projectType: web_app
  domain: entertainment
  complexity: medium
  projectContext: brownfield
inputDocuments:
  - /home/tiziano/Scaricati/OKComputer_Rust D&D Scheduler App/_bmad-output/planning-artifacts/product-brief-OKComputer_Rust D&D Scheduler App-2026-02-05.md
  - /home/tiziano/Scaricati/OKComputer_Rust D&D Scheduler App/_bmad-output/planning-artifacts/research/technical-LettuceMeet-Calendar-research-2026-02-05.md
  - /home/tiziano/Scaricati/OKComputer_Rust D&D Scheduler App/docs/project-overview.md
workflowType: 'prd'
---

# Product Requirements Document - OKComputer_Rust D&D Scheduler App

**Author:** Tiziano
**Date:** 2026-02-05

## 1. Executive Summary

The **OKComputer_Rust D&D Scheduler** solves the perennial "herding cats" problem of TTRPG scheduling. It delivers a seamless, "magical" availability heatmap that integrates directly with the user's campaign context, replacing disjointed tools like WhatsApp polls or generic schedulers (e.g., LettuceMeet).

**Core Differentiators:**
*   **Magical UX:** Scheduling feels like casting a spell, not a spreadsheet, utilizing rich animations (Anime.js) and particle effects.
*   **Trust-Based Anonymity:** A "No Login" flow for players reduces friction to zero, relying on social trust rather than security theater to ensure high participation rates.
*   **Self-Hosted Sovereignty:** Architecture ensures data stays local (SQLite), offline-capable, and private, respecting the ethos of the self-hosted community.

## 2. Success Criteria

### User Success
*   **Consistency**: Groups move from sporadic to weekly/bi-weekly sessions (Target: 3+ sessions/month).
*   **Prep Protection**: DMs feel safe prepping deep content (Cancellation rate < 20%).
*   **Zero Dead Air**: The next session is always booked before the current one ends.

### Business Success
*   **Poll Conversion**: 80% of created polls result in a confirmed session.
*   **Efficiency**: DMs spend < 5 minutes of active time per session on scheduling logistics.

### Technical Success
*   **Performance**: Availability Grid interactions render in < 50ms.
*   **Reliability**: Zero data loss during concurrent edits (Optimistic UI + Polling).
*   **Integration**: Zero-config deployment for self-hosters.

## 3. Product Scope

### Phase 1: MVP (Minimum Lovable Product)
*   **Frictionless Poll Creation**: Visual Wizard for DMs to select date ranges.
*   **Magical Grid Input**: Anonymous "Drag-to-Paint" availability for players.
*   **Heatmap Visualization**: Instant visual overlay of overlaps ("Best Time").
*   **One-Click Finalize**: Lock voting and export to ICS.

### Phase 2: Growth
*   **Discord Integration**: Bot for automated "pestering" and status updates.
*   **Recurring Polls**: "Clone this poll for next week" functionality.
*   **Dynamic OpenGraph**: Server-generated preview images for chat apps.

### Phase 3: Vision (Campaign Command Center)
*   Full integration with Session Notes, Player Profiles, and Campaign History.

## 4. User Journeys

### The Campaign Architect (DM)
**Scenario**: Session just ended. Players are non-committal about next week.
*   **Action**: Clicks "Summon Party". Selects next week's dates. Pastes the generated link into Discord.
*   **Value**: Total active time: 45 seconds. No mental math on timezones.

### The Drifter (Chaos Player)
**Scenario**: At a bar, phone buzzes with a link.
*   **Action**: Taps link. Swipes thumb over "Tuesday Evening" (Green). Closes tab.
*   **Value**: Zero friction. No login to forget. Participation is effortless.

### The Social Herder (Organizer)
**Scenario**: Wednesday morning, trying to lock in Friday.
*   **Action**: Checks Heatmap. Sees "Dave" hasn't voted. Pings Dave directly.
*   **Value**: Instant visual clarity on the bottleneck.

### The Self-Hoster (Admin)
**Scenario**: Installing the app on a home server.
*   **Action**: Runs `docker-compose up`. App starts with SQLite.
*   **Value**: "It just works" simplicity. No external databases or cloud dependencies.

## 5. Functional Requirements (Capability Contract)

### Poll Management
*   **FR1**: Users can create a new poll without an account (Anonymous DM).
*   **FR2**: Users can define a date range for the poll using a visual calendar picker.
*   **FR3**: Users can set a poll title and optional description.
*   **FR4**: Users represent the "Timezone Authority" (Poll defaults to their TZ).
*   **FR5**: Users can delete a poll they created (via Admin Key/Token).
*   **FR19**: Users can "Finalize" a poll to lock voting.
*   **FR20**: Users can export the finalized event as an ICS file.

### Participation & Voting
*   **FR6**: Users can access a poll via a shared link without login (Anonymous Player).
*   **FR7**: Users can view availability times in their auto-detected local timezone.
*   **FR8**: Users can select availability slots via "Drag-to-Paint" interaction (Desktop & Mobile).
*   **FR9**: Users can edit their previous vote (identified by LocalStorage/Session Token).
*   **FR10**: Users can toggle between "Yes" (Green) and "Maybe" (Yellow) states.

### Result Visualization
*   **FR11**: Users can view an aggregate "Heatmap" of all participant availability.
*   **FR12**: Users can hover/tap a slot to see specifically *who* voted for it.
*   **FR13**: Users can visually identify the "Best Time" (highest overlap).

### Sharing & Integration
*   **FR14**: System generates a unique, obscure URL for each poll.
*   **FR15**: System renders OpenGraph metadata (Title, Date Range) for Discord/WhatsApp previews.

### Administration & Security
*   **FR16**: System rate-limits poll creation and voting by IP address.
*   **FR17**: Admin users can delete specific votes from a poll (Troll mitigation).
*   **FR18**: System purges poll data automatically after a configurable expiration (default 30 days).

## 6. Non-Functional Requirements (Quality Attributes)

### Performance
*   **NFR1**: The Availability Grid must render interactive states in < 50ms (responsive to touch/click).
*   **NFR2**: Creating a poll must complete in < 200ms (Database Write -> Redirect).

### Security & Privacy
*   **NFR3**: No user data (votes/names) persists beyond the poll expiration date (Auto-purge).
*   **NFR4**: API must reject requests exceeding rate limits (10 req/min per IP) with 429 Too Many Requests.
*   **NFR5**: Admin "Delete" actions must be protected by a server-side Token.

### Accessibility
*   **NFR6**: Heatmap colors must pass WCAG AA contrast ratio for "Yes" vs "No" states.
*   **NFR7**: All interactive elements must be focusable/actionable via Keyboard.

### Integration / Self-Hosting
*   **NFR8**: The application binary must start with zero external configuration (defaults to `./data.db`).
*   **NFR9**: The application must function 100% offline (no external CDNs).

## 7. Domain & Innovation Analysis

### Domain Constraints (D&D / Self-Hosted)
*   **Zero Telemetry**: No external tracking or analytics.
*   **Data Ownership**: Full export/delete capabilities.
*   **Mobile Priority**: The player voting interface must be robust on Mobile Safari.

### Innovation Areas
*   **Gamified Utility**: Prioritizing "Form" (Magical feel) alongside "Function" to drive engagement.
*   **Trust-Based Anonymity**: Leveraging the high-trust social contract of D&D groups to remove authentication barriers.

### Risk Mitigation
*   **"Form over Function" Risk**: Mitigated by strict performance NFRs (< 100ms).
*   **Trust Failure Risk**: Mitigated by Admin "Delete Vote" panic button for trolls.
