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
use forge_authz::AuthzContext;
use forge_db::DbConnection;
use futures::future::BoxFuture;
use tower::{Layer, Service};
use uuid::Uuid;

/// Type alias for the token lookup closure: (db, raw_token) -> Option<user_id>.
pub type TokenLookupFn =
  Arc<dyn Fn(DbConnection, String) -> BoxFuture<'static, Option<Uuid>> + Send + Sync>;

/// Wrapper to store token-authenticated user in request extensions.
#[derive(Clone, Debug)]
pub struct TokenUser<U>(pub U);

/// Tower layer: if `Authorization: Bearer <token>` is present, looks up user and inserts TokenUser.
#[derive(Clone)]
pub struct TokenAuthLayer<B> {
  db: DbConnection,
  backend: Arc<B>,
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

pub struct TokenAuthService<B, S> {
  db: DbConnection,
  backend: Arc<B>,
  lookup: TokenLookupFn,
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

pub(crate) fn extract_bearer(value: Option<&axum::http::HeaderValue>) -> Option<String> {
  let v = value?.to_str().ok()?;
  let prefix = "Bearer ";
  v.strip_prefix(prefix).map(|s| s.trim().to_string())
}

/// Extractor: current user from token (extension) or session.
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

/// Extractor: current user or None.
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

#[cfg(test)]
mod tests {
  use super::extract_bearer;
  use axum::http::HeaderValue;

  #[test]
  fn extract_bearer_none_for_no_header() {
    assert_eq!(extract_bearer(None), None);
  }

  #[test]
  fn extract_bearer_some_for_valid_bearer() {
    let hv = HeaderValue::from_static("Bearer my-token");
    assert_eq!(extract_bearer(Some(&hv)), Some("my-token".to_string()));
  }

  #[test]
  fn extract_bearer_strips_whitespace() {
    let hv = HeaderValue::from_static("Bearer  token-with-spaces  ");
    assert_eq!(
      extract_bearer(Some(&hv)),
      Some("token-with-spaces".to_string())
    );
  }

  #[test]
  fn extract_bearer_none_for_non_bearer() {
    let hv = HeaderValue::from_static("Basic dXNlcjpwYXNz");
    assert_eq!(extract_bearer(Some(&hv)), None);
  }

  #[test]
  fn extract_bearer_none_for_empty_after_bearer() {
    let hv = HeaderValue::from_static("Bearer ");
    assert_eq!(extract_bearer(Some(&hv)), Some("".to_string()));
  }

  #[test]
  fn token_user_debug_and_clone() {
    let u = super::TokenUser(42u64);
    let _ = format!("{:?}", u);
    let u2 = u.clone();
    assert_eq!(u2.0, 42u64);
  }
}
