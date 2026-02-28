//! Forge App builder - the core of the web framework.

use axum::{Router, handler::Handler, routing::get, routing::post};
use axum_login::AuthManagerLayerBuilder;
use futures::future::BoxFuture;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend};
use sea_orm_migration::MigratorTrait;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::SqliteStore;
use tracing::{info, warn};

use crate::config::{self, ForgeConfig};
use crate::db;

/// Type alias for the idempotent seeding function.
pub type SeedFn = Box<
  dyn Fn(DatabaseConnection) -> BoxFuture<'static, Result<(), Box<dyn std::error::Error>>>
    + Send
    + Sync,
>;

/// Type alias for the migrator function.
pub type MigratorFn =
  Box<dyn Fn() -> BoxFuture<'static, Result<(), Box<dyn std::error::Error>>> + Send + Sync>;

/// Type alias for the auth installer function.
pub type AuthInstallerFn = Box<
  dyn FnOnce(
      Router<DatabaseConnection>,
      DatabaseConnection,
    ) -> BoxFuture<'static, Router<DatabaseConnection>>
    + Send,
>;

/// The main Forge application builder.
///
/// Provides a fluent API for configuring and running Axum-based web applications.
pub struct App {
  /// The Axum router containing all configured routes and middleware.
  router: Router<DatabaseConnection>,
  /// Application configuration loaded from `config/app.toml` and `config/db.toml`
  config: ForgeConfig,
  /// Database connection (if configured)
  db: Option<DatabaseConnection>,
  /// Optional migrator function to run on startup
  migrator: Option<MigratorFn>,
  /// Optional seeder to run on startup
  seeder: Option<SeedFn>,
  /// Optional auth installer
  auth_installer: Option<AuthInstallerFn>,
}

impl App {
  /// Create a new Forge application. Panics if configuration cannot be loaded.
  pub fn new() -> Self {
    Self::try_new().unwrap_or_else(|e| {
      eprintln!("Error: {}", e);
      std::process::exit(1);
    })
  }

  /// Create a new Forge application, returning an error if configuration fails.
  pub fn try_new() -> Result<Self, Box<dyn std::error::Error>> {
    info!("Initializing Forge application");

    let config = config::load_config()?;
    info!("Application config: {:?}", config.app);
    info!("Server config: {:?}", config.server);
    info!("Database config: {:?}", config.database);

    let router: Router<DatabaseConnection> = Router::new();

    // Add tracing middleware for HTTP request logging
    let trace_layer = TraceLayer::new_for_http()
      .make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
      .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
      .on_response(DefaultOnResponse::new().level(tracing::Level::INFO));

    let router = router.layer(ServiceBuilder::new().layer(trace_layer));

    info!("Forge application initialized with tracing middleware");

    Ok(Self {
      router,
      config,
      db: None,
      migrator: None,
      seeder: None,
      auth_installer: None,
    })
  }

  /// Register a migrator to be run automatically on startup.
  pub fn with_migrations<M>(mut self, _migrator: M) -> Self
  where
    M: MigratorTrait + Send + Sync + 'static,
  {
    self.migrator = Some(Box::new(move || {
      Box::pin(async move {
        let config = config::load_config()?;
        let db = db::initialize_database(&config.database).await?;
        M::up(&db, None).await.map_err(|e| e.into())
      })
    }));
    self
  }

  /// Register an idempotent seeding function to be run automatically on startup.
  pub fn with_seed<F>(mut self, seeder: F) -> Self
  where
    F: Fn(DatabaseConnection) -> BoxFuture<'static, Result<(), Box<dyn std::error::Error>>>
      + Send
      + Sync
      + 'static,
  {
    self.seeder = Some(Box::new(seeder));
    self
  }

  /// Register authentication and session management.
  pub fn with_auth<B, F>(mut self, backend_factory: F) -> Self
  where
    B: axum_login::AuthnBackend + Send + Sync + 'static,
    B::User: axum_login::AuthUser<Id = uuid::Uuid> + crate::authz::AuthzContext,
    F: Fn(DatabaseConnection) -> B + Send + Sync + 'static,
  {
    self.auth_installer = Some(Box::new(move |router, db_conn| {
      Box::pin(async move {
        if db_conn.get_database_backend() != DbBackend::Sqlite {
          warn!("Authentication currently only supports SQLite session store out-of-the-box.");
          return router;
        }

        let pool = db_conn.get_sqlite_connection_pool();
        let session_store = SqliteStore::new(pool.clone());

        if let Err(e) = session_store.migrate().await {
          warn!("Failed to migrate sessions table: {}", e);
        }

        let session_layer = SessionManagerLayer::new(session_store)
          .with_secure(false)
          .with_expiry(Expiry::OnInactivity(
            tower_sessions::cookie::time::Duration::days(30),
          ));

        let backend = backend_factory(db_conn);
        let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

        router.layer(auth_layer)
      })
    }));
    self
  }

