//! Create a new Forge project (scaffold).

use std::fs;
use std::path::Path;
use std::process::Command;

pub fn create_new_project(name: &str) -> Result<(), Box<dyn std::error::Error>> {
  let project_dir = Path::new(name);

  // Check if directory already exists
  if project_dir.exists() {
    return Err(format!("Directory '{}' already exists", name).into());
  }

  // Create workspace structure
  fs::create_dir_all(project_dir.join("crates/app/src/handlers"))?;
  fs::create_dir_all(project_dir.join("crates/db/src/migrations"))?;
  fs::create_dir_all(project_dir.join("crates/db/src/seeds"))?;
  fs::create_dir_all(project_dir.join("crates/db/src/models"))?;
  fs::create_dir_all(project_dir.join("config"))?;

  // Get the absolute path to the forge crate relative to this executable
  let exe_path = std::env::current_exe().unwrap();
  let forge_crate_path = exe_path
    .parent()
    .unwrap() // target/debug or target/release
    .parent()
    .unwrap() // target
    .parent()
    .unwrap() // forge workspace root
    .join("crates")
    .join("forge");

  // Create Root Cargo.toml (Workspace)
  let root_cargo_toml = r#"[workspace]
members = [
  "crates/app",
  "crates/db",
]
resolver = "2"
"#;
  fs::write(project_dir.join("Cargo.toml"), root_cargo_toml)?;

  // Create crates/app/Cargo.toml
  let app_cargo_toml = format!(
    r#"[package]
name = "app"
version = "0.1.0"
edition = "2024"

[dependencies]
forge = {{ path = "{}" }}
db = {{ path = "../db" }}
tokio = {{ version = "1", features = ["full"] }}
serde = {{ version = "1.0", features = ["derive"] }}
axum = "0.8"
axum-login = "0.17"
sea-orm = {{ version = "1.1", features = ["runtime-tokio-rustls", "sqlx-sqlite", "macros"] }}
chrono = {{ version = "0.4", features = ["serde"] }}
uuid = {{ version = "1.0", features = ["v4", "serde"] }}
validator = {{ version = "0.20", features = ["derive"] }}
"#,
    forge_crate_path.display()
  );
  fs::write(project_dir.join("crates/app/Cargo.toml"), app_cargo_toml)?;

  // Create crates/db/Cargo.toml
  let db_cargo_toml = format!(
    r#"[package]
name = "db"
version = "0.1.0"
edition = "2024"

[dependencies]
forge = {{ path = "{}" }}
sea-orm = {{ version = "1.1", features = ["runtime-tokio-rustls", "sqlx-sqlite", "macros"] }}
sea-orm-migration = "1.1"
serde = {{ version = "1.0", features = ["derive"] }}
uuid = {{ version = "1.0", features = ["v4", "serde"] }}
chrono = {{ version = "0.4", features = ["serde"] }}
async-trait = "0.1"
tracing = "0.1"
"#,
    forge_crate_path.display()
  );
  fs::write(project_dir.join("crates/db/Cargo.toml"), db_cargo_toml)?;

  // Create .gitignore
  let gitignore = r#"# Rust build artifacts
target/

# IDE files
.vscode/
.idea/
*.swp
*.swo

# OS files
.DS_Store
Thumbs.db

# Environment variables
.env
.env.local

# Logs
*.log

# Database files
*.db
*.sqlite
*.sqlite3
"#;
  fs::write(project_dir.join(".gitignore"), gitignore)?;

  // Create crates/app/src/main.rs
  let main_rs = r#"use forge::App;
use db::auth::Backend;

mod handlers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let app = App::new()
    .with_migrations(db::Migrator)
    .with_seed(|db| Box::pin(db::run_seeds(db)))
    .with_auth(|db| Backend::new(db));

  let app = if app.config().app.environment.eq_ignore_ascii_case("production") {
    app
      .with_rate_limit_per_ip(60)
      .with_rate_limit_per_user(60)
  } else {
    app
  };

  app
    .route("/", || async { "Hello from Forge!" })
    .post_route("/api/auth/register", handlers::auth::register)
    .post_route("/api/auth/login", handlers::auth::login)
    .route("/api/auth/logout", handlers::auth::logout)
    .route("/api/auth/profile", handlers::auth::profile)
    .route("/api/auth/admin", handlers::auth::admin_only)
    .serve()
    .await
}
"#;
  fs::write(project_dir.join("crates/app/src/main.rs"), main_rs)?;

  // Create crates/app/src/handlers/mod.rs
  fs::write(
    project_dir.join("crates/app/src/handlers/mod.rs"),
    "pub mod auth;",
  )?;

  // Create crates/app/src/handlers/auth.rs
  let auth_handlers_rs = r#"use axum::{
  extract::State,
  http::StatusCode,
  response::IntoResponse,
  Json,
};
use axum_login::AuthSession;
use chrono::Utc;
use forge::auth::hash_password;
use forge::audit::{AuditEvent, EventKind, Outcome};
use forge::authz::{Action, AuthSessionGuardExt, AuthzContext, Role};
use forge::validation::Valid;
use forge::Error as ForgeError;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set, TransactionTrait};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use db::auth::Backend;
use db::models::{organization, membership, user};

