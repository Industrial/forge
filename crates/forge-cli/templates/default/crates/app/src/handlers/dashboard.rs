//! Dashboard API: role–permission CRUD and audit log list. List/view require `dashboard.permissions.read`; add/delete require `dashboard.permissions.write`. Audit log is read-only, gated by `dashboard.audit.read`.

use axum::{
  Json,
  extract::{Query, State, Extension},
  http::StatusCode,
  response::{IntoResponse, Response},
};
use chrono::NaiveDateTime;
use forge::token_auth::RequireAuth;
use forge::{DbConnection, Error as ForgeError};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set};
use serde::Deserialize;
use uuid::Uuid;

use db::auth::Backend;
use db::models::{audit_log, organization, role_permission, user};

use crate::handlers::auth::{resolve_permissions, DASHBOARD_PERMISSIONS};

const PERMISSION_READ: &str = "dashboard.permissions.read";
const PERMISSION_WRITE: &str = "dashboard.permissions.write";
const PERMISSION_AUDIT_READ: &str = "dashboard.audit.read";
const PERMISSION_ORGS_READ: &str = "dashboard.organizations.read";
const PERMISSION_ORGS_WRITE: &str = "dashboard.organizations.write";

fn has_permission(permissions: &[String], key: &str) -> bool {
  permissions.iter().any(|p| p == key)
}

