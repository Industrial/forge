//! GET/POST/DELETE /api/auth/global-role-assignments. Flat: user_id, role_name in query/body.

use axum::{Json, extract::Query, http::StatusCode, response::IntoResponse};
use forge_auth::token_auth::RequireAuth;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;
use uuid::Uuid;

use db::auth::Backend;
use db::models::{user, user_global_role};

use crate::Error as ForgeError;

use super::shared::{
  DbFromScope, PERMISSION_USERS_WRITE, ScopeFromHeaders, has_global_scope, require_permission,
};

#[derive(Debug, Deserialize)]
pub struct ListGlobalRoleAssignmentsQuery {
  pub user_id: Option<Uuid>,
}

/// GET /api/auth/global-role-assignments — list. ?user_id= optional. Requires user.write (or all.write).
pub async fn list_global_role_assignments(
  ScopeFromHeaders(scope, _): ScopeFromHeaders<user::Model>,
  auth: RequireAuth<Backend, user::Model>,
  DbFromScope(db): DbFromScope,
  Query(q): Query<ListGlobalRoleAssignmentsQuery>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_USERS_WRITE, Some(&scope)).await {
    return Ok(resp);
  }
  if !has_global_scope(&db, user, PERMISSION_USERS_WRITE).await {
    return Ok(
      (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({ "error": "Global role assignments require global scope" })),
      )
        .into_response(),
    );
  }
  let mut query = user_global_role::Entity::find();
  if let Some(user_id) = q.user_id {
    query = query.filter(user_global_role::Column::UserId.eq(user_id));
  }
  let rows = query
    .all(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  let list: Vec<serde_json::Value> = rows
    .into_iter()
    .map(|r| {
      serde_json::json!({
        "id": r.id.to_string(),
        "user_id": r.user_id.to_string(),
        "role_name": r.role_name,
      })
    })
    .collect();
  Ok(Json(serde_json::json!({ "assignments": list })).into_response())
}

#[derive(Deserialize)]
pub struct AddGlobalRoleAssignmentBody {
  pub user_id: Uuid,
  pub role_name: String,
}

/// POST /api/auth/global-role-assignments — assign global role to user. Requires global user.write.
pub async fn add_global_role_assignment(
  ScopeFromHeaders(scope, _): ScopeFromHeaders<user::Model>,
  auth: RequireAuth<Backend, user::Model>,
  DbFromScope(db): DbFromScope,
  Json(payload): Json<AddGlobalRoleAssignmentBody>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_USERS_WRITE, Some(&scope)).await {
    return Ok(resp);
  }
  if !has_global_scope(&db, user, PERMISSION_USERS_WRITE).await {
    return Ok(
      (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({ "error": "Global role assignments require global scope" })),
      )
        .into_response(),
    );
  }
  let role_name = payload.role_name.trim();
  if role_name.is_empty() {
    return Ok(
      (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(serde_json::json!({ "error": "role_name is required" })),
      )
        .into_response(),
    );
  }
  let user_exists = user::Entity::find_by_id(payload.user_id)
    .one(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  if user_exists.is_none() {
    return Ok(
      (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "User not found" })),
      )
        .into_response(),
    );
  }
  let id = Uuid::new_v4();
  let model = user_global_role::ActiveModel {
    id: Set(id),
    user_id: Set(payload.user_id),
    role_name: Set(role_name.to_string()),
  };
  if let Err(e) = user_global_role::Entity::insert(model).exec(&db).await {
    if matches!(
      e.sql_err(),
      Some(sea_orm::SqlErr::UniqueConstraintViolation(_))
    ) {
      return Ok(
        (
          StatusCode::UNPROCESSABLE_ENTITY,
          Json(serde_json::json!({ "error": "assignment already exists" })),
        )
          .into_response(),
      );
    }
    return Err(ForgeError::Generic(e.to_string()));
  }
  Ok(
    (
      StatusCode::CREATED,
      Json(serde_json::json!({ "ok": true, "id": id.to_string() })),
    )
      .into_response(),
  )
}

#[derive(Deserialize)]
pub struct DeleteGlobalRoleAssignmentBody {
  pub user_id: Uuid,
  pub role_name: String,
}

/// DELETE /api/auth/global-role-assignments — remove assignment (body). Requires global user.write.
pub async fn delete_global_role_assignment(
  ScopeFromHeaders(scope, _): ScopeFromHeaders<user::Model>,
  auth: RequireAuth<Backend, user::Model>,
  DbFromScope(db): DbFromScope,
  Json(payload): Json<DeleteGlobalRoleAssignmentBody>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_USERS_WRITE, Some(&scope)).await {
    return Ok(resp);
  }
  if !has_global_scope(&db, user, PERMISSION_USERS_WRITE).await {
    return Ok(
      (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({ "error": "Global role assignments require global scope" })),
      )
        .into_response(),
    );
  }
  let result = user_global_role::Entity::delete_many()
    .filter(user_global_role::Column::UserId.eq(payload.user_id))
    .filter(user_global_role::Column::RoleName.eq(payload.role_name.trim()))
    .exec(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  if result.rows_affected == 0 {
    return Ok(
      (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "assignment not found" })),
      )
        .into_response(),
    );
  }
  Ok(Json(serde_json::json!({ "ok": true })).into_response())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn list_global_role_assignments_handler_exists() {
    let _ = list_global_role_assignments;
  }
}
