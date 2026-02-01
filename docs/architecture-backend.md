# Backend Architecture (Rust)

## Executive Summary
The backend is a high-performance, asynchronous REST API built with Rust and Axum. It serves dual purposes: providing JSON API endpoints for the frontend and serving static assets (HTML/JS/CSS) as a monolithic application. It prioritizes type safety, memory safety, and low overhead.

## Technology Stack
- **Language**: Rust (Edition 2021)
- **Framework**: `axum` (0.7) - Ergonomic and modular web framework.
- **Runtime**: `tokio` (1.0) - Asynchronous runtime.
- **Database**: SQLite with `sqlx` (0.8.2) - embedded SQL engine with compile-time checked queries.
- **Serialization**: `serde` / `serde_json`.
- **Logging**: `tracing` for structured logging.
- **Security**: `tower-http` (CORS), `tower_governor` (Rate Limiting), `bcrypt` (Passwords).

## Architecture Pattern
**Monolithic Service-Oriented Architecture**
The application runs as a single binary that manages its own database connection, state, and HTTP serving.

### Core Components

#### 1. Router (`main.rs`)
Defines the HTTP interface. Routes are grouped by functionality:
- `/auth`: Registration, Login, OAuth.
- `/polls`: Core domain logic.
- `/admin`: Administrative functions.
- `/`: Static file serving (fallback).

#### 2. Handlers (`handlers.rs`, `auth.rs`)
Pure async functions that take extractors (State, Json, Path) and return Responses.
- **Extraction**: Heavily uses Axum's type-safe extractors to validate input *before* handler logic runs.
- **State**: Database pool (`DbPool`) is shared via `State` extension.

#### 3. Data Layer (`db.rs`, `models.rs`)
- **SQLx**: Uses raw SQL queries with compile-time verification.
- **Schema**: Managed via "Embedded Migrations" - `init_db()` runs `CREATE TABLE IF NOT EXISTS` on startup.
- **Models**: Rust structs map 1:1 to database tables or request/response payloads.

#### 4. Authentication (`auth.rs`)
Implements a hybrid strategy:
- **Admins**: Stateless JWTs (JSON Web Tokens).
- **Users**: Database-backed sessions (secure random tokens) or Google ID Tokens.
- **Participants**: Capability URLs/Tokens (Access Token stored in `participants` table).

#### 5. Middleware Pipeline
Requests flow through a stack of Tower layers:
1. `GovernorLayer` (Rate Limiting) - Global protection.
2. `TraceLayer` - Logging and timing.
3. `CorsLayer` - Browser security.
4. `SecurityHeaders` - OWASP recommended headers.

## Development Workflow
- **Build**: `cargo build`
- **Run**: `cargo run` (Hot reload recommended via `cargo-watch`)
- **Test**: `cargo test` (Unit) or `./scripts/run_tests.sh` (Integration)

## Deployment Architecture
- **Single Binary**: Compiles to a standalone executable.
- **No External DB**: SQLite file lives alongside the binary.
- **Reverse Proxy**: Intended to run behind Nginx/Caddy for SSL termination.