/// Returns Some(403 response) if the current user does not have the given permission.
async fn require_permission(
  user: &user::Model,
  db: &DbConnection,
  permission: &str,
) -> Option<Response> {
  let permissions = resolve_permissions(db, user).await;
  if has_permission(&permissions, permission) {
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

/// GET /api/dashboard/permissions — list known permission keys (code-defined). Requires dashboard.permissions.read.
pub async fn list_permissions(
  RequireAuth(user): RequireAuth<Backend>,
  State(db): State<DbConnection>,
) -> Result<impl IntoResponse, ForgeError> {
  if let Some(resp) = require_permission(&user, &db, PERMISSION_READ).await {
    return Ok(resp);
  }
  let list: Vec<&str> = DASHBOARD_PERMISSIONS.to_vec();
  Ok(Json(serde_json::json!({ "permissions": list })).into_response())
}

/// GET /api/dashboard/role-permissions — list role–permission assignments. Requires dashboard.permissions.read. Admin: all; else only scope=org and org_id=current_org_id.
pub async fn list_role_permissions(
  RequireAuth(user): RequireAuth<Backend>,
  State(db): State<DbConnection>,
) -> Result<impl IntoResponse, ForgeError> {
  if let Some(resp) = require_permission(&user, &db, PERMISSION_READ).await {
    return Ok(resp);
  }
  let mut q = role_permission::Entity::find();
  if !user.is_admin {
    let org_id = match user.current_org_id {
      Some(id) => id,
      None => {
        return Ok(Json(serde_json::json!({ "assignments": [] })).into_response());
      }
    };
    q = q
      .filter(role_permission::Column::Scope.eq("org"))
      .filter(role_permission::Column::OrgId.eq(org_id));
  }
  let rows = q.all(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  let list: Vec<serde_json::Value> = rows
    .into_iter()
    .map(|r| {
      serde_json::json!({
        "scope": r.scope,
        "role_name": r.role_name,
        "permission_key": r.permission_key,
        "org_id": r.org_id,
      })
    })
    .collect();
  Ok(Json(serde_json::json!({ "assignments": list })).into_response())
}

/// GET /api/dashboard/tasks — list tasks (ran, running, planned). Requires dashboard. Live updates via WebSocket channel "tasks".
pub async fn list_tasks(
  RequireAuth(user): RequireAuth<Backend>,
  State(db): State<DbConnection>,
  Extension(task_state): Extension<std::sync::Arc<crate::tasks::TaskState>>,
) -> Result<impl IntoResponse, ForgeError> {
  if let Some(resp) = require_permission(&user, &db, "dashboard").await {
    return Ok(resp);
  }
  let tasks = task_state.store.read().await.clone();
  Ok(Json(serde_json::json!({ "tasks": tasks })).into_response())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ListAuditLogQuery {
  pub from: Option<String>,
  pub to: Option<String>,
  pub actor_id: Option<Uuid>,
  pub outcome: Option<String>,
  pub event_kind: Option<String>,
  pub resource_type: Option<String>,
  pub action: Option<String>,
  pub reason: Option<String>,
  #[serde(default = "default_limit")]
  pub limit: u64,
  #[serde(default)]
  pub offset: u64,
}

fn default_limit() -> u64 {
  50
}

/// GET /api/dashboard/audit-log — list audit log entries. Requires dashboard.audit.read. Non-admin: only current org. Supports filters and pagination.
pub async fn list_audit_log(
  RequireAuth(user): RequireAuth<Backend>,
  State(db): State<DbConnection>,
  Query(q): Query<ListAuditLogQuery>,
) -> Result<impl IntoResponse, ForgeError> {
  if let Some(resp) = require_permission(&user, &db, PERMISSION_AUDIT_READ).await {
    return Ok(resp);
  }
  let limit = q.limit.min(200);
  let offset = q.offset;

  let mut query = audit_log::Entity::find();
  if !user.is_admin {
    match user.current_org_id {
      Some(org_id) => {
        query = query.filter(audit_log::Column::OrganizationId.eq(org_id));
      }
      None => {
        return Ok(Json(serde_json::json!({ "entries": [], "total": 0 })).into_response());
      }
    }
  }
  if let Some(ref from) = q.from {
    if let Ok(naive) = NaiveDateTime::parse_from_str(from, "%Y-%m-%dT%H:%M:%S%.fZ") {
      query = query.filter(audit_log::Column::OccurredAt.gte(naive));
    } else if let Ok(naive) = NaiveDateTime::parse_from_str(from, "%Y-%m-%d") {
      query = query.filter(audit_log::Column::OccurredAt.gte(naive));
    }
  }
  if let Some(ref to) = q.to {
    if let Ok(naive) = NaiveDateTime::parse_from_str(to, "%Y-%m-%dT%H:%M:%S%.fZ") {
      query = query.filter(audit_log::Column::OccurredAt.lte(naive));
    } else if let Ok(naive) = NaiveDateTime::parse_from_str(to, "%Y-%m-%d") {
      let end = naive + chrono::Duration::days(1);
      query = query.filter(audit_log::Column::OccurredAt.lt(end));
    }
  }
  if let Some(actor_id) = q.actor_id {
    query = query.filter(audit_log::Column::ActorId.eq(actor_id));
  }
  if let Some(ref outcome) = q.outcome {
    query = query.filter(audit_log::Column::Outcome.eq(outcome.as_str()));
  }
  if let Some(ref event_kind) = q.event_kind {
    query = query.filter(audit_log::Column::EventKind.eq(event_kind.as_str()));
  }
  if let Some(ref resource_type) = q.resource_type {
    query = query.filter(audit_log::Column::ResourceType.eq(resource_type.as_str()));
  }
  if let Some(ref action) = q.action {
    query = query.filter(audit_log::Column::Action.eq(action.as_str()));
  }
  if let Some(ref reason) = q.reason {
    if !reason.is_empty() {
      query = query.filter(audit_log::Column::Reason.contains(reason.as_str()));
    }
  }

  let total = query
    .clone()
    .count(&db)
    .await
    .map_err(|e: sea_orm::DbErr| ForgeError::Generic(e.to_string()))?;
  let rows = query
    .order_by_desc(audit_log::Column::OccurredAt)
    .limit(limit)
    .offset(offset)
    .all(&db)
    .await
    .map_err(|e: sea_orm::DbErr| ForgeError::Generic(e.to_string()))?;

  let entries: Vec<serde_json::Value> = rows
    .into_iter()
    .map(|r| {
      serde_json::json!({
        "id": r.id.to_string(),
        "event_kind": r.event_kind,
        "actor_id": r.actor_id.to_string(),
        "subject_id": r.subject_id.map(|u: Uuid| u.to_string()),
        "organization_id": r.organization_id.map(|u: Uuid| u.to_string()),
        "action": r.action,
        "resource_type": r.resource_type,
        "resource_id": r.resource_id.map(|u: Uuid| u.to_string()),
        "outcome": r.outcome,
        "reason": r.reason,
        "occurred_at": r.occurred_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
      })
    })
    .collect();

  Ok(Json(serde_json::json!({ "entries": entries, "total": total })).into_response())
}

#[derive(Deserialize)]
pub struct AddRolePermissionBody {
  pub scope: String,
  pub role_name: String,
  pub permission_key: String,
  /// Required when scope=org; ignored when non-admin (use current_org_id).
  pub org_id: Option<Uuid>,
}

/// POST /api/dashboard/role-permissions — add one role–permission assignment. Requires dashboard.permissions.write. Admin: any scope/org; else only scope=org and current_org_id.
pub async fn add_role_permission(
  RequireAuth(user): RequireAuth<Backend>,
  State(db): State<DbConnection>,
  Json(payload): Json<AddRolePermissionBody>,
) -> Result<impl IntoResponse, ForgeError> {
  if let Some(resp) = require_permission(&user, &db, PERMISSION_WRITE).await {
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
  let org_id_opt = if user.is_admin {
    if scope == "org" {
      match payload.org_id {
        Some(id) => Some(id),
        None => {
          return Ok(
            (
              StatusCode::UNPROCESSABLE_ENTITY,
              Json(serde_json::json!({ "error": "org_id required when scope=org" })),
            )
              .into_response(),
          );
        }
      }
    } else {
      None
    }
  } else {
    if scope != "org" {
      return Ok(
        (
          StatusCode::FORBIDDEN,
          Json(serde_json::json!({ "error": "only org scope allowed for non-admin" })),
        )
          .into_response(),
      );
    }
    match user.current_org_id {
      Some(id) => Some(id),
      None => {
        return Ok(
          (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "current_org_id required for org scope" })),
          )
            .into_response(),
        );
      }
    }
  };
  let id = Uuid::new_v4();
  let mut model = role_permission::ActiveModel {
    id: Set(id),
    scope: Set(scope.to_string()),
    role_name: Set(role_name.to_string()),
    permission_key: Set(permission_key.to_string()),
    ..Default::default()
  };
  model.org_id = Set(org_id_opt);
  role_permission::Entity::insert(model)
    .exec(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  Ok((StatusCode::CREATED, Json(serde_json::json!({ "ok": true }))).into_response())
}

#[derive(Deserialize)]
pub struct DeleteRolePermissionBody {
  pub scope: String,
  pub role_name: String,
  pub permission_key: String,
  /// Required when scope=org; ignored when non-admin (use current_org_id).
  pub org_id: Option<Uuid>,
}

/// DELETE /api/dashboard/role-permissions — remove one role–permission assignment. Requires dashboard.permissions.write. Admin: any; else only scope=org and current_org_id.
pub async fn delete_role_permission(
  RequireAuth(user): RequireAuth<Backend>,
  State(db): State<DbConnection>,
  Json(payload): Json<DeleteRolePermissionBody>,
) -> Result<impl IntoResponse, ForgeError> {
  if let Some(resp) = require_permission(&user, &db, PERMISSION_WRITE).await {
    return Ok(resp);
  }
  let scope = payload.scope.trim();
  let role_name = payload.role_name.trim();
  let permission_key = payload.permission_key.trim();
  let org_id_opt = if user.is_admin {
    if scope == "org" {
      if payload.org_id.is_none() {
        return Ok(
          (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": "org_id required when scope=org" })),
          )
            .into_response(),
        );
      }
      payload.org_id
    } else {
      None
    }
  } else {
    if scope != "org" {
      return Ok(
        (
          StatusCode::FORBIDDEN,
          Json(serde_json::json!({ "error": "only org scope allowed for non-admin" })),
        )
          .into_response(),
      );
    }
    user.current_org_id
  };
  let mut q = role_permission::Entity::delete_many()
    .filter(role_permission::Column::Scope.eq(scope))
    .filter(role_permission::Column::RoleName.eq(role_name))
    .filter(role_permission::Column::PermissionKey.eq(permission_key));
  match org_id_opt {
    Some(id) => q = q.filter(role_permission::Column::OrgId.eq(id)),
    None => q = q.filter(role_permission::Column::OrgId.is_null()),
  }
  let result = q.exec(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
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
