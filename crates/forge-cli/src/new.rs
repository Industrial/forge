//! Create a new Forge project (scaffold).

use std::fs;
use std::path::Path;
use std::process::Command;

pub fn create_new_project(name: &str) -> Result<(), Box<dyn std::error::Error>> {
  let project_dir = Path::new(name);

  // Check if directory already exists
  if project_dir.exists() {
    return Err(format!("Directory '{}' already exists", name).into());
  }

  // Create workspace structure
  fs::create_dir_all(project_dir.join("crates/app/src/handlers"))?;
  fs::create_dir_all(project_dir.join("crates/app/locales/en-US"))?;
  fs::create_dir_all(project_dir.join("crates/app/locales/de"))?;
  fs::create_dir_all(project_dir.join("crates/db/src/migrations"))?;
  fs::create_dir_all(project_dir.join("crates/db/src/seeds"))?;
  fs::create_dir_all(project_dir.join("crates/db/src/models"))?;
  fs::create_dir_all(project_dir.join("config"))?;
  // 018: Inertia + Vite frontend (bun)
  fs::create_dir_all(project_dir.join("frontend/src/pages"))?;
  fs::create_dir_all(project_dir.join("frontend/public"))?;

  // Get the absolute path to the forge crate relative to this executable
  let exe_path = std::env::current_exe().unwrap();
  let forge_crate_path = exe_path
    .parent()
    .unwrap() // target/debug or target/release
    .parent()
    .unwrap() // target
    .parent()
    .unwrap() // forge workspace root
    .join("crates")
    .join("forge");

  // Create Root Cargo.toml (Workspace)
  let root_cargo_toml = r#"[workspace]
members = [
  "crates/app",
  "crates/db",
]
resolver = "2"
"#;
  fs::write(project_dir.join("Cargo.toml"), root_cargo_toml)?;

  // Create crates/app/Cargo.toml
  let app_cargo_toml = format!(
    r#"[package]
name = "app"
version = "0.1.0"
edition = "2024"

[dependencies]
forge = {{ path = "{}" }}
db = {{ path = "../db" }}
tokio = {{ version = "1", features = ["full"] }}
serde = {{ version = "1.0", features = ["derive"] }}
axum = {{ version = "0.8", features = ["ws"] }}
axum-login = "0.17"
sea-orm = {{ version = "1.1", features = ["runtime-tokio-rustls", "sqlx-sqlite", "macros"] }}
chrono = {{ version = "0.4", features = ["serde"] }}
uuid = {{ version = "1.0", features = ["v4", "serde"] }}
validator = {{ version = "0.20", features = ["derive"] }}
fluent = "0.16"
fluent-templates = "0.13"
accept-language = "3.1"
unic-langid = "0.9"
axum-inertia = "0.9"
serde_json = "1.0"
tower-http = {{ version = "0.6", features = ["fs"] }}
tracing = "0.1"
"#,
    forge_crate_path.display()
  );
  fs::write(project_dir.join("crates/app/Cargo.toml"), app_cargo_toml)?;

  // Create crates/db/Cargo.toml
  let db_cargo_toml = format!(
    r#"[package]
name = "db"
version = "0.1.0"
edition = "2024"

[dependencies]
forge = {{ path = "{}" }}
sea-orm = {{ version = "1.1", features = ["runtime-tokio-rustls", "sqlx-sqlite", "macros"] }}
sea-orm-migration = "1.1"
serde = {{ version = "1.0", features = ["derive"] }}
uuid = {{ version = "1.0", features = ["v4", "serde"] }}
chrono = {{ version = "0.4", features = ["serde"] }}
async-trait = "0.1"
tracing = "0.1"
"#,
    forge_crate_path.display()
  );
  fs::write(project_dir.join("crates/db/Cargo.toml"), db_cargo_toml)?;

  // Create .gitignore
  let gitignore = r#"# Rust build artifacts
target/

# IDE files
.vscode/
.idea/
*.swp
*.swo

# OS files
.DS_Store
Thumbs.db

# Environment variables
.env
.env.local

# Logs
*.log

# Database files
*.db
*.sqlite
*.sqlite3

# Frontend (018)
frontend/node_modules/
frontend/dist/
"#;
  fs::write(project_dir.join(".gitignore"), gitignore)?;

  // Create crates/app/src/main.rs (018: Inertia + into_router_before_state)
  let main_rs = r#"use std::time::Duration;

use axum::routing::get;
use axum_inertia::{vite, InertiaConfig};
use db::auth::Backend;
use forge::{App, CronSchedule};
use tower_http::services::ServeDir;

mod handlers;
mod state;

use state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let app = App::new()
    .with_migrations(db::Migrator)
    .with_seed(|db| Box::pin(db::run_seeds(db)))
    .with_auth(|db| Backend::new(db))
    .with_token_auth(db::token_lookup)
    .with_cron("heartbeat", CronSchedule::Interval(Duration::from_secs(60)), |_db| async move { Ok(()) });

  let app = if app.config().app.environment.eq_ignore_ascii_case("production") {
    app
      .with_rate_limit_per_ip(60)
      .with_rate_limit_per_user(60)
  } else {
    app
  };

  let is_production = app.config().app.environment.eq_ignore_ascii_case("production");
  let host = app.config().server.host.clone();
  let port = app.config().server.port;

  let app = app
    .route("/api/cache-demo", handlers::cache_demo::handler)
    .route("/api/cached-page", handlers::cached_page::handler)
    .route("/api/observability/trace-id", handlers::observability::trace_id)
    .route("/ws", handlers::ws::handler)
    .post_route("/api/auth/register", handlers::auth::register)
    .post_route("/api/auth/login", handlers::auth::login)
    .route("/api/auth/logout", handlers::auth::logout)
    .route("/api/auth/profile", handlers::auth::profile)
    .post_route("/api/auth/tokens", handlers::auth::create_token)
    .route("/api/auth/admin", handlers::auth::admin_only);

  let (router, db_conn, cron_runner, response_cache) = app.into_router_before_state().await;

  let inertia: InertiaConfig = if is_production {
    vite::Production::new("frontend/dist/.vite/manifest.json", "src/main.tsx")
      .map_err(|e| format!("Inertia production config: {}", e))?
      .lang("en")
      .title("App")
      .into_config()
  } else {
    vite::Development::default()
      .port(5173)
      .main("src/main.tsx")
      .lang("en")
      .title("App")
      .react()
      .into_config()
  };

  let api_router = router.with_state(db_conn.clone());
  let inertia_router = axum::Router::new()
    .route("/", get(handlers::inertia::home))
    .route("/login", get(handlers::inertia::login_page))
    .route("/register", get(handlers::inertia::register_page))
    .route("/dashboard", get(handlers::inertia::dashboard))
    .route("/ws-demo", get(handlers::inertia::ws_demo_page))
    .with_state(AppState {
      db: db_conn,
      inertia,
    });
  let mut router = api_router.merge(inertia_router);

  if let Some(cache_layer) = response_cache {
    router = router.layer(cache_layer);
  }

  if is_production {
    router = router.nest_service("/assets", ServeDir::new("frontend/dist"));
  }

  if let Some((db, runner)) = cron_runner {
    runner.spawn(db);
  }

  let addr = format!("{}:{}", host, port);
  let listener = tokio::net::TcpListener::bind(&addr).await?;
  tracing::info!("Forge server running on http://{}", addr);
  axum::serve(listener, router.into_make_service_with_connect_info::<std::net::SocketAddr>())
    .with_graceful_shutdown(forge::app::shutdown_signal_future())
    .await?;
  Ok(())
}
"#;
  fs::write(project_dir.join("crates/app/src/main.rs"), main_rs)?;

  // Create crates/app/src/state.rs (018: shared state for Inertia with FromRef)
  let state_rs = r#"use axum::extract::FromRef;
