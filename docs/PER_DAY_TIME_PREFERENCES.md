# Per-Day Time Preferences Feature

## Overview
This feature allows you to select **one or multiple time slots for each day** you choose, giving you complete flexibility in scheduling your D&D sessions.

## How It Works

### Backend (Rust)
The backend now supports two formats for time preferences:

1. **Legacy Format** (backward compatible):
   ```json
   {
     "timeRange": "[\"18:00\", \"19:00\", \"20:00\"]"
   }
   ```

2. **New Format** (per-day preferences):
   ```json
   {
     "timePreferences": {
       "2025-12-10": ["18:00", "19:00"],
       "2025-12-11": ["20:00", "21:00"],
       "2025-12-12": ["18:00", "19:00", "20:00"]
     }
   }
   ```

### Frontend (JavaScript)
The new `enhanced-poll-creator.js` provides:

- **Per-day time selection**: Each date gets its own set of time slots
- **Visual feedback**: See exactly which times are selected for each day
- **Quick copy**: Apply one date's times to all dates with one click
- **Validation**: Ensures at least one time is selected

## Usage

### Step-by-Step Guide

1. **Step 1: Campaign Details**
   - Enter campaign name, description, duration

2. **Step 2: Select Dates**
   - Choose one or multiple dates
   - Use "Specific Dates", "Whole Week", or "Whole Month" mode

3. **Step 3: Time Preferences** (NEW!)
   - For each selected date, you'll see:
     - Day name and formatted date
     - Grid of available time slots (9 AM - 11 PM)
     - Count of selected times
   
   - **Select times for each date individually**:
     - Click time slots to toggle selection
     - Each date can have different times
   
   - **Quick Actions**:
     - Select times for the first date
     - Click "Copy First Date to All" to apply to all dates
     - Modify individual dates as needed

4. **Step 4: Invite Players**
   - Add player email addresses

5. **Step 5: Review & Create**
   - See all dates with their specific time slots
   - Verify everything looks correct
   - Submit!

## Example Scenarios

### Scenario 1: Same Times Every Day
1. Select dates: Dec 10, 11, 12
2. For Dec 10, select: 6 PM, 7 PM, 8 PM
3. Click "Copy First Date to All"
4. Result: All three days have 6-8 PM available

### Scenario 2: Different Times Per Day
1. Select dates: Dec 10 (Monday), Dec 13 (Thursday), Dec 15 (Saturday)
2. Dec 10: Select 7 PM, 8 PM (weekday evening)
3. Dec 13: Select 7 PM, 8 PM (weekday evening)
4. Dec 15: Select 2 PM, 3 PM, 4 PM, 7 PM, 8 PM (weekend - more flexible)
5. Each day has its own custom times!

### Scenario 3: Quick Setup
1. Select dates for the whole week
2. Select times for Monday
3. Click "Copy First Date to All"
4. Adjust weekend days to add afternoon slots
5. Done!

## API Changes

### Create Poll Request
```javascript
POST /api/polls
{
  "title": "Tomb of Annihilation",
  "description": "Epic adventure campaign",
  "location": "Online",
  "dates": ["2025-12-10", "2025-12-11", "2025-12-12"],
  "timePreferences": {
    "2025-12-10": ["18:00", "19:00"],
    "2025-12-11": ["19:00", "20:00"],
    "2025-12-12": ["18:00", "19:00", "20:00"]
  },
  "participants": ["player1@email.com", "player2@email.com"]
}
```

### Response
```javascript
{
  "id": "uuid-here"
}
```

## Integration

### Using the Enhanced Creator

Replace the script tag in `create-poll.html`:

```html
<!-- OLD -->
<script src="js/poll-creator.js"></script>

<!-- NEW -->
<script src="js/enhanced-poll-creator.js"></script>
```

The enhanced version is **fully backward compatible** - it will work with existing polls and the old format.

## Benefits

✅ **Flexibility**: Different times for different days  
✅ **Convenience**: Quick copy feature for common patterns  
✅ **Clarity**: See exactly what times are available each day  
✅ **Validation**: Prevents submitting without time selections  
✅ **Backward Compatible**: Works with existing polls  

## Technical Details

### Data Storage
- Stored in `polls.time_range` column as JSON string
- Can be either:
  - Legacy array: `["18:00", "19:00"]`
  - New object: `{"2025-12-10": ["18:00"], ...}`

### Validation
- At least one date must be selected
- At least one time must be selected for at least one date
- Timezone is required

### Security
- All inputs are validated and sanitized
- Maximum limits enforced (365 dates, 1000 availability entries)
- UUID validation for all IDs

## Future Enhancements

Potential improvements:
- [ ] Time zone conversion for participants
- [ ] Recurring patterns (every Monday at 7 PM)
- [ ] Duration-aware time slots (show 4-hour blocks)
- [ ] Visual calendar view
- [ ] Import/export time preferences
- [ ] Templates for common schedules

## Troubleshooting

**Q: Times aren't saving?**  
A: Make sure you've selected at least one time for at least one date, and selected a timezone.

**Q: Can I use the old format?**  
A: Yes! The system supports both formats. Old polls will continue to work.

**Q: How do I reset a date's times?**  
A: Click all selected times to deselect them, or use the copy feature to overwrite.

**Q: What if I change my selected dates?**  
A: The system automatically cleans up time preferences for removed dates.

## Support

For issues or questions:
1. Check the browser console for errors
2. Verify your dates are selected in Step 2
3. Ensure at least one time is selected in Step 3
4. Check that timezone is selected

---

**Happy Scheduling! 🎲✨**
