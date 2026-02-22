---
stepsCompleted: [1, 2, 3, 4, 5]
inputDocuments: ["_bmad-output/brainstorming/brainstorming-session-2026-01-31.md", "_bmad-output/brainstorming/brainstorming-session-2026-02-05.md", "_bmad-output/planning-artifacts/research/technical-LettuceMeet-Calendar-research-2026-02-05.md", "docs/index.md"]
date: 2026-02-21
author: Tiziano_di_gennaro
---

# Product Brief: dndrs

## Executive Summary

**dndrs** is a specialized scheduling and logistics platform built exclusively for Dungeons & Dragons groups. For Dungeon Masters and adult players juggling complex lives, generic scheduling tools like LettuceMeet and Discord create "Scheduling Curse" - a frustrating mix of cognitive overload, social friction, and manual effort that often leads to "Campaign Ghosting." 

Unlike these generic tools that only solve for raw *availability*, dndrs serves as the "Campaign Heartbeat" by solving for *playability*. It reduces the time between sessions and increases the "Session Completion Rate" by acting as the single source of truth for group logistics, automating quorum logic, and replacing administrative friction with narrative momentum.

---

## Core Vision

### Problem Statement

D&D groups suffer from fragmented communication and "planning fatigue," relying on a disjointed combination of Discord, WhatsApp, and generic polling tools. This infrastructure failure places an immense administrative and emotional burden squarely on the Dungeon Master (the "Product Manager" of the game), leading to "Campaign Ghosting" where games die simply because the logistics become too heavy to carry.

### Problem Impact

The friction of scheduling creates three distinct taxes on the group:
*   **Cognitive Load:** DMs spend 30% of their prep time chasing players across multiple apps instead of writing the story.
*   **The "Silent Killer":** Information decay and missed notifications lead to last-minute cancellations, causing group morale to drop.
*   **Context Loss:** Scheduling negotiations get buried in chat channels, resulting in forgotten decisions and endless re-debates due to the lack of a Single Source of Truth.

### Why Existing Solutions Fall Short

Generic tools focus solely on *Availability* while ignoring the specific requirements of *Playability*:
*   **The DM Dependency:** Generic tools treat all users equally, forcing the DM to manually cross-reference their own crucial availability against the group's.
*   **The Quorum Problem:** Existing tools lack logic for minimum player thresholds (e.g., "We play if 4 out of 5 show up").
*   **The "Vague Maybe":** Tools like LettuceMeet don't allow for the nuanced pre-game negotiation required by adults with complex schedules (e.g., "I can play, but only if we start an hour later").
*   **Zero Momentum Support:** Clinical grids don't remind players *why* they are meeting, lowering the priority of the session slot.

### Proposed Solution

**dndrs** shifts the paradigm from a "grid of availability" to a "Path to Play." It is the engine that keeps the story moving by taking over the logistical heavy lifting. Success means consistency without friction: the DM opens the app to see a green "Confirmed" status for the next session without sending a single ping, and players transition from "Are we playing?" to "I can't wait."

### Key Differentiators

*   **D&D-Integrated Logic:** Built-in "Minimum Quorum" rules and "DM Status" dependencies that automatically calculate true playability.
*   **The Campaign Heartbeat:** A centralized dashboard that acts as the single source of truth, eliminating information decay and the "tab-switching tax."
*   **Narrative Momentum:** Features designed to lock in dates with lead time, reducing DM anxiety and tying the logistics directly to the excitement of the game itself.

---

## Target Users

### Primary Users

**Alex, the Burdened Bard (Dungeon Master)**

*   **Profile:** 34-year-old Senior Project Manager/Creative Lead. Overwhelmed by "Digital Fatigue" after a 9-to-5 day juggling Jira, Slack, and Zoom, along with a toddler and a mortgage.
*   **Motivation:** D&D is their creative escape and the primary social glue for their friend group. Their ultimate goal is the narrative payoff—seeing players react to the story they've carefully prepped.
*   **The Problem:** Alex suffers from "Logistical Anxiety." Preparing a session takes 20+ hours, but scheduling requires manually cross-referencing Discord emojis, LettuceMeet availability, and vague text messages. This dynamic creates resentment; when a game dies because of a scheduling failure, it feels like personal disrespect, leading to "DM Burnout."
*   **Success Vision:** Success is the 15 minutes of silence before a session starts where everyone is laughing and eating pizza, knowing the logistics were handled a week ago. They need "One-Click Quorum" (a clear "Saturday is PLAYABLE" indicator) and an "Automated Nag" to remove them from the uncomfortable role of harassing friends for availability.

### Secondary Users

**Sam, the Busy Adventurer (Player)**

