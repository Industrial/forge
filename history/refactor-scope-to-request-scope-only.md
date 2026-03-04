# Refactor: RequestScope Only (Remove CurrentProfile / RequireScope)

**Goal:** Remove `CurrentProfile` and `RequireScope`; use `RequestScope` and a single scope extractor (X-Organization-Id + X-Role-Id) everywhere. No legacy comments — old code is replaced.

**Checks/builds/tests:** To be run after refactor.

---

## 0. CRUD-able roles: RequestScope uses role_id + role_name from DB; Role type removed

**Design:** Roles are CRUD-able; the set of roles is defined in the DB (`org_role`), not by a Rust type. Scope and permission resolution are fully DB-driven.

- **forge-auth** `RequestScope`: `organization_id`, `role_id`, `role_name` (from `org_role`). No `Role` enum; **Role type has been removed.** `AuthzContext` no longer has `role()`; permission checks use permission keys (e.g. `require_permission` + `role_permission` table keyed by `role_name`).
- **forge-auth**: Removed `Role`, `TokenUserGuardExt`, `guard_user`, `role_name_to_role`. Removed `role()` from `AuthzContext`.
- **forge-audit**: Removed role-based `guard_and_audit_user` and `TokenUserGuardAuditExt`. Added `record_authz_allowed`; apps use `require_permission` then record allowed/denied.
- **Template** scope extraction: Build `RequestScope { organization_id, role_id, role_name }` from headers + DB only.

---

## 1. ~~forge-auth: Role → role name~~ (obsolete)

**Done:** Role type removed. Permission resolution uses `scope.role_name` (string from DB) only; no enum mapping.

---

## 2. Template app: Single scope extractor in one place

**Current state:** `ScopeFromHeaders` (X-Organization-Id, X-Role-Id) lives in `rest.rs` and produces `RequestScope`. `RequireScope` (X-Organization-Id, X-Role-Name) lives in `auth.rs` and produces `CurrentProfile`.

**Target state:** One extractor only: `ScopeFromHeaders` producing `RequestScope`, using X-Organization-Id and X-Role-Id. All routes (REST and dashboard) use it.

**2a. Move ScopeFromHeaders from rest.rs to auth.rs**

- **File:** `crates/forge-cli/templates/default/crates/app/src/handlers/rest.rs`
  - Remove the `ScopeFromHeaders` struct and its `FromRequestParts` impl (lines ~36–120).
  - Remove `role_name_to_authz` (or keep only in auth.rs).
  - Remove `HEADER_ORGANIZATION_ID` and `HEADER_ROLE_ID` from rest.rs (they move to auth.rs).
  - Add at top: `use crate::handlers::auth::ScopeFromHeaders;` (and use it in all REST handlers that currently use it).

- **File:** `crates/forge-cli/templates/default/crates/app/src/handlers/auth.rs`
  - Add `ScopeFromHeaders` struct and its `FromRequestParts` impl. Logic: same as current rest.rs (require Bearer, read X-Organization-Id and X-Role-Id, validate org + role + user_org_role, build `RequestScope`, insert into extensions).
  - Add `HEADER_ORGANIZATION_ID` and `HEADER_ROLE_ID` constants.
  - Add `role_name_to_authz(name: &str) -> Role` (same mapping as in rest today).
  - Ensure auth.rs has the necessary imports (RequestScope, Role, TokenUser, db models: organization, org_role, user_org_role, user, etc.).

**2b. Remove CurrentProfile and RequireScope from auth.rs**

- **File:** `crates/forge-cli/templates/default/crates/app/src/handlers/auth.rs`
  - Delete the `CurrentProfile` struct.
  - Delete the `RequireScope` struct and its `FromRequestParts` impl (the one that uses X-Role-Name).
  - Replace `get_scope_from_headers_map` and `get_scope_from_headers`: they currently return `Option<CurrentProfile>`. Change to return `Option<RequestScope>`. Implementation: reuse the same validation logic as ScopeFromHeaders (parse headers, validate org/role/membership), but without the extractor; call a shared helper that returns `Result<RequestScope, _>` and have both ScopeFromHeaders and get_scope_from_headers_map use it. So:
    - Add a helper e.g. `pub async fn try_scope_from_headers(headers: &HeaderMap, db: &DbConnection, user_id: Uuid) -> Result<RequestScope, ...>` that parses X-Organization-Id and X-Role-Id, loads org and role, checks user_org_role, builds RequestScope.
    - `ScopeFromHeaders::from_request_parts`: get user_id from TokenUser, call try_scope_from_headers(parts.headers, db, user_id), map Err to rejection, insert scope into extensions, return Ok(ScopeFromHeaders(scope)).
    - `get_scope_from_headers_map(headers, user, db)`: call try_scope_from_headers(headers, db, user.id).ok().

