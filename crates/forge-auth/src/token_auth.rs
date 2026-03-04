//! API token (Bearer) authentication: layer and extractor.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use axum::body::Body;
use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;

use axum_login::{AuthUser, AuthnBackend};

use crate::authz::{AuthzContext, RequestScope};

/// Re-export password/API token helpers so existing `forge_auth::token_auth::*` imports keep working.
pub use crate::password::{hash_api_token, hash_password, verify_api_token, verify_password};
use forge_db::DbConnection;
use futures::future::BoxFuture;
use tower::{Layer, Service};
use uuid::Uuid;

/// Type alias for the token lookup closure: (db, raw_token) -> Option<user_id>.
pub type TokenLookupFn =
  Arc<dyn Fn(DbConnection, String) -> BoxFuture<'static, Option<Uuid>> + Send + Sync>;

/// Wrapper to store token-authenticated user in request extensions.
#[derive(Clone, Debug)]
pub struct TokenUser<U> {
  pub user: U,
  pub extensions: axum::http::Extensions,
}

impl<U> AuthzContext for TokenUser<U>
where
  U: AuthzContext<RequesterId = Uuid, SubjectId = Uuid>,
{
  type RequesterId = Uuid;
  type SubjectId = Uuid;

  fn requester_id(&self) -> Uuid {
    self.user.requester_id()
  }

  fn subject_id(&self) -> Uuid {
    self.user.subject_id()
  }

  fn organization_id(&self) -> Option<Uuid> {
    self
      .extensions
      .get::<RequestScope>()
      .map(|scope| scope.organization_id)
      .or_else(|| self.user.organization_id())
  }
}

impl<U> AuthUser for TokenUser<U>
where
  U: AuthUser<Id = Uuid> + Send + Sync + 'static,
{
  type Id = Uuid;
  fn id(&self) -> Self::Id {
    self.user.id()
  }

  fn session_auth_hash(&self) -> &[u8] {
    self.user.session_auth_hash()
  }
}

/// Tower layer: if `Authorization: Bearer <token>` is present, looks up user and inserts TokenUser.
#[derive(Clone)]
pub struct TokenAuthLayer<B, U> {
  _user: std::marker::PhantomData<U>,
  db: DbConnection,
  backend: Arc<B>,
  lookup: TokenLookupFn,
}

impl<B, U> TokenAuthLayer<B, U> {
  pub fn new(db: DbConnection, backend: B, lookup: TokenLookupFn) -> Self {
    Self {
      db,
      backend: Arc::new(backend),
      lookup,
      _user: std::marker::PhantomData,
    }
  }
}

impl<B, U, S> Layer<S> for TokenAuthLayer<B, U>
where
  B: AuthnBackend<User = U> + Clone + Send + Sync + 'static,
  U: AuthUser<Id = Uuid> + Send + 'static,
{
  type Service = TokenAuthService<B, U, S>;

  fn layer(&self, inner: S) -> Self::Service {
    TokenAuthService {
      db: self.db.clone(),
      backend: Arc::clone(&self.backend),
      lookup: Arc::clone(&self.lookup),
      inner,
      _user: std::marker::PhantomData,
    }
  }
}

pub struct TokenAuthService<B, U, S> {
  _user: std::marker::PhantomData<U>,
  db: DbConnection,
  backend: Arc<B>,
  lookup: TokenLookupFn,
  inner: S,
}

impl<B, U, S> Clone for TokenAuthService<B, U, S>
where
  S: Clone,
{
  fn clone(&self) -> Self {
    Self {
      db: self.db.clone(),
      backend: Arc::clone(&self.backend),
      lookup: Arc::clone(&self.lookup),
      inner: self.inner.clone(),
      _user: std::marker::PhantomData,
    }
  }
}

impl<B, U, S> Service<axum::http::Request<Body>> for TokenAuthService<B, U, S>
where
  B: AuthnBackend<User = U> + Send + Sync + 'static,
  U: AuthUser<Id = Uuid> + Send + 'static,
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
    let req_extensions = req.extensions().clone(); // Clone extensions before mutable borrow
    let mut inner = self.inner.clone();

    Box::pin(async move {
      if let Some(token) = extract_bearer(req.headers().get(AUTHORIZATION))
        && let Some(user_id) = lookup(db.clone(), token).await
        && let Ok(Some(user)) = backend.get_user(&user_id).await
      {
        req.extensions_mut().insert(TokenUser {
          user,
          extensions: req_extensions,
        });
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

/// Extractor: current user from token (request extension set by [TokenAuthLayer]).
#[derive(Clone, Debug)]
pub struct RequireAuth<B, U>(pub U, std::marker::PhantomData<B>)
where
  B: AuthnBackend<User = U>,
  U: AuthUser;

impl<B, U, S> FromRequestParts<S> for RequireAuth<B, U>
where
  B: AuthnBackend<User = U> + Send + Sync + 'static,
  U: AuthzContext + AuthUser<Id = Uuid> + Send + Clone + 'static,
  S: Send + Sync,
{
  type Rejection = (StatusCode, &'static str);

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    if let Some(token_user) = parts.extensions.get::<TokenUser<U>>() {
      return Ok(RequireAuth(
        token_user.user.clone(),
        std::marker::PhantomData,
      ));
    }
    Err((StatusCode::UNAUTHORIZED, "Authentication required"))
  }
}

/// Extractor: current user or None.
#[derive(Clone, Debug)]
pub struct OptionalRequireAuth<B, U>(pub Option<U>, std::marker::PhantomData<B>)
where
  B: AuthnBackend<User = U>,
  U: AuthUser;

impl<B, U, S> FromRequestParts<S> for OptionalRequireAuth<B, U>
where
  B: AuthnBackend<User = U> + Send + Sync + 'static,
  U: AuthzContext + AuthUser<Id = Uuid> + Send + Clone + 'static,
  S: Send + Sync,
{
  type Rejection = (StatusCode, &'static str);

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    if let Some(token_user) = parts.extensions.get::<TokenUser<U>>() {
      return Ok(OptionalRequireAuth(
        Some(token_user.user.clone()),
        std::marker::PhantomData,
      ));
    }
    Ok(OptionalRequireAuth(None, std::marker::PhantomData))
  }
}

#[cfg(test)]
mod tests {
  use super::extract_bearer;
  use axum::http::Extensions;
  use axum::http::HeaderValue;
  use axum_login::AuthUser; // Added AuthUser import for the test
  use uuid::Uuid; // Added Extensions import for the test

  // A mock user for testing purposes
  #[derive(Clone, Debug)]
  struct MockUser {
    id: Uuid,
    session_auth_hash_val: Vec<u8>,
  }

  impl AuthUser for MockUser {
    type Id = Uuid;
    fn id(&self) -> Self::Id {
      self.id
    }
    fn session_auth_hash(&self) -> &[u8] {
      &self.session_auth_hash_val
    }
  }

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
    let user_id = Uuid::new_v4();
    let mock_user = MockUser {
      id: user_id,
      session_auth_hash_val: vec![1, 2, 3],
    };
    let u = super::TokenUser {
      user: mock_user.clone(),
      extensions: Extensions::new(),
    };
    let _ = format!("{:?}", u);
    let u2 = u.clone();
    assert_eq!(u2.user.id(), user_id);
    assert_eq!(u2.user.session_auth_hash(), mock_user.session_auth_hash());
  }
}
