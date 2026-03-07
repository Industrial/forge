use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "membership")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: Uuid,
  pub user_id: Uuid,
  pub org_id: Uuid,
  pub created_at: chrono::NaiveDateTime,
  pub updated_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[cfg(test)]
mod bdd_tests {
  use super::*;
  use chrono::Utc;
  use sea_orm::{Database, EntityTrait, Set};
  use sea_orm_migration::MigratorTrait;
  use uuid::Uuid;

  async fn test_db() -> forge_db::DbConnection {
    let conn = Database::connect(sea_orm::ConnectOptions::new("sqlite::memory:".to_string()))
      .await
      .unwrap();
    migrations::Migrator::up(&conn, None)
      .await
      .expect("migrate");
    forge_db::wrap_traced(conn)
  }

  mod model_structure_behavior {
    use super::*;

    #[tokio::test]
    async fn should_create_membership_with_required_fields() {
      // Given: a test database
      let db = test_db().await;
      let membership_id = Uuid::new_v4();
      let user_id = Uuid::new_v4();
      let org_id = Uuid::new_v4();
      let now = Utc::now().naive_utc();

      // When: inserting a membership with required fields
      let result = Entity::insert(ActiveModel {
        id: Set(membership_id),
        user_id: Set(user_id),
        org_id: Set(org_id),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await;

      // Then: should succeed
      assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_store_membership_with_user_and_org() {
      // Given: a test database
      let db = test_db().await;
      let membership_id = Uuid::new_v4();
      let user_id = Uuid::new_v4();
      let org_id = Uuid::new_v4();
      let now = Utc::now().naive_utc();

      // When: inserting a membership
      Entity::insert(ActiveModel {
        id: Set(membership_id),
        user_id: Set(user_id),
        org_id: Set(org_id),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert");

      // Then: should be able to retrieve it
      let membership = Entity::find_by_id(membership_id)
        .one(&db)
        .await
        .expect("find")
        .expect("membership should exist");
      assert_eq!(membership.id, membership_id);
      assert_eq!(membership.user_id, user_id);
      assert_eq!(membership.org_id, org_id);
    }

    #[tokio::test]
    async fn should_query_memberships_by_user_id() {
      // Given: a test database with multiple memberships
      let db = test_db().await;
      let user1_id = Uuid::new_v4();
      let user2_id = Uuid::new_v4();
      let org1_id = Uuid::new_v4();
      let org2_id = Uuid::new_v4();
      let now = Utc::now().naive_utc();

      Entity::insert(ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user1_id),
        org_id: Set(org1_id),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert membership1");

      Entity::insert(ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user1_id),
        org_id: Set(org2_id),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert membership2");

      Entity::insert(ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user2_id),
        org_id: Set(org1_id),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert membership3");

      // When: querying memberships by user1_id
      let user1_memberships = Entity::find()
        .filter(Column::UserId.eq(user1_id))
        .all(&db)
        .await
        .expect("query");

      // Then: should find 2 memberships for user1
      assert_eq!(user1_memberships.len(), 2);
      assert!(user1_memberships.iter().all(|m| m.user_id == user1_id));
    }

    #[tokio::test]
    async fn should_query_memberships_by_org_id() {
      // Given: a test database with multiple memberships
      let db = test_db().await;
      let user1_id = Uuid::new_v4();
      let user2_id = Uuid::new_v4();
      let org1_id = Uuid::new_v4();
      let org2_id = Uuid::new_v4();
      let now = Utc::now().naive_utc();

      Entity::insert(ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user1_id),
        org_id: Set(org1_id),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert membership1");

      Entity::insert(ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user2_id),
        org_id: Set(org1_id),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert membership2");

      Entity::insert(ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user1_id),
        org_id: Set(org2_id),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert membership3");

      // When: querying memberships by org1_id
      let org1_memberships = Entity::find()
        .filter(Column::OrgId.eq(org1_id))
        .all(&db)
        .await
        .expect("query");

      // Then: should find 2 memberships for org1
      assert_eq!(org1_memberships.len(), 2);
      assert!(org1_memberships.iter().all(|m| m.org_id == org1_id));
    }

    #[tokio::test]
    async fn should_query_membership_by_user_and_org() {
      // Given: a test database with memberships
      let db = test_db().await;
      let user_id = Uuid::new_v4();
      let org_id = Uuid::new_v4();
      let now = Utc::now().naive_utc();

      Entity::insert(ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        org_id: Set(org_id),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert");

      // When: querying membership by user_id and org_id
      let membership = Entity::find()
        .filter(Column::UserId.eq(user_id))
        .filter(Column::OrgId.eq(org_id))
        .one(&db)
        .await
        .expect("query");

      // Then: should find the membership
      assert!(membership.is_some());
      let m = membership.unwrap();
      assert_eq!(m.user_id, user_id);
      assert_eq!(m.org_id, org_id);
    }
  }
}
