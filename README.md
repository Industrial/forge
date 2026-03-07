# Forge

[![CI](https://github.com/Industrial/forge/actions/workflows/ci.yml/badge.svg)](https://github.com/Industrial/forge/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/forge.svg)](https://crates.io/crates/forge)
[![docs.rs](https://img.shields.io/docsrs/forge)](https://docs.rs/forge)
[![License: CC BY-SA 4.0](https://img.shields.io/badge/License-CC%20BY--SA%204.0-green.svg)](https://creativecommons.org/licenses/by-sa/4.0/)

A full-stack web framework for Rust: convention over configuration, batteries-included, one CLI from zero to shipped. The story is simple: **Rails or Django ergonomics, with Rust’s performance and type safety, and no Node/npm in the critical path.**

## How it compares

**Next.js** gives you a single language (JS/TS) and a huge ecosystem, but you still choose auth, DB, jobs, and real-time piece by piece. The “full-stack” is often a thin API layer over serverless or a separate backend. Forge is the opposite: one stack, one process, one config story—auth, DB, jobs, WebSockets, and live updates are built in and wired by convention.

**Rails and Django** are the spiritual model: sensible defaults, generators, migrations, and “it just works” for CRUD and dashboards. Forge aims for that feel in Rust. You don’t get Ruby or Python’s dynamism or their maturity of gems/packages; you get a single, coherent stack, strong typing, and no GIL—so the trade is clarity and performance for a smaller plugin ecosystem and a younger project.

**Other Rust web stacks** (raw Axum, Actix, etc.) are powerful but leave you to assemble auth, sessions, rate limiting, background jobs, and real-time yourself. Forge sits on top of Axum and Tokio and gives you those layers out of the box, so you spend time on product logic instead of glue.

Honest gaps: Forge is **early**. Not every edge is polished; the API may still evolve. If you want a stable, “boring” framework, Rails or Django are safer today. If you want one coherent Rust backend with real-time and live data sync, and you’re okay helping shape it, Forge is built for that.

## What you get

- **One CLI**: `forge new myapp`, `forge dev` / `forge serve`, migrations, generators. Strict project layout (`config/`, migrations, routes) so the tooling knows where everything lives.
- **Config as single source of truth**: No CLI flags for port or env—everything comes from `config/app.toml` (and related files). Figment-based layering for env-specific overrides.
- **Database**: SeaORM + SQLite by default. Connection and pool config in `config/db.toml`. Migrations and seeds are first-class; `forge new` generates the schema you need for users, orgs, and memberships.
- **Auth and authorization**: Sessions and login; password hashing (Argon2id); API tokens (stored as hashes). Authorization is “shallow gate + deep scope”: handler-level guards (`Action`, `Role`) and DB-level scoping so tenant data is isolated and wrong-tenant reads look like 404 (Ghost Mode). Roles are per-organization, not global.
- **Audit logging**: First-class events and outcomes so you can record who did what, when.
- **Health and observability**: Health/live/ready endpoints; optional OpenTelemetry and tracing so you can plug into existing observability stacks.
- **Rate limiting**: Governor-based, keyed by requester/org so you can throttle per user or per tenant.
- **Validation**: Shared validation types and helpers so request and domain rules stay consistent.
- **Background jobs**: Apalis-based task queue with SQLite storage by default—no Redis required. Define tasks, dispatch from handlers, run workers in-process or as a separate process. Recurring work is “scheduled tasks” enqueued on a timer.
- **API token auth**: Bearer tokens for programmatic access; tokens are hashed and checked against the DB. Scope (org, role) can come from headers or token metadata.
- **Security**: Security headers (CSP, HSTS, etc.) applied by default so responses are hardened out of the box.
- **Real-time**: WebSockets and SSE via Axum. Raw `ws` and `sse` for custom endpoints. On top of that, a **Live Query** system: clients subscribe to scoped channels (e.g. per-org, per-resource-type); the server derives subscriptions from the session and permissions, so the client doesn’t send a channel list. When data changes, handlers call a broadcast API and every subscribed connection gets the update. Single process uses in-memory pub/sub; multi-instance can use a swappable backend (e.g. Redis) so all instances see the same events. Result: UIs stay in sync without polling or hand-rolled WebSocket routing.
- **Caching**: Application cache (key-value get/set/delete) and optional HTTP response cache (middleware). Backed by Moka in-process; manual invalidation so you control when entries are busted.
- **i18n**: Hooks for internationalization so you can drive locale and translations from config and request context.
- **Frontend**: Straight-up Vite SPA (React, Vue, or Svelte). `forge dev` runs the backend and Vite dev server with HMR; production builds the frontend and serves static assets.
- **Deploy**: Documented path with Shuttle (Rust hosting, no Dockerfile) and Turso (hosted SQLite-compatible DB). One service runs HTTP + in-process worker; local dev uses file SQLite, production uses Turso via config/env.

Rust all the way on the backend: Axum, Tokio, Tower. The default template includes a Vite SPA; no JS build step if you stick to API-only.

## Status: early and growing

Forge is a **new project**. We’re building in the open: not every edge is polished, and the API may evolve. If you like the direction and want to shape it, this is the right time—issues, docs, and code are welcome.

## Quick start

Install the CLI (no Rust toolchain required), then create and run an app.

**One-liner install (recommended)**

```bash
curl -fsSL https://raw.githubusercontent.com/Industrial/forge/main/install.sh | sh

forge new myapp
cd myapp
forge dev
```

Then open http://localhost:3000. For production: build frontend and run `forge serve`.

*Once the forge.sh domain is configured, use: `curl -fsSL https://forge.sh/install | sh`*

**Other install options**

- **Pin version:** `FORGE_VERSION=v1.2.3 curl -fsSL .../install.sh | sh`
- **With Rust:** `cargo install forge-cli` (when on crates.io) or `cargo install --git https://github.com/Industrial/forge forge-cli --bin forge`
- **From clone:** `git clone https://github.com/Industrial/forge.git && cd forge && cargo run -p forge-cli -- new myapp`

## Documentation (in-repo)

Details live in the `docs/` folder: CLI and project layout, config, database and SeaORM, migrations, authentication, authorization, audit logging, health and observability, rate limiting, validation, background jobs, API token auth, security, WebSockets and real-time, i18n, caching, and deploy (Shuttle + Turso). Live Query design (channels, permissions, broadcast, swappable backend) is in the multi-crate and live-channels docs.

## Contributing

We’re **open source** and **community-first**. Contributions are welcome: code, docs, issues, and ideas. Check open issues, comment on design discussions, or open a PR. Be respectful and constructive; we’ll do the same.

1. Fork the repo, create a branch, make your changes.
2. Run tests and linters (see [Development](#development) below).
3. Open a PR with a clear description and link any related issues.

By contributing, you agree that your contributions will be licensed under the same license as the project (see [License](#license)).

## Contributors

Thanks to everyone who has contributed to Forge:

[![Contributors](https://contrib.rocks/image?repo=Industrial/forge)](https://github.com/Industrial/forge/graphs/contributors)

*(Image generated by [contrib.rocks](https://contrib.rocks).)*

[![Star History Chart](https://api.star-history.com/svg?repos=Industrial/forge&type=Date)](https://star-history.com/#Industrial/forge)

## Development

- **Rust**: 2024 edition, format with `cargo fmt`, lint with `cargo clippy`.
- **Nix / devenv**: Use `devenv shell` for the intended environment; run commands inside it (e.g. `devenv shell -- cargo test`).
- **Quality**: Tests (including e2e), `cargo-deny` for audits, and CI on every push.

See the repo root and `.cursor/rules` for formatting, testing, and workflow details.

## License

This project is licensed under the **Creative Commons Attribution-ShareAlike 4.0 International (CC BY-SA 4.0)**. You may share and adapt the material for any purpose, including commercially, as long as you give appropriate credit and distribute your contributions under the same license. See [LICENSE](LICENSE) and [Creative Commons BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/) for the full text.

The Rust crates in this repository also offer dual licensing under **MIT OR Apache-2.0** where noted in their `Cargo.toml`; for maximum permissibility in dependency use, you may use the code under those terms when applicable.

## To Do

### Tier 1 — Immediate DX Multipliers (Highest Impact)

1. **Zero-Friction Install (Binary Releases + One-Liner)**  
   Publish versioned binaries for macOS/Linux/Windows and support:
   ```bash
   curl -fsSL https://forge.sh/install | sh
   ```
   `cargo install --git` adds friction. A first impression should feel like Rails or Bun — instant.  
   **Impact:** Removes Rust toolchain friction from evaluators.

2. **“Golden Path” 5-Minute Tutorial**  
   A guided, opinionated walkthrough that:
   - Creates app
   - Adds model
   - Adds background job
   - Adds live update
   - Deploys  
   Make it impossible to get lost.  
   **Impact:** Reduces abandonment during evaluation.

3. **Interactive `forge doctor`**  
   Diagnostics command for:
   - Rust version
   - DB connectivity
   - Config validation
   - Missing migrations
   - Port conflicts
   - Env sanity  
   **Impact:** Converts frustration into actionable fixes.

4. **`forge check` (Static Project Validator)**  
   Pre-runtime validation:
   - Routes registered?
   - Guards mismatched?
   - Missing policies?
   - Unused migrations?
   - Broken live channels?  
   Like `cargo check`, but Forge-aware.

5. **Error Pages That Teach**  
   Instead of a bare 500, show e.g.:
   - *Missing org scope.* You called `require_role(Admin)` but no org context exists.
   - Include: what happened, why, a fix example, and a link to the docs section.  
   This is what made Ruby on Rails beloved.

### Tier 2 — Friction Killers

6. **First-Class Type-Safe Forms**  
   Generate: DTO, validation, handler, and frontend form scaffold.  
   Form handling is 40% of CRUD friction.

7. **Declarative Policy DSL**  
   Instead of writing Rust guards manually:
   ```rust
   policy! {
       Post {
           read: Member,
           write: Admin,
       }
   }
   ```
   Generate guards + DB scoping automatically.