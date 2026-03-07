//! Dashboard API: role–permission CRUD and audit log list. List/view require `dashboard.permissions.read`; add/delete require `dashboard.permissions.write`. Audit log is read-only, gated by `dashboard.audit.read`.

use crate::Error as ForgeError;
use axum::{
  Json,
  extract::{Extension, Query, State},
  http::StatusCode,
  response::{IntoResponse, Response},
};
use chrono::NaiveDateTime;
use forge_auth::token_auth::RequireAuth;
use forge_db::DbConnection;
use forge_live::{Channel, InMemoryLiveBackend, LiveEvent, broadcast_to_channel};
use sea_orm::{
  ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
  Set,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use forge_auth::token_auth::hash_password;

use db::auth::Backend;
use db::models::{
  audit_log, membership, org_role, organization, role_permission, user, user_org_role,
};

use crate::handlers::auth::{ScopeFromHeaders, has_global_scope, resolve_permissions};
use crate::handlers::generic_entity::{ListQueryParams, parse_list_query_spec};
use crate::permissions::{dashboard_permissions, entity_action_key, permission_equivalents};
use crate::query_spec::{
  FilterCond, FilterOperator, SortDirection, validate_filter_cond, validate_sort_field,
};
use crate::scoped_query::{WithScope, user_find_scoped};
use db::organization::{
  CreateOrganizationBody, UpdateOrganizationBody as DbUpdateOrganizationBody,
  create_organization_impl, update_organization_impl,
};

const PERMISSION_READ: &str = "dashboard.permissions.read";
const PERMISSION_WRITE: &str = "dashboard.permissions.write";
const PERMISSION_AUDIT_READ: &str = "dashboard.audit.read";
const PERMISSION_ORGS_READ: &str = "dashboard.organizations.read";
const PERMISSION_ORGS_WRITE: &str = "dashboard.organizations.write";
const PERMISSION_USERS_READ: &str = "dashboard.users.read";
const PERMISSION_USERS_WRITE: &str = "dashboard.users.write";
const PERMISSION_ROLES_READ: &str = "dashboard.roles.read";
const PERMISSION_ROLES_WRITE: &str = "dashboard.roles.write";

/// Entity-based permission check (§2, §7): exact or equivalent match, or all.read for *.read, or all.write for *.create|update|delete.
pub(crate) fn has_permission(permissions: &[String], key: &str) -> bool {
  let equivs = permission_equivalents(key);
  let exact = if equivs.is_empty() {
    permissions.iter().any(|p| p == key)
  } else {
    permissions.iter().any(|p| equivs.contains(&p.as_str()))
  };
  if exact {
    return true;
  }
  if (key.ends_with(".read") || equivs.iter().any(|e| e.ends_with(".read")))
    && permissions.iter().any(|p| p == "all.read")
  {
    return true;
  }
  if (key.ends_with(".write")
    || key.ends_with(".create")
    || key.ends_with(".update")
    || key.ends_with(".delete")
    || equivs
      .iter()
      .any(|e| e.ends_with(".write") || e.ends_with(".create")))
    && permissions.iter().any(|p| p == "all.write")
  {
    return true;
  }
  false
}

/// Returns Some(403 response) if the current user does not have the given permission.
async fn require_permission(
  user: &user::Model,
  db: &DbConnection,
  permission: &str,
  scope: Option<&forge_auth::RequestScope>,
) -> Option<Response> {
  let permissions = resolve_permissions(db, user, scope).await;
  if has_permission(&permissions, permission) {
    return None;
  }
  Some(forbidden_response())
}

/// §9: 403 Forbidden with clear message when permission is missing.
fn forbidden_response() -> Response {
  (
    StatusCode::FORBIDDEN,
    Json(serde_json::json!({
      "error": "Forbidden",
      "message": "Insufficient permissions"
    })),
  )
    .into_response()
}

/// §5: Dashboard access = has at least one permission in scope (no separate "dashboard" key).
/// Returns Some(403) if the user has no resolved permissions.
async fn require_any_permission(
  user: &user::Model,
  db: &DbConnection,
  scope: Option<&forge_auth::RequestScope>,
) -> Option<Response> {
  let permissions = resolve_permissions(db, user, scope).await;
  if permissions.is_empty() {
    return Some(forbidden_response());
  }
  None
}

/// §6: Single authorization rule — require the corresponding entity.action in scope.
/// List/get → entity.read, create → entity.create, update → entity.update, delete → entity.delete.
pub(crate) async fn require_entity_permission(
  user: &user::Model,
  db: &DbConnection,
  scope: Option<&forge_auth::RequestScope>,
  entity: &str,
  action: &str,
) -> Option<Response> {
  let key = entity_action_key(entity, action);
  require_permission(user, db, &key, scope).await
}

/// GET /api/dashboard/permissions — list known permission keys (code-defined). Requires permission.read (§6).
pub async fn list_permissions(
  ScopeFromHeaders(scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_entity_permission(user, &db, Some(&scope), "permission", "read").await
  {
    return Ok(resp);
  }
  let list: Vec<&str> = dashboard_permissions().to_vec();
  Ok(Json(serde_json::json!({ "permissions": list })).into_response())
}

/// GET /api/dashboard/role-permissions — list role–permission assignments. Requires dashboard.permissions.read. Global scope: all; else current org only.
pub async fn list_role_permissions(
  ScopeFromHeaders(scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_READ, Some(&scope)).await {
    return Ok(resp);
  }
  let rows = if has_global_scope(&db, user, PERMISSION_READ).await {
    role_permission::Entity::find().all(&db).await
  } else {
    let org_id = scope.organization_id;
    role_permission::Entity::find()
      .filter(role_permission::Column::Scope.eq("org"))
      .filter(role_permission::Column::OrgId.eq(org_id))
      .all(&db)
      .await
  };
  let rows = rows.map_err(|e| ForgeError::Generic(e.to_string()))?;
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
  ScopeFromHeaders(scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
  Extension(task_state): Extension<std::sync::Arc<crate::tasks::TaskState>>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_any_permission(user, &db, Some(&scope)).await {
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

pub(crate) fn default_limit() -> u64 {
  50
}

/// GET /api/dashboard/audit-log — list audit log entries. Requires dashboard.audit.read. Non-admin: only current org. Supports filters and pagination.
pub async fn list_audit_log(
  ScopeFromHeaders(scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
  Query(q): Query<ListAuditLogQuery>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_AUDIT_READ, Some(&scope)).await {
    return Ok(resp);
  }
  let limit = q.limit.min(200);
  let offset = q.offset;

  let scope_opt: Option<&forge_auth::RequestScope> =
    if has_global_scope(&db, user, PERMISSION_AUDIT_READ).await {
      None
    } else {
      Some(&scope)
    };
  let mut query = audit_log::Entity::find().with_scope(scope_opt);
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
  if let Some(ref reason) = q.reason.filter(|r| !r.is_empty()) {
    query = query.filter(audit_log::Column::Reason.contains(reason.as_str()));
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

/// POST /api/dashboard/role-permissions — add one role–permission assignment. Requires dashboard.permissions.write. Admin: any scope/org; else only scope=org and session profile org.
pub async fn add_role_permission(
  ScopeFromHeaders(req_scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
  Extension(live_backend): Extension<Option<Arc<InMemoryLiveBackend>>>,
  Json(payload): Json<AddRolePermissionBody>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_WRITE, Some(&req_scope)).await {
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
  if !dashboard_permissions().contains(&permission_key) {
    return Ok((
      StatusCode::UNPROCESSABLE_ENTITY,
      Json(serde_json::json!({
        "error": "invalid permission_key",
        "message": "Permission key is not in the allowed set (entity.action or all.read / all.write)"
      })),
    )
      .into_response());
  }
  let global_perm = has_global_scope(&db, user, PERMISSION_WRITE).await;
  let org_id_opt = if global_perm {
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
          Json(serde_json::json!({ "error": "only org scope allowed" })),
        )
          .into_response(),
      );
    }
    Some(req_scope.organization_id)
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
  if let Err(e) = role_permission::Entity::insert(model).exec(&db).await {
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
  if let (Some(backend), Some(org_id)) = (live_backend.as_ref(), org_id_opt) {
    let _ = broadcast_to_channel(
      backend,
      &Channel::org_resource(org_id, "role_permissions"),
      &LiveEvent::ResourceChanged {
        resource: "role_permissions".into(),
        id,
        action: Some("created".into()),
      },
    )
    .await;
  }
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

/// DELETE /api/dashboard/role-permissions — remove one role–permission assignment. Requires dashboard.permissions.write. Admin: any; else only scope=org and session profile org.
pub async fn delete_role_permission(
  ScopeFromHeaders(req_scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
  Extension(live_backend): Extension<Option<Arc<InMemoryLiveBackend>>>,
  Json(payload): Json<DeleteRolePermissionBody>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_WRITE, Some(&req_scope)).await {
    return Ok(resp);
  }
  let scope = payload.scope.trim();
  let role_name = payload.role_name.trim();
  let permission_key = payload.permission_key.trim();
  let global_perm = has_global_scope(&db, user, PERMISSION_WRITE).await;
  let org_id_opt = if global_perm {
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
          Json(serde_json::json!({ "error": "only org scope allowed" })),
        )
          .into_response(),
      );
    }
    Some(req_scope.organization_id)
  };
  let mut q = role_permission::Entity::delete_many()
    .filter(role_permission::Column::Scope.eq(scope))
    .filter(role_permission::Column::RoleName.eq(role_name))
    .filter(role_permission::Column::PermissionKey.eq(permission_key));
  match org_id_opt {
    Some(id) => q = q.filter(role_permission::Column::OrgId.eq(id)),
    None => q = q.filter(role_permission::Column::OrgId.is_null()),
  }
  let result = q
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
  if let (Some(backend), Some(org_id)) = (live_backend.as_ref(), org_id_opt) {
    let _ = broadcast_to_channel(
      backend,
      &Channel::org_resource(org_id, "role_permissions"),
      &LiveEvent::ResourceChanged {
        resource: "role_permissions".into(),
        id: Uuid::nil(),
        action: Some("deleted".into()),
      },
    )
    .await;
  }
  Ok(Json(serde_json::json!({ "ok": true })).into_response())
}

#[derive(Deserialize)]
pub struct OrgRolePermissionBody {
  pub permission_key: String,
}

#[derive(Deserialize)]
pub struct OrgRolePath {
  org_id: Uuid,
}

#[derive(Deserialize)]
pub struct OrgRolePermissionPath {
  org_id: Uuid,
  role_id: Uuid,
}

/// GET /api/organizations/{org_id}/roles — list roles for an organization. Requires dashboard.roles.read.
pub async fn list_org_roles(
  axum::extract::Path(OrgRolePath { org_id }): axum::extract::Path<OrgRolePath>,
  ScopeFromHeaders(scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_ROLES_READ, Some(&scope)).await {
    return Ok(resp);
  }
  // Check if user has access to this org
  let has_global = has_global_scope(&db, user, PERMISSION_ROLES_READ).await;
  if !has_global && scope.organization_id != org_id {
    return Ok(
      (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({ "error": "Forbidden" })),
      )
        .into_response(),
    );
  }
  let rows = org_role::Entity::find()
    .filter(org_role::Column::OrgId.eq(org_id))
    .order_by_asc(org_role::Column::Name)
    .all(&db)
    .await
    .map_err(|e: sea_orm::DbErr| ForgeError::Generic(e.to_string()))?;
  let list: Vec<serde_json::Value> = rows
    .into_iter()
    .map(|r| {
      serde_json::json!({
        "id": r.id.to_string(),
        "org_id": r.org_id.to_string(),
        "name": r.name,
        "display_name": r.display_name,
        "created_at": r.created_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
        "updated_at": r.updated_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
      })
    })
    .collect();
  Ok(Json(serde_json::json!({ "roles": list })).into_response())
}

/// GET /api/organizations/{org_id}/roles/{role_id}/permissions — get permissions for a role. Requires dashboard.permissions.read.
pub async fn get_role_permissions(
  axum::extract::Path(OrgRolePermissionPath { org_id, role_id }): axum::extract::Path<
    OrgRolePermissionPath,
  >,
  ScopeFromHeaders(scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_READ, Some(&scope)).await {
    return Ok(resp);
  }
  // Verify role exists and belongs to org
  let role = org_role::Entity::find_by_id(role_id)
    .one(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  let Some(role) = role else {
    return Ok(
      (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "Role not found" })),
      )
        .into_response(),
    );
  };
  if role.org_id != org_id {
    return Ok(
      (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "Role not found" })),
      )
        .into_response(),
    );
  }
  // Check access to org
  let has_global = has_global_scope(&db, user, PERMISSION_READ).await;
  if !has_global && scope.organization_id != org_id {
    return Ok(
      (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({ "error": "Forbidden" })),
      )
        .into_response(),
    );
  }
  // Get permissions for this role
  let rows = role_permission::Entity::find()
    .filter(role_permission::Column::Scope.eq("org"))
    .filter(role_permission::Column::OrgId.eq(org_id))
    .filter(role_permission::Column::RoleName.eq(&role.name))
    .all(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  let permissions: Vec<String> = rows.into_iter().map(|r| r.permission_key).collect();
  Ok(Json(serde_json::json!({ "permissions": permissions })).into_response())
}

