# Test Users & Email Mocking Guide

This guide explains how to use the improved test user system and email mocking functionality.

## Environment Configuration

### Admin Email Configuration

You can now customize the default admin email and password via environment variables:

```bash
# .env file
DEFAULT_ADMIN_EMAIL=admin@example.com
DEFAULT_ADMIN_PASSWORD=password123
```

These values are used when creating the default admin account during database initialization.

### Email Mocking

Enable email mocking to prevent real emails from being sent during development and testing:

```bash
# .env file
MOCK_EMAIL=true
```

When `MOCK_EMAIL=true`, all emails are logged to console instead of being sent via SMTP:

```
[MOCK EMAIL] To: user@example.com | Subject: Welcome! | Body: <html>...</html>
```

This is useful for:
- Local development
- Integration tests
- CI/CD pipelines
- Debugging email content

## Test User Helpers

### Creating Default Test Users

The easiest way to set up a complete test environment:

```rust
use crate::helpers::*;

#[tokio::test]
async fn my_test() {
    let pool = setup_test_db().await;
    let users = create_default_test_users(&pool).await;

    // Access pre-configured users:
    let admin_token = &users.admin.token;
    let dm_token = &users.dm.token;
    let player1_token = &users.player1.token;

    // All users have:
    // - id: unique user ID
    // - token: session token
    // - email: email address
    // - role: user role
}
```

Default users created:
- `admin@test.com` - admin role
- `dm@test.com` - dm role
- `player1@test.com` - player role
- `player2@test.com` - player role
- `player3@test.com` - player role

### Creating Custom Users

#### Quick Setup with TestUserConfig

```rust
use crate::helpers::TestUserConfig;

let configs = vec![
    TestUserConfig::player("gandalf@middleearth.com"),
    TestUserConfig::dm("aragorn@gondor.com"),
    TestUserConfig::admin("elrond@rivendell.com"),
];

let users = create_test_users(&pool, configs).await;
// Returns Vec<(String, String)> of (user_id, token) pairs
```

#### Fully Custom Configuration

```rust
let custom = TestUserConfig::new(
    "custom@example.com",
    "my_password",
    "Custom Name",
    "player"
);

let (user_id, token) = create_test_user_with_session(
    &pool,
    &custom.email,
    &custom.password,
    &custom.role,
).await;
```

### Creating Individual Users

For simpler cases:

```rust
// Create user without session
let user_id = create_test_user(
    &pool,
    "player@test.com",
    "password123",
    "player"
).await;

// Create user with session
let (user_id, token) = create_test_user_with_session(
    &pool,
    "player@test.com",
    "password123",
    "player"
).await;

// Create admin
let admin_id = create_test_admin(
    &pool,
    "admin@test.com",
    "admin_password"
).await;
```

## Example Test Scenarios

### Testing Role-Based Access Control

```rust
#[tokio::test]
async fn test_rbac() {
    let pool = setup_test_db().await;
    let users = create_default_test_users(&pool).await;

    // Test that only DM can create polls
    let dm_can_create = try_create_poll(&pool, &users.dm.token).await;
    assert!(dm_can_create.is_ok());

    let player_cannot_create = try_create_poll(&pool, &users.player1.token).await;
    assert!(player_cannot_create.is_err());
}
```

### Testing Multi-User Interactions

```rust
#[tokio::test]
async fn test_poll_participation() {
    let pool = setup_test_db().await;
    let users = create_default_test_users(&pool).await;

    // DM creates a poll
    let poll_id = create_poll_as_dm(&pool, &users.dm.token).await;

    // Multiple players join
    join_poll(&pool, poll_id, &users.player1.token).await;
    join_poll(&pool, poll_id, &users.player2.token).await;
    join_poll(&pool, poll_id, &users.player3.token).await;

    // Verify all players joined
    let participants = get_participants(&pool, poll_id).await;
    assert_eq!(participants.len(), 3);
}
```

### Testing Email Functionality

```rust
#[tokio::test]
async fn test_welcome_email() {
    // Set MOCK_EMAIL=true in your test environment
    std::env::set_var("MOCK_EMAIL", "true");

    let result = send_welcome_email("user@test.com", "Test User").await;

    // With mocking enabled, this will succeed without sending real email
    assert!(result.is_ok());
}
```

## Best Practices

1. **Use `create_default_test_users()` for full integration tests**
   - Provides complete user hierarchy
   - All users have valid sessions
   - Covers all common roles

2. **Use `TestUserConfig` for focused tests**
   - Create only the users you need
   - Customize emails to match your test scenario
   - Faster than creating full default set

3. **Enable MOCK_EMAIL in tests**
   - Add to your test setup or CI environment
   - Prevents accidental email sending
   - Speeds up tests

4. **Clean up after tests**
   ```rust
   cleanup_test_db(&pool).await;
   ```

## Common Patterns

### Creating a Campaign with Multiple Players

```rust
let pool = setup_test_db().await;
let users = create_default_test_users(&pool).await;

// DM creates campaign
let campaign_id = create_campaign(&users.dm.token, "Epic Quest").await;

// Add players to campaign
add_player_to_campaign(&campaign_id, &users.player1.id).await;
add_player_to_campaign(&campaign_id, &users.player2.id).await;
add_player_to_campaign(&campaign_id, &users.player3.id).await;
```

### Testing Admin Functions

```rust
let pool = setup_test_db().await;
let users = create_default_test_users(&pool).await;

// Admin-only operation
let result = delete_user(&pool, &users.admin.token, &users.player1.id).await;
assert!(result.is_ok());

// Non-admin should fail
let result = delete_user(&pool, &users.player2.token, &users.player1.id).await;
assert!(result.is_err());
```

## Migration Guide

If you have existing tests using the old system:

### Before:
```rust
let user_id = create_test_user(&pool, "test@test.com", "pass", "player").await;
let token = create_test_session(&pool, &user_id).await;
```

### After:
```rust
let (user_id, token) = create_test_user_with_session(
    &pool,
    "test@test.com",
    "pass",
    "player"
).await;
```

Or even better:
```rust
let users = create_default_test_users(&pool).await;
let player_token = &users.player1.token;
```

## Troubleshooting

### Emails Still Being Sent

Make sure `MOCK_EMAIL=true` is set in your environment:
```rust
std::env::set_var("MOCK_EMAIL", "true");
```

### User Creation Fails

Check that:
- Database schema is properly initialized
- Email addresses are unique
- All required fields are provided

### Session Token Issues

Verify:
- Session expiration times are valid
- Database has `user_sessions` table
- Token is being passed correctly in requests
