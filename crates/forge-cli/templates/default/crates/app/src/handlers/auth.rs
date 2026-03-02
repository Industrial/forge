use axum::{
  Form, Json,
  extract::{Extension, FromRequest, Request, State},
  http::StatusCode,
  response::IntoResponse,
};
use axum_login::AuthSession;
use chrono::Utc;
use forge::audit::{AuditEvent, EventKind, Outcome};
use forge::auth::{hash_api_token, hash_password};
use forge::authz::{Action, AuthzContext, record_authz_denied};
use forge::token_auth::{OptionalRequireAuth, RequireAuth};
use forge::validation::Valid;
use forge::{DbConnection, Error as ForgeError};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use tower_sessions::Session;
use uuid::Uuid;
use validator::Validate;

use db::auth::Backend;
use db::models::{api_token, membership, organization, role_permission, user};

/// Code-defined dashboard permission keys. Used for resolution and for listing in APIs.
/// Resources have .read (view/list) and .write (create/update/delete) where applicable.
pub const DASHBOARD_PERMISSIONS: &[&str] = &[
  "dashboard",
  "dashboard.organizations.read",
  "dashboard.organizations.write",
  "dashboard.users.read",
  "dashboard.users.write",
  "dashboard.permissions.read",
  "dashboard.permissions.write",
  "dashboard.audit.read",
];

/// Resolves the list of permission keys for the current user. Global admin gets all; otherwise org role's permissions from `role_permission`.
pub async fn resolve_permissions(db: &DbConnection, user: &user::Model) -> Vec<String> {
  if user.is_admin {
    return DASHBOARD_PERMISSIONS
      .iter()
      .map(|s| (*s).to_string())
      .collect();
  }
  let role_name = match user.current_role.as_deref() {
    Some(r) => r,
    None => return vec![],
  };
  let org_id = match user.current_org_id {
    Some(id) => id,
    None => return vec![],
  };
  let rows = role_permission::Entity::find()
    .filter(role_permission::Column::Scope.eq("org"))
    .filter(role_permission::Column::OrgId.eq(org_id))
    .filter(role_permission::Column::RoleName.eq(role_name))
    .all(db)
    .await
    .ok()
    .unwrap_or_default();
  rows.into_iter().map(|r| r.permission_key).collect()
}

/// Session keys for one-time flash messages (read once then cleared). No longer set by auth; kept for session_json shape.
pub const FLASH_MESSAGE: &str = "flash_message";
pub const FLASH_ERROR: &str = "flash_error";

#[derive(Deserialize, Validate)]
pub struct RegisterRequest {
  #[validate(email)]
  pub email: String,
  #[validate(length(min = 8))]
  pub password: String,
}

pub async fn register(
  State(db): State<DbConnection>,
  Valid(Json(payload)): Valid<Json<RegisterRequest>>,
) -> Result<impl IntoResponse, ForgeError> {
  tracing::debug!(target: "app::auth", "route: POST /api/auth/register email={}", payload.email);
  let password_hash = hash_password(&payload.password)?;
  let now = Utc::now().naive_utc();
  let user_id = Uuid::new_v4();
  let org_id = Uuid::new_v4();
  let membership_id = Uuid::new_v4();

  let tx = db.begin().await?;
  let new_user = user::ActiveModel {
    id: Set(user_id),
    email: Set(payload.email.clone()),
    password_hash: Set(password_hash),
    is_active: Set(true),
    is_admin: Set(false),
    current_org_id: Set(None),
    current_role: Set(None),
    created_at: Set(now),
    updated_at: Set(now),
  };
  user::Entity::insert(new_user).exec(&tx).await?;

  let slug = format!("org-{}", org_id.as_simple());
  let new_org = organization::ActiveModel {
    id: Set(org_id),
    name: Set(format!("{}'s workspace", payload.email)),
    slug: Set(slug),
    created_at: Set(now),
    updated_at: Set(now),
  };
  organization::Entity::insert(new_org).exec(&tx).await?;

  let new_membership = membership::ActiveModel {
    id: Set(membership_id),
    user_id: Set(user_id),
    org_id: Set(org_id),
    role: Set("owner".to_string()),
    created_at: Set(now),
    updated_at: Set(now),
  };
  membership::Entity::insert(new_membership).exec(&tx).await?;

  let u = user::Entity::find_by_id(user_id)
    .one(&tx)
    .await?
    .ok_or_else(|| ForgeError::Generic("User not found".into()))?;
  let mut am: user::ActiveModel = u.into();
  am.current_org_id = Set(Some(org_id));
  am.current_role = Set(Some("owner".to_string()));
  am.updated_at = Set(now);
  am.update(&tx).await?;

  tx.commit().await?;

  Ok((StatusCode::CREATED, Json(serde_json::json!({ "ok": true }))).into_response())
}

