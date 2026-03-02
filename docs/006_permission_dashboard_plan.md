# Permission-based dashboard and role–permission CRUD

**Part of:** Phase 6 (Authorization) and auth/authz feature set. See `006_authorization.md` for the base framework. This plan adds a **permission** layer on top of roles: permissions are assigned to roles and resolved per request; handlers guard by permission.

---

## 1. Overview

- **Goal**: One dashboard for all users; visibility and API access are driven by an **explicit list of permissions** (Option B from the design discussion). Permissions are **assigned to roles**; that assignment is **CRUD-able on `/dashboard/permissions`** (roles-and-permissions page). Who can see and change permissions is itself gated by a permission.
- **Scope**: Backend and frontend fully implemented; seeds provide enough users and role–permission data to test all cases. Feature is “done” only when both backend and frontend are complete and testable.

### Principles

- **Single dashboard**: One app, one layout; no separate “admin” vs “org” dashboards.
- **Backend is source of truth**: Session (or a small API) returns the current user’s **permission list**; frontend uses it only for UX (nav, route guards). Every sensitive API checks permissions and returns 403 when missing.
- **Bootstrap**: Global admin (`is_admin`) can do everything (seeded). Org admins can do everything for their org; other org roles have reduced permission sets. All role–permission assignments are seedable and later editable on the permissions page.

---

## 2. Definitions

| Term | Meaning |
|------|--------|
| **Permission** | A string key that gates visibility or an action (e.g. `dashboard`, `dashboard.organizations`, `dashboard.users`, `dashboard.permissions.manage`). Defined in code (constant list); not CRUD at runtime. |
| **Role** | Existing org roles: `owner`, `admin`, `editor`, `viewer` (stored on `membership.role`). Virtual global role: “platform_admin” for users with `is_admin` (no separate DB role row). |
| **Scope** | `org` or `global`. Org-scoped role–permission: “role X in an organization has permission Y.” Global: “platform_admin has permission Z” (used when `user.is_admin`). |
| **Role–permission assignment** | A row (scope, role_name, permission_key) meaning “this role in this scope has this permission.” CRUD on these rows is the main feature of the permissions page. |

---

## 3. Permission set (code-defined)

Suggested permission keys (to be implemented as a constant list in the app or shared config):

| Key | Purpose |
|-----|--------|
| `dashboard` | Can see dashboard at all (base). |
| `dashboard.organizations` | Can see “Organizations” nav and list/manage all organizations (global only). |
| `dashboard.users` | Can see “Users” nav and list/manage users (in current org for org scope; all for global). |
| `dashboard.permissions.manage` | Can see “Permissions” nav and CRUD role–permission assignments. |

Additional keys can be added later (e.g. `dashboard.billing`); the backend and frontend must both know the list (e.g. from a shared constant or config).

---

## 4. Backend implementation

### 4.1 Schema

- **New migration**: Table `role_permission`:
  - `scope`: string or enum (`'org'` \| `'global'`).
  - `role_name`: string (e.g. `owner`, `admin`, `editor`, `viewer`, `platform_admin`).
  - `permission_key`: string.
  - Unique on `(scope, role_name, permission_key)`.
- **No new “permission” table** for definitions: permission keys are defined in code and optionally exposed via an API (e.g. GET list of known permissions).

### 4.2 Resolving the current user’s permissions

- If `user.is_admin` → return **all** permission keys (platform_admin behavior).
- Else → query `role_permission` where `scope = 'org'` and `role_name = user.current_role` (string from `user.current_role`), return distinct `permission_key` list.
- Use this list in session and in API guards.

### 4.3 Session API

- Extend `GET /api/auth/session` response to include:
  - `permissions`: array of strings (e.g. `["dashboard", "dashboard.users", "dashboard.organizations", "dashboard.permissions.manage"]` for admin; org viewer might get `["dashboard"]` only).
- Existing fields (`user`, `profiles`, `flash`) unchanged. Do **not** remove `is_admin` from the response if it is added for convenience; permission list is the authoritative view for “what can the user do?”

