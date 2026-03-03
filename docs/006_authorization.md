# Phase 6: Authorization & RBAC (The Guard Pillar)

Forge provides a "Shallow Gate + Deep Scope" authorization framework designed for high-performance, multi-tenant SaaS applications. It prioritizes pure Rust logic over external DSLs and favors database-level isolation ("Ghost Mode") over complex middleware layers.

## 1. The Six Primitives of Authority

Forge's authorization engine is built on six core primitives:

1. **Requester**: The authenticated identity (e.g., a Support Agent or a User), established via Bearer token.
2. **Subject**: The identity context being acted upon (usually `Requester == Subject`).
3. **Object**: The resource (e.g., `Organization`, `Project`, `Invoice`).
4. **Action**: The intent (`Read`, `Create`, `Update`, `Delete`, `Manage`).
5. **Role**: A scoped attribute (e.g., `Admin` within `Organization 5`, not a global flag).
6. **Logic**: The binding function (Pure Rust traits) that evaluates the relationship between the other primitives to produce a **Result**.

### Rust Constructs (Primitives Cheat Sheet)

| Primitive   | Rust construct              | Usage |
|------------|-----------------------------|--------|
| **Action** | `enum Action`               | `Action::Read`, `Action::Manage`, etc. |
| **Role**   | `enum Role`                 | `Role::Owner`, `Role::Admin`, etc. |
| **Requester** | `auth.requester_id()`   | `uuid::Uuid` – authenticated user (from token). |
| **Subject**   | `auth.subject_id()`     | `uuid::Uuid` – identity context (self or impersonated). |
| **Organization** | `auth.organization_id()` or scope | `Option<uuid::Uuid>` – tenant scope (from `X-Organization-Id` or request scope). |
| **Object** | Generic type `R` in `ForgePolicy<R>` | Any entity (e.g. `Project`, `Invoice`). |
| **Logic**  | `trait ForgePolicy<R>`  | `async fn can(...) -> Result<bool, AuthzError>`. |

Import what you need from `forge` and `forge::authz` (e.g. `use forge::App;`, `use forge::authz::{Action, Role, AuthzContext};`).

## 2. Architecture: Shallow Gate + Deep Scope

### A. Deep Scope (Data Isolation)

Implicitly scopes all database queries at the ORM level to prevent data leakage between tenants.

- **Ghost Mode**: If a user attempts to access a resource they do not own, the query returns `None`, resulting in **404 Not Found** rather than **403 Forbidden**, preventing resource enumeration.
- **Convention**: `Entity::find().scoped(&scope)` (requires entity to derive `ForgeScoped` and have an `organization_id` column); scope comes from the request (e.g. `RequestScope` or token user + headers).

### B. Shallow Gate (Handler Guards)

Explicitly validates entry rights at the handler level for non-database actions or fast-failing.

- **Convention**: `auth.guard(Action::Manage, Role::Admin)?` – returns `Err(AuthzError::Forbidden)` if the authenticated user’s role (from token/scope) does not match.

## 3. Secure-by-Default Implementation (for `forge new`)

A generated application must include the following so that every user has a Requester, Organization, and Role and the app is a complete authn + authz solution.

### A. Core Database Schema (Migrations)

| Table           | Columns | Purpose |
|----------------|---------|---------|
| **user**       | `id`, `email`, `password_hash`, `is_active`, `is_admin`, `created_at`, `updated_at` | Identity; scope (org, role) comes from request headers (`X-Organization-Id`, `X-Role-Name`) or from membership lookup. |
| **organization** | `id`, `name`, `slug`, `created_at`, `updated_at` | Multi-tenant container. |
| **membership** | `id`, `user_id`, `org_id`, `role`, `created_at`, `updated_at` | Links users to organizations with a specific `Role` (stored as string, e.g. `owner`, `admin`, `editor`, `viewer`). |

Migration order:

1. `m20220101_000001_create_user_table` – user table.
2. `m20220101_000002_create_organizations_table` – organizations table.
3. `m20220101_000003_create_memberships_table` – memberships table (foreign keys to user and organization).
4. (Optional) `m..._create_api_tokens_table` – token hashes for Bearer auth.

### B. Atomic Registration Flow

When a user signs up, the application must perform an atomic transaction:

