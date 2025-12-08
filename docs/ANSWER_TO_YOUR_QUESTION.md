# How to Identify Users and Mark Reserved Time Slots

## Your Original Question

> "In join session it's open to everyone. How do you understand which user joined and I cannot mark available the reserved time for example 11:00 AM Wednesday, Dec 31?"

## The Answer

### ✅ **Problem 1: Identifying Which User Joined**

**Before (Insecure):**
- Users could enter any name/email
- No verification of identity
- No way to track who submitted what
- Anyone could submit availability for anyone

**After (Secure):**
1. **Email Validation**: Only invited emails can join
   ```
   User tries to join → Backend checks if email is in participants table
   → If not found: "You are not authorized to join this poll"
   → If found: User can join
   ```

2. **Unique Access Token**: Each user gets a unique token
   ```
   User joins → Backend generates/retrieves access_token
   → Frontend stores token in localStorage
   → Token is sent with every availability submission
   ```

3. **Visual Identification**: User info displayed in UI
   ```
   Navigation bar shows:
   ┌─────────────────────────────┐
   │  [JD]  John Doe             │
   │        john@example.com     │
   └─────────────────────────────┘
   ```

### ✅ **Problem 2: Marking Reserved Time Slots**

**How it works now:**

1. **User Marks Availability**
   ```
   User clicks on "11:00 AM Wednesday, Dec 31"
   → Cell turns green (available)
   → Data stored: { date: "2025-12-31", timeSlot: "11:00", status: "available" }
   ```

2. **Submission with User Identity**
   ```
   User clicks "Submit Availability"
   → Frontend sends:
     {
       "availability": [
         { "date": "2025-12-31", "timeSlot": "11:00", "status": "available" }
       ],
       "access_token": "user-unique-token"
     }
   ```

3. **Backend Validates and Saves**
   ```
   Backend receives request
   → Validates access_token matches participant
   → Saves to database:
     poll_id: "poll-123"
     participant_id: "john-doe-uuid"  ← This identifies the user!
     date: "2025-12-31"
     time_slot: "11:00"
     status: "available"
   ```

4. **Result: You Know Who Marked What**
   ```sql
   SELECT p.name, p.email, a.date, a.time_slot, a.status
   FROM availability a
   JOIN participants p ON a.participant_id = p.id
   WHERE a.poll_id = 'poll-123'
     AND a.date = '2025-12-31'
     AND a.time_slot = '11:00';
   
   Result:
   ┌───────────┬──────────────────┬────────────┬───────────┬───────────┐
   │ name      │ email            │ date       │ time_slot │ status    │
   ├───────────┼──────────────────┼────────────┼───────────┼───────────┤
   │ John Doe  │ john@example.com │ 2025-12-31 │ 11:00     │ available │
   │ Jane Smith│ jane@example.com │ 2025-12-31 │ 11:00     │ available │
   │ Bob Jones │ bob@example.com  │ 2025-12-31 │ 11:00     │ busy      │
   └───────────┴──────────────────┴────────────┴───────────┴───────────┘
   ```

---

## Example Scenario

### **Scenario**: You want to know who is available at 11:00 AM on Wednesday, Dec 31

#### **Step 1: Users Join**
```
Alice joins → Gets token: "token-alice-123"
Bob joins   → Gets token: "token-bob-456"
Carol joins → Gets token: "token-carol-789"
```

#### **Step 2: Users Mark Availability**
```
Alice marks:
  - 11:00 AM Wed Dec 31: Available ✅
  - 12:00 PM Wed Dec 31: Busy ❌

Bob marks:
  - 11:00 AM Wed Dec 31: Available ✅
  - 12:00 PM Wed Dec 31: Available ✅

Carol marks:
  - 11:00 AM Wed Dec 31: Tentative ⚠️
  - 12:00 PM Wed Dec 31: Available ✅
```

#### **Step 3: Backend Stores**
```
Database (availability table):
┌──────────────┬────────────┬───────────┬────────────┐
│ participant  │ date       │ time_slot │ status     │
├──────────────┼────────────┼───────────┼────────────┤
│ alice-uuid   │ 2025-12-31 │ 11:00     │ available  │
│ alice-uuid   │ 2025-12-31 │ 12:00     │ busy       │
│ bob-uuid     │ 2025-12-31 │ 11:00     │ available  │
│ bob-uuid     │ 2025-12-31 │ 12:00     │ available  │
│ carol-uuid   │ 2025-12-31 │ 11:00     │ tentative  │
│ carol-uuid   │ 2025-12-31 │ 12:00     │ available  │
└──────────────┴────────────┴───────────┴────────────┘
```

#### **Step 4: Query Who's Available**
```sql
-- Find who's available at 11:00 AM on Dec 31
SELECT p.name, a.status
FROM availability a
JOIN participants p ON a.participant_id = p.id
WHERE a.date = '2025-12-31'
  AND a.time_slot = '11:00';
```

**Result:**
```
Alice  → available  ✅
Bob    → available  ✅
Carol  → tentative  ⚠️
```

**Conclusion**: Alice and Bob are definitely available. Carol is tentative.

---

## Security Guarantees

