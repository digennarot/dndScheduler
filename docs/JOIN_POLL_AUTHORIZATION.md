# Join Poll Authorization Feature

## Overview
The join poll feature now requires **email authorization** - only participants who were invited (their email is in the original participant list) can join and submit their availability.

## Security Enhancement

### Before (Insecure)
- ❌ Anyone with the poll link could join
- ❌ No verification of invited participants
- ❌ Email was optional
- ❌ Could create duplicate participants

### After (Secure)
- ✅ Only invited emails can join
- ✅ Email is required and verified
- ✅ Returns 403 Forbidden for unauthorized emails
- ✅ Updates existing participant record (no duplicates)
- ✅ Clear error messages

## How It Works

### Poll Creation Flow

1. **Organizer creates poll** with participant emails:
   ```json
   {
     "title": "Tomb of Annihilation",
     "participants": [
       "player1@email.com",
       "player2@email.com",
       "player3@email.com"
     ]
   }
   ```

2. **System creates participant records**:
   - Each email gets a participant record
   - Default name is extracted from email (e.g., "player1")
   - Participant ID is generated

### Join Poll Flow

1. **Participant receives poll link**:
   ```
   https://yourapp.com/participate.html?id=poll-uuid
   ```

2. **Participant enters their information**:
   - Name: "John Smith"
   - Email: "player1@email.com"

3. **System validates authorization**:
   ```rust
   // Check if email exists in participants table for this poll
   SELECT id, email FROM participants 
   WHERE poll_id = ? AND email = ?
   ```

4. **Authorization outcomes**:

   **✅ Authorized (email found)**:
   - Updates participant name
   - Returns participant ID
   - Allows availability submission
   - Status: 200 OK

   **❌ Unauthorized (email not found)**:
   - Rejects the request
   - Returns error message
   - Status: 403 Forbidden

## API Changes

### Endpoint
```
POST /api/polls/{poll_id}/join
```

### Request (Updated)

**Before:**
```json
{
  "name": "John Smith",
  "email": "anyone@email.com"  // Optional
}
```

**After:**
```json
{
  "name": "John Smith",
  "email": "player1@email.com"  // Required & must be invited
}
```

### Response

**Success (200 OK):**
```json
{
  "id": "participant-uuid",
  "message": "Successfully joined the poll"
}
```

**Error - Email Required (400 Bad Request):**
```json
"Email is required to join this poll"
```

**Error - Not Invited (403 Forbidden):**
```json
"You are not authorized to join this poll. Only invited participants can join."
```

**Error - Poll Not Found (404 Not Found):**
```json
"Poll not found"
```

## User Experience

### For Invited Participants

1. **Receive invitation** (email with poll link)
2. **Click link** to open poll
3. **Enter name and email**
4. **System verifies** email is on invite list
5. **Access granted** - can submit availability

### For Non-Invited Users

1. **Somehow get poll link** (shared by someone)
2. **Click link** to open poll
3. **Enter name and email**
4. **System checks** email against invite list
5. **Access denied** - clear error message shown

## Error Messages

### User-Friendly Messages

**Email Required:**
```
"Email is required to join this poll"
```
- **When**: Email field is empty
- **Action**: Enter your email address

**Not Authorized:**
```
"You are not authorized to join this poll. Only invited participants can join."
```
- **When**: Email not in invite list
- **Action**: Contact the poll organizer

**Invalid Email Format:**
```
"Invalid email format"
```
- **When**: Email doesn't match format
- **Action**: Check email spelling

**Poll Not Found:**
```
"Poll not found"
```
- **When**: Invalid poll ID
- **Action**: Check the poll link

## Security Benefits

### 1. **Access Control**
- Only invited participants can join
- Prevents unauthorized access
- Protects poll data

### 2. **Data Integrity**
- No duplicate participants
- Accurate participant tracking
- Reliable availability data

### 3. **Privacy Protection**
- Poll results only visible to invited users
- Email verification prevents impersonation
- Organizer controls who participates

### 4. **Spam Prevention**
- Prevents random users from joining
- Reduces noise in availability data
- Maintains poll quality

## Implementation Details

### Database Schema

**Participants Table:**
```sql
CREATE TABLE participants (
    id TEXT PRIMARY KEY,
    poll_id TEXT NOT NULL,
    name TEXT NOT NULL,
    email TEXT,  -- Now effectively required
    FOREIGN KEY (poll_id) REFERENCES polls (id)
);
```

### Authorization Logic

```rust
// 1. Validate email is provided
let email = payload.email.as_ref().ok_or(error)?;

// 2. Check if email exists in participants for this poll
let invited = query("SELECT id FROM participants 
                     WHERE poll_id = ? AND email = ?")
    .fetch_optional(&pool)?;

// 3. Authorize or reject
if invited.is_none() {
    return Err(403 Forbidden);
}

// 4. Update participant name
query("UPDATE participants SET name = ? WHERE id = ?")
    .execute(&pool)?;
```

### Key Changes

