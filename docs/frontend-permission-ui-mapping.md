# Frontend: model-driven permission → UI mapping

This document defines how the UI derives **show**, **hide**, and **disable** from backend model permission keys. Permissions are **model-driven** (e.g. `organization.read`, `user.create`), not screen-driven, so we need a clear convention and a single place to implement it.

**Related:** [Frontend implementation and choices](frontend-implementation-and-choices.md), Epic 2 (entity-based permissions), `frontend/src/lib/permissions.ts`.

---

## 1. Source of permissions

- The frontend gets effective permissions from **`GET /api/auth/me`** (with `X-Organization-Id` and `X-Role-Id`). The response includes a **`permissions`** array of strings.
- These strings are **model permission keys** in the form **`<entity_id>.<action>`**, where:
  - `entity_id` is the backend model id (e.g. `organization`, `user`, `role`, `permission`, `audit`).
  - `action` is one of: **`create`**, **`read`**, **`update`**, **`delete`**.
- In React, permissions are available from **`useAuthentication().permissions`** (and from auth state/store elsewhere).

---

## 2. Convention: action → UI meaning

| Backend action | UI meaning | Typical use |
|----------------|------------|-------------|
| **read**       | User may **see** the resource (list, detail). | Show nav item, list, detail view; enable “View”. |
| **create**     | User may **create** new resources. | Show/enable “Add”, “Create”, “New” actions. |
| **update**     | User may **edit** existing resources. | Enable “Edit”, inline edit, save. |
| **delete**     | User may **delete** resources. | Enable “Delete”, remove. |

- **Show vs hide:** Use **read** to decide whether to show a section, tab, list, or detail view for that entity. If the user has no `.read` for an entity, hide or redirect (or show “no permission”).
- **Disable:** Use **create** / **update** / **delete** to disable buttons or actions. Prefer **disable** over hide for actions the user might expect (e.g. grey out “Edit” if no `organization.update`), unless the design prefers hiding.

---

## 3. Mapping rule (one sentence)

- **Show** a UI part that lists or displays entity `X` if the user has **`X.read`** (or an equivalent, see below).
- **Enable** create for entity `X` if the user has **`X.create`**; enable edit if **`X.update`**; enable delete if **`X.delete`**.

---

## 4. Equivalents and wildcards (backend / existing frontend)

The backend (and existing `lib/permissions.ts`) support:

- **Exact key:** e.g. `organization.read`.
- **Legacy/screen keys:** e.g. `dashboard.organizations.read` is treated as equivalent to `organization.read` (see `equivalents()` in `permissions.ts`).
- **Wildcards:**  
  - **`all.read`** → grants any **`<entity>.read`**.  
  - **`all.write`** → grants any **`<entity>.create`**, **`<entity>.update`**, **`<entity>.delete`**.

Any helper or hook that “checks permission for UI” should use the **same** logic as `hasPermission(permissions, required)` so that equivalents and wildcards are respected.

---

## 5. Required key(s) per UI intent

| UI intent | Required key(s) | Notes |
|-----------|------------------|--------|
| Show list/detail for entity `X` | `X.read` | Or `all.read` for any entity. |
| Show “Create” / “Add” for entity `X` | `X.create` | |
| Enable “Edit” / “Save” for entity `X` | `X.update` | |
| Enable “Delete” for entity `X` | `X.delete` | |
| “Dashboard” access (at least one permission in scope) | `dashboard` or any one permission | Already defined in `hasPermission`: `dashboard` means `permissions.length > 0`. |

So:

- **Show** = `hasPermission(permissions, 'X.read')` (or multiple entities for a screen that shows several).
- **Enable create** = `hasPermission(permissions, 'X.create')`.
- **Enable update** = `hasPermission(permissions, 'X.update')`.
- **Enable delete** = `hasPermission(permissions, 'X.delete')`.

---

## 6. Implementation: single helper or hook

- **One place** should implement this mapping: a **helper** and/or a **hook** that takes:
  - **Input:** `permissions: string[]`, **required key(s)** (e.g. `'organization.read'` or `['organization.read', 'organization.update']`), and optionally a **mode**: `'show'` (default) vs `'enable'`.
  - **Output:** `boolean` (show this / enable this).
- The implementation must call the existing **`hasPermission(permissions, required)`** so that equivalents and wildcards are reused.
- **Recommended:** A hook **`usePermission(required: string | string[], mode?: 'show' | 'enable')`** that reads `permissions` from `useAuthentication()` and returns `hasPermission(permissions, required)`. For “enable”, the caller passes the action key (e.g. `organization.update`); for “show” (list/detail), the caller passes `organization.read`. No need for a separate “mode” if the caller always passes the correct key; the hook can just return `hasPermission(permissions, required)`.

---

## 7. Summary

- **Source:** `useAuthentication().permissions` (from `/api/auth/me`).
- **Convention:** `read` → show list/detail; `create` → enable create; `update` → enable edit; `delete` → enable delete.
- **Implementation:** One helper or hook that uses `hasPermission(permissions, required)`; UI uses it consistently for show/disable decisions.
- **Equivalents/wildcards:** Handled inside `hasPermission` (and thus by the hook); no extra logic in screens.
