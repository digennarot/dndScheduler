# Participant Authentication & Authorization Plan

## Overview
This document outlines the token-based authentication system for restricting poll availability submission to invited participants only.

## Current Status ✅

### Backend Changes (Completed)
1. **Database Schema** (`src/db.rs`)
   - Added `access_token` column to `participants` table
   - Added migration to update existing tables

2. **Data Model** (`src/models.rs`)
   - Updated `Participant` struct to include `access_token: Option<String>`

3. **Token Generation** (`src/handlers.rs`)
   - Modified `create_poll()` to generate unique UUID tokens for each participant
   - Tokens are stored in database when participants are created

4. **Existing Authorization** (`src/handlers.rs`)
   - `join_poll()` already validates that only invited emails can join (lines 279-300)
   - Prevents unauthorized users from joining polls

## What Still Needs to Be Done 🚧

### 1. Frontend: Token-Based Access Flow

#### A. Update Participate Page URL Structure
**File:** `static/participate.html` and `static/js/availability-manager.js`

**Current:** `/participate.html?poll=<poll_id>`
**New:** `/participate.html?poll=<poll_id>&token=<access_token>`

**Changes Needed:**
```javascript
// In availability-manager.js
selectSession(pollId) {
    // Extract token from URL
    const urlParams = new URLSearchParams(window.location.search);
    const token = urlParams.get('token');
    
    if (!token) {
        this.showError('Access Denied', 'You need a valid invitation link to access this poll.');
        return;
    }
    
    // Store token for API calls
    this.accessToken = token;
    
    // Validate token with backend before showing interface
    this.validateTokenAndLoadPoll(pollId, token);
}
```

#### B. Add Token Validation Endpoint
**File:** `src/handlers.rs`

Create new endpoint:
```rust
pub async fn validate_participant_token(
    State(pool): State<DbPool>,
    Path((poll_id, token)): Path<(String, String)>,
) -> Result<Json<Value>, (StatusCode, String)> {
    // Validate poll_id and token format
    validate_uuid(&poll_id)?;
    validate_uuid(&token)?;
    
    // Check if token exists and belongs to this poll
    let participant: Option<Participant> = sqlx::query_as(
        "SELECT * FROM participants WHERE poll_id = ? AND access_token = ?"
    )
    .bind(&poll_id)
    .bind(&token)
    .fetch_optional(&pool)
    .await?;
    
    match participant {
        Some(p) => Ok(Json(json!({
            "valid": true,
            "participant_id": p.id,
            "name": p.name
        }))),
        None => Err((StatusCode::FORBIDDEN, "Invalid or expired token".to_string()))
    }
}
```

**Route:** Add to `src/main.rs`:
```rust
.route("/api/polls/:poll_id/validate/:token", get(validate_participant_token))
```

#### C. Update Availability Submission to Require Token
**File:** `src/handlers.rs` - `update_availability()`

Add token validation before allowing availability updates:
```rust
pub async fn update_availability(
    State(pool): State<DbPool>,
    Path((poll_id, participant_id)): Path<(String, String)>,
    Query(params): Query<HashMap<String, String>>, // Add this
    Json(payload): Json<UpdateAvailabilityRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Validate token
    let token = params.get("token").ok_or((
        StatusCode::UNAUTHORIZED,
        "Access token required".to_string()
    ))?;
    
    // Verify token belongs to this participant
    let valid: Option<(i64,)> = sqlx::query_as(
        "SELECT 1 FROM participants WHERE id = ? AND poll_id = ? AND access_token = ?"
    )
    .bind(&participant_id)
    .bind(&poll_id)
    .bind(token)
    .fetch_optional(&pool)
    .await?;
    
    if valid.is_none() {
        return Err((StatusCode::FORBIDDEN, "Invalid access token".to_string()));
    }
    
    // ... rest of existing code ...
}
```

#### D. Update Frontend API Calls
**File:** `static/js/availability-manager.js`

Update `submitAvailability()` to include token:
```javascript
async submitAvailability() {
    if (!this.accessToken) {
        this.showNotification('Access Denied', 'Invalid session. Please use your invitation link.', 'error');
        return;
    }
    
    const response = await fetch(
        `/api/polls/${this.selectedSession.id}/participants/${this.currentUser.id}/availability?token=${this.accessToken}`,
        {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ availability: availabilityList })
        }
    );
    
    // ... handle response ...
}
```

### 2. Invitation Link Generation

#### A. Create Invitation Links in Manage Page
**File:** `static/js/session-manager.js`