#[derive(Deserialize, Validate)]
pub struct RegisterRequest {
  #[validate(email)]
  pub email: String,
  #[validate(length(min = 8))]
  pub password: String,
}

pub async fn register(
  State(db): State<DatabaseConnection>,
  Valid(Json(payload)): Valid<Json<RegisterRequest>>,
) -> Result<impl IntoResponse, ForgeError> {
  let password_hash = hash_password(&payload.password)?;
  let now = Utc::now().naive_utc();
  let user_id = Uuid::new_v4();
  let org_id = Uuid::new_v4();
  let membership_id = Uuid::new_v4();

  let tx = db.begin().await?;
  let new_user = user::ActiveModel {
    id: Set(user_id),
    email: Set(payload.email.clone()),
    password_hash: Set(password_hash),
    is_active: Set(true),
    is_admin: Set(false),
    current_org_id: Set(None),
    current_role: Set(None),
    created_at: Set(now),
    updated_at: Set(now),
  };
  user::Entity::insert(new_user).exec(&tx).await?;

  let slug = format!("org-{}", org_id.as_simple());
  let new_org = organization::ActiveModel {
    id: Set(org_id),
    name: Set(format!("{}'s workspace", payload.email)),
    slug: Set(slug),
    created_at: Set(now),
    updated_at: Set(now),
  };
  organization::Entity::insert(new_org).exec(&tx).await?;

  let new_membership = membership::ActiveModel {
    id: Set(membership_id),
    user_id: Set(user_id),
    org_id: Set(org_id),
    role: Set("owner".to_string()),
    created_at: Set(now),
    updated_at: Set(now),
  };
  membership::Entity::insert(new_membership).exec(&tx).await?;

  let u = user::Entity::find_by_id(user_id).one(&tx).await?.ok_or_else(|| ForgeError::Generic("User not found".into()))?;
  let mut am: user::ActiveModel = u.into();
  am.current_org_id = Set(Some(org_id));
  am.current_role = Set(Some("owner".to_string()));
  am.updated_at = Set(now);
  am.update(&tx).await?;

  tx.commit().await?;

  Ok((StatusCode::CREATED, "User registered successfully"))
}

#[derive(Deserialize, Validate)]
pub struct LoginRequest {
  #[validate(email)]
  pub email: String,
  #[validate(length(min = 1))]
  pub password: String,
}