use axum_inertia::InertiaConfig;
use forge::DbConnection;

/// Combined state for Inertia routes so both DbConnection and InertiaConfig can be extracted.
#[derive(Clone)]
pub struct AppState {
  pub db: DbConnection,
  pub inertia: InertiaConfig,
}

impl FromRef<AppState> for DbConnection {
  fn from_ref(input: &AppState) -> DbConnection {
    input.db.clone()
  }
}

impl FromRef<AppState> for InertiaConfig {
  fn from_ref(input: &AppState) -> InertiaConfig {
    input.inertia.clone()
  }
}
"#;
  fs::write(project_dir.join("crates/app/src/state.rs"), state_rs)?;

  // Create crates/app/src/handlers/mod.rs
  fs::write(
    project_dir.join("crates/app/src/handlers/mod.rs"),
    "pub mod auth;\npub mod cache_demo;\npub mod cached_page;\npub mod i18n;\npub mod inertia;\npub mod observability;\npub mod ws;",
  )?;

  // Create crates/app/src/handlers/inertia.rs (018: Inertia page handlers)
  let inertia_rs = r#"use axum::{extract::State, response::IntoResponse};
use axum::http::header::ACCEPT_LANGUAGE;
use axum::http::HeaderMap;
use axum_inertia::Inertia;
use serde_json::json;

use crate::state::AppState;

const SUPPORTED: &[&str] = &["en-US", "de"];

fn resolve_locale(headers: &HeaderMap) -> String {
  let header = headers
    .get(ACCEPT_LANGUAGE)
    .and_then(|v| v.to_str().ok())
    .unwrap_or("en-US");
  let chosen = accept_language::intersection(header, SUPPORTED);
  chosen
    .first()
    .map(|s| s.as_str().to_string())
    .unwrap_or_else(|| "en-US".to_string())
}

pub async fn home(
  i: Inertia,
  headers: HeaderMap,
  State(_state): State<AppState>,
) -> impl IntoResponse {
  let locale = resolve_locale(&headers);
  i.render("Pages/Home", json!({ "locale": locale }))
}

pub async fn login_page(i: Inertia, State(_state): State<AppState>) -> impl IntoResponse {
  i.render("Pages/Auth/Login", json!({}))
}

pub async fn register_page(i: Inertia, State(_state): State<AppState>) -> impl IntoResponse {
  i.render("Pages/Auth/Register", json!({}))
}

pub async fn dashboard(i: Inertia, State(_state): State<AppState>) -> impl IntoResponse {
  i.render("Pages/Dashboard", json!({ "message": "Welcome to the dashboard" }))
}

pub async fn ws_demo_page(i: Inertia, State(_state): State<AppState>) -> impl IntoResponse {
  i.render("Pages/WsDemo", json!({ "wsUrl": "/ws" }))
}
"#;
  fs::write(
    project_dir.join("crates/app/src/handlers/inertia.rs"),
    inertia_rs,
  )?;

  // --- 018: Frontend (Vite + React + Inertia, bun) ---
  let frontend_package_json = r#"{
  "name": "app-frontend",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview"
  },
  "dependencies": {
    "@inertiajs/react": "^1.0.0",
    "react": "^18.2.0",
    "react-dom": "^18.2.0"
  },
  "devDependencies": {
    "@types/react": "^18.2.0",
    "@types/react-dom": "^18.2.0",
    "@vitejs/plugin-react": "^4.2.0",
    "typescript": "^5.0.0",
    "vite": "^5.0.0"
  }
}
"#;
  fs::write(
    project_dir.join("frontend/package.json"),
    frontend_package_json,
  )?;

  let frontend_vite_config = r#"import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  base: '/assets/',
  root: '.',
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    manifest: true,
    rollupOptions: {
      input: 'src/main.tsx',
    },
  },
  server: {
    port: 5173,
    strictPort: true,
    origin: 'http://localhost:5173',
  },
});
"#;
  fs::write(
    project_dir.join("frontend/vite.config.ts"),
    frontend_vite_config,
  )?;

  let frontend_index_html = r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>App</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
