use forge_db::DbConnection;

pub mod auth;
pub mod models;

/// Default permission keys per org role name. Used by [seed_role_permissions_for_org] and by migrations seeds.
const ORG_OWNER_ADMIN: &[&str] = &[
  "dashboard",
  "dashboard.organizations.read",
  "dashboard.organizations.write",
  "dashboard.users.read",
  "dashboard.users.write",
  "dashboard.roles.read",
  "dashboard.roles.write",
  "dashboard.permissions.read",
  "dashboard.permissions.write",
  "dashboard.audit.read",
  "all.read",
  "all.write",
];
const ORG_EDITOR: &[&str] = &[
  "dashboard",
  "dashboard.users.read",
  "dashboard.users.write",
  "dashboard.permissions.read",
  "dashboard.permissions.write",
  "dashboard.roles.read",
  "dashboard.roles.write",
];
const ORG_VIEWER: &[&str] = &[
  "dashboard",
  "dashboard.users.read",
  "dashboard.audit.read",
  "dashboard.permissions.read",
  "dashboard.roles.read",
];

/// Inserts default role_permission rows for an org (owner, admin, editor, viewer).
pub async fn seed_role_permissions_for_org<C: sea_orm::ConnectionTrait>(
  db: &C,
  org_id: uuid::Uuid,
) -> Result<(), sea_orm::DbErr> {
  use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};
  let roles = crate::models::org_role::Entity::find()
    .filter(crate::models::org_role::Column::OrgId.eq(org_id))
    .all(db)
    .await?;
  for role in roles {
    let keys: &[&str] = match role.name.as_str() {
      "owner" | "admin" => ORG_OWNER_ADMIN,
      "editor" => ORG_EDITOR,
      "viewer" => ORG_VIEWER,
      _ => continue,
    };
    for key in keys {
      let id = uuid::Uuid::new_v4();
      let row = crate::models::role_permission::ActiveModel {
        id: Set(id),
        scope: Set("org".to_string()),
        role_name: Set(role.name.clone()),
        permission_key: Set((*key).to_string()),
        org_id: Set(Some(org_id)),
        ..Default::default()
      };
      crate::models::role_permission::Entity::insert(row)
        .exec(db)
        .await?;
    }
  }
  Ok(())
}

/// Look up user id by raw API token (Bearer). Returns None if token invalid or expired.
pub async fn token_lookup(db: DbConnection, raw_token: String) -> Option<uuid::Uuid> {
  use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
  let hash = forge_auth::token_auth::hash_api_token(&raw_token);
  let row = crate::models::api_token::Entity::find()
    .filter(crate::models::api_token::Column::TokenHash.eq(hash))
    .one(&db)
    .await
    .ok()
    .flatten()?;
  if let Some(exp) = row.expires_at {
    if exp < chrono::Utc::now().naive_utc() {
      return None;
    }
  }
  Some(row.user_id)
}