pub async fn login(
  mut auth_session: AuthSession<Backend>,
  State(db): State<DatabaseConnection>,
  Valid(Json(payload)): Valid<Json<LoginRequest>>,
) -> Result<impl IntoResponse, ForgeError> {
  let credentials = db::auth::Credentials {
    email: payload.email,
    password: payload.password,
  };

  let user = auth_session
    .authenticate(credentials)
    .await
    .map_err(|e| ForgeError::Generic(format!("Authentication error: {}", e)))?;

  if let Some(ref user) = user {
    auth_session
      .login(user)
      .await
      .map_err(|e| ForgeError::Generic(format!("Login error: {}", e)))?;
    let _ = forge::audit::log(
      &db,
      AuditEvent {
        event_kind: EventKind::Auth,
        actor_id: user.id,
        subject_id: Some(user.id),
        organization_id: user.current_org_id,
        action: Action::Manage,
        resource_type: "auth".to_string(),
        resource_id: None,
        outcome: Outcome::Success,
        reason: Some("login".to_string()),
      },
    )
    .await;
    Ok(StatusCode::OK.into_response())
  } else {
    let _ = forge::audit::log(
      &db,
      AuditEvent {
        event_kind: EventKind::Auth,
        actor_id: Uuid::nil(),
        subject_id: None,
        organization_id: None,
        action: Action::Manage,
        resource_type: "auth".to_string(),
        resource_id: None,
        outcome: Outcome::Failure,
        reason: Some("failed_login".to_string()),
      },
    )
    .await;
    Ok((StatusCode::UNAUTHORIZED, "Invalid credentials").into_response())
  }
}

pub async fn logout(
  mut auth_session: AuthSession<Backend>,
  State(db): State<DatabaseConnection>,
) -> impl IntoResponse {
  let actor_id = auth_session.requester_id();
  let org_id = auth_session.organization_id();
  auth_session.logout().await.unwrap();
  let _ = forge::audit::log(
    &db,
    AuditEvent {
      event_kind: EventKind::Auth,
      actor_id,
      subject_id: Some(actor_id),
      organization_id: org_id,
      action: Action::Manage,
      resource_type: "auth".to_string(),
      resource_id: None,
      outcome: Outcome::Success,
      reason: Some("logout".to_string()),
    },
  )
  .await;
  StatusCode::OK
}

pub async fn profile(auth_session: AuthSession<Backend>) -> impl IntoResponse {
  match &auth_session.user {
    Some(user) => {
      let org = auth_session.organization_id().map(|id| id.to_string()).unwrap_or_else(|| "none".to_string());
      let role = auth_session.role().map(|r| format!("{:?}", r)).unwrap_or_else(|| "none".to_string());
      format!("Hello, {}! org={} role={}", user.email, org, role).into_response()
    }
    None => (StatusCode::UNAUTHORIZED, "Not logged in").into_response(),
  }
}

/// Shallow Gate example: only users with Role::Owner (or Admin) can access. Audits the decision.
pub async fn admin_only(
  auth_session: AuthSession<Backend>,
  State(db): State<DatabaseConnection>,
) -> Result<impl IntoResponse, ForgeError> {
  auth_session
    .guard_and_audit(&db, Action::Manage, Role::Owner, "admin", None)
    .await?;
  Ok((StatusCode::OK, "Admin only: access granted"))
}
"#;
  fs::write(
    project_dir.join("crates/app/src/handlers/auth.rs"),
    auth_handlers_rs,
  )?;

  // Create crates/db/src/lib.rs
  let db_lib_rs = r#"use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use sea_orm_migration::prelude::{MigrationTrait, MigratorTrait};

pub mod migrations;
pub mod models;
pub mod seeds;
pub mod auth;

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
    ]
  }
}

pub async fn run_seeds(db: DatabaseConnection) -> Result<(), Box<dyn std::error::Error>> {
  seeds::s20220101_000001_seed_users::seed(&db).await?;
  Ok(())
}
"#;
  fs::write(project_dir.join("crates/db/src/lib.rs"), db_lib_rs)?;

  // Create crates/db/src/auth.rs
  let db_auth_rs = r#"use async_trait::async_trait;
