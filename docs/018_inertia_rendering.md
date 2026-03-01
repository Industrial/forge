# Inertia.js Rendering with axum-inertia and Vite

Forge supports building server-driven single-page apps using **Inertia.js** with **axum-inertia** and **Vite**. Inertia keeps server-side routing and controllers while the frontend is a React, Vue, or Svelte app; **Vite** handles bundling and HMR; **axum-inertia** implements the Inertia protocol on the server.

## Objectives

1. **Server-driven SPA:** One Axum router, one frontend bundle; Inertia handles the protocol (initial HTML shell, subsequent JSON “page” responses).
2. **Single crate:** Use the **axum-inertia** crate for the protocol; no custom Inertia implementation.
3. **Vite for frontend:** Dev server (HMR) in development; built assets in production, served via `tower_http::ServeDir`.
4. **Clear state story:** App state includes both `DatabaseConnection` (Forge) and `InertiaConfig` (axum-inertia) via a combined state type and `FromRef`.

## Roles

| Layer | Responsibility |
|-------|----------------|
| **Inertia.js** | Protocol and client: `X-Inertia` header, “Page” JSON, version/409 reload. |
| **Vite** | Bundling, HMR, and emitting `manifest.json` (and assets) for production. |
| **axum-inertia** | Server implementation: `Inertia` extractor, `InertiaConfig`, and Vite-based HTML/config helpers. |
| **Forge** | Config, DB, auth, routes; app state can be extended with `InertiaConfig`. |

## Dependencies

Add to your application crate:

```toml
[dependencies]
axum-inertia = "0.9"
# and your existing: forge, axum, serde_json, tower_http, etc.
```

## InertiaConfig with Vite (axum-inertia)

axum-inertia’s **vite** module provides builders for development and production.

**Development** (Vite dev server, HMR):

```rust
use axum_inertia::{vite, InertiaConfig};

let inertia = vite::Development::default()
    .port(5173)
    .main("src/main.ts")   // or main.jsx, main.vue, etc.
    .lang("en")
    .title("My app")
    .react()               // or .vue(), .svelte() as needed
    .into_config();
```

**Production** (manifest + built assets):

```rust
use axum_inertia::{vite, InertiaConfig};

let inertia = vite::Production::new(
    "dist/.vite/manifest.json",  // path to Vite manifest
    "src/main.ts",               // entrypoint name matching manifest
)
.unwrap()
.lang("en")
.title("My app")
.into_config();
```

Choose at runtime using your Forge configuration’s **environment** option (`config/app.toml`):

```rust
use forge::config;

let is_production = config::effective_environment().eq_ignore_ascii_case("production");

let inertia = if is_production {
    vite::Production::new("dist/.vite/manifest.json", "src/main.ts")
        .unwrap()
        .lang("en")
        .title("My app")
        .into_config()
} else {
    vite::Development::default()
        .port(5173)
        .main("src/main.ts")
        .lang("en")
        .title("My app")
        .react()
        .into_config()
};
```

The effective environment is set by the CLI: `forge dev` → development, `forge serve` → production (via `FORGE_ENVIRONMENT`).

## App state: combining Forge and Inertia

axum-inertia requires **InertiaConfig** in the router state. Forge injects **DatabaseConnection**. Use a single state type and implement **FromRef** for both so extractors work.

```rust
use axum::extract::FromRef;
use axum_inertia::InertiaConfig;
use sea_orm::DatabaseConnection;

#[derive(Clone)]
struct AppState {
    pub db: DatabaseConnection,
    pub inertia: InertiaConfig,
}

impl FromRef<AppState> for DatabaseConnection {
    fn from_ref(state: &AppState) -> Self {
        state.db.clone()
    }
}

impl FromRef<AppState> for InertiaConfig {
    fn from_ref(state: &AppState) -> Self {
        state.inertia.clone()
    }
}
```

Build the router with `AppState` and use it for all routes that need DB and/or Inertia.

## Handlers: Inertia extractor and render

Use the **Inertia** extractor in handlers that serve Inertia pages:

```rust
use axum_inertia::Inertia;
use axum::response::IntoResponse;
use serde_json::json;

async fn index(i: Inertia) -> impl IntoResponse {
    i.render("Pages/Index", json!({ "title": "Home" }))
}

async fn users_list(i: Inertia, State(db): State<DatabaseConnection>) -> impl IntoResponse {
    let users = fetch_users(&db).await;
    i.render("Pages/Users/Index", json!({ "users": users }))
}
```

- **Initial load (no `X-Inertia`):** axum-inertia responds with the full HTML shell (using your `InertiaConfig`), so the client loads the Vite bundle and boots the Inertia app.
- **Inertia requests (`X-Inertia: true`):** responds with the Inertia “Page” JSON (component name + props).
- **Version mismatch:** responds with 409 Conflict so the client can reload.

## Serving static assets

- **Development:** Run Vite’s dev server (e.g. `npm run dev` or `vite`) on port 5173 (or whatever you pass to `vite::Development`). The initial HTML from axum-inertia points the browser at that server for JS/CSS and HMR.
- **Production:** Build the frontend (`npm run build`), then serve the built output with **tower_http::ServeDir**:

```rust
use tower_http::services::ServeDir;

// Mount built Vite output (e.g. dist/) at /
router.nest_service("/assets", ServeDir::new("dist"))
    // and/or serve at a path that matches your Vite base
```

Ensure Vite’s `base` in `vite.config` matches the path where you mount assets (e.g. `/assets/`).

## Forge integration (current)

Forge’s **App** currently injects only **DatabaseConnection** into the router in `into_router()`. To use Inertia you need router state that includes both **DatabaseConnection** and **InertiaConfig**.

**Option A – Custom router assembly:** Use Forge for config, DB, migrations, and auth, but build the final Axum router yourself so you can call `with_state(AppState { db, inertia })` with your combined state. You’ll reuse the same patterns (health routes, auth layer, rate limiting) but construct the router and state in your own `main` or setup.

**Option B – Future Forge API:** A future release may add something like `App::with_inertia(...)` so Forge merges `InertiaConfig` into the app state and you keep using `into_router()` / `serve()` without custom assembly.

Until then, use the **AppState + FromRef** pattern above and assemble the router with that state where you currently call `into_router()` or equivalent.

## Summary

| Topic | Approach |
|-------|----------|
| **Protocol / server** | **axum-inertia** (Inertia extractor, `InertiaConfig`, `i.render(component, props)`). |
| **Frontend build** | **Vite** (dev server + HMR in dev; build + `manifest.json` in prod). |
| **State** | Combined `AppState` with `db` + `inertia`; `FromRef<AppState>` for `DatabaseConnection` and `InertiaConfig`. |
| **Assets (prod)** | **tower_http::ServeDir** for Vite’s build output. |
| **Forge** | Use Forge for config/DB/auth; provide router state that includes both DB and InertiaConfig (custom assembly or future `with_inertia`). |

## Status

**Forge:** `into_router_before_state()` is available so apps can apply custom state (e.g. `(DbConnection, InertiaConfig)`). Forge does not add axum-inertia as a dependency; your app adds it and uses the pattern above. The scaffold (018.2–018.4) generates a Vite+Inertia frontend with combined state, demo SPA pages (Home, Auth, WebSocket, Dashboard), and E2E tests; use **bun** for the frontend build (`bun install`, `bun run build`).