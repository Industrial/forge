//! Dashboard API: role–permission CRUD and audit log list. List/view require `dashboard.permissions.read`; add/delete require `dashboard.permissions.write`. Audit log is read-only, gated by `dashboard.audit.read`.

use axum::{
  Json,
  extract::{Query, State, Extension},
  http::StatusCode,
  response::{IntoResponse, Response},
};
use chrono::NaiveDateTime;
use forge::live::{Channel, LiveEvent, InMemoryLiveBackend};
use forge::token_auth::RequireAuth;
use forge::live::broadcast_to_channel;
use forge::{DbConnection, Error as ForgeError};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use forge::auth::hash_password;

use db::auth::Backend;
use db::models::{audit_log, membership, org_role, organization, role_permission, user, user_org_role};

use crate::handlers::auth::{resolve_permissions, DASHBOARD_PERMISSIONS};

const PERMISSION_READ: &str = "dashboard.permissions.read";
const PERMISSION_WRITE: &str = "dashboard.permissions.write";
const PERMISSION_AUDIT_READ: &str = "dashboard.audit.read";
const PERMISSION_ORGS_READ: &str = "dashboard.organizations.read";
const PERMISSION_ORGS_WRITE: &str = "dashboard.organizations.write";
const PERMISSION_USERS_READ: &str = "dashboard.users.read";
const PERMISSION_USERS_WRITE: &str = "dashboard.users.write";
const PERMISSION_ROLES_READ: &str = "dashboard.roles.read";
const PERMISSION_ROLES_WRITE: &str = "dashboard.roles.write";

