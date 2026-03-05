//! Entity-based permission keys per docs/technical-choices/02-entity-based-permissions.md.
//! Format: `<entity>.<action>` with action in {create, read, update, delete}.
//! Wildcards: all.read / all.write. Allowed keys derived from entity registry (Epic 3, §6).

use crate::registry;

/// Wildcard keys retained for resolution (§2).
pub const ALL_READ: &str = "all.read";
pub const ALL_WRITE: &str = "all.write";

/// Allowed permission keys derived from the entity registry (§6). Use for validation and listing.
#[inline]
pub fn allowed_permission_keys() -> &'static [&'static str] {
  registry::allowed_permission_keys()
}

/// Legacy name: returns the same slice as [allowed_permission_keys]. Use for handlers during migration.
#[inline]
pub fn dashboard_permissions() -> &'static [&'static str] {
  registry::allowed_permission_keys()
}

/// Builds a permission key `<entity>.<action>`.
#[inline]
pub fn entity_action_key(entity: &str, action: &str) -> String {
  format!("{}.{}", entity, action)
}

/// Legacy dashboard.* key equivalence for migration (§8). Returns keys to check (entity + legacy).
/// Empty slice means caller should use exact match only (key itself).
pub fn permission_equivalents(key: &str) -> &'static [&'static str] {
  match key {
    "organization.read" => &["organization.read", "dashboard.organizations.read"],
    "organization.create" | "organization.update" | "organization.delete" => {
      &["organization.create", "dashboard.organizations.write"]
    }
    "user.read" => &["user.read", "dashboard.users.read"],
    "user.create" | "user.update" | "user.delete" => {
      &["user.create", "dashboard.users.write"]
    }
    "role.read" => &["role.read", "dashboard.roles.read"],
    "role.create" | "role.update" | "role.delete" => &["role.create", "dashboard.roles.write"],
    "permission.read" => &["permission.read", "dashboard.permissions.read"],
    "permission.create" | "permission.update" | "permission.delete" => {
      &["permission.create", "dashboard.permissions.write"]
    }
    "audit.read" => &["audit.read", "dashboard.audit.read"],
    "dashboard.organizations.read" => &["organization.read", "dashboard.organizations.read"],
    "dashboard.organizations.write" => &["organization.create", "dashboard.organizations.write"],
    "dashboard.users.read" => &["user.read", "dashboard.users.read"],
    "dashboard.users.write" => &["user.create", "dashboard.users.write"],
    "dashboard.roles.read" => &["role.read", "dashboard.roles.read"],
    "dashboard.roles.write" => &["role.create", "dashboard.roles.write"],
    "dashboard.permissions.read" => &["permission.read", "dashboard.permissions.read"],
    "dashboard.permissions.write" => &["permission.create", "dashboard.permissions.write"],
    "dashboard.audit.read" => &["audit.read", "dashboard.audit.read"],
    _ => &[],
  }
}
