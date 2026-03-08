//! Auth handlers under /api/auth (authc + authz, flat REST).

mod admin;
mod global_role_assignments;
mod login;
mod logout;
mod me;
mod organizations;
mod permissions;
mod profiles;
mod register;
mod role_permissions;
mod roles;
mod shared;
mod tokens;
mod users;

pub use admin::admin_only;
pub use login::{LoginRequest, login};
pub use logout::logout;
pub use me::get_me;
pub use permissions::list_permissions;
pub use profiles::profiles_list;
pub use register::{RegisterRequest, register};
pub use role_permissions::{add_role_permission, delete_role_permission, list_role_permissions};
pub use roles::{create_role, delete_role, list_roles, update_role};
pub use tokens::{CreateTokenRequest, create_token};
pub use users::{create_user, delete_user, list_users, update_user};

pub use global_role_assignments::{
  add_global_role_assignment, delete_global_role_assignment, list_global_role_assignments,
};
pub use organizations::{
  create_organization, delete_organization, list_organizations, update_organization,
};

// Re-export shared for dashboard and other handlers.
pub use crate::permissions::dashboard_permissions;
pub use shared::{
  HEADER_ORGANIZATION_ID, HEADER_ROLE_ID, PERMISSION_AUDIT_READ, PERMISSION_ORGS_READ,
  PERMISSION_ORGS_WRITE, PERMISSION_READ, PERMISSION_ROLES_READ, PERMISSION_ROLES_WRITE,
  PERMISSION_USERS_READ, PERMISSION_USERS_WRITE, PERMISSION_WRITE, ScopeFromHeaders,
  channels_from_permissions, forbidden_response, get_scope_from_headers_map, has_global_scope,
  has_permission, require_any_permission, require_entity_permission, require_permission,
  resolve_permissions, try_scope_from_headers,
};
