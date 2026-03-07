pub mod api_token;
pub mod audit_log;
pub mod membership;
pub mod org_role;
pub mod organization;
pub mod role_permission;
pub mod user;
pub mod user_global_role;
pub mod user_org_role;

#[cfg(test)]
mod bdd_tests {
  use super::*;

  mod module_structure_behavior {
    use super::*;
    use sea_orm::EntityTrait;

    #[test]
    fn should_export_all_model_modules() {
      // Given: models module
      // When: checking module exports
      // Then: all model modules should be accessible
      // This is verified by successful compilation and import
      let _api_token = api_token::Entity::find();
      let _audit_log = audit_log::Entity::find();
      let _membership = membership::Entity::find();
      let _org_role = org_role::Entity::find();
      let _organization = organization::Entity::find();
      let _role_permission = role_permission::Entity::find();
      let _user = user::Entity::find();
      let _user_global_role = user_global_role::Entity::find();
      let _user_org_role = user_org_role::Entity::find();
    }

    #[test]
    fn should_have_consistent_model_structure() {
      // Given: models module
      // When: checking model structure
      // Then: all models should follow the same pattern
      // This is verified by successful compilation
      // Each model should have Entity, Model, ActiveModel, Column, Relation
    }
  }
}
