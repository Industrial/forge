# Plan: Bearer-Only Auth + Scope from Headers (X-Organization-Id, X-Role-Id)

**Goal:** Remove session-based auth. Use Bearer tokens for identity. Use **only** request headers for scope: `X-Organization-Id` and `X-Role-Id` (organization id never from URL).

---

## 1. Design summary

- **Identity:** `Authorization: Bearer <token>` only. No session cookie.
- **Scope:** Every scoped request must send:
  - `X-Organization-Id`: UUID of the organization context.
  - `X-Role-Id`: UUID of the role in that org (user must have this role in this org).
- **Token issuance:** Login and register return a token in the response body (e.g. `{ "token": "...", "expires_at": "..." }`). Optionally store in `api_tokens` with TTL for revocation on logout.
- **REST routes:** Org-scoped operations get org/role from headers only. Paths that today include org id (e.g. `/api/organizations/:id/users`) can be refactored to e.g. `/api/organization/users` with required `X-Organization-Id`, or keep path for resource identity and **require** headers for scope and validate that header org matches path org and user has membership. This plan assumes **scope always from headers**; path can still contain resource ids (e.g. `:id` for “which org resource”) but **authorization context** (which org/role the user is acting in) comes only from `X-Organization-Id` and `X-Role-Id`.

---

## 2. Crates and files to change

### 2.1 `crates/forge`

| File | Changes |
|------|--------|
| **`src/app.rs`** | **with_auth:** Add a token-only mode (or new method `with_token_auth_only`). When token-only: do not create `SessionManagerLayer` or `SqliteStore`; do not run session migrations; do not use `AuthManagerLayerBuilder::new(backend, session_layer)`. Only add `TokenAuthLayer::new(db, backend, lookup)` so identity comes from Bearer. Require `token_lookup` in token-only mode. Keep `GovernorLayer` for per-user rate limit but key extractor must get user + org from token user and a new “scope” extension (see forge-rate-limit). |
| **`src/app.rs`** | **with_token_auth:** Document that it is required when using token-only auth. Behavior unchanged. |
| **`src/lib.rs`** | If session is fully removed from default stack: stop re-exporting `tower_sessions` and `tower_sessions_sqlx_store` (or keep for backward compat and mark deprecated). Optional. |
| **`src/authz.rs`** | **AuthSessionGuardExt** and **guard_and_audit_user:** Today they use `AuthSession` and `AuthzContext` (org/role from session or user). When moving to token + headers, authz may need a new context type that carries (user, org_id, role_id/role_name) from request (e.g. from an extractor that reads headers and validates membership). Either: (1) add a new guard that takes an explicit scope context (from headers), or (2) ensure `TokenUser` (or a wrapper) can implement `AuthzContext` with org/role coming from a request extension set by middleware. Plan: add/extend so that a “scope context” (org_id, role from headers) is available and used by guards. |
| **`Cargo.toml`** | When token-only is the only path: make `tower-sessions` and `tower-sessions-sqlx-store` optional (feature or conditional). So existing apps can still opt into session if needed. |

### 2.2 `crates/forge-auth`

| File | Changes |
|------|--------|
| **`src/token_auth.rs`** | **OptionalRequireAuth:** Today it checks (1) `TokenUser<B::User>` and (2) `AuthSession<B>`. In token-only mode, remove the `AuthSession<B>` branch so only `TokenUser` is used. Alternatively, keep both but Forge will not install session layer, so only TokenUser will be present. No code change if session is simply not installed. |
| **`src/token_auth.rs`** | **RequireAuth:** Unchanged; it uses OptionalRequireAuth. |
| **`src/token_auth.rs`** | Consider adding an extractor **`RequireScope`** that reads `X-Organization-Id` and `X-Role-Id` from headers, validates them (org exists, role belongs to org, user has that role in that org via user_org_role), and returns `(org_id, role_id)` or a `Scope` struct. This can live in the template app instead if forge-auth should stay session-agnostic. |

### 2.3 `crates/forge-authz`