use forge::auth::verify_password;
use forge::{axum_login::AuthnBackend, Error};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Deserialize;

use crate::models::user;

#[derive(Clone, Debug)]
pub struct Backend {
  db: DatabaseConnection,
}

impl Backend {
  pub fn new(db: DatabaseConnection) -> Self {
    Self { db }
  }
}

#[derive(Debug, Deserialize)]
pub struct Credentials {
  pub email: String,
  pub password: String,
}

#[async_trait]
impl AuthnBackend for Backend {
  type User = user::Model;
  type Credentials = Credentials;
  type Error = Error;

  async fn authenticate(
  &self,
  creds: Self::Credentials,
  ) -> Result<Option<Self::User>, Self::Error> {
    let user = user::Entity::find()
      .filter(user::Column::Email.eq(creds.email))
      .one(&self.db)
      .await?;

    if let Some(user) = user {
      if verify_password(&creds.password, &user.password_hash)? {
        return Ok(Some(user));
      }
    }

    Ok(None)
  }

  async fn get_user(&self, user_id: &forge::axum_login::UserId<Self>) -> Result<Option<Self::User>, Error> {
    let user = user::Entity::find_by_id(*user_id)
      .one(&self.db)
      .await?;
    Ok(user)
  }
}
"#;
  fs::write(project_dir.join("crates/db/src/auth.rs"), db_auth_rs)?;

  // Create crates/db/src/migrations/mod.rs
  fs::write(
    project_dir.join("crates/db/src/migrations/mod.rs"),
    "pub mod m20220101_000001_create_user_table;\npub mod m20220101_000002_create_sessions_table;\npub mod m20220101_000003_create_organizations_table;\npub mod m20220101_000004_create_memberships_table;\npub mod m20220101_000005_create_audit_log_table;",
  )?;

  // Create crates/db/src/migrations/m20220101_000001_create_user_table.rs
  let migration_rs = r#"use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum User {
  Table,
  Id,
  Email,
  PasswordHash,
  IsActive,
  IsAdmin,
  CurrentOrgId,
  CurrentRole,
  CreatedAt,
  UpdatedAt,
}

pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m20220101_000001_create_user_table"
  }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(User::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(User::Id)
              .uuid()
              .not_null()
              .primary_key(),
          )
          .col(ColumnDef::new(User::Email).string().unique_key().not_null())
          .col(ColumnDef::new(User::PasswordHash).string().not_null())
          .col(ColumnDef::new(User::IsActive).boolean().not_null().default(true))
          .col(ColumnDef::new(User::IsAdmin).boolean().not_null().default(false))
          .col(ColumnDef::new(User::CurrentOrgId).uuid())
          .col(ColumnDef::new(User::CurrentRole).string())
          .col(ColumnDef::new(User::CreatedAt).date_time().not_null())
          .col(ColumnDef::new(User::UpdatedAt).date_time().not_null())
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(User::Table).to_owned())
      .await
  }
}
"#;
  fs::write(
    project_dir.join("crates/db/src/migrations/m20220101_000001_create_user_table.rs"),
    migration_rs,
  )?;

  // Create crates/db/src/migrations/m20220101_000002_create_sessions_table.rs
  let sessions_migration_rs = r#"use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m20220101_000002_create_sessions_table"
  }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(Alias::new("sessions"))
          .if_not_exists()
          .col(ColumnDef::new(Alias::new("id")).string().not_null().primary_key())
          .col(ColumnDef::new(Alias::new("data")).binary().not_null())
          .col(ColumnDef::new(Alias::new("expiry_date")).big_integer().not_null())
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(Alias::new("sessions")).to_owned())
      .await
  }
}
"#;
  fs::write(
    project_dir.join("crates/db/src/migrations/m20220101_000002_create_sessions_table.rs"),
    sessions_migration_rs,
  )?;

  let org_migration_rs = r#"use sea_orm_migration::prelude::*;

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
    "m20220101_000003_create_organizations_table"
  }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(Organization::Table)
          .if_not_exists()
          .col(ColumnDef::new(Organization::Id).uuid().not_null().primary_key())
          .col(ColumnDef::new(Organization::Name).string().not_null())
          .col(ColumnDef::new(Organization::Slug).string().unique_key().not_null())
          .col(ColumnDef::new(Organization::CreatedAt).date_time().not_null())
          .col(ColumnDef::new(Organization::UpdatedAt).date_time().not_null())
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager.drop_table(Table::drop().table(Organization::Table).to_owned()).await
  }
}
"#;
  fs::write(
    project_dir.join("crates/db/src/migrations/m20220101_000003_create_organizations_table.rs"),
    org_migration_rs,
  )?;

  let membership_migration_rs = r#"use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum Membership {
  Table,
  Id,
  UserId,
  OrgId,
  Role,
  CreatedAt,
  UpdatedAt,
}

pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m20220101_000004_create_memberships_table"
  }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(Membership::Table)
          .if_not_exists()
          .col(ColumnDef::new(Membership::Id).uuid().not_null().primary_key())
          .col(ColumnDef::new(Membership::UserId).uuid().not_null())
          .col(ColumnDef::new(Membership::OrgId).uuid().not_null())
          .col(ColumnDef::new(Membership::Role).string().not_null())
          .col(ColumnDef::new(Membership::CreatedAt).date_time().not_null())
          .col(ColumnDef::new(Membership::UpdatedAt).date_time().not_null())
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager.drop_table(Table::drop().table(Membership::Table).to_owned()).await
  }
}
"#;
  fs::write(
    project_dir.join("crates/db/src/migrations/m20220101_000004_create_memberships_table.rs"),
    membership_migration_rs,
  )?;

  // Create crates/db/src/migrations/m20220101_000005_create_audit_log_table.rs (Phase 7)
  let audit_log_migration_rs = r#"use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum AuditLog {
  Table,
  Id,
  EventKind,
  ActorId,
  SubjectId,
  OrganizationId,
  Action,
  ResourceType,
  ResourceId,
  Outcome,
  Reason,
  OccurredAt,
}

pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m20220101_000005_create_audit_log_table"
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
          .col(
            ColumnDef::new(AuditLog::Id)
              .uuid()
              .not_null()
              .primary_key(),
          )
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
"#;
  fs::write(
    project_dir.join("crates/db/src/migrations/m20220101_000005_create_audit_log_table.rs"),
    audit_log_migration_rs,
  )?;

  // Create crates/db/src/seeds/mod.rs
  fs::write(
    project_dir.join("crates/db/src/seeds/mod.rs"),
    "pub mod s20220101_000001_seed_users;",
  )?;

  // Create crates/db/src/seeds/s20220101_000001_seed_users.rs
  let seed_rs = r#"use chrono::Utc;
use forge::auth::hash_password;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use tracing::info;
use uuid::Uuid;

use crate::models::{organization, membership, user};

