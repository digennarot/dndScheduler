# Time Preferences Enhancement Plan

## Current Implementation
- Users select multiple time slots globally (applies to all dates)
- Time slots stored as a single JSON array in `time_range` field
- No per-day customization

## Proposed Enhancement
Allow users to specify different time slots for each selected day.

### Database Changes
**Option 1: Keep current schema** (Recommended for backward compatibility)
- Store time preferences as JSON object: `{"2025-12-10": ["18:00", "19:00"], "2025-12-11": ["20:00"]}`
- No schema migration needed

**Option 2: New table**
- Create `poll_time_preferences` table
- More normalized but requires migration

### Frontend Changes
1. **Step 2 (Date Selection)**: Keep as is
2. **Step 3 (Time Preferences)**: 
   - Show each selected date
   - Allow selecting multiple time slots per date
   - Option to "Apply to all dates" for convenience
   - Visual calendar view showing dates with their time slots

### Data Structure
```json
{
  "dates": ["2025-12-10", "2025-12-11", "2025-12-12"],
  "timePreferences": {
    "2025-12-10": ["18:00", "19:00", "20:00"],
    "2025-12-11": ["19:00", "20:00"],
    "2025-12-12": ["18:00", "19:00"]
  }
}
```

### Implementation Steps
1. Update `CreatePollRequest` model to accept new structure
2. Update `handlers.rs` to handle both old and new formats (backward compatible)
3. Create new UI component for per-day time selection
4. Update poll-creator.js to manage per-day selections
5. Update availability-manager.js to display per-day times

### User Experience
- Select dates first (Step 2)
- In Step 3, see a list of selected dates
- For each date, select one or more time slots
- Quick action: "Copy times to all dates"
- Visual feedback showing which dates have times selected
