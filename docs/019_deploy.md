# 019_deploy: Deployment — Get Running Fast

**Scope:** Document deployment platforms that support Rust/Axum/Forge apps, compare them, and define a path to minimal-friction deploy so users go from zero to running in as little time as possible. **Default target: Shuttle** (Rust-first, one-command deploy). Database: configurable local SQLite or hosted Turso via config + env. Single app: web + workers in one process.

## Overview

Forge apps are standard Rust binaries: Axum HTTP server, SeaORM (SQLite by default), config in `config/*.toml`, migrations, and optionally background workers. Any platform that can build and run a Rust binary (or a container that runs it) can host a Forge app. This doc lists and compares those platforms, names **Shuttle as the default** deployment target, and specifies how we will support **local SQLite vs Turso** (config + env) and a **single Shuttle app** (web + workers in one process) for one-command deploy.

## Goals

- **Default platform**: Shuttle — Rust-first, one-command deploy (`cargo shuttle deploy`; future `forge deploy`).
- **Database**: Configurable locally and in the cloud — **local SQLite** (file) or **hosted Turso** (libSQL) via a mix of configuration files and environment variables (design in this doc; implementation later).
- **Single app**: One Shuttle service runs both the web server and the task worker (in-process); no multiple Shuttle apps required.
- **Catalog**: List other deployment platforms and compare tradeoffs.
- **Minimize time to running**: One-command deploy after minimal setup.

## Forge App Requirements (Deployment View)

| Requirement | Default / note |
|-------------|----------------|
| **Runtime** | Single Linux binary (release build) or container running it. |
| **Config** | `config/app.toml`, `config/db.toml`; can be overridden via env (e.g. `FIGMENT_PROFILE`, `DATABASE_URL`). |
| **Database** | SQLite file by default; production often uses Postgres (SeaORM supports both). Platform must provide writable storage for SQLite or a Postgres URL. |
| **Migrations** | Must run before or at startup (e.g. `forge migrate` or app-run migration). |
| **Workers** | Optional; same process (in-process worker) or separate process (`forge tasks:work`). Platform may need to run two processes or support background workers. |
| **Port** | Server binds to `PORT` or config; platform assigns port (e.g. `PORT=8080`). |
| **Secrets** | Env vars for secrets; no hardcoded credentials. |

Platforms that support “run a container” or “run a binary” plus env and (optionally) persistent volume or managed DB can host Forge.

---

## Deployment Platforms Supporting Rust / Axum / Forge

The following platforms can run Rust/Axum applications. Forge is framework-agnostic from the platform’s perspective: they see a Rust app (and optionally a Dockerfile).

### 1. Shuttle

| Aspect | Detail |
|--------|--------|
| **Rust support** | Purpose-built for Rust; first-class. |
| **Build** | `cargo shuttle deploy` — builds in Shuttle’s infra, no Dockerfile required. |
| **Deploy** | One command after `cargo install cargo-shuttle` and `shuttle init`. |
| **Database** | Managed Postgres (and others) via Shuttle resources; SQLite possible with persistent volume. |
| **Workers** | Supported (separate services or same process depending on setup). |
| **Regions** | Single region (expandable). |
| **CLI** | `cargo shuttle deploy`; `shuttle init` for config. |
| **Forge fit** | Excellent for “just deploy”; may need adapter for Forge’s config/migrations if Shuttle expects specific entry points. |

### 2. Railway

| Aspect | Detail |
| **Rust support** | Native; Axum guide and Nixpacks/buildpacks. |
| **Build** | Detects Rust/Cargo; can use Dockerfile. |
| **Deploy** | `railway init` then `railway up` or `railway deploy`; or Git push. |
| **Database** | Managed Postgres, MySQL, Redis, etc.; also bring-your-own. |
| **Workers** | Separate services for workers; can run `forge serve` and `forge tasks:work` as two services. |
| **Regions** | Multi-region. |
| **CLI** | `railway login`, `railway init`, `railway deploy`. |
| **Forge fit** | Very good; add Postgres (or SQLite + volume), run migrations in release command, then start server. |

### 3. Fly.io

| Aspect | Detail |
| **Rust support** | Via Dockerfile; Cargo Chef pattern recommended for small images. |
| **Build** | Dockerfile (multi-stage: build binary, copy into slim image). |
| **Deploy** | `fly launch` (creates `fly.toml`), then `fly deploy`. |
| **Database** | Fly Postgres, or external; SQLite on persistent volume (`fly volumes`). |
| **Workers** | Multiple processes via `[processes]` in `fly.toml` (e.g. `app` + `worker`). |
| **Regions** | 35+ regions; global by default. |
| **CLI** | `fly launch`, `fly deploy`, `fly secrets set`. |
| **Forge fit** | Strong; provide Dockerfile + `fly.toml`; run migrations in release command; use volume for SQLite or attach Fly Postgres. |