"#;
  fs::write(project_dir.join("frontend/index.html"), frontend_index_html)?;

  let frontend_main_tsx = r#"import { createInertiaApp } from '@inertiajs/react';
import React from 'react';
import { createRoot } from 'react-dom/client';

createInertiaApp({
  resolve: (name) => {
    const pages = import.meta.glob('./pages/**/*.tsx', { eager: true });
    const path = `./pages/${name.replace(/^Pages\//, '').replace(/\./g, '/')}.tsx`;
    const mod = pages[path];
    if (!mod) throw new Error(`Page not found: ${name}`);
    return mod;
  },
  setup({ el, App, props }) {
    createRoot(el).render(<App {...props} />);
  },
});
"#;
  fs::write(project_dir.join("frontend/src/main.tsx"), frontend_main_tsx)?;

  let frontend_pages_home = r#"import React from 'react';

type Props = { locale: string };

export default function Home({ locale }: Props) {
  return (
    <div>
      <h1>Home</h1>
      <p>Locale: {locale}</p>
      <nav>
        <a href="/login">Login</a> | <a href="/register">Register</a> | <a href="/dashboard">Dashboard</a> | <a href="/ws-demo">WebSocket</a>
      </nav>
    </div>
  );
}
"#;
  fs::write(
    project_dir.join("frontend/src/pages/Home.tsx"),
    frontend_pages_home,
  )?;

  fs::create_dir_all(project_dir.join("frontend/src/pages/Auth"))?;

  let frontend_pages_login = r#"import React, { useState } from 'react';
import { router } from '@inertiajs/react';

export default function Login() {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    router.post('/api/auth/login', { email, password });
  };

  return (
    <div>
      <h1>Login</h1>
      <form onSubmit={submit}>
        <input type="email" value={email} onChange={(e) => setEmail(e.target.value)} placeholder="Email" required />
        <input type="password" value={password} onChange={(e) => setPassword(e.target.value)} placeholder="Password" required />
        <button type="submit">Log in</button>
      </form>
      <a href="/register">Register</a> | <a href="/">Home</a>
    </div>
  );
}
"#;
  fs::write(
    project_dir.join("frontend/src/pages/Auth/Login.tsx"),
    frontend_pages_login,
  )?;

  let frontend_pages_register = r#"import React, { useState } from 'react';
import { router } from '@inertiajs/react';

export default function Register() {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    router.post('/api/auth/register', { email, password });
  };

  return (
    <div>
      <h1>Register</h1>
      <form onSubmit={submit}>
        <input type="email" value={email} onChange={(e) => setEmail(e.target.value)} placeholder="Email" required />
        <input type="password" value={password} onChange={(e) => setPassword(e.target.value)} placeholder="Password" required />
        <button type="submit">Register</button>
      </form>
      <a href="/login">Login</a> | <a href="/">Home</a>
    </div>
  );
}
"#;
  fs::write(
    project_dir.join("frontend/src/pages/Auth/Register.tsx"),
    frontend_pages_register,
  )?;

  let frontend_pages_dashboard = r#"import React from 'react';

type Props = { message: string };

export default function Dashboard({ message }: Props) {
  return (
    <div>
      <h1>Dashboard</h1>
      <p>{message}</p>
      <nav>
        <a href="/">Home</a> | <a href="/login">Login</a> | <a href="/ws-demo">WebSocket</a>
      </nav>
    </div>
  );
}
"#;
  fs::write(
    project_dir.join("frontend/src/pages/Dashboard.tsx"),
    frontend_pages_dashboard,
  )?;

  let frontend_pages_ws_demo = r#"import React, { useState, useEffect, useRef } from 'react';

type Props = { wsUrl: string };

export default function WsDemo({ wsUrl }: Props) {
  const [messages, setMessages] = useState<string[]>([]);
  const [input, setInput] = useState('');
  const wsRef = useRef<WebSocket | null>(null);

  useEffect(() => {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const url = `${protocol}//${window.location.host}${wsUrl}`;
    const ws = new WebSocket(url);
    wsRef.current = ws;
    ws.onmessage = (e) => setMessages((m) => [...m, e.data]);
    ws.onclose = () => (wsRef.current = null);
    return () => { ws.close(); };
  }, [wsUrl]);

  const send = () => {
    if (wsRef.current?.readyState === WebSocket.OPEN && input.trim()) {
      wsRef.current.send(input);
      setInput('');
    }
  };

  return (
    <div>
      <h1>WebSocket Demo</h1>
      <p>Connected to {wsUrl}. Type and send messages.</p>
      <input value={input} onChange={(e) => setInput(e.target.value)} onKeyDown={(e) => e.key === 'Enter' && send()} />
      <button onClick={send}>Send</button>
      <ul>{messages.map((msg, i) => <li key={i}>{msg}</li>)}</ul>
      <a href="/">Home</a>
    </div>
  );
}
"#;
  fs::write(
    project_dir.join("frontend/src/pages/WsDemo.tsx"),
    frontend_pages_ws_demo,
  )?;

  // Create config/cache.toml (application + HTTP response cache config)
  let cache_toml = r#"enabled = true

[application]
enabled = true
max_capacity = 10_000
default_ttl_secs = 300

