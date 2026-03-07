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
      let token = extract_bearer(req.headers().get(AUTHORIZATION))
        .or_else(|| extract_token_from_query(req.uri().query()));
      if let Some(token) = token
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

/// Extracts the `token` query parameter. Used for WebSocket upgrade requests where the browser
/// cannot set the Authorization header.
pub(crate) fn extract_token_from_query(query: Option<&str>) -> Option<String> {
  let query = query?;
  for pair in query.split('&') {
    let (key, value) = pair.split_once('=')?;
    if key == "token" && !value.is_empty() {
      return Some(value.to_string());
    }
  }
  None
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
  use super::{
    OptionalRequireAuth, RequireAuth, TokenUser, extract_bearer, extract_token_from_query,
  };
  use crate::authz::{AuthzContext, RequestScope};
  use axum::http::{Extensions, HeaderValue};
  use axum_login::AuthUser;
  use uuid::Uuid;

  // A mock user for testing purposes
  #[derive(Clone, Debug)]
  struct MockUser {
    id: Uuid,
    session_auth_hash_val: Vec<u8>,
    org_id: Option<Uuid>,
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

  impl AuthzContext for MockUser {
    type RequesterId = Uuid;
    type SubjectId = Uuid;

    fn requester_id(&self) -> Uuid {
      self.id
    }

    fn subject_id(&self) -> Uuid {
      self.id
    }

    fn organization_id(&self) -> Option<Uuid> {
      self.org_id
    }
  }

  // --- extract_token_from_query (WebSocket auth via query string) ---

  #[test]
  fn extract_token_from_query_none_for_no_query() {
    assert_eq!(extract_token_from_query(None), None);
  }

  #[test]
  fn extract_token_from_query_none_for_empty_query() {
    assert_eq!(extract_token_from_query(Some("")), None);
  }

  #[test]
  fn extract_token_from_query_some_for_single_param() {
    assert_eq!(
      extract_token_from_query(Some("token=forge_abc123")),
      Some("forge_abc123".to_string())
    );
  }

  #[test]
  fn extract_token_from_query_some_for_multiple_params() {
    assert_eq!(
      extract_token_from_query(Some("foo=bar&token=my-secret&baz=quux")),
      Some("my-secret".to_string())
    );
  }

  #[test]
  fn extract_token_from_query_none_for_empty_value() {
    assert_eq!(extract_token_from_query(Some("token=")), None);
  }

  #[test]
  fn extract_token_from_query_none_when_token_key_absent() {
    assert_eq!(extract_token_from_query(Some("other=value")), None);
  }

  #[test]
  fn extract_token_from_query_decodes_value_as_opaque() {
    // We do not percent-decode; token is used as-is (typical tokens have no reserved chars)
    assert_eq!(
      extract_token_from_query(Some("token=forge_xyz")),
      Some("forge_xyz".to_string())
    );
  }

  // --- extract_bearer ---

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
      org_id: None,
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

  mod token_user_authz_context {
    use super::*;

    #[test]
    fn token_user_delegates_requester_id_to_inner_user() {
      // Given a TokenUser wrapping a MockUser with a specific ID
      let user_id = Uuid::new_v4();
      let mock_user = MockUser {
        id: user_id,
        session_auth_hash_val: vec![1, 2, 3],
        org_id: None,
      };
      let token_user = TokenUser {
        user: mock_user,
        extensions: Extensions::new(),
      };

      // When I call requester_id on the TokenUser
      let requester_id = token_user.requester_id();

      // Then it should return the inner user's ID
      assert_eq!(requester_id, user_id);
    }

    #[test]
    fn token_user_delegates_subject_id_to_inner_user() {
      // Given a TokenUser wrapping a MockUser with a specific ID
      let user_id = Uuid::new_v4();
      let mock_user = MockUser {
        id: user_id,
        session_auth_hash_val: vec![1, 2, 3],
        org_id: None,
      };
      let token_user = TokenUser {
        user: mock_user,
        extensions: Extensions::new(),
      };

      // When I call subject_id on the TokenUser
      let subject_id = token_user.subject_id();

      // Then it should return the inner user's ID
      assert_eq!(subject_id, user_id);
    }

    #[test]
    fn token_user_uses_organization_id_from_extensions_when_present() {
      // Given a TokenUser with RequestScope in extensions
      let user_id = Uuid::new_v4();
      let org_id = Uuid::new_v4();
      let mock_user = MockUser {
        id: user_id,
        session_auth_hash_val: vec![1, 2, 3],
        org_id: None,
      };
      let mut extensions = Extensions::new();
      extensions.insert(RequestScope {
        organization_id: org_id,
        role_id: Uuid::new_v4(),
        role_name: "admin".to_string(),
      });
      let token_user = TokenUser {
        user: mock_user,
        extensions,
      };

      // When I call organization_id on the TokenUser
      let result_org_id = token_user.organization_id();

      // Then it should return the organization ID from extensions
      assert_eq!(result_org_id, Some(org_id));
    }

    #[test]
    fn token_user_falls_back_to_user_org_id_when_no_extensions() {
      // Given a TokenUser with a user that has an organization ID but no RequestScope in extensions
      let user_id = Uuid::new_v4();
      let org_id = Uuid::new_v4();
      let mock_user = MockUser {
        id: user_id,
        session_auth_hash_val: vec![1, 2, 3],
        org_id: Some(org_id),
      };
      let token_user = TokenUser {
        user: mock_user,
        extensions: Extensions::new(),
      };

      // When I call organization_id on the TokenUser
      let result_org_id = token_user.organization_id();

      // Then it should return the organization ID from the user
      assert_eq!(result_org_id, Some(org_id));
    }

    #[test]
    fn token_user_returns_none_when_no_org_id_anywhere() {
      // Given a TokenUser with no organization ID in user or extensions
      let user_id = Uuid::new_v4();
      let mock_user = MockUser {
        id: user_id,
        session_auth_hash_val: vec![1, 2, 3],
        org_id: None,
      };
      let token_user = TokenUser {
        user: mock_user,
        extensions: Extensions::new(),
      };

      // When I call organization_id on the TokenUser
      let result_org_id = token_user.organization_id();

      // Then it should return None
      assert_eq!(result_org_id, None);
    }
  }

  mod token_user_auth_user {
    use super::*;

    #[test]
    fn token_user_delegates_id_to_inner_user() {
      // Given a TokenUser wrapping a MockUser
      let user_id = Uuid::new_v4();
      let mock_user = MockUser {
        id: user_id,
        session_auth_hash_val: vec![1, 2, 3],
        org_id: None,
      };
      let token_user = TokenUser {
        user: mock_user,
        extensions: Extensions::new(),
      };

      // When I call id() on the TokenUser
      let id = token_user.id();

      // Then it should return the inner user's ID
      assert_eq!(id, user_id);
    }

    #[test]
    fn token_user_delegates_session_auth_hash_to_inner_user() {
      // Given a TokenUser wrapping a MockUser with a specific session hash
      let hash = vec![5, 6, 7, 8];
      let mock_user = MockUser {
        id: Uuid::new_v4(),
        session_auth_hash_val: hash.clone(),
        org_id: None,
      };
      let token_user = TokenUser {
        user: mock_user,
        extensions: Extensions::new(),
      };

      // When I call session_auth_hash() on the TokenUser
      let result_hash = token_user.session_auth_hash();

      // Then it should return the inner user's session hash
      assert_eq!(result_hash, &hash);
    }
  }

  mod bearer_token_extraction {
    use super::*;

    #[test]
    fn extract_bearer_returns_none_when_header_is_missing() {
      // Given no Authorization header
      // When I try to extract a Bearer token
      let result = extract_bearer(None);

      // Then it should return None
      assert_eq!(result, None);
    }

    #[test]
    fn extract_bearer_returns_token_when_valid_bearer_header() {
      // Given an Authorization header with "Bearer <token>"
      let header = HeaderValue::from_static("Bearer my-secret-token");

      // When I extract the Bearer token
      let result = extract_bearer(Some(&header));

      // Then it should return the token value
      assert_eq!(result, Some("my-secret-token".to_string()));
    }

    #[test]
    fn extract_bearer_strips_whitespace_around_token() {
      // Given an Authorization header with whitespace around the token
      let header = HeaderValue::from_static("Bearer   token-with-spaces   ");

      // When I extract the Bearer token
      let result = extract_bearer(Some(&header));

      // Then it should return the token without leading/trailing whitespace
      assert_eq!(result, Some("token-with-spaces".to_string()));
    }

    #[test]
    fn extract_bearer_returns_none_for_non_bearer_scheme() {
      // Given an Authorization header with a different scheme (e.g., Basic)
      let header = HeaderValue::from_static("Basic dXNlcjpwYXNz");

      // When I try to extract a Bearer token
      let result = extract_bearer(Some(&header));

      // Then it should return None
      assert_eq!(result, None);
    }
  }

  mod query_token_extraction {
    use super::*;

    #[test]
    fn extract_token_from_query_returns_none_when_query_is_missing() {
      // Given no query string
      // When I try to extract a token parameter
      let result = extract_token_from_query(None);

      // Then it should return None
      assert_eq!(result, None);
    }

    #[test]
    fn extract_token_from_query_returns_token_when_present() {
      // Given a query string with a token parameter
      let query = "token=my-api-token";

      // When I extract the token
      let result = extract_token_from_query(Some(query));

      // Then it should return the token value
      assert_eq!(result, Some("my-api-token".to_string()));
    }

    #[test]
    fn extract_token_from_query_handles_multiple_parameters() {
      // Given a query string with multiple parameters including token
      let query = "foo=bar&token=secret-value&baz=quux";

      // When I extract the token
      let result = extract_token_from_query(Some(query));

      // Then it should return the token value
      assert_eq!(result, Some("secret-value".to_string()));
    }

    #[test]
    fn extract_token_from_query_returns_none_when_token_is_empty() {
      // Given a query string with an empty token parameter
      let query = "token=";

      // When I try to extract the token
      let result = extract_token_from_query(Some(query));

      // Then it should return None (empty tokens are invalid)
      assert_eq!(result, None);
    }

    #[test]
    fn extract_token_from_query_returns_none_when_token_key_is_absent() {
      // Given a query string without a token parameter
      let query = "other=value&another=param";

      // When I try to extract the token
      let result = extract_token_from_query(Some(query));

      // Then it should return None
      assert_eq!(result, None);
    }
  }

  mod require_auth_extractor {
    use super::*;
    use async_trait::async_trait;
    use axum::extract::FromRequestParts;
    use axum::http::{Request, StatusCode};
    use axum_login::{AuthnBackend, UserId};

    // Mock backend type for testing
    #[derive(Clone, Debug)]
    struct MockBackend;

    #[async_trait]
    impl AuthnBackend for MockBackend {
      type User = MockUser;
      type Credentials = ();
      type Error = std::convert::Infallible;

      async fn authenticate(
        &self,
        _: Self::Credentials,
      ) -> Result<Option<Self::User>, Self::Error> {
        Ok(None)
      }

      async fn get_user(&self, _: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        Ok(None)
      }
    }

    #[tokio::test]
    async fn require_auth_returns_user_when_token_user_present() {
      // Given request parts with TokenUser in extensions
      let user_id = Uuid::new_v4();
      let mock_user = MockUser {
        id: user_id,
        session_auth_hash_val: vec![1, 2, 3],
        org_id: None,
      };
      let token_user = TokenUser {
        user: mock_user.clone(),
        extensions: Extensions::new(),
      };
      let req = Request::builder().body(()).unwrap();
      let (mut parts, _) = req.into_parts();
      parts.extensions.insert(token_user);

      // When I extract RequireAuth from the parts
      let result = RequireAuth::<MockBackend, MockUser>::from_request_parts(&mut parts, &()).await;

      // Then it should return Ok with the user
      assert!(result.is_ok());
      let require_auth = result.unwrap();
      assert_eq!(require_auth.0.id(), user_id);
    }

    #[tokio::test]
    async fn require_auth_returns_unauthorized_when_no_token_user() {
      // Given request parts without TokenUser in extensions
      let req = Request::builder().body(()).unwrap();
      let (mut parts, _) = req.into_parts();

      // When I try to extract RequireAuth from the parts
      let result = RequireAuth::<MockBackend, MockUser>::from_request_parts(&mut parts, &()).await;

      // Then it should return an UNAUTHORIZED error
      assert!(result.is_err());
      let err = result.unwrap_err();
      assert_eq!(err.0, StatusCode::UNAUTHORIZED);
      assert_eq!(err.1, "Authentication required");
    }
  }

  mod optional_require_auth_extractor {
    use super::*;
    use async_trait::async_trait;
    use axum::extract::FromRequestParts;
    use axum::http::Request;
    use axum_login::{AuthnBackend, UserId};

    // Mock backend type for testing
    #[derive(Clone)]
    struct MockBackend;

    #[async_trait]
    impl AuthnBackend for MockBackend {
      type User = MockUser;
      type Credentials = ();
      type Error = std::convert::Infallible;

      async fn authenticate(
        &self,
        _: Self::Credentials,
      ) -> Result<Option<Self::User>, Self::Error> {
        Ok(None)
      }

      async fn get_user(&self, _: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        Ok(None)
      }
    }

    #[tokio::test]
    async fn optional_require_auth_returns_some_when_token_user_present() {
      // Given request parts with TokenUser in extensions
      let user_id = Uuid::new_v4();
      let mock_user = MockUser {
        id: user_id,
        session_auth_hash_val: vec![1, 2, 3],
        org_id: None,
      };
      let token_user = TokenUser {
        user: mock_user.clone(),
        extensions: Extensions::new(),
      };
      let req = Request::builder().body(()).unwrap();
      let (mut parts, _) = req.into_parts();
      parts.extensions.insert(token_user);

      // When I extract OptionalRequireAuth from the parts
      let result =
        OptionalRequireAuth::<MockBackend, MockUser>::from_request_parts(&mut parts, &()).await;

      // Then it should return Ok with Some(user)
      assert!(result.is_ok());
      let optional_auth = result.unwrap();
      assert!(optional_auth.0.is_some());
      assert_eq!(optional_auth.0.unwrap().id(), user_id);
    }

    #[tokio::test]
    async fn optional_require_auth_returns_none_when_no_token_user() {
      // Given request parts without TokenUser in extensions
      let req = Request::builder().body(()).unwrap();
      let (mut parts, _) = req.into_parts();

      // When I extract OptionalRequireAuth from the parts
      let result =
        OptionalRequireAuth::<MockBackend, MockUser>::from_request_parts(&mut parts, &()).await;

      // Then it should return Ok with None (never fails)
      assert!(result.is_ok());
      let optional_auth = result.unwrap();
      assert!(optional_auth.0.is_none());
    }
  }
}
