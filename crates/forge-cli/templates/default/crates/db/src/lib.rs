use async_trait::async_trait;
use forge::DbConnection;
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
      Box::new(migrations::m20220101_000002_create_sessions_table::Migration),
      Box::new(migrations::m20220101_000003_create_organizations_table::Migration),
      Box::new(migrations::m20220101_000004_create_memberships_table::Migration),
      Box::new(migrations::m20220101_000005_create_audit_log_table::Migration),
      Box::new(migrations::m20220101_000006_create_api_tokens_table::Migration),
      Box::new(migrations::m20220101_000007_create_role_permission_table::Migration),
    ]
  }
}

pub async fn run_seeds(db: DbConnection) -> Result<(), Box<dyn std::error::Error>> {
  seeds::s20220101_000001_seed_users::seed(&db).await?;
  seeds::s20220101_000002_seed_role_permissions::seed(&db).await?;
  Ok(())
}

/// Look up user id by raw API token (Bearer). Returns None if token invalid or expired.
pub async fn token_lookup(db: DbConnection, raw_token: String) -> Option<uuid::Uuid> {
  use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
  let hash = forge::auth::hash_api_token(&raw_token);
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
  fn migrator_returns_seven_migrations() {
    let migrations = Migrator::migrations();
    assert_eq!(migrations.len(), 7);
  }
}