  /// Get a reference to the application configuration.
  pub fn config(&self) -> &ForgeConfig {
    &self.config
  }

  /// Get a reference to the database connection (if available).
  pub fn db(&self) -> Option<&DatabaseConnection> {
    self.db.as_ref()
  }

  /// Add a GET route to the application.
  pub fn route<H, T>(mut self, path: &str, handler: H) -> Self
  where
    H: Handler<T, DatabaseConnection>,
    T: 'static,
  {
    self.router = self.router.route(path, get(handler));
    self
  }

  /// Add a POST route to the application (e.g. for /auth/register, /auth/login).
  pub fn post_route<H, T>(mut self, path: &str, handler: H) -> Self
  where
    H: Handler<T, DatabaseConnection>,
    T: 'static,
  {
    self.router = self.router.route(path, post(handler));
    self
  }

  /// Consumes the App and returns the underlying Axum router.
  /// This is useful for testing or for manual server management.
  pub async fn into_router(self) -> Router {
    // Initialize tracing subscriber if not already initialized
    Self::init_tracing();

    let App {
      mut router,
      config,
      db: _,
      migrator,
      seeder,
      auth_installer,
    } = self;

    // Initialize database
    info!(
      "Initializing database connection to {}",
      config.database.url
    );
    let db_conn = db::initialize_database(&config.database)
      .await
      .unwrap_or_else(|e| {
        eprintln!("Database connection error: {}", e);
        std::process::exit(1);
      });

    // Run migrations if enabled and provided
    if config.database.auto_migrate
      && let Some(run_migrations) = migrator
    {
      info!("Running database migrations...");
      run_migrations().await.unwrap_or_else(|e| {
        eprintln!("Migration error: {}", e);
        std::process::exit(1);
      });
    }

    // Run seeder if enabled and provided
    if config.database.auto_seed
      && let Some(seeder_fn) = seeder
    {
      info!("Running database seeder...");
      seeder_fn(db_conn.clone()).await.unwrap_or_else(|e| {
        eprintln!("Seeding error: {}", e);
        std::process::exit(1);
      });
    }

    // Install authentication if configured
    if let Some(installer) = auth_installer {
      info!("Installing authentication middleware...");
      router = installer(router, db_conn.clone()).await;
    }

    // Inject database connection into state
    router.with_state(db_conn)
  }

  /// Start the server and serve the application.
  pub async fn serve(self) -> Result<(), Box<dyn std::error::Error>> {
    let host = self.config.server.host.clone();
    let port = self.config.server.port;
    let addr = format!("{}:{}", host, port);

    let router = self.into_router().await;

    match Self::bind_listener(&addr).await {
      Ok(listener) => {
        Self::log_server_start(&host, port);

        match axum::serve(listener, router)
          .with_graceful_shutdown(Self::shutdown_signal())
          .await
        {
          Ok(_) => {
            Self::log_server_stop();
            Ok(())
          }
          Err(e) => {
            Self::log_server_error(&e);
            Err(e.into())
          }
        }
      }
      Err(e) => {
        Self::log_bind_error(&host, port, &e);
        Err(e.into())
      }
    }
  }

  /// Initialize the tracing subscriber for logging.
  fn init_tracing() {
    let _ = tracing_subscriber::fmt()
      .with_env_filter(
        tracing_subscriber::EnvFilter::from_default_env()
          .add_directive("forge=info".parse().unwrap())
          .add_directive("tower_http=info".parse().unwrap()),
      )
      .try_init();
  }

  /// Log server start message.
  fn log_server_start(host: &str, port: u16) {
    info!("Forge server running on http://{}:{}", host, port);
  }

  /// Log server stop message.
  fn log_server_stop() {
    info!("Forge server shut down gracefully");
  }

  /// Log server error.
  fn log_server_error(e: &std::io::Error) {
    warn!("Error running server: {}", e);
  }

  /// Log bind error.
  fn log_bind_error(host: &str, port: u16, e: &std::io::Error) {
    warn!("Failed to bind to {}:{}: {}", host, port, e);
  }

