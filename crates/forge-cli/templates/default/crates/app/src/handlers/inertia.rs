use axum::{extract::State, response::{IntoResponse, Redirect}};
use axum::http::header::ACCEPT_LANGUAGE;
use axum::http::HeaderMap;
use axum_inertia::Inertia;
use forge::token_auth::OptionalRequireAuth;
use serde_json::json;

use db::auth::Backend;
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
  let greeting = crate::handlers::i18n::greeting(&locale);
  i.render("Pages/Home", json!({ "locale": locale, "greeting": greeting }))
}

pub async fn login_page(i: Inertia, State(_state): State<AppState>) -> impl IntoResponse {
  i.render("Pages/Auth/Login", json!({}))
}

pub async fn register_page(i: Inertia, State(_state): State<AppState>) -> impl IntoResponse {
  i.render("Pages/Auth/Register", json!({}))
}

pub async fn dashboard(
  OptionalRequireAuth(maybe_user): OptionalRequireAuth<Backend>,
  i: Inertia,
  State(_state): State<AppState>,
) -> impl IntoResponse {
  match maybe_user {
    Some(_) => i.render("Pages/Dashboard", json!({ "message": "Welcome to the dashboard" })).into_response(),
    None => Redirect::to("/login").into_response(),
  }
}

pub async fn ws_demo_page(i: Inertia, State(_state): State<AppState>) -> impl IntoResponse {
  i.render("Pages/WsDemo", json!({ "wsUrl": "/ws" }))
}
