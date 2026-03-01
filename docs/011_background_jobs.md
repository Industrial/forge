# 011_background_jobs: Tasks (Apalis)

**Scope:** Full implementation of the Apalis task system. First-class concepts: **Task** (define work), **Dispatch** (enqueue), **Schedule** (when to run). Recurring runs (interval / hourly / daily) are a **schedule** on top of tasks, not a separate system.

## Overview

Forge provides a **task system** built on [Apalis](https://github.com/nicksrandall/apalis) and SQLite: define tasks, dispatch them from handlers or schedule them to run at intervals, and process them with a worker. No extra infrastructure (e.g. Redis) is required by default.

- **Task**: A unit of work. Implemented as a serializable type that implements the task trait; the worker runs it with access to context (e.g. DB).
- **Dispatch**: Enqueue a task so it runs once (e.g. from a request handler). Example: `forge::tasks::dispatch(SendEmailTask { ... }).await`.
- **Schedule**: When a task runs. Either **once** (dispatched “now”) or **recurring** (interval, hourly, daily). Recurring schedules are implemented by a scheduler that enqueues the same task on a timer; schedule is part of the task system, not a separate subsystem.

## Goals

- **Task-first**: The public API speaks in terms of **tasks**; Apalis “jobs” are the implementation detail.
- **Core dependency**: Every Forge app can use tasks; no optional feature flag.
- **SQLite by default**: Task state lives in the app’s SQLite DB (or a dedicated task DB via config).
- **Type-safe**: Tasks are structs implementing a Forge/Apalis trait; dispatch and run are type-checked.
- **Single API**: `forge::tasks::dispatch(task)` and a standard way to run workers (e.g. `forge tasks:work` or in-process).
- **Scheduling**: Recurring runs via a minimal schedule DSL (interval / hourly / daily) without an external schedule library; the scheduler enqueues tasks into the same storage the worker consumes.

## Architecture

### Three pillars: Task, Dispatch, Schedule

| Concept | Meaning |
|--------|---------|
| **Task** | Definition of work: a type (e.g. `SendEmailTask`) that implements the task trait and is serialized into the queue. |
| **Dispatch** | Enqueue a task instance so the worker runs it (e.g. from an HTTP handler). Run “once, now.” |
| **Schedule** | When to run. **Once**: dispatch. **Recurring**: a scheduler process/task that, on a timer (interval/hourly/daily), enqueues a task (e.g. a “scheduled task” type or a named task). The worker treats all enqueued tasks the same. |

Recurring behaviour is thus: **schedule** → scheduler enqueues a task (e.g. `ScheduledTask { name }`) at the right times → worker runs it by name from a registry. It’s tasks + a schedule.

### Components

| Component | Responsibility |
|-----------|----------------|
| **Storage** | SQLite via apalis-sqlite; task payloads are stored in tables. Same DB URL as the app unless `[tasks]` config overrides. |
| **Dispatcher** | API used from app code to enqueue a task: `forge::tasks::dispatch(my_task).await`. |
| **Worker** | Polls storage, deserializes the task, runs the task handler, marks complete/failed. Can run in-process (spawned in `serve()`) or as a separate process (`forge tasks:work`). |
| **Scheduler** | For recurring runs only: in-process loop(s) that, on interval/hourly/daily, enqueue a task (e.g. `ScheduledTask { task_name }`). Worker runs it like any other task. |

### Task definition

Tasks are serializable structs. A handler type implements the Apalis job/task trait (e.g. `async fn run(&self, ctx)`). Forge exposes a thin wrapper so apps implement a Forge task trait and get dispatch + worker registration. Apalis remains the engine; “job” in Apalis maps to “task” in the Forge API.

### Worker execution

- **In-process**: Spawn a worker task in `serve()` so the same process runs HTTP and the task worker.
- **Separate process**: Run `forge tasks:work` (or `cargo run -- tasks:work`) that connects to the same SQLite DB and processes the queue.

Config (e.g. `[tasks]` in `config/app.toml` or `config/db.toml`) can control worker concurrency, poll interval, and optionally a dedicated database URL.

---

## Recurring tasks (schedule on top of tasks)

Recurring runs are **scheduled tasks**: a scheduler enqueues a task on a timer; the task system (storage + worker) is unchanged.

### Schedule types (no external schedule crate)

Using **tokio** and **chrono** (already in Forge):

| Schedule type | Meaning | Implementation |
|---------------|---------|----------------|
| **Interval** | Every `Duration` (e.g. 5 minutes) | `tokio::time::interval`; on tick, enqueue the task. |
| **Hourly** | Every hour at minute `M` (e.g. :00, :30) | `chrono` to compute next `hour:minute`, `sleep_until`, then enqueue. |
| **Daily** | Once per day at `HH:MM` (e.g. 03:00) | Same: next occurrence of that time, then enqueue. |

No full cron-style expressions (e.g. `0 */2 * * *`) in the minimal version; an optional crate can be added later for power users.

### API for recurring tasks

**1. Schedule type:**

```rust
pub enum TaskSchedule {
    /// Every fixed duration (e.g. every 5 minutes).
    Interval(std::time::Duration),
    /// Every hour at the given minute (0..59).
    Hourly { minute: u32 },
    /// Once per day at the given hour and minute (UTC or configurable).
    Daily { hour: u32, minute: u32 },
}
```

**2. Register a recurring task with the app:**

The app registers a **name** and a **schedule**; the scheduler enqueues a single internal task type (e.g. `ScheduledTask { task_name }`) on that schedule. The worker resolves `task_name` to the actual handler (e.g. a closure or registry).

```rust
app
    .with_scheduled_task("cleanup", TaskSchedule::Daily { hour: 3, minute: 0 }, cleanup_expired_sessions)
    .with_scheduled_task("metrics", TaskSchedule::Interval(Duration::from_secs(300)), emit_metrics);
```

**3. Task signature for scheduled handlers:**

Handlers need app state (e.g. DB). Same as doc 011 today: `async fn(DatabaseConnection) -> Result<(), Error>`. Forge injects the app’s `DatabaseConnection` when the worker runs the scheduled task.

**4. Execution:**

- When `serve()` starts, spawn the task worker (if in-process) and one scheduler loop per registered schedule.
- Each scheduler loop: compute next run from `TaskSchedule` (chrono + tokio), `sleep_until`, then enqueue the scheduled task (e.g. `ScheduledTask { task_name: "cleanup" }`). Worker picks it up and runs the registered handler.
- Missed runs (server down) are not replayed; recurring schedule is best-effort.

---

## Configuration

Example in `config/app.toml` or a dedicated `config/tasks.toml`:

```toml
[tasks]
# Optional: default is the app database URL
database_url = "sqlite://db.sqlite?mode=rwc"
# Worker: concurrency, poll interval (in-process or CLI)
worker_concurrency = 2
```

If `database_url` is omitted, use the main app database URL.

## Usage

**Define a task (user-defined, e.g. request-triggered):**

```rust
use forge::tasks::{Task, TaskContext};

#[derive(Clone, Serialize, Deserialize)]
pub struct SendEmailTask {
    pub to: String,
    pub subject: String,
}

#[async_trait]
impl Task for SendEmailTask {
    async fn run(&self, _ctx: TaskContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // send email
        Ok(())
    }
}
```

**Dispatch from a handler (run once, now):**

```rust
forge::tasks::dispatch(SendEmailTask { to, subject }).await?;
```

**Run worker:** `forge tasks:work` (or in-process worker) processes the queue.

**Register a recurring (scheduled) task:**

```rust
use forge::tasks::TaskSchedule;
use std::time::Duration;

async fn cleanup_expired_sessions(db: DatabaseConnection) -> Result<(), Error> {
    // ...
    Ok(())
}

app
    .with_scheduled_task("cleanup_sessions", TaskSchedule::Daily { hour: 3, minute: 0 }, cleanup_expired_sessions)
    .with_scheduled_task("heartbeat", TaskSchedule::Interval(Duration::from_secs(60)), heartbeat);
```

## Dependencies

- **apalis**: Task/job trait, worker, middleware (we expose “task” in the API; Apalis uses “job” internally).
- **apalis-sqlite**: SQLite storage backend for the task queue (version aligned with apalis).
- **Recurring schedule**: No extra crate. **tokio** (interval, sleep_until) and **chrono** (next run for Hourly/Daily); schedule is an enum (Interval / Hourly / Daily).

## Success Criteria

1. Apps define **tasks** (type + trait) and **dispatch** them from handlers.
2. Tasks are persisted in SQLite (Apalis storage) and processed by a worker.
3. Worker runs in-process or via `forge tasks:work`.
4. **Recurring runs**: Apps register scheduled tasks with `TaskSchedule::Interval` / `Hourly` / `Daily`; a scheduler enqueues a task on that schedule; the same worker processes it. Schedule is “on top of tasks.”
5. No extra infrastructure (Redis, etc.) required for the default setup.

## Future Extensions

- PostgreSQL/Redis backends for scaling.
- Full cron-style expressions (e.g. `0 */2 * * *`) via an optional crate.
- Task retry and dead-letter configuration.
- Dashboard (e.g. apalis-board) as optional.
