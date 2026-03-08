# E2E Test Failure Report

**Date**: March 8, 2025  
**Test Suite**: Guest/Unauthenticated User Tests  
**Total Failures**: 8 tests

---

## Summary

All test failures fall into two categories:

1. **Form Input Interaction Failures (7 tests)**: Tests cannot fill form inputs because `data-testid` attributes are placed on MUI TextField wrapper `<div>` elements instead of the actual `<input>` elements.

2. **API Endpoint Failure (1 test)**: The `/api/auth/logout` endpoint returns `405 Method Not Allowed` when accessed via GET, but the test expects `200 OK`.

---

## Category 1: Form Input Interaction Failures (7 tests)

### Root Cause

The `data-testid` attributes (`register-email-input`, `register-password-input`, `login-email-input`, `login-password-input`) are placed on the MUI `TextField` component wrapper `<div>`, not on the actual `<input>` element inside it.

**Error Message**:
```
Error: Element is not an <input>, <textarea>, <select> or [contenteditable] 
and does not have a role allowing [aria-readonly]
```

**What Playwright Found**:
```
locator resolved to <div data-testid="register-email-input" 
class="MuiFormControl-root MuiFormControl-fullWidth MuiTextField-root ...">
```

**What Playwright Needs**:
The actual `<input>` element inside the TextField component.

### Affected Tests

1. ✅ `invalid email shows error` (Registration Flow)
   - **Location**: `tests/01-guest.spec.ts:162:7`
   - **Test ID**: `register-email-input`
   - **Issue**: Cannot fill email field

2. ✅ `short password shows error` (Registration Flow)
   - **Location**: `tests/01-guest.spec.ts:187:7`
   - **Test ID**: `register-email-input`
   - **Issue**: Cannot fill email field

3. ✅ `redirects to login on success` (Registration Flow)
   - **Location**: `tests/01-guest.spec.ts:215:7`
   - **Test ID**: `register-email-input`
   - **Issue**: Cannot fill email field

4. ✅ `can login with new credentials` (Registration Flow)
   - **Location**: `tests/01-guest.spec.ts:237:7`
   - **Test ID**: `register-email-input`
   - **Issue**: Cannot fill email field

5. ✅ `shows error for invalid email` (Login Flow)
   - **Location**: `tests/01-guest.spec.ts:290:7`
   - **Test ID**: `login-email-input`
   - **Issue**: Cannot fill email field

6. ✅ `single profile redirects to dashboard` (Login Flow)
   - **Location**: `tests/01-guest.spec.ts:316:7`
   - **Test ID**: `login-email-input`
   - **Issue**: Cannot fill email field

7. ✅ `multi-profile redirects to scope selection` (Login Flow)
   - **Location**: `tests/01-guest.spec.ts:335:7`
   - **Test ID**: `login-email-input`
   - **Issue**: Cannot fill email field

### Solution

Move the `data-testid` attribute from the `TextField` component to the actual `<input>` element using MUI's `slotProps.htmlInput` prop.

**Current Code** (LoginPage.tsx):
```tsx
<TextField
  {...field}
  data-testid="login-email-input"  // ❌ This goes on the wrapper div
  // ...
/>
```

**Fixed Code**:
```tsx
<TextField
  {...field}
  slotProps={{
    htmlInput: {
      'data-testid': 'login-email-input',  // ✅ This goes on the actual input
      autoComplete: 'email'
    }
  }}
  // ...
/>
```

**Files to Fix**:
- `/frontend/src/features/authentication/pages/LoginPage/LoginPage.tsx`
  - `login-email-input` (line ~100)
  - `login-password-input` (line ~122)
  
- `/frontend/src/features/authentication/pages/RegisterPage/RegisterPage.tsx`
  - `register-email-input` (line ~94)
  - `register-password-input` (line ~116)

---

## Category 2: API Endpoint Failure (1 test)

### Root Cause

The `/api/auth/logout` endpoint returns `405 Method Not Allowed` when accessed via GET request, but the test expects `200 OK`.

**Error Message**:
```
Error: expect(received).toBe(expected) // Object.is equality
Expected: 200
Received: 405
```

**Test Code**:
```typescript
test('GET /api/auth/logout returns 200 OK (works for anonymous)', async ({
  request,
}) => {
  const response = await request.get(`${API_BASE_URL}/api/auth/logout`)
  expect(response.status()).toBe(200)  // ❌ Fails: receives 405
})
```

### Affected Test

8. ✅ `GET /api/auth/logout returns 200 OK (works for anonymous)`
   - **Location**: `tests/01-guest.spec.ts:397:5`
   - **Issue**: Endpoint returns 405 instead of 200

### Possible Solutions

**Option 1**: Update the backend to accept GET requests for `/api/auth/logout` (if this is the intended behavior).

**Option 2**: Update the test to use POST request instead of GET (if logout should be POST-only).

**Option 3**: Update the test expectation to accept 405 if GET is intentionally not allowed for logout.

**Recommendation**: Check the backend implementation to determine the correct HTTP method for logout. Typically, logout endpoints use POST, but if the requirement is that it should work for anonymous users via GET, the backend needs to be updated.

---

## Impact Analysis

### Severity: **HIGH**

- **7 out of 8 failures** are blocking all form interaction tests
- **All registration and login flows are broken** due to the form input issue
- **1 API endpoint test** is failing, but this is less critical for core functionality

### Priority Fix Order

1. **Fix form input test IDs** (Category 1) - **CRITICAL**
   - Blocks all authentication flow tests
   - Simple fix: move `data-testid` to `slotProps.htmlInput`
   - Affects 2 files, 4 test IDs total

2. **Fix logout endpoint** (Category 2) - **MEDIUM**
   - Single test failure
   - Requires backend investigation/change
   - May require test expectation update

---

## Next Steps

1. ✅ **Immediate**: Fix `data-testid` placement in LoginPage and RegisterPage components
2. ⏳ **Next**: Investigate `/api/auth/logout` endpoint implementation
3. ⏳ **Verify**: Run tests again after fixes

---

## Technical Details

### MUI TextField Structure

MUI's `TextField` component renders a wrapper structure:
```html
<div class="MuiTextField-root" data-testid="...">  <!-- ❌ Test ID here -->
  <div class="MuiInputBase-root">
    <input type="email" />  <!-- ✅ Test ID should be here -->
  </div>
</div>
```

### Playwright Fill Behavior

Playwright's `locator.fill()` method requires:
- An `<input>` element
- A `<textarea>` element  
- A `<select>` element
- An element with `[contenteditable]` attribute
- An element with a role allowing `[aria-readonly]`

A `<div>` wrapper does not meet these requirements.

### Solution Pattern

Use MUI's `slotProps.htmlInput` to pass props directly to the underlying `<input>` element:

```tsx
<TextField
  slotProps={{
    htmlInput: {
      'data-testid': 'login-email-input',
      // other input-specific props
    }
  }}
/>
```
