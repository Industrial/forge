# Seed data via backend API only — plan

This document lists all current seed data, how each would be replaced by an API call, and how those calls remain secure in production.

---

## 1. Complete list of seeds and data entries

### 1.1 `s20220101_000001_seed_users` (user seed)

| # | Data created | Tables / entities | Notes |
|---|--------------|-------------------|--------|
| 1 | **Organizations** (3) | `organization` | Names: Default (slug `default`), Other (slug `other`), Personal (slug `personal`). |
| 2 | **Org roles per org** (4 per org) | `org_role` | For each org: `owner`, `admin`, `editor`, `viewer`. Created when org is created. |
| 3 | **User: admin@admin.com** | `user`, `membership`, `user_org_role` | `is_admin = true`. One membership: Default org, role `owner`. |
| 4 | **User: owner@default.org** | `user`, `membership`, `user_org_role` | One membership: Default org, role `owner`. |
| 5 | **User: orgadmin@default.org** | `user`, `membership`, `user_org_role` | One membership: Default org, role `admin`. |
| 6 | **User: editor@default.org** | `user`, `membership`, `user_org_role` | One membership: Default org, role `editor`. |
| 7 | **User: viewer@default.org** | `user`, `membership`, `user_org_role` | One membership: Default org, role `viewer`. |
| 8 | **User: owner@other.org** | `user`, `membership`, `user_org_role` | One membership: Other org, role `owner`. |
| 9 | **User: orgadmin@other.org** | `user`, `membership`, `user_org_role` | One membership: Other org, role `admin`. |
| 10 | **User: viewer@other.org** | `user`, `membership`, `user_org_role` | One membership: Other org, role `viewer`. |
| 11 | **User: multi@email.com** | `user`, `membership`, `user_org_role` | Three memberships: Personal (viewer), Default (viewer), Other (editor). |

All users share password `password` (constant `SEED_PASSWORD`).  
`user.current_org_id` / `user.current_role` are deprecated; scope is provided by request headers (`X-Organization-Id`, `X-Role-Name`) when calling the API.

---

### 1.2 `s20220101_000002_seed_role_permissions` (role_permission seed)

| # | Data created | Tables / entities | Notes |
|---|--------------|-------------------|--------|
| 1 | **Legacy migration** | `role_permission` | Replace old keys (`dashboard.organizations`, `dashboard.users`, `dashboard.permissions.manage`) with `.read`/`.write`; delete old rows, insert new. Idempotent. |
| 2 | **Global role permissions** | `role_permission` | Scope `global`, role `platform_admin`, org_id `null`, all keys in `PERMISSIONS` (dashboard, dashboard.organizations.read/write, …). |
| 3 | **Per-org role permissions** | `role_permission` | For **each** organization: scope `org`, org_id = that org; for `owner` and `admin`: `ORG_OWNER_ADMIN`; for `editor`: `ORG_EDITOR`; for `viewer`: `ORG_VIEWER` **except** if org slug is `personal`, then viewer gets only `dashboard`. |

So: org creation already triggers `seed_role_permissions_for_org` in the backend (when creating an org via POST /api/dashboard/organizations). The **user seed** does **not** create `role_permission`; the **role_permissions seed** runs after the user seed and, for every org in the DB, seeds these permission rows. The only special case is `personal` org where viewer gets only `dashboard`.

---

### 1.3 `s20220101_000003_seed_user_global_roles` (global role seed)

| # | Data created | Tables / entities | Notes |
|---|--------------|-------------------|--------|
| 1 | **Global role for admin** | `user_global_role` | User `admin@admin.com` → role `platform_admin`. |

No other global roles are seeded.

---

## 2. Replacing each with an API call

### 2.1 Organizations and org roles (current: direct insert in user seed)

