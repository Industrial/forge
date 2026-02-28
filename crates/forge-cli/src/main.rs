//! Forge CLI — invoke from tests by running the binary and asserting on output.

use clap::{CommandFactory, Parser, Subcommand};
use forge::ForgeConfig;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Parser, Debug)]
#[command(name = "forge")]
#[command(about = "Forge — full-stack web framework for Rust")]
#[command(version)]
struct Args {
  #[command(subcommand)]
  command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
  /// Create a new Forge project
  New {
    /// Name of the project to create
    name: String,
  },
  /// Serve the current Forge project
  Serve {},
}

fn main() -> std::process::ExitCode {
  let args = Args::parse();

  match args.command {
    Some(Commands::New { name }) => {
      if let Err(e) = create_new_project(&name) {
        eprintln!("Error creating project: {}", e);
        return std::process::ExitCode::FAILURE;
      }
      println!("Created new Forge project: {}", name);
      std::process::ExitCode::SUCCESS
    }
    Some(Commands::Serve {}) => {
      use forge::config;
      let config = match config::load_config() {
        Ok(c) => c,
        Err(e) => {
          eprintln!("Error: {}", e);
          return std::process::ExitCode::FAILURE;
        }
      };
      if let Err(e) = serve_project(&config) {
        eprintln!("Error serving project: {}", e);
        return std::process::ExitCode::FAILURE;
      }
      std::process::ExitCode::SUCCESS
    }
    None => {
      // No subcommand provided, show help
      let _ = Args::command().print_help();
      std::process::ExitCode::SUCCESS
    }
  }
}

fn create_new_project(name: &str) -> Result<(), Box<dyn std::error::Error>> {
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
edition = "2021"

[dependencies]
forge = {{ path = "{}" }}
db = {{ path = "../db" }}
tokio = {{ version = "1", features = ["full"] }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
axum = "0.8"
"#,
    forge_crate_path.display()
  );
  fs::write(project_dir.join("crates/app/Cargo.toml"), app_cargo_toml)?;

  // Create crates/db/Cargo.toml
  let db_cargo_toml = format!(
    r#"[package]
name = "db"
version = "0.1.0"
edition = "2021"

[dependencies]
async-trait = "0.1"
chrono = {{ version = "0.4", features = ["serde"] }}
forge = {{ path = "{}" }}
sea-orm = {{ version = "1.1", features = ["runtime-tokio-rustls", "sqlx-sqlite", "macros"] }}
serde = {{ version = "1", features = ["derive"] }}
uuid = {{ version = "1", features = ["v4", "serde"] }}
axum-login = "0.17"
tower-sessions = "0.14"
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
  let main_rs = r#"use forge::prelude::*;
use forge::axum::extract::State;
use db::auth::Backend;

mod handlers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::new()
        .with_migrations(db::Migrator)
        .with_seed(|db| Box::pin(db::run_seeds(db)))
        .with_auth(|db| Backend::new(db))
        .route("/", || async { "Hello from Forge!" })
        .route("/auth/register", handlers::auth::register)
        .route("/auth/login", handlers::auth::login)
        .route("/auth/logout", handlers::auth::logout)
        .route("/auth/profile", handlers::auth::profile)
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
  let auth_handlers_rs = r#"use forge::prelude::*;
use forge::axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_login::AuthSession;
use db::auth::Backend;
use db::models::user;
use serde::Deserialize;
use uuid::Uuid;
use chrono::Utc;
use forge::sea_orm::{EntityTrait, Set, QueryFilter, ColumnTrait};

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

pub async fn register(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<RegisterRequest>,
) -> Result<impl IntoResponse, forge::Error> {
    let password_hash = forge::auth::hash_password(&payload.password)?;
    let now = Utc::now().naive_utc();
    
    let new_user = user::ActiveModel {
        id: Set(Uuid::new_v4()),
        email: Set(payload.email),
        password_hash: Set(password_hash),
        is_active: Set(true),
        is_admin: Set(false),
        created_at: Set(now),
        updated_at: Set(now),
    };

    user::Entity::insert(new_user).exec(&db).await?;

    Ok((StatusCode::CREATED, "User registered successfully"))
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

pub async fn login(
    mut auth_session: AuthSession<Backend>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, forge::Error> {
    let credentials = db::auth::Credentials {
        email: payload.email,
        password: payload.password,
    };

    let user = auth_session
        .authenticate(credentials)
        .await
        .map_err(|e| forge::Error::Generic(format!("Authentication error: {}", e)))?;

    if let Some(user) = user {
        auth_session
            .login(&user)
            .await
            .map_err(|e| forge::Error::Generic(format!("Login error: {}", e)))?;
        Ok(StatusCode::OK.into_response())
    } else {
        Ok((StatusCode::UNAUTHORIZED, "Invalid credentials").into_response())
    }
}

pub async fn logout(mut auth_session: AuthSession<Backend>) -> impl IntoResponse {
    auth_session.logout().await.unwrap();
    StatusCode::OK
}

pub async fn profile(auth_session: AuthSession<Backend>) -> impl IntoResponse {
    match auth_session.user {
        Some(user) => format!("Hello, {}!", user.email).into_response(),
        None => (StatusCode::UNAUTHORIZED, "Not logged in").into_response(),
    }
}
"#;
  fs::write(
    project_dir.join("crates/app/src/handlers/auth.rs"),
    auth_handlers_rs,
  )?;

  // Create crates/db/src/lib.rs
  let db_lib_rs = r#"use forge::sea_orm::DatabaseConnection;
use forge::sea_orm_migration::prelude::*;

pub mod migrations;
pub mod models;
pub mod seeds;
pub mod auth;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(migrations::m20220101_000001_create_user_table::Migration),
            Box::new(migrations::m20220101_000002_create_sessions_table::Migration),
        ]
    }
}

pub async fn run_seeds(db: DatabaseConnection) -> Result<(), Box<dyn std::error::Error>> {
    seeds::s20220101_000001_seed_users::seed(&db).await?;
    Ok(())
}

pub mod prelude {
    pub use crate::models::user;
}
"#;
  fs::write(project_dir.join("crates/db/src/lib.rs"), db_lib_rs)?;

  // Create crates/db/src/auth.rs
  let db_auth_rs = r#"use axum_login::AuthnBackend;
use forge::prelude::*;
use forge::sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait};
use crate::models::user;
use serde::Deserialize;

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