pub async fn seed(db: &DatabaseConnection) -> Result<(), Box<dyn std::error::Error>> {
  let email = "root@localhost";

  let existing = user::Entity::find()
    .filter(user::Column::Email.eq(email))
    .one(db)
    .await?;

  if existing.is_none() {
    let now = Utc::now().naive_utc();
    let user_id = Uuid::new_v4();
    let org_id = Uuid::new_v4();
    let membership_id = Uuid::new_v4();
    let password_hash = hash_password("password123")?;

    let root_user = user::ActiveModel {
      id: Set(user_id),
      email: Set(email.to_owned()),
      password_hash: Set(password_hash),
      is_active: Set(true),
      is_admin: Set(true),
      current_org_id: Set(Some(org_id)),
      current_role: Set(Some("owner".to_string())),
      created_at: Set(now),
      updated_at: Set(now),
    };
    user::Entity::insert(root_user).exec(db).await?;

    let default_org = organization::ActiveModel {
      id: Set(org_id),
      name: Set("Default".to_string()),
      slug: Set("default".to_string()),
      created_at: Set(now),
      updated_at: Set(now),
    };
    organization::Entity::insert(default_org).exec(db).await?;

    let root_membership = membership::ActiveModel {
      id: Set(membership_id),
      user_id: Set(user_id),
      org_id: Set(org_id),
      role: Set("owner".to_string()),
      created_at: Set(now),
      updated_at: Set(now),
    };
    membership::Entity::insert(root_membership).exec(db).await?;

    info!("Seeded root user: {} with org and Owner membership", email);
  }

  Ok(())
}
"#;
  fs::write(
    project_dir.join("crates/db/src/seeds/s20220101_000001_seed_users.rs"),
    seed_rs,
  )?;

  // Create crates/db/src/models/mod.rs
  fs::write(
    project_dir.join("crates/db/src/models/mod.rs"),
    "pub mod user;\npub mod organization;\npub mod membership;",
  )?;

  // Create crates/db/src/models/user.rs
  let user_model_rs = r#"use forge::authz::{AuthzContext, Role};
use forge::ForgeAuthUser;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, ForgeAuthUser)]
#[sea_orm(table_name = "user")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: Uuid,
  #[sea_orm(unique)]
  pub email: String,
  pub password_hash: String,
  pub is_active: bool,
  pub is_admin: bool,
  pub current_org_id: Option<Uuid>,
  pub current_role: Option<String>,
  pub created_at: chrono::NaiveDateTime,
  pub updated_at: chrono::NaiveDateTime,
}

impl AuthzContext for Model {
  fn requester_id(&self) -> Uuid {
    self.id
  }
  fn subject_id(&self) -> Uuid {
    self.id
  }
  fn organization_id(&self) -> Option<Uuid> {
    self.current_org_id
  }
  fn role(&self) -> Option<Role> {
    self.current_role.as_deref().and_then(|s| match s {
      "owner" => Some(Role::Owner),
      "admin" => Some(Role::Admin),
      "editor" => Some(Role::Editor),
      "viewer" => Some(Role::Viewer),
      _ => None,
    })
  }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
"#;
  fs::write(
    project_dir.join("crates/db/src/models/user.rs"),
    user_model_rs,
  )?;

  let org_model_rs = r#"use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "organization")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: Uuid,
  pub name: String,
  #[sea_orm(unique)]
  pub slug: String,
  pub created_at: chrono::NaiveDateTime,
  pub updated_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
"#;
  fs::write(
    project_dir.join("crates/db/src/models/organization.rs"),
    org_model_rs,
  )?;

  let membership_model_rs = r#"use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "membership")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: Uuid,
  pub user_id: Uuid,
  pub org_id: Uuid,
  pub role: String,
  pub created_at: chrono::NaiveDateTime,
  pub updated_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
"#;
  fs::write(
    project_dir.join("crates/db/src/models/membership.rs"),
    membership_model_rs,
  )?;

  // Create config/app.toml
  let app_toml = format!(
    r#"[app]
name = "{}"
environment = "development"

[server]
host = "0.0.0.0"
port = 3000
"#,
    name
  );
  fs::write(project_dir.join("config").join("app.toml"), app_toml)?;

  // Create config/db.toml
  let db_toml = r#"[database]
# SQLite connection string. The file will be created in the project root.
url = "sqlite://db.sqlite?mode=rwc"
max_connections = 5
min_connections = 1
connect_timeout = 10
idle_timeout = 600
auto_migrate = true
auto_seed = true
"#;
  fs::write(project_dir.join("config").join("db.toml"), db_toml)?;

  // Initialize git repository
  Command::new("git")
    .arg("init")
    .current_dir(project_dir)
    .status()?;

  Ok(())
}
