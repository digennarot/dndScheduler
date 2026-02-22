# Authentication System Implementation

## Overview

The D&D Session Scheduler now has a complete authentication system that protects sensitive pages and ensures only logged-in users can create polls and access certain features.

## Key Components

### 1. Backend Authentication (`src/auth.rs`)
- **Registration**: `/api/auth/register` - Create new user accounts with secure password hashing
- **Login**: `/api/auth/login` - Authenticate users and create sessions
- **Logout**: `/api/auth/logout/:token` - Invalidate user sessions
- **Get Current User**: `/api/auth/me/:token` - Retrieve current user information
- **Session Validation**: Helper function for middleware (future use)

### 2. Frontend Authentication Utility (`static/js/auth-utils.js`)

A centralized authentication manager that handles:

#### Token Management
- Stores session tokens in `localStorage` under key `dnd_session_token`
- Stores user data in `localStorage` under key `dnd_user_data`
- Automatic token cleanup on logout

#### Authentication Methods

**`isAuthenticated()`**
- Quick check if user has a token
- Returns boolean

**`verifySession()`**
- Validates token with server
- Updates user data if valid
- Clears local data if invalid
- Returns boolean

**`requireAuth(redirectUrl)`**
- **Critical for protected pages**
- Verifies session with server
- Redirects to login if not authenticated
- Stores return URL for post-login redirect
- Returns boolean

**`login(email, password)`**
- Authenticates user with credentials
- Stores token and user data
- Throws error on failure

**`register(name, email, password)`**
- Creates new user account
- Automatically logs in on success
- Stores token and user data

**`logout()`**
- Invalidates server session
- Clears local storage
- Redirects to login page

**`updateNavbar()`**
- Dynamically updates navigation menu
- Shows user name and logout button when authenticated
- Shows login/signup buttons when not authenticated

**`authenticatedFetch(url, options)`**
- Makes API requests with authentication header
- Automatically handles 401 responses
- Logs out user if session expired

## Protected Pages

### Pages Requiring Authentication

The following pages now require users to be logged in:

1. **`create-poll.html`** ✅ - Poll creation wizard
2. **`manage.html`** (recommended) - Poll management
3. **`dashboard.html`** (recommended) - User dashboard
4. **`profile.html`** (recommended) - User profile

### Public Pages

These pages remain accessible without authentication:

1. **`index.html`** - Landing page
2. **`login.html`** - Login form
3. **`register.html`** - Registration form
4. **`participate.html`** - Join session (uses invitation tokens)

## How to Protect a Page

To require authentication on any HTML page:

### Step 1: Include the auth-utils script
```html
<head>
    <!-- Other scripts -->
    <script src="js/auth-utils.js"></script>
</head>
```

### Step 2: Add user menu to navigation
```html
<nav>
    <!-- Other nav items -->
    <div id="user-menu" class="flex items-center space-x-4">
        <!-- Will be populated by auth-utils.js -->
    </div>
</nav>
```

### Step 3: Add authentication check
```html
<script>
    document.addEventListener('DOMContentLoaded', async () => {
        const isAuthenticated = await authManager.requireAuth('/current-page.html');
        if (!isAuthenticated) {
            return; // User will be redirected to login
        }
        
        // Page-specific initialization code here
    });
</script>
```

## User Flow

### Registration Flow
1. User visits `/register.html`
2. Fills out registration form (name, email, password)
3. Password must meet requirements:
   - Minimum 12 characters
   - At least one uppercase letter
   - At least one lowercase letter
   - At least one number
   - At least one special character
   - Not a common password
4. On success, user is automatically logged in
5. Redirected to dashboard

### Login Flow
1. User visits `/login.html` (or is redirected from protected page)
2. Enters email and password
3. On success, session token is stored
4. User is redirected to:
   - Original requested page (if redirected from protected page)
   - Dashboard (if direct login)

### Logout Flow
1. User clicks logout button in navigation
2. Session is invalidated on server
3. Local storage is cleared
4. User is redirected to login page

### Protected Page Access
1. User tries to access protected page (e.g., `/create-poll.html`)
2. `requireAuth()` checks for valid session
3. If valid: Page loads normally
4. If invalid: User redirected to login with return URL stored
5. After login: User automatically redirected back to original page

## Security Features

### Password Requirements (OWASP Compliant)
- Minimum 12 characters (exceeds OWASP minimum of 8)
- Complexity requirements enforced
- Common password detection
- Secure bcrypt hashing with default cost