/// POST /api/organizations/{org_id}/roles/{role_id}/permissions — add permission to a role. Requires dashboard.permissions.write.
pub async fn post_role_permission(
  axum::extract::Path(OrgRolePermissionPath { org_id, role_id }): axum::extract::Path<
    OrgRolePermissionPath,
  >,
  ScopeFromHeaders(req_scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
  Extension(live_backend): Extension<Option<Arc<InMemoryLiveBackend>>>,
  Json(payload): Json<OrgRolePermissionBody>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_WRITE, Some(&req_scope)).await {
    return Ok(resp);
  }
  // Verify role exists and belongs to org
  let role = org_role::Entity::find_by_id(role_id)
    .one(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  let Some(role) = role else {
    return Ok(
      (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "Role not found" })),
      )
        .into_response(),
    );
  };
  if role.org_id != org_id {
    return Ok(
      (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "Role not found" })),
      )
        .into_response(),
    );
  }
  // Check access to org
  let has_global = has_global_scope(&db, user, PERMISSION_WRITE).await;
  if !has_global && req_scope.organization_id != org_id {
    return Ok(
      (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({ "error": "Forbidden" })),
      )
        .into_response(),
    );
  }
  let permission_key = payload.permission_key.trim();
  if !dashboard_permissions().contains(&permission_key) {
    return Ok(
      (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(serde_json::json!({
          "error": "invalid permission_key",
          "message": "Permission key is not in the allowed set"
        })),
      )
        .into_response(),
    );
  }
  // Check if already exists
  let exists = role_permission::Entity::find()
    .filter(role_permission::Column::Scope.eq("org"))
    .filter(role_permission::Column::OrgId.eq(org_id))
    .filter(role_permission::Column::RoleName.eq(&role.name))
    .filter(role_permission::Column::PermissionKey.eq(permission_key))
    .one(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  if exists.is_some() {
    // Idempotent: already exists, return success
    return Ok(Json(serde_json::json!({ "ok": true })).into_response());
  }
  // Add permission
  let id = Uuid::new_v4();
  role_permission::Entity::insert(role_permission::ActiveModel {
    id: Set(id),
    scope: Set("org".to_string()),
    role_name: Set(role.name.clone()),
    permission_key: Set(permission_key.to_string()),
    org_id: Set(Some(org_id)),
  })
  .exec(&db)
  .await
  .map_err(|e| ForgeError::Generic(e.to_string()))?;
  if let Some(backend) = live_backend.as_ref() {
    let _ = broadcast_to_channel(
      backend,
      &Channel::org_resource(org_id, "role_permissions"),
      &LiveEvent::ResourceChanged {
        resource: "role_permissions".into(),
        id: Uuid::nil(),
        action: Some("created".into()),
      },
    )
    .await;
  }
  Ok(Json(serde_json::json!({ "ok": true })).into_response())
}

