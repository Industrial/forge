//! Shared auth/scope/permission helpers used by auth handlers and dashboard.

use crate::Error as ForgeError;
use axum::{
  Form, Json,
  extract::{FromRequest, FromRequestParts},
  http::StatusCode,
  http::request::Parts,
  response::{IntoResponse, Response},
};
use forge_audit::{AuditEvent, LogResult};
use forge_auth::RequestScope;
use forge_db::DbConnection;
use forge_live::LiveBackend;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::de::DeserializeOwned;

use db::models::{org_role, organization, role_permission, user, user_global_role, user_org_role};

use crate::permissions::entity_action_key;

/// Permission key constants (entity-based).
pub const PERMISSION_READ: &str = "permission.read";
pub const PERMISSION_WRITE: &str = "permission.create";
pub const PERMISSION_ORGS_READ: &str = "organization.read";
pub const PERMISSION_ORGS_WRITE: &str = "organization.create";
pub const PERMISSION_USERS_READ: &str = "user.read";
pub const PERMISSION_USERS_WRITE: &str = "user.create";
pub const PERMISSION_ROLES_READ: &str = "role.read";
pub const PERMISSION_ROLES_WRITE: &str = "role.create";
pub const PERMISSION_AUDIT_READ: &str = "audit.read";

/// Resolves the list of permission keys for the current user from org-scoped and global-scope role_permission.
pub async fn resolve_permissions(
  db: &DbConnection,
  user: &user::Model,
  scope: Option<&RequestScope>,
) -> Vec<String> {
  let mut keys = std::collections::HashSet::<String>::new();

  let global_roles: Vec<String> = user_global_role::Entity::find()
    .filter(user_global_role::Column::UserId.eq(user.id))
    .all(db)
    .await
    .ok()
    .unwrap_or_default()
    .into_iter()
    .map(|r| r.role_name)
    .collect();
  if !global_roles.is_empty() {
    let global_perms = role_permission::Entity::find()
      .filter(role_permission::Column::Scope.eq("global"))
      .filter(role_permission::Column::OrgId.is_null())
      .filter(role_permission::Column::RoleName.is_in(global_roles))
      .all(db)
      .await
      .ok()
      .unwrap_or_default();
    for r in global_perms {
      keys.insert(r.permission_key);
    }
  }

  if let Some(s) = scope {
    let rows = role_permission::Entity::find()
      .filter(role_permission::Column::Scope.eq("org"))
      .filter(role_permission::Column::OrgId.eq(s.organization_id))
      .filter(role_permission::Column::RoleName.eq(&s.role_name))
      .all(db)
      .await
      .ok()
      .unwrap_or_default();
    for r in rows {
      keys.insert(r.permission_key);
    }
  }

  keys.into_iter().collect()
}

/// True if the user has the given permission at global scope.
pub async fn has_global_scope(db: &DbConnection, user: &user::Model, permission_key: &str) -> bool {
  let global_roles: Vec<String> = user_global_role::Entity::find()
    .filter(user_global_role::Column::UserId.eq(user.id))
    .all(db)
    .await
    .ok()
    .unwrap_or_default()
    .into_iter()
    .map(|r| r.role_name)
    .collect();
  if global_roles.is_empty() {
    return false;
  }
  let keys: std::collections::HashSet<String> = role_permission::Entity::find()
    .filter(role_permission::Column::Scope.eq("global"))
    .filter(role_permission::Column::OrgId.is_null())
    .filter(role_permission::Column::RoleName.is_in(global_roles))
    .all(db)
    .await
    .ok()
    .unwrap_or_default()
    .into_iter()
    .map(|r| r.permission_key)
    .collect();
  if keys.contains(permission_key) {
    return true;
  }
  let is_read = permission_key == "all.read" || permission_key.ends_with(".read");
  let is_write = permission_key == "all.write"
    || permission_key.ends_with(".write")
    || permission_key.ends_with(".create")
    || permission_key.ends_with(".update")
    || permission_key.ends_with(".delete");
  (is_read && keys.contains("all.read")) || (is_write && keys.contains("all.write"))
}

/// Entity-based permission check: exact match or all.read / all.write wildcard.
pub fn has_permission(permissions: &[String], key: &str) -> bool {
  // Direct exact match
  if permissions.iter().any(|p| p == key) {
    return true;
  }
  // Wildcard: all.read grants any .read permission
  if key.ends_with(".read") && permissions.iter().any(|p| p == "all.read") {
    return true;
  }
  // Wildcard: all.write grants any .write/.create/.update/.delete permission
  if (key.ends_with(".write")
    || key.ends_with(".create")
    || key.ends_with(".update")
    || key.ends_with(".delete"))
    && permissions.iter().any(|p| p == "all.write")
  {
    return true;
  }
  false
}

/// Returns Some(403 response) if the current user does not have the given permission.
pub async fn require_permission(
  user: &user::Model,
  db: &DbConnection,
  permission: &str,
  scope: Option<&RequestScope>,
) -> Option<Response> {
  let permissions = resolve_permissions(db, user, scope).await;
  if has_permission(&permissions, permission) {
    return None;
  }
  Some(forbidden_response())
}

