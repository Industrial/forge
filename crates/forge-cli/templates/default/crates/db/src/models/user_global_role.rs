use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Assigns a global-scope role (e.g. platform_admin) to a user.
/// Permissions for that role are in role_permission with scope=global and org_id=null.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user_global_role")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: Uuid,
  pub user_id: Uuid,
  pub role_name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
