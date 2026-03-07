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

#[cfg(test)]
mod bdd_tests {
  use super::*;
  use sea_orm::{Database, EntityTrait};
  use sea_orm_migration::MigratorTrait;
  use uuid::Uuid;

  async fn test_db() -> forge_db::DbConnection {
    let conn = Database::connect(sea_orm::ConnectOptions::new("sqlite::memory:".to_string()))
      .await
      .unwrap();
    Migrator::up(&conn, None).await.expect("migrate");
    forge_db::wrap_traced(conn)
  }

  mod migrator_behavior {
    use super::*;

    #[test]
    fn should_have_migrations_list() {
      // Given: Migrator
      // When: checking migrations
      // Then: should return list of migrations
      let migrations = Migrator::migrations();
      assert!(!migrations.is_empty());
    }

    #[test]
    fn should_include_user_table_migration() {
      // Given: Migrator
      // When: checking migrations
      // Then: should include create_user_table migration
      let migrations = Migrator::migrations();
      let names: Vec<String> = migrations.iter().map(|m| m.name().to_string()).collect();
      assert!(names.iter().any(|n| n.contains("create_user_table")));
    }

    #[test]
    fn should_include_organizations_table_migration() {
      // Given: Migrator
      // When: checking migrations
      // Then: should include create_organizations_table migration
      let migrations = Migrator::migrations();
      let names: Vec<String> = migrations.iter().map(|m| m.name().to_string()).collect();
      assert!(
        names
          .iter()
          .any(|n| n.contains("create_organizations_table"))
      );
    }

    #[tokio::test]
    async fn should_run_all_migrations_up() {
      // Given: a test database
      let db = test_db().await;

      // When: migrations are run
      // Then: should succeed (verified by test_db helper)
      // Database should have all tables created
      assert!(true); // Success if test_db() completes
    }
  }

  mod run_seeds_behavior {
    use super::*;

    #[tokio::test]
    async fn should_run_all_seeds() {
      // Given: a test database with migrations applied
      let db = test_db().await;

      // When: running seeds
      let result = run_seeds(db).await;

      // Then: should succeed
      assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_create_seed_data() {
      // Given: a test database with migrations applied
      let db = test_db().await;

      // When: running seeds
      run_seeds(db.clone()).await.expect("run seeds");

      // Then: seed data should exist
      // Verify admin user exists
      use crate::models::user;
      let admin_users = user::Entity::find()
        .filter(user::Column::Email.eq("admin@admin.com"))
        .all(&db)
        .await
        .expect("query");
      assert!(!admin_users.is_empty());
    }
  }

  mod seed_role_permissions_for_org_behavior {
    use super::*;

    #[tokio::test]
    async fn should_delegate_to_db_seed_function() {
      // Given: a test database with org and roles
      let db = test_db().await;
      let org_id = Uuid::new_v4();
      let now = chrono::Utc::now().naive_utc();

      use crate::models::{org_role, organization};
      use sea_orm::Set;

      organization::Entity::insert(organization::ActiveModel {
        id: Set(org_id),
        name: Set("Test Org".to_string()),
        slug: Set("test-org".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert org");

      for name in ["owner", "admin", "editor", "viewer"] {
        org_role::Entity::insert(org_role::ActiveModel {
          id: Set(Uuid::new_v4()),
          org_id: Set(org_id),
          name: Set(name.to_string()),
          display_name: Set(None),
          created_at: Set(now),
          updated_at: Set(now),
        })
        .exec(&db)
        .await
        .expect("insert role");
      }

      // When: seeding role permissions via migrations module
      let result = seed_role_permissions_for_org(&db, org_id).await;

      // Then: should succeed
      assert!(result.is_ok());

      // And: permissions should be created
      use crate::models::role_permission;
      let perms = role_permission::Entity::find()
        .filter(role_permission::Column::OrgId.eq(Some(org_id)))
        .all(&db)
        .await
        .expect("query");
      assert!(!perms.is_empty());
    }
  }
}
