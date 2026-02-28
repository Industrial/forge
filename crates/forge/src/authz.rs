pub use forge_authz::*;

use axum_login::{AuthSession, AuthnBackend};

// AuthzContext is implemented for AuthSession in forge-authz directly.

/// Extension trait for AuthSession to provide authorization guards.
pub trait AuthSessionGuardExt {
  /// Guard a handler by requiring a specific action and role.
  fn guard(&self, action: Action, role: Role) -> Result<(), AuthzError>;
}

impl<B> AuthSessionGuardExt for AuthSession<B>
where
  B: AuthnBackend,
  B::User: AuthzContext,
{
  fn guard(&self, _action: Action, role: Role) -> Result<(), AuthzError> {
    if self.user.is_none() {
      return Err(AuthzError::Forbidden);
    }

    // Require the session user to have the given role (or higher) in the current org.
    match self.role() {
      Some(user_role) if user_role == role => Ok(()),
      Some(Role::Owner) => Ok(()), // Owner can satisfy any role check
      Some(Role::Admin) if matches!(role, Role::Admin | Role::Editor | Role::Viewer) => Ok(()),
      Some(Role::Editor) if matches!(role, Role::Editor | Role::Viewer) => Ok(()),
      Some(Role::Viewer) if role == Role::Viewer => Ok(()),
      _ => Err(AuthzError::Forbidden),
    }
  }
}
