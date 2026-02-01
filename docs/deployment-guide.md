# Deployment Guide

## Infrastructure Requirements
- **OS**: Linux (recommended)
- **Database**: SQLite (Embedded, no external server needed)
- **Reverse Proxy**: Nginx or Apache (for HTTPS)
- **Process Manager**: Systemd or Docker

## Deployment Process

### 1. Build
```bash
# Create optimization release build
cargo build --release
```
The binary will be located at `./target/release/dnd_scheduler`.

### 2. Packaging
You need to transfer the following artifacts to your production server:
- Binary: `target/release/dnd_scheduler`
- Assets: `static/` directory
- Config: `.env` file

### 3. Server Configuration (Nginx Example)
```nginx
server {
    listen 80;
    server_name your-domain.com;
    return 301 https://$host$request_uri;
}

server {
    listen 443 ssl http2;
    server_name your-domain.com;

    # SSL Config...

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

## Environment Variables
Ensure these are set in your production `.env`:
```bash
DEFAULT_ADMIN_EMAIL=admin@your-domain.com
DEFAULT_ADMIN_PASSWORD=StrongPassword
MOCK_EMAIL=false
SMTP_HOST=smtp.provider.com
SMTP_PORT=587
SMTP_USERNAME=user
SMTP_PASSWORD=pass
```

## Maintenance
- **Backups**: Periodically backup the `dnd_scheduler.db` file.
- **Updates**: Replace the binary and restart the service. DB migrations run automatically on startup.
- **Logs**: Monitor standard output (stdout/stderr) or configure a logging service.

## Verification
See `docs/DEPLOYMENT_CHECKLIST.md` for a comprehensive pre-flight verification checklist.