  /// Bind TCP listener to address.
  async fn bind_listener(addr: &str) -> Result<tokio::net::TcpListener, std::io::Error> {
    tokio::net::TcpListener::bind(addr).await
  }

  /// Create a future that completes when a shutdown signal is received.
  async fn shutdown_signal() {
    let ctrl_c = signal::ctrl_c();
    let terminate = async {
      #[cfg(unix)]
      {
        signal::unix::signal(signal::unix::SignalKind::terminate())
          .expect("failed to install signal handler")
          .recv()
          .await;
      }
      #[cfg(not(unix))]
      {
        std::future::pending::<()>().await;
      }
    };

    tokio::select! {
      _ = ctrl_c => {
        info!("Received Ctrl+C signal, initiating graceful shutdown");
      },
      _ = terminate => {
        info!("Received SIGTERM signal, initiating graceful shutdown");
      },
    }
  }
}

impl Default for App {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use axum::response::Html;
  use std::fs;
  use std::path::Path;

  fn setup_test_config(dir: &Path, project_name: &str) {
    let config_dir = dir.join("config");
    fs::create_dir_all(&config_dir).unwrap();
    let app_toml_content = format!(
      r#"[app]
name = "{}"
environment = "test"

[server]
host = "127.0.0.1"
port = 3000
"#,
      project_name
    );
    fs::write(config_dir.join("app.toml"), app_toml_content).unwrap();

    let db_toml_content = r#"[database]
url = "sqlite::memory:"
"#;
    fs::write(config_dir.join("db.toml"), db_toml_content).unwrap();
  }

  mod app_creation {
    use super::*;

    #[test]
    fn new_creates_app_with_empty_router() {
      let temp_dir = tempfile::tempdir().unwrap();
      let original_cwd = std::env::current_dir().unwrap();
      std::env::set_current_dir(temp_dir.path()).unwrap();
      setup_test_config(temp_dir.path(), "test_app");

      let _app = App::new();

      std::env::set_current_dir(original_cwd).unwrap();
    }

    #[test]
    fn default_trait_creates_same_as_new() {
      let temp_dir = tempfile::tempdir().unwrap();
      let original_cwd = std::env::current_dir().unwrap();
      std::env::set_current_dir(temp_dir.path()).unwrap();
      setup_test_config(temp_dir.path(), "test_app");

      let _app_new = App::new();
      let _app_default = App::default();

      std::env::set_current_dir(original_cwd).unwrap();
    }
  }

  mod route_registration {
    use super::*;

    #[test]
    fn route_method_accepts_static_handler() {
      let temp_dir = tempfile::tempdir().unwrap();
      let original_cwd = std::env::current_dir().unwrap();
      std::env::set_current_dir(temp_dir.path()).unwrap();
      setup_test_config(temp_dir.path(), "test_app");

      let app = App::new().route("/", || async { "Hello" });
      let _ = app;

      std::env::set_current_dir(original_cwd).unwrap();
    }

    #[test]
    fn route_method_accepts_complex_handler() {
      let temp_dir = tempfile::tempdir().unwrap();
      let original_cwd = std::env::current_dir().unwrap();
      std::env::set_current_dir(temp_dir.path()).unwrap();
      setup_test_config(temp_dir.path(), "test_app");

      let app = App::new().route("/api", || async { Html("<h1>API</h1>") });
      let _ = app;

      std::env::set_current_dir(original_cwd).unwrap();
    }
  }

  mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn app_uses_config_host_and_port() {
      let temp_dir = tempfile::tempdir().unwrap();
      let original_cwd = std::env::current_dir().unwrap();
      std::env::set_current_dir(temp_dir.path()).unwrap();

      let config_dir = temp_dir.path().join("config");
      fs::create_dir_all(&config_dir).unwrap();
      let app_toml_content = r#"[app]
name = "test_app"
environment = "test"

[server]
host = "127.0.0.1"
port = 0
"#;
      fs::write(config_dir.join("app.toml"), app_toml_content).unwrap();

      let db_toml_content = r#"[database]
url = "sqlite::memory:"
"#;
      fs::write(config_dir.join("db.toml"), db_toml_content).unwrap();

      let app = App::new();
      assert_eq!(app.config().server.host, "127.0.0.1");
      assert_eq!(app.config().server.port, 0);

      std::env::set_current_dir(original_cwd).unwrap();
    }
  }
}
