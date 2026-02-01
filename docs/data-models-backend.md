# Data Models

## Schema Overview
The application uses SQLite as its primary data store. The schema is managed via `sqlx` migrations embedded in `src/db.rs`.

## Core Entities

### Polls (`polls`)
Stores D&D session planning instances.
- **id** (TEXT, PK): UUID
- **title** (TEXT): Campaign/Session title
- **description** (TEXT)
- **location** (TEXT)
- **created_at** (INTEGER): Unix timestamp
- **dates** (TEXT): JSON array of candidate dates
- **time_range** (TEXT): JSON configuration for time preferences
- **status** (TEXT): 'active', 'finalized', 'cancelled'
- **finalized_at** (INTEGER): Timestamp when finalized
- **finalized_time** (TEXT): Selected final time string
- **notes** (TEXT): Host notes

### Participants (`participants`)
Users or guests participating in a poll.
- **id** (TEXT, PK): UUID
- **poll_id** (TEXT, FK): Reference to `polls`
- **user_id** (TEXT, FK): Optional link to registered `users`
- **name** (TEXT)
- **email** (TEXT)
- **access_token** (TEXT): Token for updating availability without login

### Availability (`availability`)
Votes cast by participants.
- **id** (INTEGER, PK)
- **poll_id** (TEXT, FK)
- **participant_id** (TEXT, FK)
- **date** (TEXT): Date string (YYYY-MM-DD)
- **time_slot** (TEXT): Time slot string (e.g., "18:00")
- **status** (TEXT): 'available', 'tentative', 'busy'

### Users (`users`)
Registered application users (DMs and Players).
- **id** (TEXT, PK): UUID
- **email** (TEXT, Unique)
- **password_hash** (TEXT): Bcrypt hash or sentinel `"GOOGLE_OAUTH_USER"`
- **name** (TEXT)
- **role** (TEXT): 'dm' or 'player'
- **phone** (TEXT): Optional for WhatsApp reminders

### Admins (`admins`)
System administrators.
- **id** (TEXT, PK)
- **username** (TEXT)
- **password_hash** (TEXT)
- **email** (TEXT)
- **role** (TEXT): 'admin', 'superadmin'

## Supporting Tables

### Security (OWASP)
- **login_attempts**: Rate limiting and brute force protection log.
- **account_locks**: Temporary account lockouts.
- **audit_log**: Detailed security audit trail (who, what, when).
- **user_sessions**: Active session tokens (Hybrid: UUID + Expiry).

### GDPR & Privacy
- **consent_records**: Log of consent changes (timestamp, IP, type).
- **data_export_requests**: Tracking for "Right to Access" requests.

### Features
- **activities**: Event sourcing for the Activity Feed (e.g., "User X joined Poll Y").
