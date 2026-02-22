---
stepsCompleted: [1, 2, 3, 4, 5, 6, 7, 8]
inputDocuments: ["_bmad-output/planning-artifacts/prd.md", "product-brief-dndrs-2026-02-21.md", "docs/index.md"]
---

# UX Design Specification dndrs

**Author:** Tiziano_di_gennaro
**Date:** 2026-02-21

---

## Executive Summary

### Project Vision

**dndrs** is a frictionless, visually engaging scheduling platform designed specifically for Dungeons & Dragons groups. It replaces disjointed polls and generic calendar tools with a centralized "Campaign Heartbeat." The vision is to solve for *playability* rather than just *availability* through an automated "Quorum Engine" and a zero-login "Magic Link" experience that feels like casting a spell rather than doing a chore.

### Target Users

*   **The Campaign Architect (Alex the DM):** Overwhelmed by "Logistical Anxiety." Needs an automated system that confirms sessions based on predefined rules (Quorum Engine), removing the burden of manual coordination and "nagging" players.
*   **The Drifter (Sam the Chaos Player):** Multitasking and forgetful. Needs an absolutely frictionless "No Login" flow, heavily optimized for speed (especially on mobile), to input availability in seconds via a "Magic Link."
*   **The Self-Hoster (Admin):** Valuing data sovereignty, requiring a zero-config, offline-capable setup.

### Key Design Challenges

1.  **Balancing 'Magic' with Utility & Performance:** The UI requires rich animations and particle effects to feel thematic (D&D spell), but strict non-functional requirements mandate <50ms interactive rendering and mobile-first responsiveness. "Form" cannot kill "Function."
2.  **The "Anonymous yet Persistent" Flow:** Managing a frictionless "no-login" state for players (via Magic Links and LocalStorage tokens) while ensuring they can reliably return to edit their specific votes without stepping on others.
3.  **Visualizing the Quorum Engine:** Taking complex logic (e.g., "Need DM + 3 out of 5 players") and translating it into a simple, glanceable "Pending" vs. "Confirmed" state for both the DM and the players.

### Design Opportunities

1.  **Micro-Interactions that Delight:** Making the simple act of painting a timeslot feel satisfying and rewarding (e.g., visual feedback that feels like gaining XP or rolling a natural 20).
2.  **The "Call to Action" Notification:** Designing Discord bot integration (the "Town Crier") so that the "Magic Link" embed feels like an urgent, narrative quest prompt rather than a generic calendar invite.

---

## Core User Experience

### Defining Experience

The most critical interaction—the "make or break" moment—is the **Player’s "One-Tap Commitment."** The Magic Link landing page must feel less like a form and more like a "Ready" button in a video game lobby. If the player feels friction, they revert to vague Discord messages.

### Platform Strategy

A **Bi-Modal UX** approach is required:
*   **DM (Desktop-First):** The DM needs a "Command Center" view to manage the Quorum and visualize heatmaps, likely accessed on a desktop while prepping.
*   **Player (Aggressively Mobile-First):** Players access the app via a Discord link on their phones. The experience must require zero pinching, zooming, or logging in.

### Effortless Interactions

The most magical element is **Identity Persistence without Authentication** (The "Ghost Identity"). When a player clicks their personalized Magic Link, the app instantly recognizes them via url tokens: *"Welcome back, Jordan! Alex is looking at Saturday at 7 PM. You in?"* 

### Critical Success Moments

The "Aha!" moment is the **"Quorum Snap" Visual Resolution**. When a player taps "Available" and the progress bar instantly snaps to 100% (Green) displaying **"SESSION CONFIRMED"**, it provides an immediate dopamine hit, transforming logistics into a unified victory.

### Experience Principles

1.  **Zero-Friction Participation:** Player input must be instant and effortless, requiring no logins or complex navigation.
2.  **Bi-Modal Optimization:** Design the DM tools for desktop power-usage and the player tools for mobile speed.
3.  **Identity via Context:** Leverage Magic Links to provide personalized experiences without traditional authentication barriers.
4.  **Rewarding Resolution:** Celebrate quorum achievement with clear, satisfying visual feedback that mimics a game's success state.

---

## 2. Core User Experience (Mechanics)

### 2.1 Defining Experience

The defining interaction is the **"Magic Link to Quorum Snap"**. It transforms the mundane task of filling out a calendar poll into a satisfying, collaborative game-like confirmation. It removes all friction, validating the user instantly.

### 2.2 User Mental Model

Currently, users view scheduling as a chore—a spreadsheet that forces them to cross-reference multiple calendars and wait for slow responses. The mental model for **dndrs** shifts this to a "Lobby Checkout" or "Raid Ready" concept. Users expect to jump in, announce "I am ready," and instantly see the collective status of the group.

### 2.3 Success Criteria

1.  **Sub-second onboarding:** The moment the Discord link is clicked, the UI is instantly interactive. Zero loading spinners for auth.
2.  **Visceral feedback:** Tactile, glowing feedback upon selecting timeslots.
3.  **Instant state resolution:** Polling/WebSockets update other clients in real-time, allowing the "Quorum Snap" to happen concurrently if multiple people vote at once.

