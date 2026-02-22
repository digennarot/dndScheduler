# 🚀 Production Deployment Checklist

## Pre-Deployment

### 1. Environment Variables (.env)
```bash
# ✅ Set Real Admin Credentials
DEFAULT_ADMIN_EMAIL=your-real-email@company.com
DEFAULT_ADMIN_PASSWORD=YourSecurePassword123!

# ✅ Disable Email Mocking
MOCK_EMAIL=false

# ✅ Set Real SMTP Config
SMTP_HOST=smtp.your-provider.com
SMTP_PORT=587
SMTP_USERNAME=your-smtp-user
SMTP_PASSWORD=your-smtp-password
SMTP_FROM_EMAIL=noreply@your-domain.com
```

### 2. Security

- [ ] **Remove or Protect Debug Page**
  ```bash
  # Option 1: Delete it
  rm static/clear-cache.html

  # Option 2: Add to .gitignore
  echo "static/clear-cache.html" >> .gitignore
  ```

- [ ] **Enable HTTPS**
  - Use reverse proxy (nginx/Apache)
  - Configure SSL certificates
  - Force HTTPS redirect

- [ ] **Set Secure Headers**
  ```nginx
  # nginx example
  add_header X-Frame-Options "SAMEORIGIN";
  add_header X-Content-Type-Options "nosniff";
  add_header X-XSS-Protection "1; mode=block";
  ```

### 3. Database

- [ ] **Backup Current Database**
  ```bash
  cp dnd_scheduler.db dnd_scheduler.db.backup
  ```

- [ ] **Clear Old Test Data**
  ```sql
  DELETE FROM users WHERE email LIKE '%@test.com';
  DELETE FROM admins WHERE email = 'admin@example.com';
  ```

- [ ] **Verify Default Admin**
  ```bash
  sqlite3 dnd_scheduler.db "SELECT email FROM admins;"
  ```

### 4. Build & Test

- [ ] **Run Tests**
  ```bash
  cargo test
  ```

- [ ] **Build Release**
  ```bash
  cargo build --release
  ```

- [ ] **Test Locally**
  ```bash
  ./target/release/your-app-name
  # Open http://localhost:3000/admin.html
  # Login and verify email displays correctly
  ```

---

## Deployment Steps

### 1. Upload Files
```bash
# Copy to server
scp -r ./target/release/your-app user@server:/opt/app/
scp -r ./static user@server:/opt/app/
scp .env user@server:/opt/app/
```

### 2. Start Service
```bash
# SSH into server
ssh user@server

# Navigate to app directory
cd /opt/app

# Start application
./your-app-name
```

### 3. Verify Deployment
```bash
# Test API endpoint
curl https://your-domain.com/api/polls

# Test admin page
curl https://your-domain.com/admin.html
```

---

## Post-Deployment Verification

### 1. Test Admin Login
- [ ] Go to `https://your-domain.com/admin.html`
- [ ] Login with real credentials
- [ ] Verify **YOUR** email shows, not "admin@example.com"
- [ ] Check browser console for errors (should be none)

### 2. Test Token Validation
- [ ] Login successfully
- [ ] Wait 5 minutes
- [ ] Check Network tab for `/api/admin/me` calls
- [ ] Should see periodic validation requests

### 3. Test Security
- [ ] Try accessing `/clear-cache.html` → Should be blocked/404
- [ ] Try accessing admin pages without login → Should redirect
- [ ] Test HTTPS redirect works
- [ ] Verify secure headers are set

### 4. Test Email
- [ ] Create test poll
- [ ] Add participant with real email
- [ ] Verify email is sent (not mocked)
- [ ] Check SMTP logs if needed

---

## Monitoring Setup

### 1. Log Monitoring
```bash
# View application logs
tail -f /var/log/your-app.log

# Watch for errors
grep "ERROR" /var/log/your-app.log
```

### 2. Database Monitoring
```bash
# Check database size
ls -lh dnd_scheduler.db

# Monitor active sessions
sqlite3 dnd_scheduler.db "SELECT COUNT(*) FROM sessions;"

# Check failed logins
sqlite3 dnd_scheduler.db "SELECT * FROM login_attempts WHERE success = 0 ORDER BY attempt_time DESC LIMIT 10;"
```

### 3. Performance Monitoring
- [ ] Set up uptime monitoring (e.g., UptimeRobot)
- [ ] Configure error alerting
- [ ] Monitor response times
- [ ] Track resource usage (CPU, RAM, disk)

---

## Rollback Plan

If something goes wrong:

### Quick Rollback
```bash
# Stop current service
systemctl stop your-app

# Restore backup
cp dnd_scheduler.db.backup dnd_scheduler.db

# Revert to previous version
cp /backup/previous-version /opt/app/

# Restart
systemctl start your-app
```

### Database Rollback
```sql
-- Restore specific admin
DELETE FROM admins WHERE email = 'wrong@email.com';
INSERT INTO admins (id, username, password_hash, email, role, created_at)
VALUES ('uuid-here', 'admin', 'hash-here', 'correct@email.com', 'superadmin', 1234567890);
```

---

## Common Issues & Solutions

### Issue: "admin@example.com" still showing

**Solution**:
1. Clear browser cache completely
2. Open `/clear-cache.html` (if accessible)
3. Clear localStorage
4. Logout and login again

### Issue: Auto-logout after a few minutes

**Cause**: Token validation failing

**Solution**:
1. Check server is running
2. Verify `/api/admin/me` endpoint works:
   ```bash
   curl -H "Authorization: Bearer your-token" https://your-domain.com/api/admin/me
   ```
3. Check server logs for errors

### Issue: CORS errors

**Solution**:
Add CORS configuration in server:
```rust
// In main.rs
let cors = CorsLayer::new()
    .allow_origin("https://your-domain.com".parse::<HeaderValue>().unwrap())
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);
```

### Issue: SSL/HTTPS not working

**Solution**:
1. Verify certificates are valid
2. Check nginx/Apache configuration
3. Test with `curl -I https://your-domain.com`

---

## Maintenance Tasks

### Daily
- [ ] Check error logs
- [ ] Monitor active sessions
- [ ] Verify email delivery

### Weekly
- [ ] Backup database
- [ ] Review audit logs
- [ ] Check failed login attempts
- [ ] Update dependencies if needed

### Monthly
- [ ] Rotate logs
- [ ] Clean up expired sessions
- [ ] Review security patches
- [ ] Performance optimization

---

## Emergency Contacts

- **Server Access**: [admin credentials location]
- **Database Backup**: [backup location]
- **SSL Certificates**: [certificate provider]
- **SMTP Provider**: [email service details]

---

## Success Criteria

Deployment is successful when:

- ✅ Admin login works with real credentials
- ✅ Real email displays (not "admin@example.com")
- ✅ Token validation works (5-minute checks)
- ✅ HTTPS enabled and working
- ✅ Email notifications sent (not mocked)
- ✅ No errors in console or logs
- ✅ Database properly initialized
- ✅ All security headers set
- ✅ Debug pages protected/removed

---

**Deployment Date**: _____________
**Deployed By**: _____________
**Server**: _____________
**Domain**: _____________
**Status**: ⬜ SUCCESS / ⬜ FAILED / ⬜ PARTIAL

---

## Notes

_Add any deployment-specific notes here..._