#[derive(Deserialize, Validate)]
pub struct LoginRequest {
  #[validate(email)]
  pub email: String,
  #[validate(length(min = 1))]
  pub password: String,
}

/// Accepts JSON (Inertia) or form-urlencoded (e.g. e2e native form submit).
pub(crate) struct JsonOrForm<T>(pub(crate) T);

impl<S, T> FromRequest<S> for JsonOrForm<T>
where
  T: DeserializeOwned + Send,
  S: Send + Sync,
{
  type Rejection = (StatusCode, &'static str);

  async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
    let (parts, body) = req.into_parts();
    let content_type = parts.headers.get(axum::http::header::CONTENT_TYPE);
    let is_form = content_type
      .and_then(|v| v.to_str().ok())
      .map(|v| v.starts_with("application/x-www-form-urlencoded"))
      .unwrap_or(false);
    let req = Request::from_parts(parts, body);
    if is_form {
      let Form(payload) = Form::<T>::from_request(req, state)
        .await
        .map_err(|_| (StatusCode::BAD_REQUEST, "invalid form body for login"))?;
      Ok(JsonOrForm(payload))
    } else {
      let Json(payload) = Json::<T>::from_request(req, state)
        .await
        .map_err(|_| (StatusCode::BAD_REQUEST, "invalid JSON body for login"))?;
      Ok(JsonOrForm(payload))
    }
  }
}

fn broadcast_audit_entry(
  task_state: &std::sync::Arc<crate::tasks::TaskState>,
  event: &AuditEvent,
  result: &forge::audit::LogResult,
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
  let _ = task_state.audit_broadcast.send(entry);
}

pub async fn login(
  mut auth_session: AuthSession<Backend>,
  State(db): State<DbConnection>,
  Extension(task_state): Extension<std::sync::Arc<crate::tasks::TaskState>>,
  JsonOrForm(payload): JsonOrForm<LoginRequest>,
) -> Result<impl IntoResponse, ForgeError> {
  let email = payload.email.clone();
  tracing::debug!(target: "app::auth", "route: POST /api/auth/login email={}", email);
  payload
    .validate()
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  let credentials = db::auth::Credentials {
    email: payload.email,
    password: payload.password,
  };

  let user = auth_session
    .authenticate(credentials)
    .await
    .map_err(|e| ForgeError::Generic(format!("Authentication error: {}", e)))?;

  if let Some(ref user) = user {
    tracing::debug!(target: "app::auth", "login success user_id={} email={}", user.id, user.email);
    auth_session
      .login(user)
      .await
      .map_err(|e| ForgeError::Generic(format!("Login error: {}", e)))?;
    let event = AuditEvent {
      event_kind: EventKind::Auth,
      actor_id: user.id,
      subject_id: Some(user.id),
      organization_id: user.current_org_id,
      action: Action::Manage,
      resource_type: "auth".to_string(),
      resource_id: None,
      outcome: Outcome::Success,
      reason: Some("login".to_string()),
    };
    if let Ok(result) = forge::audit::log(&db, event.clone()).await {
      broadcast_audit_entry(&task_state, &event, &result);
    }
    Ok((StatusCode::OK, Json(serde_json::json!({ "ok": true }))).into_response())
  } else {
    tracing::debug!(target: "app::auth", "login failed: invalid credentials email={}", email);
    let event = AuditEvent {
      event_kind: EventKind::Auth,
      actor_id: Uuid::nil(),
      subject_id: None,
      organization_id: None,
      action: Action::Manage,
      resource_type: "auth".to_string(),
      resource_id: None,
      outcome: Outcome::Failure,
      reason: Some("failed_login".to_string()),
    };
    if let Ok(result) = forge::audit::log(&db, event.clone()).await {
      broadcast_audit_entry(&task_state, &event, &result);
    }
    Ok(
      (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({ "error": "Invalid email or password" })),
      )
        .into_response(),
    )
  }
}