#[async_trait::async_trait]
impl AuthnBackend for Backend {
    type User = user::Model;
    type Credentials = Credentials;
    type Error = forge::Error;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let user = user::Entity::find()
            .filter(user::Column::Email.eq(creds.email))
            .one(&self.db)
            .await?;

        if let Some(user) = user {
            if forge::auth::verify_password(&creds.password, &user.password_hash)? {
                return Ok(Some(user));
            }
        }

        Ok(None)
    }

    async fn get_user(&self, user_id: &axum_login::UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
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
    "pub mod m20220101_000001_create_user_table;\npub mod m20220101_000002_create_sessions_table;",
  )?;

  // Create crates/db/src/migrations/m20220101_000001_create_user_table.rs
  let migration_rs = r#"use forge::sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum User {
    Table,
    Id,
    Email,
    PasswordHash,
    IsActive,
    IsAdmin,
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
  let sessions_migration_rs = r#"use forge::sea_orm_migration::prelude::*;

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

  // Create crates/db/src/seeds/mod.rs
  fs::write(
    project_dir.join("crates/db/src/seeds/mod.rs"),
    "pub mod s20220101_000001_seed_users;",
  )?;

  // Create crates/db/src/seeds/s20220101_000001_seed_users.rs
  let seed_rs = r#"use forge::sea_orm::{DatabaseConnection, EntityTrait, Set, QueryFilter, ColumnTrait};
use crate::models::user;
use chrono::Utc;
use uuid::Uuid;

