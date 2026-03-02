//! Dashboard API: role–permission CRUD. All handlers require `dashboard.permissions.manage`.

use axum::{
  Json,
  extract::State,
  http::StatusCode,
  response::{IntoResponse, Response},
};
use forge::token_auth::RequireAuth;
use forge::{DbConnection, Error as ForgeError};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;
use uuid::Uuid;

use db::auth::Backend;
use db::models::{role_permission, user};

use crate::handlers::auth::{resolve_permissions, DASHBOARD_PERMISSIONS};

const PERMISSION_MANAGE: &str = "dashboard.permissions.manage";

/// Returns Some(403 response) if the current user does not have `dashboard.permissions.manage`.
async fn require_permissions_manage(
  user: &user::Model,
  db: &DbConnection,
) -> Option<Response> {
  let permissions = resolve_permissions(db, user).await;
  if permissions.iter().any(|p| p == PERMISSION_MANAGE) {
    return None;
  }
  Some(
    (
      StatusCode::FORBIDDEN,
      Json(serde_json::json!({ "error": "Forbidden" })),
    )
      .into_response(),
  )
}

/// GET /api/dashboard/permissions — list known permission keys (code-defined).
pub async fn list_permissions(
  RequireAuth(user): RequireAuth<Backend>,
  State(db): State<DbConnection>,
) -> Result<impl IntoResponse, ForgeError> {
  if let Some(resp) = require_permissions_manage(&user, &db).await {
    return Ok(resp);
  }
  let list: Vec<&str> = DASHBOARD_PERMISSIONS.to_vec();
  Ok(Json(serde_json::json!({ "permissions": list })).into_response())
}

/// GET /api/dashboard/role-permissions — list all role–permission assignments.
pub async fn list_role_permissions(
  RequireAuth(user): RequireAuth<Backend>,
  State(db): State<DbConnection>,
) -> Result<impl IntoResponse, ForgeError> {
  if let Some(resp) = require_permissions_manage(&user, &db).await {
    return Ok(resp);
  }
  let rows = role_permission::Entity::find()
    .all(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  let list: Vec<serde_json::Value> = rows
    .into_iter()
    .map(|r| {
      serde_json::json!({
        "scope": r.scope,
        "role_name": r.role_name,
        "permission_key": r.permission_key,
      })
    })
    .collect();
  Ok(Json(serde_json::json!({ "assignments": list })).into_response())
}

#[derive(Deserialize)]
pub struct AddRolePermissionBody {
  pub scope: String,
  pub role_name: String,
  pub permission_key: String,
}

/// POST /api/dashboard/role-permissions — add one role–permission assignment.
pub async fn add_role_permission(
  RequireAuth(user): RequireAuth<Backend>,
  State(db): State<DbConnection>,
  Json(payload): Json<AddRolePermissionBody>,
) -> Result<impl IntoResponse, ForgeError> {
  if let Some(resp) = require_permissions_manage(&user, &db).await {
    return Ok(resp);
  }
  let scope = payload.scope.trim();
  let role_name = payload.role_name.trim();
  let permission_key = payload.permission_key.trim();
  if scope != "org" && scope != "global" {
    return Ok(
      (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(serde_json::json!({ "error": "scope must be 'org' or 'global'" })),
      )
        .into_response(),
    );
  }
  if !DASHBOARD_PERMISSIONS.contains(&permission_key) {
    return Ok(
      (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(serde_json::json!({ "error": "invalid permission_key" })),
      )
        .into_response(),
    );
  }
  let id = Uuid::new_v4();
  let model = role_permission::ActiveModel {
    id: Set(id),
    scope: Set(scope.to_string()),
    role_name: Set(role_name.to_string()),
    permission_key: Set(permission_key.to_string()),
    ..Default::default()
  };
  role_permission::Entity::insert(model).exec(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  Ok((StatusCode::CREATED, Json(serde_json::json!({ "ok": true }))).into_response())
}

#[derive(Deserialize)]
pub struct DeleteRolePermissionBody {
  pub scope: String,
  pub role_name: String,
  pub permission_key: String,
}

/// DELETE /api/dashboard/role-permissions — remove one role–permission assignment (by composite key).
pub async fn delete_role_permission(
  RequireAuth(user): RequireAuth<Backend>,
  State(db): State<DbConnection>,
  Json(payload): Json<DeleteRolePermissionBody>,
) -> Result<impl IntoResponse, ForgeError> {
  if let Some(resp) = require_permissions_manage(&user, &db).await {
    return Ok(resp);
  }
  let result = role_permission::Entity::delete_many()
    .filter(role_permission::Column::Scope.eq(payload.scope.trim()))
    .filter(role_permission::Column::RoleName.eq(payload.role_name.trim()))
    .filter(role_permission::Column::PermissionKey.eq(payload.permission_key.trim()))
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
