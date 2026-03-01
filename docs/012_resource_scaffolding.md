# 012_resource_scaffolding: Resource Scaffolding (forge generate)

## Overview

Forge provides **resource scaffolding** via the CLI: `forge generate resource <name> [fields...]` generates files (migration, entity, handlers, route snippet) in a Rails-like way. **No injection** into existing files: the generator only creates new files and prints instructions (e.g. “add this to your router”). The user restarts the server and the new resource is available.

## Goals

- **File-only generation**: Create migration, SeaORM entity, handler module, and optionally a route snippet. Do not parse or edit existing Rust files.
- **Convention over configuration**: Output follows the conventional layout (e.g. `crates/db/src/entities/`, `crates/app/src/handlers/`).
- **Arguments or defaults**: User can pass field list (e.g. `name:string organization_id:uuid`) or accept defaults (e.g. `id`, `created_at`, `updated_at` plus a single `name:string`).
- **Multi-tenant aware**: When `organization_id` is present, entity and handlers use Forge’s scoping (e.g. `ForgeScoped`) and auth context.

## Architecture

### Generated Artifacts

| Artifact | Location | Purpose |
|----------|----------|---------|
| Migration | `crates/db/migrations/` | New table (e.g. `m..._create_projects_table.rs`). |
| Entity | `crates/db/src/entities/` | SeaORM entity (e.g. `project.rs`) with optional `ForgeScoped`. |
| Handlers | `crates/app/src/handlers/` | CRUD handlers (index, show, create, update, delete) as a module (e.g. `projects.rs`). |
| Snippet | Printed to stdout | “Add this to your router” (e.g. `router.merge(projects::routes(db))` or a block to copy into `main.rs` / `routes.rs`). |

### Field Syntax

- Simple: `name:string`, `body:text`, `count:integer`, `published:boolean`, `organization_id:uuid`.
- Defaults: `id` (UUID), `created_at`, `updated_at` added by convention unless overridden.

### Migration

- Use SeaORM migration API (or generate migration file content) to create the table with the given columns.
- If `organization_id` is in the field list, add it and document that the entity will use `ForgeScoped`.

### Handlers

- Stubs that take `State<DatabaseConnection>` and (for scoped resources) `AuthSession`; perform find_all, find_by_id, insert, update, delete.
- Return JSON or Inertia responses (convention: JSON for API, or document both).

## Generator Set

Forge provides the following generators. **Scaffold** is a compound of several others; the rest are single-purpose or small compounds.

| Generator | Purpose |
|-----------|---------|
| **Migration** | Standalone migration file (schema change). |
| **Seed** | Seed data snippet or seeder for a table/resource. |
| **Entity** | SeaORM entity only (no migration, no handlers). |
| **Job** | Background job struct + handler; cron supported via parameterization (e.g. register with `CronSchedule`). |
| **Handler** | New handler module with N actions and route snippet (no entity/migration). |
| **View** | View stubs (e.g. Inertia/React/Vue page or component). |
| **Resource** | Migration + Entity + Handlers (CRUD) + route snippet; no views. |
| **Scaffold** | Compound: Resource + View stubs + optional validation/policy (full-stack resource). |
| **Policy** | `ForgePolicy<Resource>` stub for an entity (authz). |
| **RequestValidation** | Request DTO with `Validate` and validator attrs (e.g. create/update body). |
| **ResponseValidation** | Response DTO or serializer for an entity (e.g. JSON shape). |

Not in scope for now: **Helper** (skipped), **Mailer** (deferred until mail API exists). Naming: we use **Handler** (not Controller) to match the codebase (axum handlers).

## Configuration

None. Generator runs in the current directory and expects a Forge workspace (e.g. `crates/app`, `crates/db`). If not found, print an error and suggest `forge new`.

## Usage

**Generate a resource with fields:**

```bash
forge generate resource Project name:string organization_id:uuid description:text
```

**Output:**

- `crates/db/migrations/YYYYMMDDHHMMSS_create_projects_table.rs`
- `crates/db/src/entities/project.rs`
- `crates/app/src/handlers/projects.rs`
- Instructions: “Run `forge migrate` (or your migration command). Add the following to your router: …”

**Then:**

1. Run migrations.
2. Copy the route snippet into the app.
3. Restart the server.

## Dependencies

- **forge-cli**: Implements the generator (file writing, templates or string generation). No new crate; may use `sea-orm-cli` for migration creation if desired, or generate migration code directly.

## Success Criteria

1. `forge generate resource <Name> [field:type ...]` creates the migration, entity, and handler files.
2. Entity and handlers follow Forge conventions (e.g. `ForgeScoped` when `organization_id` is present).
3. No modification of existing source files; only new files and printed snippet.
4. Documented in this doc and in CLI help.

## Future Extensions

- Implement the full generator set: **Migration**, **Seed**, **Entity**, **Job** (cron via parameterization), **Handler**, **View**, **Resource**, **Scaffold**, **Policy**, **RequestValidation**, **ResponseValidation**.
- **Job**: Forge has jobs (Apalis + SQLite); cron is a use case (scheduled tasks enqueued on a schedule). Generator produces job struct + run impl; optional params register it as a cron task.
- **Scaffold**: Compound of Resource + View stubs; optionally include RequestValidation, ResponseValidation, and Policy stubs.
- Inertia/React/Vue view stubs (View generator).
- Seed snippet for the new resource (Seed generator).