| File | Changes |
|------|--------|
| **`src/lib.rs`** | **AuthzContext:** Implemented for `AuthSession<B>` and for `B::User`. When using token-only, the “user” in request extensions is `TokenUser<B::User>`. So either: (1) implement `AuthzContext` for `TokenUser<U>` where `U: AuthzContext` (delegate to inner user) and ensure `B::User` provides `organization_id()` and `role()` from somewhere (e.g. from request extension set by a scope middleware), or (2) introduce a separate “request scope” type (e.g. `OrgScope { org_id, role_id }`) that is set by middleware from headers and used by guards. Recommendation: add a type e.g. `RequestScope { org_id, role_id }` that can be used as part of authz context; guards then take (user, request_scope). Template app can set `RequestScope` from headers in a layer or extractor. |

### 2.4 `crates/forge-rate-limit`

| File | Changes |
|------|--------|
| **`src/lib.rs`** | **RequesterOrgKeyExtractor:** Today it gets user from `AuthSession<B>`. With token-only, user is in `TokenUser<B::User>`. Change extractor to: first try `TokenUser<B::User>`, then fall back to `AuthSession<B>`. And organization_id must come from scope (headers): either from a new request extension (e.g. `OrgScope` or `X-Organization-Id` parsed and validated) or from `AuthzContext::organization_id()` if the user type is extended to carry current scope (set by middleware from headers). So: (1) read user from `TokenUser` or `AuthSession`; (2) read org from a new extension (e.g. `ScopeFromHeaders { org_id }`) set by app middleware. Add that extension in template app and have rate limiter use it. |

### 2.5 `crates/forge-cli/templates/default/crates/db`

| File | Changes |
|------|--------|
| **`src/auth.rs`** | **Backend:** No change. Still used to load user by id for TokenAuthLayer. |
| **`src/lib.rs`** | **token_lookup:** No change. Keep as is. Optionally add a second token type for “login tokens” (short-lived) if we want logout to revoke; then token_lookup could check both api_tokens and login_tokens tables, or a single table with a `kind` column. |

### 2.6 `crates/forge-cli/templates/default/crates/app`

