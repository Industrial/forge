use forge::authz::{AuthzContext, Role};
use forge::ForgeAuthUser;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, ForgeAuthUser)]
#[sea_orm(table_name = "user")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: Uuid,
  #[sea_orm(unique)]
  pub email: String,
  pub password_hash: String,
  pub is_active: bool,
  pub is_admin: bool,
  pub current_org_id: Option<Uuid>,
  pub current_role: Option<String>,
  pub created_at: chrono::NaiveDateTime,
  pub updated_at: chrono::NaiveDateTime,
}

impl AuthzContext for Model {
  fn requester_id(&self) -> Uuid {
    self.id
  }
  fn subject_id(&self) -> Uuid {
    self.id
  }
  fn organization_id(&self) -> Option<Uuid> {
    self.current_org_id
  }
  fn role(&self) -> Option<Role> {
    self.current_role.as_deref().and_then(|s| match s {
      "owner" => Some(Role::Owner),
      "admin" => Some(Role::Admin),
      "editor" => Some(Role::Editor),
      "viewer" => Some(Role::Viewer),
      _ => None,
    })
  }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