*   **Profile:** 32-year-old Software Engineer/Mid-level Manager. Sam lives a fluid, on-demand life and views D&D as a fun, passive social activity to decompress.
*   **Motivation:** Hanging out with friends and enjoying the escapism. Sam doesn't do prep work and just wants to show up to play.
*   **The Problem:** Filling out a massive LettuceMeet grid feels like doing taxes—an administrative chore that breaks the fantasy. They often read scheduling messages while commuting and resort to vague "emoji reactions" because they can't check their calendar at that exact moment, unintentionally ghosting the DM.
*   **Success Vision:** A "frictionless" experience. Instead of a daunting 7-day grid, Sam wants a "One-Tap Magic" notification ("A Dragon Approaches... Can you make it next Friday?") or a highly optimized, D&D-themed UI that makes providing availability feel like the first step of the game, rather than a corporate form.

### User Journey

*   **Discovery & Onboarding:** Alex (DM) sends a link to the campaign dashboard. Sam (Player) clicks the link, creates an account quickly, and immediately sees their character's name and the campaign art, reinforcing that this is their game, not a generic tool.
*   **The Weekly Rhythm:**
    1.  **Monday:** Sam receives an automated notification to input availability for the upcoming week.
    2.  **The Action:** Sam opens the app on their phone and quickly drags over open times on a fast, responsive, vertically optimized D&D-themed UI.
    3.  **The Payoff:** On Thursday morning, Sam and Alex both get a notification: "Quorum Reached! Session is Confirmed for Saturday at 6 PM."
*   **The Session:** Sam arrives to a relaxed group. Alex is prepared, and there were no last-minute "are we still on?" texts. The system handled the burden, and the group simply plays.

---

## Success Metrics

### User Success Metrics

*   **For the DM (Alex):** Time saved on "Go/No-Go" logic and manual checking. Visualized by the presence of a "Quorum Reached" auto-confirmation.
*   **For the Player (Sam):** Providing availability effortlessly. Visualized by a high percentage of players responding to the "Magic Link" prompt within 6 hours.

### Business Objectives

*   **The "Town Crier" Efficacy:** A significant reduction of manual scheduling "pings" in Discord/WhatsApp, demonstrating that dndrs has become the central scheduling hub.

### Key Performance Indicators (KPIs)

*   **The "North Star Metric":** Session Interval (the time between games). 
    *   *Target:* Shorten a group's average interval from 21 days to a consistent 14 days (or weekly).
*   **Response Velocity:** Percentage of players responding to session availability prompts within the first 6 hours.
*   **Session Completion Rate:** The ratio of scheduled games that actually occur without cancellation.

---

## MVP Scope

### Core Features

1.  **The "Quorum Engine" (DM Control Center):** A rule-based logic gate configured during campaign setup. The DM sets the "Minimum Hero Requirement" (e.g., 1 DM + 3 Players). The dashboard automatically toggles a session from "Pending" to "Confirmed" when the requirement is met, removing the burden of manual calculation.
2.  **The "Magic Link" Initiative (Player UX):** A frictionless "One-Tap Response" interface for players. It uses a persistent token to bypass logins. Players simply tap large, D&D-themed buttons (e.g., "Battle Ready", "Needs Insight") in response to a simple prompt: "Next Session: Saturday @ 7pm. Are you coming?"
3.  **The "Automated Town Crier" (Integration):** A Discord/Telegram bot integration that acts as "Nag-as-a-Service." It automatically posts the "Magic Link" when a session is created and playfully pings unresponsive players 48 hours before the deadline if quorum isn't met.

### Out of Scope for MVP

*   **No Virtual Tabletops (VTT):** Exploring maps, tokens, or dice rolling is strictly excluded. The focus is on the "Lobby," not the "Game Room."
*   **No Character Management:** Storing character sheets, stats, or inventory is excluded to avoid competing with D&D Beyond. 
*   **No Robust Payment Systems:** Escrow or payment processing for "Professional DMs" is excluded to avoid engineering and regulatory overhead in the MVP phase.
*   **No In-App Chat:** Building a chat system is a distraction; the application will leverage existing social rails (Discord/WhatsApp) instead.

### MVP Success Criteria

*   The MVP successfully demonstrates a reduction in the "Session Interval" towards the 14-day target. 
*   High adoption and response velocity (within 6 hours) of the "Magic Link" feature by players.
*   Observed reduction in manual coordination pings within external chat platforms.

### Future Vision

*   **Phase 2: The "Campaign Chronicler" (Year 2):** Addition of light narrative features like a collaborative "Session Recap" wiki and a shared "Loot Tracker" ledger to manage party resources.
*   **Phase 3: The "Tabletop OS" (Year 3+):** Horizontal expansion to other TTRPGs (e.g., Pathfinder, Call of Cthulhu). Development of a "Player Matchmaker" to connect DMs with reliable players based on Logistical XP. Introduction of "Smart Calendar AI" to suggest session times based on group history (e.g., "You are 80% more likely to play on Sundays").

<!-- Content will be appended sequentially through collaborative workflow steps -->

<!-- Content will be appended sequentially through collaborative workflow steps -->