pub async fn logout(
  mut auth_session: AuthSession<Backend>,
  State(db): State<DbConnection>,
  Extension(task_state): Extension<std::sync::Arc<crate::tasks::TaskState>>,
) -> impl IntoResponse {
  tracing::debug!(target: "app::auth", "route: GET /api/auth/logout");
  let actor_id = auth_session.requester_id();
  let org_id = auth_session.organization_id();
  auth_session.logout().await.unwrap();
  let event = AuditEvent {
    event_kind: EventKind::Auth,
    actor_id,
    subject_id: Some(actor_id),
    organization_id: org_id,
    action: Action::Manage,
    resource_type: "auth".to_string(),
    resource_id: None,
    outcome: Outcome::Success,
    reason: Some("logout".to_string()),
  };
  if let Ok(result) = forge::audit::log(&db, event.clone()).await {
    broadcast_audit_entry(&task_state, &event, &result);
  }
  (StatusCode::OK, Json(serde_json::json!({ "ok": true }))).into_response()
}

pub async fn profile(auth_session: AuthSession<Backend>) -> impl IntoResponse {
  tracing::debug!(target: "app::auth", "route: GET /api/auth/profile has_user={}", auth_session.user.is_some());
  match &auth_session.user {
    Some(user) => Json(serde_json::json!({
      "user": { "id": user.id.to_string(), "email": user.email }
    }))
    .into_response(),
    None => (
      StatusCode::UNAUTHORIZED,
      Json(serde_json::json!({ "error": "Not logged in" })),
    )
      .into_response(),
  }
}

/// List of profiles (org + role) the current user can switch to.
pub async fn profiles_list(
  RequireAuth(user): RequireAuth<Backend>,
  State(db): State<DbConnection>,
) -> Result<impl IntoResponse, ForgeError> {
  let memberships = membership::Entity::find()
    .filter(membership::Column::UserId.eq(user.id))
    .all(&db)
    .await?;
  let org_ids: Vec<Uuid> = memberships.iter().map(|m| m.org_id).collect();
  if org_ids.is_empty() {
    return Ok(Json(serde_json::json!({ "profiles": [] })).into_response());
  }
  let orgs = organization::Entity::find()
    .filter(organization::Column::Id.is_in(org_ids))
    .all(&db)
    .await?;
  let org_map: std::collections::HashMap<Uuid, organization::Model> =
    orgs.into_iter().map(|o| (o.id, o)).collect();
  let profiles: Vec<serde_json::Value> = memberships
    .into_iter()
    .filter_map(|m| {
      org_map.get(&m.org_id).map(|o| {
        serde_json::json!({
          "org_id": m.org_id.to_string(),
          "org_name": o.name,
          "role": m.role,
        })
      })
    })
    .collect();
  Ok(Json(serde_json::json!({ "profiles": profiles })).into_response())
}

#[derive(Deserialize)]
pub struct SwitchProfileRequest {
  pub org_id: String,
}

/// Switch the current user's active profile (org + role). Updates user row; session will see new context on next request.
pub async fn switch_profile(
  RequireAuth(user): RequireAuth<Backend>,
  State(db): State<DbConnection>,
  Json(payload): Json<SwitchProfileRequest>,
) -> Result<impl IntoResponse, ForgeError> {
  let org_id = Uuid::parse_str(&payload.org_id).map_err(|_| ForgeError::Generic("Invalid org_id".into()))?;
  let membership = membership::Entity::find()
    .filter(membership::Column::UserId.eq(user.id))
    .filter(membership::Column::OrgId.eq(org_id))
    .one(&db)
    .await?
    .ok_or_else(|| ForgeError::Generic("Membership not found".into()))?;
  let mut am: user::ActiveModel = user::Entity::find_by_id(user.id)
    .one(&db)
    .await?
    .ok_or_else(|| ForgeError::Generic("User not found".into()))?
    .into();
  am.current_org_id = Set(Some(org_id));
  am.current_role = Set(Some(membership.role));
  am.updated_at = Set(Utc::now().naive_utc());
  am.update(&db).await?;
  Ok(Json(serde_json::json!({ "ok": true })).into_response())
}

