# 003_database: Phase 3 Database & ORM (SeaORM)

## Overview

The `003_database` phase integrates **SeaORM** as the primary ORM for the Forge framework. This phase enables automated database connection management based on a conventional project structure and configuration, focusing on **SQLite** for zero-friction local development.

## Goals

- **Zero-Boilerplate Connectivity**: Forge automatically manages the database lifecycle.
- **Convention over Configuration**: Database settings live in a dedicated `config/db.toml`.
- **Developer-Centric Defaults**: `forge new` generates a ready-to-use SQLite configuration.
- **Direct Injection**: The database connection is automatically provided to Axum handlers via `State`.
- **Fail-Fast Strategy**: Missing or invalid database configuration results in an immediate crash with a helpful error.

## Architecture Decisions

### 1. Dedicated Database Configuration (`config/db.toml`)

**Decision**: Database-specific settings are stored in `config/db.toml`, separate from the general `app.toml`.

**Rationale**:
- Keeps `app.toml` focused on core application behavior.
- Clearly separates concerns (e.g., server settings vs. data storage).
- Follows the convention of other major frameworks like Rails or AdonisJS.

**Generated `config/db.toml`**:
```toml
# Forge Database Configuration
# See: https://forge.rs/docs/database

[database]
# SQLite connection string. The file will be created in the project root.
url = "sqlite://db.sqlite?mode=rwc"
max_connections = 5
min_connections = 1
connect_timeout = 10
idle_timeout = 600
```

### 2. Automatic Initialization in `App::new()`

**Decision**: The Forge `App` builder automatically reads `config/db.toml` and initializes a `sea_orm::DatabaseConnection` during instantiation.

**Implementation Logic**:
- If `config/db.toml` exists, it **must** contain a valid `[database]` section with a `url`.
- If the file exists but the connection fails, the application crashes immediately.
- If the file is missing, the application currently crashes (enforcing the conventional layout).

### 3. Automatic State Injection

**Decision**: Forge automatically wraps the `DatabaseConnection` in Axum's `State` and injects it into the application router.

**User Experience**:
The user does **not** need to manually configure the Axum `State` or add layers. They simply use the `State` extractor in their handlers:

```rust
use axum::extract::State;
use sea_orm::DatabaseConnection;

async fn my_handler(State(db): State<DatabaseConnection>) {
    // db is ready to use for queries
}
```

### 4. SQLite as the "Immediate Start" Default

**Decision**: `forge new` defaults to a local SQLite database file named `db.sqlite` in the project root.

**Rationale**:
- Enables new developers to run `forge serve` and have a working database-backed application instantly.
- No external dependencies (PostgreSQL/MySQL) required for initial development.
- Single-file storage makes it easy to inspect or delete during early prototyping.

## Implementation Plan

### Phase 3A: Library Core (`crates/forge/`)

**Dependencies to Add**:
```toml
# crates/forge/Cargo.toml
[dependencies]
sea-orm = { version = "1.1", features = ["runtime-tokio-rustls", "sqlx-sqlite", "macros"] }
```

**Database Module** (`src/db.rs`):
- `DatabaseConfig` struct for deserialization.
- `initialize_database(config: &DatabaseConfig)` function returning a `DatabaseConnection`.
- Error handling for connection failures.

**App Builder Updates** (`src/app.rs`):
- Load `config/db.toml` in `App::new()`.
- Store the `DatabaseConnection` in the `App` struct.
- In `.serve()`, add the `DatabaseConnection` as Axum `State`.

### Phase 3B: CLI Enhancement (`crates/forge-cli/`)

**`forge new` updates**:
- Include `config/db.toml` in the project template.
- (Optional) Provide a commented-out example route in `src/main.rs` demonstrating `State<DatabaseConnection>`.

### Phase 3C: E2E Test (`test/e2e/003_database.rs`)

**Test Scenarios**:
1. **Config Generation**: Verify `forge new` creates `config/db.toml` with the correct SQLite URL.
2. **Startup Connectivity**: Verify `forge serve` starts without error when a valid `config/db.toml` is present.
3. **Data Access**: Register a route that performs a basic DB operation (e.g., `db.get_database_backend()`) and verify the HTTP response.
4. **Failure Handling**: Verify `forge serve` fails if `config/db.toml` is present but contains an invalid URL.

## Success Criteria

- `forge new myapp` creates a directory with `config/db.toml`.
- `config/db.toml` defaults to `sqlite://db.sqlite?mode=rwc`.
- Axum handlers can successfully extract `State<DatabaseConnection>`.
- The framework correctly handles the connection pool lifetime.
- Clear error messages are provided if the database cannot be reached.

## Integration Notes

### Phase 2 Compatibility
Phase 3 builds on the configuration system from Phase 2. `load_config` will be extended or mirrored to handle `db.toml`.

### Phase 4 Preview (Migrations)
While Phase 3 provides connectivity, Phase 4 will introduce the `database/migrations/` structure and the `forge db migrate` command.