/// 403 Forbidden with clear message when permission is missing.
pub fn forbidden_response() -> Response {
  (
    StatusCode::FORBIDDEN,
    Json(serde_json::json!({
      "error": "Forbidden",
      "message": "Insufficient permissions"
    })),
  )
    .into_response()
}

/// Require the corresponding entity.action in scope.
pub async fn require_entity_permission(
  user: &user::Model,
  db: &DbConnection,
  scope: Option<&RequestScope>,
  entity: &str,
  action: &str,
) -> Option<Response> {
  let key = entity_action_key(entity, action);
  require_permission(user, db, &key, scope).await
}

/// Returns Some(403) if the user has no resolved permissions.
pub async fn require_any_permission(
  user: &user::Model,
  db: &DbConnection,
  scope: Option<&RequestScope>,
) -> Option<Response> {
  let permissions = resolve_permissions(db, user, scope).await;
  if permissions.is_empty() {
    return Some(forbidden_response());
  }
  None
}

/// Header names for scope.
pub const HEADER_ORGANIZATION_ID: &str = "x-organization-id";
pub const HEADER_ROLE_ID: &str = "x-role-id";

/// Builds [RequestScope] from headers. Validates org, role, and user membership.
pub async fn try_scope_from_headers(
  headers: &axum::http::HeaderMap,
  db: &DbConnection,
  user_id: uuid::Uuid,
) -> Result<RequestScope, ForgeError> {
  let org_id_str = headers
    .get(HEADER_ORGANIZATION_ID)
    .and_then(|v| v.to_str().ok())
    .ok_or_else(|| {
      ForgeError::Auth(
        StatusCode::BAD_REQUEST,
        "Missing or invalid X-Organization-Id header".to_string(),
      )
    })?;
  let org_id = uuid::Uuid::parse_str(org_id_str).map_err(|_| {
    ForgeError::Auth(
      StatusCode::BAD_REQUEST,
      "Invalid X-Organization-Id".to_string(),
    )
  })?;
  let role_id_str = headers
    .get(HEADER_ROLE_ID)
    .and_then(|v| v.to_str().ok())
    .ok_or_else(|| {
      ForgeError::Auth(
        StatusCode::BAD_REQUEST,
        "Missing or invalid X-Role-Id header".to_string(),
      )
    })?;
  let role_id = uuid::Uuid::parse_str(role_id_str)
    .map_err(|_| ForgeError::Auth(StatusCode::BAD_REQUEST, "Invalid X-Role-Id".to_string()))?;

  organization::Entity::find_by_id(org_id)
    .one(db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?
    .ok_or_else(|| ForgeError::Auth(StatusCode::NOT_FOUND, "Organization not found".to_string()))?;

  let role_row = org_role::Entity::find_by_id(role_id)
    .one(db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?
    .ok_or_else(|| ForgeError::Auth(StatusCode::NOT_FOUND, "Role not found".to_string()))?;

  if role_row.org_id != org_id {
    return Err(ForgeError::Auth(
      StatusCode::BAD_REQUEST,
      "X-Role-Id does not belong to X-Organization-Id".to_string(),
    ));
  }

  let has = user_org_role::Entity::find()
    .filter(user_org_role::Column::UserId.eq(user_id))
    .filter(user_org_role::Column::OrgId.eq(org_id))
    .filter(user_org_role::Column::RoleId.eq(role_id))
    .one(db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;

  if has.is_none() {
    return Err(ForgeError::Auth(
      StatusCode::FORBIDDEN,
      "You do not have this role in this organization".to_string(),
    ));
  }

  Ok(RequestScope {
    organization_id: org_id,
    role_id,
    role_name: role_row.name.clone(),
  })
}

/// Extractor: requires Bearer auth and X-Organization-Id + X-Role-Id headers.
#[derive(Clone, Debug)]
pub struct ScopeFromHeaders(pub RequestScope);

impl FromRequestParts<DbConnection> for ScopeFromHeaders {
  type Rejection = ForgeError;

  fn from_request_parts(
    parts: &mut Parts,
    state: &DbConnection,
  ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
    let user_id = parts
      .extensions
      .get::<forge_auth::token_auth::TokenUser<user::Model>>()
      .map(|tu| tu.user.id);
    let db = state.clone();
    let headers = parts.headers.clone();
    async move {
      let user_id = user_id.ok_or_else(|| {
        ForgeError::Auth(
          StatusCode::UNAUTHORIZED,
          "Authentication required".to_string(),
        )
      })?;
      let scope = try_scope_from_headers(&headers, &db, user_id).await?;
      Ok(ScopeFromHeaders(scope))
    }
  }
}

/// Optional scope from request headers.
pub async fn get_scope_from_headers_map(
  headers: &axum::http::HeaderMap,
  user: &user::Model,
  db: &DbConnection,
) -> Option<RequestScope> {
  try_scope_from_headers(headers, db, user.id).await.ok()
}

/// Derives live-update channel subscriptions from the user's permissions and current org.
pub fn channels_from_permissions(
  permissions: &[String],
  current_org_id: Option<uuid::Uuid>,
) -> Vec<forge_live::Channel> {
  let perms: std::collections::HashSet<_> = permissions.iter().map(String::as_str).collect();
  let mut out = Vec::new();
  if perms.contains("organization.read") {
    out.push(forge_live::Channel::raw("organizations"));
  }
  if perms.contains("audit.read") {
    out.push(forge_live::Channel::raw("audit-log"));
  }
  out.push(forge_live::Channel::raw("tasks"));
  if let Some(org_id) = current_org_id {
    if perms.contains("user.read") {
      out.push(forge_live::Channel::org_resource(org_id, "users"));
    }
    if perms.contains("role.read") {
      out.push(forge_live::Channel::org_resource(org_id, "roles"));
    }
    if perms.contains("permission.read") {
      out.push(forge_live::Channel::org_resource(
        org_id,
        "role_permissions",
      ));
    }
  }
  out
}

/// Accepts JSON or form-urlencoded (e.g. login). Generic over the payload type.
pub struct JsonOrForm<T>(pub T);

impl<S, T> FromRequest<S> for JsonOrForm<T>
where
  T: DeserializeOwned + Send,
  S: Send + Sync,
{
  type Rejection = (StatusCode, &'static str);

  async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
    let (parts, body) = req.into_parts();
    let content_type = parts.headers.get(axum::http::header::CONTENT_TYPE);
    let is_form = content_type
      .and_then(|v| v.to_str().ok())
      .map(|v| v.starts_with("application/x-www-form-urlencoded"))
      .unwrap_or(false);
    let req = axum::extract::Request::from_parts(parts, body);
    if is_form {
      let Form(payload) = Form::<T>::from_request(req, state)
        .await
        .map_err(|_| (StatusCode::BAD_REQUEST, "invalid form body"))?;
      Ok(JsonOrForm(payload))
    } else {
      let Json(payload) = Json::<T>::from_request(req, state)
        .await
        .map_err(|_| (StatusCode::BAD_REQUEST, "invalid JSON body"))?;
      Ok(JsonOrForm(payload))
    }
  }
}

/// Broadcasts an audit log entry to the live channel if backend is present.
pub async fn broadcast_audit_entry(
  live_backend: Option<&std::sync::Arc<forge_live::InMemoryLiveBackend>>,
  event: &AuditEvent,
  result: &LogResult,
) {
  let entry = serde_json::json!({
    "id": result.id.to_string(),
    "event_kind": event.event_kind.as_str(),
    "actor_id": event.actor_id.to_string(),
    "subject_id": event.subject_id.map(|u| u.to_string()),
    "organization_id": event.organization_id.map(|u| u.to_string()),
    "action": event.action_str(),
    "resource_type": event.resource_type,
    "resource_id": event.resource_id.map(|u| u.to_string()),
    "outcome": event.outcome.as_str(),
    "reason": event.reason,
    "occurred_at": result.occurred_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
  });
  if let Some(backend) = live_backend {
    let payload = serde_json::to_vec(&serde_json::json!({ "type": "audit_log", "entry": entry }))
      .unwrap_or_default();
    let channel = forge_live::Channel::raw("audit-log");
    backend.broadcast(&channel, &payload).await;
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use forge_auth::RequestScope;

  #[test]
  fn request_scope_holds_org_id_role_id_and_role_name() {
    let org_id = uuid::Uuid::new_v4();
    let role_id = uuid::Uuid::new_v4();
    let scope = RequestScope {
      organization_id: org_id,
      role_id,
      role_name: "viewer".to_string(),
    };
    assert_eq!(scope.organization_id, org_id);
    assert_eq!(scope.role_id, role_id);
    assert_eq!(scope.role_name, "viewer");
  }

  mod has_permission_behavior {
    use super::*;

    #[test]
    fn should_grant_permission_when_exact_key_matches() {
      let perms = vec!["a".to_string(), "user.read".to_string(), "b".to_string()];
      assert!(has_permission(&perms, "user.read"));
    }

    #[test]
    fn should_grant_permission_when_all_read_grants_entity_read() {
      let perms = vec!["all.read".to_string()];
      assert!(has_permission(&perms, "user.read"));
    }

    #[test]
    fn should_deny_permission_when_key_not_in_list() {
      let perms = vec!["a".to_string(), "b".to_string()];
      assert!(!has_permission(&perms, "user.read"));
    }
  }

  mod channels_from_permissions_behavior {
    use super::*;

    #[test]
    fn should_create_organizations_channel_when_permission_present() {
      let permissions = vec!["organization.read".to_string()];
      let org_id = Some(uuid::Uuid::new_v4());
      let channels = channels_from_permissions(&permissions, org_id);
      assert!(channels.iter().any(|c| c.as_str() == "organizations"));
    }

    #[test]
    fn should_always_include_tasks_channel() {
      let permissions = vec![];
      let channels = channels_from_permissions(&permissions, None);
      assert!(channels.iter().any(|c| c.as_str() == "tasks"));
    }
  }
}
