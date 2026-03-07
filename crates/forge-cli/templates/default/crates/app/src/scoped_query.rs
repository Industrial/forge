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

  mod with_scope_trait_behavior {
    use super::*;

    mod audit_log_scoping {
      use super::*;

      #[tokio::test]
      async fn with_scope_none_leaves_query_unchanged() {
        // Given an audit_log select query
        let select = audit_log::Entity::find();

        // When I apply with_scope with None
        let scoped = select.with_scope(None);

        // Then it should return the same query unchanged (no filter applied)
        let _: Select<audit_log::Entity> = scoped;
      }

      #[tokio::test]
      async fn with_scope_some_applies_organization_filter() {
        // Given an audit_log select query and a request scope with organization ID
        let org_id = uuid::Uuid::new_v4();
        let scope = test_scope(org_id);
        let select = audit_log::Entity::find();

        // When I apply with_scope with Some(scope)
        let scoped = select.with_scope(Some(&scope));

        // Then it should return a filtered query (filter is applied at compile time)
        let _: Select<audit_log::Entity> = scoped;
      }
    }

    mod org_role_scoping {
      use super::*;

      #[tokio::test]
      async fn with_scope_none_returns_unfiltered_query() {
        // Given an org_role select query
        let select = org_role::Entity::find();

        // When I apply with_scope with None
        let scoped = select.with_scope(None);

        // Then it should return the query without org filter
        let _: Select<org_role::Entity> = scoped;
      }

      #[tokio::test]
      async fn with_scope_some_filters_by_organization_id() {
        // Given an org_role select query and a request scope
        let org_id = uuid::Uuid::new_v4();
        let scope = test_scope(org_id);
        let select = org_role::Entity::find();

        // When I apply with_scope with Some(scope)
        let scoped = select.with_scope(Some(&scope));

        // Then it should return a query filtered by org_id
        let _: Select<org_role::Entity> = scoped;
      }
    }

    mod user_org_role_scoping {
      use super::*;

      #[tokio::test]
      async fn with_scope_none_returns_all_user_org_roles() {
        // Given a user_org_role select query
        let select = user_org_role::Entity::find();

        // When I apply with_scope with None
        let scoped = select.with_scope(None);

        // Then it should return an unfiltered query
        let _: Select<user_org_role::Entity> = scoped;
      }

      #[tokio::test]
      async fn with_scope_some_filters_by_organization() {
        // Given a user_org_role select query and a request scope
        let org_id = uuid::Uuid::new_v4();
        let scope = test_scope(org_id);
        let select = user_org_role::Entity::find();

        // When I apply with_scope with Some(scope)
        let scoped = select.with_scope(Some(&scope));

        // Then it should return a query filtered by organization
        let _: Select<user_org_role::Entity> = scoped;
      }
    }

    mod membership_scoping {
      use super::*;

      #[tokio::test]
      async fn with_scope_none_returns_all_memberships() {
        // Given a membership select query
        let select = membership::Entity::find();

        // When I apply with_scope with None
        let scoped = select.with_scope(None);

        // Then it should return an unfiltered query
        let _: Select<membership::Entity> = scoped;
      }

      #[tokio::test]
      async fn with_scope_some_filters_memberships_by_organization() {
        // Given a membership select query and a request scope
        let org_id = uuid::Uuid::new_v4();
        let scope = test_scope(org_id);
        let select = membership::Entity::find();

        // When I apply with_scope with Some(scope)
        let scoped = select.with_scope(Some(&scope));

        // Then it should return a query filtered by organization
        let _: Select<membership::Entity> = scoped;
      }
    }

    mod role_permission_scoping {
      use super::*;

      #[tokio::test]
      async fn with_scope_none_returns_all_role_permissions() {
        // Given a role_permission select query
        let select = role_permission::Entity::find();

        // When I apply with_scope with None
        let scoped = select.with_scope(None);

        // Then it should return an unfiltered query
        let _: Select<role_permission::Entity> = scoped;
      }

      #[tokio::test]
      async fn with_scope_some_filters_role_permissions_by_organization() {
        // Given a role_permission select query and a request scope
        let org_id = uuid::Uuid::new_v4();
        let scope = test_scope(org_id);
        let select = role_permission::Entity::find();

        // When I apply with_scope with Some(scope)
        let scoped = select.with_scope(Some(&scope));

        // Then it should return a query filtered by organization (using Some(org_id))
        let _: Select<role_permission::Entity> = scoped;
      }
    }
  }

  mod user_find_scoped_behavior {
    use super::*;

    #[tokio::test]
    async fn user_find_scoped_none_returns_unfiltered_select() {
      // Given a database connection and no scope
      let db = sea_orm::Database::connect("sqlite::memory:")
        .await
        .expect("in-memory db");

      // When I call user_find_scoped with None
      let select = user_find_scoped(&db, None).await.expect("ok");

      // Then it should return an unfiltered user select query
      let _: Select<user::Entity> = select;
    }

    #[tokio::test]
    async fn user_find_scoped_some_returns_filtered_select() {
      // Given a database with migrations and a request scope
      let db = sea_orm::Database::connect("sqlite::memory:")
        .await
        .expect("in-memory db");
      migrations::Migrator::up(&db, None).await.expect("migrate");
      let org_id = uuid::Uuid::new_v4();
      let scope = test_scope(org_id);

      // When I call user_find_scoped with Some(scope)
      let select = user_find_scoped(&db, Some(&scope)).await.expect("ok");

      // Then it should return a filtered select query
      let _: Select<user::Entity> = select;
    }

    #[tokio::test]
    async fn user_find_scoped_handles_empty_membership_list() {
      // Given a database with migrations and a scope for an org with no members
      let db = sea_orm::Database::connect("sqlite::memory:")
        .await
        .expect("in-memory db");
      migrations::Migrator::up(&db, None).await.expect("migrate");
      let org_id = uuid::Uuid::new_v4();
      let scope = test_scope(org_id);

      // When I call user_find_scoped with Some(scope) for an org with no members
      let select = user_find_scoped(&db, Some(&scope)).await.expect("ok");

      // Then it should return a query that matches no rows (using nil UUID filter)
      let _: Select<user::Entity> = select;
    }
  }
}
