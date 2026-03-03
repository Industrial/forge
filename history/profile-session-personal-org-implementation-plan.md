# Implementation Plan: Profile in Session, Personal Org, Profile Select

**Historical:** Session-based auth has been removed. Scope (org, role) is now provided by request headers (`X-Organization-Id`, `X-Role-Name`) and token; there is no server-side session or session profile. This doc is kept for context on personal-org and profile-select UX; implementation uses token + headers.

**Scope**: Template app (default) in `crates/forge-cli/templates/default/`.  
**Goals**: (1) Personal org = first org; auto-create on user creation; seeds use same logic. (2) Profile (org, role) stored in session (Option B). (3) Profile-select page after login when multiple profiles; all data/UI scoped to session profile.

---

## 1. Entities

### 1.1 No new entities

- **User**: Keep `current_org_id` and `current_role` for now as optional "last used" only; they are **not** the source of truth for the active profile (session is). Alternatively remove them in a later migration to avoid confusion.
- **Organization, membership, user_org_role, org_role, role_permission**: No schema changes.
- **Personal org**: No new column. "Personal" = first org by convention (first membership by `created_at`, or the org created in the same transaction as the user at signup).

### 1.2 Optional: `user.personal_org_id`

- If we want an explicit personal org (e.g. for "can't leave personal org" or billing), add `personal_org_id: Option<Uuid>` to user and set it once at user+org creation. Not required for "first org" rule; document as optional follow-up.

**Deliverables**: None required for MVP; optionally document deprecation of `current_org_id`/`current_role` for authz.

**Done**: Authz uses session profile only. `user.current_org_id` and `user.current_role` are deprecated for authorization; the source of truth is the session keys `SESSION_CURRENT_ORG_ID` and `SESSION_CURRENT_ROLE_NAME`. Seeds and register may still set these on the user row for legacy or display; handlers must not rely on them for scoping or permissions.

---

## 2. Migrations

### 2.1 Optional: drop `user.current_org_id` and `user.current_role`

- If we fully move to session-only profile, add a migration that drops these columns (and update user model). Alternatively keep them as cache of "last used" and ignore in authz.
- **Recommendation**: Keep columns for now; read profile from session in all handlers. Migrate to drop later if desired.

**Deliverables**: No migration required for MVP (session stores profile; user row unchanged or deprecated in code only).

---

## 3. Seeds

### 3.1 One org per user (personal org) via same backend logic

- **Current state**: Seed creates orgs (Default, Other) and users with memberships. Registration already creates user + org + membership + owner role in one transaction.
- **Target**: Every seeded user has at least one org (their "first" org). Two options:
  - **Strict**: Use a shared helper "create user with personal org" (same as register flow): create user, create org (e.g. `"{email}'s workspace"` or named per seed), create org_roles, seed_role_permissions_for_org, add membership + user_org_role(owner). Then add extra memberships for multi-org tests (e.g. multi@email.com in Default and Other). Result: more orgs (one per seed user), clear 1:1 for "first org."
  - **Relaxed**: Keep Default/Other; seed users are attached to them. "First org" = first membership by insert order (Default for most). Same rule, fewer orgs.
- **Decision**: Document choice (strict vs relaxed). If strict, add `ensure_user_with_personal_org(db, email, password, …)` used by both register and seeds; seeds call it then optionally add second membership for multi@email.com.

**Deliverables**:
- Either: refactor seeds to call a shared "create user + auto-create one org" helper (and use it from register).
- Or: keep current seed structure; document "first org = first membership" and ensure every seed user has at least one membership.

---

## 4. Backend helpers and auth flow

### 4.1 Session keys for profile

- Define constants, e.g. `SESSION_CURRENT_ORG_ID: &str = "current_org_id"`, `SESSION_CURRENT_ROLE_ID: &str = "current_role_id"` (or `SESSION_CURRENT_ROLE_NAME`). Store in tower-sessions (same store as flash).
- Session shape: `user_id` (from auth backend) + `current_org_id` (Uuid) + `current_role_id` (Uuid) or `current_role_name` (String). Prefer role_id for consistency with profiles list; if we store role_name, we avoid a join when building session JSON.

### 4.2 Current profile extractor

- New type, e.g. `CurrentProfile { org_id: Uuid, role_name: String }` (or role_id). Implement `FromRequestParts`: require auth first, then read `current_org_id` and `current_role_*` from session. If missing, return a specific rejection (e.g. "profile required") so the router can return 403 or redirect to profile-select.
- Handlers that need "current org" or "current role" take `CurrentProfile` (or a wrapper that includes User + Profile). Replace every use of `user.current_org_id` / `user.current_role` with profile from this extractor (or from session inside the handler if we don’t use an extractor everywhere).

### 4.3 Where profile is required

