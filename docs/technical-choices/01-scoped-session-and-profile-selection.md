# Technical choices: Epic 1 — Scoped session and profile selection

This document records technical decisions for the first deliverable (scoped session and profile selection). It does not prescribe implementation details that are left to the codebase; it fixes the choices that affect architecture and contracts.

**Epic reference:** [01_scoped-session-and-profile-selection](../deliverables/01_scoped-session-and-profile-selection.md)

---

## 1. Scope persistence and storage

- **Choice:** Client-only. Scope is stored only on the client (no server-side session storing scope). Aligns with a Zero Trust future: the server does not trust the client’s identity beyond the token and does not maintain server-side scope state.
- **Implication:** Scope (and token) must be sent on every request (e.g. headers). The backend continues to derive scope from request headers only; no new backend endpoints or session store for scope.

---

## 2. Single runtime and request-time scope

- **Choice:** One runtime. The Effect runtime (and thus the HTTP client layer) is built once. Scope is not baked into the runtime at build time; it is read at **request time** when each request is made.
- **Implication:** Every outbound request must obtain the current scope (and token) from a single source of truth (e.g. a ref or synchronous store) at the moment the request is sent. No rebuild of the runtime when the user selects or switches scope. The HTTP client layer (or a request interceptor / request mapper) must read current scope and attach the appropriate headers (e.g. `X-Organization-Id`, `X-Role-Id`) to each request.

---

## 3. Post-login navigation for multi-org users

- **Choice:** Multi-org users are always sent to the scope-selection screen before any other app content. After login, if the backend indicates that scope selection is required (e.g. `needs_profile_select` or equivalent), the client navigates to the scope-selection route first; only after a scope is chosen does the client navigate to the intended destination (e.g. `from`).
- **Implication:** The login flow must not navigate to `from` (or any other route) until it has checked whether scope selection is required; if required, navigate to scope selection and only then (after scope is set) to the final destination. Single-org users can continue to go straight to the app without seeing the scope-selection screen.

---

## 4. Terminology: scope, not profile

- **Choice:** Use “scope” as the canonical term. “Profile” is legacy; naming (APIs, UI, docs, storage keys) moves to “scope” (e.g. current scope, scope selection, set scope). The concept is “session scope” (org + role, and optionally a display label for the role).
- **Implication:** No `set-profile` or “profile” endpoint or concept in the new design. Scope is set and persisted on the client only; it is sent per request via headers. After the user selects or switches scope, the client: (1) updates scope in its store and persisted storage, (2) refetches `/api/auth/me` with the new scope (so permissions and UI state are correct). No backend endpoint for “setting” scope.

---

## 5. Backend scope contract (no changes)

- **Choice:** No backend changes for this epic. The backend continues to derive scope from request headers (e.g. `X-Organization-Id`, `X-Role-Id`), validate them against the authenticated user’s memberships, and return permissions and `needs_profile_select` accordingly.
- **Implication:** All changes for Epic 1 are on the client: persistence, request-time scope injection, navigation, and terminology. Backend contract (headers, `/api/auth/me` response shape) is unchanged.

---

## 6. Role name (and other scope display fields)

- **Choice:** Role name (and any similar display-only fields in scope) are purely presentational. They are used only for UI (e.g. “Default · Editor”). They do not drive backend logic or permission resolution; the backend uses org id and role id only.
- **Implication:** The client may store and display role name (or org name) for the current scope; it is not part of the authorization or API contract beyond what the backend already returns (e.g. in `/api/auth/me` profiles).

---

## 7. Error handling (complete)

Error handling for scope and auth in this epic is defined as follows.

### 7.1 Invalid or missing scope on request

- **Scenario:** The user has a scope set (e.g. org id + role id in storage), but the backend rejects it (e.g. 400 Bad Request, 403 Forbidden, or 404 Not Found) because the scope is invalid, expired, or the user no longer has that role in that org.
- **Behaviour:** The client treats the response as a scope error. The client must: clear or invalidate the current scope in store and storage, and either (a) redirect the user to the scope-selection screen (if the user has multiple scopes to choose from), or (b) clear session and redirect to login if no valid scope remains. The user must not remain in a state where every request fails with a scope error; the UI must reflect “no valid scope” and guide the user to re-select scope or re-authenticate.

### 7.2 Unauthenticated (401) or token expired

- **Scenario:** A request returns 401 Unauthorized (e.g. token missing, invalid, or expired).
- **Behaviour:** The client treats the user as logged out: clear token and scope from store and storage, and redirect to the login screen. Optional: preserve the intended destination (e.g. `from`) so that after re-login the user can be sent to scope selection (if needed) and then to that destination.

### 7.3 Forbidden (403) on a specific operation

- **Scenario:** The request is authenticated and scope is valid, but the server returns 403 Forbidden for the requested operation (e.g. insufficient permissions for that entity or action).
- **Behaviour:** Do not clear scope or token. Show an appropriate error to the user (e.g. “You don’t have permission to do this”) and do not redirect to scope selection or login. The user may switch scope (if they have another org/role) to get different permissions, or stay and see the error.

### 7.4 Scope selection or switch fails (e.g. network or server error)

- **Scenario:** After the user selects a scope, the client fails to refetch `/api/auth/me` (e.g. network error, 5xx, or non-2xx).
- **Behaviour:** Do not persist the new scope as “current” until the refetch succeeds. Optionally: show a retriable error (e.g. “Could not load session; try again”). If the user had a previous valid scope, the client may keep showing it until the new scope is confirmed; otherwise, leave the user on the scope-selection screen with the ability to retry or choose another scope.

### 7.5 No scope set and user tries to access protected content

- **Scenario:** The user is authenticated but has not yet selected a scope (e.g. they navigated directly to a deep link or skipped scope selection due to a bug), and the backend requires scope headers and returns an error or empty permissions.
- **Behaviour:** If the user has multiple possible scopes (e.g. `needs_profile_select` was true or the list of scopes is non-empty), redirect to the scope-selection screen and do not allow access to protected content until a scope is set. If the user has only one scope, the client may auto-select it and then proceed (so single-org users do not see the scope-selection screen).

### 7.6 Stale or inconsistent scope (e.g. after tab or device change)

- **Scenario:** Scope is stored client-side only (e.g. localStorage). Another tab or device might have a different scope; or the user’s membership might have been revoked on the server.
- **Behaviour:** No cross-tab or cross-device sync is required for this epic. If a request fails with 403/404 due to invalid or revoked scope, apply 7.1 (treat as scope error and redirect to scope selection or login). Optional later improvement: listen for storage events to hint other tabs to refresh scope or re-fetch session; out of scope for Epic 1.

---

## Summary table

| Topic              | Choice                                                                 |
|--------------------|-----------------------------------------------------------------------|
| Scope persistence  | Client-only (Zero Trust); no server-side scope store.                 |
| Runtime            | Single runtime; scope read at request time and sent per request.     |
| Post-login         | Multi-org users always go to scope-selection first, then destination. |
| Terminology        | “Scope” everywhere; retire “profile” in this context.                  |
| Backend            | No changes; scope via existing headers and `/api/auth/me`.            |
| Role name          | Presentational only.                                                  |
| Error handling     | As in §7: scope errors → re-select or login; 401 → logout; 403 → show error; refetch failure → do not commit new scope. |
