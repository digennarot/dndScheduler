# Source Tree Analysis

## Project Structure Overview
The project follows a **Monolithic** structure where the Rust backend serves the frontend as static assets.

```
dnd_scheduler/
├── src/                     # Backend Source Code (Rust)
│   ├── main.rs              # Application Entry Point & Router
│   ├── db.rs                # Database Connection & Migrations
│   ├── auth.rs              # Authentication Logic (Hybrid: Session/JWT)
│   ├── models.rs            # Data Structures (Rust Structs <-> SQL/JSON)
│   ├── handlers.rs          # REST API Endpoints (Polls, Participants)
│   ├── security.rs          # Security Middleware (Headers, Rate Limits)
│   └── ...                  # Feature modules (activity, gdpr, reminder)
├── static/                  # Frontend Assets (Served at /)
│   ├── index.html           # Landing Page
│   ├── dashboard.html       # User Dashboard
│   ├── create-poll.html     # Poll Creation Wizard
│   ├── js/                  # Vanilla JavaScript Logic
│   │   ├── app.js           # Main Entry Point & Routing
│   │   ├── auth-utils.js    # Auth State Management
│   │   ├── api-client.js    # (Implicit) API fetch wrappers
│   │   └── components/      # (Logical) UI Logic modules
│   ├── css/                 # Stylesheets (Tailwind + Custom)
│   └── resources/           # Images and Assets
├── docs/                    # Project Documentation
├── scripts/                 # Utility Scripts (Test runners, etc.)
├── tests/                   # Integration Tests
└── Cargo.toml               # Rust Dependencies & Config
```

## Critical Directories

### Backend (`src/`)
Contains the core business logic and API server.
- **`handlers.rs`**: The implementation of all API features. This is the primary place for business logic modification.
- **`auth.rs`**: Centralizes all security logic, including password hashing, token validation, and OAuth integration.
- **`db.rs`**: Manages the SQLite schema. **Note:** Schema changes here (via `sqlx::query`) serve as "migrations".

### Frontend (`static/`)
A Multi-Page Application (MPA) where each HTML file represents a distinct view.
- **`js/`**: Contains the application logic. Unlike modern bundlers, these scripts are often included directly in HTML files.
- **`auth-utils.js`**: Key file for managing the frontend authentication state (storing tokens, handling login flows).

### Documentation (`docs/`)
Extensive project documentation.
- `api-contracts-backend.md`: Generated API definition.
- `data-models-backend.md`: Generated Database Schema.