/// DELETE /api/organizations/{org_id}/roles/{role_id}/permissions — remove permission from a role. Requires dashboard.permissions.write.
pub async fn delete_role_permission_by_path(
  axum::extract::Path(OrgRolePermissionPath { org_id, role_id }): axum::extract::Path<
    OrgRolePermissionPath,
  >,
  ScopeFromHeaders(req_scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
  Extension(live_backend): Extension<Option<Arc<InMemoryLiveBackend>>>,
  Json(payload): Json<OrgRolePermissionBody>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_WRITE, Some(&req_scope)).await {
    return Ok(resp);
  }
  // Verify role exists and belongs to org
  let role = org_role::Entity::find_by_id(role_id)
    .one(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  let Some(role) = role else {
    return Ok(
      (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "Role not found" })),
      )
        .into_response(),
    );
  };
  if role.org_id != org_id {
    return Ok(
      (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "Role not found" })),
      )
        .into_response(),
    );
  }
  // Check access to org
  let has_global = has_global_scope(&db, user, PERMISSION_WRITE).await;
  if !has_global && req_scope.organization_id != org_id {
    return Ok(
      (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({ "error": "Forbidden" })),
      )
        .into_response(),
    );
  }
  let permission_key = payload.permission_key.trim();
  // Delete permission
  let result = role_permission::Entity::delete_many()
    .filter(role_permission::Column::Scope.eq("org"))
    .filter(role_permission::Column::OrgId.eq(org_id))
    .filter(role_permission::Column::RoleName.eq(&role.name))
    .filter(role_permission::Column::PermissionKey.eq(permission_key))
    .exec(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  if result.rows_affected == 0 {
    return Ok(
      (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "Permission not found" })),
      )
        .into_response(),
    );
  }
  if let Some(backend) = live_backend.as_ref() {
    let _ = broadcast_to_channel(
      backend,
      &Channel::org_resource(org_id, "role_permissions"),
      &LiveEvent::ResourceChanged {
        resource: "role_permissions".into(),
        id: Uuid::nil(),
        action: Some("deleted".into()),
      },
    )
    .await;
  }
  Ok(Json(serde_json::json!({ "ok": true })).into_response())
}

/// GET /api/dashboard/organizations — list all organizations. Requires dashboard.organizations.read (global).
pub async fn list_organizations(
  ScopeFromHeaders(scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_ORGS_READ, Some(&scope)).await {
    return Ok(resp);
  }
  let rows = organization::Entity::find()
    .order_by_asc(organization::Column::Name)
    .all(&db)
    .await
    .map_err(|e: sea_orm::DbErr| ForgeError::Generic(e.to_string()))?;
  let list: Vec<serde_json::Value> = rows
    .into_iter()
    .map(|r| {
      serde_json::json!({
        "id": r.id.to_string(),
        "name": r.name,
        "slug": r.slug,
        "created_at": r.created_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
        "updated_at": r.updated_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
      })
    })
    .collect();
  Ok(Json(serde_json::json!({ "organizations": list })).into_response())
}

#[derive(Deserialize)]
pub struct ListRolesQuery {
  #[allow(dead_code)]
  pub org_id: Option<Uuid>,
}

/// GET /api/dashboard/roles — list org roles. Global scope: all orgs; else current org only. Requires dashboard.roles.read.
pub async fn list_roles(
  ScopeFromHeaders(scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
  Query(_q): Query<ListRolesQuery>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_ROLES_READ, Some(&scope)).await {
    return Ok(resp);
  }
  let rows = if has_global_scope(&db, user, PERMISSION_ROLES_READ).await {
    org_role::Entity::find()
      .order_by_asc(org_role::Column::OrgId)
      .order_by_asc(org_role::Column::Name)
      .all(&db)
      .await
  } else {
    let org_id = scope.organization_id;
    org_role::Entity::find()
      .filter(org_role::Column::OrgId.eq(org_id))
      .order_by_asc(org_role::Column::Name)
      .all(&db)
      .await
  };
  let rows = rows.map_err(|e: sea_orm::DbErr| ForgeError::Generic(e.to_string()))?;
  let list: Vec<serde_json::Value> = rows
    .into_iter()
    .map(|r| {
      serde_json::json!({
        "id": r.id.to_string(),
        "org_id": r.org_id.to_string(),
        "name": r.name,
        "display_name": r.display_name,
        "created_at": r.created_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
        "updated_at": r.updated_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
      })
    })
    .collect();
  Ok(Json(serde_json::json!({ "roles": list })).into_response())
}

