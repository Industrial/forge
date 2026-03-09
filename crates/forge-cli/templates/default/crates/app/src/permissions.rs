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
}