- **Dashboard routes**: All routes under `/api/dashboard/*` and any route that uses `scope_org_for_dashboard_lists` or `user.current_org_id` must require a profile (session has org_id + role). So either:
  - Protect dashboard with a middleware that ensures session profile is set; if not, return 403 with a body that the frontend can interpret as "redirect to profile-select," or
  - Have each handler that needs profile use `CurrentProfile` extractor (and return 403 if missing).
- **Session JSON**: `session_json` should return `current_org_id` and current role from **session** (and list of profiles from DB). So after login, frontend can show profile-select until session has profile, then show dashboard.

### 4.4 Login flow (backend)

- After successful login: if session has no `current_org_id` (and no role), load user’s profiles (user_org_role + org + role). If exactly one profile, set it in session (session.insert(ORG_ID, …), session.insert(ROLE_NAME, …)) and return success. If multiple, return success but include a flag e.g. `needs_profile_select: true` so the frontend redirects to profile-select. If zero profiles (shouldn’t happen if every user has personal org), return error or set nothing and let frontend show "no profiles."
- Optional: endpoint `GET /api/auth/needs-profile-select` that returns `{ needs_profile_select: bool }` based on session (authenticated + no profile in session, or profile not in user’s profiles).

### 4.5 Set profile (replace switch_profile)

- **POST /api/auth/set-profile** (or keep **POST /api/auth/switch-profile**): Body `{ org_id, role_id? }`. Verify user has that profile (user_org_role for this user + org_id + role_id). If valid, set session `current_org_id` and `current_role_id`/`current_role_name`. Do **not** update user row. Return 200. Optionally keep user.current_org_id/current_role update as "last used" for UX outside this session.

### 4.6 Register flow

- Already creates user + org + membership + owner. After register, either:
  - Log the user in and set session profile to that org+owner (so they don’t need profile-select), or
  - Redirect to login then profile-select. Prefer: log in and set session profile so they land in dashboard with one profile.

### 4.7 List of call sites to change (backend)

- **auth.rs**: `resolve_permissions`: today uses `user.current_org_id` and `user.current_role` for org-scoped permissions. Change to use profile from session (pass `CurrentProfile` or session).  
- **auth.rs**: `session_json`: return `current_org_id` and role from **session**; if no profile in session, return null and frontend shows profile-select.  
- **auth.rs**: `switch_profile` → `set_profile`: write to session only (and optionally to user row as "last used").  
- **auth.rs**: Login: after login, if no profile in session, set from first profile if len==1, else return `needs_profile_select: true`.  
- **auth.rs**: Register: after creating user+org, log in and set session profile to new org+owner.  
- **auth.rs**: Audit event at login: use profile from session if present, else omit organization_id.  
- **dashboard.rs**: All handlers that use `scope_org_for_dashboard_lists(&user)` or `user.current_org_id`: take `CurrentProfile` (or session) and use that for scope. Replace `scope_org_for_dashboard_lists` with a function that returns `session_profile.org_id` (or None if global scope).  
- **dashboard.rs**: create_user, create_role, add_role_permission, etc.: where we check `user.current_org_id`, use session profile org_id.  
- **ws.rs**: `channels_from_permissions(…, current_org_id)`: pass current org from session, not from user.  
- **lib.rs (router)**: Protect dashboard routes with a layer or extractor that ensures session has profile (or return 403 with a code so frontend redirects to profile-select).

**Deliverables**:
- Session key constants; `CurrentProfile` extractor; set-profile endpoint (session-only).
- Login: set profile in session when 1 profile; return needs_profile_select when >1.
- Register: set session profile after create.
- Replace all `user.current_org_id` / `user.current_role` with session profile in auth, dashboard, ws.
- session_json returns profile from session.

---

## 5. Frontend

### 5.1 Session type and refresh

- Session type: include `current_org_id: string | null`, `current_role_name?: string` (or role_id), and optionally `needs_profile_select: boolean`. After login, if `needs_profile_select` or no `current_org_id`, redirect to profile-select route.

### 5.2 Profile-select page

- **Route**: e.g. `/select-profile` (or `/profile-select`). Shown when user is authenticated but session has no profile (or backend said needs_profile_select). Fetch profiles from `/api/auth/profiles` (existing). List org+role for each; on select, POST to `/api/auth/set-profile` (or switch-profile) with chosen org_id and role_id. On success, refresh session and redirect to `/dashboard`.

### 5.3 Login redirect

- After POST login success: read response for `needs_profile_select`. If true, navigate to `/select-profile`. If false, refresh session and navigate to `/dashboard`. If single profile, backend sets profile in session and returns needs_profile_select: false.

### 5.4 Navbar / profile switcher

- Current navbar already has profile list and calls switch-profile. Change to call set-profile (same API); after success, refresh session so that `current_org_id` and role in session are updated. UI already shows "current" based on session (after we return it from session in session_json).

### 5.5 Guard: dashboard requires profile

- Before rendering dashboard (or before any dashboard API call), if session has no `current_org_id`, redirect to `/select-profile`. So dashboard layout or a wrapper checks session and redirects if profile missing.

