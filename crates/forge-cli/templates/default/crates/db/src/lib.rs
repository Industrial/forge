use async_trait::async_trait;
use forge_db::DbConnection;
use sea_orm_migration::prelude::{MigrationTrait, MigratorTrait};

pub mod auth;
pub mod migrations;
pub mod models;
pub mod seeds;

pub struct Migrator;

#[async_trait]
impl MigratorTrait for Migrator {
  fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
      Box::new(migrations::m20220101_000001_create_user_table::Migration),
      Box::new(migrations::m20220101_000003_create_organizations_table::Migration),
      Box::new(migrations::m20220101_000004_create_memberships_table::Migration),
      Box::new(migrations::m20220101_000005_create_audit_log_table::Migration),
      Box::new(migrations::m20220101_000006_create_api_tokens_table::Migration),
      Box::new(migrations::m20220101_000007_create_role_permission_table::Migration),
      Box::new(migrations::m20220101_000008_add_org_id_to_role_permission::Migration),
      Box::new(migrations::m20220101_000009_create_org_role_table::Migration),
      Box::new(migrations::m20220101_000010_create_user_org_role_table::Migration),
      Box::new(migrations::m20220101_000011_org_roles_and_user_org_role_data::Migration),
      Box::new(migrations::m20220101_000012_create_user_global_role_table::Migration),
    ]
  }
}

pub async fn run_seeds(db: DbConnection) -> Result<(), Box<dyn std::error::Error>> {
  seeds::s20220101_000001_seed_admin::seed(&db).await?;
  seeds::s20220101_000002_seed_default_users::seed(&db).await?;
  seeds::s20220101_000003_seed_coolorg_users::seed(&db).await?;
  seeds::s20220101_000004_seed_multi_org_user::seed(&db).await?;
  Ok(())
}

/// Default permission keys per org role name. Used by [seed_role_permissions_for_org] and seeds.
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
/// Call after creating org and org_roles. Works with a connection or transaction.
pub async fn seed_role_permissions_for_org<C: sea_orm::ConnectionTrait>(
  db: &C,
  org_id: uuid::Uuid,
) -> Result<(), sea_orm::DbErr> {
  use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};
  let roles = models::org_role::Entity::find()
    .filter(models::org_role::Column::OrgId.eq(org_id))
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
      let row = models::role_permission::ActiveModel {
        id: Set(id),
        scope: Set("org".to_string()),
        role_name: Set(role.name.clone()),
        permission_key: Set((*key).to_string()),
        org_id: Set(Some(org_id)),
        ..Default::default()
      };
      models::role_permission::Entity::insert(row).exec(db).await?;
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn migrator_returns_eleven_migrations() {
    let migrations = Migrator::migrations();
    assert_eq!(migrations.len(), 11);
  }
}
