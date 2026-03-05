//! Entity-based permission keys per docs/technical-choices/02-entity-based-permissions.md.
//! Format: `<entity>.<action>` with action in {create, read, update, delete}.
//! Wildcards: all.read (grants every entity.read), all.write (grants every entity.create/update/delete).
//! Allowed keys are derived from the entity set (§4).

/// Entity identifiers (lowercase). Epic 3 will replace with entity registry.
pub const ENTITIES: &[&str] = &[
  "organization",
  "user",
  "role",
  "permission",
  "audit",
];

/// Allowed actions per entity (§1).
pub const ENTITY_ACTIONS: &[&str] = &["create", "read", "update", "delete"];

/// Wildcard keys retained for resolution (§2).
pub const ALL_READ: &str = "all.read";
pub const ALL_WRITE: &str = "all.write";

/// Returns the full list of allowed permission keys: four per entity plus all.read, all.write (§4).
pub fn allowed_permission_keys() -> Vec<String> {
  let mut keys: Vec<String> = ENTITIES
    .iter()
    .flat_map(|entity| {
      ENTITY_ACTIONS
        .iter()
        .map(move |action| format!("{}.{}", entity, action))
    })
    .collect();
  keys.push(ALL_READ.to_string());
  keys.push(ALL_WRITE.to_string());
  keys
}

/// Builds a permission key `<entity>.<action>`.
#[inline]
pub fn entity_action_key(entity: &str, action: &str) -> String {
  format!("{}.{}", entity, action)
}

/// All allowed permission keys as a static list for validation and listing.
/// Derived from ENTITIES × ENTITY_ACTIONS + all.read, all.write.
pub static ALLOWED_PERMISSION_KEYS: &[&str] = &[
  "organization.create",
  "organization.read",
  "organization.update",
  "organization.delete",
  "user.create",
  "user.read",
  "user.update",
  "user.delete",
  "role.create",
  "role.read",
  "role.update",
  "role.delete",
  "permission.create",
  "permission.read",
  "permission.update",
  "permission.delete",
  "audit.create",
  "audit.read",
  "audit.update",
  "audit.delete",
  ALL_READ,
  ALL_WRITE,
];

/// Legacy alias for ALLOWED_PERMISSION_KEYS (entity-based keys). Used by handlers during migration.
pub const DASHBOARD_PERMISSIONS: &[&str] = ALLOWED_PERMISSION_KEYS;

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