### 2.4 Novel UX Patterns

We are combining an established pattern (the mobile interaction of drag-to-select, popularized by iOS calendar/drag-to-paint) with a novel context (anonymous persistent identity via Magic Links). The novelty lies in stripping away the traditional "Enter Your Name and Password" modal before acting. 

### 2.5 Experience Mechanics

**1. Initiation (0.0s - 0.5s):**
*   User taps link in Discord on their phone.
*   The page loads instantly (cached, lightweight CSS). The URL token is parsed.
*   A personalized toast/banner slides in safely: *"Welcome back, Jordan! The party needs you."* The "Battle Ready" calendar is already fully visible and interactive.

**2. Interaction (The "Painting"):**
*   The user taps and holds to paint across desired time blocks. 
*   **Feedback:** Haptic feedback on mobile (if supported via JS) for each block touched. The blocks aggressively transition from a muted dark slate to a glowing, vibrant gold. 

**3. Feedback & Completion (The Quorum Snap):**
*   The user taps the "SUBMIT" or "CAST SPELL" button. 
*   If this vote meets the DM's Quorum rule, the "Party Readiness" bar at the top ignites into gold. 
*   Golden motes (Anime.js particles) burst outward.
*   A "QUEST CONFIRMED" seal animates with a slight screen shake and a resonant "thrum" sound effect.
*   The grid elegantly fades away, replaced by an overlaid "Countdown" layout ("03 Days : 14 Hours until Initiative") with a prominent "Share hype to Discord" button.

---

## Desired Emotional Response

### Primary Emotional Goals

**Excitement & Relief.** Users should feel the anticipatory thrill of an impending game ("Stop Scheduling. Start Playing.") rather than the draining obligation of a corporate meeting poll. DMs should feel a profound sense of *Relief* knowing that the logistical burden has been lifted.

### Emotional Journey Mapping

1.  **The Call to Arms (Discord Ping):** Player feels *intrigue and excitement*. It's not a generic calendar link; it's a stylized prompt that feels like part of the campaign.
2.  **The "Painting" Action:** Player feels *empowered and fast*. Swiping their availability yields satisfying, magical visual feedback (Arcane Tech aesthetic).
3.  **The Quorum Reached (Confirmation):** Both DM and Player feel *victorious and relieved*. A collective dopamine hit as the "Party Readiness" bar hits 100% with a gold glow.

### Micro-Emotions

*   **Delight over Satisfaction:** It's not just "done"; hitting submit should feel like rolling a Natural 20.
*   **Confidence over Confusion:** The "Ghost Identity" must explicitly reassure the player ("We know it's you, Jordan") without triggering anxiety about privacy or account creation.

### Design Implications

*   **Arcane Tech Aesthetic:** Dark mode by default, glassmorphism, and parchment/glowing dice motifs to enforce the fantasy feel over generic SaaS.
*   **Particle Effects for Positive Actions:** Using high-performance Anime.js particles when a user "paints" a timeslot or when the session status flips to "PLAYABLE".
*   **Warm "Success" Palettes:** Replacing clinical greens and reds with magical gold, arcane purple, and soft ambient glows to signify success states.

### Emotional Design Principles

1.  **Enchant the Mundane:** Every interaction (from a button click to a date selection) should have a trace of "magic" through micro-animations.
2.  **Heroic Confirmation:** Reaching Quorum is treated not as a data point, but as a party victory.

---

## UX Pattern Analysis & Inspiration

### Inspiring Products Analysis

*   **Video Game Lobbies (e.g., Overwatch, Destiny):** The UX pattern of hitting a "Ready" button and watching a lobby fill up. This translates directly into the "Party Readiness" bar, creating instant shared momentum.
*   **Discord/Slack Integration:** Leveraging existing social hubs rather than forcing users into a new ecosystem. The bot notification acts as the entry point, treating scheduling as a quest hook.

### Transferable UX Patterns

*   **The "Quorum Snap":** When the final required player taps "Battle Ready", the progress bar doesn't just fill—it *ignites*, shifting from pending gray/blue to glowing gold.
*   **The Particle Burst:** A central burst of Golden Motes (Arcane Tech aesthetic) when quorum is achieved, validating the group's effort.
*   **The Heroic Transition:** Immediately after quorum, the UI transitions from "Voting Mode" to a "Countdown Mode" ("03 Days : 14 Hours : 22 Minutes until Initiative"), building hype.
*   **Auditory Feedback:** A deep resonant "thrum" followed by a sheath sound for confirmation, and a distinct melodic chime for notifications.

### Anti-Patterns to Avoid

*   **The Clinical Grid (LettuceMeet/Doodle):** Spreadsheets feel like work. We must avoid grids that look like Excel.
*   **Authentication Walls:** Requiring a login or password reset explicitly breaks the magic and must be avoided.

### Design Inspiration Strategy

