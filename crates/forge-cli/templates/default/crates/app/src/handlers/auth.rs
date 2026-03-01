use axum::{
  extract::{FromRequest, Request, State},
  http::StatusCode,
  response::{IntoResponse, Redirect},
  Form, Json,
};
use axum_login::AuthSession;
use tower_sessions::Session;
use chrono::Utc;
use forge::auth::{hash_api_token, hash_password};
use forge::audit::{AuditEvent, EventKind, Outcome};
use forge::authz::{Action, record_authz_denied, AuthzContext};
use forge::validation::Valid;
use forge::token_auth::{OptionalRequireAuth, RequireAuth};
use forge::{DbConnection, Error as ForgeError};
use sea_orm::{ActiveModelTrait, EntityTrait, Set, TransactionTrait};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::handlers::inertia_shared;
use db::auth::Backend;
use db::models::{api_token, organization, membership, user};

#[derive(Deserialize, Validate)]
pub struct RegisterRequest {
  #[validate(email)]
  pub email: String,
  #[validate(length(min = 8))]
  pub password: String,
}

pub async fn register(
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

  let new_membership = membership::ActiveModel {
    id: Set(membership_id),
    user_id: Set(user_id),
    org_id: Set(org_id),
    role: Set("owner".to_string()),
    created_at: Set(now),
    updated_at: Set(now),
  };
  membership::Entity::insert(new_membership).exec(&tx).await?;

  let u = user::Entity::find_by_id(user_id).one(&tx).await?.ok_or_else(|| ForgeError::Generic("User not found".into()))?;
  let mut am: user::ActiveModel = u.into();
  am.current_org_id = Set(Some(org_id));
  am.current_role = Set(Some("owner".to_string()));
  am.updated_at = Set(now);
  am.update(&tx).await?;

  tx.commit().await?;

  session
    .insert(inertia_shared::FLASH_MESSAGE, "Thanks for registering. Please log in.")
    .await
    .ok();
  Ok(Redirect::to("/login").into_response())
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

pub async fn login(
  session: Session,
  mut auth_session: AuthSession<Backend>,
  State(db): State<DbConnection>,
  JsonOrForm(payload): JsonOrForm<LoginRequest>,
) -> Result<impl IntoResponse, ForgeError> {
  let email = payload.email.clone();
  tracing::debug!(target: "app::auth", "route: POST /api/auth/login email={}", email);
  payload.validate().map_err(|e| ForgeError::Generic(e.to_string()))?;
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
    let _ = forge::audit::log(
      &db,
      AuditEvent {
        event_kind: EventKind::Auth,
        actor_id: user.id,
        subject_id: Some(user.id),
        organization_id: user.current_org_id,
        action: Action::Manage,
        resource_type: "auth".to_string(),
        resource_id: None,
        outcome: Outcome::Success,
        reason: Some("login".to_string()),
      },
    )
    .await;
    session
      .insert(inertia_shared::FLASH_MESSAGE, "Welcome back!")
      .await
      .ok();
    // 303 See Other: correct for POST→GET redirect (RFC 7231). Axum's Redirect::to() uses 303.
    Ok(Redirect::to("/dashboard").into_response())
  } else {
    tracing::debug!(target: "app::auth", "login failed: invalid credentials email={}", email);
    let _ = forge::audit::log(
      &db,
      AuditEvent {
        event_kind: EventKind::Auth,
        actor_id: Uuid::nil(),
        subject_id: None,
        organization_id: None,
        action: Action::Manage,
        resource_type: "auth".to_string(),
        resource_id: None,
        outcome: Outcome::Failure,
        reason: Some("failed_login".to_string()),
      },
    )
    .await;
    Ok((StatusCode::UNAUTHORIZED, "Invalid credentials").into_response())
  }
}

pub async fn logout(
  mut auth_session: AuthSession<Backend>,
  State(db): State<DbConnection>,
) -> impl IntoResponse {
  tracing::debug!(target: "app::auth", "route: GET /api/auth/logout");
  let actor_id = auth_session.requester_id();
  let org_id = auth_session.organization_id();
  auth_session.logout().await.unwrap();
  let _ = forge::audit::log(
    &db,
    AuditEvent {
      event_kind: EventKind::Auth,
      actor_id,
      subject_id: Some(actor_id),
      organization_id: org_id,
      action: Action::Manage,
      resource_type: "auth".to_string(),
      resource_id: None,
      outcome: Outcome::Success,
      reason: Some("logout".to_string()),
    },
  )
  .await;
  StatusCode::OK
}

pub async fn profile(auth_session: AuthSession<Backend>) -> impl IntoResponse {
  tracing::debug!(target: "app::auth", "route: GET /api/auth/profile has_user={}", auth_session.user.is_some());
  match &auth_session.user {
    Some(user) => {
      let org_id = auth_session.organization_id().expect("profile requires organization_id");
      let role = auth_session.role().expect("profile requires role");
      format!("Hello, {}! org={} role={:?}", user.email, org_id, role).into_response()
    }
    None => (StatusCode::UNAUTHORIZED, "Not logged in").into_response(),
  }
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
      return Ok((StatusCode::UNAUTHORIZED, "Authentication required").into_response());
    }
  };
  if !user.is_admin {
    record_authz_denied(&db, Action::Manage, "admin", Some(user.id)).await;
    return Ok((StatusCode::FORBIDDEN, "Forbidden").into_response());
  }
  Ok((StatusCode::OK, format!("Admin only: access granted for {}", user.email)).into_response())
}
