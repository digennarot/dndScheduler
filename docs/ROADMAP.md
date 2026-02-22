# D&D Scheduler - Project Roadmap

## 🚀 Epic 5: Advanced Scheduling
**Goal**: Support complex scheduling scenarios for long-running campaigns.
- [ ] **Recurring Events**: Support for "Every Friday", "Bi-weekly", and custom patterns (e.g., "First Saturday of the month").
- [ ] **Calendar Integration**: Two-way sync with Google Calendar and Outlook (.ics subscriptions).
- [ ] **Timezone Intelligence**: Auto-detect and convert times for remote groups across different timezones.

## 👥 Epic 6: Community & Profiles
**Goal**: Transform the scheduler into a hub for D&D groups.
- [ ] **User Profiles**: Avatars, bios, RPG preferences (D&D 5e, PF2e, etc.).
- [ ] **Persistent Campaigns**: dedicated pages with session logs, loot tracking, and NPC databases.
- [ ] **Friend Groups**: Quick-add entire existing parties to new polls.

## ⚡ Epic 7: Real-Time Interactivity
**Goal**: Make the application feel alive.
- [ ] **WebSockets**: Live updates on poll voting and activity feed (remove need for page refreshes).
- [ ] **Push Notifications**: Service Workers for browser-based push notifications on mobile/desktop.

## 🛠️ Technical Improvements (Backlog)
- **Architecture**: Split `main.rs` into `startup` and `configuration` modules to clean up entry point.
- **Testing**: Implement E2E tests with Playwright for critical flows (Login -> Vote).
- **Database**: Evaluate migration from SQLite to PostgreSQL for high-concurrency deployments if user base grows >10k.