**2c. resolve_permissions and require_permission**

- **File:** `crates/forge-cli/templates/default/crates/app/src/handlers/auth.rs`
  - `resolve_permissions(db, user, profile: Option<&CurrentProfile>)` → `resolve_permissions(db, user, scope: Option<&RequestScope>)`. In the org-scoped branch, use `scope.organization_id` and `scope.role_name` (DB-driven; no Role enum).
  - `require_permission(user, db, permission, profile: Option<&CurrentProfile>)` → `require_permission(user, db, permission, scope: Option<&RequestScope>)`. Forward to resolve_permissions(db, user, scope).
  - Remove `scope_org_from_profile(profile)` from auth.rs; callers will use `scope.organization_id` directly.

---

## 3. Template app: Dashboard handlers use ScopeFromHeaders and RequestScope

**File:** `crates/forge-cli/templates/default/crates/app/src/handlers/dashboard.rs`

- Change imports: remove `CurrentProfile`, `RequireScope`; add `ScopeFromHeaders` (from crate::handlers::auth).
- Replace every handler signature: `RequireScope(profile): RequireScope` → `ScopeFromHeaders(scope): ScopeFromHeaders`.
- Replace every use of `profile.org_id` → `scope.organization_id`.
- Replace every use of `profile.role_name` (if any) with `scope.role_name`.
- Remove `scope_org_from_profile`; call sites use `scope.organization_id` directly.
- Update `require_permission` call sites: pass `Some(&scope.0)` or `Some(scope)` (depending on signature) instead of `Some(&profile)`.
- Unit tests at bottom that construct `CurrentProfile`: replace with `RequestScope { organization_id, role_id, role_name }` and adjust assertions.

---

## 4. Template app: REST handlers

**File:** `crates/forge-cli/templates/default/crates/app/src/handlers/rest.rs`

- All handlers already use `ScopeFromHeaders(scope)` and `scope.organization_id`. After moving ScopeFromHeaders to auth.rs, only add the import and remove the local definition. No further signature or logic changes.

---

## 5. Template app: WebSocket and other auth callers

**File:** `crates/forge-cli/templates/default/crates/app/src/handlers/ws.rs`

- Currently uses `get_scope_from_headers_map(req.headers(), &user, &db).await` which returns `Option<CurrentProfile>`. After refactor it returns `Option<RequestScope>`.
  - Replace `profile.as_ref().map(|p| p.org_id)` with `scope.as_ref().map(|s| s.organization_id)`.
  - Pass `scope` (Option<&RequestScope>) into `resolve_permissions` and any other call that previously took profile.

**File:** `crates/forge-cli/templates/default/crates/app/src/handlers/auth.rs`

- Any remaining callers of get_scope_from_headers / get_scope_from_headers_map (e.g. in tests): update to use Option<RequestScope> and the new helper.

---

## 6. Template app: profiles_list response

**File:** `crates/forge-cli/templates/default/crates/app/src/handlers/auth.rs`

- `profiles_list` already returns `org_id`, `role_id`, `org_name`, `role` (name). No change required for response shape. Frontend will use `role_id` for the scope header.

---

## 7. Frontend: Send X-Role-Id (role UUID), not role name

**File:** `crates/forge-cli/templates/default/frontend/src/context/Auth.tsx`

- Add `currentRoleId: string | null` to state (and persist to localStorage like currentOrgId/currentRoleName).
- When user selects a profile, set both `currentRoleId` (from profile.role_id) and keep `currentRoleName` for display if needed (or derive display from profile.role).
- Provide `currentRoleId` and setter in context.

**File:** `crates/forge-cli/templates/default/frontend/src/utils/api.ts`

- Use `currentRoleId` for the scope header: `headers.set("X-Role-Id", currentRoleId)` (not currentRoleName). Backend expects UUID.

**File:** Any component that sets the selected profile (e.g. SelectProfilePage or where profile is chosen)

- On profile select, set `currentRoleId` to the selected profile’s `role_id` (and currentOrgId to org_id). Ensure profiles from API include `role_id` (they already do).

---

## 8. Template app: Test helpers and tests

**File:** `crates/forge-cli/templates/default/crates/app/src/lib.rs`

