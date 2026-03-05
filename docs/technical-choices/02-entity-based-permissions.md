# Technical choices: Epic 2 — Entity-based permissions

This document records technical decisions for the second deliverable (entity-based permissions). It fixes the permission model, key format, storage, and resolution rules so that later epics (entity registry, generic handler, RPC, subscriptions) can rely on a single permission system.

**Epic reference:** [02_entity-based-permissions](../deliverables/02_entity-based-permissions.md)

---

## 1. Permission key format

- **Choice:** Permissions use the form `<entity>.<action>` where:
  - **entity** is the stable identifier of an entity type (e.g. user, organization, role). Casing and normalization are defined by the entity registry (Epic 3); for consistency with URLs and storage, a single convention (e.g. lowercase, snake_case) is used everywhere.
  - **action** is exactly one of: `create`, `read`, `update`, `delete`.
- **Implication:** No ad-hoc or page-based keys (e.g. `dashboard.users.read`). Every permission is an entity plus a CRUD action. The backend and frontend use the same key format; the list of valid keys is derived from the list of entities (see §4).

---

## 2. Wildcard permissions: all.read and all.write

- **Choice:** The wildcard permissions `all.read` and `all.write` are retained. A role that has `all.read` is treated as having every `<entity>.read` for resolution and authorization; a role that has `all.write` is treated as having every `<entity>.create`, `<entity>.update`, and `<entity>.delete`.
- **Implication:** Resolution (e.g. “effective permissions for this user in this scope”) expands `all.read` / `all.write` when checking a specific entity action, or the check logic treats “has all.read” as “has *.read”. Backend and frontend use the same rule so that UI and API stay in sync. No other wildcards (e.g. per-entity “*.read”) are required for this epic.

---

## 3. Scope of permissions (global vs org)

- **Choice:** The existing storage model is kept. Permissions are assigned to roles in one of two scopes:
  - **Global:** Role is a global role (e.g. via `user_global_role`); permission rows have scope=global and no org. The user has that permission regardless of request scope.
  - **Org:** Role is an org-scoped role (the role id in the request scope); permission rows have scope=org and the org id. The user has that permission only when the request is in that org (and with that role).
- **Implication:** No change to the `role_permission` table’s scope/org_id/role_name structure. Only the **permission_key** values change from the current dashboard-style keys to `<entity>.<action>`. Resolution continues to union global permissions and org-scoped permissions for the current request scope.

---

## 4. Source of the set of permission keys

- **Choice:** The set of allowed permission keys is **derived from the set of entities**. For each entity in that set, the four keys `<entity>.create`, `<entity>.read`, `<entity>.update`, `<entity>.delete` are the only valid keys for that entity. The union over all entities (plus `all.read` and `all.write`) is the full allowed set. In Epic 2, the “set of entities” may be a code-defined list; Epic 3 will replace this with the entity registry as the single source of truth.
- **Implication:** Any API that lists “available permissions” (e.g. for role-assignment UI) or that validates “permission_key” on add must use this derived set. There is no separate, hand-maintained permission list. Adding or removing an entity (in the list or registry) automatically changes the allowed keys.

---

## 5. Dashboard / app entry (no separate “dashboard” permission)

- **Choice:** There is no dedicated “dashboard” or “app” permission. Access to the dashboard (or any “main app” area) is granted if the user has **at least one entity permission** in the current scope (e.g. at least one `<entity>.read` or any action). So “can open the app” is “has any permission in scope” (or equivalent: non-empty permission list after resolution).
- **Implication:** The frontend and backend treat “can access dashboard” as “user has at least one resolved permission in the current scope”. No special key like `dashboard` or `app.read` is required. Sidebar and route guards use entity permissions (e.g. show “Users” if user has `user.read`, etc.); the dashboard layout is shown if the user has any permission.

---

## 6. Authorization check (single rule everywhere)

- **Choice:** Every data operation (list, get by id, create, update, delete) is gated by one rule: the identity must have the corresponding entity permission in the current scope. Concretely:
  - **List / get:** requires `<entity>.read`
  - **Create:** requires `<entity>.create`
  - **Update:** requires `<entity>.update`
  - **Delete:** requires `<entity>.delete`
  The same rule applies on REST, and later on RPC and subscriptions (a subscriber may only subscribe to queries for entities they have `<entity>.read` for).
