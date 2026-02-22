# Debugging: Cannot Mark Availability

## 🔍 Steps to Debug:

1. **Open the page**: http://localhost:3000/participate.html

2. **Open Browser DevTools**:
   - Press `F12`
   - Go to the **Console** tab

3. **Clear localStorage** (important!):
   ```javascript
   localStorage.clear()
   ```
   Then refresh the page.

4. **Select "Wednesday Dec 31 Session"**

5. **Join with**:
   - Name: `AAA`
   - Email: `aaa@aaa.it`

6. **Check Console Output**:
   You should see:
   ```
   Generating grid for session: {title: "Wednesday Dec 31 Session", ...}
   Parsed dates: ["2025-12-31"]
   Parsed time slots: ["10:00", "11:00", "12:00", "13:00", "14:00"]
   ```

7. **Click on a cell** (e.g., 11:00 AM under Wed, Dec 31)

8. **Check Console Output**:
   You should see:
   ```
   toggleAvailability called with cellId: 2025-12-31_11:00
   Found cell: <div id="2025-12-31_11:00" class="grid-cell clickable">
   Set to available
   Current availabilityData: {2025-12-31_11:00: "available"}
   ```

9. **Check if cell turned green**

---

## ❌ **Possible Issues:**

### **Issue 1: Grid not showing**
**Symptoms**: No grid appears, or shows "No dates available"
**Cause**: Session data not loaded properly
**Solution**: Check console for "Parsed dates" and "Parsed time slots"

### **Issue 2: Click does nothing**
**Symptoms**: Click on cell, no color change, no console log
**Cause**: Click handler not attached
**Solution**: Check if you see "toggleAvailability called" in console

### **Issue 3: Cell found but color doesn't change**
**Symptoms**: Console shows "Set to available" but cell stays white
**Cause**: CSS not loading or being overridden
**Solution**: Check browser DevTools → Elements tab → Inspect the cell

### **Issue 4: "Cell not found" error**
**Symptoms**: Console shows "Cell not found for id: ..."
**Cause**: Cell ID mismatch
**Solution**: Check the cell IDs in the HTML vs what's being clicked

---

## 🎯 **Expected Behavior:**

When you click a cell:
1. **First click**: Cell turns **green** (available)
2. **Second click**: Cell turns **orange** (tentative)
3. **Third click**: Cell turns **red** (busy)
4. **Fourth click**: Cell turns **white** (cleared)

---

## 📋 **What to Report:**

Please copy and paste from the browser console:
1. The output after "Generating grid for session"
2. The output after clicking a cell
3. Any error messages (in red)

This will help me identify the exact issue!
