//! API token (Bearer) authentication: layer and extractor for dual session/token auth.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use axum::body::Body;
use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use axum_login::AuthSession;
use axum_login::AuthUser;
use axum_login::AuthnBackend;
use futures::future::BoxFuture;
use tower::{Layer, Service};
use uuid::Uuid;

use crate::DbConnection;
use crate::authz::AuthzContext;

/// Type alias for the token lookup closure: (db, raw_token) -> Option<user_id>.
pub type TokenLookupFn =
  Arc<dyn Fn(DbConnection, String) -> BoxFuture<'static, Option<Uuid>> + Send + Sync>;

/// Wrapper to store token-authenticated user in request extensions (avoids type key collision).
#[derive(Clone, Debug)]
pub struct TokenUser<U>(pub U);

/// Tower layer that runs before the auth layer: if `Authorization: Bearer <token>` is present,
/// looks up the user via the provided closure and inserts `TokenUser(user)` into request extensions.
#[derive(Clone)]
pub struct TokenAuthLayer<B> {
  /// Database connection for the token lookup.
  db: DbConnection,
  /// Auth backend used to load the user by id after lookup.
  backend: Arc<B>,
  /// Async closure that resolves a raw token to a user id.
  lookup: TokenLookupFn,
}

impl<B> TokenAuthLayer<B> {
  pub fn new(db: DbConnection, backend: B, lookup: TokenLookupFn) -> Self {
    Self {
      db,
      backend: Arc::new(backend),
      lookup,
    }
  }
}

impl<B, S> Layer<S> for TokenAuthLayer<B>
where
  B: AuthnBackend + Clone + 'static,
  B::User: Send + 'static,
{
  type Service = TokenAuthService<B, S>;

  fn layer(&self, inner: S) -> Self::Service {
    TokenAuthService {
      db: self.db.clone(),
      backend: Arc::clone(&self.backend),
      lookup: Arc::clone(&self.lookup),
      inner,
    }
  }
}

/// Service that runs the token lookup and inserts TokenUser into extensions when Bearer is valid.
pub struct TokenAuthService<B, S> {
  /// Database connection for the token lookup.
  db: DbConnection,
  /// Auth backend used to load the user by id after lookup.
  backend: Arc<B>,
  /// Async closure that resolves a raw token to a user id.
  lookup: TokenLookupFn,
  /// Inner service (router or next layer).
  inner: S,
}

impl<B, S> Clone for TokenAuthService<B, S>
where
  S: Clone,
{
  fn clone(&self) -> Self {
    Self {
      db: self.db.clone(),
      backend: Arc::clone(&self.backend),
      lookup: Arc::clone(&self.lookup),
      inner: self.inner.clone(),
    }
  }
}

impl<B, S> Service<axum::http::Request<Body>> for TokenAuthService<B, S>
where
  B: AuthnBackend + Send + Sync + 'static,
  B::User: AuthUser<Id = Uuid> + Send + 'static,
  S: Service<axum::http::Request<Body>, Response = axum::response::Response>
    + Clone
    + Send
    + 'static,
  S::Future: Send + 'static,
{
  type Response = S::Response;
  type Error = S::Error;
  type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

  fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
    self.inner.poll_ready(cx)
  }

  fn call(&mut self, mut req: axum::http::Request<Body>) -> Self::Future {
    let db = self.db.clone();
    let backend = Arc::clone(&self.backend);
    let lookup = Arc::clone(&self.lookup);
    let mut inner = self.inner.clone();

    Box::pin(async move {
      if let Some(token) = extract_bearer(req.headers().get(AUTHORIZATION))
        && let Some(user_id) = lookup(db.clone(), token).await
        && let Ok(Some(user)) = backend.get_user(&user_id).await
      {
        req.extensions_mut().insert(TokenUser(user));
      }
      inner.call(req).await
    })
  }
}

/// Extracts the Bearer token value from an `Authorization` header, or None if missing/invalid.
fn extract_bearer(value: Option<&axum::http::HeaderValue>) -> Option<String> {
  let v = value?.to_str().ok()?;
  let prefix = "Bearer ";
  v.strip_prefix(prefix).map(|s| s.trim().to_string())
}

/// Extractor that returns the current user from token (extension) or session.
/// Use for protected routes that accept either `Authorization: Bearer <token>` or session cookie.
#[derive(Clone, Debug)]
pub struct RequireAuth<B: AuthnBackend>(pub B::User);

impl<B, S> FromRequestParts<S> for RequireAuth<B>
where
  B: AuthnBackend + Send + Sync + 'static,
  B::User: AuthzContext + AuthUser<Id = Uuid> + Send + Clone + 'static,
  S: Send + Sync,
{
  type Rejection = (StatusCode, &'static str);

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    if let Some(user) = OptionalRequireAuth::<B>::from_request_parts(parts, _state)
      .await
      .ok()
      .and_then(|o| o.0)
    {
      return Ok(RequireAuth(user));
    }
    Err((StatusCode::UNAUTHORIZED, "Authentication required"))
  }
}

/// Extractor that returns the current user or None. Use when the handler must run for unauthenticated
/// requests (e.g. to record authz denied in audit before returning 401).
#[derive(Clone, Debug)]
pub struct OptionalRequireAuth<B: AuthnBackend>(pub Option<B::User>);

impl<B, S> FromRequestParts<S> for OptionalRequireAuth<B>
where
  B: AuthnBackend + Send + Sync + 'static,
  B::User: AuthzContext + AuthUser<Id = Uuid> + Send + Clone + 'static,
  S: Send + Sync,
{
  type Rejection = (StatusCode, &'static str);

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    if let Some(token_user) = parts.extensions.get::<TokenUser<B::User>>() {
      return Ok(OptionalRequireAuth(Some(token_user.0.clone())));
    }
    if let Some(session) = parts.extensions.get::<AuthSession<B>>()
      && let Some(ref user) = session.user
    {
      return Ok(OptionalRequireAuth(Some(user.clone())));
    }
    Ok(OptionalRequireAuth(None))
  }
}
