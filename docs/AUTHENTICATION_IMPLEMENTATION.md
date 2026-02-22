# User Authentication & Availability Tracking - Implementation Summary

## Problem Statement

The join session functionality was open to everyone without proper authentication, making it impossible to:
1. **Identify which user** joined a session
2. **Prevent unauthorized access** to availability submission
3. **Track who marked which time slots** (e.g., "11:00 AM Wednesday, Dec 31")
4. **Secure the availability data** from tampering

## Solution Overview

Implemented a **token-based authentication system** that:
- ✅ Generates unique access tokens for each participant
- ✅ Validates tokens when submitting availability
- ✅ Stores user identity securely across sessions
- ✅ Displays current user information in the UI
- ✅ Prevents unauthorized availability submissions

---

## Backend Changes

### 1. **Access Token Generation** (`src/handlers.rs`)

#### `join_poll` Handler
- **Modified**: Now returns the `access_token` along with participant ID
- **Purpose**: Frontend can store and use the token for authentication

```rust
// Fetch the access token for this participant
let access_token: Option<String> = sqlx::query_scalar("SELECT access_token FROM participants WHERE id = ?")
    .bind(&existing_participant_id)
    .fetch_optional(&pool)
    .await
    // ... error handling
    .flatten();

Ok(Json(json!({
    "id": existing_participant_id,
    "access_token": access_token,  // ← NEW: Return token to frontend
    "message": "Successfully joined the poll"
})))
```

### 2. **Token Validation** (`src/handlers.rs`)

#### `update_availability` Handler
- **Added**: Access token validation before allowing availability updates
- **Security**: Only users with valid tokens can submit availability

```rust
// AUTHORIZATION CHECK: Validate access token
if let Some(provided_token) = &payload.access_token {
    let stored_token: Option<String> = sqlx::query_scalar("SELECT access_token FROM participants WHERE id = ?")
        .bind(&participant_id)
        .fetch_optional(&pool)
        .await
        // ... error handling
        .flatten();

    if stored_token.is_none() || stored_token.as_ref() != Some(provided_token) {
        return Err((
            StatusCode::FORBIDDEN,
            "Invalid access token. You are not authorized to update this participant's availability.".to_string(),
        ));
    }
} else {
    return Err((
        StatusCode::UNAUTHORIZED,
        "Access token is required to update availability.".to_string(),
    ));
}
```

### 3. **Model Updates** (`src/models.rs`)

#### `UpdateAvailabilityRequest`
- **Added**: `access_token` field to the request payload

```rust
#[derive(Debug, Deserialize)]
pub struct UpdateAvailabilityRequest {
    pub availability: Vec<AvailabilityEntry>,
    pub access_token: Option<String>,  // ← NEW: Token for authentication
}
```

---

## Frontend Changes

### 1. **Token Storage** (`static/js/availability-manager.js`)

#### `joinSession` Method
- **Modified**: Stores the access token returned from the backend
- **Storage**: Saved in `localStorage` along with user data

```javascript
const data = await response.json();
this.currentUser.id = data.id;
this.currentUser.accessToken = data.access_token; // ← NEW: Store token
localStorage.setItem('currentUser', JSON.stringify(this.currentUser));
```

### 2. **Token Submission** (`static/js/availability-manager.js`)

#### `submitAvailability` Method
- **Modified**: Includes access token in the request payload
- **Validation**: Checks for token before submission

```javascript
if (!this.currentUser?.accessToken) {
    this.showNotification('Authorization Error', 'Access token missing. Please rejoin the session.', 'error');
    throw new Error('Access token missing');
}

const response = await fetch(`/api/polls/${this.selectedSession.id}/participants/${this.currentUser.id}/availability`, {
    method: 'POST',
    headers: {
        'Content-Type': 'application/json'
    },
    body: JSON.stringify({ 
        availability: availabilityList,
        access_token: this.currentUser.accessToken  // ← NEW: Include token
    })
});
```

### 3. **User Display** (`static/participate.html` & `availability-manager.js`)

#### Navigation Bar User Info
- **Added**: Visual indicator showing the currently logged-in user
- **Display**: Shows user's name, email, and initial

**HTML:**
```html
<div id="user-info-display" style="display: none;" class="flex items-center space-x-2 px-3 py-1 bg-emerald/10 rounded-lg border border-emerald/20">
    <div class="w-8 h-8 bg-gradient-to-br from-emerald to-mystic rounded-full flex items-center justify-center">
        <span class="text-white font-semibold text-sm" id="user-initial">U</span>
    </div>
    <div class="text-sm">
        <div class="font-semibold text-forest" id="user-display-name">User</div>
        <div class="text-xs text-gray-500" id="user-display-email">email@example.com</div>
    </div>
</div>
```

