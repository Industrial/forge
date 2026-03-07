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
    "user.create" | "user.update" | "user.delete" => &["user.create", "dashboard.users.write"],
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

#[cfg(test)]
mod tests {
  use super::*;

  mod allowed_permission_keys_behavior {
    use super::*;

    #[test]
    fn allowed_permission_keys_returns_non_empty_slice() {
      // Given the permission system
      // When I call allowed_permission_keys
      let keys = allowed_permission_keys();

      // Then it should return a non-empty slice
      assert!(
        !keys.is_empty(),
        "Should return at least some permission keys"
      );
    }

    #[test]
    fn allowed_permission_keys_includes_wildcard_keys() {
      // Given the permission system
      // When I call allowed_permission_keys
      let keys = allowed_permission_keys();

      // Then it should include the wildcard keys
      assert!(keys.contains(&ALL_READ), "Should include all.read wildcard");
      assert!(
        keys.contains(&ALL_WRITE),
        "Should include all.write wildcard"
      );
    }

    #[test]
    fn dashboard_permissions_returns_same_as_allowed_permission_keys() {
      // Given the permission system
      // When I call both functions
      let allowed = allowed_permission_keys();
      let dashboard = dashboard_permissions();

      // Then they should return the same slice
      assert_eq!(
        allowed.as_ptr(),
        dashboard.as_ptr(),
        "dashboard_permissions should return same slice as allowed_permission_keys"
      );
      assert_eq!(
        allowed.len(),
        dashboard.len(),
        "Both should have same length"
      );
    }
  }

  mod entity_action_key_behavior {
    use super::*;

    #[test]
    fn entity_action_key_builds_correct_format() {
      // Given an entity name and action
      let entity = "user";
      let action = "read";

      // When I build a permission key
      let key = entity_action_key(entity, action);

      // Then it should be in the format "entity.action"
      assert_eq!(key, "user.read");
    }

    #[test]
    fn entity_action_key_handles_different_actions() {
      // Given an entity name and different actions
      let entity = "organization";

      // When I build permission keys for different actions
      let create_key = entity_action_key(entity, "create");
      let read_key = entity_action_key(entity, "read");
      let update_key = entity_action_key(entity, "update");
      let delete_key = entity_action_key(entity, "delete");

      // Then they should all be correctly formatted
      assert_eq!(create_key, "organization.create");
      assert_eq!(read_key, "organization.read");
      assert_eq!(update_key, "organization.update");
      assert_eq!(delete_key, "organization.delete");
    }

    #[test]
    fn entity_action_key_handles_different_entities() {
      // Given different entity names and the same action
      let action = "read";

      // When I build permission keys for different entities
      let user_key = entity_action_key("user", action);
      let role_key = entity_action_key("role", action);
      let permission_key = entity_action_key("permission", action);

      // Then they should all be correctly formatted
      assert_eq!(user_key, "user.read");
      assert_eq!(role_key, "role.read");
      assert_eq!(permission_key, "permission.read");
    }
  }

  mod permission_equivalents_behavior {
    use super::*;

    #[test]
    fn permission_equivalents_returns_entity_and_legacy_for_organization_read() {
      // Given the permission key "organization.read"
      let key = "organization.read";

      // When I get permission equivalents
      let equivalents = permission_equivalents(key);

      // Then it should return both entity and legacy dashboard keys
      assert_eq!(equivalents.len(), 2);
      assert!(equivalents.contains(&"organization.read"));
      assert!(equivalents.contains(&"dashboard.organizations.read"));
    }

    #[test]
    fn permission_equivalents_returns_write_equivalents_for_organization_create() {
      // Given the permission key "organization.create"
      let key = "organization.create";

      // When I get permission equivalents
      let equivalents = permission_equivalents(key);

      // Then it should return both entity create and legacy write keys
      assert_eq!(equivalents.len(), 2);
      assert!(equivalents.contains(&"organization.create"));
      assert!(equivalents.contains(&"dashboard.organizations.write"));
    }

    #[test]
    fn permission_equivalents_handles_user_permissions() {
      // Given user permission keys
      // When I get permission equivalents for user.read
      let read_equivs = permission_equivalents("user.read");
      assert_eq!(read_equivs.len(), 2);
      assert!(read_equivs.contains(&"user.read"));
      assert!(read_equivs.contains(&"dashboard.users.read"));

      // And when I get permission equivalents for user.create
      let create_equivs = permission_equivalents("user.create");
      assert_eq!(create_equivs.len(), 2);
      assert!(create_equivs.contains(&"user.create"));
      assert!(create_equivs.contains(&"dashboard.users.write"));
    }

    #[test]
    fn permission_equivalents_handles_role_permissions() {
      // Given role permission keys
      // When I get permission equivalents for role.read
      let read_equivs = permission_equivalents("role.read");
      assert_eq!(read_equivs.len(), 2);
      assert!(read_equivs.contains(&"role.read"));
      assert!(read_equivs.contains(&"dashboard.roles.read"));

      // And when I get permission equivalents for role.update
      let update_equivs = permission_equivalents("role.update");
      assert_eq!(update_equivs.len(), 2);
      assert!(update_equivs.contains(&"role.create"));
      assert!(update_equivs.contains(&"dashboard.roles.write"));
    }

    #[test]
    fn permission_equivalents_handles_legacy_dashboard_keys() {
      // Given legacy dashboard permission keys
      // When I get permission equivalents for dashboard.users.read
      let equivs = permission_equivalents("dashboard.users.read");

      // Then it should return both entity and legacy keys
      assert_eq!(equivs.len(), 2);
      assert!(equivs.contains(&"user.read"));
      assert!(equivs.contains(&"dashboard.users.read"));
    }

    #[test]
    fn permission_equivalents_returns_empty_for_unknown_keys() {
      // Given an unknown permission key
      let key = "unknown.permission";

      // When I get permission equivalents
      let equivalents = permission_equivalents(key);

      // Then it should return an empty slice
      assert!(
        equivalents.is_empty(),
        "Unknown keys should return empty slice"
      );
    }

    #[test]
    fn permission_equivalents_handles_all_crud_actions_for_organization() {
      // Given organization CRUD actions
      // When I get permission equivalents for each action
      let create_equivs = permission_equivalents("organization.create");
      let update_equivs = permission_equivalents("organization.update");
      let delete_equivs = permission_equivalents("organization.delete");

      // Then create, update, and delete should all map to the same write equivalents
      assert_eq!(create_equivs, update_equivs);
      assert_eq!(update_equivs, delete_equivs);
      assert!(create_equivs.contains(&"organization.create"));
      assert!(create_equivs.contains(&"dashboard.organizations.write"));
    }

    #[test]
    fn permission_equivalents_handles_audit_read() {
      // Given the audit.read permission key
      let key = "audit.read";

      // When I get permission equivalents
      let equivalents = permission_equivalents(key);

      // Then it should return both entity and legacy keys
      assert_eq!(equivalents.len(), 2);
      assert!(equivalents.contains(&"audit.read"));
      assert!(equivalents.contains(&"dashboard.audit.read"));
    }
  }
}
