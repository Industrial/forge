# Phase 6: Authorization & RBAC (The Guard Pillar)

Forge provides a "Shallow Gate + Deep Scope" authorization framework designed for high-performance, multi-tenant SaaS applications. It prioritizes pure Rust logic over external DSLs and favors database-level isolation ("Ghost Mode") over complex middleware layers.

## 1. The Six Primitives of Authority

Forge's authorization engine is built on six core primitives:

1.  **Requester**: The session holder (e.g., a Support Agent or a User).
2.  **Subject**: The identity context being acted upon (usually `Requester == Subject`).
3.  **Object**: The resource (e.g., `Organization`, `Project`, `Invoice`).
4.  **Action**: The intent (`Read`, `Write`, `Delete`, `Manage`).
5.  **Role**: A scoped attribute (e.g., `Admin` within `Organization 5`, not a global flag).
6.  **Logic**: The binding function (Pure Rust traits) that evaluates the relationship between the other primitives to produce a **Result**.

## 2. Architecture: Shallow Gate + Deep Scope

To adhere to **Zero Trust** principles while maintaining **Lean** performance, Forge splits authorization into two layers:

### A. Deep Scope (Data Isolation)
Implicitly scopes all database queries at the ORM level. This prevents "Data Leakage" between tenants.
*   **Ghost Mode**: If a user attempts to access a resource they do not own, the query returns `None`, resulting in a `404 Not Found` rather than a `403 Forbidden`. This prevents resource enumeration.
*   **Convention**: `Entity::find().scoped(&auth_session)`

### B. Shallow Gate (Handler Guards)
Explicitly validates "Entry Rights" at the handler level for non-database actions or fast-failing.
*   **Convention**: `auth.guard(Action::Manage, Role::Admin)?`

## 3. The `ForgeAuthz` Crate

All authorization logic resides in `crates/forge-authz`. This crate provides the traits and macros required to implement ReBAC (Relationship-Based Access Control) natively in SeaORM.

### `ForgeScoped` Trait
Generated via macros for SeaORM Entities. It automatically injects filters based on the `AuthSession`.

```rust
// Implicitly filters by current user's Organization context
let invoices = Invoice::find()
  .scoped(&auth) // Logic: .filter(invoice::Column::OrgId.eq(auth.org_id()))
  .all(db)
  .await?;
```

### `ForgePolicy` Trait
Allows for complex, multi-entity relationship logic (ReBAC) defined in pure Rust.

```rust
#[async_trait]
impl ForgePolicy<Project> for User {
  async fn can(&self, action: Action, project: &Project) -> bool {
      match action {
          Action::Read => self.is_member_of(project.org_id),
          Action::Write => self.is_admin_of(project.org_id) || project.owner_id == self.id,
          _ => false,
      }
  }
}
```

## 4. Why This Approach?

1.  **No Double Fetch**: Traditional systems fetch a resource, then check permissions. Forge's **Deep Scope** makes the permission check part of the initial fetch.
2.  **Zero DSL**: No OPA/Rego or Polar files. Logic is pure Rust—type-safe, IDE-friendly, and zero-parsing overhead.
3.  **SaaS-Ready**: Multi-tenancy is baked into the database queries, making it virtually impossible to accidentally leak data between customers.
4.  **Zero-Trust**: Every request is verified against the `Subject` and `Requester` context, ensuring that even internal "impersonation" flows are audit-safe.
