//! Migrations and seeds. Depends on `db` and `app` so seeds can call app handler impls (same logic as API).

use async_trait::async_trait;
use forge_db::DbConnection;
use sea_orm_migration::prelude::{MigrationTrait, MigratorTrait};

pub mod migrations;
pub mod seeds;

pub struct Migrator;

#[async_trait]
impl MigratorTrait for Migrator {
  fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
      Box::new(migrations::m20220101_000001_create_user_table::Migration),
      Box::new(migrations::m20220101_000002_create_organizations_table::Migration),
      Box::new(migrations::m20220101_000003_create_memberships_table::Migration),
      Box::new(migrations::m20220101_000004_create_audit_log_table::Migration),
      Box::new(migrations::m20220101_000005_create_api_tokens_table::Migration),
      Box::new(migrations::m20220101_000006_create_role_permission_table::Migration),
      Box::new(migrations::m20220101_000007_add_org_id_to_role_permission::Migration),
      Box::new(migrations::m20220101_000008_create_org_role_table::Migration),
      Box::new(migrations::m20220101_000009_create_user_org_role_table::Migration),
      Box::new(migrations::m20220101_000010_org_roles_and_user_org_role_data::Migration),
      Box::new(migrations::m20220101_000011_create_user_global_role_table::Migration),
    ]
  }
}

pub async fn run_seeds(db: DbConnection) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  seeds::s20220101_000001_seed_admin::seed(&db).await?;
  seeds::s20220101_000002_seed_default_users::seed(&db).await?;
  seeds::s20220101_000003_seed_coolorg_users::seed(&db).await?;
  seeds::s20220101_000004_seed_multi_org_user::seed(&db).await?;
  Ok(())
}

/// Delegates to db so app handlers and migrations seeds share the same logic.
pub async fn seed_role_permissions_for_org<C: sea_orm::ConnectionTrait>(
  db: &C,
  org_id: uuid::Uuid,
) -> Result<(), sea_orm::DbErr> {
  db::seed_role_permissions_for_org(db, org_id).await
}