| File | Changes |
|------|--------|
| **`src/lib.rs`** | **make_app:** Stop calling `.with_auth(...)` in session-based form. Call new Forge API for token-only auth (e.g. `.with_token_only_auth(|db| Backend::new(db), db::token_lookup)` or `.with_auth(...).with_token_auth(...)` and a flag to skip session). Remove routes that are session-only: **POST /api/auth/set-profile**, **POST /api/auth/switch-profile**, **GET /api/auth/session**. Add or keep: **POST /api/auth/login** (return token in body), **POST /api/auth/register** (return token), **GET /api/auth/me** (Bearer + optional X-Organization-Id, return user and optionally permissions for that org). Protect all REST routes that mutate or list scoped data: require Bearer and scope headers; use new extractor for scope. Optionally refactor route paths so org is not in path (e.g. `/api/organization/users` with header) — see below. |
| **`src/lib.rs`** | **Route registration:** For org-scoped REST endpoints, either: (A) Keep paths like `/api/organizations/:id/users` but add middleware/extractor that requires `X-Organization-Id` and `X-Role-Id` and validates `:id == X-Organization-Id` and user has that role; or (B) Change to `/api/organization/users` (singular) and require headers only. Plan recommends (B) so “organization id never from URL”: e.g. `GET/POST /api/organization/users`, `GET/POST /api/organization/roles`, `GET/POST/DELETE /api/organization/roles/:role_id/permissions`, etc. Keep `GET /api/organizations` (list) and `GET /api/organizations/:id` (get one by id for navigation/details) but require auth; scoped actions use `/api/organization/...` + headers. |
| **`src/handlers/auth.rs`** | **Remove:** All session and profile-from-session logic. Remove `tower_sessions::Session` import and extractor. Remove `SESSION_CURRENT_ORG_ID`, `SESSION_CURRENT_ROLE_NAME`, `get_profile_from_session`, `require_profile`, `set_profile`, and any handler that writes/reads these. Remove **session_json** handler. Remove **set_profile** / **switch_profile** handlers. |
| **`src/handlers/auth.rs`** | **Login:** Change to: authenticate with Backend; if success, create a token (e.g. insert into api_tokens with optional short TTL), return `{ "token": "...", "expires_at": "..." }` in JSON. Do not set any session cookie. |
| **`src/handlers/auth.rs`** | **Register:** Same: after creating user and org/roles, create a token and return it in JSON so client is “logged in”. |
| **`src/handlers/auth.rs`** | **Logout:** Either remove (client discards token) or add **POST /api/auth/logout** that invalidates the current token (e.g. delete from api_tokens by token hash). If logout is implemented, client must send Bearer token in the request. |
| **`src/handlers/auth.rs`** | **profile / profiles_list:** Replace with **GET /api/auth/me**. Require Bearer. Return user id, email. If `X-Organization-Id` (and optionally `X-Role-Id`) are sent, validate membership and return permissions for that org (and list of roles in that org). Optionally return list of (org_id, org_name, role_id, role_name) for “profiles” so client can choose which to send in headers. |
| **`src/handlers/auth.rs`** | **resolve_permissions:** Keep; change call sites so that `profile: Option<&CurrentProfile>` is built from request scope (headers) instead of session. So `CurrentProfile` becomes (org_id, role_name) from headers + DB validation. |
| **`src/handlers/auth.rs`** | **channels_from_permissions:** Keep; used by WebSocket. WebSocket must get scope from somewhere: either (1) WebSocket upgrade request carries query params or first message with org_id/role_id, or (2) connection is tied to a single “default” org from token metadata. Plan: WebSocket upgrade reads `X-Organization-Id` and `X-Role-Id` from request headers (browsers send them on WS upgrade). So WS handler will use headers for scope. |
| **`src/handlers/auth.rs`** | **create_token:** Keep; still used for long-lived API tokens. Require Bearer (or a separate admin auth) for creating tokens. |
| **`src/handlers/auth.rs`** | **admin_only:** Keep; require Bearer and check user is admin (or has global role). No session. |
| **`src/handlers/rest.rs`** | **New extractor or middleware:** Add **ScopeFromHeaders** (or **RequireScope**): read `X-Organization-Id` and `X-Role-Id` from headers; validate org exists, role belongs to org, and authenticated user has that role in that org (user_org_role); return 401/403 if missing or invalid. Attach (org_id, role_id, role_name) to request extensions so handlers can use it. |
| **`src/handlers/rest.rs`** | **All org-scoped handlers:** Refactor to use scope from extractor (headers) instead of path. So: replace `Path(org_id)` (and path `:id` for org) with the new scope extractor. For routes that currently are `/api/organizations/:id/...`, change to `/api/organization/...` and get org_id from scope extractor. Handlers: list_org_users, add_org_user, add_org_user_roles, get_org_user, update_org_user, delete_org_user, list_org_roles, create_org_role, get_org_role, update_org_role, delete_org_role, list_org_role_permissions, add_org_role_permission, delete_org_role_permission. |
| **`src/handlers/rest.rs`** | **list_organizations, create_organization, get_organization, update_organization, delete_organization:** Keep paths. Require Bearer. For get/update/delete by id, optionally require `X-Organization-Id` to match when the operation is org-scoped (or leave as resource-by-id with Bearer only). |
| **`src/handlers/rest.rs`** | **users_me:** Implement properly: require Bearer, return current user and optionally memberships/profiles (orgs + roles). |
| **`src/handlers/rest.rs`** | **list_permissions:** Can stay public or require Bearer; no scope needed. |
| **`src/handlers/rest.rs`** | **list_audit_log, get_audit_log:** Require Bearer; optionally require `X-Organization-Id` to filter by org. |
| **`src/handlers/rest.rs`** | **list_users, create_user, get_user, update_user, delete_user, get_user_organizations:** Require Bearer. Scope (which org) may come from headers when needed for create_user (which org to add user to). |
| **`src/handlers/ws.rs`** | **handler:** Remove `Session` and `AuthSession`. Use token for auth: either (1) accept Bearer in query string or first WebSocket message, or (2) require cookie or header on the HTTP upgrade request. Standard approach: send Bearer in `Authorization` header on the WebSocket upgrade request (browsers support that). So: extract user from `TokenUser` (or from Authorization header in upgrade request if Forge injects it). For scope: read `X-Organization-Id` and `X-Role-Id` from upgrade request headers; validate and compute permissions; then call `channels_from_permissions` and `resolve_permissions` with that profile. Remove `get_profile_from_session(&session)`. |
| **`src/handlers/dashboard.rs`** | Already unused (dashboard routes removed). Leave as-is or delete the file and remove from `handlers/mod.rs` if desired. |
| **`src/handlers/mod.rs`** | Remove or keep `pub mod dashboard` depending on whether dashboard.rs is deleted. |

