# Timezone Configuration

## Default Timezone: Europe/Rome

The D&D Scheduler application now defaults to **Europe/Rome** timezone (Central European Time - Italy, CET/CEST).

## Changes Made

### File: `static/create-poll.html`

**Added:**
- New timezone option: `Europe/Rome`
- Label: "Central European Time - Italy (CET/CEST)"
- Set as default with `selected` attribute

**Location:** Step 3 - Time Preferences section

## Available Timezones

The application supports the following timezones:

1. **America/New_York** - Eastern Time (EST/EDT)
2. **America/Chicago** - Central Time (CST/CDT)
3. **America/Denver** - Mountain Time (MST/MDT)
4. **America/Los_Angeles** - Pacific Time (PST/PDT)
5. **Europe/London** - Greenwich Mean Time (GMT)
6. **Europe/Paris** - Central European Time (CET)
7. **Europe/Rome** - Central European Time - Italy (CET/CEST) ⭐ **DEFAULT**

## User Experience

### When Creating a Poll

1. Navigate to Step 3 (Time Preferences)
2. The timezone dropdown will show "Central European Time - Italy (CET/CEST)" pre-selected
3. Users can change it if needed
4. The timezone is required before proceeding

### Timezone Information

**Europe/Rome (CET/CEST):**
- **Standard Time (CET):** UTC+1 (Winter)
- **Daylight Saving Time (CEST):** UTC+2 (Summer)
- **DST Period:** Last Sunday in March to last Sunday in October
- **Current Offset:** UTC+1 (as of December 2025)

## Technical Details

### HTML Implementation

```html
<select id="timezone" name="timezone" required>
  <option value="">Select timezone</option>
  <!-- Other timezones... -->
  <option value="Europe/Rome" selected>
    Central European Time - Italy (CET/CEST)
  </option>
</select>
```

### JavaScript Handling

The timezone value is:
1. Read from the form field
2. Validated (required field)
3. Included in poll creation payload
4. Stored with the poll data

### Backend Storage

- Stored as string in database
- Format: IANA timezone identifier (e.g., "Europe/Rome")
- Used for time calculations and display

## Adding More Timezones

To add additional timezones, edit `static/create-poll.html`:

```html
<option value="Europe/Berlin">
  Central European Time - Germany (CET/CEST)
</option>
```

### Common European Timezones

If you want to add more European options:

```html
<option value="Europe/Madrid">
  Central European Time - Spain (CET/CEST)
</option>
<option value="Europe/Amsterdam">
  Central European Time - Netherlands (CET/CEST)
</option>
<option value="Europe/Brussels">
  Central European Time - Belgium (CET/CEST)
</option>
<option value="Europe/Vienna">
  Central European Time - Austria (CET/CEST)
</option>
<option value="Europe/Zurich">
  Central European Time - Switzerland (CET/CEST)
</option>
```

### Common Italian Timezones

Note: All of Italy uses the same timezone (Europe/Rome), but you could add aliases:

```html
<option value="Europe/Rome">
  Rome, Milan, Naples - Italy (CET/CEST)
</option>
```

## Timezone Best Practices

### For Users

1. **Always verify timezone** - Even though Rome is default, check it's correct
2. **Consider participants** - If players are in different timezones, note this in description
3. **DST awareness** - Remember that times shift during daylight saving transitions

### For Developers

1. **Store in UTC** - Convert display times to UTC for storage
2. **Use IANA identifiers** - Always use standard timezone names
3. **Handle DST** - Account for daylight saving time transitions
4. **Validate input** - Ensure timezone is a valid IANA identifier

## Future Enhancements

Potential improvements:

- [ ] Auto-detect user's timezone from browser
- [ ] Show current time in selected timezone
- [ ] Timezone converter for participants
- [ ] Support for more timezones
- [ ] Grouped timezone selector (by region)
- [ ] Search/filter timezone list
- [ ] Display UTC offset next to timezone name

## Testing

To verify the default timezone:

1. **Open create poll page:**
   ```
   http://localhost:3000/create-poll.html
   ```

2. **Navigate to Step 3** (Time Preferences)

3. **Check timezone dropdown:**
   - Should show "Central European Time - Italy (CET/CEST)" selected
   - Dropdown should be pre-filled (not empty)

4. **Create a poll:**
   - Timezone should be included in payload
   - No validation error should occur

## Troubleshooting

### Timezone Not Pre-selected

**Issue:** Dropdown shows "Select timezone" instead of Rome

**Solution:**
- Check that `selected` attribute is on Europe/Rome option
- Clear browser cache
- Verify HTML file was saved correctly

### Validation Error

**Issue:** "Please select your timezone" error appears

**Solution:**
- Ensure the option has a non-empty `value` attribute
- Check that the field is not being cleared by JavaScript
- Verify form validation logic

### Wrong Time Display

**Issue:** Times showing in wrong timezone

**Solution:**
- Verify timezone is being sent to backend
- Check backend timezone handling
- Ensure proper UTC conversion

## Support

For timezone-related issues:
1. Verify Europe/Rome is in the dropdown
2. Check browser console for errors
3. Confirm timezone value in network request
4. Test with different timezones to isolate issue

---

**Default Timezone: Europe/Rome (CET/CEST) 🇮🇹**
