# Component Inventory (Frontend)

## Overview
This inventory tracks the logical components and pages that make up the frontend application.

## Pages (Views)
| File | Role | Description |
|------|------|-------------|
| `index.html` | Landing | Marketing hero, login entry points, feature summary |
| `dashboard.html` | User Hub | Shows user's active/joined polls and activity feed |
| `create-poll.html` | Wizard | Multi-step form for creating new sessions |
| `manage.html` | Admin/DM | Detailed view for poll owners to finalize times |
| `participate.html` | Participation | View for guests/users to vote on times |
| `admin.html` | System Admin | System-wide statistics and user management |
| `login.html` | Auth | Login form (Email/Password + Google) |
| `register.html` | Auth | Registration form |
| `profile.html` | User Settings | Profile management (password, standard settings) |

## JS Modules (Controllers)
| File | Responsibility |
|------|----------------|
| `app.js` | **Core**: Global routing logic and initialization |
| `auth-utils.js` | **Auth**: Token management, login/logout, Google callbacks |
| `dashboard.js` | **Logic**: Fetches and renders user-specific poll lists |
| `poll-creator.js` | **Logic**: Manages the create-poll wizard state |
| `particles.js` | **Visuals**: Canvas-based background animation system |
| `statistics.js` | **Visuals**: ECharts wrapper for dashboard analytics |
| `cookie-consent.js`| **Compliance**: GDPR consent banner logic |

## Reusable UI Elements (Logical)
While not using a component framework, these patterns are reused:
- **Navbar**: Present on every page (managed via HTML inclusion or copy-paste pattern).
- **Cards**: "Glassmorphism" card style defined `dnd-theme.css`.
- **Modals**: Standardized modal structure (hidden/block toggle).