[http_response]
enabled = true
default_ttl_secs = 60
no_cache_paths = ["/", "/login", "/register", "/dashboard", "/ws-demo", "/api/auth", "/api/observability", "/healthz", "/livez", "/readyz"]
"#;
  fs::write(project_dir.join("config").join("cache.toml"), cache_toml)?;

  // Create crates/app/src/handlers/cache_demo.rs (demo route using app cache)
  let cache_demo_rs = r#"use axum::{extract::State, response::IntoResponse};
use forge::{DbConnection, AppCache};
use std::sync::Arc;

const CACHE_KEY: &str = "demo";

pub async fn handler(
  State(_db): State<DbConnection>,
  cache: axum::extract::Extension<Option<Arc<AppCache>>>,
) -> impl IntoResponse {
  let value = if let Some(c) = cache.0.as_ref() {
    if let Some(v) = c.get(CACHE_KEY).await {
      v
    } else {
      let v = format!("cached-{}", forge::uuid::Uuid::new_v4());
      c.set(CACHE_KEY, v.clone()).await;
      v
    }
  } else {
    "cache-disabled".to_string()
  };
  value.into_response()
}
"#;
  fs::write(
    project_dir.join("crates/app/src/handlers/cache_demo.rs"),
    cache_demo_rs,
  )?;

  // Create crates/app/src/handlers/cached_page.rs (demo for HTTP response cache: unique per request, cached by middleware)
  let cached_page_rs = r#"use axum::{extract::State, response::IntoResponse};
use forge::DbConnection;

pub async fn handler(State(_db): State<DbConnection>) -> impl IntoResponse {
  let v = format!("cached-page-{}", forge::uuid::Uuid::new_v4());
  v.into_response()
}
"#;
  fs::write(
    project_dir.join("crates/app/src/handlers/cached_page.rs"),
    cached_page_rs,
  )?;

  // Create crates/app/src/handlers/observability.rs (017: current trace id for e2e)
  let observability_rs = r#"use axum::{
  extract::Request,
  response::IntoResponse,
};
use forge::{find_current_trace_id, trace_id_from_traceparent};

/// Returns the current OpenTelemetry trace id (for e2e and debugging).
/// Prefers trace id from traceparent header when present, then current context.
pub async fn trace_id(req: Request) -> impl IntoResponse {
  let from_header = req
    .headers()
    .get("traceparent")
    .and_then(|v| v.to_str().ok())
    .and_then(|s| trace_id_from_traceparent(Some(s)));
  from_header
    .or_else(find_current_trace_id)
    .unwrap_or_else(|| "".to_string())
}
"#;
  fs::write(
    project_dir.join("crates/app/src/handlers/observability.rs"),
    observability_rs,
  )?;

  // Create crates/app/locales/en-US/main.ftl and de/main.ftl (i18n)
  fs::write(
    project_dir.join("crates/app/locales/en-US/main.ftl"),
    "greeting = Hello, { $name }!\n",
  )?;
  fs::write(
    project_dir.join("crates/app/locales/de/main.ftl"),
    "greeting = Hallo, { $name }!\n",
  )?;

  // Create crates/app/src/handlers/i18n.rs (stub; locale for Inertia pages is in handlers::inertia)
  let i18n_rs = r#"//! i18n: locale for Inertia pages is resolved in `handlers::inertia` from Accept-Language.
//! Use this module for API routes that need server-side translation (e.g. fluent_templates).
"#;
  fs::write(project_dir.join("crates/app/src/handlers/i18n.rs"), i18n_rs)?;

  // Create crates/app/src/handlers/ws.rs (WebSocket echo for real-time /ws)
  let ws_rs = r#"use axum::{
  extract::ws::{WebSocket, WebSocketUpgrade},
  response::Response,
};

pub async fn handler(ws: WebSocketUpgrade) -> Response {
  ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
  while let Some(msg) = socket.recv().await {
    let msg = match msg {
      Ok(m) => m,
      Err(_) => return,
    };
    if socket.send(msg).await.is_err() {
      return;
    }
  }
}
"#;
  fs::write(project_dir.join("crates/app/src/handlers/ws.rs"), ws_rs)?;

  // Create crates/app/src/handlers/auth.rs
  let auth_handlers_rs = r#"use axum::{
  extract::State,
  http::StatusCode,
  response::IntoResponse,
  Json,
};
use axum_login::AuthSession;
use chrono::Utc;
use forge::auth::{hash_api_token, hash_password};
use forge::audit::{AuditEvent, EventKind, Outcome};
use forge::authz::{guard_and_audit_user, Action, AuthzContext, Role};
use forge::validation::Valid;
use forge::token_auth::{OptionalRequireAuth, RequireAuth};
use forge::authz::record_authz_denied;
use forge::{DbConnection, Error as ForgeError};
use sea_orm::{ActiveModelTrait, EntityTrait, Set, TransactionTrait};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use db::auth::Backend;
use db::models::{api_token, organization, membership, user};

#[derive(Deserialize, Validate)]
pub struct RegisterRequest {
  #[validate(email)]
  pub email: String,
  #[validate(length(min = 8))]
  pub password: String,
}

