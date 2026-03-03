use forge_auth::{AuthzContext, Role};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: Uuid,
  #[sea_orm(unique)]
  pub email: String,
  pub password_hash: String,
  pub is_active: bool,
  pub is_admin: bool,
  /// Deprecated for authz: session profile (current_org_id, current_role_name) is the source of truth. Kept for seeds/display.
  pub current_org_id: Option<Uuid>,
  /// Deprecated for authz: use session profile. Kept for seeds/display.
  pub current_role: Option<String>,
  pub created_at: chrono::NaiveDateTime,
  pub updated_at: chrono::NaiveDateTime,
}

impl axum_login::AuthUser for Model {
  type Id = Uuid;
  fn id(&self) -> Self::Id {
    self.id
  }
  fn session_auth_hash(&self) -> &[u8] {
    self.password_hash.as_bytes()
  }
}

impl AuthzContext for Model {
  type RequesterId = Uuid;
  type SubjectId = Uuid;

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
