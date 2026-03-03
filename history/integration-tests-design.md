# Integration test strategy: where and how

## 1. Where do integration tests live?

There are two distinct layers; each has its own “every path” and its own test home.

### Layer A: Framework (forge)

**Scope:** Forge’s own guarantees. No notion of “users”, “organizations”, or “dashboard.permissions.read”. Only: auth middleware, session, migrations, health, token lookup, rate limits, etc.

**Examples of paths:**
- Unauthenticated request to a route protected by `RequireAuth` → 401.
- Authenticated request with valid session → handler runs.
- `GET /healthz`, `GET /livez`, `GET /readyz` → 200 with expected body.
- App with migrations and seed runs without panic; DB is migrated and seeded.

**Where:** In the **forge repo**, e.g.:
- `crates/forge/` (tests that build a minimal `App` with auth/migrations and assert behavior), or
- A dedicated `crates/forge-integration-tests/` that uses a **minimal app** (few routes, no dashboard) to test framework behavior.

**What “100%” means here:** Every framework feature that affects request/response (auth, session, token, health, migrations) is exercised by at least one integration test. The list is finite and small.

---

### Layer B: Application (default template)

**Scope:** The **template app’s** API: every route, every method, under every relevant user/org state. This is “every possible path of interaction” for a **user** of the template backend.

**Examples of paths:**
- `GET /api/dashboard/users` as unauthenticated → 401.
- `GET /api/dashboard/users` as `viewer@default.org` (Default org) → 200, only Default org users, no “Other” in memberships.
- `GET /api/dashboard/users` as `viewer@other.org` (Other org) → 200, only Other org users.
- `POST /api/dashboard/users` with `org_id = Other` as `viewer@default.org` → 403.
- `DELETE /api/dashboard/users` as viewer → 403 (no write permission).

**Where:** With the **default template** (the application), not inside the forge crate:
- **Option B1:** Inside the template’s app crate, e.g. `crates/forge-cli/templates/default/crates/app/tests/` (integration tests that build the template’s router + in-memory DB and call it in-process), or
- **Option B2:** A separate crate in the **template workspace** (e.g. `crates/forge-cli/templates/default/crates/integration-tests/`) that depends on `app` and optionally uses a shared test router builder exposed by the app.

**What “100%” means here:** Every (route × method × user class × org/scope variant) that the template defines is exercised with the expected outcome (success / 401 / 403 / 404 / 422). That list is derived below.

---

### Summary

| Layer   | What we’re testing        | Where tests live                          |
|---------|---------------------------|-------------------------------------------|
| Forge   | Auth, session, health, DB | Forge repo (forge or forge-integration-tests) |
| Template| Full API + authz + scope   | Template app (app/tests or template integration crate) |

So: **forge** is tested for framework behavior; **forge + default template** is tested for “every user path” on the template’s backend. The “100% fool-proof” list you care about is the **template** list; the forge list is smaller and separate.

---

## 2. How to derive the full list of (template) integration tests

To approach “every possible path” in a systematic way, we do three things: enumerate the API, enumerate user/context states, then define the coverage model and generate the test list.

### Step 1: Enumerate the API surface

List every (method, path) and any query/body parameters that change behavior. For the default template, from `main.rs` and the handlers:

**Auth (no permission checks, but session required for some):**
- `POST /api/auth/register` — public
- `POST /api/auth/login` — public
- `POST /api/auth/logout` — authenticated
- `GET /api/auth/profile` — authenticated
- `GET /api/auth/profiles` — authenticated
- `POST /api/auth/switch-profile` — authenticated (body: org_id)
- `GET /api/auth/session` — authenticated
- `POST /api/auth/tokens` — authenticated
- `GET /api/auth/admin` — authenticated (admin-only)

**Dashboard (permission + org scope where relevant):**
- `GET /api/dashboard/permissions` — auth + `dashboard.permissions.read`
- `GET /api/dashboard/role-permissions` — auth + `dashboard.permissions.read`, scope: current org
- `POST /api/dashboard/role-permissions` — auth + `dashboard.permissions.write`, scope: current org
- `DELETE /api/dashboard/role-permissions` — auth + `dashboard.permissions.write`, scope: current org
- `GET /api/dashboard/tasks` — auth (any)
- `GET /api/dashboard/audit-log` — auth + `dashboard.audit.read`
- `GET /api/dashboard/organizations` — auth + `dashboard.organizations.read`
- `POST /api/dashboard/organizations` — auth + `dashboard.organizations.write`
- `PATCH /api/dashboard/organizations` — auth + `dashboard.organizations.write`
- `DELETE /api/dashboard/organizations` — auth + `dashboard.organizations.write`
- `GET /api/dashboard/users` — auth + `dashboard.users.read`, scope: current org
- `POST /api/dashboard/users` — auth + `dashboard.users.write`, scope: current org only (body.org_id must equal current_org_id)
- `PATCH /api/dashboard/users` — auth + `dashboard.users.write`, scope: current org
- `DELETE /api/dashboard/users` — auth + `dashboard.users.write`, scope: current org
- `GET /api/dashboard/roles` — auth + `dashboard.roles.read`, scope: current org
- `POST /api/dashboard/roles` — auth + `dashboard.roles.write`, scope: current org
- `PATCH /api/dashboard/roles` — auth + `dashboard.roles.write`, scope: current org
- `DELETE /api/dashboard/roles` — auth + `dashboard.roles.write`, scope: current org