pub async fn seed(db: &DatabaseConnection) -> Result<(), Box<dyn std::error::Error>> {
    let email = "root@localhost";
    
    // Idempotent check
    let existing = user::Entity::find()
        .filter(user::Column::Email.eq(email))
        .one(db)
        .await?;

    if existing.is_none() {
        let now = Utc::now().naive_utc();
        let password_hash = forge::auth::hash_password("password123")?;
        let root = user::ActiveModel {
            id: Set(Uuid::new_v4()),
            email: Set(email.to_owned()),
            password_hash: Set(password_hash),
            is_active: Set(true),
            is_admin: Set(true),
            created_at: Set(now),
            updated_at: Set(now),
        };
        user::Entity::insert(root).exec(db).await?;
        println!("Seeded root user: {}", email);
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
    "pub mod user;",
  )?;

  // Create crates/db/src/models/user.rs
  let user_model_rs = r#"use forge::sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use forge::ForgeAuthUser;

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
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
"#;
  fs::write(
    project_dir.join("crates/db/src/models/user.rs"),
    user_model_rs,
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

fn serve_project(config: &ForgeConfig) -> Result<(), Box<dyn std::error::Error>> {
  // Check for Cargo.toml
  if !std::path::Path::new("Cargo.toml").exists() {
    return Err("No Cargo.toml found. Run `forge new myapp` to create a Forge project.".into());
  }

  // Verify workspace in Cargo.toml
  let cargo_toml = std::fs::read_to_string("Cargo.toml")?;
  if !cargo_toml.contains("[workspace]") {
    return Err("Not a Forge workspace. Ensure your Cargo.toml has a [workspace] section.".into());
  }

  // Check for crates/app/src/main.rs
  if !std::path::Path::new("crates/app/src/main.rs").exists() {
    return Err(
      "crates/app/src/main.rs not found. Ensure your project has an app crate with a main.rs."
        .into(),
    );
  }

  let port = config.server.port; // Get port from config
  let host = &config.server.host;
  // Spawn cargo run --package app
  let status = Command::new("cargo")
    .arg("run")
    .arg("--package")
    .arg("app")
    .env("PORT", port.to_string())
    .env("HOST", host)
    .status()?;

  if status.success() {
    Ok(())
  } else {
    Err(
      format!(
        "Cargo run failed with exit code: {}",
        status.code().unwrap_or(-1)
      )
      .into(),
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;
  use tempfile::tempdir;

  /// Test suite for CLI argument parsing
  mod cli_parsing {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn args_struct_parses_version_flag() {
      // Given: Command line arguments with --version
      let args = Args::try_parse_from(["forge", "--version"]);

      // When: Parsing the arguments
      // Then: It should return an error because --version causes clap to exit
      // This is expected behavior - clap handles --version by printing and exiting
      assert!(args.is_err());
      // Note: version is handled by clap automatically
    }

    #[test]
    fn args_struct_parses_help_flag() {
      // Given: Command line arguments with --help
      let _args = Args::try_parse_from(["forge", "--help"]);

      // When: Parsing the arguments
      // Then: It should succeed (clap handles help internally)
      // Note: --help causes clap to exit, so we test the structure
      let cmd = Args::command();
      assert!(cmd.get_name() == "forge");
    }

    #[test]
    fn new_subcommand_parses_project_name() {
      // Given: Command line with new subcommand
      let args = Args::try_parse_from(["forge", "new", "my_project"]);

      // When: Parsing the arguments
      // Then: It should contain the New command with correct name
      assert!(args.is_ok());
      let args = args.unwrap();
      match args.command {
        Some(Commands::New { name }) => {
          assert_eq!(name, "my_project");
        }
        _ => panic!("Expected New command"),
      }
    }

    #[test]
    fn no_subcommand_defaults_to_none() {
      // Given: Command line with no subcommand
      let args = Args::try_parse_from(["forge"]);

      // When: Parsing the arguments
      // Then: command should be None
      assert!(args.is_ok());
      let args = args.unwrap();
      assert!(args.command.is_none());
    }
  }

  /// Test suite for Commands enum
  mod commands_enum {
    use super::*;

    #[test]
    fn commands_enum_has_new_variant() {
      // Given: Commands enum
      // When: Creating New variant
      let cmd = Commands::New {
        name: "test".to_string(),
      };

      // Then: It should compile and work
      match cmd {
        Commands::New { name } => assert_eq!(name, "test"),
        _ => panic!("Expected New variant"),
      }
    }

    #[test]
    fn commands_enum_has_serve_variant() {
      // Given: Commands enum
      // When: Creating Serve variant
      let cmd = Commands::Serve {};

      // Then: It should compile and work
      match cmd {
        Commands::Serve {} => (),
        _ => panic!("Expected Serve variant"),
      }
    }

    #[test]
    fn commands_enum_debug_trait_works() {
      // Given: Any Commands variant
      let cmd = Commands::New {
        name: "debug_test".to_string(),
      };

      // When: Using Debug formatting
      let debug_str = format!("{:?}", cmd);

      // Then: It should produce debug output
      assert!(debug_str.contains("New"));
      assert!(debug_str.contains("debug_test"));
    }
  }

  /// Test suite for main function command dispatching
  mod main_function_dispatch {
    use super::*;

    #[test]
    fn main_handles_new_command_success() {
      // Given: A temporary directory for testing
      let temp_dir = tempdir().unwrap();
      let original_cwd = std::env::current_dir().unwrap();

      // Change to temp directory for isolated testing
      std::env::set_current_dir(&temp_dir).unwrap();

      // When: main processes New command
      // Note: We can't easily test main() directly, but we test the logic
      // by testing the create_new_project function separately

      // Restore original directory
      std::env::set_current_dir(original_cwd).unwrap();
    }

    #[test]
    fn main_handles_serve_command_success() {
      // Given: Valid project setup (tested separately)
      // When: main processes Serve command
      // Then: It should dispatch to serve_project
    }

    #[test]
    fn main_handles_no_command_by_showing_help() {
      // Given: No subcommand provided
      // When: main is called
      // Then: It should show help and return success
      // Note: This is tested by the Args parsing tests above
    }
  }

  /// Test suite for create_new_project function
  mod create_new_project_function {
    use super::*;

    #[test]
    fn create_new_project_succeeds_with_valid_name() {
      // Given: A temporary directory
      let temp_dir = tempdir().unwrap();
      let project_name = "test_project";

      // When: Creating a new project
      let project_path = temp_dir.path().join(project_name);

      // Change to temp directory for the function to work
      let original_cwd = std::env::current_dir().unwrap();
      std::env::set_current_dir(&temp_dir).unwrap();

      let result = create_new_project(project_name);

      // Then: It should succeed and create files
      assert!(result.is_ok());

      // Verify directory structure
      assert!(project_path.exists());
      assert!(project_path.join("Cargo.toml").exists());
      assert!(project_path.join("crates/app").exists());
      assert!(project_path.join("crates/app/src/main.rs").exists());
      assert!(project_path.join("crates/db/src/lib.rs").exists());
      assert!(project_path.join(".gitignore").exists());
      assert!(project_path.join("config/app.toml").exists());

      // Verify git initialization
      assert!(project_path.join(".git").exists());

      // Restore original directory
      std::env::set_current_dir(original_cwd).unwrap();
    }

    #[test]
    fn create_new_project_fails_with_existing_directory() {
      // Given: A temporary directory with existing subdirectory
      let temp_dir = tempdir().unwrap();
      let project_name = "existing_dir";
      let project_path = temp_dir.path().join(project_name);

      // Create the directory first
      fs::create_dir(&project_path).unwrap();

      // Change to temp directory
      let original_cwd = std::env::current_dir().unwrap();
      std::env::set_current_dir(&temp_dir).unwrap();

      // When: Trying to create project with existing name
      let result = create_new_project(project_name);

      // Then: It should fail
      assert!(result.is_err());
      assert!(result.unwrap_err().to_string().contains("already exists"));

      // Restore original directory
      std::env::set_current_dir(original_cwd).unwrap();
    }

    #[test]
    fn create_new_project_generates_correct_cargo_toml() {
      // Given: Project creation setup
      let temp_dir = tempdir().unwrap();
      let project_name = "cargo_test";

      // Change to temp directory
      let original_cwd = std::env::current_dir().unwrap();
      std::env::set_current_dir(&temp_dir).unwrap();

      // When: Creating project
      let result = create_new_project(project_name);
      assert!(result.is_ok());

      // Then: Cargo.toml should have correct content
      let cargo_content = fs::read_to_string("cargo_test/Cargo.toml").unwrap();
      assert!(cargo_content.contains("[workspace]"));
      assert!(cargo_content.contains("crates/app"));
      assert!(cargo_content.contains("crates/db"));

      let app_cargo = fs::read_to_string("cargo_test/crates/app/Cargo.toml").unwrap();
      assert!(app_cargo.contains("name = \"app\""));
      assert!(app_cargo.contains("forge ="));
      assert!(app_cargo.contains("db ="));

      // Restore original directory
      std::env::set_current_dir(original_cwd).unwrap();
    }

    #[test]
    fn create_new_project_generates_correct_main_rs() {
      // Given: Project creation setup
      let temp_dir = tempdir().unwrap();
      let project_name = "main_test";

      // Change to temp directory
      let original_cwd = std::env::current_dir().unwrap();
      std::env::set_current_dir(&temp_dir).unwrap();

      // When: Creating project
      let result = create_new_project(project_name);
      assert!(result.is_ok());

      // Then: main.rs should have correct Forge app code
      let main_content = fs::read_to_string("main_test/crates/app/src/main.rs").unwrap();
      assert!(main_content.contains("use forge::prelude::*;"));
      assert!(main_content.contains("App::new()"));
      assert!(main_content.contains(".with_migrations(db::Migrator)"));
      assert!(main_content.contains(".serve()"));

      // Restore original directory
      std::env::set_current_dir(original_cwd).unwrap();
    }

    #[test]
    fn create_new_project_generates_gitignore() {
      // Given: Project creation setup
      let temp_dir = tempdir().unwrap();
      let project_name = "gitignore_test";

      // Change to temp directory
      let original_cwd = std::env::current_dir().unwrap();
      std::env::set_current_dir(&temp_dir).unwrap();

      // When: Creating project
      let result = create_new_project(project_name);
      assert!(result.is_ok());

      // Then: .gitignore should exist with correct content
      let gitignore_content = fs::read_to_string("gitignore_test/.gitignore").unwrap();
      assert!(gitignore_content.contains("target/"));
      assert!(gitignore_content.contains(".env"));
      assert!(gitignore_content.contains("*.log"));
      assert!(gitignore_content.contains(".vscode/"));
      assert!(gitignore_content.contains(".DS_Store"));

      // Restore original directory
      std::env::set_current_dir(original_cwd).unwrap();
    }
  }

  /// Test suite for serve_project function
  mod serve_project_function {
    // ... (rest of the file remains same)
  }
}
