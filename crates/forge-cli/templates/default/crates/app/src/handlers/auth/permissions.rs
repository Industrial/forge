//! GET /api/auth/permissions — list known permission keys (code-defined).

use axum::{Json, response::IntoResponse};
use forge_auth::token_auth::RequireAuth;

use db::auth::Backend;
use db::models::user;

use crate::Error as ForgeError;

use super::shared::{DbFromScope, ScopeFromHeaders, require_entity_permission};

pub async fn list_permissions(
  ScopeFromHeaders(scope, _): ScopeFromHeaders<user::Model>,
  auth: RequireAuth<Backend, user::Model>,
  DbFromScope(db): DbFromScope,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_entity_permission(user, &db, Some(&scope), "permission", "read").await
  {
    return Ok(resp);
  }
  let list: Vec<&str> = crate::permissions::dashboard_permissions().to_vec();
  Ok(Json(serde_json::json!({ "permissions": list })).into_response())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn list_permissions_handler_exists() {
    let _ = list_permissions;
  }
}
