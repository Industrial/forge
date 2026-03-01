# 018 Implementation Plan: Inertia + Vite Frontend Demo

## Goal

Deliver a **complete hello-world-style SPA** in the Forge scaffold that showcases **i18n**, **auth**, **websockets**, and other Forge features, with **e2e tests** that verify the full flow.

Existing doc: `docs/018_inertia_rendering.md` (Inertia.js + axum-inertia + Vite). Status: documentation only.

## Scope

1. **Forge integration** – Allow apps to use Inertia without hand-rolling router assembly: optional `InertiaConfig` in app state (e.g. `App::with_inertia(...)` or documented custom state type).
2. **Scaffold** – `forge new` generates a Vite + Inertia frontend (React or Vue), `axum-inertia` in the app crate, combined state (DbConnection + InertiaConfig), and a minimal build pipeline (npm/pnpm in crate dir).
3. **Demo SPA** – Pages that demonstrate:
   - **Home** – i18n (locale from Accept-Language or switcher), greeting.
   - **Auth** – Login / Register forms, logout; session-based auth (existing Forge auth).
   - **WebSocket** – Page that opens `/ws`, sends/receives messages (echo or chat).
   - **Dashboard** (optional) – Authenticated page showing user + maybe trace id (observability).
4. **E2E** – Test 018: build frontend in prebuilt project, start server, verify:
   - GET `/` returns HTML shell with root div and script pointing at app bundle.
   - i18n: Accept-Language yields correct locale in shell or first payload.
   - Auth: register → login → access protected page (or API).
   - WebSocket: connect to `/ws`, round-trip message.

## Phases

| Phase | Task | Outcome |
|-------|------|---------|
| 1 | 018.1 Forge API for optional Inertia state | Forge supports extended state (e.g. `with_inertia`) or clear pattern for custom state so app can use axum-inertia. |
| 2 | 018.2 Scaffold Vite+Inertia | New project has `frontend/` (or `crates/app/frontend/`), package.json, vite.config, axum-inertia dep, combined state, one Inertia route. |
| 3 | 018.3 Demo SPA pages | Home (i18n), Auth (login/register), WebSocket page, optional dashboard. |
| 4 | 018.4 E2E 018 | `bin/test-e2e` runs npm install + build for prebuilt, then e2e test 018 verifies HTML, i18n, auth, websocket. |

## Technical Notes

- **State**: Forge currently does `router.with_state(db_conn)`. We need either generic state (breaking) or a dedicated path: e.g. `App::with_inertia(InertiaConfig)` so Forge builds `(DbConnection, InertiaConfig)` (or a struct) and implements `FromRef` for both. Doc already describes `AppState { db, inertia }` + `FromRef`; we can generate that in the scaffold and keep Forge only injecting `DbConnection`, with scaffold providing the combined state and custom router assembly, **or** add `with_inertia` so Forge merges Inertia into state.
- **E2E build**: `bin/test-e2e` currently does `forge new` → `cargo build` → overwrite config → `cargo run`. We must add a frontend build step (e.g. `npm ci && npm run build`) in the prebuilt app before or after `cargo build`, and ensure production Inertia config serves from built assets (e.g. `dist/`).
- **Vite in scaffold**: Use Vite in production mode for e2e (no dev server): build to `dist/`, serve via `tower_http::ServeDir`. In dev, user runs Vite dev server separately (doc already describes this).

## Beads

- Epic: **forge-i38** – 018: Inertia+Vite frontend demo and e2e
- forge-i38.1 – Forge API for optional Inertia state
- forge-i38.2 – Scaffold Vite+Inertia
- forge-i38.3 – Demo SPA pages (Home, Auth, WS, dashboard)
- forge-i38.4 – E2E test 018
