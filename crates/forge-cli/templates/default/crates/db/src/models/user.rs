use crate::entity_metadata::{EntityMetadata, ACTIONS};
use forge_auth::AuthzContext;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Allowed filter/sort columns. Response columns (allow-list) exclude password_hash; defined below.
const FILTER_SORT_COLUMNS: &[&str] = &["id", "email", "is_active", "created_at", "updated_at"];
/// Columns returned for list/get (allow-list; never include password_hash).
const RESPONSE_COLUMNS: &[&str] = &["id", "email", "is_active", "created_at", "updated_at"];

/// Entity metadata: API id, actions, display name, filter/sort/response columns. Defined alongside the model (Option A).
pub const ENTITY_METADATA: EntityMetadata = EntityMetadata {
  id: "user",
  supported_actions: ACTIONS,
  display_name: Some("User"),
  allowed_filter_fields: Some(FILTER_SORT_COLUMNS),
  allowed_sort_fields: Some(FILTER_SORT_COLUMNS),
  response_columns_allow: Some(RESPONSE_COLUMNS),
  response_columns_exclude: None,
};

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
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
