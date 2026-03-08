//! GET /api/auth/admin — global admin only (is_admin or global permission).

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use forge_audit::record_authz_denied;
use forge_auth::{Action, token_auth::OptionalRequireAuth};
use forge_db::DbConnection;

use db::auth::Backend;
use db::models::user;

use crate::Error as ForgeError;

pub async fn admin_only(
  opt_auth: OptionalRequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
) -> Result<impl IntoResponse, ForgeError> {
  let maybe_user = opt_auth.0;
  tracing::debug!(target: "app::auth", "route: GET /api/auth/admin authenticated={}", maybe_user.is_some());
  let user = match maybe_user {
    Some(u) => u,
    None => {
      record_authz_denied(&db, Action::Manage, "admin", None, None).await;
      return Ok(
        (
          StatusCode::UNAUTHORIZED,
          Json(serde_json::json!({ "error": "Authentication required" })),
        )
          .into_response(),
      );
    }
  };
  if !user.is_admin {
    record_authz_denied(&db, Action::Manage, "admin", Some(user.id), None).await;
    return Ok(
      (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({ "error": "Forbidden" })),
      )
        .into_response(),
    );
  }
  Ok(
    (
      StatusCode::OK,
      Json(serde_json::json!({
        "message": format!("Admin only: access granted for {}", user.email)
      })),
    )
      .into_response(),
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn admin_only_handler_exists() {
    let _ = admin_only;
  }
}
