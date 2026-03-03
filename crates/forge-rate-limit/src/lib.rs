//! Rate limiting key extractors and helpers.
//!
//! Per-IP limiting uses tower_governor's `PeerIpKeyExtractor` (in the app layer).
//! Per-requester (per user per organization) uses `RequesterOrgKeyExtractor`,
//! which reads identity from `TokenUser` (Bearer) and organization from `RequestScope`
//! in request extensions (e.g. from scope-from-headers middleware) or from the user's auth context.

use axum_login::AuthnBackend;
use forge_auth::token_auth::TokenUser;
use forge_auth::{AuthzContext, RequestScope};
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
pub fn requester_org_key_from_user<U: AuthzContext>(user: Option<&U>) -> RequesterOrgKey
where
  <U as AuthzContext>::RequesterId: Into<uuid::Uuid>,
{
  user
    .map(|u| RequesterOrgKey {
      organization_id: u.organization_id(),
      user_id: u.requester_id().into(),
    })
    .unwrap_or(RequesterOrgKey {
      organization_id: None,
      user_id: uuid::Uuid::nil(),
    })
}

/// Extracts `(organization_id, user_id)` from request extensions: user from
/// `TokenUser` (Bearer); organization from `RequestScope` (scope-from-headers middleware)
/// or from the user's context.
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
  B::User: AuthzContext<RequesterId = uuid::Uuid, SubjectId = uuid::Uuid> + Send,
{
  type Key = RequesterOrgKey;

  fn extract<T>(&self, req: &axum::http::Request<T>) -> Result<Self::Key, GovernorError> {
    let ext = req.extensions();
    let (user_id, org_from_user) = if let Some(tu) = ext.get::<TokenUser<B::User>>() {
      (tu.requester_id(), tu.organization_id())
    } else {
      (uuid::Uuid::nil(), None)
    };
    let organization_id = ext
      .get::<RequestScope>()
      .map(|s| s.organization_id)
      .or(org_from_user);
    Ok(RequesterOrgKey {
      organization_id,
      user_id,
    })
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
    type RequesterId = uuid::Uuid;
    type SubjectId = uuid::Uuid;
    fn requester_id(&self) -> Self::RequesterId {
      self.user_id
    }
    fn subject_id(&self) -> Self::SubjectId {
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

  /// Extractor with TokenUser + RequestScope: org from scope extension, user_id from TokenUser.
  #[test]
  fn requester_org_key_extractor_with_token_user_and_scope() {
    use axum::http::Extensions;
    use forge_auth::token_auth::TokenUser;
    use forge_auth::RequestScope;
    use forge_auth::Role;

    static H: [u8; 0] = [];
    #[derive(Clone, Debug)]
    struct U;
    impl AuthUser for U {
      type Id = uuid::Uuid;
      fn id(&self) -> Self::Id {
        uuid::Uuid::nil()
      }
      fn session_auth_hash(&self) -> &[u8] {
        &H
      }
    }
    impl AuthzContext for U {
      type RequesterId = uuid::Uuid;
      type SubjectId = uuid::Uuid;
      fn requester_id(&self) -> Self::RequesterId {
        uuid::Uuid::nil()
      }
      fn subject_id(&self) -> Self::SubjectId {
        uuid::Uuid::nil()
      }
      fn organization_id(&self) -> Option<uuid::Uuid> {
        None
      }
    }

    #[derive(Clone)]
    struct Be;
    #[async_trait::async_trait]
    impl axum_login::AuthnBackend for Be {
      type User = U;
      type Credentials = ();
      type Error = std::io::Error;
      async fn authenticate(
        &self,
        _: Self::Credentials,
      ) -> Result<Option<Self::User>, Self::Error> {
        Ok(None)
      }
      async fn get_user(
        &self,
        _: &axum_login::UserId<Self>,
      ) -> Result<Option<Self::User>, Self::Error> {
        Ok(None)
      }
    }

    let org_id = uuid::Uuid::new_v4();
    let mut req = Request::builder().body(()).unwrap();
    req.extensions_mut().insert(TokenUser::<U> {
      user: U,
      extensions: Extensions::new(),
    });
    req.extensions_mut().insert(RequestScope {
      organization_id: org_id,
      role: Role::Viewer,
    });
    let key = RequesterOrgKeyExtractor::<Be>::new().extract(&req).unwrap();
    assert_eq!(key.organization_id, Some(org_id), "org from RequestScope");
    assert_eq!(key.user_id, uuid::Uuid::nil(), "user_id from TokenUser");
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
      type RequesterId = uuid::Uuid;
      type SubjectId = uuid::Uuid;
      fn requester_id(&self) -> Self::RequesterId {
        uuid::Uuid::nil()
      }
      fn subject_id(&self) -> Self::SubjectId {
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

  /// Test extractor when TokenUser and RequestScope are in extensions (user present branch).
  #[tokio::test]
  async fn requester_org_key_extractor_with_token_user() {
    use async_trait::async_trait;
    use axum::http::Request;
    use axum_login::{AuthnBackend, UserId};
    use forge_auth::Role;

    static AUTH_HASH: [u8; 0] = [];
    #[derive(Clone, Debug)]
    struct TestUser {
      organization_id: Option<uuid::Uuid>,
      user_id: uuid::Uuid,
    }
    impl axum_login::AuthUser for TestUser {
      type Id = uuid::Uuid;
      fn id(&self) -> Self::Id {
        self.user_id
      }
      fn session_auth_hash(&self) -> &[u8] {
        &AUTH_HASH
      }
    }
    impl AuthzContext for TestUser {
      type RequesterId = uuid::Uuid;
      type SubjectId = uuid::Uuid;
      fn requester_id(&self) -> Self::RequesterId {
        self.user_id
      }
      fn subject_id(&self) -> Self::SubjectId {
        self.user_id
      }
      fn organization_id(&self) -> Option<uuid::Uuid> {
        self.organization_id
      }
    }

    #[derive(Clone)]
    struct TestBackend;
    #[async_trait]
    impl AuthnBackend for TestBackend {
      type User = TestUser;
      type Credentials = ();
      type Error = std::convert::Infallible;
      async fn authenticate(&self, _: Self::Credentials) -> Result<Option<Self::User>, Self::Error> {
        Ok(None)
      }
      async fn get_user(&self, _: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        Ok(None)
      }
    }
    let org_id = uuid::Uuid::new_v4();
    let user_id = uuid::Uuid::new_v4();
    let user = TestUser {
      organization_id: Some(org_id),
      user_id,
    };
    let token_user = TokenUser::<TestUser> {
      user: user.clone(),
      extensions: axum::http::Extensions::new(),
    };
    let mut req = Request::builder().body(()).unwrap();
    req.extensions_mut().insert(token_user);
    req.extensions_mut().insert(RequestScope {
      organization_id: org_id,
      role: Role::Admin,
    });
    let key = RequesterOrgKeyExtractor::<TestBackend>::new().extract(&req).unwrap();
    assert_eq!(key.organization_id, Some(org_id));
    assert_eq!(key.user_id, user_id);
  }
}
