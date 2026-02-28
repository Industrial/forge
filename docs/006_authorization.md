# Phase 6: Authorization & RBAC (The Guard Pillar)

Forge provides a "Shallow Gate + Deep Scope" authorization framework designed for high-performance, multi-tenant SaaS applications. It prioritizes pure Rust logic over external DSLs and favors database-level isolation ("Ghost Mode") over complex middleware layers.

## 1. The Six Primitives of Authority

Forge's authorization engine is built on six core primitives:

1. **Requester**: The session holder (e.g., a Support Agent or a User).
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
| **Requester** | `auth.requester_id()`   | `uuid::Uuid` – who is holding the session. |
| **Subject**   | `auth.subject_id()`     | `uuid::Uuid` – identity context (self or impersonated). |
| **Organization** | `auth.organization_id()` | `Option<uuid::Uuid>` – current tenant scope. |
| **Object** | Generic type `R` in `ForgePolicy<R>` | Any entity (e.g. `Project`, `Invoice`). |
| **Logic**  | `trait ForgePolicy<R>`  | `async fn can(...) -> Result<bool, AuthzError>`. |

All of the above are re-exported in `forge::prelude` for use in handlers and policies.

## 2. Architecture: Shallow Gate + Deep Scope

### A. Deep Scope (Data Isolation)

Implicitly scopes all database queries at the ORM level to prevent data leakage between tenants.

- **Ghost Mode**: If a user attempts to access a resource they do not own, the query returns `None`, resulting in **404 Not Found** rather than **403 Forbidden**, preventing resource enumeration.
- **Convention**: `Entity::find().scoped(&auth_session)` (requires entity to derive `ForgeScoped` and have an `organization_id` column).

### B. Shallow Gate (Handler Guards)

Explicitly validates entry rights at the handler level for non-database actions or fast-failing.

- **Convention**: `auth.guard(Action::Manage, Role::Admin)?` – returns `Err(AuthzError::Forbidden)` if the session user’s role does not match.

## 3. Secure-by-Default Implementation (for `forge new`)

A generated application must include the following so that every user has a Requester, Organization, and Role and the app is a complete authn + authz solution.

### A. Core Database Schema (Migrations)

| Table           | Columns | Purpose |
|----------------|---------|---------|
| **user**       | `id`, `email`, `password_hash`, `is_active`, `is_admin`, `current_org_id`, `current_role`, `created_at`, `updated_at` | Identity; `current_org_id` and `current_role` track active scope and role for the session. |
| **organization** | `id`, `name`, `slug`, `created_at`, `updated_at` | Multi-tenant container. |
| **membership** | `id`, `user_id`, `org_id`, `role`, `created_at`, `updated_at` | Links users to organizations with a specific `Role` (stored as string, e.g. `owner`, `admin`, `editor`, `viewer`). |
| **sessions**   | (existing) | Session store. |

Migration order:

1. `m20220101_000001_create_user_table` – user table including nullable `current_org_id` (UUID) and `current_role` (string).
2. `m20220101_000002_create_sessions_table` – sessions table.
3. `m20220101_000003_create_organizations_table` – organizations table.
4. `m20220101_000004_create_memberships_table` – memberships table (foreign keys to user and organization).

### B. Atomic Registration Flow

When a user signs up, the application must perform an atomic transaction:

1. **Create User** (the Requester).
2. **Create Organization** (default tenant, e.g. “Personal” or derived from email).
3. **Create Membership** (link user + org with `Role::Owner`).
4. **Set current context**: Update `user.current_org_id` and `user.current_role` to the new org and `"owner"`.

All in a single database transaction so the user is immediately in a valid authz state.

### C. Auth Backend (Eager-Load Role and Org)

The `AuthnBackend` used by the app must ensure that the loaded user has `current_org_id` and `current_role` set so that `AuthzContext` and `guard()` work:

- **authenticate**: After validating credentials, load the user’s default (or first) membership; set `user.current_org_id` and `user.current_role` from that membership, then return the user.
- **get_user**: When loading the user by ID, re-load or join membership so that `current_org_id` and `current_role` are populated (or keep them stored on the user row and read them).

This makes the session “authz-ready” without extra lookups in every handler.

### D. The `AuthzContext` Trait

The `User` model (or the type used as `AuthnBackend::User`) must implement `AuthzContext` so that `AuthSession` can provide Requester, Subject, Organization, and Role:

```rust
impl AuthzContext for user::Model {
    fn requester_id(&self) -> Uuid {
        self.id
    }
    fn subject_id(&self) -> Uuid {
        self.id
    }
    fn organization_id(&self) -> Option<Uuid> {
        self.current_org_id
    }
    fn role(&self) -> Option<Role> {
        self.current_role.as_deref().and_then(|s| match s {
            "owner" => Some(Role::Owner),
            "admin" => Some(Role::Admin),
            "editor" => Some(Role::Editor),
            "viewer" => Some(Role::Viewer),
            _ => None,
        })
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
    auth: AuthSession<Backend>,
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
    auth: AuthSession<Backend>,
    State(db): State<DatabaseConnection>,
) -> Result<StatusCode, Error> {
    auth.guard(Action::Delete, Role::Owner)?;

    // ... delete logic ...
    Ok(StatusCode::NO_CONTENT)
}
```

### `ForgeScoped` Trait

Entities that belong to an organization should derive `ForgeScoped` (and have an `organization_id` column). The macro injects a filter so that `.scoped(&auth)` restricts rows to `auth.organization_id()`.

```rust
// In db crate, entity with organization_id column
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, ForgeScoped)]
#[sea_orm(table_name = "project")]
pub struct Model {
    // ...
    pub organization_id: Uuid,
    // ...
}
```

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
