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
- `cargo run -p app` — backend only (port 4000). Use `FORGE_ENVIRONMENT=development` for local HTTP so the session cookie is not `Secure` (required for login to persist after redirect).
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
