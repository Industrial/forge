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
    let key = req
      .extensions()
      .get::<AuthSession<B>>()
      .and_then(|auth| auth.user.as_ref())
      .map(|user| RequesterOrgKey {
        organization_id: user.organization_id(),
        user_id: user.requester_id(),
      })
      .unwrap_or(RequesterOrgKey {
        organization_id: None,
        user_id: uuid::Uuid::nil(),
      });
    Ok(key)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use axum::http::Request;
  use axum_login::AuthUser;

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
  }

  #[test]
  fn requester_org_key_extractor_new_and_default() {
    use async_trait::async_trait;
    #[derive(Clone)]
    struct DummyBackend;
    #[async_trait]
    impl AuthnBackend for DummyBackend {
      type User = MockUser;
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
        _: &<Self::User as AuthUser>::Id,
      ) -> Result<Option<Self::User>, Self::Error> {
        Ok(None)
      }
    }
    static MOCK_AUTH_HASH: [u8; 0] = [];
    #[derive(Clone, Debug)]
    struct MockUser;
    impl AuthUser for MockUser {
      type Id = uuid::Uuid;
      fn id(&self) -> Self::Id {
        uuid::Uuid::nil()
      }
      fn session_auth_hash(&self) -> &[u8] {
        &MOCK_AUTH_HASH
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
}