- `auth_with_profile`: currently returns `(token, org_id, role_name)`. Change to return `(token, org_id, role_id: String)` so tests can send X-Role-Id. Implementation: from GET /api/auth/profiles, read first profile’s `org_id` and `role_id`; return (token, org_id, role_id).

**Files:** All test files under `crates/forge-cli/templates/default/crates/app/tests/` that pass scope headers:

- `dashboard_*.rs`, `auth_profile_session.rs`: change headers from `("X-Role-Name", role_name)` to `("X-Role-Id", role_id)`. Use the new `auth_with_profile` that returns role_id, or parse role_id from profiles response in the test.
- Update any helper that builds `extra_headers` to use X-Role-Id with the role_id from auth_with_profile (or equivalent).

---

## 9. Order of operations (suggested)

0. **forge-auth (forge-ltez):** RequestScope uses `role_id: Uuid` + `role_name: String`; AuthzContext::role() for RequestScope derives Role from role_name for guard compatibility.
1. **forge-auth:** Add `Role::as_role_name()` (or `role_to_name`) in authz.rs and re-export (done).
2. **auth.rs:** Add `try_scope_from_headers`, `ScopeFromHeaders` (and impl), header constants; build RequestScope with role_id + role_name from org_role. Then change `resolve_permissions` and `require_permission` to take `Option<&RequestScope>` (use scope.role_name); remove `scope_org_from_profile`; replace `get_scope_from_headers_map` to return `Option<RequestScope>`; delete `CurrentProfile` and `RequireScope`.
3. **rest.rs:** Remove local `ScopeFromHeaders`, `role_name_to_authz`, header constants; import `ScopeFromHeaders` from auth.
4. **dashboard.rs:** Replace all `RequireScope(profile)` with `ScopeFromHeaders(scope)`, `profile.org_id` → `scope.organization_id`, and any `profile.role_name` with role name from scope.role; update unit tests.
5. **ws.rs:** Use new `get_scope_from_headers_map` returning `Option<RequestScope>` and pass scope into resolve_permissions.
6. **Frontend:** Add currentRoleId, set X-Role-Id in api.ts, set currentRoleId on profile select.
7. **Tests:** Update auth_with_profile to return role_id; update all test files to send X-Role-Id with role_id.

---

## 10. Files touched (checklist)

| Area | File | Changes |
|------|------|---------|
| forge-auth | `crates/forge-auth/src/authz.rs` | RequestScope: role_id + role_name; Role → role name (as_role_name). Optional: role_name_to_role for AuthzContext::role(). |
| forge-auth | `crates/forge-auth/src/lib.rs` | Re-export if needed. |
| Template | `crates/forge-cli/templates/default/crates/app/src/handlers/auth.rs` | Add ScopeFromHeaders, try_scope_from_headers, role_name_to_authz, header constants; remove CurrentProfile, RequireScope; change resolve_permissions, require_permission, get_scope_from_headers_map to RequestScope. |
| Template | `crates/forge-cli/templates/default/crates/app/src/handlers/rest.rs` | Remove ScopeFromHeaders impl and role_name_to_authz; import ScopeFromHeaders from auth. |
| Template | `crates/forge-cli/templates/default/crates/app/src/handlers/dashboard.rs` | RequireScope → ScopeFromHeaders, profile → scope, profile.org_id → scope.organization_id; remove scope_org_from_profile; update tests. |
| Template | `crates/forge-cli/templates/default/crates/app/src/handlers/ws.rs` | get_scope_from_headers_map returns Option<RequestScope>; use scope.organization_id. |
| Template | `crates/forge-cli/templates/default/crates/app/src/lib.rs` | auth_with_profile returns (token, org_id, role_id). |
| Template | `crates/forge-cli/templates/default/frontend/src/context/Auth.tsx` | Add currentRoleId state and set on profile select. |
| Template | `crates/forge-cli/templates/default/frontend/src/utils/api.ts` | Set X-Role-Id from currentRoleId. |
| Template | `crates/forge-cli/templates/default/crates/app/tests/*.rs` | Use X-Role-Id with role_id; get role_id from auth_with_profile or profiles response. |

---

## 11. After refactor

- Run `cargo check` / `cargo build` for workspace and template app.
- Run template app tests: `cargo test -p app -- --test-threads=1` (from template root).
- Run frontend build and manual smoke test: login, select profile, call dashboard and REST endpoints (ensure X-Role-Id is sent and accepted).