### 2.7 Frontend (`crates/forge-cli/templates/default/frontend`)

| File | Changes |
|------|--------|
| **`src/context/Session.tsx`** | **Rename or repurpose to AuthContext / TokenContext.** Remove `fetchSession()` that calls `GET /api/auth/session` with credentials. Instead: (1) Store token in state (and optionally in localStorage or sessionStorage). (2) Expose a way to set token after login/register (from response body). (3) Provide “me” or “session” state by calling **GET /api/auth/me** with `Authorization: Bearer <token>` and optionally `X-Organization-Id` / `X-Role-Id`; store user, permissions, and list of (org, role) “profiles”. (4) Remove `needs_profile_select` and “profile” as something set on the server; instead, client picks “current” org/role and sends them as headers on every request. (5) Add a function to set “current scope” (org_id, role_id) in context and include them in a shared fetch/API helper as headers. |
| **`src/context/Session.tsx`** | **refresh:** Call GET /api/auth/me with Bearer and current scope headers; update user, permissions, profiles. |
| **`src/features/authentication/pages/LoginPage/LoginPage.tsx`** | On successful login, read `token` from response body; store in auth context (and optionally storage). Do not rely on cookie. Do not call `refresh()` that hits session; call new “setToken” and then fetch /api/auth/me. Redirect to dashboard or to a “select scope” page if you want user to pick org/role before first API call. Remove `needs_profile_select` redirect to /select-profile unless you keep a client-side “pick org/role” step. |
| **`src/features/authentication/pages/RegisterPage/RegisterPage.tsx`** | Same as login: read token from response, store it, fetch me, redirect. |
| **`src/features/authentication/pages/SelectProfilePage/SelectProfilePage.tsx`** | Repurpose to “select current scope” (org + role): when user has no scope selected, show list of (org, role) from me/profiles; on select, set scope in context (org_id, role_id) and redirect to dashboard. No server set-profile call. |
| **`src/features/authentication/layouts/AuthenticationLayout/AuthenticationLayout.tsx`** | No session dependency; minimal or no change. |
| **`src/features/dashboard/*`** | Every API call that is org-scoped must send `X-Organization-Id` and `X-Role-Id` (from context). Add a shared API client or fetch wrapper that: (1) Adds `Authorization: Bearer <token>`, (2) Adds `X-Organization-Id` and `X-Role-Id` from context when available. Use it in all dashboard pages (OrganizationsPage, UsersPage, RolesPage, PermissionsPage, AuditLogPage). |
| **`src/features/dashboard/layouts/DashboardLayout/DashboardLayout.tsx`** | Ensure layout has access to current scope and can pass it (or the API helper) to children. |
| **`src/features/dashboard/components/Sidebar/Sidebar.tsx`** | Permissions and nav items may come from /api/auth/me (with scope headers). No session. |
| **`src/features/dashboard/pages/OrganizationsPage/OrganizationsPage.tsx`** | Use API client with Bearer + scope headers. Change endpoints if backend paths change (e.g. list orgs: GET /api/organizations with Bearer). |
| **`src/features/dashboard/pages/UsersPage/UsersPage.tsx`** | Use API client with Bearer + scope headers. If backend moves to GET /api/organization/users with headers, call that. |
| **`src/features/dashboard/pages/RolesPage/RolesPage.tsx`** | Same: Bearer + scope headers; use new paths if any (e.g. GET /api/organization/roles). |
| **`src/features/dashboard/pages/PermissionsPage/PermissionsPage.tsx`** | Same. |
| **`src/features/dashboard/pages/AuditLogPage/AuditLogPage.tsx`** | Same; send scope headers for filtered audit log if backend requires them. |
| **`src/components/ProtectedRoute.tsx`** | Base on token presence (and optionally valid me) instead of session. Redirect to /login if no token. |
| **`src/components/DashboardProfileGuard.tsx`** | Replace with “scope required” guard: redirect to select-profile (or scope picker) if no org/role selected in context. No server call. |
| **`src/components/Navbar.tsx`** | If it reads session or user from context, use new auth context (token + me). Logout: clear token and optionally call POST /api/auth/logout with Bearer before clearing. |
| **`src/features/profile/pages/ProfilePage/ProfilePage.tsx`** | Load user via GET /api/auth/me or GET /api/users/me with Bearer. |
| **`src/effects/users.ts`** | If this contains API calls, add Bearer and scope headers via shared client. |
| **`src/schemas/*`** | No change unless form schemas reference session shape. |
| **`src/App.tsx`** | Wrap app with the updated Auth/Token provider. Ensure ProtectedRoute and DashboardProfileGuard (or scope guard) are used correctly. Remove or update any route that assumed session (e.g. select-profile). |
| **`src/utils/ws.ts`** | If WebSocket URL or options are built here, ensure the WS upgrade request can include Authorization and X-Organization-Id, X-Role-Id (e.g. via query params or ensure fetch/WS supports custom headers depending on browser API). Browsers allow headers on WebSocket constructor in some environments; otherwise pass token in query and read scope from context. |

