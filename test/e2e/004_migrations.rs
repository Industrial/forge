//! End-to-End tests for Forge migrations and seeding

use axum::extract::State;
use forge::App;
use forge::sea_orm::DatabaseConnection;
use forge::sea_orm_migration::prelude::*;
use http::{Request, StatusCode};
use std::fs;
use std::process::Command;
use tower::ServiceExt;

/// Helper function to get the path to the forge binary
fn get_forge_binary_path() -> std::path::PathBuf {
  if let Ok(path) = std::env::var("CARGO_BIN_EXE_forge") {
    std::path::PathBuf::from(path)
  } else {
    let mut current_dir = std::env::current_exe().unwrap();
    while current_dir.file_name().and_then(|s| s.to_str()) != Some("target") {
      if let Some(parent) = current_dir.parent() {
        current_dir = parent.to_path_buf();
      } else {
        break;
      }
    }
    let workspace_root = if current_dir.file_name().and_then(|s| s.to_str()) == Some("target") {
      current_dir.parent().unwrap().to_path_buf()
    } else {
      std::env::current_dir().unwrap()
    };

    workspace_root.join("target").join("debug").join("forge")
  }
}

#[tokio::test]
async fn forge_new_generates_workspace_structure() {
  let temp_dir = tempfile::tempdir().unwrap();
  let project_name = "workspace_test_app";
  let forge_binary = get_forge_binary_path();

  let new_result = Command::new(&forge_binary)
    .arg("new")
    .arg(project_name)
    .current_dir(&temp_dir)
    .output()
    .expect("Failed to run forge new");

  assert!(new_result.status.success());
  let root = temp_dir.path().join(project_name);

  // Workspace structure checks
  assert!(root.join("Cargo.toml").exists(), "Root Cargo.toml missing");
  assert!(
    root.join("crates/app/Cargo.toml").exists(),
    "crates/app/Cargo.toml missing"
  );
  assert!(
    root.join("crates/db/Cargo.toml").exists(),
    "crates/db/Cargo.toml missing"
  );
  assert!(root.join("crates/db/src/migrations/mod.rs").exists());
  assert!(root.join("crates/db/src/models/mod.rs").exists());
  assert!(root.join("crates/db/src/seeds/mod.rs").exists());

  let cargo_toml = fs::read_to_string(root.join("Cargo.toml")).unwrap();
  assert!(
    cargo_toml.contains("[workspace]"),
    "Root Cargo.toml should be a workspace"
  );
}

// Mock Migrator for testing
struct MockMigrator;

#[async_trait::async_trait]
impl MigratorTrait for MockMigrator {
  fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![Box::new(MockMigration)]
  }
}

struct MockMigration;

impl MigrationName for MockMigration {
  fn name(&self) -> &str {
    "m20220101_000001_mock"
  }
}

#[async_trait::async_trait]
impl MigrationTrait for MockMigration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(Alias::new("mock_table"))
          .if_not_exists()
          .col(
            ColumnDef::new(Alias::new("id"))
              .integer()
              .not_null()
              .auto_increment()
              .primary_key(),
          )
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(Alias::new("mock_table")).to_owned())
      .await
  }
}

async fn mock_seed(_db: DatabaseConnection) -> Result<(), Box<dyn std::error::Error>> {
  Ok(())
}

#[tokio::test]
async fn forge_app_applies_migrations_and_seeds_in_process() {
  let temp_dir = tempfile::tempdir().unwrap();
  let original_cwd = std::env::current_dir().unwrap();
  std::env::set_current_dir(temp_dir.path()).unwrap();

  // 1. Manually setup the environment (simulating forge new workspace structure)
  fs::create_dir_all("config").unwrap();
  fs::write(
    "config/app.toml",
    r#"[app]
name = "test"
environment = "test"
[server]
host = "127.0.0.1"
port = 3000
"#,
  )
  .unwrap();
  fs::write(
    "config/db.toml",
    r#"[database]
url = "sqlite::memory:"
auto_migrate = true
auto_seed = true
"#,
  )
  .unwrap();

  // 2. Initialize App with migrator and seed
  let app = App::new()
    .with_migrations(MockMigrator)
    .with_seed(|db| Box::pin(async move { mock_seed(db).await }))
    .route(
      "/db-check",
      |State(db): State<DatabaseConnection>| async move {
        let backend = db.get_database_backend();
        format!("Connected to {:?}", backend)
      },
    );

  let router = app.into_router().await;

  // 3. Send a virtual request
  let response = router
    .oneshot(
      Request::builder()
        .uri("/db-check")
        .body(axum::body::Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body_str = String::from_utf8_lossy(&body);
  assert!(body_str.contains("Sqlite"));

  std::env::set_current_dir(original_cwd).unwrap();
}