- **Implication:** One shared authorization helper or layer that, given (user, scope, entity, action), returns allow/deny using the resolved permissions (including all.read / all.write expansion). All handlers (current and future generic handler) use it. No per-endpoint permission constants beyond the entity and action.

---

## 7. Frontend permission check (parity with backend)

- **Choice:** The client receives the resolved permission list (e.g. from `/api/auth/me`) and uses the same semantics as the backend: exact match on `<entity>.<action>`, and `all.read` grants any `<entity>.read`, `all.write` grants any `<entity>.create|update|delete`. The frontend does not implement different or broader rules than the backend.
- **Implication:** The existing `hasPermission(permissions, required)` (or equivalent) is updated to work with entity-style keys and the same wildcard rules. Route and component visibility (e.g. sidebar items, permission guards) use this so that the UI never shows actions the user is not allowed to perform.

---

## 8. Migration from current dashboard.* keys

- **Choice:** The **canonical** format is `<entity>.<action>`. Existing data and code that use `dashboard.*` and `dashboard` are migrated or mapped in a separate migration step (data migration and/or backward-compatibility layer). This document does not prescribe the exact migration strategy (e.g. big-bang vs phased, or mapping table); it only fixes that the target model is entity-based and that the system must not rely on dashboard-style keys once migration is complete.
- **Implication:** Epic 2 implementation can introduce the new resolution and check logic and the new key derivation from entities; the migration of existing `role_permission` rows and of UI/API that still reference old keys is a follow-up. During a transition, both key styles may be supported for resolution if needed; the technical choice is that the **target** is entity-only (plus all.read / all.write).

---

## 9. Error handling for permissions

- **Missing permission (403):** When an operation requires a permission the user does not have (after resolution, including wildcards), the server responds with 403 Forbidden and a clear error (e.g. “Forbidden” or “Insufficient permissions”). The client does not clear scope or token; it shows an error and allows the user to switch scope (if they have another org/role) or to stop the action.
- **Invalid permission key (e.g. on role-permission add):** If a client or admin tries to assign a permission key that is not in the derived set (see §4), the server responds with 4xx (e.g. 422 Unprocessable Entity) and an error indicating the key is invalid. The list of valid keys is derived from the entity set and returned by the “list permissions” (or equivalent) API so that UIs only offer valid keys.
- **No scope when scope is required:** If an endpoint requires scope (e.g. org-scoped entity operations) and the request has no valid scope (e.g. missing or invalid headers), the server responds with 400 or 403 as appropriate; the client should follow Epic 1 error handling (redirect to scope selection or login). Permission checks are only run after authentication and (where required) scope are established.

---

## 10. Transport consistency

- **Choice:** The same permission model (entity + action, resolution with global/org and wildcards, single authorization rule) is used for REST, and later for RPC and subscriptions. There is no separate “REST permission” vs “subscription permission”; if a user has `user.read` in scope, they can list/get users via REST and (later) subscribe to user queries via WebSocket.
- **Implication:** Resolution and authorization logic are shared and not transport-specific. Epic 2 does not implement RPC or subscriptions; it only fixes the model so that when they are added, they reuse the same permissions.

---

## Summary table

| Topic | Choice |
|-------|--------|
| Key format | `<entity>.<action>` with action in {create, read, update, delete}. |
| Wildcards | Keep `all.read` and `all.write`; they grant every *.read and *.write respectively. |
| Storage scope | Keep global vs org; only permission_key values move to entity.action. |
| Allowed key set | Derived from entity set (four keys per entity + all.read, all.write). |
| Dashboard access | No separate key; “has any permission in scope” grants dashboard access. |
| Authz rule | One rule: operation requires the corresponding entity.action (and resolution). |
| Frontend | Same semantics as backend (exact + wildcards). |
| Migration | Target is entity-based; migration of dashboard.* is a separate step. |
| Errors | 403 when permission missing; 4xx for invalid key on assign; scope errors per Epic 1. |
| Transports | Same model for REST, RPC, and subscriptions. |