### Session Management
- 7-day session duration
- Server-side session validation
- Automatic cleanup of expired sessions
- Secure token generation using UUIDs

### Input Validation
- Email format validation
- Email length limits (max 254 characters)
- Name length limits (max 100 characters)
- Password length limits (max 128 characters)
- SQL injection prevention via parameterized queries
- XSS prevention via HTML escaping

### Protection Against Attacks
- **Timing Attacks**: Constant-time password comparison
- **SQL Injection**: Parameterized queries throughout
- **XSS**: HTML escaping in user-facing content
- **CSRF**: Token-based authentication (stateless)
- **Session Fixation**: New session on each login

## API Endpoints

### POST `/api/auth/register`
**Request:**
```json
{
    "name": "John Doe",
    "email": "john@example.com",
    "password": "SecurePass123!"
}
```

**Response (200):**
```json
{
    "token": "uuid-session-token",
    "user": {
        "id": "uuid-user-id",
        "email": "john@example.com",
        "name": "John Doe",
        "created_at": 1234567890
    }
}
```

### POST `/api/auth/login`
**Request:**
```json
{
    "email": "john@example.com",
    "password": "SecurePass123!"
}
```

**Response (200):**
```json
{
    "token": "uuid-session-token",
    "user": {
        "id": "uuid-user-id",
        "email": "john@example.com",
        "name": "John Doe",
        "created_at": 1234567890
    }
}
```

### POST `/api/auth/logout/:token`
**Response (200):**
```
OK
```

### GET `/api/auth/me/:token`
**Response (200):**
```json
{
    "id": "uuid-user-id",
    "email": "john@example.com",
    "name": "John Doe",
    "created_at": 1234567890
}
```

## Testing the Authentication

### Manual Testing Steps

1. **Test Registration:**
   ```
   1. Navigate to http://127.0.0.1:3000/register.html
   2. Fill in name, email, and password
   3. Submit form
   4. Verify redirect to dashboard
   5. Check that navbar shows user name
   ```

2. **Test Login:**
   ```
   1. Logout if logged in
   2. Navigate to http://127.0.0.1:3000/login.html
   3. Enter credentials
   4. Submit form
   5. Verify redirect to dashboard
   ```

3. **Test Protected Page:**
   ```
   1. Logout if logged in
   2. Navigate to http://127.0.0.1:3000/create-poll.html
   3. Verify redirect to login page
   4. Login with valid credentials
   5. Verify redirect back to create-poll.html
   ```

4. **Test Logout:**
   ```
   1. Login if not logged in
   2. Click logout button in navbar
   3. Verify redirect to login page
   4. Try accessing protected page
   5. Verify redirect to login (session cleared)
   ```

## Database Schema

### Users Table
```sql
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    name TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    last_login INTEGER
);
```

### User Sessions Table
```sql
CREATE TABLE user_sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    token TEXT UNIQUE NOT NULL,
    expires_at INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
```

## Future Enhancements

1. **Email Verification**: Send confirmation emails on registration
2. **Password Reset**: Implement forgot password flow
3. **Two-Factor Authentication**: Add 2FA support
4. **OAuth Integration**: Google/Discord login
5. **Session Management UI**: View and revoke active sessions
6. **Role-Based Access Control**: Admin vs regular users
7. **API Key Authentication**: For programmatic access
8. **Rate Limiting**: Prevent brute force attacks

## Troubleshooting

### User can't login
- Check password meets requirements
- Verify email is correct
- Check server logs for errors
- Ensure database is accessible

### Redirect loop
- Clear browser localStorage
- Check `requireAuth()` is not called on login page
- Verify return URL logic

### Session expires too quickly
- Check `SESSION_DURATION_HOURS` constant in `auth.rs`
- Verify server time is correct
- Check for clock skew

### Navbar doesn't update
- Ensure `auth-utils.js` is loaded
- Check for JavaScript errors in console
- Verify `user-menu` div exists in HTML

## Summary

The authentication system is now fully functional and provides:
- ✅ Secure user registration and login
- ✅ Session management with automatic expiration
- ✅ Protected pages with automatic redirects
- ✅ Clean user interface with dynamic navigation
- ✅ OWASP-compliant security practices
- ✅ Centralized authentication logic
- ✅ Easy-to-use API for developers

Users can no longer access the poll creation page without logging in, solving the original issue of illogical access control.
