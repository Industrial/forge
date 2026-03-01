# 019_deploy: Deploy with Shuttle + Turso

**Scope:** The minimal path to get a Forge app deployed: **Shuttle** (hosting) + **Turso** (database). One-command deploy, one app (web + worker in-process), configurable so local dev uses file SQLite and production uses Turso.

## Overview

To get a Forge app running in the cloud with the least setup, we use:

- **Shuttle** — Rust-first hosting; no Dockerfile; deploy with `cargo shuttle deploy`.
- **Turso** — Hosted SQLite-compatible database (libSQL). Shuttle’s container disk is ephemeral, so file-based SQLite does not persist; Turso is the way to keep SQLite semantics on Shuttle.

The app is configurable via config files and environment variables so that **locally** it uses a **local SQLite file** and **on Shuttle** it uses **Turso**. Same codebase; backend chosen by configuration.

One Shuttle service runs the whole app: HTTP server and task worker in the same process (in-process worker). No second Shuttle app.

## Goals

- **One deployment path**: Shuttle + Turso only for this feature.
- **One-command deploy**: After one-time setup (Shuttle init, Turso DB + token), deploy with `cargo shuttle deploy` (or future `forge deploy`).
- **Configurable database**: Local = file SQLite (`config/db.toml`). Shuttle = Turso, configured via config and/or env (e.g. `TURSO_DATABASE_URL`, `TURSO_AUTH_TOKEN`).
- **Single app**: One Shuttle service = web + in-process worker. Workers behave the same locally and in the cloud.

## Shuttle + Turso

| What | How |
|------|-----|
| **Hosting** | Shuttle. Install: `cargo install cargo-shuttle`. Init: `shuttle init`. Deploy: `cargo shuttle deploy`. |
| **Database** | Turso. Create a database and token at [turso.tech](https://turso.tech). Forge connects via config/env (addr + token). |
| **Process model** | One process: HTTP server + task worker loop (in-process). No separate worker service. |

## Database: local vs Turso (design)

- **Local development**: `config/db.toml` with `url = "sqlite://db.sqlite?mode=rwc"`. File-based SQLite, current Forge behavior.
- **Shuttle (production)**: Turso. Configuration via:
  - **Config**: Optional `[database.turso]` (or equivalent) in `config/db.toml` with `addr` and `token`, and/or
  - **Env**: `TURSO_DATABASE_URL`, `TURSO_AUTH_TOKEN` (or Shuttle secrets).

When Turso is configured (or when the URL indicates libSQL/Turso), Forge will connect to Turso instead of a local file. Implementation follows this design in a later phase.

## Single app (web + worker)

- One Shuttle service runs the Forge binary: it serves HTTP and runs the task worker in the same process (see 011_background_jobs).
- No second Shuttle app. Same behavior as local: one process, same DB, same task queue.

## Success criteria

1. This doc defines the minimal deploy path: Shuttle + Turso only.
2. When implemented: user can deploy a Forge app to Shuttle with Turso and have it run (web + worker) with one command after initial Shuttle and Turso setup.
3. When implemented: app is configurable so local dev uses file SQLite and Shuttle uses Turso (config + env).
