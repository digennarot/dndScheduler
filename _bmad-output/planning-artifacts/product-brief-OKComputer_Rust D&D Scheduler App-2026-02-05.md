---
stepsCompleted: [1, 2, 3, 4, 5]
inputDocuments:
  - /home/tiziano/Scaricati/OKComputer_Rust D&D Scheduler App/_bmad-output/planning-artifacts/research/technical-LettuceMeet-Calendar-research-2026-02-05.md
  - /home/tiziano/Scaricati/OKComputer_Rust D&D Scheduler App/docs/project-overview.md
  - /home/tiziano/Scaricati/OKComputer_Rust D&D Scheduler App/docs/design.md
date: 2026-02-05
author: Tiziano
---

# Product Brief: OKComputer_Rust D&D Scheduler App

<!-- Content will be appended sequentially through collaborative workflow steps -->

## Executive Summary

The **OKComputer_Rust D&D Scheduler** aims to solve the perennial "herding cats" problem of scheduling D&D sessions. Currently, groups rely on disjointed tools like WhatsApp polls or generic schedulers (LettuceMeet) that lack context and campaign integration. This project will deliver a seamless, "magical" scheduling experience directly tied to the user's campaign, featuring a high-performance, interactive availability heat map. By combining the utility of a dedicated scheduler with a rich, immersive UX, we ensure that the task of scheduling feels like an extension of the game itself, increasing session frequency and player engagement.

---

## Core Vision

### Problem Statement

Scheduling D&D sessions for 5+ people is a logistical nightmare, often leading to cancelled sessions and "scheduling fatigue" for DMs. Current methods (WhatsApp polls, text threads) are chaotic, lack visual clarity, and are easily buried in chat history.

### Problem Impact

Inconsistent play, loss of campaign momentum, and significant administrative burden on the Dungeon Master, potentially leading to burnout.

### Why Existing Solutions Fall Short

*   **WhatsApp/Discord Polls**: Clunky interaction, lack visual "best time" identification, and separate from game notes.
*   **LettuceMeet**: Excellent UI, but generic. It lacks D&D context (character names, campaign vibes), requires external links management, and disconnects scheduling from the campaign platform.

### Proposed Solution

An embedded, immersive calendar "island" within the existing D&D Scheduler app. It replicates the intuitive "drag-to-paint" interface of LettuceMeet but enhances it with "mystical" visual effects (Anime.js) and integrates directly with the app's authentication and campaign management.

### Key Differentiators

*   **"Magical" UX**: Scheduling feels like casting a spell (visual flourishes, fluid animations), not data entry.
*   **Deep Integration**: Linked directly to User Profiles and Campaigns; no anonymous links or external accounts needed.
*   **Privacy & Control**: Self-hosted architecture keeps group data private and secure.

## Target Users

### Primary Users

*   **The Campaign Architect (DM)**: The heart of the campaign who carries the mental load of prep and continuity. They need **control**—the ability to set rules (e.g., "min 3 players"), view attendance trends to manage burnout, and finalize dates decisively to protect their prep time.
*   **The Social Herder (Organizer Player)**: The motivated player who acts as the logistical engine. They need **efficiency**—tools to poll availability, spot overlaps, and chase RSVPs without checking five different apps, enabling them to lock in sessions even if they aren't running the game.

### Secondary Users

*   **The Drifter (Busy/Unreliable Player)**: Users with chaotic schedules or shifting priorities. They need **zero friction**—a mobile-first, "two-tap" interface (Link -> Swipe -> Done) with clear feedback ("You're in"), ensuring they don't become the bottleneck.
*   **The Regular (Committed Player)**: The bedrock players who prioritize the game. They need **certainty**—fast confirmation of dates and transparency on who is attending, so they can keep the game alive and avoid "scheduling fatigue" despair.

### User Journey (The "Herding Cats" Flow)

1.  **Trigger**: DM/Organizer sends a "Summon Party" link to the group chat (WhatsApp/Discord).
2.  **Interaction (The Magic)**: Players click the link (no login required for view-only, or auto-login via persistent session). They see the "Magical Grid" and paint their free time.
    *   *Drifter*: Paints 2 slots, closes app. (< 30s)
    *   *Regular*: Paints slots, checks who else is free.
3.  **Resolution**: The "Best Time" heatmap reveals the winner. DM clicks "Finalize".
4.  **Confirmation**: Automated message goes back to the group chat: "Session Confirmed: Tuesday @ 8PM". Calendar invites sent.

## Success Metrics

### User Success Outcomes

*   **Consistency is King**: The primary measure of success is the **Frequency of Play**. Moving from sporadic monthly sessions to a consistent weekly/bi-weekly cadence (Target: 3+ sessions/month).
*   **Zero "Dead Air"**: The next session should be booked *before* momentum dies. The tool ensures there is rarely a moment where "no date is on the calendar."
*   **Prep Protection**: DMs feel safe investing 10+ hours into adventure design because confidence in attendance is high (Cancellation rate < 20%).

### Business Objectives (Project Goals)

*   **Poll Conversion (Polls → Sessions)**: Achieve an **80%+ conversion rate** from "Poll Created" to "Session Confirmed". We want to eliminate "abandoned polls" where groups give up.
*   **Efficiency**: Reduce the "Scheduling Tax" from days of back-and-forth to **< 5 minutes** of active time per session for the DM.
*   **Campaign Survival**: Increase the % of campaigns that survive past the 3-month mark (The "Campaign Death Valley"). Target **70% retention**.

### Key Performance Indicators (KPIs)

*   **Avg. Time to Consensus**: Time from "Poll Created" to "Date Finalized". Target: < 24 hours.
*   **Player Response Rate**: % of players who respond to the link within 12 hours. Target: > 90%.
*   **Session Reliability**: Ratio of Scheduled vs. Played sessions.

## MVP Scope

### Core Features

*   **Frictionless Poll Creation**: A dedicated "Wizard" for DMs to select date ranges and time slots. Generates a unique, shareable link (no player accounts required).
*   **Magical Grid Input**: The signature feature. A responsive, "drag-to-paint" availability grid that updates in real-time.
*   **Heatmap Visualization**: Instant visual feedback of the "Best Time" based on player overlaps (darker = more available).
*   **One-Click Finalize**: A simple "Lock it in" button for the DM that generates a confirmation page with a copyable ICS link.

### Out of Scope for MVP

*   **Automated Notifications**: No email reminders or Discord bot integrations. We rely on the "Social Herder" to paste the link.
*   **Calendar API Integration**: No complex OAuth flows (Google/Outlook). Simple ICS file export only.
*   **Recurring Polls**: One-off session planning only for now.
*   **User Profiles for Players**: Players are anonymous (identified by name input on the grid) to maintain zero friction.

### MVP Success Criteria

*   **Speed**: A group can go from "Link Sent" to "Date Finalized" in **< 24 hours**.
*   **Utility**: DMs prefer this over WhatsApp polls because the visual overlap saves them mental math.
*   **Reliability**: The system never "loses" a vote or suggests a time that someone marked as unavailable.

### Future Vision

Post-MVP, this evolves into the "Command Center" for the campaign.
*   **Automated Nudges**: "Hey [Name], you haven't voted yet."
*   **Campaign Integration**: "Session Confirmed" automatically creates a blank Session Note entry in the app.
*   **Discord Integration**: A bot that posts the "Summon Party" link and updates the channel topic with the next session date.