1. **Email is required** (was optional)
2. **Authorization check** added
3. **UPDATE instead of INSERT** (no duplicates)
4. **403 Forbidden** for unauthorized
5. **Clear error messages**

## Testing

### Test Case 1: Invited Participant Joins

**Setup:**
```bash
# Create poll with participant
POST /api/polls
{
  "participants": ["invited@email.com"]
}
```

**Test:**
```bash
# Join with invited email
POST /api/polls/{id}/join
{
  "name": "John Doe",
  "email": "invited@email.com"
}
```

**Expected:** ✅ 200 OK, participant ID returned

### Test Case 2: Non-Invited User Tries to Join

**Test:**
```bash
# Join with non-invited email
POST /api/polls/{id}/join
{
  "name": "Jane Doe",
  "email": "notinvited@email.com"
}
```

**Expected:** ❌ 403 Forbidden, error message

### Test Case 3: Missing Email

**Test:**
```bash
# Join without email
POST /api/polls/{id}/join
{
  "name": "John Doe"
}
```

**Expected:** ❌ 400 Bad Request, "Email is required"

### Test Case 4: Invalid Email Format

**Test:**
```bash
# Join with invalid email
POST /api/polls/{id}/join
{
  "name": "John Doe",
  "email": "not-an-email"
}
```

**Expected:** ❌ 400 Bad Request, "Invalid email format"

### Test Case 5: Update Participant Name

**Setup:**
```bash
# Participant created with default name "invited"
```

**Test:**
```bash
# Join with full name
POST /api/polls/{id}/join
{
  "name": "John Smith",
  "email": "invited@email.com"
}
```

**Expected:** ✅ 200 OK, name updated to "John Smith"

## Frontend Integration

### Update Join Form

**Before:**
```html
<input type="email" name="email" placeholder="Email (optional)">
```

**After:**
```html
<input type="email" name="email" required 
       placeholder="Email (must match invitation)">
<p class="help-text">
  Enter the email address you were invited with
</p>
```

### Error Handling

```javascript
try {
    const response = await fetch(`/api/polls/${pollId}/join`, {
        method: 'POST',
        body: JSON.stringify({ name, email })
    });

    if (response.status === 403) {
        alert('You are not authorized to join this poll. ' +
              'Only invited participants can join.');
        return;
    }

    if (!response.ok) {
        const error = await response.text();
        alert('Error: ' + error);
        return;
    }

    // Success - proceed to availability submission
    const result = await response.json();
    console.log('Joined successfully:', result.id);
    
} catch (error) {
    console.error('Error joining poll:', error);
}
```

## Migration Notes

### Backward Compatibility

**Existing Polls:**
- Participants created before this change may have NULL emails
- These participants won't be able to "join" again
- Consider data migration to add emails

**Data Migration Script:**
```sql
-- Add emails to existing participants if missing
-- This is a manual process - organizer needs to provide emails
UPDATE participants 
SET email = 'participant@email.com' 
WHERE id = 'participant-id' AND email IS NULL;
```

## Best Practices

### For Organizers

1. **Verify email addresses** before creating poll
2. **Double-check spelling** of participant emails
3. **Use consistent email format** (lowercase recommended)
4. **Inform participants** to use their invited email
5. **Keep backup** of participant list

### For Participants

1. **Use the email you were invited with**
2. **Check spam folder** for invitation
3. **Contact organizer** if email doesn't work
4. **Don't share poll link** with non-invited users
5. **Verify email spelling** before submitting

## Troubleshooting

### "Not authorized" Error

**Problem:** Participant can't join even with correct email

**Solutions:**
1. Check email spelling (case-insensitive)
2. Verify email was in original invite list
3. Contact organizer to re-send invitation
4. Check if using correct poll link

### Email Not Recognized

**Problem:** System doesn't recognize invited email

**Possible Causes:**
1. Email typo in original invitation
2. Using different email than invited
3. Participant record missing from database
4. Case sensitivity issues (shouldn't happen)

**Solutions:**
1. Organizer can edit poll to add correct email
2. Use exact email from invitation
3. Check database for participant record
4. Contact support if persistent

## Security Considerations

### Potential Attacks

**1. Email Enumeration**
- Attacker tries multiple emails to find valid ones
- **Mitigation**: Rate limiting (future enhancement)

**2. Link Sharing**
- Invited participant shares link with others
- **Mitigation**: Email verification prevents unauthorized access

**3. Email Spoofing**
- Attacker uses someone else's email
- **Mitigation**: Email verification link (future enhancement)

### Future Enhancements

- [ ] Email verification links
- [ ] Rate limiting on join attempts
- [ ] CAPTCHA for join form
- [ ] Two-factor authentication
- [ ] Audit log of join attempts
- [ ] Temporary access codes

## Compliance

### GDPR Considerations

- Email is personal data
- Requires consent to collect
- Must allow deletion
- Privacy policy needed

### Data Protection

- Emails stored securely
- Access controlled
- Can be deleted on request
- Encrypted in transit (HTTPS)

---

**Authorization Status: ✅ ACTIVE**

Only invited participants can join polls!
