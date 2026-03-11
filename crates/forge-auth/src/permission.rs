//! Permission check ([has_permission]), resolver trait ([PermissionResolver]), and
//! [require_permission] / [require_entity_permission]. The app provides the [PermissionResolver] implementation.

use async_trait::async_trait;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use forge_db::DbConnection;
use serde::Serialize;
use uuid::Uuid;

use crate::authz::RequestScope;

#[derive(Serialize)]
struct ForbiddenBody {
  error: &'static str,
  message: &'static str,
}

/// Trait for resolving a user's permission keys (e.g. from role_permission and user_global_role).
#[async_trait]
pub trait PermissionResolver: Send + Sync {
  /// Resolve the list of permission keys for the user in the given scope.
  async fn resolve(
    &self,
    db: &DbConnection,
    user_id: Uuid,
    scope: Option<&RequestScope>,
  ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>>;
}

/// Builds a permission key `<entity>.<action>`.
#[inline]
pub fn entity_action_key(entity: &str, action: &str) -> String {
  format!("{}.{}", entity, action)
}

/// Returns a 403 JSON response for forbidden access.
pub fn forbidden_response() -> Response {
  (
    StatusCode::FORBIDDEN,
    axum::Json(ForbiddenBody {
      error: "Forbidden",
      message: "You do not have permission to perform this action",
    }),
  )
    .into_response()
}

/// True if the resolved permissions grant the given key. Supports exact match and
/// wildcards: `all.read` grants any `*.read`, `all.write` grants `*.create`/`*.update`/`*.delete`/`*.write`.
pub fn has_permission(perms: &[String], key: &str) -> bool {
  let set: std::collections::HashSet<&str> = perms.iter().map(String::as_str).collect();
  if set.contains(key) {
    return true;
  }
  let is_read = key == "all.read" || key.ends_with(".read");
  let is_write = key == "all.write"
    || key.ends_with(".write")
    || key.ends_with(".create")
    || key.ends_with(".update")
    || key.ends_with(".delete");
  (is_read && set.contains("all.read")) || (is_write && set.contains("all.write"))
}

/// Returns `Some(403 Response)` if the user does not have the given permission; otherwise `None`.
pub async fn require_permission<R, U>(
  resolver: &R,
  user: &U,
  db: &DbConnection,
  permission_key: &str,
  scope: Option<&RequestScope>,
) -> Option<Response>
where
  R: PermissionResolver,
  U: axum_login::AuthUser + Send + Sync,
  U::Id: Into<Uuid>,
{
  let user_id = user.id().into();
  let perms = resolver.resolve(db, user_id, scope).await.ok()?;
  if has_permission(&perms, permission_key) {
    None
  } else {
    Some(forbidden_response())
  }
}

/// Returns `Some(403 Response)` if the user does not have the entity action permission; otherwise `None`.
pub async fn require_entity_permission<R, U>(
  resolver: &R,
  user: &U,
  db: &DbConnection,
  scope: Option<&RequestScope>,
  entity: &str,
  action: &str,
) -> Option<Response>
where
  R: PermissionResolver,
  U: axum_login::AuthUser + Send + Sync,
  U::Id: Into<Uuid>,
{
  let key = entity_action_key(entity, action);
  require_permission(resolver, user, db, &key, scope).await
}
