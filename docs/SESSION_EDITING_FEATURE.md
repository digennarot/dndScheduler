# Session Editing Feature

## Overview
The session editing feature allows you to modify existing D&D scheduling polls after they've been created. You can update campaign details, dates, time preferences, and participants without losing existing responses.

## Features

### ✅ What You Can Edit

1. **Basic Information**
   - Campaign name/title
   - Description
   - Location (Online, Discord, Roll20, etc.)

2. **Dates**
   - Add new dates
   - Remove dates
   - Change date selection entirely

3. **Time Preferences**
   - Modify time slots for each date
   - Use per-day time preferences
   - Quick copy times to all dates

4. **Participants**
   - Add new participants
   - Update participant emails
   - Existing participants are preserved

### 🔒 What's Preserved

- Poll ID (URL remains the same)
- Existing participant responses
- Creation timestamp
- Poll status (active/finalized)

## How to Use

### Step 1: Access the Edit Feature

1. Navigate to the **Manage** page (`manage.html`)
2. Select a session from your list
3. Click the **"Edit Session"** button in the session details panel

### Step 2: Edit Modal Interface

The edit modal opens with the following sections:

#### **Basic Information**
- **Campaign Name**: Update the title of your campaign
- **Description**: Modify the campaign description
- **Location**: Change where the session will be held

#### **Available Dates**
- Click the date picker to select new dates
- Uses Flatpickr for easy multi-date selection
- Dates can be added or removed

#### **Time Preferences**
- For each selected date, choose available time slots
- Click time slots to toggle selection (9 AM - 11 PM)
- Use "Copy to All" button to apply first date's times to all dates
- Each date shows:
  - Day name and formatted date
  - Number of selected times
  - Interactive time slot grid

#### **Participants**
- Enter participant emails (comma or line-break separated)
- Existing participants will be preserved
- New participants will be added

### Step 3: Save Changes

1. Review all your changes
2. Click **"Save Changes"** button
3. The system will:
   - Validate your inputs
   - Send the update to the server
   - Show a success message
   - Reload the page with updated data

## Technical Details

### API Endpoint

```javascript
PUT /api/polls/{poll_id}
```

### Request Payload

```json
{
  "title": "Updated Campaign Name",
  "description": "Updated description",
  "location": "Discord",
  "dates": ["2025-12-10", "2025-12-11", "2025-12-15"],
  "timePreferences": {
    "2025-12-10": ["18:00", "19:00", "20:00"],
    "2025-12-11": ["19:00", "20:00"],
    "2025-12-15": ["14:00", "15:00", "18:00", "19:00"]
  },
  "participants": ["player1@email.com", "player2@email.com"]
}
```

### Response

```json
{
  "success": true
}
```

## User Interface

### Edit Modal Features

- **Responsive Design**: Works on desktop, tablet, and mobile
- **Smooth Animations**: Fade in/out transitions using Anime.js
- **Real-time Validation**: Immediate feedback on required fields
- **Visual Feedback**: Selected time slots highlighted
- **Scrollable Content**: Modal adapts to content height

### Time Slot Selection

- **Interactive Grid**: Click to toggle time slots
- **Visual States**:
  - Unselected: Gray border
  - Hover: Amber border
  - Selected: Green background, darker border
- **Per-Day Configuration**: Each date has its own time slots
- **Quick Copy**: Apply one date's times to all dates

## Examples

### Example 1: Update Campaign Name and Description

1. Click "Edit Session"
2. Change title from "Tomb of Annihilation" to "ToA - Chapter 2"
3. Update description with new chapter details
4. Click "Save Changes"

### Example 2: Add More Dates

1. Click "Edit Session"
2. Click the date picker
3. Select additional dates (e.g., add next week)
4. Configure time preferences for new dates
5. Click "Save Changes"

### Example 3: Change Time Preferences

1. Click "Edit Session"
2. For each date, click time slots to toggle
3. Use "Copy to All" if times are consistent
4. Adjust individual dates as needed
5. Click "Save Changes"

### Example 4: Add New Participants

1. Click "Edit Session"
2. In the Participants field, add new emails
3. Separate with commas: `existing@email.com, new1@email.com, new2@email.com`
4. Click "Save Changes"

## Validation Rules

### Required Fields
- ✅ Campaign Name (title)
- ✅ At least one date selected

### Optional Fields
- Description
- Location
- Participants (can be empty)
- Time preferences (can be empty)

### Constraints
- Maximum 200 characters for title
- Maximum 2000 characters for description
- Maximum 200 characters for location
- Maximum 365 dates
- Maximum 100 participants
- Time slots: 9:00 AM - 11:00 PM

## Error Handling

### Common Errors

**"Please fill in all required fields"**
- Cause: Title or dates are missing
- Solution: Enter a campaign name and select at least one date

**"Failed to update poll: [error message]"**
- Cause: Server error or validation failure
- Solution: Check console for details, verify all inputs are valid

**"Poll not found"**
- Cause: Invalid poll ID or poll was deleted
- Solution: Return to manage page and select a valid poll

## Best Practices

### When to Edit

✅ **Good Times to Edit:**
- Before participants have responded
- To fix typos or clarify information
- To add more date options
- To adjust time slots based on feedback

⚠️ **Be Careful When:**
- Participants have already responded
- Poll is close to finalized
- Removing dates that participants selected

### Tips

1. **Communicate Changes**: Let participants know when you make significant changes
2. **Preserve Responses**: Avoid removing dates that have responses
3. **Test First**: Make small changes and verify they work
4. **Use Copy Feature**: Save time by copying times to all dates
5. **Review Before Saving**: Double-check all changes before clicking Save

## Keyboard Shortcuts

- **ESC**: Close edit modal (when implemented)
- **Enter**: Submit form (when in text input)
- **Tab**: Navigate between fields

## Browser Compatibility

- ✅ Chrome 90+
- ✅ Firefox 88+
- ✅ Safari 14+
- ✅ Edge 90+

## Dependencies

- **Flatpickr**: Date picker library
- **Anime.js**: Animation library (optional, graceful degradation)
- **Tailwind CSS**: Styling framework

## Troubleshooting

### Modal Doesn't Open
- Check browser console for errors
- Verify `sessionManager` is initialized
- Ensure a session is selected

### Dates Not Saving
- Verify dates are in YYYY-MM-DD format
- Check that at least one date is selected
- Look for validation errors in console

### Time Slots Not Toggling
- Ensure JavaScript is enabled
- Check for console errors
- Try refreshing the page

### Changes Not Persisting
- Verify API endpoint is accessible
- Check network tab for failed requests
- Ensure proper authentication (if required)

## Future Enhancements

Planned improvements:
- [ ] Undo/Redo functionality
- [ ] Change history/audit log
- [ ] Bulk edit multiple sessions
- [ ] Template system for common configurations
- [ ] Drag-and-drop date selection
- [ ] Keyboard navigation for time slots
- [ ] Export/import session configurations

## Support

For issues or questions:
1. Check the browser console for errors
2. Verify all required fields are filled
3. Ensure you have a stable internet connection
4. Try refreshing the page
5. Check that the poll exists and you have permission to edit it

---

**Happy Editing! 🎲✨**
