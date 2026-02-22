# Quick Reference: User Authentication & Availability Tracking

## 🎯 What Changed?

The join session feature now **identifies and authenticates users** so you can track who marked which time slots.

---

## 🔑 Key Features

### 1. **Secure User Identification**
- Each user gets a unique access token when they join
- Token is stored securely in the browser
- Only invited users can join (email must be in the participant list)

### 2. **Visual User Display**
- User's name and email appear in the navigation bar
- Shows a colored avatar with the user's initial
- Persists across page refreshes

### 3. **Protected Availability Submission**
- Access token is required to submit availability
- Backend validates the token before saving data
- Prevents unauthorized users from submitting on behalf of others

---

## 📋 How to Use

### For Players (Participants)

#### **Step 1: Join a Session**
1. Go to the "Join Session" page
2. Click on a session you want to join
3. Enter your **name** and **email** (must be invited)
4. Click "Continue"

#### **Step 2: Verify You're Logged In**
- Look at the top-right of the navigation bar
- You should see a badge with your name and email
- Example: "John Doe | john@example.com"

#### **Step 3: Mark Your Availability**
1. Click on time slots to mark when you're available
2. Click multiple times to cycle through:
   - **Green** = Available
   - **Orange** = Tentative
   - **Red** = Busy
   - **White** = No preference
3. Use bulk actions for quick marking:
   - "Mark All Available"
   - "Weekends Only"
   - "Weekdays Only"

#### **Step 4: Submit**
1. Click "Submit Availability"
2. Your data is sent with your access token
3. Success message appears
4. You're redirected to the dashboard

### For Dungeon Masters (Organizers)

#### **Creating a Poll**
1. Add participant emails when creating a poll
2. Each participant gets a unique access token automatically
3. Only those emails can join and submit availability

#### **Viewing Availability**
1. Go to "Manage" page
2. Select your poll
3. View who has responded
4. See availability data linked to specific participants

---

## 🔒 Security Features

### **Email Validation**
- Only invited emails can join
- Backend checks the participants table
- Returns 403 Forbidden if email not found

### **Token Validation**
- Every availability submission requires a valid token
- Backend compares provided token with stored token
- Returns 401/403 if token is missing or invalid

### **Data Integrity**
- Each availability entry is linked to a participant ID
- Prevents users from submitting for others
- Audit trail of who marked what

---

## ⚠️ Troubleshooting

### "Access token missing. Please rejoin the session."
**Cause**: Your browser's localStorage was cleared or you're on a new device.
**Solution**: Rejoin the session by entering your name and email again.

### "Invalid access token. You are not authorized..."
**Cause**: The token in your browser doesn't match the server's records.
**Solution**: Clear your browser data and rejoin the session.

### "You are not authorized to join this poll. Only invited participants can join."
**Cause**: Your email is not in the participant list.
**Solution**: Contact the DM to add your email to the poll.

### User info not showing in navigation bar
**Cause**: You haven't joined a session yet.
**Solution**: Select a session and complete the join process.

---

## 🧪 Testing Checklist

### ✅ **Basic Flow**
- [ ] Can select a session
- [ ] Join modal appears
- [ ] Can enter name and email
- [ ] User info appears in nav bar after joining
- [ ] Can mark availability
- [ ] Can submit availability successfully

### ✅ **Security**
- [ ] Cannot join with non-invited email
- [ ] Cannot submit without access token
- [ ] Cannot submit with invalid token
- [ ] User info persists after page refresh

### ✅ **Error Handling**
- [ ] Clear error message for missing token
- [ ] Clear error message for invalid token
- [ ] Clear error message for unauthorized email

---

## 📊 API Endpoints

### `POST /api/polls/:poll_id/join`
**Request:**
```json
{
  "name": "John Doe",
  "email": "john@example.com"
}
```

**Response:**
```json
{
  "id": "participant-uuid",
  "access_token": "token-uuid",
  "message": "Successfully joined the poll"
}
```

### `POST /api/polls/:poll_id/participants/:participant_id/availability`
**Request:**
```json
{
  "availability": [
    {
      "date": "2025-12-31",
      "timeSlot": "11:00",
      "status": "available"
    }
  ],
  "access_token": "token-uuid"
}
```

**Response:**
- `200 OK` - Success
- `401 Unauthorized` - Missing token
- `403 Forbidden` - Invalid token

---

## 🎨 UI Components

### **User Info Badge** (Navigation Bar)
```
┌─────────────────────────────────┐
│  [JD]  John Doe                 │
│        john@example.com         │
└─────────────────────────────────┘
```
- **Avatar**: Circular badge with user's initial
- **Name**: Bold, forest green color
- **Email**: Small, gray text
- **Background**: Light emerald with border

### **Availability Grid**
```
Time    Mon    Tue    Wed    Thu    Fri
11:00   🟢     🟢     🟢     ⚪     🔴
12:00   🟢     🟠     🟢     🟢     🔴
```
- **Green**: Available
- **Orange**: Tentative
- **Red**: Busy
- **White**: No preference

---

## 📝 Developer Notes

### **localStorage Structure**
```javascript
{
  "currentUser": {
    "id": "participant-uuid",
    "name": "John Doe",
    "email": "john@example.com",
    "accessToken": "token-uuid"
  }
}
```

### **Database Schema**
```sql
-- Participants table includes access_token
participants (
  id TEXT PRIMARY KEY,
  poll_id TEXT,
  name TEXT,
  email TEXT,
  access_token TEXT UNIQUE  -- ← Used for authentication
)

-- Availability linked to participant
availability (
  id INTEGER PRIMARY KEY,
  poll_id TEXT,
  participant_id TEXT,  -- ← Links to specific user
  date TEXT,
  time_slot TEXT,
  status TEXT
)
```

---

## 🚀 Next Steps

### **Immediate**
1. Test the feature with real users
2. Verify error messages are clear
3. Check mobile responsiveness

### **Future Enhancements**
1. Email notifications with access links
2. Token expiration and refresh
3. Admin view showing who marked what
4. Export availability data with user names
5. Session management (logout, multiple sessions)

---

## 📞 Support

If you encounter issues:
1. Check the browser console for errors
2. Verify your email is in the participant list
3. Try clearing browser data and rejoining
4. Contact the DM if problems persist

---

**Last Updated**: December 5, 2025
**Version**: 1.0.0
