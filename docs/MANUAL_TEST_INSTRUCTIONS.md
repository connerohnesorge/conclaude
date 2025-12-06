# Manual Testing Instructions for Star-Warp Search

## Quick Start

1. Open your web browser (Chrome, Firefox, Safari, or Edge)
2. Navigate to: **http://localhost:4321/**
3. Follow the test scenarios below

---

## Test Scenario 1: Search Button Visibility

**What to Look For:**
- In the site header (top of page), between the "conclaude" logo and GitHub icon
- A search button with a magnifying glass icon
- On desktop: Should show "Search" text and "Ctrl K" keyboard hint
- On mobile: Should show only the magnifying glass icon

**Expected Appearance:**
```
[conclaude logo] [🔍 Search Ctrl K] [GitHub icon] [Theme selector]
```

**Pass Criteria:**
- [ ] Search button is visible
- [ ] Icon is present
- [ ] Text is visible on desktop
- [ ] Keyboard hint visible on desktop

---

## Test Scenario 2: Opening the Search Modal

### Method A: Click
1. Click the search button in the header
2. A search modal should appear over the page

### Method B: Keyboard Shortcut
1. Press **Ctrl+K** (Windows/Linux) or **Cmd+K** (Mac)
2. The same search modal should appear

**Pass Criteria:**
- [ ] Modal opens when clicking button
- [ ] Modal opens when pressing Ctrl+K / Cmd+K
- [ ] Modal has a search input field
- [ ] Input field is automatically focused
- [ ] Page background is dimmed/overlayed

---

## Test Scenario 3: Searching for Content

1. Open the search modal (click button or press Ctrl+K)
2. Type: **guide**
3. Wait 0.5 seconds

**What Should Happen:**
- Search results appear instantly (no loading spinner needed)
- Results show page titles containing "guide"
- Each result shows:
  - Page title (clickable)
  - Brief excerpt with highlighted search term
  - Breadcrumb/URL path
- Results update as you type

**Pass Criteria:**
- [ ] Results appear quickly (< 1 second)
- [ ] Results are relevant to the query
- [ ] Search terms are highlighted in results
- [ ] Multiple results are shown

---

## Test Scenario 4: Different Search Queries

Test these queries one by one:

### Query: "changelog"
- **Expected**: Pages related to changelogs

### Query: "example"
- **Expected**: Pages with code examples

### Query: "configuration"
- **Expected**: Configuration-related pages

**Pass Criteria:**
- [ ] Each query returns different results
- [ ] Results are relevant to the query
- [ ] Search is responsive and fast

---

## Test Scenario 5: Navigating from Results

1. Open search modal
2. Search for: **guide**
3. Click on any search result

**What Should Happen:**
- Modal closes automatically
- Browser navigates to the clicked page
- URL in address bar updates
- Page content loads

**Pass Criteria:**
- [ ] Clicking result navigates to correct page
- [ ] Modal closes automatically
- [ ] Navigation is smooth (no errors)

---

## Test Scenario 6: No Results Handling

1. Open search modal
2. Type: **zzzznonexistent**

**What Should Happen:**
- No results are shown
- A "No results" message appears
- No JavaScript errors in console

**Pass Criteria:**
- [ ] Shows appropriate "No results" message
- [ ] No errors in browser console
- [ ] UI remains functional

---

## Test Scenario 7: Closing the Modal

Try each method:

### Method A: Escape Key
1. Open modal
2. Press **Esc**
3. Modal should close

### Method B: Click Outside
1. Open modal
2. Click on the darkened area outside the modal
3. Modal should close

### Method C: Cancel Button (Mobile)
1. Open modal on mobile device or narrow browser window
2. Click "Cancel" button
3. Modal should close

**Pass Criteria:**
- [ ] Escape key closes modal
- [ ] Click outside closes modal
- [ ] Cancel button works (mobile)
- [ ] Focus returns to page

---

## Test Scenario 8: Star-Warp "I'm Feeling Lucky"

1. In the address bar, type: **http://localhost:4321/warp?q=guide**
2. Press Enter

**What Should Happen:**
- Page loads briefly
- Automatically redirects to the best match for "guide"
- You land on the most relevant page
- No 404 error

**Pass Criteria:**
- [ ] Redirects to a relevant page
- [ ] No 404 error
- [ ] Redirect happens quickly

---

## Test Scenario 9: Keyboard Navigation

1. Open search modal (Ctrl+K)
2. Type a search query
3. Press **Tab** key multiple times

**What Should Happen:**
- Focus moves through search results
- Current focused result is highlighted
- Can press **Enter** to navigate to focused result

**Pass Criteria:**
- [ ] Tab key moves through results
- [ ] Focused result is visually highlighted
- [ ] Enter key navigates to focused result

---

## Test Scenario 10: Console Errors Check

1. Open browser Developer Tools (F12)
2. Go to "Console" tab
3. Perform several searches
4. Navigate through results

**What to Look For:**
- Should be NO red errors
- May see some informational logs (blue/gray)
- No WASM compilation errors
- No module loading errors

**Pass Criteria:**
- [ ] No JavaScript errors in console
- [ ] No failed network requests
- [ ] No WASM errors

---

## Responsive Design Testing

### Desktop View (> 768px width)
- [ ] Search button shows full text "Search"
- [ ] Keyboard hint "Ctrl K" is visible
- [ ] Modal is centered and sized appropriately

### Mobile View (< 768px width)
- [ ] Search button shows only icon
- [ ] Modal is full-screen or nearly full-screen
- [ ] "Cancel" button is visible
- [ ] Results are touch-friendly

---

## Performance Testing

1. Open browser DevTools
2. Go to "Network" tab
3. Reload page
4. Open search modal
5. Type a query

**Check:**
- [ ] No network requests when typing (client-side search)
- [ ] Pagefind WASM file loaded successfully
- [ ] Results appear in < 500ms

---

## Final Checklist

After completing all tests:

- [ ] All core functionality works
- [ ] No console errors
- [ ] Search is fast and responsive
- [ ] Results are relevant
- [ ] Navigation works correctly
- [ ] Modal opens and closes properly
- [ ] Keyboard shortcuts work
- [ ] Responsive design adapts to screen size

---

## Reporting Issues

If any test fails, note:
1. Which test scenario failed
2. What you expected to happen
3. What actually happened
4. Any error messages in console
5. Browser and version (e.g., Chrome 120, Firefox 121)
6. Screen size/device type

---

## Expected Success Rate

Based on infrastructure verification: **95%+ of tests should pass**

If more than 1-2 tests fail, there may be a configuration issue.

---

## Quick Test (1 minute)

If you just want to verify basics:

1. [ ] Open http://localhost:4321/
2. [ ] Press Ctrl+K (or Cmd+K)
3. [ ] Type "guide"
4. [ ] See results appear
5. [ ] Click a result
6. [ ] Navigate to page

If all these work, the core functionality is operational!

