//! Scope-from-headers resolution and extractors.
//!
//! The app provides a [ScopeResolver] implementation (e.g. using DB to validate
//! org/role and user membership). [ScopeFromHeaders] and [OptionalScopeFromHeaders]
//! use it so forge-auth stays free of template DB types.

use async_trait::async_trait;
use axum::Json;
use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use forge_db::DbConnection;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
struct ErrorBody {
  error: String,
  message: String,
}

use crate::authz::RequestScope;

/// Header names for scope. Use these when implementing [ScopeResolver] or when
/// sending requests (e.g. `X-Organization-Id`, `X-Role-Id`).
pub const HEADER_ORGANIZATION_ID: &str = "x-organization-id";
pub const HEADER_ROLE_ID: &str = "x-role-id";

/// Errors produced when resolving scope from headers.
#[derive(Debug)]
pub enum ScopeResolveError {
  Unauthorized(String),
  BadRequest(String),
  NotFound(String),
  Forbidden(String),
  Other(String),
}

impl std::fmt::Display for ScopeResolveError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ScopeResolveError::Unauthorized(m) => write!(f, "Unauthorized: {}", m),
      ScopeResolveError::BadRequest(m) => write!(f, "Bad Request: {}", m),
      ScopeResolveError::NotFound(m) => write!(f, "Not Found: {}", m),
      ScopeResolveError::Forbidden(m) => write!(f, "Forbidden: {}", m),
      ScopeResolveError::Other(m) => write!(f, "{}", m),
    }
  }
}

impl std::error::Error for ScopeResolveError {}

impl IntoResponse for ScopeResolveError {
  fn into_response(self) -> Response {
    let (status, message) = match &self {
      ScopeResolveError::Unauthorized(m) => (StatusCode::UNAUTHORIZED, m.clone()),
      ScopeResolveError::BadRequest(m) => (StatusCode::BAD_REQUEST, m.clone()),
      ScopeResolveError::NotFound(m) => (StatusCode::NOT_FOUND, m.clone()),
      ScopeResolveError::Forbidden(m) => (StatusCode::FORBIDDEN, m.clone()),
      ScopeResolveError::Other(m) => (StatusCode::INTERNAL_SERVER_ERROR, m.clone()),
    };
    (
      status,
      Json(ErrorBody {
        error: status.as_str().to_string(),
        message,
      }),
    )
      .into_response()
  }
}

/// Resolves [RequestScope] from request headers and the authenticated user.
/// Implement this in the app (e.g. validate org/role and user membership via DB).
#[async_trait]
pub trait ScopeResolver: Send + Sync {
  /// Resolve scope when headers may be missing. Returns `Ok(None)` when the user
  /// has zero or multiple org/role assignments and headers are missing (so list
  /// can use global read). Returns `Ok(Some(scope))` when scope is determined.
  async fn resolve_optional(
    &self,
    headers: &axum::http::HeaderMap,
    user_id: Uuid,
    db: &DbConnection,
  ) -> Result<Option<RequestScope>, ScopeResolveError>;

  /// Resolve scope when scope headers are required. Returns error when headers
  /// are missing or invalid or user is not a member of the org/role.
  async fn resolve_required(
    &self,
    headers: &axum::http::HeaderMap,
    user_id: Uuid,
    db: &DbConnection,
  ) -> Result<RequestScope, ScopeResolveError> {
    let opt = self.resolve_optional(headers, user_id, db).await?;
    opt.ok_or_else(|| {
      ScopeResolveError::BadRequest(
        "Missing or invalid X-Organization-Id / X-Role-Id header".into(),
      )
    })
  }
}

/// State required by scope extractors: DB connection and the app’s scope resolver.
#[derive(Clone)]
pub struct ScopeExtractorState {
  /// Database connection for the resolver (e.g. to validate org/role/membership).
  pub db: DbConnection,
  /// App-provided resolver (e.g. implementation that queries user_org_role, org_role, organization).
  pub scope_resolver: std::sync::Arc<dyn ScopeResolver>,
}