pub(crate) fn has_permission(permissions: &[String], key: &str) -> bool {
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

/// Org scope for dashboard list endpoints (users, roles, role-permissions). No is_admin bypass — only current org.
pub(crate) fn scope_org_for_dashboard_lists(user: &user::Model) -> Option<Uuid> {
  user.current_org_id
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

/// GET /api/dashboard/role-permissions — list role–permission assignments. Requires dashboard.permissions.read. Scoped to current org only.
pub async fn list_role_permissions(
  RequireAuth(user): RequireAuth<Backend>,
  State(db): State<DbConnection>,
) -> Result<impl IntoResponse, ForgeError> {
  if let Some(resp) = require_permission(&user, &db, PERMISSION_READ).await {
    return Ok(resp);
  }
  let org_id = match scope_org_for_dashboard_lists(&user) {
    Some(id) => id,
    None => return Ok(Json(serde_json::json!({ "assignments": [] })).into_response()),
  };
  let rows = role_permission::Entity::find()
    .filter(role_permission::Column::Scope.eq("org"))
    .filter(role_permission::Column::OrgId.eq(org_id))
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

pub(crate) fn default_limit() -> u64 {
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
  Extension(live_backend): Extension<Option<Arc<InMemoryLiveBackend>>>,
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
  if let (Some(ref backend), Some(org_id)) = (live_backend.as_ref(), org_id_opt) {
    let _ = broadcast_to_channel(backend, &Channel::org_resource(org_id, "role_permissions"), &LiveEvent::ResourceChanged { resource: "role_permissions".into(), id, action: Some("created".into()) }).await;
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

/// DELETE /api/dashboard/role-permissions — remove one role–permission assignment. Requires dashboard.permissions.write. Admin: any; else only scope=org and current_org_id.
pub async fn delete_role_permission(
  RequireAuth(user): RequireAuth<Backend>,
  State(db): State<DbConnection>,
  Extension(live_backend): Extension<Option<Arc<InMemoryLiveBackend>>>,
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
  if let (Some(ref backend), Some(org_id)) = (live_backend.as_ref(), org_id_opt) {
    let _ = broadcast_to_channel(backend, &Channel::org_resource(org_id, "role_permissions"), &LiveEvent::ResourceChanged { resource: "role_permissions".into(), id: Uuid::nil(), action: Some("deleted".into()) }).await;
  }
  Ok(Json(serde_json::json!({ "ok": true })).into_response())
}

/// GET /api/dashboard/organizations — list all organizations. Requires dashboard.organizations.read (global).
pub async fn list_organizations(
  RequireAuth(user): RequireAuth<Backend>,
  State(db): State<DbConnection>,
) -> Result<impl IntoResponse, ForgeError> {
  if let Some(resp) = require_permission(&user, &db, PERMISSION_ORGS_READ).await {
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

/// GET /api/dashboard/roles — list org roles. Scoped to current org only (query org_id ignored). Requires dashboard.roles.read.
pub async fn list_roles(
  RequireAuth(user): RequireAuth<Backend>,
  State(db): State<DbConnection>,
  Query(_q): Query<ListRolesQuery>,
) -> Result<impl IntoResponse, ForgeError> {
  if let Some(resp) = require_permission(&user, &db, PERMISSION_ROLES_READ).await {
    return Ok(resp);
  }
  let org_id = match scope_org_for_dashboard_lists(&user) {
    Some(id) => id,
    None => return Ok(Json(serde_json::json!({ "roles": [] })).into_response()),
  };
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

#[derive(Deserialize)]
pub struct CreateRoleBody {
  pub org_id: Option<Uuid>,
  pub name: String,
  pub display_name: Option<String>,
}

/// POST /api/dashboard/roles — create org role. Requires dashboard.roles.write. org_id must be current org.
pub async fn create_role(
  RequireAuth(user): RequireAuth<Backend>,
  State(db): State<DbConnection>,
  Extension(live_backend): Extension<Option<Arc<InMemoryLiveBackend>>>,
  Json(payload): Json<CreateRoleBody>,
) -> Result<impl IntoResponse, ForgeError> {
  if let Some(resp) = require_permission(&user, &db, PERMISSION_ROLES_WRITE).await {
    return Ok(resp);
  }
  let org_id = match user.current_org_id {
    Some(id) => id,
    None => {
      return Ok(
        (
          StatusCode::FORBIDDEN,
          Json(serde_json::json!({ "error": "No organization context" })),
        )
          .into_response(),
      );
    }
  };
  if payload.org_id.map(|pid| pid != org_id).unwrap_or(false) {
    return Ok(
      (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({ "error": "Can only create roles in your current organization" })),
      )
        .into_response(),
    );
  }
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
  let display_name = payload.display_name.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()).map(|s| s.to_string());
  let model = org_role::ActiveModel {
    id: Set(id),
    org_id: Set(org_id),
    name: Set(name.to_string()),
    display_name: Set(display_name.clone()),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  };
  org_role::Entity::insert(model).exec(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  if let Some(ref backend) = live_backend {
    let _ = broadcast_to_channel(backend, &Channel::org_resource(org_id, "roles"), &LiveEvent::ResourceChanged { resource: "roles".into(), id, action: Some("created".into()) }).await;
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
  RequireAuth(user): RequireAuth<Backend>,
  State(db): State<DbConnection>,
  Extension(live_backend): Extension<Option<Arc<InMemoryLiveBackend>>>,
  Json(payload): Json<UpdateRoleBody>,
) -> Result<impl IntoResponse, ForgeError> {
  if let Some(resp) = require_permission(&user, &db, PERMISSION_ROLES_WRITE).await {
    return Ok(resp);
  }
  let role = org_role::Entity::find_by_id(payload.id)
    .one(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?
    .ok_or_else(|| ForgeError::Generic("Role not found".into()))?;
  if user.current_org_id != Some(role.org_id) && !user.is_admin {
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
  am.update(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  if let Some(ref backend) = live_backend {
    let _ = broadcast_to_channel(backend, &Channel::org_resource(org_id, "roles"), &LiveEvent::ResourceChanged { resource: "roles".into(), id: role_id, action: Some("updated".into()) }).await;
  }
  Ok(Json(serde_json::json!({ "ok": true })).into_response())
}

#[derive(Deserialize)]
pub struct DeleteRoleBody {
  pub id: Uuid,
}

/// DELETE /api/dashboard/roles — delete org role. Requires dashboard.roles.write. Fails if any user has this role.
pub async fn delete_role(
  RequireAuth(user): RequireAuth<Backend>,
  State(db): State<DbConnection>,
  Extension(live_backend): Extension<Option<Arc<InMemoryLiveBackend>>>,
  Json(payload): Json<DeleteRoleBody>,
) -> Result<impl IntoResponse, ForgeError> {
  if let Some(resp) = require_permission(&user, &db, PERMISSION_ROLES_WRITE).await {
    return Ok(resp);
  }
  let role = org_role::Entity::find_by_id(payload.id)
    .one(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?
    .ok_or_else(|| ForgeError::Generic("Role not found".into()))?;
  if user.current_org_id != Some(role.org_id) && !user.is_admin {
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
    let _ = broadcast_to_channel(backend, &Channel::org_resource(role.org_id, "roles"), &LiveEvent::ResourceChanged { resource: "roles".into(), id: payload.id, action: Some("deleted".into()) }).await;
  }
  Ok(Json(serde_json::json!({ "ok": true })).into_response())
}

#[derive(Deserialize)]
pub struct CreateOrganizationBody {
  pub name: String,
  pub slug: Option<String>,
}

fn slug_from_name(name: &str) -> String {
  name
    .to_lowercase()
    .chars()
    .map(|c| if c.is_alphanumeric() || c == ' ' { c } else { '-' })
    .collect::<String>()
    .split_whitespace()
    .filter(|s| !s.is_empty())
    .collect::<Vec<_>>()
    .join("-")
}

/// POST /api/dashboard/organizations — create organization. Requires dashboard.organizations.write.
pub async fn create_organization(
  RequireAuth(user): RequireAuth<Backend>,
  State(db): State<DbConnection>,
  Extension(live_backend): Extension<Option<Arc<InMemoryLiveBackend>>>,
  Json(payload): Json<CreateOrganizationBody>,
) -> Result<impl IntoResponse, ForgeError> {
  if let Some(resp) = require_permission(&user, &db, PERMISSION_ORGS_WRITE).await {
    return Ok(resp);
  }
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
  let slug: String = match payload
    .slug
    .as_deref()
    .map(|s| s.trim())
    .filter(|s| !s.is_empty())
  {
    Some(s) => s.to_string(),
    None => slug_from_name(name),
  };
  let now = chrono::Utc::now().naive_utc();
  let id = Uuid::new_v4();
  let model = organization::ActiveModel {
    id: Set(id),
    name: Set(name.to_string()),
    slug: Set(slug.to_string()),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  };
  organization::Entity::insert(model)
    .exec(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  for &role_name in &["owner", "admin", "editor", "viewer"] {
    let role_id = Uuid::new_v4();
    let r = org_role::ActiveModel {
      id: Set(role_id),
      org_id: Set(id),
      name: Set(role_name.to_string()),
      display_name: Set(None),
      created_at: Set(now),
      updated_at: Set(now),
      ..Default::default()
    };
    org_role::Entity::insert(r).exec(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  }
  db::seed_role_permissions_for_org(&db, id).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  if let Some(ref backend) = live_backend {
    let ch = Channel::raw("organizations");
    let _ = broadcast_to_channel(backend, &ch, &LiveEvent::ResourceChanged { resource: "organizations".into(), id, action: Some("created".into()) }).await;
  }
  Ok(
    (
      StatusCode::CREATED,
      Json(serde_json::json!({
        "id": id.to_string(),
        "name": name,
        "slug": slug,
        "created_at": now.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
        "updated_at": now.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
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
  RequireAuth(user): RequireAuth<Backend>,
  State(db): State<DbConnection>,
  Extension(live_backend): Extension<Option<Arc<InMemoryLiveBackend>>>,
  Json(payload): Json<UpdateOrganizationBody>,
) -> Result<impl IntoResponse, ForgeError> {
  if let Some(resp) = require_permission(&user, &db, PERMISSION_ORGS_WRITE).await {
    return Ok(resp);
  }
  let org = organization::Entity::find_by_id(payload.id)
    .one(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?
    .ok_or_else(|| ForgeError::Generic("Organization not found".into()))?;
  let mut am: organization::ActiveModel = org.into();
  if let Some(name) = payload.name {
    let t = name.trim();
    if !t.is_empty() {
      am.name = Set(t.to_string());
    }
  }
  if let Some(slug) = payload.slug {
    let t = slug.trim();
    if !t.is_empty() {
      am.slug = Set(t.to_string());
    }
  }
  am.updated_at = Set(chrono::Utc::now().naive_utc());
  am.update(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  if let Some(ref backend) = live_backend {
    let ch = Channel::raw("organizations");
    let _ = broadcast_to_channel(backend, &ch, &LiveEvent::ResourceChanged { resource: "organizations".into(), id: payload.id, action: Some("updated".into()) }).await;
  }
  let updated = organization::Entity::find_by_id(payload.id)
    .one(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?
    .ok_or_else(|| ForgeError::Generic("Organization not found".into()))?;
  Ok(Json(serde_json::json!({
    "id": updated.id.to_string(),
    "name": updated.name,
    "slug": updated.slug,
    "created_at": updated.created_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
    "updated_at": updated.updated_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
  })).into_response())
}

#[derive(Deserialize)]
pub struct DeleteOrganizationBody {
  pub id: Uuid,
}

/// DELETE /api/dashboard/organizations — delete organization. Requires dashboard.organizations.write.
pub async fn delete_organization(
  RequireAuth(user): RequireAuth<Backend>,
  State(db): State<DbConnection>,
  Extension(live_backend): Extension<Option<Arc<InMemoryLiveBackend>>>,
  Json(payload): Json<DeleteOrganizationBody>,
) -> Result<impl IntoResponse, ForgeError> {
  if let Some(resp) = require_permission(&user, &db, PERMISSION_ORGS_WRITE).await {
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
    let _ = broadcast_to_channel(backend, &ch, &LiveEvent::ResourceChanged { resource: "organizations".into(), id: payload.id, action: Some("deleted".into()) }).await;
  }
  Ok(Json(serde_json::json!({ "ok": true })).into_response())
}

/// GET /api/dashboard/users — list users. Requires dashboard.users.read. Scoped to current org only (no is_admin bypass).
pub async fn list_users(
  RequireAuth(user): RequireAuth<Backend>,
  State(db): State<DbConnection>,
) -> Result<impl IntoResponse, ForgeError> {
  if let Some(resp) = require_permission(&user, &db, PERMISSION_USERS_READ).await {
    return Ok(resp);
  }
  let scope_org_id = match scope_org_for_dashboard_lists(&user) {
    Some(id) => id,
    None => return Ok(Json(serde_json::json!({ "users": [] })).into_response()),
  };
  let user_ids: Vec<Uuid> = membership::Entity::find()
    .filter(membership::Column::OrgId.eq(scope_org_id))
    .all(&db)
    .await
    .map_err(|e: sea_orm::DbErr| ForgeError::Generic(e.to_string()))?
    .into_iter()
    .map(|m| m.user_id)
    .collect::<std::collections::HashSet<_>>()
    .into_iter()
    .collect();
  if user_ids.is_empty() {
    return Ok(Json(serde_json::json!({ "users": [] })).into_response());
  }
  let users = user::Entity::find()
    .filter(user::Column::Id.is_in(user_ids.clone()))
    .all(&db)
    .await
    .map_err(|e: sea_orm::DbErr| ForgeError::Generic(e.to_string()))?;
  let org_row = organization::Entity::find_by_id(scope_org_id)
    .one(&db)
    .await
    .map_err(|e: sea_orm::DbErr| ForgeError::Generic(e.to_string()))?;
  let org_name = org_row.as_ref().map(|o| o.name.clone()).unwrap_or_else(|| "—".to_string());
  let all_uors = user_org_role::Entity::find()
    .filter(user_org_role::Column::UserId.is_in(user_ids.clone()))
    .filter(user_org_role::Column::OrgId.eq(scope_org_id))
    .all(&db)
    .await
    .map_err(|e: sea_orm::DbErr| ForgeError::Generic(e.to_string()))?;
  let role_ids: Vec<Uuid> = all_uors.iter().map(|x| x.role_id).collect::<std::collections::HashSet<_>>().into_iter().collect();
  let roles = if role_ids.is_empty() {
    vec![]
  } else {
    org_role::Entity::find()
      .filter(org_role::Column::Id.is_in(role_ids))
      .all(&db)
      .await
      .map_err(|e: sea_orm::DbErr| ForgeError::Generic(e.to_string()))?
  };
  let role_map: std::collections::HashMap<Uuid, org_role::Model> =
    roles.into_iter().map(|r| (r.id, r)).collect();
  let list: Vec<serde_json::Value> = users
    .into_iter()
    .map(|u| {
      let role_names: Vec<String> = all_uors
        .iter()
        .filter(|x| x.user_id == u.id)
        .filter_map(|x| role_map.get(&x.role_id).map(|r| r.name.clone()))
        .collect();
      let mems = vec![serde_json::json!({
        "org_id": scope_org_id.to_string(),
        "org_name": org_name,
        "roles": role_names,
      })];
      serde_json::json!({
        "id": u.id.to_string(),
        "email": u.email,
        "is_active": u.is_active,
        "is_admin": u.is_admin,
        "created_at": u.created_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
        "memberships": mems,
      })
    })
    .collect();
  Ok(Json(serde_json::json!({ "users": list })).into_response())
}

#[derive(Deserialize)]
pub struct CreateUserBody {
  pub email: String,
  pub password: String,
  pub org_id: Uuid,
  pub role_ids: Vec<Uuid>,
}

/// POST /api/dashboard/users — create user and add to org with given roles. Requires dashboard.users.write. Non-admin: org_id must be current_org_id.
pub async fn create_user(
  RequireAuth(user): RequireAuth<Backend>,
  State(db): State<DbConnection>,
  Extension(live_backend): Extension<Option<Arc<InMemoryLiveBackend>>>,
  Json(payload): Json<CreateUserBody>,
) -> Result<impl IntoResponse, ForgeError> {
  if let Some(resp) = require_permission(&user, &db, PERMISSION_USERS_WRITE).await {
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
  let org_id = match user.current_org_id {
    Some(id) if id == payload.org_id => id,
    _ => {
      return Ok(
        (
          StatusCode::FORBIDDEN,
          Json(serde_json::json!({ "error": "Can only add users to your current organization" })),
        )
          .into_response(),
      );
    }
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
    ..Default::default()
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
    ..Default::default()
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
      ..Default::default()
    };
    user_org_role::Entity::insert(uor).exec(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
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
    let _ = broadcast_to_channel(backend, &Channel::org_resource(org_id, "users"), &LiveEvent::UsersUpdated { user_id: Some(user_id), org_id: Some(org_id) }).await;
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
  RequireAuth(user): RequireAuth<Backend>,
  State(db): State<DbConnection>,
  Extension(live_backend): Extension<Option<Arc<InMemoryLiveBackend>>>,
  Json(payload): Json<UpdateUserBody>,
) -> Result<impl IntoResponse, ForgeError> {
  if let Some(resp) = require_permission(&user, &db, PERMISSION_USERS_WRITE).await {
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
  am.update(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  if let Some(ref backend) = live_backend {
    if let Some(org_id) = user.current_org_id {
      let _ = broadcast_to_channel(backend, &Channel::org_resource(org_id, "users"), &LiveEvent::UsersUpdated { user_id: Some(payload.id), org_id: Some(org_id) }).await;
    }
  }
  Ok(Json(serde_json::json!({ "ok": true })).into_response())
}

#[derive(Deserialize)]
pub struct DeleteUserBody {
  pub id: Uuid,
}

/// DELETE /api/dashboard/users — delete user and their memberships. Requires dashboard.users.write.
pub async fn delete_user(
  RequireAuth(user): RequireAuth<Backend>,
  State(db): State<DbConnection>,
  Extension(live_backend): Extension<Option<Arc<InMemoryLiveBackend>>>,
  Json(payload): Json<DeleteUserBody>,
) -> Result<impl IntoResponse, ForgeError> {
  if let Some(resp) = require_permission(&user, &db, PERMISSION_USERS_WRITE).await {
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
    if let Some(org_id) = user.current_org_id {
      let _ = broadcast_to_channel(backend, &Channel::org_resource(org_id, "users"), &LiveEvent::UsersUpdated { user_id: Some(payload.id), org_id: Some(org_id) }).await;
    }
  }
  Ok(Json(serde_json::json!({ "ok": true })).into_response())
}

#[cfg(test)]
mod tests {
  use super::*;
  use chrono::Utc;

  #[test]
  fn has_permission_true_when_key_in_list() {
    let perms = vec!["a".to_string(), "dashboard.read".to_string(), "b".to_string()];
    assert!(has_permission(&perms, "dashboard.read"));
  }

  #[test]
  fn has_permission_false_when_key_missing() {
    let perms = vec!["a".to_string(), "b".to_string()];
    assert!(!has_permission(&perms, "dashboard.read"));
  }

  #[test]
  fn has_permission_false_when_empty() {
    assert!(!has_permission(&[], "any"));
  }

  #[test]
  fn default_limit_returns_50() {
    assert_eq!(default_limit(), 50);
  }

  /// Scope for dashboard lists must be current org only; is_admin must not expand scope.
  #[test]
  fn scope_org_for_dashboard_lists_uses_only_current_org() {
    let now = Utc::now().naive_utc();
    let org_a = Uuid::new_v4();
    let user_none = user::Model {
      id: Uuid::new_v4(),
      email: "a@x.org".to_string(),
      password_hash: "".to_string(),
      is_active: true,
      is_admin: false,
      current_org_id: None,
      current_role: Some("viewer".to_string()),
      created_at: now,
      updated_at: now,
    };
    let user_org_a = user::Model {
      id: Uuid::new_v4(),
      email: "b@x.org".to_string(),
      password_hash: "".to_string(),
      is_active: true,
      is_admin: false,
      current_org_id: Some(org_a),
      current_role: Some("viewer".to_string()),
      created_at: now,
      updated_at: now,
    };
    let user_admin_org_a = user::Model {
      id: Uuid::new_v4(),
      email: "admin@x.org".to_string(),
      password_hash: "".to_string(),
      is_active: true,
      is_admin: true,
      current_org_id: Some(org_a),
      current_role: Some("owner".to_string()),
      created_at: now,
      updated_at: now,
    };
    assert_eq!(scope_org_for_dashboard_lists(&user_none), None);
    assert_eq!(scope_org_for_dashboard_lists(&user_org_a), Some(org_a));
    assert_eq!(scope_org_for_dashboard_lists(&user_admin_org_a), Some(org_a));
  }

  /// Ensures that even with is_admin=true we do not get "all orgs" — only current_org_id.
  #[test]
  fn scope_org_ignores_is_admin_no_cross_org() {
    let now = Utc::now().naive_utc();
    let org_a = Uuid::new_v4();
    let admin = user::Model {
      id: Uuid::new_v4(),
      email: "admin@x.org".to_string(),
      password_hash: "".to_string(),
      is_active: true,
      is_admin: true,
      current_org_id: Some(org_a),
      current_role: Some("owner".to_string()),
      created_at: now,
      updated_at: now,
    };
    let scope = scope_org_for_dashboard_lists(&admin);
    assert_eq!(scope, Some(org_a));
  }
}
