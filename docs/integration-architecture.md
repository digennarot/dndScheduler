# Integration Architecture

## Overview
This application uses a **Monolithic Service** pattern where the backend acts as both the API provider and the static asset server.

## Integration Points

### 1. Frontend <-> Backend (REST API)
- **Protocol**: HTTP/1.1 (JSON)
- **Direction**: Bidirectional (Request/Response)
- **Endpoints**: `/api/*`
- **Authentication**: 
  - Admin: Stateless JWT (Bearer Token)
  - Users: Hybrid Session (Cookie/Token)
  - Participants: Access Token (Query/Body)

#### Data Flow
1. User loads page (e.g., `index.html`) -> Served by Axum (`ServeDir` service).
2. JS loads -> Validates local token via `/api/auth/me`.
3. JS fetches data -> `GET /api/polls` -> Axum Handlers -> SQLite.
4. User action -> `POST /api/polls` -> Axum Handlers -> SQLite -> JSON Response.

### 2. Backend <-> Database (SQLite)
- **Protocol**: Native/Embedded (Zero-copy where possible)
- **Library**: `sqlx` (Async)
- **File**: `dnd_scheduler.db`
- **Concurrency**: Managed via WAL mode and connection pool.

### 3. External Services (Optional)
- **Google OAuth**: Backend validates ID Tokens directly with Google APIs.
- **SMTP**: Email notifications (if configured).
- **WhatsApp/Telegram**: Message delivery via external APIs (if configured).
