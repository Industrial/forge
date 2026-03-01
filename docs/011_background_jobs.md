# 011_background_jobs: Background Jobs (Apalis + SQLite)

## Overview

Forge provides **background job processing** as a **core** feature using [apalis](https://github.com/apalis-dev/apalis) with **SQLite** storage. Jobs are dispatched from request handlers and processed by a worker (in-process or separate process), so apps can defer work (e.g. email, exports) without blocking the response.

## Goals

- **Core dependency**: Every Forge app can use jobs; no optional feature.
- **SQLite by default**: No extra services for development or small deployments; job state lives in the app’s SQLite DB (or a dedicated job DB).
- **Type-safe jobs**: Jobs are structs that implement a trait; dispatch and run are type-checked.
- **Single API**: `forge::jobs::dispatch(job)` (or equivalent) and a standard way to run workers (e.g. `forge jobs:work` or document running the app with a worker).

## Architecture

### Components

| Component | Responsibility |
|-----------|----------------|
| **Storage** | SQLite (apalis-sqlite); jobs are stored in tables. |
| **Dispatcher** | Called from handlers; enqueues a job (e.g. `SendEmailJob { to, subject }`). |
| **Worker** | Polls storage, runs job handlers, marks complete/failed. |

### Job Definition

Jobs are serializable structs. A handler type implements apalis’s job trait (e.g. `Job` with `async fn run(&self, ctx)`). Forge may wrap this in a thin API so users implement a Forge trait and get dispatch + worker registration.

### Worker Execution

- **Option A**: Same process (spawn a worker task in `serve()`).
- **Option B**: Separate process running `forge jobs:work` (or `cargo run -- jobs:work`) that connects to the same SQLite DB.

Config (e.g. in `config/app.toml`) may control worker concurrency and queue names.

## Configuration

Example in `config/app.toml` or `config/db.toml`:

```toml
[jobs]
# Use same DB as app, or dedicated URL
database_url = "sqlite://db.sqlite?mode=rwc"
# Worker: concurrency, poll interval, etc. (if in-process or CLI)
worker_concurrency = 2
```

If `database_url` is omitted, use the main app database URL.

## Usage

**Define a job:**

```rust
use forge::jobs::{Job, JobContext};

#[derive(Clone, Serialize, Deserialize)]
pub struct SendEmailJob {
    pub to: String,
    pub subject: String,
}

#[async_trait]
impl Job for SendEmailJob {
    async fn run(&self, _ctx: JobContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // send email
        Ok(())
    }
}
```

**Dispatch from a handler:**

```rust
forge::jobs::dispatch(SendEmailJob { to, subject }).await?;
```

**Run worker:** Document `forge jobs:work` (or in-process worker) that processes the queue.

## Dependencies

- **apalis**: Job trait, worker, middleware.
- **apalis-sqlite**: SQLite storage backend (version aligned with apalis; may be 1.0.0-rc.x until stable).

## Success Criteria

1. Apps can define jobs and dispatch them from handlers.
2. Jobs are persisted in SQLite and processed by a worker.
3. Worker can run in the same process or as a separate CLI command.
4. No extra infrastructure (Redis, etc.) required for the default setup.

## Future Extensions

- PostgreSQL/Redis backends for scaling.
- Scheduled jobs (cron-like).
- Job retry and dead-letter configuration.
- Dashboard (e.g. apalis-board) as optional.