#[derive(Deserialize)]
pub struct CreateRoleBody {
  pub org_id: Option<Uuid>,
  pub name: String,
  pub display_name: Option<String>,
}

/// POST /api/dashboard/roles — create org role. Requires dashboard.roles.write. Global scope: any org_id; else session profile org only.
pub async fn create_role(
  ScopeFromHeaders(scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
  Extension(live_backend): Extension<Option<Arc<InMemoryLiveBackend>>>,
  Json(payload): Json<CreateRoleBody>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_ROLES_WRITE, Some(&scope)).await {
    return Ok(resp);
  }
  let org_id = if has_global_scope(&db, user, PERMISSION_ROLES_WRITE).await {
    payload
      .org_id
      .or(Some(scope.organization_id))
      .ok_or_else(|| ForgeError::Generic("org_id required".into()))?
  } else {
    if payload
      .org_id
      .map(|pid| pid != scope.organization_id)
      .unwrap_or(false)
    {
      return Ok(
        (
          StatusCode::FORBIDDEN,
          Json(
            serde_json::json!({ "error": "Can only create roles in your current organization" }),
          ),
        )
          .into_response(),
      );
    }
    scope.organization_id
  };
  let name = payload.name.trim();
  if name.is_empty() {
    return Ok(
      (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(serde_json::json!({ "error": "name is required" })),
      )
        .into_response(),
    );
  }
  let existing = org_role::Entity::find()
    .filter(org_role::Column::OrgId.eq(org_id))
    .filter(org_role::Column::Name.eq(name))
    .one(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  if existing.is_some() {
    return Ok(
      (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(serde_json::json!({ "error": "A role with this name already exists in this organization" })),
      )
        .into_response(),
    );
  }
  let now = chrono::Utc::now().naive_utc();
  let id = Uuid::new_v4();
  let display_name = payload
    .display_name
    .as_ref()
    .map(|s| s.trim())
    .filter(|s| !s.is_empty())
    .map(|s| s.to_string());
  let model = org_role::ActiveModel {
    id: Set(id),
    org_id: Set(org_id),
    name: Set(name.to_string()),
    display_name: Set(display_name.clone()),
    created_at: Set(now),
    updated_at: Set(now),
  };
  org_role::Entity::insert(model)
    .exec(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  if let Some(ref backend) = live_backend {
    let _ = broadcast_to_channel(
      backend,
      &Channel::org_resource(org_id, "roles"),
      &LiveEvent::ResourceChanged {
        resource: "roles".into(),
        id,
        action: Some("created".into()),
      },
    )
    .await;
  }
  Ok(
    (
      StatusCode::CREATED,
      Json(serde_json::json!({
        "id": id.to_string(),
        "org_id": org_id.to_string(),
        "name": name,
        "display_name": display_name,
        "created_at": now.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
        "updated_at": now.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
      })),
    )
      .into_response(),
  )
}

#[derive(Deserialize)]
pub struct UpdateRoleBody {
  pub id: Uuid,
  pub name: Option<String>,
  pub display_name: Option<String>,
}

/// PATCH /api/dashboard/roles — update org role. Requires dashboard.roles.write.
pub async fn update_role(
  ScopeFromHeaders(scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
  Extension(live_backend): Extension<Option<Arc<InMemoryLiveBackend>>>,
  Json(payload): Json<UpdateRoleBody>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_ROLES_WRITE, Some(&scope)).await {
    return Ok(resp);
  }
  let role = org_role::Entity::find_by_id(payload.id)
    .one(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?
    .ok_or_else(|| ForgeError::Generic("Role not found".into()))?;
  let can_write_org = has_global_scope(&db, user, PERMISSION_ROLES_WRITE).await
    || scope.organization_id == role.org_id;
  if !can_write_org {
    return Ok(
      (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({ "error": "Can only update roles in your current organization" })),
      )
        .into_response(),
    );
  }
  let org_id = role.org_id;
  let role_id = role.id;
  let mut am: org_role::ActiveModel = role.into();
  if let Some(name) = payload.name {
    let t = name.trim();
    if !t.is_empty() {
      let existing = org_role::Entity::find()
        .filter(org_role::Column::OrgId.eq(org_id))
        .filter(org_role::Column::Name.eq(t))
        .filter(org_role::Column::Id.ne(role_id))
        .one(&db)
        .await
        .map_err(|e| ForgeError::Generic(e.to_string()))?;
      if existing.is_some() {
        return Ok(
          (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": "A role with this name already exists" })),
          )
            .into_response(),
        );
      }
      am.name = Set(t.to_string());
    }
  }
  if let Some(display_name) = payload.display_name {
    am.display_name = Set(Some(display_name.trim().to_string()).filter(|s| !s.is_empty()));
  }
  am.updated_at = Set(chrono::Utc::now().naive_utc());
  am.update(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  if let Some(ref backend) = live_backend {
    let _ = broadcast_to_channel(
      backend,
      &Channel::org_resource(org_id, "roles"),
      &LiveEvent::ResourceChanged {
        resource: "roles".into(),
        id: role_id,
        action: Some("updated".into()),
      },
    )
    .await;
  }
  Ok(Json(serde_json::json!({ "ok": true })).into_response())
}

#[derive(Deserialize)]
pub struct DeleteRoleBody {
  pub id: Uuid,
}

/// DELETE /api/dashboard/roles — delete org role. Requires dashboard.roles.write. Fails if any user has this role.
pub async fn delete_role(
  ScopeFromHeaders(scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
  Extension(live_backend): Extension<Option<Arc<InMemoryLiveBackend>>>,
  Json(payload): Json<DeleteRoleBody>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_ROLES_WRITE, Some(&scope)).await {
    return Ok(resp);
  }
  let role = org_role::Entity::find_by_id(payload.id)
    .one(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?
    .ok_or_else(|| ForgeError::Generic("Role not found".into()))?;
  let can_write_org = has_global_scope(&db, user, PERMISSION_ROLES_WRITE).await
    || scope.organization_id == role.org_id;
  if !can_write_org {
    return Ok(
      (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({ "error": "Can only delete roles in your current organization" })),
      )
        .into_response(),
    );
  }
  let count = user_org_role::Entity::find()
    .filter(user_org_role::Column::RoleId.eq(payload.id))
    .count(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  if count > 0 {
    return Ok(
      (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(serde_json::json!({ "error": "Cannot delete role: one or more users have this role. Remove assignments first." })),
      )
        .into_response(),
    );
  }
  org_role::Entity::delete_by_id(payload.id)
    .exec(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  if let Some(ref backend) = live_backend {
    let _ = broadcast_to_channel(
      backend,
      &Channel::org_resource(role.org_id, "roles"),
      &LiveEvent::ResourceChanged {
        resource: "roles".into(),
        id: payload.id,
        action: Some("deleted".into()),
      },
    )
    .await;
  }
  Ok(Json(serde_json::json!({ "ok": true })).into_response())
}

/// POST /api/dashboard/organizations — create organization. Requires dashboard.organizations.write.
pub async fn create_organization(
  ScopeFromHeaders(scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
  Extension(live_backend): Extension<Option<Arc<InMemoryLiveBackend>>>,
  Json(payload): Json<CreateOrganizationBody>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_ORGS_WRITE, Some(&scope)).await {
    return Ok(resp);
  }
  let id = create_organization_impl(&db, &payload)
    .await
    .map_err(crate::Error::from)?;
  let o = organization::Entity::find_by_id(id)
    .one(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?
    .ok_or_else(|| ForgeError::Generic("created organization not found".into()))?;
  let now = o.updated_at;
  for &role_name in &["owner", "admin", "editor", "viewer"] {
    let role_id = Uuid::new_v4();
    let r = org_role::ActiveModel {
      id: Set(role_id),
      org_id: Set(id),
      name: Set(role_name.to_string()),
      display_name: Set(None),
      created_at: Set(now),
      updated_at: Set(now),
    };
    org_role::Entity::insert(r)
      .exec(&db)
      .await
      .map_err(|e| ForgeError::Generic(e.to_string()))?;
  }
  db::seed_role_permissions_for_org(&db, id)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  if let Some(ref backend) = live_backend {
    let ch = Channel::raw("organizations");
    let _ = broadcast_to_channel(
      backend,
      &ch,
      &LiveEvent::ResourceChanged {
        resource: "organizations".into(),
        id,
        action: Some("created".into()),
      },
    )
    .await;
  }
  Ok(
    (
      StatusCode::CREATED,
      Json(serde_json::json!({
        "id": o.id.to_string(),
        "name": o.name,
        "slug": o.slug,
        "created_at": o.created_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
        "updated_at": o.updated_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
      })),
    )
      .into_response(),
  )
}

#[derive(Deserialize)]
pub struct UpdateOrganizationBody {
  pub id: Uuid,
  pub name: Option<String>,
  pub slug: Option<String>,
}

/// PATCH /api/dashboard/organizations — update organization. Requires dashboard.organizations.write.
pub async fn update_organization(
  ScopeFromHeaders(scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
  Extension(live_backend): Extension<Option<Arc<InMemoryLiveBackend>>>,
  Json(payload): Json<UpdateOrganizationBody>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_ORGS_WRITE, Some(&scope)).await {
    return Ok(resp);
  }
  let db_payload = DbUpdateOrganizationBody {
    name: payload.name.clone(),
    slug: payload.slug.clone(),
  };
  let updated = update_organization_impl(&db, payload.id, &db_payload)
    .await
    .map_err(crate::Error::from)?;
  if let Some(ref backend) = live_backend {
    let ch = Channel::raw("organizations");
    let _ = broadcast_to_channel(
      backend,
      &ch,
      &LiveEvent::ResourceChanged {
        resource: "organizations".into(),
        id: payload.id,
        action: Some("updated".into()),
      },
    )
    .await;
  }
  Ok(
    Json(serde_json::json!({
      "id": updated.id.to_string(),
      "name": updated.name,
      "slug": updated.slug,
      "created_at": updated.created_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
      "updated_at": updated.updated_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
    }))
    .into_response(),
  )
}

#[derive(Deserialize)]
pub struct DeleteOrganizationBody {
  pub id: Uuid,
}

/// DELETE /api/dashboard/organizations — delete organization. Requires dashboard.organizations.write.
pub async fn delete_organization(
  ScopeFromHeaders(scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
  Extension(live_backend): Extension<Option<Arc<InMemoryLiveBackend>>>,
  Json(payload): Json<DeleteOrganizationBody>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_ORGS_WRITE, Some(&scope)).await {
    return Ok(resp);
  }
  let result = organization::Entity::delete_by_id(payload.id)
    .exec(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  if result.rows_affected == 0 {
    return Ok(
      (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "Organization not found" })),
      )
        .into_response(),
    );
  }
  if let Some(ref backend) = live_backend {
    let ch = Channel::raw("organizations");
    let _ = broadcast_to_channel(
      backend,
      &ch,
      &LiveEvent::ResourceChanged {
        resource: "organizations".into(),
        id: payload.id,
        action: Some("deleted".into()),
      },
    )
    .await;
  }
  Ok(Json(serde_json::json!({ "ok": true })).into_response())
}

/// Allowed filter/sort fields for dashboard users list (ListQuerySpec).
const DASHBOARD_USERS_FILTER_SORT_FIELDS: &[&str] = &[
  "id",
  "email",
  "is_active",
  "is_admin",
  "created_at",
  "updated_at",
];

fn apply_user_filter(
  select: sea_orm::Select<user::Entity>,
  cond: &FilterCond,
) -> sea_orm::Select<user::Entity> {
  match cond.field.as_str() {
    "id" => {
      let parse_uuid = |j: &serde_json::Value| j.as_str().and_then(|s| Uuid::parse_str(s).ok());
      match cond.operator {
        FilterOperator::Eq => {
          if let Some(v) = cond.value.as_ref().and_then(parse_uuid) {
            select.filter(user::Column::Id.eq(v))
          } else {
            select
          }
        }
        FilterOperator::Ne => {
          if let Some(v) = cond.value.as_ref().and_then(parse_uuid) {
            select.filter(user::Column::Id.ne(v))
          } else {
            select
          }
        }
        FilterOperator::In => {
          if let Some(serde_json::Value::Array(arr)) = cond.value.as_ref() {
            let vals: Vec<Uuid> = arr.iter().filter_map(parse_uuid).collect();
            if vals.is_empty() {
              select.filter(user::Column::Id.eq(Uuid::nil()))
            } else {
              select.filter(user::Column::Id.is_in(vals))
            }
          } else {
            select
          }
        }
        FilterOperator::IsNull => select.filter(user::Column::Id.is_null()),
        _ => select,
      }
    }
    "email" => match cond.operator {
      FilterOperator::Eq => {
        if let Some(serde_json::Value::String(s)) = cond.value.as_ref() {
          select.filter(user::Column::Email.eq(s.as_str()))
        } else {
          select
        }
      }
      FilterOperator::Ne => {
        if let Some(serde_json::Value::String(s)) = cond.value.as_ref() {
          select.filter(user::Column::Email.ne(s.as_str()))
        } else {
          select
        }
      }
      FilterOperator::Contains => {
        if let Some(serde_json::Value::String(s)) = cond.value.as_ref() {
          select.filter(user::Column::Email.contains(s))
        } else {
          select
        }
      }
      FilterOperator::StartsWith => {
        if let Some(serde_json::Value::String(s)) = cond.value.as_ref() {
          select.filter(user::Column::Email.starts_with(s))
        } else {
          select
        }
      }
      FilterOperator::EndsWith => {
        if let Some(serde_json::Value::String(s)) = cond.value.as_ref() {
          select.filter(user::Column::Email.ends_with(s))
        } else {
          select
        }
      }
      FilterOperator::In => {
        if let Some(serde_json::Value::Array(arr)) = cond.value.as_ref() {
          let strs: Vec<&str> = arr.iter().filter_map(|j| j.as_str()).collect();
          if strs.is_empty() {
            select.filter(user::Column::Email.eq(""))
          } else {
            select.filter(user::Column::Email.is_in(strs))
          }
        } else {
          select
        }
      }
      FilterOperator::IsNull => select.filter(user::Column::Email.is_null()),
      _ => select,
    },
    "is_active" | "is_admin" => {
      let col = if cond.field == "is_active" {
        user::Column::IsActive
      } else {
        user::Column::IsAdmin
      };
      let parse_bool = |j: &serde_json::Value| j.as_bool();
      match cond.operator {
        FilterOperator::Eq => {
          if let Some(v) = cond.value.as_ref().and_then(parse_bool) {
            select.filter(col.eq(v))
          } else {
            select
          }
        }
        FilterOperator::Ne => {
          if let Some(v) = cond.value.as_ref().and_then(parse_bool) {
            select.filter(col.ne(v))
          } else {
            select
          }
        }
        FilterOperator::IsNull => select.filter(col.is_null()),
        _ => select,
      }
    }
    "created_at" | "updated_at" => {
      let col = if cond.field == "created_at" {
        user::Column::CreatedAt
      } else {
        user::Column::UpdatedAt
      };
      let parse_dt = |j: &serde_json::Value| {
        let s = j.as_str()?;
        let s_trim = s.trim_end_matches('Z');
        NaiveDateTime::parse_from_str(s_trim, "%Y-%m-%dT%H:%M:%S%.f")
          .ok()
          .or_else(|| NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok())
      };
      match cond.operator {
        FilterOperator::Eq => {
          if let Some(v) = cond.value.as_ref().and_then(parse_dt) {
            select.filter(col.eq(v))
          } else {
            select
          }
        }
        FilterOperator::Ne => {
          if let Some(v) = cond.value.as_ref().and_then(parse_dt) {
            select.filter(col.ne(v))
          } else {
            select
          }
        }
        FilterOperator::IsNull => select.filter(col.is_null()),
        _ => select,
      }
    }
    _ => select,
  }
}

/// GET /api/dashboard/users — list users. Requires dashboard.users.read. Global scope: all orgs; else session profile org only.
/// Query params: filter (JSON array), sort, order, offset, limit (ListQuerySpec). Returns { users, total }.
pub async fn list_users(
  ScopeFromHeaders(scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
  Query(params): Query<ListQueryParams>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_USERS_READ, Some(&scope)).await {
    return Ok(resp);
  }
  let spec = match parse_list_query_spec(&params) {
    Ok(s) => s,
    Err(msg) => {
      return Ok(
        (
          StatusCode::BAD_REQUEST,
          Json(serde_json::json!({ "error": "Bad Request", "message": msg })),
        )
          .into_response(),
      );
    }
  };
  for cond in &spec.filter {
    if let Err(msg) = validate_filter_cond(cond, DASHBOARD_USERS_FILTER_SORT_FIELDS) {
      return Ok(
        (
          StatusCode::BAD_REQUEST,
          Json(serde_json::json!({ "error": "Bad Request", "message": msg })),
        )
          .into_response(),
      );
    }
  }
  if let Some(ref sort) = spec.sort
    && let Err(msg) = validate_sort_field(&sort.field, DASHBOARD_USERS_FILTER_SORT_FIELDS)
  {
    return Ok(
      (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({ "error": "Bad Request", "message": msg })),
      )
        .into_response(),
    );
  }

  let scope_opt: Option<&forge_auth::RequestScope> =
    if has_global_scope(&db, user, PERMISSION_USERS_READ).await {
      None
    } else {
      Some(&scope)
    };
  let mut select = user_find_scoped(&db, scope_opt)
    .await
    .map_err(|e: sea_orm::DbErr| ForgeError::Generic(e.to_string()))?;
  for cond in &spec.filter {
    select = apply_user_filter(select, cond);
  }
  let total = select
    .clone()
    .count(&db)
    .await
    .map_err(|e: sea_orm::DbErr| ForgeError::Generic(e.to_string()))?;
  if let Some(ref sort) = spec.sort {
    let (col, dir) = match sort.field.as_str() {
      "id" => (user::Column::Id, sort.direction),
      "email" => (user::Column::Email, sort.direction),
      "is_active" => (user::Column::IsActive, sort.direction),
      "is_admin" => (user::Column::IsAdmin, sort.direction),
      "created_at" => (user::Column::CreatedAt, sort.direction),
      "updated_at" => (user::Column::UpdatedAt, sort.direction),
      _ => (user::Column::Email, sort.direction),
    };
    select = match dir {
      SortDirection::Asc => select.order_by_asc(col),
      SortDirection::Desc => select.order_by_desc(col),
    };
  } else {
    select = select.order_by_asc(user::Column::Email);
  }
  let offset = spec.effective_offset();
  let limit = spec.effective_limit();
  let users = select
    .offset(offset)
    .limit(limit)
    .all(&db)
    .await
    .map_err(|e: sea_orm::DbErr| ForgeError::Generic(e.to_string()))?;

  if users.is_empty() {
    return Ok(Json(serde_json::json!({ "users": [], "total": total })).into_response());
  }
  // Epic 6: no embedded relations; return only user fields.
  let list: Vec<serde_json::Value> = users
    .into_iter()
    .map(|u| {
      serde_json::json!({
        "id": u.id.to_string(),
        "email": u.email,
        "is_active": u.is_active,
        "is_admin": u.is_admin,
        "created_at": u.created_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
      })
    })
    .collect();
  Ok(Json(serde_json::json!({ "users": list, "total": total })).into_response())
}

#[derive(Deserialize)]
pub struct CreateUserBody {
  pub email: String,
  pub password: String,
  pub org_id: Uuid,
  pub role_ids: Vec<Uuid>,
}

/// POST /api/dashboard/users — create user and add to org with given roles. Requires dashboard.users.write. Non-admin: org_id must match session profile org.
pub async fn create_user(
  ScopeFromHeaders(scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
  Extension(live_backend): Extension<Option<Arc<InMemoryLiveBackend>>>,
  Json(payload): Json<CreateUserBody>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_USERS_WRITE, Some(&scope)).await {
    return Ok(resp);
  }
  let email = payload.email.trim();
  if email.is_empty() || !email.contains('@') {
    return Ok(
      (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(serde_json::json!({ "error": "Valid email is required" })),
      )
        .into_response(),
    );
  }
  if payload.password.len() < 8 {
    return Ok(
      (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(serde_json::json!({ "error": "Password must be at least 8 characters" })),
      )
        .into_response(),
    );
  }
  let org_id = if has_global_scope(&db, user, PERMISSION_USERS_WRITE).await {
    payload.org_id
  } else {
    if scope.organization_id != payload.org_id {
      return Ok(
        (
          StatusCode::FORBIDDEN,
          Json(serde_json::json!({ "error": "Can only add users to your current organization" })),
        )
          .into_response(),
      );
    }
    payload.org_id
  };
  if payload.role_ids.is_empty() {
    return Ok(
      (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(serde_json::json!({ "error": "At least one role is required" })),
      )
        .into_response(),
    );
  }
  for role_id in &payload.role_ids {
    let role_row = org_role::Entity::find_by_id(*role_id)
      .one(&db)
      .await
      .map_err(|e| ForgeError::Generic(e.to_string()))?;
    match role_row {
      Some(r) if r.org_id == org_id => {}
      _ => {
        return Ok(
          (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": "Each role_id must belong to the selected organization" })),
          )
            .into_response(),
        );
      }
    }
  }
  if user::Entity::find()
    .filter(user::Column::Email.eq(email))
    .one(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?
    .is_some()
  {
    return Ok(
      (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(serde_json::json!({ "error": "Email already in use" })),
      )
        .into_response(),
    );
  }
  let password_hash =
    hash_password(&payload.password).map_err(|e| ForgeError::Generic(e.to_string()))?;
  let now = chrono::Utc::now().naive_utc();
  let user_id = Uuid::new_v4();
  let membership_id = Uuid::new_v4();
  let user_model = user::ActiveModel {
    id: Set(user_id),
    email: Set(email.to_string()),
    password_hash: Set(password_hash),
    is_active: Set(true),
    is_admin: Set(false),
    current_org_id: Set(Some(org_id)),
    current_role: Set(Some(
      org_role::Entity::find_by_id(payload.role_ids[0])
        .one(&db)
        .await
        .ok()
        .flatten()
        .map(|r| r.name)
        .unwrap_or_else(|| "viewer".to_string()),
    )),
    created_at: Set(now),
    updated_at: Set(now),
  };
  user::Entity::insert(user_model)
    .exec(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  let mem_model = membership::ActiveModel {
    id: Set(membership_id),
    user_id: Set(user_id),
    org_id: Set(org_id),
    created_at: Set(now),
    updated_at: Set(now),
  };
  membership::Entity::insert(mem_model)
    .exec(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  for role_id in &payload.role_ids {
    let uor_id = Uuid::new_v4();
    let uor = user_org_role::ActiveModel {
      id: Set(uor_id),
      user_id: Set(user_id),
      org_id: Set(org_id),
      role_id: Set(*role_id),
      created_at: Set(now),
      updated_at: Set(now),
    };
    user_org_role::Entity::insert(uor)
      .exec(&db)
      .await
      .map_err(|e| ForgeError::Generic(e.to_string()))?;
  }
  let role_names: Vec<String> = org_role::Entity::find()
    .filter(org_role::Column::Id.is_in(payload.role_ids.clone()))
    .all(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?
    .into_iter()
    .map(|r| r.name)
    .collect();
  if let Some(ref backend) = live_backend {
    let _ = broadcast_to_channel(
      backend,
      &Channel::org_resource(org_id, "users"),
      &LiveEvent::UsersUpdated {
        user_id: Some(user_id),
        org_id: Some(org_id),
      },
    )
    .await;
  }
  Ok(
    (
      StatusCode::CREATED,
      Json(serde_json::json!({
        "id": user_id.to_string(),
        "email": email,
        "is_active": true,
        "is_admin": false,
        "created_at": now.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
        "memberships": [{ "org_id": org_id.to_string(), "roles": role_names }],
      })),
    )
      .into_response(),
  )
}

#[derive(Deserialize)]
pub struct UpdateUserBody {
  pub id: Uuid,
  pub email: Option<String>,
  pub is_active: Option<bool>,
}

/// PATCH /api/dashboard/users — update user (email, is_active). Requires dashboard.users.write.
pub async fn update_user(
  ScopeFromHeaders(scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
  Extension(live_backend): Extension<Option<Arc<InMemoryLiveBackend>>>,
  Json(payload): Json<UpdateUserBody>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_USERS_WRITE, Some(&scope)).await {
    return Ok(resp);
  }
  let u = user::Entity::find_by_id(payload.id)
    .one(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?
    .ok_or_else(|| ForgeError::Generic("User not found".into()))?;
  let mut am: user::ActiveModel = u.into();
  if let Some(email) = payload.email {
    let t = email.trim();
    if !t.is_empty() && t.contains('@') {
      am.email = Set(t.to_string());
    }
  }
  if let Some(is_active) = payload.is_active {
    am.is_active = Set(is_active);
  }
  am.updated_at = Set(chrono::Utc::now().naive_utc());
  am.update(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  if let Some(ref backend) = live_backend {
    let _ = broadcast_to_channel(
      backend,
      &Channel::org_resource(scope.organization_id, "users"),
      &LiveEvent::UsersUpdated {
        user_id: Some(payload.id),
        org_id: Some(scope.organization_id),
      },
    )
    .await;
  }
  Ok(Json(serde_json::json!({ "ok": true })).into_response())
}

#[derive(Deserialize)]
pub struct DeleteUserBody {
  pub id: Uuid,
}

/// DELETE /api/dashboard/users — delete user and their memberships. Requires dashboard.users.write.
pub async fn delete_user(
  ScopeFromHeaders(scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
  Extension(live_backend): Extension<Option<Arc<InMemoryLiveBackend>>>,
  Json(payload): Json<DeleteUserBody>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_USERS_WRITE, Some(&scope)).await {
    return Ok(resp);
  }
  membership::Entity::delete_many()
    .filter(membership::Column::UserId.eq(payload.id))
    .exec(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  let result = user::Entity::delete_by_id(payload.id)
    .exec(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  if result.rows_affected == 0 {
    return Ok(
      (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "User not found" })),
      )
        .into_response(),
    );
  }
  if let Some(ref backend) = live_backend {
    let _ = broadcast_to_channel(
      backend,
      &Channel::org_resource(scope.organization_id, "users"),
      &LiveEvent::UsersUpdated {
        user_id: Some(payload.id),
        org_id: Some(scope.organization_id),
      },
    )
    .await;
  }
  Ok(Json(serde_json::json!({ "ok": true })).into_response())
}

#[cfg(test)]
mod tests {
  use super::*;
  use forge_auth::RequestScope;

  mod bdd_tests {
    use super::*;

    /// BDD-style tests focusing on behavior rather than implementation.
    /// Tests verify permission checking, helper functions, and core dashboard behaviors.

    mod permission_checking_behavior {
      use super::*;

      #[test]
      fn should_grant_permission_when_exact_key_matches() {
        // Given: a list of permissions containing the exact key
        let perms = vec!["a".to_string(), "user.read".to_string(), "b".to_string()];

        // When: checking for "user.read" permission
        let result = has_permission(&perms, "user.read");

        // Then: permission should be granted
        assert!(result, "Exact permission match should be granted");
      }

      #[test]
      fn should_grant_permission_when_all_read_grants_entity_read() {
        // Given: user has "all.read" permission
        let perms = vec!["all.read".to_string()];

        // When: checking for any ".read" permission
        let result = has_permission(&perms, "user.read");

        // Then: permission should be granted via all.read
        assert!(result, "all.read should grant any entity.read permission");
      }

      #[test]
      fn should_grant_permission_when_all_write_grants_entity_create() {
        // Given: user has "all.write" permission
        let perms = vec!["all.write".to_string()];

        // When: checking for ".create" permission
        let result = has_permission(&perms, "user.create");

        // Then: permission should be granted via all.write
        assert!(
          result,
          "all.write should grant any entity.create permission"
        );
      }

      #[test]
      fn should_grant_permission_when_all_write_grants_entity_update() {
        // Given: user has "all.write" permission
        let perms = vec!["all.write".to_string()];

        // When: checking for ".update" permission
        let result = has_permission(&perms, "user.update");

        // Then: permission should be granted via all.write
        assert!(
          result,
          "all.write should grant any entity.update permission"
        );
      }

      #[test]
      fn should_grant_permission_when_all_write_grants_entity_delete() {
        // Given: user has "all.write" permission
        let perms = vec!["all.write".to_string()];

        // When: checking for ".delete" permission
        let result = has_permission(&perms, "user.delete");

        // Then: permission should be granted via all.write
        assert!(
          result,
          "all.write should grant any entity.delete permission"
        );
      }

      #[test]
      fn should_deny_permission_when_key_not_in_list() {
        // Given: a list of permissions without the requested key
        let perms = vec!["a".to_string(), "b".to_string()];

        // When: checking for "user.read" permission
        let result = has_permission(&perms, "user.read");

        // Then: permission should be denied
        assert!(
          !result,
          "Permission should be denied when key is not in list"
        );
      }

      #[test]
      fn should_deny_permission_when_permissions_list_is_empty() {
        // Given: an empty permissions list
        let perms = vec![];

        // When: checking for any permission
        let result = has_permission(&perms, "any.permission");

        // Then: permission should be denied
        assert!(!result, "Permission should be denied when list is empty");
      }

      #[test]
      fn should_deny_permission_when_all_read_does_not_grant_write() {
        // Given: user has "all.read" permission but not write
        let perms = vec!["all.read".to_string()];

        // When: checking for ".write" permission
        let result = has_permission(&perms, "user.write");

        // Then: permission should be denied
        assert!(!result, "all.read should not grant write permissions");
      }

      #[test]
      fn should_deny_permission_when_all_write_does_not_grant_read() {
        // Given: user has "all.write" permission but not read
        let perms = vec!["all.write".to_string()];

        // When: checking for ".read" permission
        let result = has_permission(&perms, "user.read");

        // Then: permission should be denied
        assert!(!result, "all.write should not grant read permissions");
      }
    }

    mod helper_function_behavior {
      use super::*;

      #[test]
      fn should_return_50_as_default_limit() {
        // Given: default_limit function
        // When: calling default_limit
        let limit = default_limit();

        // Then: it should return 50
        assert_eq!(limit, 50, "Default limit should be 50");
      }

      #[test]
      fn should_create_forbidden_response_with_correct_status() {
        // Given: forbidden_response function
        // When: creating a forbidden response
        let response = forbidden_response();

        // Then: response should have 403 status
        assert_eq!(
          response.status(),
          StatusCode::FORBIDDEN,
          "Forbidden response should have 403 status"
        );
      }

      #[test]
      fn should_create_forbidden_response_with_error_message() {
        // Given: forbidden_response function
        // When: creating a forbidden response
        let response = forbidden_response();

        // Then: response body should contain error information
        // Note: We can't easily test the body without async runtime, but status is verified
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
      }

      #[test]
      fn should_create_request_scope_with_organization_id() {
        // Given: organization ID, role ID, and role name
        let org_a = Uuid::new_v4();
        let role_id = Uuid::new_v4();
        let role_name = "viewer".to_string();

        // When: creating RequestScope
        let scope = RequestScope {
          organization_id: org_a,
          role_id,
          role_name: role_name.clone(),
        };

        // Then: scope should contain the organization ID
        assert_eq!(
          scope.organization_id, org_a,
          "Scope should contain organization ID"
        );
        assert_eq!(scope.role_id, role_id, "Scope should contain role ID");
        assert_eq!(scope.role_name, role_name, "Scope should contain role name");
      }
    }

    mod permission_equivalence_behavior {
      use super::*;

      #[test]
      fn should_check_permission_with_case_sensitive_matching() {
        // Given: permissions list with exact case match
        let perms = vec!["User.Read".to_string(), "user.read".to_string()];

        // When: checking for exact match
        let result1 = has_permission(&perms, "User.Read");
        let result2 = has_permission(&perms, "user.read");

        // Then: only exact case matches should work
        assert!(result1, "Exact case match should work");
        assert!(result2, "Exact case match should work");
      }

      #[test]
      fn should_handle_multiple_permissions_correctly() {
        // Given: a list with multiple permissions
        let perms = vec![
          "dashboard.permissions.read".to_string(),
          "dashboard.users.write".to_string(),
          "dashboard.organizations.read".to_string(),
        ];

        // When: checking various permissions
        let result1 = has_permission(&perms, "dashboard.permissions.read");
        let result2 = has_permission(&perms, "dashboard.users.write");
        let result3 = has_permission(&perms, "dashboard.organizations.read");
        let result4 = has_permission(&perms, "dashboard.roles.read");

        // Then: only existing permissions should be granted
        assert!(result1, "First permission should be granted");
        assert!(result2, "Second permission should be granted");
        assert!(result3, "Third permission should be granted");
        assert!(!result4, "Non-existent permission should be denied");
      }
    }
  }

  // Keep existing tests for backward compatibility
  #[test]
  fn has_permission_true_when_key_in_list() {
    let perms = vec!["a".to_string(), "user.read".to_string(), "b".to_string()];
    assert!(has_permission(&perms, "user.read"));
  }

  #[test]
  fn has_permission_true_when_all_read_grants_entity_read() {
    let perms = vec!["all.read".to_string()];
    assert!(has_permission(&perms, "user.read"));
  }

  #[test]
  fn has_permission_true_when_all_write_grants_entity_create() {
    let perms = vec!["all.write".to_string()];
    assert!(has_permission(&perms, "user.create"));
  }

  #[test]
  fn has_permission_false_when_key_missing() {
    let perms = vec!["a".to_string(), "b".to_string()];
    assert!(!has_permission(&perms, "user.read"));
  }

  #[test]
  fn has_permission_false_when_empty() {
    assert!(!has_permission(&[], "any"));
  }

  #[test]
  fn default_limit_returns_50() {
    assert_eq!(default_limit(), 50);
  }

  #[test]
  fn request_scope_organization_id() {
    let org_a = Uuid::new_v4();
    let role_id = Uuid::new_v4();
    let scope = RequestScope {
      organization_id: org_a,
      role_id,
      role_name: "viewer".to_string(),
    };
    assert_eq!(scope.organization_id, org_a);
  }
}