1. **Create User** (the Requester).
2. **Create Organization** (default tenant, e.g. “Personal” or derived from email).
3. **Create Membership** (link user + org with `Role::Owner`).
All in a single database transaction. The client receives a token; scope (org, role) is sent on each request via headers (`X-Organization-Id`, `X-Role-Name`) or derived from the token and membership.

### C. Auth and Scope

The app uses token auth (Bearer) and optional scope extractors:

- **Identity**: Validated from `Authorization: Bearer <token>`; the handler receives a `TokenUser` (or similar) with user id.
- **Scope**: Request scope (organization, role) comes from headers or from a `RequestScope` extractor that validates the user's membership and injects org/role. This feeds `AuthzContext` and `guard()`.

### D. The `AuthzContext` Trait

The type representing the authenticated user (and scope) must implement `AuthzContext` so that handlers can access Requester, Subject, Organization, and Role:

```rust
// Example: AuthzContext can be implemented by a wrapper that holds user + scope (org/role from request).
impl AuthzContext for AuthenticatedUser {
    fn requester_id(&self) -> Uuid {
        self.user.id
    }
    fn subject_id(&self) -> Uuid {
        self.user.id
    }
    fn organization_id(&self) -> Option<Uuid> {
        self.organization_id  // from X-Organization-Id / RequestScope
    }
    fn role(&self) -> Option<Role> {
        self.role.as_ref().cloned()  // from X-Role-Name or membership lookup
    }
}
```

### E. Default System Seeding

The seed run at startup should create a minimal bootstrap state in an idempotent way, for example:

- One default **Organization** (e.g. “Default” or “System”).
- One **User** (e.g. `root@localhost`) with a known password.
- One **Membership** linking that user to the org with `Role::Owner`.
- Set that user’s `current_org_id` and `current_role` to the new org and `"owner"`.

This gives a ready-to-use admin identity for the generated app.

## 4. Using the Traits in Handlers

### Example: Ghost Mode (Deep Scope)

Fetch a resource only if it belongs to the current user’s organization. Unauthorized access yields 404.

```rust
pub async fn get_project(
    auth: RequireAuth,  // or your token + scope extractor
    Path(id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<Json<Project>, Error> {
    let project = Project::find_by_id(id)
        .scoped(&auth)
        .one(&db)
        .await?
        .ok_or(Error::NotFound)?;

    Ok(Json(project))
}
```

### Example: Shallow Gate (Guard)

Restrict an action to a specific role (e.g. only Owner can delete the org).

```rust
pub async fn delete_org(
    auth: RequireAuth,  // or your token + scope extractor
    State(db): State<DatabaseConnection>,
) -> Result<StatusCode, Error> {
    auth.guard(Action::Delete, Role::Owner)?;

    // ... delete logic ...
    Ok(StatusCode::NO_CONTENT)
}
```

### `ForgeScoped` Trait

Entities that belong to an organization can use the `ForgeScoped` trait (from `forge_auth`) so that `.scoped(&context)` restricts rows to `context.organization_id()`. Implement `ForgeScoped<Entity>` for `Select<Entity>` (filter by the appropriate column, e.g. `organization_id`).

### `ForgePolicy` Trait (ReBAC)

For relationship-based rules, implement `ForgePolicy` on the user (or a policy type) for a resource type:

```rust
#[async_trait]
impl ForgePolicy<Project> for User {
  async fn can<C: AuthzContext>(
    &self,
    context: &C,
    action: Action,
    resource: &Project,
    db: &DatabaseConnection,
  ) -> Result<bool, AuthzError> {
    match action {
      Action::Read => Ok(context.organization_id() == Some(resource.organization_id)),
      Action::Update => {
        let role = context.role();
        Ok(role == Some(Role::Owner) || role == Some(Role::Admin))
      }
      _ => Ok(false),
    }
  }
}
```

## 5. Why This Approach?

1. **No double fetch**: Permission check is part of the scoped fetch.
2. **Zero DSL**: Logic is pure Rust – type-safe and no parsing overhead.
3. **SaaS-ready**: Multi-tenancy is baked into queries.
4. **Zero-trust**: Requester, Subject, and Organization are explicit; impersonation flows remain audit-safe.