**What to Adopt:** The visual and auditory feedback loops of modern gaming lobbies to make the scheduling process feel like the start of a quest.
**What to Adapt:** Calendar and time-picking interactions must be simplified into large, D&D-themed "Battle Ready" buttons or fluid drag-to-paint actions.
**What to Avoid:** Corporate SaaS aesthetics; any UI that reminds the user of their 9-to-5 job.

---

## Design System Foundation

### 1.1 Design System Choice

**Themeable Utility System (Tailwind CSS)**

### Rationale for Selection

1.  **Visual Flexibility:** The MVP requires an "Arcane Tech" aesthetic (dark mode, glassmorphism, rich colors). A utility-first framework like Tailwind CSS provides the raw speed of a design system without enforcing a specific corporate look (unlike Material Design).
2.  **Performance:** The project has strict NFRs (<50ms interactive render times and aggressive mobile-first layouts). A utility CSS approach ensures a minimal payload by purging unused styles, keeping the application lightning fast.
3.  **Integration with Vanilla JS:** The frontend is utilizing Vanilla JS and Anime.js/p5.js for animations to achieve the "magical" feel. Bringing in heavy UI component libraries (like React-based MUI/Ant Design) could complicate this lightweight architecture. Tailwind acts as a powerful styling layer that doesn't dictate the logical framework.

### Implementation Approach

1.  **Tailwind Configuration:** Extend the default Tailwind theme to include custom "Arcane" color palettes (e.g., glowing golds, deep purples, parchment textures) and specialized typography.
2.  **Animation Interoperability:** Use CSS where possible for standard layout transitions, reserving Anime.js/p5.js exclusively for complex particle effects (e.g., the "Quorum Snap" golden mote burst).

### Customization Strategy

1.  **Global Base Layer:** Establish a custom "Dark Mode Default" baseline.
2.  **Utility Classes as Macros:** Standardize combined utility classes into reusable CSS components for common elements (e.g., `.btn-battle-ready`, `.glass-panel`) to maintain consistency in the HTML files.
3.  **Systemic Magic:** Integrate Anime.js targets with predefined Tailwind classes, ensuring elements can transition between stylistic states seamlessly.

---

## Visual Design Foundation

### Color System

The color system is rooted in the **"Arcane Tech" aesthetic**, avoiding corporate whites and blues in favor of high-contrast, immersive hues.
*   **Base (Backgrounds):** `Obsidian Dark` (`#0F172A` to `#0B0F19`)—providing a deep, cavernous backdrop that makes interactive elements pop while saving battery on OLED mobile screens.
*   **Interactive Primary:** `Arcane Purple` (`#8B5CF6`)—used for standard buttons and subtle hover states, providing a "magical" but neutral interactable color.
*   **Success state (The "Ready"):** `Golden Glow` (`#FBBF24` with intense box-shadow drops)—used exclusively when a player selects a time or when a Quorum is reached. It replaces traditional "Go Green."
*   **Alert/Warning state:** `Ember Red` (`#EF4444`)—used sparingly for delete actions or missed quorum deadlines.
*   **Text/Readability:** `Parchment White` (`#F8FAFC`)—for high contrast, avoiding stark `#FFFFFF` to reduce eye strain.

### Typography System

A dual-font strategy balances the fantasy theme with the rigid necessity of scheduling legibility.
*   **Primary Typeface (UI & Grids):** *Inter* or *Roboto* (Sans-serif). Used for all dates, times, buttons, and grid numbers. It ensures numbers are perfectly legible and tabular, vital for scanning a calendar on a small screen.
*   **Secondary Typeface (Headers & Seals):** *Cinzel* or *Playfair Display* (Serif). Used exclusively for the Campaign Title, the "Quest Confirmed" seal, and major headings. It provides the tabletop/fantasy anchor without interfering with utility reading.

### Spacing & Layout Foundation

The layout foundation must strictly adhere to an **Airy & Oversized** philosophy, optimizing for aggressive "thumb-first" mobile use.
*   **Oversized Hit Targets:** All interactive blocks (days/hours) must exceed standard mobile tap targets (minimum 48x48px, ideally 64px+). The user should never have to "aim" their finger carefully.
*   **Edge-to-Edge Grid:** To maximize screen real estate efficiently, the calendar grid should push toward the edges of the device, minimizing structural margins but maintaining heavy padding *within* the interactive cells.
*   **The "Bottom Sheet" Action Area:** Critical "Submit" or "Confirm" buttons should float near the thumb's natural resting place at the bottom of the screen.

### Accessibility Considerations

*   **Contrast Ratios:** The `Parchment White` text on `Obsidian Dark` backgrounds will easily clear WCAG AAA standards. 
*   **Color Independence:** While `Golden Glow` signifies success, the transition must also include structural state changes (e.g., a checkmark icon or a text label change from "Pending" to "Confirmed") to ensure it is accessible to colorblind users. 
*   **Focus States:** Since the DM uses Desktop, strong, high-contrast focus rings (likely `Golden Glow`) must be implemented for keyboard navigation through the grid.

<!-- UX design content will be appended sequentially through collaborative workflow steps -->
