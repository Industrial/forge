use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Assigns a global-scope role (e.g. platform_admin) to a user.
/// Permissions for that role are in role_permission with scope=global and org_id=null.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user_global_role")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: Uuid,
  pub user_id: Uuid,
  pub role_name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[cfg(test)]
mod bdd_tests {
  use super::*;
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
    async fn should_create_user_global_role_with_required_fields() {
      // Given: a test database
      let db = test_db().await;
      let id = Uuid::new_v4();
      let user_id = Uuid::new_v4();

      // When: inserting a user_global_role with required fields
      let result = Entity::insert(ActiveModel {
        id: Set(id),
        user_id: Set(user_id),
        role_name: Set("platform_admin".to_string()),
      })
      .exec(&db)
      .await;

      // Then: should succeed
      assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_store_user_global_role_association() {
      // Given: a test database
      let db = test_db().await;
      let id = Uuid::new_v4();
      let user_id = Uuid::new_v4();

      // When: inserting a user_global_role
      Entity::insert(ActiveModel {
        id: Set(id),
        user_id: Set(user_id),
        role_name: Set("platform_admin".to_string()),
      })
      .exec(&db)
      .await
      .expect("insert");

      // Then: should be able to retrieve it
      let association = Entity::find_by_id(id)
        .one(&db)
        .await
        .expect("find")
        .expect("association should exist");
      assert_eq!(association.id, id);
      assert_eq!(association.user_id, user_id);
      assert_eq!(association.role_name, "platform_admin");
    }

    #[tokio::test]
    async fn should_query_user_global_roles_by_user_id() {
      // Given: a test database with user_global_role associations
      let db = test_db().await;
      let user1_id = Uuid::new_v4();
      let user2_id = Uuid::new_v4();

      Entity::insert(ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user1_id),
        role_name: Set("platform_admin".to_string()),
      })
      .exec(&db)
      .await
      .expect("insert assoc1");

      Entity::insert(ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user1_id),
        role_name: Set("platform_viewer".to_string()),
      })
      .exec(&db)
      .await
      .expect("insert assoc2");

      Entity::insert(ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user2_id),
        role_name: Set("platform_admin".to_string()),
      })
      .exec(&db)
      .await
      .expect("insert assoc3");

      // When: querying associations by user1_id
      let user1_assocs = Entity::find()
        .filter(Column::UserId.eq(user1_id))
        .all(&db)
        .await
        .expect("query");

      // Then: should find 2 associations for user1
      assert_eq!(user1_assocs.len(), 2);
      assert!(user1_assocs.iter().all(|a| a.user_id == user1_id));
    }

    #[tokio::test]
    async fn should_query_user_global_roles_by_role_name() {
      // Given: a test database with user_global_role associations
      let db = test_db().await;
      let user1_id = Uuid::new_v4();
      let user2_id = Uuid::new_v4();

      Entity::insert(ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user1_id),
        role_name: Set("platform_admin".to_string()),
      })
      .exec(&db)
      .await
      .expect("insert assoc1");

      Entity::insert(ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user2_id),
        role_name: Set("platform_admin".to_string()),
      })
      .exec(&db)
      .await
      .expect("insert assoc2");

      Entity::insert(ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user1_id),
        role_name: Set("platform_viewer".to_string()),
      })
      .exec(&db)
      .await
      .expect("insert assoc3");

      // When: querying associations by role_name
      let admin_assocs = Entity::find()
        .filter(Column::RoleName.eq("platform_admin"))
        .all(&db)
        .await
        .expect("query");

      // Then: should find 2 associations for platform_admin
      assert_eq!(admin_assocs.len(), 2);
      assert!(admin_assocs.iter().all(|a| a.role_name == "platform_admin"));
    }

    #[tokio::test]
    async fn should_query_user_global_role_by_user_and_role() {
      // Given: a test database with user_global_role associations
      let db = test_db().await;
      let user_id = Uuid::new_v4();

      Entity::insert(ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        role_name: Set("platform_admin".to_string()),
      })
      .exec(&db)
      .await
      .expect("insert");

      // When: querying association by user_id and role_name
      let association = Entity::find()
        .filter(Column::UserId.eq(user_id))
        .filter(Column::RoleName.eq("platform_admin"))
        .one(&db)
        .await
        .expect("query");

      // Then: should find the association
      assert!(association.is_some());
      let a = association.unwrap();
      assert_eq!(a.user_id, user_id);
      assert_eq!(a.role_name, "platform_admin");
    }
  }
}