pub async fn register(
  State(db): State<DbConnection>,
  Valid(Json(payload)): Valid<Json<RegisterRequest>>,
) -> Result<impl IntoResponse, ForgeError> {
  let password_hash = hash_password(&payload.password)?;
  let now = Utc::now().naive_utc();
  let user_id = Uuid::new_v4();
  let org_id = Uuid::new_v4();
  let membership_id = Uuid::new_v4();

  let tx = db.begin().await?;
  let new_user = user::ActiveModel {
    id: Set(user_id),
    email: Set(payload.email.clone()),
    password_hash: Set(password_hash),
    is_active: Set(true),
    is_admin: Set(false),
    current_org_id: Set(None),
    current_role: Set(None),
    created_at: Set(now),
    updated_at: Set(now),
  };
  user::Entity::insert(new_user).exec(&tx).await?;

  let slug = format!("org-{}", org_id.as_simple());
  let new_org = organization::ActiveModel {
    id: Set(org_id),
    name: Set(format!("{}'s workspace", payload.email)),
    slug: Set(slug),
    created_at: Set(now),
    updated_at: Set(now),
  };
  organization::Entity::insert(new_org).exec(&tx).await?;

  let new_membership = membership::ActiveModel {
    id: Set(membership_id),
    user_id: Set(user_id),
    org_id: Set(org_id),
    role: Set("owner".to_string()),
    created_at: Set(now),
    updated_at: Set(now),
  };
  membership::Entity::insert(new_membership).exec(&tx).await?;

  let u = user::Entity::find_by_id(user_id).one(&tx).await?.ok_or_else(|| ForgeError::Generic("User not found".into()))?;
  let mut am: user::ActiveModel = u.into();
  am.current_org_id = Set(Some(org_id));
  am.current_role = Set(Some("owner".to_string()));
  am.updated_at = Set(now);
  am.update(&tx).await?;

  tx.commit().await?;

  Ok((StatusCode::CREATED, "User registered successfully"))
}

#[derive(Deserialize, Validate)]
pub struct LoginRequest {
  #[validate(email)]
  pub email: String,
  #[validate(length(min = 1))]
  pub password: String,
}

pub async fn login(
  mut auth_session: AuthSession<Backend>,
  State(db): State<DbConnection>,
  Valid(Json(payload)): Valid<Json<LoginRequest>>,
) -> Result<impl IntoResponse, ForgeError> {
  let credentials = db::auth::Credentials {
    email: payload.email,
    password: payload.password,
  };

  let user = auth_session
    .authenticate(credentials)
    .await
    .map_err(|e| ForgeError::Generic(format!("Authentication error: {}", e)))?;

  if let Some(ref user) = user {
    auth_session
      .login(user)
      .await
      .map_err(|e| ForgeError::Generic(format!("Login error: {}", e)))?;
    let _ = forge::audit::log(
      &db,
      AuditEvent {
        event_kind: EventKind::Auth,
        actor_id: user.id,
        subject_id: Some(user.id),
        organization_id: user.current_org_id,
        action: Action::Manage,
        resource_type: "auth".to_string(),
        resource_id: None,
        outcome: Outcome::Success,
        reason: Some("login".to_string()),
      },
    )
    .await;
    Ok(StatusCode::OK.into_response())
  } else {
    let _ = forge::audit::log(
      &db,
      AuditEvent {
        event_kind: EventKind::Auth,
        actor_id: Uuid::nil(),
        subject_id: None,
        organization_id: None,
        action: Action::Manage,
        resource_type: "auth".to_string(),
        resource_id: None,
        outcome: Outcome::Failure,
        reason: Some("failed_login".to_string()),
      },
    )
    .await;
    Ok((StatusCode::UNAUTHORIZED, "Invalid credentials").into_response())
  }
}

pub async fn logout(
  mut auth_session: AuthSession<Backend>,
  State(db): State<DbConnection>,
) -> impl IntoResponse {
  let actor_id = auth_session.requester_id();
  let org_id = auth_session.organization_id();
  auth_session.logout().await.unwrap();
  let _ = forge::audit::log(
    &db,
    AuditEvent {
      event_kind: EventKind::Auth,
      actor_id,
      subject_id: Some(actor_id),
      organization_id: org_id,
      action: Action::Manage,
      resource_type: "auth".to_string(),
      resource_id: None,
      outcome: Outcome::Success,
      reason: Some("logout".to_string()),
    },
  )
  .await;
  StatusCode::OK
}

pub async fn profile(auth_session: AuthSession<Backend>) -> impl IntoResponse {
  match &auth_session.user {
    Some(user) => {
      let org = auth_session.organization_id().map(|id| id.to_string()).unwrap_or_else(|| "none".to_string());
      let role = auth_session.role().map(|r| format!("{:?}", r)).unwrap_or_else(|| "none".to_string());
      format!("Hello, {}! org={} role={}", user.email, org, role).into_response()
    }
    None => (StatusCode::UNAUTHORIZED, "Not logged in").into_response(),
  }
}

#[derive(Deserialize, Validate)]
pub struct CreateTokenRequest {
  pub name: Option<String>,
}

pub async fn create_token(
  RequireAuth(user): RequireAuth<Backend>,
  State(db): State<DbConnection>,
  Valid(Json(payload)): Valid<Json<CreateTokenRequest>>,
) -> Result<impl IntoResponse, ForgeError> {
  let secret = format!("forge_{}", Uuid::new_v4().to_string().replace('-', ""));
  let token_hash = hash_api_token(&secret);
  let now = Utc::now().naive_utc();
  let id = Uuid::new_v4();
  let model = api_token::ActiveModel {
    id: Set(id),
    user_id: Set(user.id),
    token_hash: Set(token_hash),
    name: Set(payload.name),
    last_used_at: Set(None),
    expires_at: Set(None),
    created_at: Set(now),
    updated_at: Set(now),
  };
  api_token::Entity::insert(model).exec(&db).await?;
  Ok(Json(forge::serde_json::json!({ "token": secret })))
}

/// Shallow Gate example: only users with Role::Owner (or Admin) can access. Audits the decision.
pub async fn admin_only(
  OptionalRequireAuth(maybe_user): OptionalRequireAuth<Backend>,
  State(db): State<DbConnection>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = match maybe_user {
    Some(u) => u,
    None => {
      record_authz_denied(&db, Action::Manage, "admin", None).await;
      return Ok((StatusCode::UNAUTHORIZED, "Authentication required").into_response());
    }
  };
  guard_and_audit_user(&user, &db, Action::Manage, Role::Owner, "admin", None).await?;
  Ok((StatusCode::OK, format!("Admin only: access granted for {}", user.email)).into_response())
}
"#;
  fs::write(
    project_dir.join("crates/app/src/handlers/auth.rs"),
    auth_handlers_rs,
  )?;

  // Create crates/db/src/lib.rs
  let db_lib_rs = r#"use async_trait::async_trait;