### 4. Render

| Aspect | Detail |
| **Rust support** | Docker or native buildpack; Rust supported. |
| **Build** | Dockerfile or Render buildpack; Blueprint YAML for IaC. |
| **Deploy** | Git push or `render deploy`; web UI. |
| **Database** | Managed Postgres; persistent disks for file storage (SQLite possible). |
| **Workers** | Background workers as separate services. |
| **Regions** | Several regions. |
| **CLI** | `render deploy` (CLI exists but Git-based flow common). |
| **Forge fit** | Good; Dockerfile or buildpack; add DB and worker services in Blueprint. |

### 5. DigitalOcean App Platform

| Aspect | Detail |
| **Rust support** | Paketo Rust buildpack; detects `Cargo.toml` / `Cargo.lock`. |
| **Build** | Buildpack or Dockerfile. |
| **Deploy** | Git push or Do CLI; app spec YAML. |
| **Database** | Managed DBs (Postgres, etc.); no built-in SQLite persistence (use Postgres for production). |
| **Workers** | Worker components in app spec. |
| **Regions** | Multiple. |
| **CLI** | `doctl` for apps. |
| **Forge fit** | Good; use Postgres in production; run migrations in build/run. |

### 6. LaunchFlow

| Aspect | Detail |
| **Rust support** | Axum guide; deploys to AWS ECS Fargate. |
| **Build** | Docker; infra as code. |
| **Deploy** | LaunchFlow config + deploy pipeline. |
| **Database** | AWS RDS or other; no SQLite focus. |
| **Workers** | ECS tasks (separate task definitions). |
| **Regions** | AWS regions. |
| **CLI** | LaunchFlow CLI / dashboard. |
| **Forge fit** | Good for AWS-centric teams; more infra to configure. |

### 7. Docker + self‑hosted / VPS / Kubernetes

| Aspect | Detail |
| **Rust support** | Full control; any Rust Dockerfile. |
| **Build** | Multi-stage Dockerfile (builder + runtime). |
| **Deploy** | `docker build` / `docker run`; or K8s manifests; or systemd on VPS. |
| **Database** | Your choice: SQLite on volume, Postgres, etc. |
| **Workers** | Same or separate containers/processes. |
| **Regions** | Any. |
| **CLI** | `docker`, `kubectl`, or SSH + systemd. |
| **Forge fit** | Maximum flexibility; more ops work; good for “we already have Docker/K8s.” |

### 8. Other options (brief)

- **Vercel / Netlify (serverless)** — Rust serverless functions exist but are a different model (per-request); not ideal for long-lived Axum + SQLite/Postgres + workers. Omit for “standard Forge app” unless we add a serverless adapter later.
- **AWS Lambda / Google Cloud Run (serverless)** — Can run containers; often stateless. Forge can run as a single container (e.g. Cloud Run) with external DB; workers need separate scheduling (e.g. Lambda cron or Cloud Run job).
- **Cloud Run (container)** — Good fit: Dockerfile, one service (and optionally a second for workers), managed Postgres or SQLite on volume (if supported).

---

## Comparison Summary

| Platform      | One-command feel | Rust-native | DB (Forge-friendly)     | Workers | Best for                    |
|---------------|------------------|-------------|--------------------------|--------|-----------------------------|
| **Shuttle**   | ✅ `shuttle deploy` | ✅          | Postgres / volume        | ✅     | Fastest path, Rust-first    |
| **Railway**   | ✅ `railway deploy` | ✅          | Postgres, add-ons        | ✅     | Simple PaaS, multi-service  |
| **Fly.io**    | ✅ `fly deploy`   | Via Docker  | Postgres or volume       | ✅     | Global edge, full control   |
| **Render**    | Git / CLI        | Docker/buildpack | Postgres, disks       | ✅     | IaC, zero-downtime         |
| **DigitalOcean** | Git / CLI     | Buildpack   | Postgres                 | ✅     | DO ecosystem                |
| **LaunchFlow**| Config + deploy  | Docker      | RDS / external           | ✅     | AWS / ECS                   |
| **Docker/VPS**| Manual           | ✅          | Any                      | ✅     | Existing infra, max control |

---

## Automating Deployment: Toward `forge deploy`