---

## 3. New constants / types

- **Header names:** `X-Organization-Id`, `X-Role-Id` (document in API docs and in code).
- **Scope type (template app):** e.g. `struct OrgScope { org_id: Uuid, role_id: Uuid, role_name: String }` populated from headers and validated against user_org_role.

---

## 4. Route path changes (template app, optional but aligned with “org id never from URL”)

- Current: `GET/POST /api/organizations/:id/users`, `.../roles`, `.../roles/:role_id/permissions`, etc.
- New: `GET/POST /api/organization/users`, `GET/POST /api/organization/roles`, `GET/POST/DELETE /api/organization/roles/:role_id/permissions`, etc., all with required `X-Organization-Id` and `X-Role-Id`.
- Keep: `GET /api/organizations`, `POST /api/organizations`, `GET /api/organizations/:id`, `PATCH /api/organizations/:id`, `DELETE /api/organizations/:id` for listing and CRUD by resource id; require Bearer. Scope headers required only where the operation is “in context of an org” (e.g. create_organization might not need scope; list_organizations might require Bearer and optionally filter by scope).

---

## 5. Summary table

| Area | Remove | Add / Change |
|------|--------|---------------|
| **Forge** | Session layer when token-only | Token-only auth mode; optional session deps |
| **forge-auth** | Session fallback in OptionalRequireAuth (optional) | Optional: RequireScope extractor |
| **forge-authz** | — | RequestScope / TokenUser + scope for AuthzContext |
| **forge-rate-limit** | AuthSession-only key extraction | Also TokenUser + scope extension |
| **Template app** | Session, set_profile, session_json, profile from session | Login/register return token; GET /api/auth/me; scope extractor; Bearer + headers on all protected routes; refactor org-scoped paths to use headers only |
| **Template db** | — | No structural change |
| **Frontend** | Session fetch, set-profile, cookie auth | Token storage; API client with Bearer + X-Organization-Id, X-Role-Id; scope picker (client-side); WebSocket with token + headers |

This plan is a complete file-level checklist for moving to Bearer-only auth with scope always from `X-Organization-Id` and `X-Role-Id`.
