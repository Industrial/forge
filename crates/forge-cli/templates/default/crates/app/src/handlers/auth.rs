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
use forge::live::{Channel, InMemoryLiveBackend, LiveBackend};
use forge::{DbConnection, Error as ForgeError};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use tower_sessions::Session;
use uuid::Uuid;
use validator::Validate;

use db::auth::Backend;
use db::models::{api_token, membership, org_role, organization, role_permission, user, user_global_role, user_org_role};

/// Code-defined dashboard permission keys. Used for resolution and for listing in APIs.
/// Resources have .read (view/list) and .write (create/update/delete) where applicable.
pub const DASHBOARD_PERMISSIONS: &[&str] = &[
  "dashboard",
  "dashboard.organizations.read",
  "dashboard.organizations.write",
  "dashboard.users.read",
  "dashboard.users.write",
  "dashboard.roles.read",
  "dashboard.roles.write",
  "dashboard.permissions.read",
  "dashboard.permissions.write",
  "dashboard.audit.read",
];

/// Resolves the list of permission keys for the current user from org-scoped and global-scope role_permission.
/// Uses session profile when provided; otherwise only global-scope permissions are included.
pub async fn resolve_permissions(db: &DbConnection, user: &user::Model, profile: Option<&CurrentProfile>) -> Vec<String> {
  let mut keys = std::collections::HashSet::<String>::new();

  // Global-scope: user_global_role -> role_permission with scope=global and org_id=null.
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

  // Org-scoped: session profile (org + role) from role_permission.
  if let Some(p) = profile {
    let rows = role_permission::Entity::find()
      .filter(role_permission::Column::Scope.eq("org"))
      .filter(role_permission::Column::OrgId.eq(p.org_id))
      .filter(role_permission::Column::RoleName.eq(&p.role_name))
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

/// True if the user has the given permission at global scope (via user_global_role + role_permission scope=global).
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
  let exists = role_permission::Entity::find()
    .filter(role_permission::Column::Scope.eq("global"))
    .filter(role_permission::Column::OrgId.is_null())
    .filter(role_permission::Column::RoleName.is_in(global_roles))
    .filter(role_permission::Column::PermissionKey.eq(permission_key))
    .one(db)
    .await
    .ok()
    .flatten();
  exists.is_some()
}

/// Derives live-update channel subscriptions from the user's permissions and current org.
/// One channel per permission scope (see docs/021_live_channels_and_permissions.md).
pub fn channels_from_permissions(
  permissions: &[String],
  current_org_id: Option<Uuid>,
) -> Vec<Channel> {
  let perms: std::collections::HashSet<_> = permissions.iter().map(String::as_str).collect();
  let mut out = Vec::new();
  if perms.contains("dashboard.organizations.read") {
    out.push(Channel::raw("organizations"));
  }
  if perms.contains("dashboard.audit.read") {
    out.push(Channel::raw("audit-log"));
  }
  // Tasks: any authenticated user gets the demo tasks channel for the template.
  out.push(Channel::raw("tasks"));
  if let Some(org_id) = current_org_id {
    if perms.contains("dashboard.users.read") {
      out.push(Channel::org_resource(org_id, "users"));
    }
    if perms.contains("dashboard.roles.read") {
      out.push(Channel::org_resource(org_id, "roles"));
    }
    if perms.contains("dashboard.permissions.read") {
      out.push(Channel::org_resource(org_id, "role_permissions"));
    }
  }
  out
}

/// Session keys for one-time flash messages (read once then cleared). No longer set by auth; kept for session_json shape.
pub const FLASH_MESSAGE: &str = "flash_message";
pub const FLASH_ERROR: &str = "flash_error";

/// Session keys for the active profile (org + role). Source of truth for dashboard scope; not stored on user row.
pub const SESSION_CURRENT_ORG_ID: &str = "current_org_id";
pub const SESSION_CURRENT_ROLE_NAME: &str = "current_role_name";

/// Active profile for the current request: org and role from session. Use [get_profile_from_session] or [require_profile] in handlers.
#[derive(Clone, Debug)]
pub struct CurrentProfile {
  pub org_id: Uuid,
  pub role_name: String,
}

/// Reads current profile from session if both org_id and role_name are set.
pub async fn get_profile_from_session(session: &Session) -> Option<CurrentProfile> {
  let org_id: Uuid = session.get(SESSION_CURRENT_ORG_ID).await.ok().flatten()?;
  let role_name: String = session.get(SESSION_CURRENT_ROLE_NAME).await.ok().flatten()?;
  Some(CurrentProfile { org_id, role_name })
}

/// Requires a profile in session; returns 403 with code `profile_required` if missing (frontend can redirect to profile-select).
pub async fn require_profile(
  session: &Session,
) -> Result<CurrentProfile, (StatusCode, Json<serde_json::Value>)> {
  match get_profile_from_session(session).await {
    Some(p) => Ok(p),
    None => Err((
      StatusCode::FORBIDDEN,
      Json(serde_json::json!({ "code": "profile_required", "error": "Please select a profile" })),
    )),
  }
}

#[derive(Deserialize, Validate)]
pub struct RegisterRequest {
  #[validate(email)]
  pub email: String,
  #[validate(length(min = 8))]
  pub password: String,
}

pub async fn register(
  mut auth_session: AuthSession<Backend>,
  session: Session,
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

  for &name in &["owner", "admin", "editor", "viewer"] {
    let role_id = Uuid::new_v4();
    let r = org_role::ActiveModel {
      id: Set(role_id),
      org_id: Set(org_id),
      name: Set(name.to_string()),
      display_name: Set(None),
      created_at: Set(now),
      updated_at: Set(now),
      ..Default::default()
    };
    org_role::Entity::insert(r).exec(&tx).await?;
  }
  db::seed_role_permissions_for_org(&tx, org_id)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;

  let new_membership = membership::ActiveModel {
    id: Set(membership_id),
    user_id: Set(user_id),
    org_id: Set(org_id),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  };
  membership::Entity::insert(new_membership).exec(&tx).await?;

  let owner_role = org_role::Entity::find()
    .filter(org_role::Column::OrgId.eq(org_id))
    .filter(org_role::Column::Name.eq("owner"))
    .one(&tx)
    .await?
    .ok_or_else(|| ForgeError::Generic("org_role owner not found".into()))?;
  let uor_id = Uuid::new_v4();
  let uor = user_org_role::ActiveModel {
    id: Set(uor_id),
    user_id: Set(user_id),
    org_id: Set(org_id),
    role_id: Set(owner_role.id),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  };
  user_org_role::Entity::insert(uor).exec(&tx).await?;

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

  let user = user::Entity::find_by_id(user_id)
    .one(&db)
    .await?
    .ok_or_else(|| ForgeError::Generic("User not found after register".into()))?;
  auth_session
    .login(&user)
    .await
    .map_err(|e| ForgeError::Generic(format!("Login after register: {}", e)))?;
  session.insert(SESSION_CURRENT_ORG_ID, org_id).await.ok();
  session.insert(SESSION_CURRENT_ROLE_NAME, "owner".to_string()).await.ok();

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
pub struct JsonOrForm<T>(pub T);

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

async fn broadcast_audit_entry(
  live_backend: Option<&std::sync::Arc<InMemoryLiveBackend>>,
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
  if let Some(backend) = live_backend {
    let payload =
      serde_json::to_vec(&serde_json::json!({ "type": "audit_log", "entry": entry })).unwrap_or_default();
    let channel = Channel::raw("audit-log");
    backend.broadcast(&channel, &payload).await;
  }
}

pub async fn login(
  mut auth_session: AuthSession<Backend>,
  session: Session,
  State(db): State<DbConnection>,
  Extension(live_backend): Extension<Option<std::sync::Arc<InMemoryLiveBackend>>>,
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
    let uors = user_org_role::Entity::find()
      .filter(user_org_role::Column::UserId.eq(user.id))
      .all(&db)
      .await
      .unwrap_or_default();
    let needs_profile_select = if uors.len() == 1 {
      let uor = &uors[0];
      if let Ok(Some(role)) = org_role::Entity::find_by_id(uor.role_id).one(&db).await {
        session.insert(SESSION_CURRENT_ORG_ID, uor.org_id).await.ok();
        session.insert(SESSION_CURRENT_ROLE_NAME, role.name.clone()).await.ok();
        false
      } else {
        true
      }
    } else {
      true
    };
    let event = AuditEvent {
      event_kind: EventKind::Auth,
      actor_id: user.id,
      subject_id: Some(user.id),
      organization_id: if uors.len() == 1 { Some(uors[0].org_id) } else { None },
      action: Action::Manage,
      resource_type: "auth".to_string(),
      resource_id: None,
      outcome: Outcome::Success,
      reason: Some("login".to_string()),
    };
    if let Ok(result) = forge::audit::log(&db, event.clone()).await {
      broadcast_audit_entry(live_backend.as_ref(), &event, &result).await;
    }
    Ok((StatusCode::OK, Json(serde_json::json!({ "ok": true, "needs_profile_select": needs_profile_select }))).into_response())
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
      broadcast_audit_entry(live_backend.as_ref(), &event, &result).await;
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
  Extension(live_backend): Extension<Option<std::sync::Arc<InMemoryLiveBackend>>>,
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
    broadcast_audit_entry(live_backend.as_ref(), &event, &result).await;
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

/// List of profiles (org + role) the current user can switch to. One entry per (org, role).
pub async fn profiles_list(
  RequireAuth(user): RequireAuth<Backend>,
  State(db): State<DbConnection>,
) -> Result<impl IntoResponse, ForgeError> {
  let uors = user_org_role::Entity::find()
    .filter(user_org_role::Column::UserId.eq(user.id))
    .all(&db)
    .await?;
  if uors.is_empty() {
    return Ok(Json(serde_json::json!({ "profiles": [] })).into_response());
  }
  let role_ids: Vec<Uuid> = uors.iter().map(|u| u.role_id).collect();
  let roles = org_role::Entity::find()
    .filter(org_role::Column::Id.is_in(role_ids))
    .all(&db)
    .await?;
  let org_ids: Vec<Uuid> = uors.iter().map(|u| u.org_id).collect::<std::collections::HashSet<_>>().into_iter().collect();
  let orgs = organization::Entity::find()
    .filter(organization::Column::Id.is_in(org_ids))
    .all(&db)
    .await?;
  let org_map: std::collections::HashMap<Uuid, organization::Model> =
    orgs.into_iter().map(|o| (o.id, o)).collect();
  let role_map: std::collections::HashMap<Uuid, org_role::Model> =
    roles.into_iter().map(|r| (r.id, r)).collect();
  let profiles: Vec<serde_json::Value> = uors
    .into_iter()
    .filter_map(|u| {
      let r = role_map.get(&u.role_id)?;
      let o = org_map.get(&u.org_id)?;
      Some(serde_json::json!({
        "org_id": u.org_id.to_string(),
        "org_name": o.name,
        "role_id": u.role_id.to_string(),
        "role": r.name,
      }))
    })
    .collect();
  Ok(Json(serde_json::json!({ "profiles": profiles })).into_response())
}

#[derive(Deserialize)]
pub struct SwitchProfileRequest {
  pub org_id: String,
  pub role_id: Option<String>,
}

/// Set the current session's active profile (org + role). Writes to session only; does not update user row.
pub async fn set_profile(
  session: Session,
  RequireAuth(user): RequireAuth<Backend>,
  State(db): State<DbConnection>,
  Json(payload): Json<SwitchProfileRequest>,
) -> Result<impl IntoResponse, ForgeError> {
  let org_id = Uuid::parse_str(&payload.org_id).map_err(|_| ForgeError::Generic("Invalid org_id".into()))?;
  let role_name = if let Some(rid) = &payload.role_id {
    let role_id = Uuid::parse_str(rid).map_err(|_| ForgeError::Generic("Invalid role_id".into()))?;
    let uor = user_org_role::Entity::find()
      .filter(user_org_role::Column::UserId.eq(user.id))
      .filter(user_org_role::Column::OrgId.eq(org_id))
      .filter(user_org_role::Column::RoleId.eq(role_id))
      .one(&db)
      .await?
      .ok_or_else(|| ForgeError::Generic("Profile not found".into()))?;
    let role_row = org_role::Entity::find_by_id(uor.role_id)
      .one(&db)
      .await?
      .ok_or_else(|| ForgeError::Generic("Role not found".into()))?;
    role_row.name
  } else {
    let uor = user_org_role::Entity::find()
      .filter(user_org_role::Column::UserId.eq(user.id))
      .filter(user_org_role::Column::OrgId.eq(org_id))
      .one(&db)
      .await?
      .ok_or_else(|| ForgeError::Generic("Membership not found".into()))?;
    let role_row = org_role::Entity::find_by_id(uor.role_id)
      .one(&db)
      .await?
      .ok_or_else(|| ForgeError::Generic("Role not found".into()))?;
    role_row.name
  };
  session.insert(SESSION_CURRENT_ORG_ID, org_id).await.ok();
  session.insert(SESSION_CURRENT_ROLE_NAME, role_name.clone()).await.ok();
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
  let session_org_id: Option<Uuid> = session.get(SESSION_CURRENT_ORG_ID).await.ok().flatten();
  let session_role_name: Option<String> = session.get(SESSION_CURRENT_ROLE_NAME).await.ok().flatten();
  let session_profile = session_org_id.and_then(|o| session_role_name.clone().map(|r| CurrentProfile { org_id: o, role_name: r }));
  let user = maybe_user.as_ref().map(|u| serde_json::json!({
    "id": u.id.to_string(),
    "email": u.email,
    "current_org_id": session_org_id.map(|id| id.to_string()),
    "current_role_name": session_role_name,
  }));
  let permissions: Vec<String> = match &maybe_user {
    Some(u) => resolve_permissions(&db, u, session_profile.as_ref()).await,
    None => vec![],
  };
  let profiles: Vec<serde_json::Value> = match &maybe_user {
    Some(u) => {
      let uors = user_org_role::Entity::find()
        .filter(user_org_role::Column::UserId.eq(u.id))
        .all(&db)
        .await
        .ok()
        .unwrap_or_default();
      if uors.is_empty() {
        vec![]
      } else {
        let role_ids: Vec<Uuid> = uors.iter().map(|x| x.role_id).collect();
        let roles = org_role::Entity::find()
          .filter(org_role::Column::Id.is_in(role_ids))
          .all(&db)
          .await
          .ok()
          .unwrap_or_default();
        let org_ids: Vec<Uuid> = uors.iter().map(|x| x.org_id).collect::<std::collections::HashSet<_>>().into_iter().collect();
        let orgs = organization::Entity::find()
          .filter(organization::Column::Id.is_in(org_ids))
          .all(&db)
          .await
          .ok()
          .unwrap_or_default();
        let org_map: std::collections::HashMap<Uuid, organization::Model> =
          orgs.into_iter().map(|o| (o.id, o)).collect();
        let role_map: std::collections::HashMap<Uuid, org_role::Model> =
          roles.into_iter().map(|r| (r.id, r)).collect();
        uors
          .into_iter()
          .filter_map(|u| {
            let r = role_map.get(&u.role_id)?;
            let o = org_map.get(&u.org_id)?;
            Some(serde_json::json!({
              "org_id": u.org_id.to_string(),
              "org_name": o.name,
              "role_id": u.role_id.to_string(),
              "role": r.name,
            }))
          })
          .collect()
      }
    }
    None => vec![],
  };
  let needs_profile_select = maybe_user.is_some() && (session_org_id.is_none() || session_role_name.is_none());
  Json(serde_json::json!({
    "user": user,
    "profiles": profiles,
    "permissions": permissions,
    "flash": { "message": message, "error": error },
    "needs_profile_select": needs_profile_select
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

  #[test]
  fn current_profile_holds_org_id_and_role_name() {
    let org_id = Uuid::new_v4();
    let profile = CurrentProfile {
      org_id,
      role_name: "viewer".to_string(),
    };
    assert_eq!(profile.org_id, org_id);
    assert_eq!(profile.role_name, "viewer");
  }
}