### 🔒 **1. Only Invited Users Can Join**
```
Email not in participants table → 403 Forbidden
Email in participants table     → Join allowed + token issued
```

### 🔒 **2. Only Authorized Users Can Submit**
```
No token              → 401 Unauthorized
Invalid token         → 403 Forbidden
Valid token           → Availability saved
```

### 🔒 **3. Users Can't Submit for Others**
```
Alice's token → Can only submit for Alice's participant_id
Bob's token   → Can only submit for Bob's participant_id
```

### 🔒 **4. Data Integrity**
```
Every availability entry is linked to a specific participant
Backend validates token matches participant_id
Prevents tampering and unauthorized submissions
```

---

## Visual Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                    USER JOINS SESSION                            │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │ Enter Name      │
                    │ Enter Email     │
                    └─────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │ Backend Checks  │
                    │ Email is Invited│
                    └─────────────────┘
                              │
                    ┌─────────┴─────────┐
                    │                   │
                    ▼                   ▼
            ✅ Invited          ❌ Not Invited
                    │                   │
                    ▼                   ▼
        ┌───────────────────┐   ┌──────────────┐
        │ Issue Token       │   │ 403 Forbidden│
        │ Store in Browser  │   └──────────────┘
        └───────────────────┘
                    │
                    ▼
        ┌───────────────────┐
        │ Show User Info    │
        │ in Navigation Bar │
        └───────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────────────────────┐
│                  USER MARKS AVAILABILITY                         │
└─────────────────────────────────────────────────────────────────┘
                    │
                    ▼
        ┌───────────────────┐
        │ Click Time Slots  │
        │ 11:00 AM Wed      │
        └───────────────────┘
                    │
                    ▼
        ┌───────────────────┐
        │ Click Submit      │
        └───────────────────┘
                    │
                    ▼
        ┌───────────────────┐
        │ Send Data +       │
        │ Access Token      │
        └───────────────────┘
                    │
                    ▼
        ┌───────────────────┐
        │ Backend Validates │
        │ Token             │
        └───────────────────┘
                    │
        ┌───────────┴───────────┐
        │                       │
        ▼                       ▼
    ✅ Valid              ❌ Invalid
        │                       │
        ▼                       ▼
┌───────────────┐       ┌──────────────┐
│ Save to DB    │       │ 401/403 Error│
│ with User ID  │       └──────────────┘
└───────────────┘
        │
        ▼
┌───────────────────────────────────────┐
│ Database Record:                      │
│ participant_id: "alice-uuid"          │
│ date: "2025-12-31"                    │
│ time_slot: "11:00"                    │
│ status: "available"                   │
│                                       │
│ ✅ You now know Alice marked this!   │
└───────────────────────────────────────┘
```

---

## Code Example: How to Query User Availability

### **Backend (Rust)**
```rust
// Get all availability for a specific time slot
let availability: Vec<(String, String, String)> = sqlx::query_as(
    "SELECT p.name, p.email, a.status 
     FROM availability a 
     JOIN participants p ON a.participant_id = p.id 
     WHERE a.poll_id = ? 
       AND a.date = ? 
       AND a.time_slot = ?"
)
.bind(&poll_id)
.bind("2025-12-31")
.bind("11:00")
.fetch_all(&pool)
.await?;

// Result: Vec of (name, email, status)
// [("Alice", "alice@example.com", "available"),
//  ("Bob", "bob@example.com", "available"),
//  ("Carol", "carol@example.com", "tentative")]
```

### **Frontend (JavaScript)**
```javascript
// When viewing poll results
async function showWhoIsAvailable(pollId, date, timeSlot) {
    const response = await fetch(`/api/polls/${pollId}`);
    const data = await response.json();
    
    // Filter availability for specific time
    const available = data.availability.filter(a => 
        a.date === date && a.time_slot === timeSlot
    );
    
    // Get participant names
    const users = available.map(a => {
        const participant = data.participants.find(p => p.id === a.participant_id);
        return {
            name: participant.name,
            email: participant.email,
            status: a.status
        };
    });
    
    console.log(`Who's available at ${timeSlot} on ${date}:`, users);
    // Output: [
    //   { name: "Alice", email: "alice@...", status: "available" },
    //   { name: "Bob", email: "bob@...", status: "available" },
    //   { name: "Carol", email: "carol@...", status: "tentative" }
    // ]
}
```

---

## Summary

### ✅ **You Can Now:**

1. **Identify which user joined**
   - Email validation ensures only invited users
   - Unique access token per user
   - User info displayed in UI

2. **Track who marked which time slots**
   - Every availability entry linked to participant_id
   - Database stores: user + date + time + status
   - Query to see who's available when

3. **Prevent unauthorized access**
   - Token validation on every submission
   - Users can't submit for others
   - Secure and auditable

### 📊 **Example Query Result**

"Who marked 11:00 AM Wednesday, Dec 31 as available?"

```
Alice (alice@example.com)   → Available ✅
Bob (bob@example.com)       → Available ✅
Carol (carol@example.com)   → Tentative ⚠️
Dave (dave@example.com)     → Busy ❌
```

**You now have complete visibility into who marked what!** 🎉