- **Current:** `ensure_org(db, name, slug)` inserts `organization` and then `ensure_org_roles` inserts four `org_role` rows. No API today that creates an org “by name/slug” without an authenticated user.
- **Replace with:** **POST /api/dashboard/organizations** with body `{ "name": "Default", "slug": "default" }` (and same for Other, Personal).  
- **Catch:** This endpoint requires an authenticated user with `dashboard.organizations.write`. So we need **at least one user** who can create orgs before we can create Default/Other/Personal. That implies either:
  - **Bootstrap user:** One user is created **outside** the “API-only” rule (e.g. a single bootstrap insert or a dedicated “seed admin” creation path), then all other data is done via API; or
  - **Register then create orgs:** First call **POST /api/auth/register** for e.g. `admin@admin.com`; register creates that user and their **personal** org (e.g. “admin@admin.com's workspace”). Then we need a way for that user to **create additional** orgs (Default, Other, Personal). So we need the first user to have `dashboard.organizations.write` in their **personal** org. That would require assigning permissions to the **owner** role of the org created by register (today register creates org + owner; role_permissions for that org are created by `seed_role_permissions_for_org`, which is called from the create_organization handler and from the role_permissions seed). So if the first user is created via register, their personal org gets org_roles and (when we run role_permissions seed) role_permissions. So after **role_permissions seed** runs, that user’s org has owner with ORG_OWNER_ADMIN — but that does **not** include `dashboard.organizations.read/write` (those are not in ORG_OWNER_ADMIN; they’re platform-only in the template). So we cannot get “create organizations” from register alone without adding those permissions to owner or adding a “seed bootstrap” step.
- **Practical approach:** Introduce a **seed-only** or **bootstrap** mechanism that is only allowed when e.g. no users exist (or when an env flag is set), and that creates exactly one user (admin) and optionally the three orgs, so that all other seed steps can be expressed as API calls using that admin. Alternatively, add a **POST /api/auth/register** option (e.g. query or body) that sets `is_admin = true` and is only accepted when the app is “empty” or when a secret/env is set, so the first user can create orgs via existing dashboard APIs.

So for “organizations and org roles”:

- **Proposed:** Use **POST /api/dashboard/organizations** to create Default, Other, Personal.  
- **Prerequisite:** Seed runs as (or as-if) an authenticated user who has `dashboard.organizations.write`. That implies either a bootstrap user created by a one-off mechanism, or register extended with a controlled way to create the first admin (see below).

---

### 2.2 Users and memberships (current: direct insert in user seed)

- **Current:** For each user, insert `user`, then for each (org, role) insert `membership` and `user_org_role`. Password is hashed with `hash_password(SEED_PASSWORD)`.
- **Replace with:**
  - **First user (admin):** Either **POST /api/auth/register** with body `{ "email": "admin@admin.com", "password": "password" }` plus a **controlled** way to set `is_admin = true` (only when DB has no users or when a seed secret is provided), or a dedicated **bootstrap** endpoint that creates this single user and returns a session/token for subsequent seed calls.
  - **Other users in one org:** **POST /api/dashboard/users** with body `{ "email": "...", "password": "password", "org_id": "<uuid>", "role_ids": [<uuid of role>] }`. Requires an authenticated user with `dashboard.users.write` (e.g. the bootstrap admin). This creates one user and one membership in that org.
  - **Adding the same user to more orgs (multi@email.com):** There is **no** current API for “add existing user to another org”. We need **POST /api/dashboard/organizations/:org_id/members** (or similar) with body e.g. `{ "user_id": "<uuid>", "role_id": "<uuid>" }` to create a second membership and `user_org_role` for an existing user. So this is a **new** API.

Summary:

- **admin@admin.com:** Register (with optional is_admin/bootstrap) or bootstrap endpoint.
- **owner@default.org, orgadmin@default.org, editor@default.org, viewer@default.org:** POST /api/dashboard/users (org_id = Default, role_ids = [owner|admin|editor|viewer]).
- **owner@other.org, orgadmin@other.org, viewer@other.org:** POST /api/dashboard/users (org_id = Other, role_ids = [owner|admin|viewer]).
- **multi@email.com:**  
  - Option A: Create with POST /api/dashboard/users in Personal (viewer), then **new** “add member to org” API to add them to Default (viewer) and Other (editor).  
  - Option B: Create with POST /api/dashboard/users in Default (viewer), then add-member API to add to Personal (viewer) and Other (editor). Personal must exist and have a “viewer” role with only `dashboard` permission (see 2.4).

---

### 2.3 Role permissions (current: role_permissions seed)

- **Current:**  
  - Legacy: delete old permission keys, insert .read/.write.  
  - Global: insert rows for scope=global, role=platform_admin, all PERMISSIONS.  
  - Per-org: for each org, insert rows for owner/admin (ORG_OWNER_ADMIN), editor (ORG_EDITOR), viewer (ORG_VIEWER or, for slug=personal, only dashboard).

- **Replace with:**
  - **Legacy migration:** Can remain a one-off DB migration or a small “migrate legacy permissions” internal step (not necessarily an HTTP API).
  - **Global permissions for platform_admin:** There is **no** current API that creates **global** role_permission rows (scope=global, org_id=null). Dashboard **POST /api/dashboard/role-permissions** is for adding a single role–permission; we’d need to allow scope=global and org_id=null and restrict that to a super-admin or seed context. So either: **new** endpoint (e.g. **POST /api/dashboard/global-role-permissions** or extend role-permissions to allow global scope) gated by a very high privilege, or a **seed-only** internal function that runs only when “seeding” is enabled.
  - **Per-org permissions:** When we create an org via **POST /api/dashboard/organizations**, the backend already calls `seed_role_permissions_for_org(org_id)`, so **default** org role permissions (owner, admin, editor, viewer) are created by the existing API. No extra API needed for the standard case.
  - **Personal org (viewer = dashboard only):** Today this is a special case in the role_permissions seed (if org.slug == "personal", only add "dashboard" for viewer). Options: (1) **POST /api/dashboard/organizations** for Personal could accept an option like `viewer_minimal: true` that triggers a different permission set; or (2) after creating Personal, seed calls **DELETE** and **POST /api/dashboard/role-permissions** to remove extra viewer permissions and leave only `dashboard`; or (3) a dedicated “apply template” for an org (e.g. “minimal”) that sets viewer to dashboard-only. So either extend create_organization or add a small “template”/patch step via existing role-permissions API.

Summary:

- **Global platform_admin permissions:** New or extended API (e.g. global role-permissions) or seed-only internal step.
- **Per-org (default) permissions:** Already done by **POST /api/dashboard/organizations** (backend calls `seed_role_permissions_for_org`).
- **Personal org viewer = dashboard only:** Either option on create_organization, or multiple **POST/DELETE /api/dashboard/role-permissions** calls after creating Personal.

---

### 2.4 User global role (current: user_global_roles seed)

- **Current:** Insert one row into `user_global_role`: user = admin@admin.com, role_name = platform_admin.
- **Replace with:** There is **no** current API to assign a **global** role to a user. We need something like **POST /api/dashboard/users/:user_id/global-roles** with body `{ "role_name": "platform_admin" }` (or a single “assign platform_admin” endpoint). This must be restricted to a super-admin or bootstrap context so that in production only authorized actors can grant global roles.

---

## 3. Security in production (per API / capability)

How each kind of call stays secure once the app is launched in production.

| API / capability | How it stays secure in production |
|------------------|-----------------------------------|
| **POST /api/auth/register** | Same as today: no auth required; anyone can register. No `is_admin` in body unless we add a **restricted** path (e.g. only when `FORGE_SEED_BOOTSTRAP_KEY` is set and DB has no users). So: normal register stays safe; optional bootstrap path is disabled without the secret/env. |
| **Bootstrap / “first admin” endpoint** (if added) | Only available when e.g. no user exists in DB (or when a strong secret/env is present). Disabled or 404 in production once the first admin exists. Rate-limited and logged. |
| **POST /api/dashboard/organizations** | Already requires authenticated user with `dashboard.organizations.write`. In production, only users who have that permission (e.g. platform_admin or an org admin with that right) can create orgs. No change. |
| **POST /api/dashboard/users** | Already requires authenticated user with `dashboard.users.write` and (for non–global-admin) limits creation to the current org. In production, only admins with that permission can create users. No change. |
| **POST “add user to org”** (new) | Should require authenticated user with e.g. `dashboard.users.write` and (for non–global-admin) limit to orgs they administer. Same permission model as create_user. |
| **POST /api/dashboard/role-permissions** (existing) | Already requires `dashboard.permissions.write` and (for non–global-admin) limits to current org. In production, only users with that permission can add role-permissions. No change. |
| **Global role-permissions or “assign global role”** (new or extended) | Must be restricted to users who are already platform_admin (or a dedicated “super admin” role). In production, only existing global admins can assign global roles or global role-permissions. Optionally require a second factor or audit every such change. |

Summary:

- **Existing** dashboard APIs (organizations, users, role-permissions) stay as they are; they are already protected by auth and permission checks.
- **New** capabilities (bootstrap first admin, add user to org, global role assignment / global role-permissions) must be designed so that in production they are either:
  - Only available when a “seed/bootstrap” mode is enabled (e.g. env or “no users in DB”), or
  - Only callable by users who already have the highest privilege (e.g. platform_admin), with audit and optional extra checks.

---

## 4. Summary table: seed entry → API replacement → security

| Seed data | Replaced by API | Security in production |
|-----------|------------------|------------------------|
| Orgs Default, Other, Personal | POST /api/dashboard/organizations (×3) | Requires auth + `dashboard.organizations.write`. |
| Org roles (4 per org) | Created by POST /api/dashboard/organizations | Same as above. |
| admin@admin.com (is_admin, owner in Default) | Register (with optional is_admin when empty) or bootstrap endpoint; then assign to Default via add-member or create_user in Default | Register: no is_admin unless bootstrap mode. Bootstrap: only when no users or with secret. |
| owner/orgadmin/editor/viewer @default.org | POST /api/dashboard/users (×4) | Requires auth + `dashboard.users.write`. |
| owner/orgadmin/viewer @other.org | POST /api/dashboard/users (×3) | Same. |
| multi@email.com (3 orgs) | POST /api/dashboard/users (1 org) + **new** add-member API (2 more orgs) | create_user: same as above. Add-member: require `dashboard.users.write`, scope to orgs caller can manage. |
| Global role_permission (platform_admin) | **New** global role-permissions API or seed-only step | Only callable by bootstrap or by existing platform_admin; audit. |
| Per-org role_permission | Already done inside POST /api/dashboard/organizations | Same as org creation. |
| Personal org viewer = dashboard only | Option on create_organization or POST/DELETE role-permissions after create | Same as org and role-permissions. |
| user_global_role (admin → platform_admin) | **New** assign-global-role API | Only callable by bootstrap or by existing platform_admin; audit. |

---

## 5. Open decisions

1. **Bootstrap strategy:** Single “bootstrap” endpoint (create first admin + optionally orgs) vs. extending register with a guarded `is_admin` vs. keeping one minimal “insert first user” in seed and doing everything else via API.
2. **Add existing user to org:** Exact route and body (e.g. POST /api/dashboard/organizations/:id/members vs. POST /api/dashboard/users/:id/memberships) and permission model.
3. **Global role and global role-permissions:** One endpoint for “assign global role” and one for “add global role-permission”, or a single “platform admin setup” that is only available in bootstrap mode.
4. **Personal org minimal viewer:** Implement as option on create_organization vs. multiple role-permission API calls after creation.
5. **How the seed runner authenticates:** Seed script runs with a DB connection only today. To “only use API”, the seed runner would need to either (a) build the router and call handlers in-process with a synthetic “bootstrap” or “seed” context, or (b) run against a live server (e.g. localhost) and send HTTP requests with a bootstrap token or a session cookie obtained from a bootstrap login. (a) is easier for “same process” and no network; (b) is true “only API” and forces all logic behind HTTP.

This document is the basis for discussing what to add and how it will work before implementing.
