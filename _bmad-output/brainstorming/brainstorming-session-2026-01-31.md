---
stepsCompleted: [1, 2]
inputDocuments: []
session_topic: 'Google Login with Authelia & D&D Scheduler'
session_goals: 'Enable Google Authentication while maintaining interoperability with existing Authelia SSO and Rust backend logic.'
selected_approach: 'ai-recommended'
techniques_used: ['Constraint Mapping', 'SCAMPER Method', 'Role Playing']
ideas_generated: []
context_file: ''
---

# Brainstorming Session Results

**Facilitator:** Antigravity
**Participant:** Tiziano
**Date:** 2026-01-31

## Session Overview

**Topic:** Google Login with Authelia & D&D Scheduler
**Goals:** Enable Google Authentication while maintaining interoperability with existing Authelia SSO and Rust backend logic.

### Session Setup

The user specifically requested "google login works with authelia". This implies a need to integrate Google as an Identity Provider (IdP) or parallel authentication method without breaking the current Authelia-based session creation and user syncing.

## Technique Execution Results

### Phase 1: Constraint Mapping
**Focus:** Visualizing integration boundaries.

- **Dual Mode Auth:** Authelia is the main gateway, but the system allows direct Google Login (bypassing Authelia).
- **Identity Merging:** Email is the unique identifier. `user@example.com` is the same person regardless of auth method.
- **Session Strategy:** Google users receive **Stateless JWTs**, distinct from the DB-backed `user_sessions` table used by local/Authelia users.
- **Frontend Layer:** `session-manager.js` needs to support both session cookies and JWTs.

### Phase 2: SCAMPER Method
**Focus:** Adapting `system-auth-handlers`

- **S - Substitute (Unified Principal):** Replace rigid session checks with a single `Principal` extractor that accepts:
    1.  Authelia Headers
    2.  Local Session Token
    3.  Google JWT
- **C - Combine (JIT Provisioning):** Combine Login and Registration. A valid Google Token automatically creates the user if they don't exist.
- **A - Adapt (Sentinel Hash):** Adapt the `password_hash` column to accept a sentinel value (e.g., `"GOOGLE_OAUTH_USER"`) for Google-only users, preserving the schema without allowing password login for them.

### Phase 3: Role Playing
**Focus:** Validation & Edge Cases ("The Hybrid User")

- **Scenario:** User with existing password logs in via Google.
- **Decision:** **Silent Merge (Option A)**.
- **Rationale:** Frictionless experience. If they own the email (proven by Google), they own the account. No need to force "linking" or reject them. Use email as the single source of truth.

## Technique Execution Results - Conclusion
We have generated a complete architectural blueprint:
1.  **Dual Auth Stack:** Authelia and Google side-by-side.
2.  **Unified Identity:** Email-based merging.
3.  **Stateless Session:** Google users use JWTs, verified by public key/certs.
4.  **Auto-Onboarding:** JIT provisioning for new Google users.
## Idea Organization & Prioritization

### Theme 1: Architectural Integration (System)
*Focus: Seamlessly merging parallel auth streams without DB redundancy.*
- **Unified Principal Extractor:** Decouple "who is logged in" from "how they logged in".
- **Stateless JWTs:** Avoids `user_sessions` bloat for Google users.
- **Sentinel Hashes:** Preserves schema integrity (`NOT NULL`) for password-less users.

### Theme 2: User Experience (Flow)
*Focus: Removing friction for new and returning users.*
- **JIT Provisioning:** No separate registration form. Click Google -> Account Created.
- **Silent Merge:** Automatic recognition of existing email users.
- **Dual Mode:** User choice preserved (Authelia for power users/admins, Google for casuals).

### Implementation Action Plan

**1. Database Migration `schema.sql`**
- [ ] No structural changes needed (thanks to Sentinel Hash idea).
- [ ] *Validation:* Ensure `password_hash` column is wide enough for the sentinel (it is).

**2. Backend Logic `src/auth.rs`**
- [ ] Create `GoogleUser` struct for JWT claims.
- [ ] Implement `verify_google_token` (stateless verification).
- [ ] Update `get_current_user` to accept JWTs.
- [ ] Implement `login_or_register_google` handler (JIT logic).

**3. Frontend Integration `static/js/`**
- [ ] Add "Sign in with Google" button to login page.
- [ ] Update `app.js` to store/send JWTs in `Authorization` header.

## Session Summary
We successfully navigated the complexity of adding a second auth provider to a strict existing system. By choosing a **Stateless/Hybrid approach**, we avoided complex database migrations and session synchronization issues. The key breakthrough was the **Silent Merge** strategy, ensuring users are never blocked by their own account history.