### Options

1. **`forge deploy` as a target picker + wrapper**
   - Subcommands or flags: `forge deploy shuttle`, `forge deploy fly`, `forge deploy railway`, etc.
   - Forge CLI invokes the platform’s CLI (`cargo shuttle deploy`, `fly deploy`, `railway deploy`) and ensures preconditions (e.g. `forge migrate` equivalent, env hints).
   - Pros: One entry point; we document and script one flow per platform. Cons: Requires platform CLIs installed; we don’t own their APIs.

2. **First-class integration with one platform**
   - E.g. “Forge + Shuttle” or “Forge + Railway”: `forge deploy` means “deploy to Shuttle” with Shuttle-specific init (e.g. `Shuttle.toml`, resource bindings) generated by `forge new` or `forge deploy --init`.
   - Pros: Best UX for that platform. Cons: Ties us to one vendor; others remain manual or wrapper-only.

3. **Dockerfile + platform config generators**
   - `forge deploy --generate-only` (or `forge generate deploy-fly` / `deploy-render`) writes a production Dockerfile + `fly.toml` or Render YAML so the user runs `fly deploy` or Git push themselves.
   - Pros: No dependency on platform CLIs in Forge; works with any Docker-based platform. Cons: Not literally one command; “generate then deploy.”

4. **Hybrid**
   - Forge ships: (a) generated Dockerfile + sample `fly.toml` / Render YAML in docs or `forge new`; (b) optional `forge deploy <target>` that runs the right CLI when present and does pre-deploy checks (migrate, env).

### Recommended direction (for 019)

- **Short term:** Document and maintain a **reference Dockerfile** (multi-stage, release build, minimal runtime) and one **reference `fly.toml`** (and optionally Render/DigitalOcean snippets) in the repo or docs. Ensure `forge new` or a follow-up doc tells users: set `PORT`, `DATABASE_URL` (or SQLite path), run migrations, then start the binary. That gets “running” time down without new CLI code.
- **Next step:** Add **`forge deploy`** as a thin wrapper: `forge deploy fly` → check `fly` CLI, run `forge migrate` (or equivalent), then `fly deploy`; same idea for `railway` / `shuttle` if we want. No need to implement all platforms at once; start with Fly.io and optionally Railway/Shuttle.
- **Later:** If we partner with one platform (e.g. Shuttle), add `forge deploy` (no arg) as that platform’s one-command flow and keep `forge deploy <target>` for others.

### Success criteria (for this doc and follow-up work)

1. **019_deploy** doc exists and lists the platforms above with the comparison table.
2. Users can get a Forge app running on at least one platform (e.g. Fly.io or Railway) using the reference Dockerfile + instructions in this doc (and 001–018 as needed).
3. A path to `forge deploy [target]` is clearly described (wrapper around platform CLIs + optional generators); implementation can be phased.

---

## Reference: Minimal Dockerfile (Forge app)

For platforms that use Docker (Fly.io, Render, Cloud Run, self-hosted):

```dockerfile
# Build stage
FROM rust:1.83-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
# Or for a single-crate app: COPY src ./src
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/myapp /app/myapp
COPY config ./config
COPY migrations ./migrations
ENV PORT=8080
EXPOSE 8080
# Run migrations then server (or use entrypoint script)
CMD ["./myapp", "serve"]
```

For a **Forge-generated app** (single binary at project root), adjust paths: e.g. `COPY . .` in builder and copy `target/release/<binary_name>` and `config/`, `migrations/` into the runtime image. Use `PORT` from env so the platform can assign the port.

---

## Reference: Fly.io (fly.toml)

After `fly launch` (and optional `fly postgres create`), a minimal `fly.toml` for a Forge app:

```toml
app = "my-forge-app"

[build]

[env]
  PORT = "8080"

[http_service]
  internal_port = 8080
  force_https = true
  auto_stop_machines = true
  auto_start_machines = true
  min_machines_running = 0
  processes = ["app"]

[[services]]
  process_group = "app"
  http_checks = [ { path = "/healthz", interval = "10s" } ]
```

If using a worker: add a `[processes]` section and a second process that runs `forge tasks:work` (or the app binary with a `tasks:work` subcommand).

---

## E2E / Validation (future)

- Add an optional e2e or smoke test that builds the reference Dockerfile and runs the app in a container (e.g. `docker run -e PORT=8080 ...`), then hits `/healthz` (see 008_health). No platform account required.
- Manual or CI check: deploy to Fly.io (or Railway) from a test Forge app and verify `/healthz` and `/readyz` return 200.
