# 011_background_jobs: Background Jobs + Cron Jobs

**Cron as jobs:** Implemented. Cron is a use case of jobs: scheduler (same process as HTTP) enqueues `ScheduledTaskJob` on schedule; Apalis worker runs them by task name. `forge::cron::CronSchedule` (Interval, Hourly, Daily), `App::with_cron(name, schedule, task)`. Storage: Apalis + SQLite (same DB URL as app). E2E: `test/e2e/011_cron.rs`. Unit: `forge::cron::tests`, `forge::jobs::tests`. Request-triggered job dispatch (e.g. `forge::jobs::dispatch`) not yet implemented.

## Overview

Forge provides **background job processing** (Apalis + SQLite) and **cron-style scheduled tasks** so apps can defer work and run recurring logic without extra infrastructure.

- **Background jobs**: Dispatched from request handlers; processed by a worker (in-process or separate process).
- **Cron jobs**: Recurring tasks run on a schedule (interval, hourly, daily) in the same process as the server.

## Goals

- **Core dependency**: Every Forge app can use jobs and cron; no optional feature.
- **SQLite by default**: Job state lives in the app’s SQLite DB (or a dedicated job DB).
- **Type-safe jobs**: Jobs are structs that implement a trait; dispatch and run are type-checked.
- **Single API**: `forge::jobs::dispatch(job)` and a standard way to run workers (e.g. `forge jobs:work`).
- **Cron jobs**: Minimal schedule DSL (interval / hourly / daily) without a cron-parsing library; run in-process alongside the server.

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

---

## Cron Jobs

Scheduled tasks that run at fixed intervals or at clock times (e.g. every hour, daily at 03:00), in the same process as the HTTP server.

### Can we do minimal cron without a library?

Yes. We can avoid a cron-parsing crate by supporting a small schedule model and using **tokio** + **chrono** (already in Forge):

| Schedule type | Meaning | Implementation |
|---------------|---------|----------------|
| **Interval** | Every `Duration` (e.g. 5 minutes) | `tokio::time::interval(duration)`; fire, then tick. No calendar logic. |
| **Hourly** | Every hour at minute `M` (e.g. :00, :30) | `chrono` to compute next `hour:minute`, then `sleep_until` / `interval`. |
| **Daily** | Once per day at `HH:MM` (e.g. 03:00) | Same: next occurrence of that time, then sleep. |

No need for full cron expressions (`0 */2 * * *`) in the minimal version; we can add a `cron` crate later for power users. This keeps the core implementation simple and dependency-light.

### Proposed API

**1. Schedule type (no external cron crate):**

```rust
pub enum CronSchedule {
    /// Every fixed duration (e.g. every 5 minutes).
    Interval(std::time::Duration),
    /// Every hour at the given minute (0..60).
    Hourly { minute: u32 },
    /// Once per day at the given hour and minute (UTC or configurable).
    Daily { hour: u32, minute: u32 },
}
```

**2. Register a cron task with the app:**

```rust
app
    .with_cron("cleanup", CronSchedule::Daily { hour: 3, minute: 0 }, cleanup_expired_sessions)
    .with_cron("metrics", CronSchedule::Interval(Duration::from_secs(300)), emit_metrics);
```

**3. Task signature:**

Tasks need access to app state (e.g. DB). Options:

- **`async fn(Arc<Db>) -> Result<(), Error>`** — Forge injects the same `DatabaseConnection` (or shared state) used by the app. Simple and consistent with handlers.
- **`async fn(CronContext) -> Result<(), Error>`** where `CronContext { db, log, ... }` — Extensible later (e.g. job ID, cancel token).

**4. Execution:**

- When `serve()` starts, spawn a background task per registered cron.
- Each task loop: compute next run time from `CronSchedule` (using `chrono::Utc::now()` and the schedule), `tokio::time::sleep_until` (or `sleep` for interval), then run the async closure. Catch panics and log; continue the loop.
- No persistence required for “run at time X” — just in-process timers. Missed runs (e.g. server down) are not replayed; cron is best-effort.

**5. Optional: run once on startup**

- Some tasks (“warmup”, “bootstrap”) only need to run once when the process starts. Could be `app.with_cron_once("bootstrap", bootstrap_fn)` or a separate hook. Defer if not needed for v1.

### Summary

- **Minimal cron without a library**: Yes — use `CronSchedule` (Interval / Hourly / Daily), `chrono` for next-run calculation, and `tokio::time::sleep` / `sleep_until` in a loop. No `cron` crate required for v1.
- **API**: `App::with_cron(name, schedule, async_fn)` where `async_fn` receives shared state (e.g. `DatabaseConnection`). Cron tasks run in-process alongside the server.

---

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

**Register a cron task (no library beyond tokio + chrono):**

```rust
use forge::cron::CronSchedule;
use std::time::Duration;

async fn cleanup_expired_sessions(db: DatabaseConnection) -> Result<(), Error> {
    // ...
    Ok(())
}

app
    .with_cron("cleanup_sessions", CronSchedule::Daily { hour: 3, minute: 0 }, cleanup_expired_sessions)
    .with_cron("heartbeat", CronSchedule::Interval(Duration::from_secs(60)), heartbeat);
```

## Dependencies

- **apalis**: Job trait, worker, middleware.
- **apalis-sqlite**: SQLite storage backend (version aligned with apalis; may be 1.0.0-rc.x until stable).
- **Cron**: No extra crate. Uses **tokio** (interval, sleep_until) and **chrono** (next run time for Hourly/Daily); schedule is an enum (Interval / Hourly / Daily), not cron expressions.

## Success Criteria

1. Apps can define jobs and dispatch them from handlers.
2. Jobs are persisted in SQLite and processed by a worker.
3. Worker can run in the same process or as a separate CLI command.
4. No extra infrastructure (Redis, etc.) required for the default setup.
5. **Cron**: Apps can register cron tasks with `CronSchedule::Interval` / `Hourly` / `Daily`; tasks run in-process on schedule without a cron library.

## Future Extensions

- PostgreSQL/Redis backends for scaling.
- **Cron**: Full cron expressions (e.g. `0 */2 * * *`) via optional `cron` crate; or persist cron run history in DB.
- Job retry and dead-letter configuration.
- Dashboard (e.g. apalis-board) as optional.
