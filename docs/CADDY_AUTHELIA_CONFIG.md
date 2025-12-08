# Caddy Configuration for Authelia SSO

This document provides the Caddy configuration needed to enable Authelia ForwardAuth protection for the D&D Scheduler.

## Prerequisites

1. Authelia must be running at `auth.cronachednd.it`
2. The D&D Scheduler backend running on port 3000
3. Caddy serving as the reverse proxy

## Caddyfile Configuration

Add this to your Caddyfile:

```caddyfile
# D&D Scheduler - Protected by Authelia
scheduler.cronachednd.it {
    # ForwardAuth with Authelia
    forward_auth * {
        uri https://auth.cronachednd.it/api/authz/forward-auth
        copy_headers Remote-User Remote-Name Remote-Groups Remote-Email
    }
    
    # Reverse proxy to the scheduler backend
    reverse_proxy localhost:3000
    
    # TLS configuration (Let's Encrypt automatic)
    tls {
        dns cloudflare {env.CF_API_TOKEN}
    }
}
```

## Header Details

Authelia passes these headers after successful authentication:

| Header | Description | Example |
|--------|-------------|---------|
| `Remote-User` | User's email (primary identifier) | `user@cronachednd.it` |
| `Remote-Name` | User's display name | `Mario Rossi` |
| `Remote-Groups` | Comma-separated group list | `players,admins` |
| `Remote-Email` | Alternative email header | `user@cronachednd.it` |

## Authelia Configuration Updates

Add the scheduler domain to your Authelia `configuration.yml`:

```yaml
access_control:
  rules:
    # ... existing rules ...
    
    # D&D Scheduler - requires authentication
    - domain: 'scheduler.cronachednd.it'
      policy: 'one_factor'
      resources:
        - "^.*$"
```

## Session Cookie Configuration

Ensure your Authelia session cookies cover the subdomain:

```yaml
session:
  cookies:
    - domain: 'cronachednd.it'
      authelia_url: 'https://auth.cronachednd.it'
      default_redirection_url: 'https://scheduler.cronachednd.it'
      # ... other settings
```

## Testing the Integration

1. **Visit the scheduler**: Navigate to `https://scheduler.cronachednd.it`
2. **Verify redirect**: You should be redirected to `https://auth.cronachednd.it`
3. **Login**: Use your Authelia credentials
4. **Verify access**: After login, you should be redirected back to the scheduler
5. **Check user**: The scheduler should display your name with an "(SSO)" badge

## Troubleshooting

### User not showing in scheduler

Check that Authelia is passing the headers correctly:

```bash
curl -I -H "Cookie: authelia_session=..." https://scheduler.cronachednd.it/api/auth/authelia/session
```

### ForwardAuth failing

Check Caddy logs:

```bash
journalctl -u caddy -f
```

### Headers not being passed

Verify the `copy_headers` directive in Caddyfile includes all required headers.

## Security Notes

- The headers are only trustworthy when the request comes through Caddy with ForwardAuth
- Never expose the scheduler directly on port 3000 to the internet
- The `AUTHELIA_ENABLED=true` environment variable must be set for the backend to trust headers
