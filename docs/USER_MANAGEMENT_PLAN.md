# User Management System - Implementation Plan

## Overview
Implement a comprehensive user authentication and management system to replace the current email-only approach.

## Database Schema Changes

### New Table: `users`
```sql
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    name TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    last_login INTEGER,
    is_verified BOOLEAN DEFAULT 0,
    verification_token TEXT,
    reset_token TEXT,
    reset_token_expires INTEGER
);
```

### New Table: `user_sessions`
```sql
CREATE TABLE user_sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    token TEXT NOT NULL UNIQUE,
    expires_at INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users (id)
);
```

### Update `participants` Table
```sql
ALTER TABLE participants ADD COLUMN user_id TEXT;
ALTER TABLE participants ADD FOREIGN KEY (user_id) REFERENCES users (id);
```

## Backend API Endpoints

### Authentication
- `POST /api/auth/register` - Register new user
- `POST /api/auth/login` - Login user
- `POST /api/auth/logout` - Logout user
- `POST /api/auth/refresh` - Refresh session token
- `POST /api/auth/forgot-password` - Request password reset
- `POST /api/auth/reset-password` - Reset password with token

### User Management
- `GET /api/user/profile` - Get current user profile
- `PUT /api/user/profile` - Update user profile
- `GET /api/user/polls` - Get all polls user is part of
- `GET /api/user/availability` - Get all user's availability submissions

## Frontend Pages

### New Pages
1. **`/register.html`** - User registration page
2. **`/login.html`** - User login page
3. **`/profile.html`** - User profile management
4. **`/forgot-password.html`** - Password reset request
5. **`/reset-password.html`** - Password reset form

### Updated Pages
1. **`/participate.html`** - Check if logged in, redirect to login if not
2. **`/index.html`** - Show user-specific dashboard
3. **Navigation** - Add login/logout, profile links

## Security Features

1. **Password Hashing** - bcrypt with salt
2. **Session Tokens** - Secure, expiring tokens
3. **CSRF Protection** - Token-based
4. **Rate Limiting** - Prevent brute force
5. **Email Verification** - Confirm email addresses
6. **Secure Cookies** - HttpOnly, Secure flags

## User Flow

### Registration Flow
```
1. User visits /register.html
2. Enters: name, email, password
3. Backend validates & creates user
4. Sends verification email (optional)
5. Auto-login or redirect to login
```

### Login Flow
```
1. User visits /login.html
2. Enters: email, password
3. Backend validates credentials
4. Creates session token
5. Stores token in localStorage/cookie
6. Redirects to dashboard
```

### Join Poll Flow (New)
```
1. User receives poll invitation email
2. Clicks link with poll_id
3. If not logged in → redirect to login
4. If logged in → auto-join poll
5. Redirect to availability page
```

## Implementation Steps

### Phase 1: Backend (Priority)
1. ✅ Create database migrations
2. ✅ Implement user models
3. ✅ Create auth handlers
4. ✅ Add session management
5. ✅ Update poll join logic

### Phase 2: Frontend (Priority)
1. ✅ Create registration page
2. ✅ Create login page
3. ✅ Add auth state management
4. ✅ Update navigation
5. ✅ Add protected routes

### Phase 3: Enhancements
1. ⏳ Email verification
2. ⏳ Password reset
3. ⏳ Profile management
4. ⏳ User dashboard
5. ⏳ Remember me functionality

## Benefits

### For Users
- ✅ No need to enter name/email every time
- ✅ Secure password-based authentication
- ✅ Persistent login across sessions
- ✅ Manage all polls in one place
- ✅ Profile customization

### For Developers
- ✅ Proper user tracking
- ✅ Better security
- ✅ Easier to add features
- ✅ Standard authentication flow
- ✅ Scalable architecture

## Migration Strategy

### Backward Compatibility
- Keep existing email-based join for legacy polls
- Gradually migrate users to accounts
- Offer "Create account" prompt after joining

### Data Migration
- Convert existing participants to users
- Generate temporary passwords
- Send migration emails

## Timeline

- **Phase 1 Backend**: 2-3 hours
- **Phase 2 Frontend**: 2-3 hours
- **Phase 3 Enhancements**: 4-6 hours
- **Total**: 8-12 hours

## Next Steps

1. Approve this plan
2. Start with Phase 1 (Backend)
3. Test authentication flow
4. Implement Phase 2 (Frontend)
5. Deploy and test
6. Add Phase 3 features incrementally

---

**Ready to start implementation?** 🚀
