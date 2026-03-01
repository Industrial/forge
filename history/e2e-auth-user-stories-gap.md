# E2E Auth: Current Coverage vs User Stories

Comparison of existing E2E tests with the auth/ReBAC user stories, and what to implement (focus: 005).

## What We Already Have

### Test files
- **005_auth.rs** – Single test: project layout assertions (user model with `current_org_id`/`current_role`/AuthzContext, auth handlers with guard/audit, main.rs with `post_route` and `/api/auth/admin`). **Browser flow**: register (unique email) → login → assert dashboard visible. **Skips browser** if `E2E_WEBDRIVER_URL` is unset.
- **006_authz.rs** – Single test: same layout assertions (org, membership, AuthzContext, guard). **Browser flow**: assert unauthed `/dashboard` redirects to login → register → login → assert dashboard visible.

### Test lib (browser.rs)
- `register(c, base, email, password)` – submit register form, wait for redirect to `/login`
- `login(c, base, email, password)` – submit login form, wait for dashboard (URL or h1)
- `assert_dashboard_visible(c, base)` – goto `/dashboard`, assert h1 "Dashboard"
- `assert_dashboard_redirects_to_login(c, base)` – goto `/dashboard` unauthed, assert URL contains "login"

### Template auth (handlers/auth.rs)
- Register, login, logout, profile, create_token, **admin_only**
- **admin_only** uses `guard_and_audit_user(..., Role::Owner, ...)` → **org-scoped Owner**, not global `is_admin`. So any org Owner can access `/api/auth/admin`; it is not “global admin only”.

### E2E harness (bin/test-e2e)
- **auto_seed = false** → no seed users (admin@admin.com, viewer@default.org, etc.) in DB. All current tests use register + login with a newly created user.

---

## User Stories vs Coverage

| Story | Covered? | Where / Note |
|-------|----------|---------------|
| **Login** (existing user) | ❌ | Only register-then-login with new user |
| **Login** (wrong password) | ❌ | No failed-login test |
| **Login** (inactive user) | ❌ | No test; seed has no inactive user |
| **Logout** | ❌ | No test |
| **Unauthed /dashboard → login** | ✅ | 006 + `assert_dashboard_redirects_to_login` |
| **Register → login → dashboard** | ✅ | 005, 006 |
| **Global admin** (is_admin) access admin | ⚠️ | Template uses Role::Owner, not is_admin; no e2e |
| **Non-admin** cannot access admin | ❌ | No e2e; need viewer/organadmin vs admin |
| **Org switching** | ❌ | No UI/API in template yet |
| **Role-based** (viewer vs owner) | ❌ | No e2e |
| **Multi-org user** | ❌ | No e2e; no seed in e2e anyway |
| **Session / redirect after login** | ✅ | Implicit in login → dashboard |

---

## What to Do (Implement in 005)

### 1. Seed users in E2E
- **Option A (recommended):** In `bin/test-e2e`, set **auto_seed = true** so seed users (admin@admin.com, owner@default.org, viewer@default.org, multi@email.com, etc.) exist. Then 005 can log in as specific users.
- **Option B:** Keep auto_seed = false and create users in test (e.g. via API or DB). Heavier and duplicates seed logic.

### 2. Tests to add in 005 (browser, when WebDriver available)

- **Login with seed user** – Login as `admin@admin.com` / `password`, then assert dashboard visible (proves existing-user login).
- **Failed login** – Login with wrong password, assert stay on login or error message (no dashboard).
- **Logout** – Login → logout (call `/api/auth/logout` or use a “Logout” link if present) → goto `/dashboard` → assert redirect to login (reuse `assert_dashboard_redirects_to_login`).
- **Unauthed dashboard redirect** – Goto `/dashboard` without logging in, assert redirect to login. (Already in 006; can consolidate in 005 or keep 006 as-is.)
- **Admin endpoint: only global admin** – Align template with design: gate `/api/auth/admin` on **is_admin** (not only Role::Owner). Then:
  - Login as `admin@admin.com` → request GET `/api/auth/admin` (with session cookie) or open an /admin page that calls it → assert 200 and “access granted” (or equivalent).
  - Login as `viewer@default.org` → same request → assert 403 (or 401).
- **Optional: inactive user** – Add an inactive seed user or create one in test; attempt login → assert failure.

### 3. Browser lib additions (if needed)

- **logout(c, base)** – Goto a page that triggers logout (e.g. link to `/api/auth/logout` or POST from form), then wait for redirect or session clear. Or from test: after login, perform a request that logs out (e.g. GET `/api/auth/logout` in template) and then assert_dashboard_redirects_to_login.
- **login_fails(c, base, email, password)** – Submit login form, assert URL still login or error visible (no redirect to dashboard).
- **Optional: get_session_cookie(c)** – If we test admin by HTTP from test process, we need the cookie; possible but more complex. Prefer testing admin via a dedicated Inertia page that calls the admin API and shows result, then assert page content.

### 4. Template changes for “admin panel” story

- **admin_only** in `handlers/auth.rs`: require **is_admin** (e.g. `user.is_admin`) for access, not only `Role::Owner`. Optionally still allow Owner for backward compat, but for “global admin panel” the gate should be `is_admin`.
- **Optional:** Add an `/admin` Inertia route that calls `/api/auth/admin` (or equivalent) and renders “Admin only” or “Forbidden”. Then E2E: login as admin → goto `/admin` → assert “Admin only”; login as viewer → goto `/admin` → assert “Forbidden” or redirect.

### 5. Out of scope for this pass (no E2E yet)

- Org switching (no UI in template).
- Multi-org user flows (multi@email.com) – can add once switcher exists.
- Registration validation, forgot password, change password – add when those flows exist.
- Inactive-user seed and test – optional follow-up.

---

## Suggested order of work

1. **Enable seed in e2e** – `bin/test-e2e`: set `auto_seed = true`.
2. **Template: gate admin on is_admin** – In `auth.rs`, change `admin_only` to require `user.is_admin` (and optionally allow Role::Owner if desired).
3. **005: add tests** – Login with seed admin, failed login, logout then dashboard redirect, (optional) unauthed dashboard redirect, admin endpoint for admin vs viewer.
4. **Lib: add logout helper** – e.g. `logout(c, base)` that triggers logout and optionally asserts redirect.
5. **Lib: add login_fails** – for wrong-password test.
6. **(Optional)** Add `/admin` page and E2E that checks content after login as admin vs viewer.

This gives full coverage of the “login, logout, switch user, interactions” set that can be tested today; org switching and multi-org stay for when the UI exists.
