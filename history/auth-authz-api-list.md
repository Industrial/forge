# Full authc/authz under `/api/auth` (flat REST)

This document lists **all functionalities needed for authentication and authorization to function correctly**, suitable for exposing under `/api/auth` with **flat REST** (no nested resources like `/api/organizations/{org_id}/roles/{role_id}/permissions`).

---

## 1. Authentication (authc) — already under `/api/auth`

| Method | Path | Purpose |
|--------|------|---------|
| POST | `/api/auth/register` | Create account (email + password) |
| POST | `/api/auth/login` | Session login (cookie) |
| GET/POST | `/api/auth/logout` | Clear session |
| GET | `/api/auth/me` | Current user + profiles + resolved permissions (uses `resolve_permissions`, scope from headers) |
| GET | `/api/auth/profiles` | List (org, role) profiles for scope picker |
| POST | `/api/auth/tokens` | Create API token (Bearer) for machine access |
| GET | `/api/auth/admin` | Check admin (is_admin or global permission) |

**Backend support that must remain:**

- `db::auth::Backend` — password auth for login
- `db::token_lookup` — Bearer token → user (used by app’s token auth)
- `resolve_permissions(db, user, scope)` — builds permission list from `user_global_role` + `role_permission` (+ scope)
- `has_global_scope(db, user, permission_key)` — for global-scope checks
- Scope from headers: `X-Organization-Id`, `X-Role-Id` (e.g. `ScopeFromHeaders`, `try_scope_from_headers`)
- Guards: `require_permission`, `require_entity_permission` (used by handlers that protect authz data)

---

## 2. Authorization (authz) — data that feeds permission resolution

Permission resolution uses: **user**, **user_global_role**, **user_org_role**, **org_role**, **role_permission**, **organization**, **membership**. The following endpoints provide the data needed for authz to work. All can be flat under `/api/auth/*`.

### 2.1 Users (identity + org membership + org roles)

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/api/auth/users` | List users (optional query `?org_id=...`) |
| POST | `/api/auth/users` | Create user with org membership and org roles (body: email, password, org_id, role_ids) |
| PATCH | `/api/auth/users/{id}` | Update user (email, is_active; optionally membership/roles in body) |
| DELETE | `/api/auth/users/{id}` | Delete user |

*Currently implemented as dashboard: `list_users`, `create_user`, `update_user`, `delete_user`.*

### 2.2 Organizations

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/api/auth/organizations` | List organizations |
| POST | `/api/auth/organizations` | Create organization |
| PATCH | `/api/auth/organizations/{id}` | Update organization |
| DELETE | `/api/auth/organizations/{id}` | Delete organization |

*Can be shared with or replaced by generic entity `organization` if desired; for authz we only need CRUD so orgs exist for scope.*

### 2.3 Roles (org_role)

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/api/auth/roles` | List roles (query `?org_id=...` for org-scoped) |
| POST | `/api/auth/roles` | Create role (body: org_id, name, …) |
| PATCH | `/api/auth/roles/{id}` | Update role |
| DELETE | `/api/auth/roles/{id}` | Delete role |

*Currently: dashboard `list_roles`, `create_role`, `update_role`, `delete_role`; and `list_org_roles` under `/api/organizations/{org_id}/roles`. Flat version: single list with `org_id` query.*

### 2.4 Role–permission assignments (role_permission)

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/api/auth/role-permissions` | List assignments (query `?scope=global` or `?org_id=...&role_name=...`) |
| POST | `/api/auth/role-permissions` | Add assignment (body: scope, org_id?, role_name, permission_key) |
| DELETE | `/api/auth/role-permissions` | Remove assignment (query or body: id, or scope+org_id+role_name+permission_key) |

*Currently: `get_role_permissions`, `post_role_permission`, `delete_role_permission_by_path` under `/api/organizations/{org_id}/roles/{role_id}/permissions`. Flat version: no nesting.*

### 2.5 Permission keys (code-defined)

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/api/auth/permissions` | List known permission keys (e.g. dashboard.users.read, entity.read, …) |

*Currently: dashboard `list_permissions` at GET `/api/dashboard/permissions`.*

### 2.6 Global role assignments (user_global_role) — optional but recommended

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/api/auth/global-role-assignments` | List (query `?user_id=...`) |
| POST | `/api/auth/global-role-assignments` | Assign global role to user (body: user_id, role_name) |
| DELETE | `/api/auth/global-role-assignments` | Remove (query or body: user_id, role_name) |

*Currently no REST endpoint; only seeds. Needed for assigning e.g. `platform_admin` without touching DB directly.*

---

## 3. Out of scope for “authc/authz only”

- **Audit log** — GET dashboard/audit-log: product feature, not required for auth to function.
- **Tasks** — GET dashboard/tasks: same.
- **Other dashboard UI endpoints** — can remain under `/api/dashboard` or be migrated later; the above list is the **minimum** for full authc/authz under `/api/auth` with flat REST.

---

## 4. Summary table (flat `/api/auth` only)

| Area | Endpoints (flat) |
|------|-------------------|
| **Authc** | register, login, logout, me, profiles, tokens, admin |
| **Users** | GET/POST /api/auth/users, PATCH/DELETE /api/auth/users/{id} |
| **Organizations** | GET/POST /api/auth/organizations, PATCH/DELETE /api/auth/organizations/{id} |
| **Roles** | GET/POST /api/auth/roles, PATCH/DELETE /api/auth/roles/{id} (org_id in query/body) |
| **Role–permissions** | GET/POST/DELETE /api/auth/role-permissions (scope/org_id/role_name in query/body) |
| **Permission keys** | GET /api/auth/permissions |
| **Global roles** | GET/POST/DELETE /api/auth/global-role-assignments (user_id/role_name in query/body) |

All of the above can be implemented without nested paths; org and role are specified by query parameters or request body where needed.
