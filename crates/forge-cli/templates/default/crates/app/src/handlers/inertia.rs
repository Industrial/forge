use axum::{extract::State, response::{IntoResponse, Redirect}};
use axum::http::header::{ACCEPT_LANGUAGE, COOKIE};
use axum::http::HeaderMap;
use axum_inertia::Inertia;
use forge::token_auth::OptionalRequireAuth;
use serde_json::json;
use tower_sessions::Session;

use db::auth::Backend;
use crate::handlers::inertia_shared;
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
  session: Session,
  headers: HeaderMap,
  OptionalRequireAuth(maybe_user): OptionalRequireAuth<Backend>,
  State(_state): State<AppState>,
) -> impl IntoResponse {
  tracing::debug!(target: "app::handlers", "route: GET /");
  let locale = resolve_locale(&headers);
  let greeting = crate::handlers::i18n::greeting(&locale);
  inertia_shared::render_with_shared(
    i,
    session,
    maybe_user,
    "Pages/Home",
    json!({ "locale": locale, "greeting": greeting }),
  )
  .await
}

pub async fn login_page(
  i: Inertia,
  session: Session,
  OptionalRequireAuth(maybe_user): OptionalRequireAuth<Backend>,
  State(_state): State<AppState>,
) -> impl IntoResponse {
  tracing::debug!(target: "app::handlers", "route: GET /login");
  // Pass maybe_user by value (not .as_ref()) so the async future doesn't capture a reference (Rust 2024).
  inertia_shared::render_with_shared(i, session, maybe_user, "Pages/Auth/Login", json!({}))
    .await
}

pub async fn register_page(
  i: Inertia,
  session: Session,
  OptionalRequireAuth(maybe_user): OptionalRequireAuth<Backend>,
  State(_state): State<AppState>,
) -> impl IntoResponse {
  tracing::debug!(target: "app::handlers", "route: GET /register");
  // Pass maybe_user by value (not .as_ref()) for Rust 2024 lifetime rules.
  inertia_shared::render_with_shared(i, session, maybe_user, "Pages/Auth/Register", json!({}))
    .await
}

pub async fn dashboard(
  headers: HeaderMap,
  OptionalRequireAuth(maybe_user): OptionalRequireAuth<Backend>,
  i: Inertia,
  session: Session,
  State(_state): State<AppState>,
) -> impl IntoResponse {
  let cookie_header = headers.get(COOKIE).and_then(|v| v.to_str().ok()).unwrap_or("");
  let has_cookie = !cookie_header.is_empty();
  let cookie_names: Vec<&str> = cookie_header
    .split("; ")
    .filter_map(|s| s.split('=').next().map(|n| n.trim()))
    .collect();
  // tower_sessions 0.14 uses cookie name "id" by default; also check common alternates.
  let has_session_cookie = cookie_names.iter().any(|&n| {
    n == "id" || n == "tower-session" || n == "tower_session" || n.eq_ignore_ascii_case("tower-session")
  });
  let cookie_names_str = cookie_names.join(",");
  tracing::debug!(
    target: "app::handlers",
    "route: GET /dashboard authenticated={} cookie_present={} session_cookie_present={} cookie_len={} cookie_names=\"{}\"",
    maybe_user.is_some(),
    has_cookie,
    has_session_cookie,
    cookie_header.len(),
    cookie_names_str
  );
  match maybe_user {
    Some(user) => {
      inertia_shared::render_with_shared(
        i,
        session,
        Some(user),
        "Pages/Dashboard",
        json!({ "message": "Welcome to the dashboard" }),
      )
      .await
      .into_response()
    }
    None => Redirect::to("/login").into_response(),
  }
}

pub async fn ws_demo_page(
  i: Inertia,
  session: Session,
  OptionalRequireAuth(maybe_user): OptionalRequireAuth<Backend>,
  State(_state): State<AppState>,
) -> impl IntoResponse {
  tracing::debug!(target: "app::handlers", "route: GET /ws-demo");
  // Pass maybe_user by value (not .as_ref()) for Rust 2024 lifetime rules.
  inertia_shared::render_with_shared(i, session, maybe_user, "Pages/WsDemo", json!({ "wsUrl": "/ws" }))
    .await
} 
