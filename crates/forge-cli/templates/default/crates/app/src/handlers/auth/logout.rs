//! GET/POST /api/auth/logout — clear session. Client should discard the token.
//! Works for both authenticated and anonymous users.

use axum::{Json, http::StatusCode, response::IntoResponse};

pub async fn logout() -> impl IntoResponse {
  tracing::debug!(target: "app::auth", "route: GET/POST /api/auth/logout");
  (StatusCode::OK, Json(serde_json::json!({ "ok": true }))).into_response()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn logout_returns_200_and_ok() {
    let response = logout().await.into_response();
    let status = response.status();
    assert_eq!(status, StatusCode::OK);
  }
}