/// Extractor: requires Bearer auth and valid scope (from [ScopeResolver]).
/// Use for get/create/update/delete so the handler always has a scope.
#[derive(Clone, Debug)]
pub struct ScopeFromHeaders<U>(pub RequestScope, pub std::marker::PhantomData<fn() -> U>);

/// Extractor: requires X-Organization-Id and X-Role-Id to be present (no inference from single org).
/// Returns 400 when either header is missing. Use for endpoints that must have explicit scope.
#[derive(Clone, Debug)]
pub struct ScopeHeadersRequired<U>(pub RequestScope, pub std::marker::PhantomData<fn() -> U>);

/// Extractor: optional scope. When resolver returns `None` (e.g. missing headers
/// and user has 0 or multiple org/roles), yields `None`. Use for list so
/// global-read users can list without scope headers.
#[derive(Clone, Debug)]
pub struct OptionalScopeFromHeaders<U>(
  pub Option<RequestScope>,
  pub std::marker::PhantomData<fn() -> U>,
);

impl<U> FromRequestParts<ScopeExtractorState> for OptionalScopeFromHeaders<U>
where
  U: axum_login::AuthUser + Send + Sync + 'static,
  U::Id: Into<Uuid>,
{
  type Rejection = ScopeResolveError;

  fn from_request_parts(
    parts: &mut Parts,
    state: &ScopeExtractorState,
  ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
    let user_id = parts
      .extensions
      .get::<crate::token_auth::TokenUser<U>>()
      .map(|tu| tu.user.id().into());
    let headers = parts.headers.clone();
    let db = state.db.clone();
    let resolver = state.scope_resolver.clone();
    async move {
      let user_id: Uuid =
        user_id.ok_or_else(|| ScopeResolveError::Unauthorized("Authentication required".into()))?;
      let scope_opt = resolver.resolve_optional(&headers, user_id, &db).await?;
      Ok(OptionalScopeFromHeaders(
        scope_opt,
        std::marker::PhantomData,
      ))
    }
  }
}

impl<U> FromRequestParts<ScopeExtractorState> for ScopeFromHeaders<U>
where
  U: axum_login::AuthUser + Send + Sync + 'static,
  U::Id: Into<Uuid>,
{
  type Rejection = ScopeResolveError;

  fn from_request_parts(
    parts: &mut Parts,
    state: &ScopeExtractorState,
  ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
    let user_id = parts
      .extensions
      .get::<crate::token_auth::TokenUser<U>>()
      .map(|tu| tu.user.id().into());
    let headers = parts.headers.clone();
    let db = state.db.clone();
    let resolver = state.scope_resolver.clone();
    async move {
      let user_id: Uuid =
        user_id.ok_or_else(|| ScopeResolveError::Unauthorized("Authentication required".into()))?;
      let scope = resolver.resolve_required(&headers, user_id, &db).await?;
      Ok(ScopeFromHeaders(scope, std::marker::PhantomData))
    }
  }
}

impl<U> FromRequestParts<ScopeExtractorState> for ScopeHeadersRequired<U>
where
  U: axum_login::AuthUser + Send + Sync + 'static,
  U::Id: Into<Uuid>,
{
  type Rejection = ScopeResolveError;

  fn from_request_parts(
    parts: &mut Parts,
    state: &ScopeExtractorState,
  ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
    let user_id = parts
      .extensions
      .get::<crate::token_auth::TokenUser<U>>()
      .map(|tu| tu.user.id().into());
    let headers = parts.headers.clone();
    let db = state.db.clone();
    let resolver = state.scope_resolver.clone();
    async move {
      let user_id: Uuid =
        user_id.ok_or_else(|| ScopeResolveError::Unauthorized("Authentication required".into()))?;
      if headers
        .get(HEADER_ORGANIZATION_ID)
        .and_then(|v| v.to_str().ok())
        .is_none()
        || headers
          .get(HEADER_ROLE_ID)
          .and_then(|v| v.to_str().ok())
          .is_none()
      {
        return Err(ScopeResolveError::BadRequest(
          "Missing or invalid X-Organization-Id / X-Role-Id header".into(),
        ));
      }
      let scope = resolver.resolve_required(&headers, user_id, &db).await?;
      Ok(ScopeHeadersRequired(scope, std::marker::PhantomData))
    }
  }
}
