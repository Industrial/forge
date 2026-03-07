//! Re-export REST model trait from forge-entity.

pub use forge_entity::{REST_ACTIONS, RestModel};

#[cfg(test)]
mod bdd_tests {
  use super::*;

  mod re_export_behavior {
    use super::*;

    #[test]
    fn should_re_export_rest_model_trait() {
      // Given: rest_model module
      // When: checking RestModel trait
      // Then: should be accessible (verified by compilation)
      // RestModel trait can be used to implement RestModel for models
    }

    #[test]
    fn should_re_export_rest_actions_constant() {
      // Given: rest_model module
      // When: checking REST_ACTIONS constant
      // Then: should be accessible
      // REST_ACTIONS contains standard CRUD actions
      let _actions = REST_ACTIONS;
    }

    #[test]
    fn should_provide_rest_model_trait_for_implementation() {
      // Given: rest_model module
      // When: implementing RestModel
      // Then: trait should be available
      // This is verified by successful compilation of RestModel implementations
    }
  }
}