Add method to generate and display invitation links:
```javascript
showInvitationLinks() {
    if (!this.selectedSession) return;
    
    // Fetch participants with tokens
    fetch(`/api/polls/${this.selectedSession.id}`)
        .then(r => r.json())
        .then(data => {
            const links = data.participants.map(p => ({
                name: p.name,
                email: p.email,
                link: `${window.location.origin}/participate.html?poll=${this.selectedSession.id}&token=${p.access_token}`
            }));
            
            this.displayInvitationModal(links);
        });
}

displayInvitationModal(links) {
    // Create modal with copyable links for each participant
    // Include "Copy Link" and "Send Email" buttons
}
```

#### B. Add "Share Invitations" Button to Manage Page
**File:** `static/manage.html`

Add button in session detail view:
```html
<button onclick="sessionManager.showInvitationLinks()" 
        class="action-button secondary">
    📧 Share Invitation Links
</button>
```

### 3. Security Enhancements

#### A. Remove Open Join Functionality
**File:** `static/js/availability-manager.js`

Remove or restrict the `joinSession()` method:
```javascript
// OLD: Anyone could join
async joinSession(pollId) {
    // This allowed unauthorized access
}

// NEW: Only allow if they have a valid token
selectSession(pollId) {
    const token = new URLSearchParams(window.location.search).get('token');
    if (!token) {
        this.showAccessDeniedMessage();
        return;
    }
    // Proceed with token validation
}
```

#### B. Hide Session List for Unauthenticated Users
**File:** `static/participate.html`

Update to show different UI based on whether user has a token:
```javascript
init() {
    const urlParams = new URLSearchParams(window.location.search);
    const pollId = urlParams.get('poll');
    const token = urlParams.get('token');
    
    if (pollId && token) {
        // Direct access with invitation link
        this.validateAndShowPoll(pollId, token);
    } else {
        // Show message: "Please use your invitation link to access polls"
        this.showInvitationRequiredMessage();
    }
}
```

### 4. Email Integration (Future Enhancement)

When creating a poll, send invitation emails:
```javascript
// In poll-creator.js after poll creation
async sendInvitationEmails(pollId, participants) {
    await fetch(`/api/polls/${pollId}/send-invitations`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' }
    });
}
```

Backend endpoint:
```rust
pub async fn send_invitations(
    State(pool): State<DbPool>,
    Path(poll_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Get poll and participants
    // For each participant:
    //   - Generate invitation email with link containing their token
    //   - Send via email service (SendGrid, AWS SES, etc.)
}
```

## Implementation Priority

1. **High Priority** (Implement First):
   - [ ] Add token validation endpoint
   - [ ] Update availability submission to require token
   - [ ] Modify participate.html to extract and use token from URL
   - [ ] Remove open session list (require token to view polls)

2. **Medium Priority**:
   - [ ] Add invitation link generation in manage page
   - [ ] Create shareable link modal
   - [ ] Add copy-to-clipboard functionality

3. **Low Priority** (Future):
   - [ ] Email integration for automatic invitation sending
   - [ ] Token expiration/refresh mechanism
   - [ ] Revoke/regenerate token functionality

## Testing Checklist

- [ ] Create a poll with participants
- [ ] Verify tokens are generated in database
- [ ] Try accessing poll without token (should fail)
- [ ] Try accessing poll with invalid token (should fail)
- [ ] Access poll with valid token (should succeed)
- [ ] Try submitting availability without token (should fail)
- [ ] Submit availability with valid token (should succeed)
- [ ] Verify non-invited users cannot access poll

## Security Considerations

1. **Token Storage**: Tokens are UUIDs stored in database, not exposed in API responses except when needed
2. **HTTPS Required**: In production, ensure all traffic uses HTTPS to prevent token interception
3. **Token Rotation**: Consider implementing token expiration and refresh for long-running polls
4. **Rate Limiting**: Add rate limiting to validation endpoint to prevent brute force attacks
5. **Audit Logging**: Log all token validation attempts for security monitoring

## Migration Notes

For existing polls without tokens:
```sql
-- Generate tokens for existing participants
UPDATE participants 
SET access_token = lower(hex(randomblob(16)))
WHERE access_token IS NULL;
```

Or in Rust during app startup:
```rust
// In init_db() after migration
sqlx::query("UPDATE participants SET access_token = ? WHERE access_token IS NULL")
    .bind(Uuid::new_v4().to_string())
    .execute(&pool)
    .await?;
```