### 4.4 Role–permission CRUD API

- **List assignments**: e.g. `GET /api/dashboard/role-permissions` (or nested under a resource). Returns list of `{ scope, role_name, permission_key }`. Guard: caller must have `dashboard.permissions.manage`.
- **List known permissions**: e.g. `GET /api/dashboard/permissions` (or part of role-permissions response). Returns the code-defined list of permission keys. Guard: `dashboard.permissions.manage`.
- **List roles (for UI)**: Can be derived from code (org: owner, admin, editor, viewer; global: platform_admin) or a small config. No separate “role” table required for this plan.
- **Add assignment**: e.g. `POST /api/dashboard/role-permissions` with body `{ scope, role_name, permission_key }`. Guard: `dashboard.permissions.manage`; validate scope/role_name/permission_key.
- **Remove assignment**: e.g. `DELETE /api/dashboard/role-permissions` with body or query identifying `(scope, role_name, permission_key)`. Guard: `dashboard.permissions.manage`.

All these endpoints must return 403 when the caller lacks `dashboard.permissions.manage`.

### 4.5 Guarding existing dashboard APIs

- Any API that today is “admin only” or “org-scoped” should be guarded by the appropriate **permission**:
  - List/manage all organizations → require `dashboard.organizations` (and enforce only for non-org-scoped context or when is_admin).
  - List/manage users → require `dashboard.users` (backend enforces org scope when not global admin).
  - Role–permission CRUD → require `dashboard.permissions.manage`.
- Return 403 and optionally audit when permission is missing.

### 4.6 Seeds

- **Role–permission seeds** (run after migration):
  - **Global**: `(scope=global, role_name=platform_admin, permission_key)` for every permission key. (In practice: when resolving permissions for `is_admin`, return all keys; seeds for platform_admin can still be inserted for consistency and for the permissions page to show them.)
  - **Org**:
    - `owner`: all org-relevant permissions (`dashboard`, `dashboard.users`, `dashboard.permissions.manage`; do **not** grant `dashboard.organizations` to org roles).
    - `admin`: same as owner for org scope (everything for their org, including `dashboard.permissions.manage`).
    - `editor`: e.g. `dashboard`, `dashboard.users` (read-only or limited as desired).
    - `viewer`: e.g. `dashboard` only.
- **User seeds** (existing, keep and adjust if needed):
  - `admin@admin.com`: `is_admin = true` (sees all; no need to assign platform_admin in a user–role table; resolution uses `is_admin`).
  - `owner@default.org`, `orgadmin@default.org`, `editor@default.org`, `viewer@default.org`: org “Default” with roles owner, admin, editor, viewer.
  - `owner@other.org`, `orgadmin@other.org`, `viewer@other.org`: org “Other.”
  - `multi@email.com`: viewer in Default, editor in Other (for org switching).
- Ensure enough variety to test: global admin sees everything; org admin/owner see org-scoped only (no “Organizations”); editor/viewer see reduced nav; permissions page visible only when user has `dashboard.permissions.manage`.

---

## 5. Frontend implementation

### 5.1 Session and types

- Extend session type to include `permissions: string[]` (from `GET /api/auth/session`).
- Use this list everywhere for visibility and route guards; do not duplicate permission logic (e.g. avoid “if is_admin then show X” except for optional shortcuts; prefer “if permissions includes X then show X”).

### 5.2 Sidebar (nav)

- Define nav items with a **required permission** per item (e.g. “Organizations” requires `dashboard.organizations`; “Users” requires `dashboard.users`; “Permissions” requires `dashboard.permissions.manage`).
- Render only nav items whose required permission is in `session.permissions`.
- Base “Dashboard” (home) can require `dashboard`.

### 5.3 Route guards

- For routes under `/dashboard`:
  - **Dashboard index** (`/dashboard`): require `dashboard`; redirect or show “Forbidden” if missing.
  - **Organizations** (`/dashboard/organizations`): require `dashboard.organizations`.
  - **Users** (`/dashboard/users`): require `dashboard.users`.
  - **Permissions** (`/dashboard/roles-and-permissions`): require `dashboard.permissions.manage`.
