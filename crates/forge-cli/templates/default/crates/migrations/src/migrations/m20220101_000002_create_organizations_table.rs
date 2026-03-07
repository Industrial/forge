use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum Organization {
  Table,
  Id,
  Name,
  Slug,
  CreatedAt,
  UpdatedAt,
}

pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m20220101_000002_create_organizations_table"
  }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(Alias::new("organization"))
          .if_not_exists()
          .col(
            ColumnDef::new(Organization::Id)
              .uuid()
              .not_null()
              .primary_key(),
          )
          .col(ColumnDef::new(Organization::Name).string().not_null())
          .col(
            ColumnDef::new(Organization::Slug)
              .string()
              .unique_key()
              .not_null(),
          )
          .col(
            ColumnDef::new(Organization::CreatedAt)
              .date_time()
              .not_null(),
          )
          .col(
            ColumnDef::new(Organization::UpdatedAt)
              .date_time()
              .not_null(),
          )
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(Alias::new("organization")).to_owned())
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
    async fn should_create_organization_table() {
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
      use db::models::organization;
      let orgs = organization::Entity::find().all(&db).await;
      assert!(orgs.is_ok());
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

      // When: inserting organization with id
      // Then: should succeed
      use chrono::Utc;
      use db::models::organization;
      use sea_orm::Set;
      use uuid::Uuid;

      let result = organization::Entity::insert(organization::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set("Test Org".to_string()),
        slug: Set("test-org".to_string()),
        created_at: Set(Utc::now().naive_utc()),
        updated_at: Set(Utc::now().naive_utc()),
      })
      .exec(&db)
      .await;

      assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_enforce_unique_slug() {
      // Given: a test database with organization table
      let db = test_db().await;
      let migration = Migration;
      migration
        .up(&SchemaManager::new(&db))
        .await
        .expect("migrate");

      use chrono::Utc;
      use db::models::organization;
      use sea_orm::Set;
      use uuid::Uuid;

      let slug = "unique-slug".to_string();
      let now = Utc::now().naive_utc();

      // When: inserting organization with slug
      organization::Entity::insert(organization::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set("Org 1".to_string()),
        slug: Set(slug.clone()),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert first");

      // Then: inserting duplicate slug should fail
      let result = organization::Entity::insert(organization::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set("Org 2".to_string()),
        slug: Set(slug),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await;

      assert!(result.is_err());
    }
  }

  mod migration_down_behavior {
    use super::*;

    #[tokio::test]
    async fn should_drop_organization_table() {
      // Given: a test database with organization table created
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
      // Given: a test database with organization table
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

      // When: trying to query organization table
      // Then: should fail (table doesn't exist)
      use db::models::organization;
      let result = organization::Entity::find().all(&db).await;
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
      assert_eq!(name, "m20220101_000002_create_organizations_table");
    }
  }
}