/// Returns session state for the SPA: current user, profiles (org+role list), and flash (consumed on read).
pub async fn session_json(
  session: Session,
  OptionalRequireAuth(maybe_user): OptionalRequireAuth<Backend>,
  State(db): State<DbConnection>,
) -> impl IntoResponse {
  let message: Option<String> = session.get(FLASH_MESSAGE).await.ok().flatten();
  let error: Option<String> = session.get(FLASH_ERROR).await.ok().flatten();
  session.remove::<String>(FLASH_MESSAGE).await.ok();
  session.remove::<String>(FLASH_ERROR).await.ok();
  let user = maybe_user.as_ref().map(|u| serde_json::json!({ "id": u.id.to_string(), "email": u.email }));
  let permissions: Vec<String> = match &maybe_user {
    Some(u) => resolve_permissions(&db, u).await,
    None => vec![],
  };
  let profiles: Vec<serde_json::Value> = match &maybe_user {
    Some(u) => {
      let memberships = membership::Entity::find()
        .filter(membership::Column::UserId.eq(u.id))
        .all(&db)
        .await
        .ok()
        .unwrap_or_default();
      let org_ids: Vec<Uuid> = memberships.iter().map(|m| m.org_id).collect();
      if org_ids.is_empty() {
        vec![]
      } else {
        let orgs = organization::Entity::find()
          .filter(organization::Column::Id.is_in(org_ids))
          .all(&db)
          .await
          .ok()
          .unwrap_or_default();
        let org_map: std::collections::HashMap<Uuid, organization::Model> =
          orgs.into_iter().map(|o| (o.id, o)).collect();
        memberships
          .into_iter()
          .filter_map(|m| {
            org_map.get(&m.org_id).map(|o| {
              serde_json::json!({
                "org_id": m.org_id.to_string(),
                "org_name": o.name,
                "role": m.role,
              })
            })
          })
          .collect()
      }
    }
    None => vec![],
  };
  Json(serde_json::json!({
    "user": user,
    "profiles": profiles,
    "permissions": permissions,
    "flash": { "message": message, "error": error }
  }))
}

#[derive(Deserialize, Validate)]
pub struct CreateTokenRequest {
  pub name: Option<String>,
}

pub async fn create_token(
  RequireAuth(user): RequireAuth<Backend>,
  State(db): State<DbConnection>,
  Valid(Json(payload)): Valid<Json<CreateTokenRequest>>,
) -> Result<impl IntoResponse, ForgeError> {
  tracing::debug!(target: "app::auth", "route: POST /api/auth/tokens user_id={}", user.id);
  let secret = format!("forge_{}", Uuid::new_v4().to_string().replace('-', ""));
  let token_hash = hash_api_token(&secret);
  let now = Utc::now().naive_utc();
  let id = Uuid::new_v4();
  let model = api_token::ActiveModel {
    id: Set(id),
    user_id: Set(user.id),
    token_hash: Set(token_hash),
    name: Set(payload.name),
    last_used_at: Set(None),
    expires_at: Set(None),
    created_at: Set(now),
    updated_at: Set(now),
  };
  api_token::Entity::insert(model).exec(&db).await?;
  Ok(Json(forge::serde_json::json!({ "token": secret })))
}

/// Global admin only: only users with is_admin can access. Audits the decision.
pub async fn admin_only(
  OptionalRequireAuth(maybe_user): OptionalRequireAuth<Backend>,
  State(db): State<DbConnection>,
) -> Result<impl IntoResponse, ForgeError> {
  tracing::debug!(target: "app::auth", "route: GET /api/auth/admin authenticated={}", maybe_user.is_some());
  let user = match maybe_user {
    Some(u) => u,
    None => {
      record_authz_denied(&db, Action::Manage, "admin", None).await;
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
    record_authz_denied(&db, Action::Manage, "admin", Some(user.id)).await;
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
  use validator::Validate;

  #[test]
  fn register_request_valid_email_and_password_passes() {
    let r = RegisterRequest {
      email: "user@example.com".to_string(),
      password: "password123".to_string(),
    };
    assert!(r.validate().is_ok());
  }

  #[test]
  fn register_request_invalid_email_fails() {
    let r = RegisterRequest {
      email: "not-an-email".to_string(),
      password: "password123".to_string(),
    };
    assert!(r.validate().is_err());
  }

  #[test]
  fn register_request_short_password_fails() {
    let r = RegisterRequest {
      email: "user@example.com".to_string(),
      password: "short".to_string(),
    };
    assert!(r.validate().is_err());
  }

  #[test]
  fn login_request_valid_passes() {
    let r = LoginRequest {
      email: "user@example.com".to_string(),
      password: "anything".to_string(),
    };
    assert!(r.validate().is_ok());
  }

  #[test]
  fn login_request_invalid_email_fails() {
    let r = LoginRequest {
      email: "bad".to_string(),
      password: "x".to_string(),
    };
    assert!(r.validate().is_err());
  }

  #[test]
  fn flash_constants_are_non_empty() {
    assert!(!FLASH_MESSAGE.is_empty());
    assert!(!FLASH_ERROR.is_empty());
  }
}
