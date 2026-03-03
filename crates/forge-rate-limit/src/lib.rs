//! Rate limiting key extractors and helpers.
//!
//! Per-IP limiting uses tower_governor's `PeerIpKeyExtractor` (in the app layer).
//! Per-requester (per user per organization) uses `RequesterOrgKeyExtractor`,
//! which reads `AuthSession` from request extensions (set by axum-login's layer).

use axum_login::{AuthSession, AuthnBackend};
use forge_authz::AuthzContext;
use tower_governor::{errors::GovernorError, key_extractor::KeyExtractor};

/// Key for per-requester rate limiting: `(organization_id, user_id)`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RequesterOrgKey {
  pub organization_id: Option<uuid::Uuid>,
  pub user_id: uuid::Uuid,
}

/// Builds a [`RequesterOrgKey`] from an optional user implementing [`AuthzContext`].
/// Used by [`RequesterOrgKeyExtractor`] and testable in isolation.
#[inline]
pub fn requester_org_key_from_user<U: AuthzContext>(user: Option<&U>) -> RequesterOrgKey {
  user
    .map(|u| RequesterOrgKey {
      organization_id: u.organization_id(),
      user_id: u.requester_id(),
    })
    .unwrap_or(RequesterOrgKey {
      organization_id: None,
      user_id: uuid::Uuid::nil(),
    })
}

/// Extracts `(organization_id, user_id)` from `AuthSession` in request extensions.
#[derive(Clone, Debug)]
pub struct RequesterOrgKeyExtractor<B> {
  _backend: std::marker::PhantomData<B>,
}

impl<B> RequesterOrgKeyExtractor<B> {
  pub fn new() -> Self {
    Self {
      _backend: std::marker::PhantomData,
    }
  }
}

impl<B> Default for RequesterOrgKeyExtractor<B> {
  fn default() -> Self {
    Self::new()
  }
}

