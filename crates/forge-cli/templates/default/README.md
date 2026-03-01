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
- `cargo run -p app` — backend only (port 4000); set `FORGE_ENVIRONMENT=development` or `production` as needed
- `bun run dev` (in `frontend/`) — Vite only (port 3000); ensure backend is running for API/pages