### 5.6 Data and UI scoped to profile

- All dashboard API calls already send cookies; backend will use session profile for scoping. No change needed except ensuring backend uses session profile. Frontend shows whatever the API returns (users/roles/etc. for current org or all orgs if global scope).

**Deliverables**:
- Profile-select page (route, fetch profiles, submit set-profile, redirect to dashboard).
- Login flow: handle needs_profile_select; redirect to profile-select or dashboard.
- Session type and session_json contract: current_org_id and role from session.
- Dashboard guard: redirect to profile-select if no profile.
- Navbar: switch profile calls set-profile; refresh session.

---

## 6. Unit tests (backend)

### 6.1 CurrentProfile extractor

- Test: when session has org_id and role, extractor returns them.
- Test: when session has no org_id, extractor rejects (e.g. 403 or specific error).

### 6.2 resolve_permissions with profile from session

- Test: with a user and a given (org_id, role_name) from session, resolve_permissions returns org-scoped permissions for that org/role (mock or real DB). No use of user.current_org_id.

### 6.3 scope_org from session profile

- Replace or extend tests that use `scope_org_for_dashboard_lists(&user)` with a function that takes `CurrentProfile` and returns Option<Uuid>. Unit test that function.

**Deliverables**: Tests for profile extractor (or session profile reader), resolve_permissions with profile, scope helper with profile.

---

## 7. Integration tests

### 7.1 Login and profile in session

- Login as user with one profile: assert session (or next request) has profile set (e.g. GET session returns current_org_id).
- Login as multi@email.com: assert response indicates needs_profile_select or session has no profile until set-profile is called.

### 7.2 Set profile

- POST set-profile with valid org_id+role_id for the user; then GET dashboard/users (or session) and assert response is scoped to that org (e.g. only users from that org, or session shows that org).

### 7.3 Dashboard without profile

- With authenticated session but no profile in session (e.g. clear session profile in test), GET dashboard/users returns 403 or redirect; after set-profile, same request returns 200 and scoped data.

### 7.4 Register

- Register new user; assert they have one org and one profile; assert session (or next request) has profile set so they don’t need profile-select.

**Deliverables**: Integration tests for login (1 vs multiple profiles), set-profile, dashboard without profile, register with profile set.

---

## 8. End-to-end tests (Playwright)

### 8.1 Profile select after login (multi-profile user)

- Log in as multi@email.com; assert redirect to profile-select page (or equivalent). Select a profile (e.g. Default); assert redirect to dashboard and that dashboard shows data for that org (e.g. users list scoped to Default).

### 8.2 Single-profile user: no profile select

- Log in as viewer@default.org; assert redirect to dashboard (no profile-select) and dashboard shows Default-org data.

### 8.3 Switch profile

- Log in as multi@email.com; select Default; go to dashboard/users; assert visible data. Then switch profile (navbar) to Other; assert dashboard/users updates to show Other-org data (or mixed if global admin).

### 8.4 Profile-limited data and UI

- Log in as org-scoped user (e.g. viewer@default.org), go to dashboard/users; assert only users from current org (no users from Other). Switch profile to Other (if user has two orgs); assert list updates. For global admin, assert they see all orgs.

### 8.5 New user: register and land in dashboard

- Register new user; assert one org created and they land in dashboard (no profile-select) with that org as current.

**Deliverables**: E2E tests for profile-select when multiple profiles, no profile-select when single, switch profile and data update, profile-limited lists, register flow.

---

## 9. Implementation order (suggested)

1. **Session profile storage and extractor**  
   Add session keys; implement CurrentProfile extractor (or equivalent); ensure dashboard can read profile from session (stub or minimal set-profile).

2. **Backend: set-profile and login flow**  
   Implement set-profile (session only); login sets profile when 1 profile, returns needs_profile_select when >1; session_json returns profile from session.

3. **Backend: replace user.current_* with session profile**  
   Migrate auth (resolve_permissions, session_json), dashboard (all handlers), ws to use session profile. Remove or stop using user.current_org_id/current_role for authz.

4. **Seeds**  
   Refactor so every user has at least one org (shared helper or current seed structure); document "first org = personal."

5. **Frontend: profile-select page and login redirect**  
   Add route, API calls, redirect after login when needs_profile_select; dashboard guard when no profile.

6. **Frontend: navbar switch profile**  
   Ensure switch calls set-profile and refreshes session.

7. **Unit tests**  
   Extractor, resolve_permissions with profile, scope helper.

8. **Integration tests**  
   Login/set-profile/dashboard without profile/register.

9. **E2E tests**  
   Profile select, single-profile skip, switch profile, profile-limited data, register.

---

## 10. Out of scope / follow-ups

- Removing `user.current_org_id` and `user.current_role` from schema (migration) once all code uses session.
- Explicit `user.personal_org_id` if product needs "can’t leave personal org" or billing.
- Per-device "remember last profile" (could still be session-based; no extra work for MVP).