**Other:**
- `GET /api/cache-demo`, `GET /api/cached-page`, `GET /api/observability/trace-id` — behavior can be defined (auth or not) and added to the list.
- `GET /ws` — WebSocket; separate test strategy (e.g. connect with session, assert subscribed channels).

So we have a finite set of **(method, path)** pairs and, for each, a short spec: public vs auth, which permission(s), and whether scope is “current org only” or “global”.

---

### Step 2: Enumerate user/context states (equivalence classes)

We don’t need a test per user id; we need one per **equivalence class** of (auth, permissions, org context). Suggested classes for the template:

1. **Unauthenticated**
2. **Authenticated, no current org** (e.g. just logged in, never switched — if the app allows that)
3. **Authenticated, current org = Default**
   - **viewer** (dashboard.users.read, dashboard.roles.read, dashboard.permissions.read, dashboard.audit.read; no write)
   - **editor** (read + some write; from seeds)
   - **admin** (org admin: more write)
   - **owner** (org owner: full org)
4. **Authenticated, current org = Other**
   - **viewer**, **admin**, **owner** (same idea as above but for Other)
5. **Authenticated, global admin** (admin@admin.com — all permissions, but we still scope list/mutate to current org in the template)
6. **Multi-org user** (e.g. multi@email.com): current org = Default then current org = Other (switch-profile), to test that list/mutate scope follows current org

We can label these as: `anon`, `viewer_default`, `editor_default`, `admin_default`, `owner_default`, `viewer_other`, `owner_other`, `global_admin`, `multi_default`, `multi_other`. Each of these is a “user state” we’ll use as a row or dimension in the matrix.

---

### Step 3: Coverage model and test list generation

We want two kinds of guarantees:

- **Authz (authorization):** For each protected route, “allowed” user → success, “disallowed” user → 403, unauthenticated → 401.
- **Scope:** For org-scoped routes, “user in org A, operate on A” → success; “user in org A, operate on B” (if the API allows specifying B) → 403 or equivalent.

**Model: route × user_class → expected outcome**

For each (method, path) that requires auth:

- Add a test: unauthenticated → 401.
- For each user class that **has** the required permission and (when relevant) correct org:
  - Add a test: expected 200/201/204 and, for list endpoints, assert response shape and that no data from another org leaks.
- For each user class that **lacks** the required permission (or uses wrong org):
  - Add a test: expected 403.

For **public** routes (e.g. login, register):

- Add tests for “valid request → success” and “invalid request → 4xx” as needed.

For **scope-sensitive** routes (users, roles, role-permissions, and mutate with org_id):

- Add tests that “user in Default, list users” returns only Default users (and no “Other” in memberships).
- Add tests that “user in Default, POST users with org_id = Other” → 403.
- Optionally: “user in Other, list users” returns only Other users.

**Concrete generation algorithm**

1. **Build route list** from Step 1 (and optionally from OpenAPI/spec if you generate one).
2. **Build user-class list** from Step 2 (seeded users or roles that cover each equivalence class).
3. **For each route R:**
   - If R is public: add tests for success and validation failures (no auth).
   - If R requires auth:
     - Add one test: no cookie → 401.
     - For each user class U: compute expected outcome (success / 403) from R’s permission and scope and U’s permissions and current_org_id.
     - Add one test per (R, U) with that outcome; for success, add assertions on response shape and scope (no cross-org data).
4. **Add scope invariants:** For each org-scoped list endpoint, at least one test with a user in org A that asserts the response contains only org A data (e.g. no “Other” in memberships for `viewer@default.org` on `GET /api/dashboard/users`).
5. **Add mutation scope tests:** For create/update/delete that take org_id or resource id, add tests that try to mutate another org → 403 (or 404 if you hide existence).

That gives a **finite, enumerable list** of test cases. The “100%” is relative to this model: every (route, user_class) pair has a defined expected outcome and a test that asserts it.

---

## 3. Suggested next steps

1. **Decide test location for template:** e.g. `templates/default/crates/app/tests/` (in-process, router + test DB) or a sibling `integration-tests` crate. Prefer in-process for speed and determinism; add a small “running server” suite only if you want a smoke test of the real binary. **Decision:** Template integration tests live in `crates/app/tests/`; the app exposes `build_router_for_test()` so tests get the same router with a temp config and in-memory DB.
2. **Export a test router (and optionally test DB) from the template app** so integration tests can build the same app with a temp config + in-memory (or temp-file) DB and call the router without binding to a port.
3. **Implement the derivation:** maintain a single source of truth (e.g. a table or a small DSL) for “route R + user class U → outcome”, then generate or hand-write one test per cell. That way the “list” is explicit and you can check that every route and every user class is covered.
4. **For forge (framework):** keep a short list of framework integration tests (auth, health, session, migrations) in the forge repo; that list is independent of the template’s “every user path” list.

Once the route list and user classes are fixed, the test count is bounded and you can say: “every path of user interaction we care about is covered by this set of tests.”
