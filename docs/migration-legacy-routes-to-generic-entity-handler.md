# Migration path: legacy routes → generic entity handler

This document describes the relationship between **legacy** REST routes (`/api/users`, `/api/organizations`, dashboard routes) and the **generic entity handler** (`/api/entities/:entity_id`), and how to migrate or keep legacy routes as official.

**Related:** [02-entity-based-permissions](technical-choices/02-entity-based-permissions.md), [03-entity-registry](technical-choices/03-entity-registry.md), [05-single-generic-rest-handler](technical-choices/05-single-generic-rest-handler.md).

---

## 1. Route overview

### 1.1 Legacy routes (current)

| Route pattern | Methods | Permission (legacy) | Notes |
|---------------|---------|--------------------|--------|
| `/api/users` | GET, POST | `dashboard.users.read` / `dashboard.users.write` | List/create users; scope from headers (org or global). |
| `/api/users/me` | GET | (self or token) | Current user; no entity permission. |
| `/api/users/{id}` | GET, PATCH, DELETE | `dashboard.users.read` / write or self | Single user. |
| `/api/users/{id}/organizations` | GET | (user's orgs) | User's memberships; no direct entity equivalent. |
| `/api/organizations` | GET, POST | `dashboard.organizations.read` / write | List/create organizations. |
| `/api/organizations/{id}` | GET, PATCH, DELETE | org-scoped or global | Single organization. |
| `/api/organizations/{id}/users` | GET, POST | `dashboard.users.read` / write | Org members (list/add). |
| `/api/organizations/{id}/users/{user_id}` | GET, PATCH, DELETE | `dashboard.users.read` / write | Single org member. |
| `/api/organizations/{id}/users/{user_id}/roles` | POST | `dashboard.roles.write` | Assign roles to user in org. |
| `/api/organizations/{id}/roles` | GET, POST | `dashboard.roles.read` / write | Org roles. |
| `/api/organizations/{id}/roles/{role_id}` | GET, PATCH, DELETE | `dashboard.roles.read` / write | Single org role. |
| `/api/organizations/{id}/roles/{role_id}/permissions` | GET, POST, DELETE | `dashboard.permissions.read` / write | Role permissions. |
| `/api/audit-log` | GET | `dashboard.audit.read` | List audit log entries. |
| `/api/audit-log/{id}` | GET | `dashboard.audit.read` | Single audit entry. |
| `/api/permissions` | GET | (unprotected) | List allowed permission keys. |

Scope is applied via `X-Organization-Id` and `X-Role-Id`; with scope, org-scoped permissions apply; without, global permissions (e.g. `dashboard.users.read`) are required where applicable.

**ListQuerySpec:** Legacy list routes do **not** accept the unified query spec (filter, sort, offset, limit). They return the full scoped list with no pagination or filter/sort params. For filter/sort/pagination, use the generic entity API `GET /api/entities/{entity_id}` with query params (see [04-unified-query-specification](technical-choices/04-unified-query-specification.md)). Where the generic handler does not yet implement list for an entity (e.g. user), the legacy route is the only option and returns all scoped rows.

### 1.2 Generic entity routes

| Route pattern | Methods | Permission (entity) | Notes |
|---------------|---------|---------------------|--------|
| `/api/entities/{entity_id}` | GET, POST | `{entity_id}.read` / `.create` | List (with filter/sort/pagination) or create. |
| `/api/entities/{entity_id}/{id}` | GET, PATCH, DELETE | `{entity_id}.read` / `.update` / `.delete` | Get, update, or delete by id. |

- **entity_id** is the registry id (e.g. `user`, `organization`, `role`, `permission`, `audit`). Unknown entity_id → 404.
- Authz uses entity-based permissions (§2) and the same scope-from-headers contract.
- List supports the unified query spec (filter, sort, offset, limit) per Epic 4; expand/include are rejected with 400.

---

## 2. Permission mapping (legacy ↔ entity)

The app supports **both** legacy `dashboard.*` keys and entity `<entity>.<action>` keys during migration. Resolution uses `permission_equivalents()` so that either style grants access.

| Entity permission | Legacy equivalent |
|-------------------|-------------------|
| `organization.read` | `dashboard.organizations.read` |
| `organization.create` / `.update` / `.delete` | `dashboard.organizations.write` |
| `user.read` | `dashboard.users.read` |
| `user.create` / `.update` / `.delete` | `dashboard.users.write` |
| `role.read` | `dashboard.roles.read` |
| `role.create` / `.update` / `.delete` | `dashboard.roles.write` |
| `permission.read` | `dashboard.permissions.read` |
| `permission.create` / `.update` / `.delete` | `dashboard.permissions.write` |
| `audit.read` | `dashboard.audit.read` |

So a client can call either legacy or generic routes with the same role: if the role has `dashboard.users.read`, it is treated as having `user.read` for the generic handler, and vice versa.

---

## 3. Per-route migration mapping

Where a **simple CRUD** operation exists on a single entity table, the generic route is the equivalent. Nested or relationship-only operations have no direct generic equivalent.

### 3.1 Direct equivalents (use generic instead of legacy)

| Legacy | Generic equivalent | Notes |
|--------|--------------------|--------|
| `GET /api/users` | `GET /api/entities/user` | Add scope headers; use query params for filter/sort/pagination. |
| `GET /api/users/{id}` | `GET /api/entities/user/{id}` | When generic handler implements user get. |
| `POST /api/users` | `POST /api/entities/user` | When generic handler implements user create. |
| `PATCH /api/users/{id}` | `PATCH /api/entities/user/{id}` | When generic handler implements user update. |
| `DELETE /api/users/{id}` | `DELETE /api/entities/user/{id}` | When generic handler implements user delete. |
| `GET /api/organizations` | `GET /api/entities/organization` | Same data; filter/sort/pagination via query spec. |
| `GET /api/organizations/{id}` | `GET /api/entities/organization/{id}` | Same. |
| `POST /api/organizations` | `POST /api/entities/organization` | Same. |
| `PATCH /api/organizations/{id}` | `PATCH /api/entities/organization/{id}` | Same. |
| `DELETE /api/organizations/{id}` | `DELETE /api/entities/organization/{id}` | Same. |
| `GET /api/organizations/{id}/roles` | `GET /api/entities/role?filter=[{"field":"org_id","operator":"eq","value":"<id>"}]` | Filter by org_id. |
| (roles by id) | `GET /api/entities/role/{id}` etc. | When generic handler implements role CRUD. |
| `GET /api/audit-log` | `GET /api/entities/audit` | When generic handler implements audit list (read-only). |
| `GET /api/audit-log/{id}` | `GET /api/entities/audit/{id}` | When generic handler implements audit get. |

Implementation status of the generic handler is code-defined; the table above describes the **intended** mapping. Today, the generic handler fully implements CRUD for `organization`; other entities may be list-only or not yet wired.

### 3.2 No direct generic equivalent (keep legacy or add dedicated APIs)

- **`GET /api/users/me`** — Current user; not “get user by id” with a generic entity id. Keep as a dedicated route.
- **`GET /api/users/{id}/organizations`** — User’s memberships/organizations; relationship view. Either keep legacy or add a dedicated “my organizations” or “user memberships” endpoint.
- **`GET/POST /api/organizations/{id}/users`** — Org members (membership + user); list is “users in this org”, add is “add user to org”. Generic `GET /api/entities/user` with filter `org_id` (if exposed) can approximate list only if the backing model supports it; “add to org” is membership, not user create. Keep legacy for membership semantics or add a dedicated membership API.
- **`GET/PATCH/DELETE /api/organizations/{id}/users/{user_id}`** — Org-scoped user view and update/remove from org. Keep legacy or a dedicated org-member API.
- **`POST /api/organizations/{id}/users/{user_id}/roles`** — Assign roles to a user in an org (junction). No single-entity create; keep legacy or dedicated role-assignment API.
- **`GET/POST/DELETE /api/organizations/{id}/roles/{role_id}/permissions`** — Role–permission junction. Keep legacy or dedicated permission-assignment API.
- **`GET /api/permissions`** — List of allowed permission keys (derived from registry). Unprotected, metadata; keep as-is.

---

## 4. Migration options

### Option A: Migrate clients to generic routes only (for simple CRUD)

- **When:** You want a single, consistent API surface and are willing to change clients.
- **Steps:**
  1. For list/get/create/update/delete on **user**, **organization**, **role**, **permission**, **audit**: switch clients to `GET/POST /api/entities/{entity_id}` and `GET/PATCH/DELETE /api/entities/{entity_id}/{id}`.
  2. Use the same scope headers and entity-based permissions (or legacy equivalents; both work during transition).
  3. Keep legacy routes only for operations that have **no** generic equivalent (§3.2): e.g. `/api/users/me`, org members, role assignment, permission assignment, `/api/permissions`.
- **Result:** Simple CRUD goes through the generic handler; relationship/junction and “me” stay on legacy (or future dedicated endpoints).

### Option B: Keep legacy as official (maintain both)

- **When:** You need backward compatibility and don’t want to change existing clients.
- **Approach:** Keep both legacy and generic routes registered. Document that:
  - Legacy routes remain the **official** (or default) API for users and orgs; generic routes are **alternative** for new clients or tooling.
  - Same permissions (via equivalence) and scope apply to both.
- **Result:** No client migration; new code can choose generic for consistency or legacy for parity with existing behavior.

### Option C: Deprecate legacy and remove after transition

- **When:** You want to converge on the generic handler and reduce maintenance.
- **Steps:**
  1. Announce deprecation and a timeline; document the mapping (this doc) and recommend generic routes for new usage.
  2. Migrate clients to generic routes (and to dedicated endpoints for §3.2 where needed).
  3. Optionally add deprecation headers or a deprecation notice in responses for legacy routes.
  4. After the transition period, remove legacy route registration for operations that are fully covered by generic (and any dedicated replacements for §3.2).
- **Result:** Single set of routes for entity CRUD; fewer code paths and clearer API surface.

---

## 5. Dashboard and frontend

- **Dashboard routes** in this doc refer to the **backend** REST routes that use `dashboard.*` permissions (e.g. `/api/users`, `/api/organizations`). The **frontend** “dashboard” (UI pages) can keep calling either legacy or generic endpoints; permission checks use the same resolution (entity + legacy equivalents).
- **Scope:** Both legacy and generic APIs use the same scope contract (headers). No change required for scope when switching a client from legacy to generic for the same operation.

---

## 6. Summary

| Topic | Guidance |
|-------|----------|
| **Generic routes** | `GET/POST /api/entities/{entity_id}`, `GET/PATCH/DELETE /api/entities/{entity_id}/{id}`; entity_id from registry (e.g. user, organization, role, permission, audit). |
| **Permissions** | Entity `<entity>.<action>` and legacy `dashboard.*` are equivalent during migration; same scope. |
| **Migrate** | Replace legacy list/get/create/update/delete for a single entity with the corresponding generic route where the generic handler implements that entity. |
| **Keep legacy** | For `/api/users/me`, org members, role/permission assignment, and `/api/permissions`; or keep all legacy as official (Option B). |
| **Deprecate** | Option C: move clients to generic (and dedicated endpoints for §3.2), then remove legacy route registration. |