use forge::DbConnection;
use sea_orm_migration::prelude::{MigrationTrait, MigratorTrait};

pub mod migrations;
pub mod models;
pub mod seeds;
pub mod auth;

pub struct Migrator;

#[async_trait]
impl MigratorTrait for Migrator {
  fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
      Box::new(migrations::m20220101_000001_create_user_table::Migration),
      Box::new(migrations::m20220101_000002_create_sessions_table::Migration),
      Box::new(migrations::m20220101_000003_create_organizations_table::Migration),
      Box::new(migrations::m20220101_000004_create_memberships_table::Migration),
      Box::new(migrations::m20220101_000005_create_audit_log_table::Migration),
      Box::new(migrations::m20220101_000006_create_api_tokens_table::Migration),
    ]
  }
}

pub async fn run_seeds(db: DbConnection) -> Result<(), Box<dyn std::error::Error>> {
  seeds::s20220101_000001_seed_users::seed(&db).await?;
  Ok(())
}

/// Look up user id by raw API token (Bearer). Returns None if token invalid or expired.
pub async fn token_lookup(db: DbConnection, raw_token: String) -> Option<uuid::Uuid> {
  use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
  let hash = forge::auth::hash_api_token(&raw_token);
  let row = crate::models::api_token::Entity::find()
    .filter(crate::models::api_token::Column::TokenHash.eq(hash))
    .one(&db)
    .await
    .ok()
    .flatten()?;
  if let Some(exp) = row.expires_at {
    if exp < chrono::Utc::now().naive_utc() {
      return None;
    }
  }
  Some(row.user_id)
}
"#;
  fs::write(project_dir.join("crates/db/src/lib.rs"), db_lib_rs)?;

  // Create crates/db/src/auth.rs
  let db_auth_rs = r#"use async_trait::async_trait;
use forge::auth::verify_password;
use forge::{axum_login::AuthnBackend, DbConnection, Error};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;

use crate::models::user;

#[derive(Clone, Debug)]
pub struct Backend {
  db: DbConnection,
}

impl Backend {
  pub fn new(db: DbConnection) -> Self {
    Self { db }
  }
}

#[derive(Debug, Deserialize)]
pub struct Credentials {
  pub email: String,
  pub password: String,
}

#[async_trait]
impl AuthnBackend for Backend {
  type User = user::Model;
  type Credentials = Credentials;
  type Error = Error;

  async fn authenticate(
  &self,
  creds: Self::Credentials,
  ) -> Result<Option<Self::User>, Self::Error> {
    let user = user::Entity::find()
      .filter(user::Column::Email.eq(creds.email))
      .one(&self.db)
      .await?;

    if let Some(user) = user {
      if verify_password(&creds.password, &user.password_hash)? {
        return Ok(Some(user));
      }
    }

    Ok(None)
  }

  async fn get_user(&self, user_id: &forge::axum_login::UserId<Self>) -> Result<Option<Self::User>, Error> {
    let user = user::Entity::find_by_id(*user_id)
      .one(&self.db)
      .await?;
    Ok(user)
  }
}
"#;
  fs::write(project_dir.join("crates/db/src/auth.rs"), db_auth_rs)?;

  // Create crates/db/src/migrations/mod.rs
  fs::write(
    project_dir.join("crates/db/src/migrations/mod.rs"),
    "pub mod m20220101_000001_create_user_table;\npub mod m20220101_000002_create_sessions_table;\npub mod m20220101_000003_create_organizations_table;\npub mod m20220101_000004_create_memberships_table;\npub mod m20220101_000005_create_audit_log_table;\npub mod m20220101_000006_create_api_tokens_table;",
  )?;

  // Create crates/db/src/migrations/m20220101_000001_create_user_table.rs
  let migration_rs = r#"use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum User {
  Table,
  Id,
  Email,
  PasswordHash,
  IsActive,
  IsAdmin,
  CurrentOrgId,
  CurrentRole,
  CreatedAt,
  UpdatedAt,
}

pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m20220101_000001_create_user_table"
  }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(User::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(User::Id)
              .uuid()
              .not_null()
              .primary_key(),
          )
          .col(ColumnDef::new(User::Email).string().unique_key().not_null())
          .col(ColumnDef::new(User::PasswordHash).string().not_null())
          .col(ColumnDef::new(User::IsActive).boolean().not_null().default(true))
          .col(ColumnDef::new(User::IsAdmin).boolean().not_null().default(false))
          .col(ColumnDef::new(User::CurrentOrgId).uuid())
          .col(ColumnDef::new(User::CurrentRole).string())
          .col(ColumnDef::new(User::CreatedAt).date_time().not_null())
          .col(ColumnDef::new(User::UpdatedAt).date_time().not_null())
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(User::Table).to_owned())
      .await
  }
}
"#;
  fs::write(
    project_dir.join("crates/db/src/migrations/m20220101_000001_create_user_table.rs"),
    migration_rs,
  )?;

  // Create crates/db/src/migrations/m20220101_000002_create_sessions_table.rs
  let sessions_migration_rs = r#"use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m20220101_000002_create_sessions_table"
  }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(Alias::new("sessions"))
          .if_not_exists()
          .col(ColumnDef::new(Alias::new("id")).string().not_null().primary_key())
          .col(ColumnDef::new(Alias::new("data")).binary().not_null())
          .col(ColumnDef::new(Alias::new("expiry_date")).big_integer().not_null())
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(Alias::new("sessions")).to_owned())
      .await
  }
}
"#;
  fs::write(
    project_dir.join("crates/db/src/migrations/m20220101_000002_create_sessions_table.rs"),
    sessions_migration_rs,
  )?;

  let org_migration_rs = r#"use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum Organization {
  Table,
  Id,
  Name,
  Slug,
  CreatedAt,
  UpdatedAt,
}

pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m20220101_000003_create_organizations_table"
  }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(Organization::Table)
          .if_not_exists()
          .col(ColumnDef::new(Organization::Id).uuid().not_null().primary_key())
          .col(ColumnDef::new(Organization::Name).string().not_null())
          .col(ColumnDef::new(Organization::Slug).string().unique_key().not_null())
          .col(ColumnDef::new(Organization::CreatedAt).date_time().not_null())
          .col(ColumnDef::new(Organization::UpdatedAt).date_time().not_null())
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager.drop_table(Table::drop().table(Organization::Table).to_owned()).await
  }
}
"#;
  fs::write(
    project_dir.join("crates/db/src/migrations/m20220101_000003_create_organizations_table.rs"),
    org_migration_rs,
  )?;

  let membership_migration_rs = r#"use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum Membership {
  Table,
  Id,
  UserId,
  OrgId,
  Role,
  CreatedAt,
  UpdatedAt,
}

pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m20220101_000004_create_memberships_table"
  }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(Membership::Table)
          .if_not_exists()
          .col(ColumnDef::new(Membership::Id).uuid().not_null().primary_key())
          .col(ColumnDef::new(Membership::UserId).uuid().not_null())
          .col(ColumnDef::new(Membership::OrgId).uuid().not_null())
          .col(ColumnDef::new(Membership::Role).string().not_null())
          .col(ColumnDef::new(Membership::CreatedAt).date_time().not_null())
          .col(ColumnDef::new(Membership::UpdatedAt).date_time().not_null())
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager.drop_table(Table::drop().table(Membership::Table).to_owned()).await
  }
}
"#;
  fs::write(
    project_dir.join("crates/db/src/migrations/m20220101_000004_create_memberships_table.rs"),
    membership_migration_rs,
  )?;

  // Create crates/db/src/migrations/m20220101_000005_create_audit_log_table.rs (Phase 7)
  let audit_log_migration_rs = r#"use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum AuditLog {
  Table,
  Id,
  EventKind,
  ActorId,
  SubjectId,
  OrganizationId,
  Action,
  ResourceType,
  ResourceId,
  Outcome,
  Reason,
  OccurredAt,
}

pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m20220101_000005_create_audit_log_table"
  }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(AuditLog::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(AuditLog::Id)
              .uuid()
              .not_null()
              .primary_key(),
          )
          .col(ColumnDef::new(AuditLog::EventKind).string().not_null())
          .col(ColumnDef::new(AuditLog::ActorId).uuid().not_null())
          .col(ColumnDef::new(AuditLog::SubjectId).uuid())
          .col(ColumnDef::new(AuditLog::OrganizationId).uuid())
          .col(ColumnDef::new(AuditLog::Action).string().not_null())
          .col(ColumnDef::new(AuditLog::ResourceType).string().not_null())
          .col(ColumnDef::new(AuditLog::ResourceId).uuid())
          .col(ColumnDef::new(AuditLog::Outcome).string().not_null())
          .col(ColumnDef::new(AuditLog::Reason).string())
          .col(ColumnDef::new(AuditLog::OccurredAt).date_time().not_null())
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_audit_log_org_occurred")
          .table(AuditLog::Table)
          .col(AuditLog::OrganizationId)
          .col(AuditLog::OccurredAt)
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_audit_log_actor_occurred")
          .table(AuditLog::Table)
          .col(AuditLog::ActorId)
          .col(AuditLog::OccurredAt)
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(AuditLog::Table).to_owned())
      .await
  }
}
"#;
  fs::write(
    project_dir.join("crates/db/src/migrations/m20220101_000005_create_audit_log_table.rs"),
    audit_log_migration_rs,
  )?;

  // Create crates/db/src/migrations/m20220101_000006_create_api_tokens_table.rs (Phase 012)
  let api_tokens_migration_rs = r#"use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum ApiTokens {
  Table,
  Id,
  UserId,
  TokenHash,
  Name,
  LastUsedAt,
  ExpiresAt,
  CreatedAt,
  UpdatedAt,
}

pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m20220101_000006_create_api_tokens_table"
  }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(ApiTokens::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(ApiTokens::Id)
              .uuid()
              .not_null()
              .primary_key(),
          )
          .col(ColumnDef::new(ApiTokens::UserId).uuid().not_null())
          .col(ColumnDef::new(ApiTokens::TokenHash).string().not_null())
          .col(ColumnDef::new(ApiTokens::Name).string())
          .col(ColumnDef::new(ApiTokens::LastUsedAt).date_time())
          .col(ColumnDef::new(ApiTokens::ExpiresAt).date_time())
          .col(ColumnDef::new(ApiTokens::CreatedAt).date_time().not_null())
          .col(ColumnDef::new(ApiTokens::UpdatedAt).date_time().not_null())
          .foreign_key(
            ForeignKey::create()
              .name("fk_api_tokens_user_id")
              .from_tbl(ApiTokens::Table)
              .from_col(ApiTokens::UserId)
              .to_tbl(Alias::new("user"))
              .to_col(Alias::new("id")),
          )
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(ApiTokens::Table).to_owned())
      .await
  }
}
"#;
  fs::write(
    project_dir.join("crates/db/src/migrations/m20220101_000006_create_api_tokens_table.rs"),
    api_tokens_migration_rs,
  )?;

  // Create crates/db/src/seeds/mod.rs
  fs::write(
    project_dir.join("crates/db/src/seeds/mod.rs"),
    "pub mod s20220101_000001_seed_users;",
  )?;

  // Create crates/db/src/seeds/s20220101_000001_seed_users.rs
  let seed_rs = r#"use chrono::Utc;
