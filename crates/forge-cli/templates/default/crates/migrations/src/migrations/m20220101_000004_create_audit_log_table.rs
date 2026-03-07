use sea_orm_migration::prelude::*;

/// Audit log table. Table and column names must match forge_audit::log raw SQL (snake_case).
#[derive(Iden)]
pub enum AuditLog {
  #[iden = "audit_log"]
  Table,
  Id,
  #[iden = "event_kind"]
  EventKind,
  #[iden = "actor_id"]
  ActorId,
  #[iden = "subject_id"]
  SubjectId,
  #[iden = "organization_id"]
  OrganizationId,
  Action,
  #[iden = "resource_type"]
  ResourceType,
  #[iden = "resource_id"]
  ResourceId,
  Outcome,
  Reason,
  #[iden = "occurred_at"]
  OccurredAt,
}

pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m20220101_000004_create_audit_log_table"
  }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(AuditLog::Table)
          .if_not_exists()
          .col(ColumnDef::new(AuditLog::Id).uuid().not_null().primary_key())
          .col(ColumnDef::new(AuditLog::EventKind).string().not_null())
          .col(ColumnDef::new(AuditLog::ActorId).uuid().not_null())
          .col(ColumnDef::new(AuditLog::SubjectId).uuid())
          .col(ColumnDef::new(AuditLog::OrganizationId).uuid())
          .col(ColumnDef::new(AuditLog::Action).string().not_null())
          .col(ColumnDef::new(AuditLog::ResourceType).string().not_null())
          .col(ColumnDef::new(AuditLog::ResourceId).uuid())
          .col(ColumnDef::new(AuditLog::Outcome).string().not_null())
          .col(ColumnDef::new(AuditLog::Reason).string())
          .col(ColumnDef::new(AuditLog::OccurredAt).date_time().not_null())
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_audit_log_org_occurred")
          .table(AuditLog::Table)
          .col(AuditLog::OrganizationId)
          .col(AuditLog::OccurredAt)
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_audit_log_actor_occurred")
          .table(AuditLog::Table)
          .col(AuditLog::ActorId)
          .col(AuditLog::OccurredAt)
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(AuditLog::Table).to_owned())
      .await
  }
}

#[cfg(test)]
mod bdd_tests {
  use super::*;
  use sea_orm::{ConnectionTrait, Database, EntityTrait};
  use sea_orm_migration::prelude::*;

  async fn test_db() -> sea_orm::DatabaseConnection {
    Database::connect(sea_orm::ConnectOptions::new("sqlite::memory:".to_string()))
      .await
      .unwrap()
  }

  mod migration_up_behavior {
    use super::*;

    #[tokio::test]
    async fn should_create_audit_log_table() {
      // Given: a test database
      let db = test_db().await;
      let migration = Migration;

      // When: running migration up
      let result = migration.up(&SchemaManager::new(&db)).await;

      // Then: should succeed
      assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_create_table_with_all_columns() {
      // Given: a test database
      let db = test_db().await;
      let migration = Migration;
      migration
        .up(&SchemaManager::new(&db))
        .await
        .expect("migrate");

      // When: checking table structure
      // Then: table should exist with all columns
      use db::models::audit_log;
      let logs = audit_log::Entity::find().all(&db).await;
      assert!(logs.is_ok());
    }

    #[tokio::test]
    async fn should_create_table_with_primary_key() {
      // Given: a test database
      let db = test_db().await;
      let migration = Migration;
      migration
        .up(&SchemaManager::new(&db))
        .await
        .expect("migrate");

      // When: inserting audit log with id
      // Then: should succeed
      use chrono::Utc;
      use db::models::audit_log;
      use sea_orm::Set;
      use uuid::Uuid;

      let result = audit_log::Entity::insert(audit_log::ActiveModel {
        id: Set(Uuid::new_v4()),
        event_kind: Set("test".to_string()),
        actor_id: Set(Uuid::new_v4()),
        subject_id: Set(None),
        organization_id: Set(None),
        action: Set("test_action".to_string()),
        resource_type: Set("test_resource".to_string()),
        resource_id: Set(None),
        outcome: Set("success".to_string()),
        reason: Set(None),
        occurred_at: Set(Utc::now().naive_utc()),
      })
      .exec(&db)
      .await;

      assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_create_indexes() {
      // Given: a test database
      let db = test_db().await;
      let migration = Migration;

      // When: running migration up
      let result = migration.up(&SchemaManager::new(&db)).await;

      // Then: should succeed (indexes created)
      assert!(result.is_ok());
      // Indexes: idx_audit_log_org_occurred, idx_audit_log_actor_occurred
    }
  }

  mod migration_down_behavior {
    use super::*;

    #[tokio::test]
    async fn should_drop_audit_log_table() {
      // Given: a test database with audit_log table created
      let db = test_db().await;
      let migration = Migration;
      migration
        .up(&SchemaManager::new(&db))
        .await
        .expect("migrate up");

      // When: running migration down
      let result = migration.down(&SchemaManager::new(&db)).await;

      // Then: should succeed
      assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_remove_table_after_down() {
      // Given: a test database with audit_log table
      let db = test_db().await;
      let migration = Migration;
      migration
        .up(&SchemaManager::new(&db))
        .await
        .expect("migrate up");
      migration
        .down(&SchemaManager::new(&db))
        .await
        .expect("migrate down");

      // When: trying to query audit_log table
      // Then: should fail (table doesn't exist)
      use db::models::audit_log;
      let result = audit_log::Entity::find().all(&db).await;
      assert!(result.is_err());
    }
  }

  mod migration_name_behavior {
    use super::*;

    #[test]
    fn should_have_correct_migration_name() {
      // Given: Migration struct
      let migration = Migration;

      // When: getting migration name
      let name = migration.name();

      // Then: should match expected name
      assert_eq!(name, "m20220101_000004_create_audit_log_table");
    }
  }
}