**JavaScript:**
```javascript
updateUserDisplay() {
    const userInfoDisplay = document.getElementById('user-info-display');
    if (!userInfoDisplay) return;

    if (this.currentUser) {
        document.getElementById('user-display-name').textContent = this.currentUser.name;
        document.getElementById('user-display-email').textContent = this.currentUser.email;
        document.getElementById('user-initial').textContent = this.currentUser.name.charAt(0).toUpperCase();
        userInfoDisplay.style.display = 'flex';
    } else {
        userInfoDisplay.style.display = 'none';
    }
}
```

---

## Security Flow

### 1. **User Joins Session**
```
User → Enter Name & Email → Backend validates email is invited
     → Backend generates/retrieves access_token
     → Frontend stores token in localStorage
     → UI displays user info
```

### 2. **User Marks Availability**
```
User → Marks time slots (e.g., "11:00 AM Wed, Dec 31")
     → Clicks "Submit Availability"
     → Frontend includes access_token in request
     → Backend validates token matches participant
     → If valid: Save availability
     → If invalid: Return 403 Forbidden
```

### 3. **Authorization Checks**
- **Email Verification**: Only invited emails can join (existing feature)
- **Token Validation**: Only users with valid tokens can submit availability (NEW)
- **Participant Matching**: Token must match the participant ID (NEW)

---

## Benefits

### 🔒 **Security**
- Prevents unauthorized users from submitting availability
- Each participant has a unique, non-guessable token
- Tokens are validated server-side before any data modification

### 👤 **User Identification**
- Clear visual indicator of who is logged in
- User info persists across page refreshes (localStorage)
- Easy to see which user is marking availability

### 📊 **Data Integrity**
- Each availability entry is tied to a specific participant
- Backend can track who marked which time slots
- Prevents users from submitting on behalf of others

### 🎯 **User Experience**
- Seamless login flow with email validation
- Visual feedback showing current user
- Clear error messages if token is missing/invalid

---

## Database Schema

The existing schema already supports this feature:

```sql
CREATE TABLE participants (
    id TEXT PRIMARY KEY,
    poll_id TEXT NOT NULL,
    name TEXT NOT NULL,
    email TEXT,
    access_token TEXT UNIQUE,  -- ← Used for authentication
    FOREIGN KEY (poll_id) REFERENCES polls (id)
);

CREATE TABLE availability (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    poll_id TEXT NOT NULL,
    participant_id TEXT NOT NULL,  -- ← Links to specific user
    date TEXT NOT NULL,
    time_slot TEXT NOT NULL,
    status TEXT NOT NULL,
    FOREIGN KEY (poll_id) REFERENCES polls (id),
    FOREIGN KEY (participant_id) REFERENCES participants (id)
);
```

---

## Testing the Feature

### 1. **Join a Session**
1. Navigate to `participate.html`
2. Select a session
3. Enter your name and email (must be invited)
4. Verify user info appears in navigation bar

### 2. **Mark Availability**
1. Click on time slots to mark availability
2. Example: Click "11:00 AM" on "Wed, Dec 31"
3. Click "Submit Availability"
4. Verify submission succeeds

### 3. **Test Authorization**
1. Clear localStorage (simulates token loss)
2. Try to submit availability
3. Verify error: "Access token is required"

### 4. **Test Invalid Token**
1. Manually edit localStorage to change token
2. Try to submit availability
3. Verify error: "Invalid access token"

---

## Future Enhancements

### 📧 **Email Notifications**
- Send access token via email when user is invited
- Include magic link for one-click access

### 🔐 **Token Expiration**
- Add expiration timestamps to tokens
- Require re-authentication after expiry

### 📊 **Admin View**
- Show which user marked which time slots
- Display availability by participant
- Export availability data with user names

### 🔄 **Session Management**
- Allow users to log out
- Support multiple sessions per user
- Remember user across different polls

---

## Error Messages

| Scenario | HTTP Status | Message |
|----------|-------------|---------|
| No token provided | 401 Unauthorized | "Access token is required to update availability." |
| Invalid token | 403 Forbidden | "Invalid access token. You are not authorized to update this participant's availability." |
| Email not invited | 403 Forbidden | "You are not authorized to join this poll. Only invited participants can join." |
| Token missing (frontend) | - | "Access token missing. Please rejoin the session." |

---

## Files Modified

### Backend
- ✅ `src/handlers.rs` - Added token return and validation
- ✅ `src/models.rs` - Added access_token field to UpdateAvailabilityRequest

### Frontend
- ✅ `static/js/availability-manager.js` - Token storage, submission, and UI updates
- ✅ `static/participate.html` - User info display in navigation

### Database
- ✅ No changes needed (schema already supports access_token)

---

## Summary

This implementation successfully addresses the original problem by:

1. **Identifying users**: Each user has a unique token tied to their participant record
2. **Securing submissions**: Only users with valid tokens can submit availability
3. **Tracking availability**: Each availability entry is linked to a specific participant
4. **Improving UX**: Users can see who they're logged in as

The system now ensures that when a user marks "11:00 AM Wednesday, Dec 31" as available, the backend knows exactly which user made that selection and can prevent unauthorized modifications.
