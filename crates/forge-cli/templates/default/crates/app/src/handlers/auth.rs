use crate::Error as ForgeError;
use axum::{
  Form, Json,
  extract::{Extension, FromRequest, FromRequestParts, Request, State},
  http::StatusCode,
  http::request::Parts,
  response::IntoResponse,
};
use chrono::Utc;
use forge_audit::record_authz_denied;
use forge_audit::{AuditEvent, EventKind, LogResult, Outcome, log};
use forge_auth::token_auth::{
  OptionalRequireAuth, RequireAuth, TokenUser, hash_api_token, hash_password,
};
use forge_auth::{Action, RequestScope};
use forge_core::Valid;
use forge_db::DbConnection;
use forge_live::{Channel, InMemoryLiveBackend, LiveBackend};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use uuid::Uuid;
use validator::Validate;

use axum_login::AuthnBackend;
use db::auth::Backend;
use db::models::{
  api_token, membership, org_role, organization, role_permission, user, user_global_role,
  user_org_role,
};

pub use crate::permissions::DASHBOARD_PERMISSIONS;

/// Resolves the list of permission keys for the current user from org-scoped and global-scope role_permission.
/// Uses scope (X-Organization-Id, X-Role-Id) when provided; otherwise only global-scope permissions are included.
pub async fn resolve_permissions(
  db: &DbConnection,
  user: &user::Model,
  scope: Option<&RequestScope>,
) -> Vec<String> {
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

  // Org-scoped: request scope (org + role) from role_permission.
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

/// True if the user has the given permission at global scope (via user_global_role + role_permission scope=global).
/// Global `all.read` grants any read permission (e.g. `*.read`); global `all.write` grants any write permission.
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
  let is_write = permission_key == "all.write" || permission_key.ends_with(".write");
  (is_read && keys.contains("all.read")) || (is_write && keys.contains("all.write"))
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

/// Header names for scope (X-Organization-Id, X-Role-Id).
pub const HEADER_ORGANIZATION_ID: &str = "x-organization-id";
pub const HEADER_ROLE_ID: &str = "x-role-id";

/// Builds [RequestScope] from headers (X-Organization-Id, X-Role-Id). Validates org, role, and user membership.
pub async fn try_scope_from_headers(
  headers: &axum::http::HeaderMap,
  db: &DbConnection,
  user_id: Uuid,
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
  let org_id = Uuid::parse_str(org_id_str).map_err(|_| {
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
  let role_id = Uuid::parse_str(role_id_str)
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

/// Extractor: requires Bearer auth and X-Organization-Id + X-Role-Id headers; validates user has that role in org.
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
      .get::<TokenUser<user::Model>>()
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

/// Optional scope from request headers; returns None if headers missing or invalid.
pub async fn get_scope_from_headers(
  parts: &Parts,
  user: &user::Model,
  db: &DbConnection,
) -> Option<RequestScope> {
  get_scope_from_headers_map(&parts.headers, user, db).await
}

/// Optional scope from a [axum::http::HeaderMap]; use from handlers that have the request (e.g. WebSocket upgrade).
pub async fn get_scope_from_headers_map(
  headers: &axum::http::HeaderMap,
  user: &user::Model,
  db: &DbConnection,
) -> Option<RequestScope> {
  try_scope_from_headers(headers, db, user.id).await.ok()
}

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
  let password_hash =
    hash_password(&payload.password).map_err(|e| ForgeError::Generic(e.to_string()))?;
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

  let secret = format!("forge_{}", Uuid::new_v4().to_string().replace('-', ""));
  let token_hash = hash_api_token(&secret);
  let token_id = Uuid::new_v4();
  let token_model = api_token::ActiveModel {
    id: Set(token_id),
    user_id: Set(user_id),
    token_hash: Set(token_hash),
    name: Set(Some("Registration".to_string())),
    last_used_at: Set(None),
    expires_at: Set(None),
    created_at: Set(now),
    updated_at: Set(now),
  };
  api_token::Entity::insert(token_model).exec(&db).await?;

  Ok(
    (
      StatusCode::CREATED,
      Json(serde_json::json!({ "ok": true, "token": secret })),
    )
      .into_response(),
  )
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
    let channel = Channel::raw("audit-log");
    backend.broadcast(&channel, &payload).await;
  }
}

pub async fn login(
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

  let backend = Backend::new(db.clone());
  let user = backend
    .authenticate(credentials)
    .await
    .map_err(|e| ForgeError::Generic(format!("Authentication error: {}", e)))?;

  if let Some(ref user) = user {
    tracing::debug!(target: "app::auth", "login success user_id={} email={}", user.id, user.email);
    let uors = user_org_role::Entity::find()
      .filter(user_org_role::Column::UserId.eq(user.id))
      .all(&db)
      .await
      .unwrap_or_default();
    let needs_profile_select = uors.len() != 1;

    let secret = format!("forge_{}", Uuid::new_v4().to_string().replace('-', ""));
    let token_hash = hash_api_token(&secret);
    let now = Utc::now().naive_utc();
    let token_id = Uuid::new_v4();
    let token_model = api_token::ActiveModel {
      id: Set(token_id),
      user_id: Set(user.id),
      token_hash: Set(token_hash),
      name: Set(Some("Login".to_string())),
      last_used_at: Set(None),
      expires_at: Set(None),
      created_at: Set(now),
      updated_at: Set(now),
    };
    api_token::Entity::insert(token_model).exec(&db).await?;

    let event = AuditEvent {
      event_kind: EventKind::Auth,
      actor_id: user.id,
      subject_id: Some(user.id),
      organization_id: if uors.len() == 1 {
        Some(uors[0].org_id)
      } else {
        None
      },
      action: Action::Manage,
      resource_type: "auth".to_string(),
      resource_id: None,
      outcome: Outcome::Success,
      reason: Some("login".to_string()),
    };
    if let Ok(result) = log(&db, event.clone()).await {
      broadcast_audit_entry(live_backend.as_ref(), &event, &result).await;
    }
    Ok(
      (
        StatusCode::OK,
        Json(serde_json::json!({
          "ok": true,
          "token": secret,
          "needs_profile_select": needs_profile_select
        })),
      )
        .into_response(),
    )
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
    if let Ok(result) = log(&db, event.clone()).await {
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

/// GET /api/auth/me — current user, profiles, permissions (from optional scope headers), and needs_profile_select.
/// Frontend uses this on load; if the request includes X-Organization-Id and X-Role-Id, permissions are org-scoped.
pub async fn get_me(
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
  req: Request,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  let scope = get_scope_from_headers_map(req.headers(), user, &db).await;
  let permissions = resolve_permissions(&db, user, scope.as_ref()).await;
  let uors = user_org_role::Entity::find()
    .filter(user_org_role::Column::UserId.eq(user.id))
    .all(&db)
    .await?;
  let profiles: Vec<serde_json::Value> = if uors.is_empty() {
    vec![]
  } else {
    let role_ids: Vec<Uuid> = uors.iter().map(|u| u.role_id).collect();
    let roles = org_role::Entity::find()
      .filter(org_role::Column::Id.is_in(role_ids))
      .all(&db)
      .await?;
    let org_ids: Vec<Uuid> = uors
      .iter()
      .map(|u| u.org_id)
      .collect::<std::collections::HashSet<_>>()
      .into_iter()
      .collect();
    let orgs = organization::Entity::find()
      .filter(organization::Column::Id.is_in(org_ids))
      .all(&db)
      .await?;
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
  };
  let needs_profile_select = profiles.len() != 1;
  Ok(Json(serde_json::json!({
    "user": { "id": user.id.to_string(), "email": user.email },
    "profiles": profiles,
    "permissions": permissions,
    "flash": serde_json::Value::Null,
    "needs_profile_select": needs_profile_select,
  })))
}

/// Client should discard the token; no server-side session to clear.
pub async fn logout() -> impl IntoResponse {
  tracing::debug!(target: "app::auth", "route: GET /api/auth/logout");
  (StatusCode::OK, Json(serde_json::json!({ "ok": true }))).into_response()
}

/// List of profiles (org + role) the current user can switch to. One entry per (org, role).
pub async fn profiles_list(
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
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
  let org_ids: Vec<Uuid> = uors
    .iter()
    .map(|u| u.org_id)
    .collect::<std::collections::HashSet<_>>()
    .into_iter()
    .collect();
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

#[derive(Deserialize, Validate)]
pub struct CreateTokenRequest {
  pub name: Option<String>,
}

pub async fn create_token(
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
  Valid(Json(payload)): Valid<Json<CreateTokenRequest>>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
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
  Ok(Json(serde_json::json!({ "token": secret })))
}

/// Global admin only: only users with is_admin can access. Audits the decision.
pub async fn admin_only(
  opt_auth: OptionalRequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
) -> Result<impl IntoResponse, ForgeError> {
  let maybe_user = opt_auth.0;
  tracing::debug!(target: "app::auth", "route: GET /api/auth/admin authenticated={}", maybe_user.is_some());
  let user = match maybe_user {
    Some(u) => u,
    None => {
      record_authz_denied(&db, Action::Manage, "admin", None, None).await;
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
    record_authz_denied(&db, Action::Manage, "admin", Some(user.id), None).await;
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
  fn request_scope_holds_org_id_role_id_and_role_name() {
    let org_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();
    let scope = RequestScope {
      organization_id: org_id,
      role_id,
      role_name: "viewer".to_string(),
    };
    assert_eq!(scope.organization_id, org_id);
    assert_eq!(scope.role_id, role_id);
    assert_eq!(scope.role_name, "viewer");
  }
}
