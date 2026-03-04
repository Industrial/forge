# {{PROJECT_NAME}}

Rust full-stack app built with [Forge](https://github.com/your-org/forge).

## Development

```bash
forge dev
```

Starts the backend (port 4000) and the Vite dev server (port 3000). **Open http://localhost:3000** — Vite is the single entry point and proxies `/api` and page routes to the backend. Ports and backend URL come from `config/app.toml` ([server], [frontend]). If you run `bun run dev` in `frontend/` alone, set `VITE_BACKEND_URL` to your backend URL.

## Production

```bash
forge serve
```

Builds the frontend (`bun run build`), then runs the backend serving static assets from `frontend/dist`. Use for production or staging.

## Commands

- `forge dev` — development: backend + Vite dev server; open http://localhost:3000
- `forge serve` — production: build frontend, then run backend serving static files
- `cargo run -p server` — backend only (port 4000). Login and register return a Bearer token in the response; the frontend stores it and sends `Authorization: Bearer <token>` on API requests. No cookies are used for auth.
- `bun run dev` (in `frontend/`) — Vite only (port 3000); ensure backend is running for API/pages

## Seed users

After running migrations and seeds, these users exist for development.

**Default org** (domain `@default.org`):

| Email | Password | Global admin | Role | Purpose |
|-------|----------|--------------|------|--------|
| admin@admin.com | password | ✓ | owner | App-wide admin; owner in Default org |
| owner@default.org | password | — | owner | Org owner only (no global admin) |
| orgadmin@default.org | password | — | admin | Org-scoped admin (members, settings) |
| editor@default.org | password | — | editor | Create/edit content in Default |
| viewer@default.org | password | — | viewer | Read-only in Default |

**Other org** (domain `@other.org`):

| Email | Password | Global admin | Role | Purpose |
|-------|----------|--------------|------|--------|
| owner@other.org | password | — | owner | Org owner for Other |
| orgadmin@other.org | password | — | admin | Org-scoped admin for Other |
| viewer@other.org | password | — | viewer | Read-only in Other |

**Multi-org** (tests org switching):

| Email | Password | Global admin | Roles | Purpose |
|-------|----------|--------------|-------|--------|
| multi@email.com | password | — | viewer in Default, editor in Other | Switch orgs to see different roles |

Two organizations are seeded: **Default** and **Other**. Use `multi@email.com` to test switching orgs and different roles per org.

## Frontend: forms and Effect

The frontend uses **React Hook Form (RHF)** with **Effect** and **Effect Schema** for validation and side effects.

- **Forms**: Use `useForm` with `effectSchemaResolver(schema)` from `src/lib/effectSchemaResolver.ts`. Define schemas in `src/schemas/` (shared fragments in `fragments.ts`, form schemas in `userFormSchemas.ts`). Wire fields with `Controller` and pass `form.handleSubmit(handler)` to submit; the handler receives validated, typed data.
- **Side effects / API**: Use `runPromise(effect)` from `src/lib/runEffect.ts` to run Effect programs (e.g. `fetchUsersEffect` in `src/effects/users.ts`) from event handlers or `useEffect`, and handle success/error with React state.

## Integration tests

Backend integration tests live in `crates/app/tests/`. They use a temp config and shared in-memory SQLite, with migrations and seed run. Run them with **single-threaded** execution so the test harness (which sets CWD) does not conflict:

```bash
cargo test -p app --manifest-path Cargo.toml -- --test-threads=1
```

From the template root (`crates/forge-cli/templates/default` when developing the template, or your project root after `forge new`).
