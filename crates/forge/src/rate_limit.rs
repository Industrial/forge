//! Rate limiting key extractors and helpers.
//!
//! Per-IP limiting uses tower_governor's `PeerIpKeyExtractor` (see `app.rs`).
//! Per-requester (per user per organization) uses `RequesterOrgKeyExtractor`,
//! which reads `AuthSession` from request extensions (set by axum-login's layer).

use axum::http::StatusCode;
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
/// Requires auth to be installed so that `AuthSession<B>` is in extensions.
/// If there is no session or no user, returns `GovernorError` with 401.
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
    let auth = req
      .extensions()
      .get::<AuthSession<B>>()
      .ok_or_else(|| GovernorError::Other {
        code: StatusCode::UNAUTHORIZED,
        msg: Some("Rate limit requires authentication".to_string()),
        headers: None,
      })?;
    let user = auth.user.as_ref().ok_or_else(|| GovernorError::Other {
      code: StatusCode::UNAUTHORIZED,
      msg: Some("Rate limit requires authenticated user".to_string()),
      headers: None,
    })?;
    Ok(RequesterOrgKey {
      organization_id: user.organization_id(),
      user_id: user.requester_id(),
    })
  }
}
