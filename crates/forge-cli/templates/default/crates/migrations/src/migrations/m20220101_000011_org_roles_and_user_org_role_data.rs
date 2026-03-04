use sea_orm::*;
use sea_orm_migration::prelude::*;

use db::models::{org_role, organization, user_org_role};

const TEMPLATE_ROLES: &[&str] = &["owner", "admin", "editor", "viewer"];

#[derive(Iden)]
enum Membership {
  Table,
  Role,
}

pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m20220101_000011_org_roles_and_user_org_role_data"
  }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    let db = manager.get_connection();
    let backend = manager.get_database_backend();
    let now = chrono::Utc::now().naive_utc();

    // 1. For each organization, create the four template org_roles.
    let orgs = organization::Entity::find().all(db).await?;
    for org in orgs {
      for &name in TEMPLATE_ROLES {
        let id = uuid::Uuid::new_v4();
        let model = org_role::ActiveModel {
          id: Set(id),
          org_id: Set(org.id),
          name: Set(name.to_string()),
          display_name: Set(None),
          created_at: Set(now),
          updated_at: Set(now),
          ..Default::default()
        };
        org_role::Entity::insert(model).exec(db).await?;
      }
    }

    // 2. Read memberships with raw SQL (table still has role column).
    let stmt = Statement::from_string(
      backend,
      "SELECT user_id, org_id, role FROM membership".to_string(),
    );
    let rows = db.query_all(stmt).await?;

    for row in rows {
      let user_id: uuid::Uuid = row.try_get("", "user_id")?;
      let org_id: uuid::Uuid = row.try_get("", "org_id")?;
      let role: String = row.try_get("", "role")?;

      let role_model = org_role::Entity::find()
        .filter(org_role::Column::OrgId.eq(org_id))
        .filter(org_role::Column::Name.eq(&role))
        .one(db)
        .await?;
      if let Some(role_row) = role_model {
        let id = uuid::Uuid::new_v4();
        let model = user_org_role::ActiveModel {
          id: Set(id),
          user_id: Set(user_id),
          org_id: Set(org_id),
          role_id: Set(role_row.id),
          created_at: Set(now),
          updated_at: Set(now),
          ..Default::default()
        };
        user_org_role::Entity::insert(model).exec(db).await?;
      }
    }

    // 3. Drop the role column from membership.
    manager
      .alter_table(
        Table::alter()
          .table(Membership::Table)
          .drop_column(Membership::Role)
          .to_owned(),
      )
      .await
  }

  async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
    // Restoring the role column and data would require repopulating from user_org_role;
    // we do not implement full down for this data migration.
    Ok(())
  }
}
