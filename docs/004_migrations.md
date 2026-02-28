# 004_migrations: Phase 4 Migrations & Seeding

## Status
Proposed (Debated)

## Context
In Phase 3, we successfully integrated SeaORM and SQLite. However, schema changes were handled manually. To achieve a "Zero-Configuration" developer experience, Forge needs a system to evolve the database schema and populate it with initial data automatically.

## Decision
Implement a **Type-Safe Migration and Seeding Engine** integrated directly into the `forge` runtime and CLI.

### 1. Workspace-First Architecture
`forge new <name>` will now generate a Cargo Workspace:
- `crates/app/`: The web application logic.
- `crates/db/`: The database schema, migrations, and seeds.

This separation is required by SeaORM to avoid circular dependencies between Entities and the Migrator.

### 2. Automatic Migration & Seeding
The `App` runtime will orchestrate the following sequence upon calling `.serve()`:
1. **Connect**: Establish the `DatabaseConnection`.
2. **Migrate**: Run all pending migrations from `crates/db`.
3. **Seed**: Execute the idempotent seeding logic.
4. **Bind**: Start the Axum listener.

### 3. Explicit Registration
Instead of auto-discovery, developers explicitly register their migrators:
```rust
// crates/app/src/main.rs
App::new()
  .with_migrations(db::Migrator)
  .with_seed(db::seed)
  .serve()
  .await
```

### 4. Configuration
Added to `ForgeConfig` (and `db.toml`):
- `auto_migrate`: Boolean (default `true`).
- `auto_seed`: Boolean (default `true`).

### 5. Default "Users" Schema
`forge new` will include an initial migration creating a `users` table:
- `id`: UUID (Primary Key)
- `email`: String (Unique)
- `password`: String (Plaintext for Phase 4; hashed in Phase 5)
- `created_at`: DateTime
- `updated_at`: DateTime

A default idempotent seed will create a `root@localhost` user if one does not exist.

## Implementation Plan

### Phase 4.1: Workspace Generation
- [ ] Update `forge-cli` to generate a `Cargo.toml` workspace.
- [ ] Move `src/main.rs` template to `crates/app/src/main.rs`.
- [ ] Create `crates/db/` with SeaORM Migration boilerplates.

### Phase 4.2: Runtime Orchestration
- [ ] Add `with_migrations` and `with_seed` methods to `App`.
- [ ] Update `App::serve` to execute the sequence.
- [ ] Add configuration flags to `ForgeConfig`.

### Phase 4.3: Default Schema
- [ ] Implement the `m20220101_000001_create_user_table` migration.
- [ ] Implement the idempotent `root` user seed.

## Success Criteria
1. `forge new my_project` creates a working workspace.
2. Running `forge serve` automatically creates the `db.sqlite` and the `users` table.
3. Accessing a `/db-check` route confirms that the `root@localhost` user exists in the database.
4. E2E tests for the whole suite run in < 1 second.