use forge::auth::hash_password;
use forge::DbConnection;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};
use tracing::info;
use uuid::Uuid;

use crate::models::{organization, membership, user};

pub async fn seed(db: &DbConnection) -> Result<(), Box<dyn std::error::Error>> {
  let email = "root@localhost";

  let existing = user::Entity::find()
    .filter(user::Column::Email.eq(email))
    .one(db)
    .await?;

  if existing.is_none() {
    let now = Utc::now().naive_utc();
    let user_id = Uuid::new_v4();
    let org_id = Uuid::new_v4();
    let membership_id = Uuid::new_v4();
    let password_hash = hash_password("password123")?;

    let root_user = user::ActiveModel {
      id: Set(user_id),
      email: Set(email.to_owned()),
      password_hash: Set(password_hash),
      is_active: Set(true),
      is_admin: Set(true),
      current_org_id: Set(Some(org_id)),
      current_role: Set(Some("owner".to_string())),
      created_at: Set(now),
      updated_at: Set(now),
    };
    user::Entity::insert(root_user).exec(db).await?;

    let default_org = organization::ActiveModel {
      id: Set(org_id),
      name: Set("Default".to_string()),
      slug: Set("default".to_string()),
      created_at: Set(now),
      updated_at: Set(now),
    };
    organization::Entity::insert(default_org).exec(db).await?;

    let root_membership = membership::ActiveModel {
      id: Set(membership_id),
      user_id: Set(user_id),
      org_id: Set(org_id),
      role: Set("owner".to_string()),
      created_at: Set(now),
      updated_at: Set(now),
    };
    membership::Entity::insert(root_membership).exec(db).await?;

    info!("Seeded root user: {} with org and Owner membership", email);
  }

  Ok(())
}
"#;
  fs::write(
    project_dir.join("crates/db/src/seeds/s20220101_000001_seed_users.rs"),
    seed_rs,
  )?;

  // Create crates/db/src/models/mod.rs
  fs::write(
    project_dir.join("crates/db/src/models/mod.rs"),
    "pub mod user;\npub mod organization;\npub mod membership;\npub mod api_token;",
  )?;

  // Create crates/db/src/models/user.rs
  let user_model_rs = r#"use forge::authz::{AuthzContext, Role};
use forge::ForgeAuthUser;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, ForgeAuthUser)]
#[sea_orm(table_name = "user")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: Uuid,
  #[sea_orm(unique)]
  pub email: String,
  pub password_hash: String,
  pub is_active: bool,
  pub is_admin: bool,
  pub current_org_id: Option<Uuid>,
  pub current_role: Option<String>,
  pub created_at: chrono::NaiveDateTime,
  pub updated_at: chrono::NaiveDateTime,
}

impl AuthzContext for Model {
  fn requester_id(&self) -> Uuid {
    self.id
  }
  fn subject_id(&self) -> Uuid {
    self.id
  }
  fn organization_id(&self) -> Option<Uuid> {
    self.current_org_id
  }
  fn role(&self) -> Option<Role> {
    self.current_role.as_deref().and_then(|s| match s {
      "owner" => Some(Role::Owner),
      "admin" => Some(Role::Admin),
      "editor" => Some(Role::Editor),
      "viewer" => Some(Role::Viewer),
      _ => None,
    })
  }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
"#;
  fs::write(
    project_dir.join("crates/db/src/models/user.rs"),
    user_model_rs,
  )?;

  let org_model_rs = r#"use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "organization")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: Uuid,
  pub name: String,
  #[sea_orm(unique)]
  pub slug: String,
  pub created_at: chrono::NaiveDateTime,
  pub updated_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
"#;
  fs::write(
    project_dir.join("crates/db/src/models/organization.rs"),
    org_model_rs,
  )?;

  let membership_model_rs = r#"use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "membership")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: Uuid,
  pub user_id: Uuid,
  pub org_id: Uuid,
  pub role: String,
  pub created_at: chrono::NaiveDateTime,
  pub updated_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
"#;
  fs::write(
    project_dir.join("crates/db/src/models/membership.rs"),
    membership_model_rs,
  )?;

  let api_token_model_rs = r#"use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "api_tokens")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: Uuid,
  pub user_id: Uuid,
  pub token_hash: String,
  pub name: Option<String>,
  pub last_used_at: Option<chrono::NaiveDateTime>,
  pub expires_at: Option<chrono::NaiveDateTime>,
  pub created_at: chrono::NaiveDateTime,
  pub updated_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
"#;
  fs::write(
    project_dir.join("crates/db/src/models/api_token.rs"),
    api_token_model_rs,
  )?;

  // Create config/app.toml
  let app_toml = format!(
    r#"[app]
name = "{}"
environment = "development"

[server]
host = "0.0.0.0"
port = 3000
"#,
    name
  );
  fs::write(project_dir.join("config").join("app.toml"), app_toml)?;

  // Create config/db.toml
  let db_toml = r#"[database]
# SQLite connection string. The file will be created in the project root.
url = "sqlite://db.sqlite?mode=rwc"
max_connections = 5
min_connections = 1
connect_timeout = 10
idle_timeout = 600
auto_migrate = true
auto_seed = true
"#;
  fs::write(project_dir.join("config").join("db.toml"), db_toml)?;

  // Initialize git repository
  Command::new("git")
    .arg("init")
    .current_dir(project_dir)
    .status()?;

  Ok(())
}
