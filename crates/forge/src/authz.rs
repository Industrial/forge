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
  fn guard(&self, _action: Action, _role: Role) -> Result<(), AuthzError> {
    // Shallow Gate Implementation:
    // In this lean implementation, we check if the user is authenticated.
    // Complex Role/Action mapping can be added here or delegated to the User model.
    if self.user.is_none() {
      return Err(AuthzError::Forbidden);
    }

    // For now, we assume any authenticated user can perform the action.
    // In a real implementation, we would call a policy or check user roles.
    Ok(())
  }
}
