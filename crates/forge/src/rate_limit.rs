//! Rate limiting key extractors and helpers.
//!
//! Per-IP limiting uses tower_governor's `PeerIpKeyExtractor` (see `app.rs`).
//! Per-requester (per user per organization) uses `RequesterOrgKeyExtractor`,
//! which reads `AuthSession` from request extensions (set by axum-login's layer).

use axum_login::{AuthSession, AuthnBackend};
use tower_governor::{errors::GovernorError, key_extractor::KeyExtractor};

use crate::authz::AuthzContext;

/// Key for per-requester rate limiting: `(organization_id, user_id)`.
/// Each user has a separate limit within each organization.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RequesterOrgKey {
  pub organization_id: Option<uuid::Uuid>,
  pub user_id: uuid::Uuid,
}

/// Extracts `(organization_id, user_id)` from `AuthSession` in request extensions.
/// Used for rate limiting per requester (per user, per organization).
///
/// Authenticated requests are keyed by (org_id, user_id). Unauthenticated requests
/// use a sentinel key `(None, nil)` so they are rate-limited in a single bucket
/// but not rejected (register/login must work without auth).
#[derive(Clone, Debug)]
pub struct RequesterOrgKeyExtractor<B> {
  /// Phantom data for the backend type `B` (no runtime value).
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