impl<B> KeyExtractor for RequesterOrgKeyExtractor<B>
where
  B: AuthnBackend + Send + Sync + 'static,
  B::User: AuthzContext + Send,
{
  type Key = RequesterOrgKey;

  fn extract<T>(&self, req: &axum::http::Request<T>) -> Result<Self::Key, GovernorError> {
    let user = req
      .extensions()
      .get::<AuthSession<B>>()
      .and_then(|auth| auth.user.as_ref());
    Ok(requester_org_key_from_user(user))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use axum::http::Request;
  use axum_login::AuthUser;

  /// Mock user with configurable org for testing requester_org_key_from_user.
  #[derive(Clone, Debug)]
  struct MockUserWithOrg {
    organization_id: Option<uuid::Uuid>,
    user_id: uuid::Uuid,
  }
  impl AuthzContext for MockUserWithOrg {
    fn requester_id(&self) -> uuid::Uuid {
      self.user_id
    }
    fn subject_id(&self) -> uuid::Uuid {
      self.user_id
    }
    fn organization_id(&self) -> Option<uuid::Uuid> {
      self.organization_id
    }
  }

  #[test]
  fn requester_org_key_equality_and_hash() {
    let id = uuid::Uuid::new_v4();
    let a = RequesterOrgKey {
      organization_id: None,
      user_id: id,
    };
    let b = RequesterOrgKey {
      organization_id: None,
      user_id: id,
    };
    assert_eq!(a, b);
    let mut set = std::collections::HashSet::new();
    set.insert(a);
    set.insert(b);
    assert_eq!(set.len(), 1);
    let org_id = uuid::Uuid::new_v4();
    let c = RequesterOrgKey {
      organization_id: Some(org_id),
      user_id: id,
    };
    let d = RequesterOrgKey {
      organization_id: Some(org_id),
      user_id: id,
    };
    assert_eq!(c, d);
    assert_ne!(a, c);
    set.insert(c);
    set.insert(d);
    assert_eq!(set.len(), 2);
  }

  #[test]
  fn requester_org_key_from_user_none() {
    let key = requester_org_key_from_user::<MockUserWithOrg>(None);
    assert!(key.organization_id.is_none());
    assert_eq!(key.user_id, uuid::Uuid::nil());
  }

  #[test]
  fn requester_org_key_from_user_some_without_org() {
    let user = MockUserWithOrg {
      organization_id: None,
      user_id: uuid::Uuid::new_v4(),
    };
    let key = requester_org_key_from_user(Some(&user));
    assert!(key.organization_id.is_none());
    assert_eq!(key.user_id, user.user_id);
  }

  #[test]
  fn requester_org_key_from_user_some_with_org() {
    let org_id = uuid::Uuid::new_v4();
    let user_id = uuid::Uuid::new_v4();
    let user = MockUserWithOrg {
      organization_id: Some(org_id),
      user_id,
    };
    let key = requester_org_key_from_user(Some(&user));
    assert_eq!(key.organization_id, Some(org_id));
    assert_eq!(key.user_id, user_id);
  }

  #[test]
  fn requester_org_key_extractor_new_and_default() {
    use async_trait::async_trait;
    use axum_login::AuthnBackend;
    use axum_login::UserId;
    #[derive(Clone)]
    struct DummyBackend;
    #[async_trait]
    impl AuthnBackend for DummyBackend {
      type User = MockUser;
      type Credentials = ();
      type Error = std::io::Error;
      async fn authenticate(
        &self,
        _creds: Self::Credentials,
      ) -> Result<Option<Self::User>, Self::Error> {
        Ok(None)
      }
      async fn get_user(&self, _user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        Ok(None)
      }
    }
    static _MOCK_AUTH_HASH: [u8; 0] = [];
    #[derive(Clone, Debug)]
    struct MockUser;
    impl AuthUser for MockUser {
      type Id = uuid::Uuid;
      fn id(&self) -> Self::Id {
        uuid::Uuid::nil()
      }
      fn session_auth_hash(&self) -> &[u8] {
        &_MOCK_AUTH_HASH
      }
    }
    impl AuthzContext for MockUser {
      fn requester_id(&self) -> uuid::Uuid {
        uuid::Uuid::nil()
      }
      fn subject_id(&self) -> uuid::Uuid {
        uuid::Uuid::nil()
      }
      fn organization_id(&self) -> Option<uuid::Uuid> {
        None
      }
    }
    let ext = RequesterOrgKeyExtractor::<DummyBackend>::new();
    let _ext_default = RequesterOrgKeyExtractor::<DummyBackend>::default();
    let req = Request::builder().body(()).unwrap();
    let key = ext.extract(&req).unwrap();
    assert!(key.organization_id.is_none());
    assert_eq!(key.user_id, uuid::Uuid::nil());
  }

  /// Integration test: request through auth layer so AuthSession is present with a user,
  /// then call the extractor to cover the "user present" branch in extract().
  #[tokio::test]
  async fn requester_org_key_extractor_with_auth_session_user() {
    use async_trait::async_trait;
    use axum::Router;
    use axum::body::Body;
    use axum::extract::Request;
    use axum::http::{Request as HttpRequest, StatusCode};
    use axum::routing::{get, post};
    use axum_login::{AuthManagerLayerBuilder, AuthnBackend, UserId};
    use tower::ServiceExt;
    use tower_sessions::{MemoryStore, SessionManagerLayer};

    static AUTH_HASH: [u8; 0] = [];
    #[derive(Clone, Debug)]
    struct TestUser {
      organization_id: Option<uuid::Uuid>,
      user_id: uuid::Uuid,
    }
    impl AuthUser for TestUser {
      type Id = uuid::Uuid;
      fn id(&self) -> Self::Id {
        self.user_id
      }
      fn session_auth_hash(&self) -> &[u8] {
        &AUTH_HASH
      }
    }
    impl AuthzContext for TestUser {
      fn requester_id(&self) -> uuid::Uuid {
        self.user_id
      }
      fn subject_id(&self) -> uuid::Uuid {
        self.user_id
      }
      fn organization_id(&self) -> Option<uuid::Uuid> {
        self.organization_id
      }
    }

    #[derive(Clone)]
    struct TestBackend {
      user: TestUser,
    }
    #[async_trait]
    impl AuthnBackend for TestBackend {
      type User = TestUser;
      type Credentials = ();
      type Error = std::convert::Infallible;
      async fn authenticate(
        &self,
        _creds: Self::Credentials,
      ) -> Result<Option<Self::User>, Self::Error> {
        Ok(Some(self.user.clone()))
      }
      async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        if *user_id == self.user.id() {
          Ok(Some(self.user.clone()))
        } else {
          Ok(None)
        }
      }
    }

    async fn key_handler(req: Request) -> (StatusCode, String) {
      let ext = RequesterOrgKeyExtractor::<TestBackend>::new();
      let key = ext.extract(&req).unwrap();
      (
        StatusCode::OK,
        format!(
          "{}:{}",
          key
            .organization_id
            .map(|u| u.to_string())
            .unwrap_or_default(),
          key.user_id
        ),
      )
    }

    async fn login_handler(mut auth: axum_login::AuthSession<TestBackend>) -> &'static str {
      let user = auth.authenticate(()).await.unwrap().unwrap();
      auth.login(&user).await.unwrap();
      "ok"
    }

    let org_id = uuid::Uuid::new_v4();
    let user_id = uuid::Uuid::new_v4();
    let backend = TestBackend {
      user: TestUser {
        organization_id: Some(org_id),
        user_id,
      },
    };
    let store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(store);
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    let app = Router::new()
      .route("/login", post(login_handler))
      .route("/key", get(key_handler))
      .layer(auth_layer);

    // Request /key without logging in: AuthSession present but user None (covers that branch)
    let key_req_anon = HttpRequest::builder()
      .uri("/key")
      .body(Body::empty())
      .unwrap();
    let key_res_anon = app.clone().oneshot(key_req_anon).await.unwrap();
    assert!(key_res_anon.status().is_success());
    let body_anon = axum::body::to_bytes(key_res_anon.into_body(), usize::MAX)
      .await
      .unwrap();
    assert_eq!(
      std::str::from_utf8(&body_anon).unwrap(),
      ":00000000-0000-0000-0000-000000000000"
    );

    let login_req = HttpRequest::builder()
      .method("POST")
      .uri("/login")
      .body(Body::empty())
      .unwrap();
    let login_res = app.clone().oneshot(login_req).await.unwrap();
    assert!(login_res.status().is_success());

    let cookie = login_res
      .headers()
      .get("set-cookie")
      .cloned()
      .expect("session cookie");
    let key_req = HttpRequest::builder()
      .uri("/key")
      .header("cookie", cookie)
      .body(Body::empty())
      .unwrap();
    let key_res = app.oneshot(key_req).await.unwrap();
    assert!(key_res.status().is_success());
    let body = axum::body::to_bytes(key_res.into_body(), usize::MAX)
      .await
      .unwrap();
    let s = std::str::from_utf8(&body).unwrap();
    assert_eq!(s, format!("{}:{}", org_id, user_id));
  }
}
