//! Channel names for scoped broadcast.

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// A subscription channel (e.g. `org:{id}`, `resource:users:{id}`).
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Channel(String);

impl Channel {
  /// Channel for all connections in an organization.
  pub fn org(org_id: Uuid) -> Self {
    Channel(format!("org:{}", org_id))
  }

  /// Channel for a resource type within an org (permission-scoped live updates).
  /// Format: `org:{org_id}:{resource_type}` (e.g. `org:...:users`, `org:...:roles`).
  pub fn org_resource(org_id: Uuid, resource_type: &str) -> Self {
    Channel(format!("org:{}:{}", org_id, resource_type))
  }

  /// Channel for a specific resource (e.g. `resource:users:{id}`).
  pub fn resource(resource_type: &str, id: Uuid) -> Self {
    Channel(format!("resource:{}:{}", resource_type, id))
  }

  /// Raw channel name (for custom patterns).
  pub fn raw(name: impl Into<String>) -> Self {
    Channel(name.into())
  }

  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl fmt::Display for Channel {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

#[cfg(test)]
mod bdd_tests {
  use super::*;

  /// BDD-style tests focusing on behavior rather than implementation.
  /// Tests are organized by feature/behavior area with descriptive names.
  mod org_channel_creation_behavior {
    use super::*;

    #[test]
    fn should_create_channel_with_org_prefix() {
      // Given: an organization ID
      let org_id = Uuid::new_v4();

      // When: creating an org channel
      let channel = Channel::org(org_id);

      // Then: should have org: prefix
      let channel_str = channel.as_str();
      assert!(
        channel_str.starts_with("org:"),
        "Channel should start with 'org:' prefix"
      );
    }

    #[test]
    fn should_include_org_id_in_channel_name() {
      // Given: an organization ID
      let org_id = Uuid::new_v4();

      // When: creating an org channel
      let channel = Channel::org(org_id);

      // Then: channel name should include the org ID
      let channel_str = channel.as_str();
      assert!(
        channel_str.contains(&org_id.to_string()),
        "Channel should include organization ID"
      );
    }

    #[test]
    fn should_create_different_channels_for_different_orgs() {
      // Given: different organization IDs
      let org1_id = Uuid::new_v4();
      let org2_id = Uuid::new_v4();

      // When: creating channels for each org
      let channel1 = Channel::org(org1_id);
      let channel2 = Channel::org(org2_id);

      // Then: channels should be different
      assert_ne!(
        channel1.as_str(),
        channel2.as_str(),
        "Different orgs should have different channels"
      );
    }

    #[test]
    fn should_create_same_channel_for_same_org() {
      // Given: the same organization ID
      let org_id = Uuid::new_v4();

      // When: creating channels multiple times
      let channel1 = Channel::org(org_id);
      let channel2 = Channel::org(org_id);

      // Then: channels should be identical
      assert_eq!(
        channel1.as_str(),
        channel2.as_str(),
        "Same org should produce same channel"
      );
    }
  }

  mod org_resource_channel_creation_behavior {
    use super::*;

    #[test]
    fn should_create_channel_with_org_and_resource_type() {
      // Given: an organization ID and resource type
      let org_id = Uuid::new_v4();
      let resource_type = "users";

      // When: creating an org resource channel
      let channel = Channel::org_resource(org_id, resource_type);

      // Then: should have org: prefix and resource type
      let channel_str = channel.as_str();
      assert!(
        channel_str.starts_with("org:"),
        "Channel should start with 'org:' prefix"
      );
      assert!(
        channel_str.contains(&org_id.to_string()),
        "Channel should include organization ID"
      );
      assert!(
        channel_str.ends_with(":users"),
        "Channel should end with resource type"
      );
    }

    #[test]
    fn should_create_different_channels_for_different_resource_types() {
      // Given: same org ID but different resource types
      let org_id = Uuid::new_v4();

      // When: creating channels for different resource types
      let users_channel = Channel::org_resource(org_id, "users");
      let roles_channel = Channel::org_resource(org_id, "roles");

      // Then: channels should be different
      assert_ne!(
        users_channel.as_str(),
        roles_channel.as_str(),
        "Different resource types should have different channels"
      );
    }

    #[test]
    fn should_create_different_channels_for_different_orgs_with_same_resource() {
      // Given: different org IDs but same resource type
      let org1_id = Uuid::new_v4();
      let org2_id = Uuid::new_v4();

      // When: creating channels for each org with same resource
      let channel1 = Channel::org_resource(org1_id, "users");
      let channel2 = Channel::org_resource(org2_id, "users");

      // Then: channels should be different
      assert_ne!(
        channel1.as_str(),
        channel2.as_str(),
        "Different orgs should have different channels even with same resource"
      );
    }
  }

  mod resource_channel_creation_behavior {
    use super::*;

    #[test]
    fn should_create_channel_with_resource_prefix() {
      // Given: a resource type and ID
      let resource_type = "users";
      let resource_id = Uuid::new_v4();

      // When: creating a resource channel
      let channel = Channel::resource(resource_type, resource_id);

      // Then: should have resource: prefix
      let channel_str = channel.as_str();
      assert!(
        channel_str.starts_with("resource:"),
        "Channel should start with 'resource:' prefix"
      );
    }

    #[test]
    fn should_include_resource_type_and_id_in_channel_name() {
      // Given: a resource type and ID
      let resource_type = "tasks";
      let resource_id = Uuid::new_v4();

      // When: creating a resource channel
      let channel = Channel::resource(resource_type, resource_id);

      // Then: channel name should include both type and ID
      let channel_str = channel.as_str();
      assert!(
        channel_str.contains(resource_type),
        "Channel should include resource type"
      );
      assert!(
        channel_str.contains(&resource_id.to_string()),
        "Channel should include resource ID"
      );
    }

    #[test]
    fn should_create_different_channels_for_different_resources() {
      // Given: different resource IDs
      let resource_id1 = Uuid::new_v4();
      let resource_id2 = Uuid::new_v4();

      // When: creating channels for each resource
      let channel1 = Channel::resource("users", resource_id1);
      let channel2 = Channel::resource("users", resource_id2);

      // Then: channels should be different
      assert_ne!(
        channel1.as_str(),
        channel2.as_str(),
        "Different resources should have different channels"
      );
    }
  }

  mod raw_channel_creation_behavior {
    use super::*;

    #[test]
    fn should_create_channel_from_string() {
      // Given: a string channel name
      let channel_name = "custom:channel:name";

      // When: creating a raw channel
      let channel = Channel::raw(channel_name);

      // Then: channel should have the exact name
      assert_eq!(
        channel.as_str(),
        channel_name,
        "Raw channel should have exact name provided"
      );
    }

    #[test]
    fn should_create_channel_from_string_slice() {
      // Given: a string slice
      let channel_name = "test-channel";

      // When: creating a raw channel
      let channel = Channel::raw(channel_name);

      // Then: channel should have the name
      assert_eq!(
        channel.as_str(),
        channel_name,
        "Raw channel should accept string slice"
      );
    }

    #[test]
    fn should_create_channel_from_owned_string() {
      // Given: an owned String
      let channel_name = String::from("owned-channel");

      // When: creating a raw channel
      let channel = Channel::raw(channel_name.clone());

      // Then: channel should have the name
      assert_eq!(
        channel.as_str(),
        channel_name.as_str(),
        "Raw channel should accept owned String"
      );
    }
  }

  mod channel_string_access_behavior {
    use super::*;

    #[test]
    fn should_return_channel_as_string_slice() {
      // Given: a channel
      let org_id = Uuid::new_v4();
      let channel = Channel::org(org_id);

      // When: getting channel as string
      let channel_str = channel.as_str();

      // Then: should return valid string slice
      assert!(
        !channel_str.is_empty(),
        "Channel string should not be empty"
      );
      assert!(
        channel_str.starts_with("org:"),
        "Channel string should match channel content"
      );
    }

    #[test]
    fn should_return_same_string_for_same_channel() {
      // Given: the same channel created twice
      let org_id = Uuid::new_v4();
      let channel1 = Channel::org(org_id);
      let channel2 = Channel::org(org_id);

      // When: getting string representation
      let str1 = channel1.as_str();
      let str2 = channel2.as_str();

      // Then: strings should be equal
      assert_eq!(str1, str2, "Same channel should return same string");
    }
  }

  mod channel_display_behavior {
    use super::*;

    #[test]
    fn should_format_channel_via_display_trait() {
      // Given: a channel
      let org_id = Uuid::new_v4();
      let channel = Channel::org(org_id);

      // When: formatting via Display trait
      let formatted = format!("{}", channel);

      // Then: should match as_str output
      assert_eq!(
        formatted,
        channel.as_str(),
        "Display format should match as_str"
      );
    }

    #[test]
    fn should_include_channel_in_formatted_string() {
      // Given: a channel
      let channel = Channel::raw("test-channel");

      // When: formatting with additional text
      let formatted = format!("Channel: {}", channel);

      // Then: should include channel name
      assert!(
        formatted.contains("test-channel"),
        "Formatted string should include channel name"
      );
    }
  }

  mod channel_equality_behavior {
    use super::*;

    #[test]
    fn should_be_equal_for_same_channel_name() {
      // Given: channels with same name created differently
      let org_id = Uuid::new_v4();
      let channel1 = Channel::org(org_id);
      let channel2 = Channel::org(org_id);

      // When: comparing channels
      // Then: should be equal
      assert_eq!(
        channel1, channel2,
        "Channels with same name should be equal"
      );
    }

    #[test]
    fn should_not_be_equal_for_different_channel_names() {
      // Given: channels with different names
      let org1_id = Uuid::new_v4();
      let org2_id = Uuid::new_v4();
      let channel1 = Channel::org(org1_id);
      let channel2 = Channel::org(org2_id);

      // When: comparing channels
      // Then: should not be equal
      assert_ne!(
        channel1, channel2,
        "Channels with different names should not be equal"
      );
    }

    #[test]
    fn should_be_hashable_for_use_in_hash_maps() {
      // Given: channels
      use std::collections::HashMap;
      let org_id = Uuid::new_v4();
      let channel = Channel::org(org_id);

      // When: using channel as HashMap key
      let mut map = HashMap::new();
      map.insert(channel.clone(), "value");

      // Then: should be able to retrieve value
      assert_eq!(
        map.get(&channel),
        Some(&"value"),
        "Channel should work as HashMap key"
      );
    }
  }

  mod channel_cloning_behavior {
    use super::*;

    #[test]
    fn should_clone_channel_with_same_content() {
      // Given: a channel
      let org_id = Uuid::new_v4();
      let original = Channel::org(org_id);

      // When: cloning the channel
      let cloned = original.clone();

      // Then: cloned channel should be equal
      assert_eq!(
        original, cloned,
        "Cloned channel should be equal to original"
      );
      assert_eq!(
        original.as_str(),
        cloned.as_str(),
        "Cloned channel should have same string representation"
      );
    }

    #[test]
    fn should_create_independent_copy_when_cloned() {
      // Given: a channel
      let original = Channel::raw("original");

      // When: cloning and modifying (if possible)
      let cloned = original.clone();

      // Then: original should remain unchanged
      assert_eq!(
        original.as_str(),
        "original",
        "Original channel should remain unchanged after clone"
      );
      assert_eq!(
        cloned.as_str(),
        "original",
        "Cloned channel should have same content"
      );
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::collections::HashSet;

  mod bdd_tests {
    use super::*;

    /// BDD-style tests focusing on behavior rather than implementation.
    /// Tests verify Channel creation methods, trait implementations, and serialization.
    mod org_channel_behavior {
      use super::*;

      #[test]
      fn should_create_org_channel_with_uuid() {
        // Given: an organization UUID
        let org_id = Uuid::new_v4();

        // When: creating org channel
        let channel = Channel::org(org_id);

        // Then: channel name should be formatted as "org:{uuid}"
        let expected = format!("org:{}", org_id);
        assert_eq!(channel.as_str(), expected);
      }

      #[test]
      fn should_create_different_org_channels_for_different_orgs() {
        // Given: different organization UUIDs
        let org_id1 = Uuid::new_v4();
        let org_id2 = Uuid::new_v4();

        // When: creating org channels
        let channel1 = Channel::org(org_id1);
        let channel2 = Channel::org(org_id2);

        // Then: channels should be different
        assert_ne!(channel1.as_str(), channel2.as_str());
      }

      #[test]
      fn should_create_same_org_channel_for_same_org() {
        // Given: same organization UUID
        let org_id = Uuid::new_v4();

        // When: creating org channels with same UUID
        let channel1 = Channel::org(org_id);
        let channel2 = Channel::org(org_id);

        // Then: channels should be equal
        assert_eq!(channel1, channel2);
        assert_eq!(channel1.as_str(), channel2.as_str());
      }
    }

    mod org_resource_channel_behavior {
      use super::*;

      #[test]
      fn should_create_org_resource_channel() {
        // Given: organization UUID and resource type
        let org_id = Uuid::new_v4();
        let resource_type = "users";

        // When: creating org_resource channel
        let channel = Channel::org_resource(org_id, resource_type);

        // Then: channel name should be formatted as "org:{uuid}:{resource_type}"
        let expected = format!("org:{}:{}", org_id, resource_type);
        assert_eq!(channel.as_str(), expected);
      }

      #[test]
      fn should_create_different_org_resource_channels_for_different_resources() {
        // Given: same organization but different resource types
        let org_id = Uuid::new_v4();

        // When: creating org_resource channels
        let channel1 = Channel::org_resource(org_id, "users");
        let channel2 = Channel::org_resource(org_id, "tasks");

        // Then: channels should be different
        assert_ne!(channel1.as_str(), channel2.as_str());
      }

      #[test]
      fn should_create_different_org_resource_channels_for_different_orgs() {
        // Given: different organizations but same resource type
        let org_id1 = Uuid::new_v4();
        let org_id2 = Uuid::new_v4();

        // When: creating org_resource channels
        let channel1 = Channel::org_resource(org_id1, "users");
        let channel2 = Channel::org_resource(org_id2, "users");

        // Then: channels should be different
        assert_ne!(channel1.as_str(), channel2.as_str());
      }

      #[test]
      fn should_support_various_resource_types() {
        // Given: different resource types
        let org_id = Uuid::new_v4();
        let resource_types = vec!["users", "tasks", "projects", "roles", "permissions"];

        // When: creating org_resource channels for each type
        // Then: all should be valid and different
        let mut channels = Vec::new();
        for resource_type in &resource_types {
          let channel = Channel::org_resource(org_id, resource_type);
          assert!(channel.as_str().contains(resource_type));
          channels.push(channel);
        }

        // Verify all are unique
        let mut unique = HashSet::new();
        for channel in &channels {
          assert!(unique.insert(channel.as_str()), "Channels should be unique");
        }
      }
    }

    mod resource_channel_behavior {
      use super::*;

      #[test]
      fn should_create_resource_channel() {
        // Given: resource type and resource ID
        let resource_type = "users";
        let resource_id = Uuid::new_v4();

        // When: creating resource channel
        let channel = Channel::resource(resource_type, resource_id);

        // Then: channel name should be formatted as "resource:{resource_type}:{id}"
        let expected = format!("resource:{}:{}", resource_type, resource_id);
        assert_eq!(channel.as_str(), expected);
      }

      #[test]
      fn should_create_different_resource_channels_for_different_ids() {
        // Given: same resource type but different IDs
        let resource_type = "users";
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        // When: creating resource channels
        let channel1 = Channel::resource(resource_type, id1);
        let channel2 = Channel::resource(resource_type, id2);

        // Then: channels should be different
        assert_ne!(channel1.as_str(), channel2.as_str());
      }

      #[test]
      fn should_create_different_resource_channels_for_different_types() {
        // Given: different resource types but same ID
        let id = Uuid::new_v4();

        // When: creating resource channels
        let channel1 = Channel::resource("users", id);
        let channel2 = Channel::resource("tasks", id);

        // Then: channels should be different
        assert_ne!(channel1.as_str(), channel2.as_str());
      }

      #[test]
      fn should_support_various_resource_types() {
        // Given: different resource types
        let resource_types = vec!["users", "tasks", "projects", "organizations"];
        let id = Uuid::new_v4();

        // When: creating resource channels for each type
        // Then: all should be valid and different
        let mut channels = Vec::new();
        for resource_type in &resource_types {
          let channel = Channel::resource(resource_type, id);
          assert!(channel.as_str().contains(resource_type));
          channels.push(channel);
        }

        // Verify all are unique
        let mut unique = HashSet::new();
        for channel in &channels {
          assert!(unique.insert(channel.as_str()), "Channels should be unique");
        }
      }
    }

    mod raw_channel_behavior {
      use super::*;

      #[test]
      fn should_create_raw_channel_from_string() {
        // Given: a raw channel name
        let name = "custom:channel:name".to_string();

        // When: creating raw channel
        let channel = Channel::raw(name.clone());

        // Then: channel name should match input
        assert_eq!(channel.as_str(), name);
      }

      #[test]
      fn should_create_raw_channel_from_str() {
        // Given: a raw channel name as &str
        let name = "custom:channel";

        // When: creating raw channel
        let channel = Channel::raw(name);

        // Then: channel name should match input
        assert_eq!(channel.as_str(), name);
      }

      #[test]
      fn should_create_raw_channel_with_any_pattern() {
        // Given: various custom channel patterns
        let patterns = vec![
          "custom:pattern",
          "webhook:deployment",
          "notification:all",
          "system:health",
        ];

        // When: creating raw channels for each pattern
        // Then: all should be valid
        for pattern in &patterns {
          let channel = Channel::raw(*pattern);
          assert_eq!(channel.as_str(), *pattern);
        }
      }

      #[test]
      fn should_accept_string_into_trait() {
        // Given: a String value
        let name: String = "test:channel".to_string();

        // When: creating raw channel
        let channel = Channel::raw(name);

        // Then: should work with Into<String> trait
        assert_eq!(channel.as_str(), "test:channel");
      }
    }

    mod as_str_behavior {
      use super::*;

      #[test]
      fn should_return_channel_name_as_str() {
        // Given: a channel
        let org_id = Uuid::new_v4();
        let channel = Channel::org(org_id);

        // When: getting channel name as str
        let name = channel.as_str();

        // Then: should return string slice
        assert_eq!(name, format!("org:{}", org_id));
      }

      #[test]
      fn should_return_same_str_for_equal_channels() {
        // Given: equal channels
        let org_id = Uuid::new_v4();
        let channel1 = Channel::org(org_id);
        let channel2 = Channel::org(org_id);

        // When: getting channel names
        // Then: should return same string slice content
        assert_eq!(channel1.as_str(), channel2.as_str());
      }
    }

    mod display_trait_behavior {
      use super::*;

      #[test]
      fn should_format_channel_for_display() {
        // Given: a channel
        let org_id = Uuid::new_v4();
        let channel = Channel::org(org_id);

        // When: formatting for display
        let display_str = format!("{}", channel);

        // Then: should format as channel name
        assert_eq!(display_str, format!("org:{}", org_id));
      }

      #[test]
      fn should_format_different_channel_types() {
        // Given: different channel types
        let org_id = Uuid::new_v4();
        let resource_id = Uuid::new_v4();

        // When: formatting for display
        let org_channel = format!("{}", Channel::org(org_id));
        let org_resource_channel = format!("{}", Channel::org_resource(org_id, "users"));
        let resource_channel = format!("{}", Channel::resource("users", resource_id));
        let raw_channel = format!("{}", Channel::raw("custom:channel"));

        // Then: all should format correctly
        assert_eq!(org_channel, format!("org:{}", org_id));
        assert_eq!(org_resource_channel, format!("org:{}:users", org_id));
        assert_eq!(resource_channel, format!("resource:users:{}", resource_id));
        assert_eq!(raw_channel, "custom:channel");
      }
    }

    mod clone_trait_behavior {
      use super::*;

      #[test]
      fn should_clone_channel() {
        // Given: a channel
        let org_id = Uuid::new_v4();
        let channel = Channel::org(org_id);

        // When: cloning the channel
        let cloned = channel.clone();

        // Then: cloned channel should be equal
        assert_eq!(channel, cloned);
        assert_eq!(channel.as_str(), cloned.as_str());
      }

      #[test]
      fn should_clone_different_channel_types() {
        // Given: different channel types
        let org_id = Uuid::new_v4();
        let resource_id = Uuid::new_v4();

        // When: cloning channels
        let org_clone = Channel::org(org_id).clone();
        let org_resource_clone = Channel::org_resource(org_id, "users").clone();
        let resource_clone = Channel::resource("users", resource_id).clone();
        let raw_clone = Channel::raw("custom").clone();

        // Then: all should clone correctly
        assert_eq!(org_clone.as_str(), format!("org:{}", org_id));
        assert_eq!(org_resource_clone.as_str(), format!("org:{}:users", org_id));
        assert_eq!(
          resource_clone.as_str(),
          format!("resource:users:{}", resource_id)
        );
        assert_eq!(raw_clone.as_str(), "custom");
      }
    }

    mod equality_behavior {
      use super::*;

      #[test]
      fn should_be_equal_for_same_channel_name() {
        // Given: channels with same name
        let org_id = Uuid::new_v4();
        let channel1 = Channel::org(org_id);
        let channel2 = Channel::org(org_id);

        // When: comparing channels
        // Then: should be equal
        assert_eq!(channel1, channel2);
      }

      #[test]
      fn should_be_unequal_for_different_channel_names() {
        // Given: channels with different names
        let org_id1 = Uuid::new_v4();
        let org_id2 = Uuid::new_v4();
        let channel1 = Channel::org(org_id1);
        let channel2 = Channel::org(org_id2);

        // When: comparing channels
        // Then: should be unequal
        assert_ne!(channel1, channel2);
      }

      #[test]
      fn should_be_equal_for_same_raw_channel() {
        // Given: raw channels with same name
        let channel1 = Channel::raw("test:channel");
        let channel2 = Channel::raw("test:channel");

        // When: comparing channels
        // Then: should be equal
        assert_eq!(channel1, channel2);
      }

      #[test]
      fn should_be_unequal_for_different_raw_channels() {
        // Given: raw channels with different names
        let channel1 = Channel::raw("test:channel1");
        let channel2 = Channel::raw("test:channel2");

        // When: comparing channels
        // Then: should be unequal
        assert_ne!(channel1, channel2);
      }
    }

    mod hash_behavior {
      use super::*;

      #[test]
      fn should_be_hashable() {
        // Given: channels
        let org_id = Uuid::new_v4();
        let channel1 = Channel::org(org_id);
        let channel2 = Channel::org(org_id);
        let channel3 = Channel::org(Uuid::new_v4());

        // When: using in HashSet
        let mut set = HashSet::new();
        set.insert(channel1.clone());
        set.insert(channel2.clone());
        set.insert(channel3.clone());

        // Then: equal channels should be treated as same key
        assert_eq!(set.len(), 2); // channel1 and channel2 are equal, channel3 is different
        assert!(set.contains(&channel1));
        assert!(set.contains(&channel2));
        assert!(set.contains(&channel3));
      }

      #[test]
      fn should_hash_different_channel_types() {
        // Given: different channel types
        let org_id = Uuid::new_v4();
        let resource_id = Uuid::new_v4();

        // When: using in HashSet
        let mut set = HashSet::new();
        set.insert(Channel::org(org_id));
        set.insert(Channel::org_resource(org_id, "users"));
        set.insert(Channel::resource("users", resource_id));
        set.insert(Channel::raw("custom"));

        // Then: all should be stored separately
        assert_eq!(set.len(), 4);
      }
    }

    mod debug_trait_behavior {
      use super::*;

      #[test]
      fn should_format_channel_for_debug() {
        // Given: a channel
        let org_id = Uuid::new_v4();
        let channel = Channel::org(org_id);

        // When: formatting for debug
        let debug_str = format!("{:?}", channel);

        // Then: should format successfully
        assert!(debug_str.contains("Channel"));
        assert!(debug_str.contains(&org_id.to_string()));
      }

      #[test]
      fn should_format_different_channel_types_for_debug() {
        // Given: different channel types
        let org_id = Uuid::new_v4();
        let resource_id = Uuid::new_v4();

        // When: formatting for debug
        let org_debug = format!("{:?}", Channel::org(org_id));
        let org_resource_debug = format!("{:?}", Channel::org_resource(org_id, "users"));
        let resource_debug = format!("{:?}", Channel::resource("users", resource_id));
        let raw_debug = format!("{:?}", Channel::raw("custom"));

        // Then: all should format successfully
        assert!(org_debug.contains("Channel"));
        assert!(org_resource_debug.contains("Channel"));
        assert!(resource_debug.contains("Channel"));
        assert!(raw_debug.contains("Channel"));
      }
    }

    mod serialization_behavior {
      use super::*;

      #[test]
      fn should_serialize_org_channel() {
        // Given: an org channel
        let org_id = Uuid::new_v4();
        let channel = Channel::org(org_id);

        // When: serializing to JSON
        let json = serde_json::to_string(&channel).unwrap();

        // Then: should serialize successfully
        assert!(json.contains(&org_id.to_string()));
      }

      #[test]
      fn should_deserialize_org_channel() {
        // Given: JSON string for org channel
        let org_id = Uuid::new_v4();
        let json = format!(r#""org:{}""#, org_id);

        // When: deserializing from JSON
        let channel: Channel = serde_json::from_str(&json).unwrap();

        // Then: should deserialize successfully
        assert_eq!(channel.as_str(), format!("org:{}", org_id));
      }

      #[test]
      fn should_serialize_and_deserialize_resource_channel() {
        // Given: a resource channel
        let resource_id = Uuid::new_v4();
        let channel = Channel::resource("users", resource_id);

        // When: serializing and deserializing
        let json = serde_json::to_string(&channel).unwrap();
        let deserialized: Channel = serde_json::from_str(&json).unwrap();

        // Then: should round-trip correctly
        assert_eq!(channel, deserialized);
        assert_eq!(channel.as_str(), deserialized.as_str());
      }

      #[test]
      fn should_serialize_and_deserialize_raw_channel() {
        // Given: a raw channel
        let channel = Channel::raw("custom:channel:name");

        // When: serializing and deserializing
        let json = serde_json::to_string(&channel).unwrap();
        let deserialized: Channel = serde_json::from_str(&json).unwrap();

        // Then: should round-trip correctly
        assert_eq!(channel, deserialized);
        assert_eq!(channel.as_str(), deserialized.as_str());
      }

      #[test]
      fn should_serialize_and_deserialize_org_resource_channel() {
        // Given: an org_resource channel
        let org_id = Uuid::new_v4();
        let channel = Channel::org_resource(org_id, "users");

        // When: serializing and deserializing
        let json = serde_json::to_string(&channel).unwrap();
        let deserialized: Channel = serde_json::from_str(&json).unwrap();

        // Then: should round-trip correctly
        assert_eq!(channel, deserialized);
        assert_eq!(channel.as_str(), deserialized.as_str());
      }
    }
  }
}