- If the user navigates directly to a path they don’t have permission for (e.g. manual URL), redirect to `/dashboard` or show a “You don’t have access” view; do not rely only on hiding the link (API would still return 403).

### 5.4 Permissions page (CRUD)

- **List view**: Show roles (by scope: global vs org) and for each role the set of assigned permissions. Data from `GET /api/dashboard/role-permissions` (and optionally `GET /api/dashboard/permissions` for the full list of keys).
- **Add assignment**: Form or control to pick scope, role, and permission; submit to `POST /api/dashboard/role-permissions`. Only show this to users who have `dashboard.permissions.manage` (page itself is already guarded).
- **Remove assignment**: Control per row to remove `(scope, role_name, permission_key)`; call `DELETE /api/dashboard/role-permissions`.
- Handle loading and errors (403, 422, network). Do not allow editing permission **definitions** (the list of keys); only assignments.

### 5.5 Consistency with backend

- If an API returns 403, show a clear message or redirect; do not assume the frontend’s permission list is always in sync (e.g. after a role change elsewhere). Optional: refresh session after critical actions so that `permissions` stays up to date.

---

## 6. Testing and “done” criteria

### 6.1 Backend

- [ ] Migration adds `role_permission` and is runnable.
- [ ] Session response includes `permissions`; global admin gets all; org users get permissions for their current role only.
- [ ] CRUD API for role–permission: list, add, remove; all guarded by `dashboard.permissions.manage`.
- [ ] Dashboard-related APIs (organizations, users, etc.) are guarded by the correct permissions and return 403 when missing.
- [ ] Seeds: default role–permission rows + existing user seeds; at least one global admin, org owner/admin/editor/viewer in two orgs, and a multi-org user.

### 6.2 Frontend

- [ ] Session type and fetch include `permissions`.
- [ ] Sidebar shows only nav items the user has permission for.
- [ ] Route guards enforce permissions for `/dashboard/*` and redirect or show Forbidden when missing.
- [ ] Permissions page: list assignments, add, remove; only accessible with `dashboard.permissions.manage`.

### 6.3 Manual / E2E

- [ ] Log in as `admin@admin.com`: all nav items visible; can open Organizations, Users, Permissions; can change role–permission assignments.
- [ ] Log in as org admin (e.g. `orgadmin@default.org`): no “Organizations” nav; has “Users” and “Permissions” (if seeded); cannot access `/dashboard/organizations` (redirect or 403).
- [ ] Log in as viewer (e.g. `viewer@default.org`): only “Dashboard” (and any other granted items); no Permissions page; direct URL to `/dashboard/organizations` or `/dashboard/roles-and-permissions` is blocked.
- [ ] After changing a role’s permissions (e.g. remove `dashboard.users` from viewer), log in as that role and confirm “Users” is hidden and API returns 403.

---

## 7. Implementation order (suggested)

1. **Backend**: Migration + SeaORM entity for `role_permission`; permission key list in code; resolve function (is_admin → all; else org role_permission); extend session handler to include `permissions`.
2. **Backend**: CRUD handlers for role–permission (list, add, remove) with `dashboard.permissions.manage` guard; wire routes.
3. **Backend**: Guard existing dashboard APIs by permission; add seeds for role_permission and verify user seeds.
4. **Frontend**: Session type + sidebar filtering by `permissions`; route guards for `/dashboard/*`.
5. **Frontend**: Permissions page: list, add, remove assignments; error handling and 403 handling.
6. **E2E**: Update existing e2e tests for permission-based dashboard scenarios.

---

## 8. References

- Design discussion: one dashboard; Option B (explicit permission list); CRUD = assignment of permissions to roles; bootstrap via seeds; global admin and org hierarchy.
- Existing docs: `006_authorization.md` (AuthzContext, Role, Action, Shallow Gate, Deep Scope, ForgePolicy). This plan adds a **permission** layer on top of roles: permissions are assigned to roles and resolved per request; handlers guard by permission instead of (or in addition to) role.
