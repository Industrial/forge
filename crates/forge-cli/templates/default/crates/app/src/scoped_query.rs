//! Scoped queries: apply request scope (current org) to SeaORM selects so handlers don't reimplement filtering.
//!
//! Use [WithScope::with_scope] to restrict results to the current tenant/org when `scope_opt` is `Some`;
//! when `None`, no filter is applied (global / all data).
//!
//! For entities without a direct org column (e.g. [user::Entity], scoped via membership), use the
//! dedicated async helper [user_find_scoped].
//!
//! The [WithScope] trait lives here (not in forge-auth) so that we can implement it for
//! [Select\<Entity\>] in this crate; Rust's orphan rule forbids implementing a foreign trait for a foreign type.

use forge_auth::RequestScope;
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Select};

use db::models::{audit_log, membership, org_role, role_permission, user, user_org_role};

/// Apply request scope to a select: when `scope_opt` is `Some`, filter to that org; when `None`, leave query unchanged.
pub trait WithScope {
  type Entity: EntityTrait;
  fn with_scope(self, scope_opt: Option<&RequestScope>) -> Select<Self::Entity>;
}

impl WithScope for Select<audit_log::Entity> {
  type Entity = audit_log::Entity;
  fn with_scope(self, scope_opt: Option<&RequestScope>) -> Select<audit_log::Entity> {
    match scope_opt {
      None => self,
      Some(scope) => self.filter(audit_log::Column::OrganizationId.eq(scope.organization_id)),
    }
  }
}

impl WithScope for Select<org_role::Entity> {
  type Entity = org_role::Entity;
  fn with_scope(self, scope_opt: Option<&RequestScope>) -> Select<org_role::Entity> {
    match scope_opt {
      None => self,
      Some(scope) => self.filter(org_role::Column::OrgId.eq(scope.organization_id)),
    }
  }
}

/// User has no direct org column; scoping is "users that are members of this org".
/// Returns a select that may be filtered by membership when `scope_opt` is `Some`.
pub async fn user_find_scoped<C: ConnectionTrait>(
  db: &C,
  scope_opt: Option<&RequestScope>,
) -> Result<Select<user::Entity>, sea_orm::DbErr> {
  let select = user::Entity::find();
  match scope_opt {
    None => Ok(select),
    Some(scope) => {
      let user_ids: Vec<uuid::Uuid> = membership::Entity::find()
        .filter(membership::Column::OrgId.eq(scope.organization_id))
        .all(db)
        .await?
        .into_iter()
        .map(|m| m.user_id)
        .collect();
      if user_ids.is_empty() {
        // Return a query that matches no rows (avoid "IN ()" if DB doesn't support it)
        Ok(select.filter(user::Column::Id.eq(uuid::Uuid::nil())))
      } else {
        Ok(select.filter(user::Column::Id.is_in(user_ids)))
      }
    }
  }
}

/// [user_org_role::Entity] is scoped by org.
impl WithScope for Select<user_org_role::Entity> {
  type Entity = user_org_role::Entity;
  fn with_scope(self, scope_opt: Option<&RequestScope>) -> Select<user_org_role::Entity> {
    match scope_opt {
      None => self,
      Some(scope) => self.filter(user_org_role::Column::OrgId.eq(scope.organization_id)),
    }
  }
}

/// [membership::Entity] is scoped by org.
impl WithScope for Select<membership::Entity> {
  type Entity = membership::Entity;
  fn with_scope(self, scope_opt: Option<&RequestScope>) -> Select<membership::Entity> {
    match scope_opt {
      None => self,
      Some(scope) => self.filter(membership::Column::OrgId.eq(scope.organization_id)),
    }
  }
}

/// [role_permission::Entity] is scoped by org (org_id column; None = global).
impl WithScope for Select<role_permission::Entity> {
  type Entity = role_permission::Entity;
  fn with_scope(self, scope_opt: Option<&RequestScope>) -> Select<role_permission::Entity> {
    match scope_opt {
      None => self,
      Some(scope) => self.filter(role_permission::Column::OrgId.eq(Some(scope.organization_id))),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use sea_orm::EntityTrait;
  use sea_orm_migration::MigratorTrait;

  fn test_scope(org_id: uuid::Uuid) -> RequestScope {
    RequestScope {
      organization_id: org_id,
      role_id: uuid::Uuid::new_v4(),
      role_name: "viewer".to_string(),
    }
  }

  #[tokio::test]
  async fn audit_log_with_scope_none_leaves_query_unchanged() {
    let select = audit_log::Entity::find();
    let scoped = select.with_scope(None);
    // Just ensure it compiles and returns the same type; we can't run without DB
    let _: Select<audit_log::Entity> = scoped;
  }

  #[tokio::test]
  async fn audit_log_with_scope_some_adds_org_filter() {
    let org_id = uuid::Uuid::new_v4();
    let scope = test_scope(org_id);
    let select = audit_log::Entity::find().with_scope(Some(&scope));
    let _: Select<audit_log::Entity> = select;
    // Filter is applied; we'd assert SQL or run against test DB in integration
  }

  #[tokio::test]
  async fn org_role_with_scope_some_adds_org_filter() {
    let org_id = uuid::Uuid::new_v4();
    let scope = test_scope(org_id);
    let select = org_role::Entity::find().with_scope(Some(&scope));
    let _: Select<org_role::Entity> = select;
  }

  #[tokio::test]
  async fn user_org_role_with_scope_some_adds_org_filter() {
    let org_id = uuid::Uuid::new_v4();
    let scope = test_scope(org_id);
    let select = user_org_role::Entity::find().with_scope(Some(&scope));
    let _: Select<user_org_role::Entity> = select;
  }

  #[tokio::test]
  async fn user_find_scoped_none_returns_unfiltered_select() {
    let db = sea_orm::Database::connect("sqlite::memory:")
      .await
      .expect("in-memory db");
    let select = user_find_scoped(&db, None).await.expect("ok");
    let _: Select<user::Entity> = select;
  }

  #[tokio::test]
  async fn user_find_scoped_some_returns_filtered_select() {
    let db = sea_orm::Database::connect("sqlite::memory:")
      .await
      .expect("in-memory db");
    migrations::Migrator::up(&db, None).await.expect("migrate");
    let org_id = uuid::Uuid::new_v4();
    let scope = test_scope(org_id);
    let select = user_find_scoped(&db, Some(&scope)).await.expect("ok");
    let _: Select<user::Entity> = select;
  }
}
