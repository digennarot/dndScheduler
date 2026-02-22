# Frontend Architecture (Web)

## Executive Summary
The frontend is a lightweight, dependency-free Multi-Page Application (MPA). It uses Vanilla JavaScript (ES6+) for logic and Tailwind CSS for styling. It avoids heavy frameworks (React/Vue/Angular) in favor of direct DOM manipulation and browser-native APIs, resulting in extremely fast load times and simple maintenance.

## Technology Stack
- **Language**: JavaScript (ES6+)
- **Templating**: Plain HTML5
- **Styling**: Tailwind CSS (CDN) + Custom `dnd-theme.css`
- **Visuals**:
    - `anime.js`: Route transitions and element animations.
    - `p5.js`: Canvas-based particle backgrounds.
    - `echarts`: Data visualization for availability.
    - `splide`: Carousel interactions.

## Architecture Pattern
**Multi-Page Application (MPA) with Unidirectional Data Flow**

### Core Concepts

#### 1. Page Structure
Each view is a separate HTML file (`index.html`, `manage.html`).
- **Benefit**: Browser handles routing naturally. SEO friendly.
- **State**: Authentication state is stored in `localStorage` and re-hydrated on each page load via `auth-utils.js`.

#### 2. Component Logic (`js/`)
Logic is partitioned by feature, not by component framework.
- **Shared Utilities**: `auth-utils.js`, `api-client.js` (implicit pattern).
- **Page Controllers**: `dashboard.js`, `poll-creator.js` act as "Main" functions for their respective pages.

#### 3. Styling System
- **Utility-First**: Tailwind CSS handles 90% of layout and spacing.
- **Theme Layer**: `dnd-theme.css` defines the "D&D" aesthetic (fonts, colors, glow effects).
- **Dynamic**: JS toggles Tailwind classes to change state (e.g., hidden/visible, active/inactive).

#### 4. Authentication Integration
- **AuthManager**: A global singleton class (in `auth-utils.js`) that manages:
    - Token storage (`localStorage`).
    - Login/Logout flows.
    - User session validation (`/api/auth/me`).
    - Google Sign-In callbacks.

## Development Workflow
- **No Build Step**: Edit HTML/JS files and refresh the browser.
- **Assets**: Images and resources served from `static/resources/`.

## Testing
- **Manual**: Browser testing of user flows.
- **Responsiveness**: Tailwind ensures mobile compatibility; verified via browser dev tools.
