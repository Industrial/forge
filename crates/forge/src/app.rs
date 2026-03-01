//! Forge App builder - the core of the web framework.

use std::future::Future;
use std::sync::Arc;

use axum::{Router, handler::Handler, routing::delete, routing::get, routing::post};
use axum_login::AuthManagerLayerBuilder;
use futures::future::BoxFuture;
use governor::middleware::NoOpMiddleware;
use sea_orm::{ConnectionTrait, DbBackend};

use crate::DbConnection;
use sea_orm_migration::MigratorTrait;
use tokio::signal;
use tower::ServiceBuilder;
use tower_governor::{
  GovernorLayer,
  governor::{GovernorConfig, GovernorConfigBuilder},
  key_extractor::PeerIpKeyExtractor,
};
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::SqliteStore;
use tracing::{info, warn};
use tracing_subscriber::prelude::*;

use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};

use crate::cache;
use crate::cache_http_layer;
use crate::config::{self, ForgeConfig};
use crate::cron::{CronRunner, CronSchedule, CronTaskBox};
use crate::db;
use crate::observability;
use crate::security_headers;
use crate::token_auth::TokenAuthLayer;
use crate::token_auth::TokenLookupFn;

/// Type alias for the idempotent seeding function.
pub type SeedFn = Box<
  dyn Fn(DbConnection) -> BoxFuture<'static, Result<(), Box<dyn std::error::Error>>> + Send + Sync,
>;

/// Type alias for the migrator function.
pub type MigratorFn =
  Box<dyn Fn() -> BoxFuture<'static, Result<(), Box<dyn std::error::Error>>> + Send + Sync>;

/// Type alias for the auth installer function.
/// Args: router, db, optional per-user rate limit, optional token lookup for Bearer auth.
pub type AuthInstallerFn = Box<
  dyn FnOnce(
      Router<DbConnection>,
      DbConnection,
      Option<u32>,
      Option<TokenLookupFn>,
    ) -> BoxFuture<'static, Router<DbConnection>>
    + Send,
>;

/// The main Forge application builder.
///
/// Provides a fluent API for configuring and running Axum-based web applications.
pub struct App {
  /// The Axum router containing all configured routes and middleware.
  router: Router<DbConnection>,
  /// Application configuration loaded from `config/app.toml` and `config/db.toml`
  config: ForgeConfig,
  /// Database connection (if configured)
  db: Option<DbConnection>,
  /// Optional migrator function to run on startup
  migrator: Option<MigratorFn>,
  /// Optional seeder to run on startup
  seeder: Option<SeedFn>,
  /// Optional auth installer
  auth_installer: Option<AuthInstallerFn>,
  /// Optional per-IP rate limit config (health routes are excluded)
  rate_limit_per_ip: Option<Arc<GovernorConfig<PeerIpKeyExtractor, NoOpMiddleware>>>,
  /// Optional per-user (per-org) rate limit: requests per minute per (org, user). Requires auth.
  rate_limit_per_user: Option<u32>,
  /// Optional token lookup for Bearer auth: (db, raw_token) -> Option<user_id>. Used when with_token_auth is set.
  token_lookup: Option<TokenLookupFn>,
  /// Cron tasks to run in-process when serve() is used.
  cron_tasks: Vec<(String, CronSchedule, CronTaskBox)>,
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

    let router: Router<DbConnection> = Router::new();

    // HTTP request logging (tower-http)
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
      rate_limit_per_ip: None,
      rate_limit_per_user: None,
      token_lookup: None,
      cron_tasks: Vec::new(),
    })
  }

  /// Enable per-requester (per user, per organization) rate limiting.
  /// Only has effect when [`.with_auth`](Self::with_auth) is also used. Limits are applied
  /// per (organization_id, user_id) so individual users in an org can be throttled.
  /// Unauthenticated requests to rate-limited routes receive 401.
  pub fn with_rate_limit_per_user(mut self, requests_per_minute: u32) -> Self {
    self.rate_limit_per_user = Some(requests_per_minute.max(1));
    self
  }

  /// Enable per-IP rate limiting. `requests_per_minute` sets burst size and replenishment
  /// (e.g. 60 = 60 requests then 1 per second). Health endpoints (/healthz, /livez, /readyz)
  /// are never rate limited.
  pub fn with_rate_limit_per_ip(mut self, requests_per_minute: u32) -> Self {
    let burst = requests_per_minute.max(1);
    let conf = GovernorConfigBuilder::default()
      .per_second(1)
      .burst_size(burst)
      .finish()
      .expect("GovernorConfigBuilder");
    self.rate_limit_per_ip = Some(Arc::new(conf));
    self
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
    F: Fn(DbConnection) -> BoxFuture<'static, Result<(), Box<dyn std::error::Error>>>
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
    B: axum_login::AuthnBackend + Send + Sync + Clone + 'static,
    B::User:
      axum_login::AuthUser<Id = uuid::Uuid> + crate::authz::AuthzContext + Send + Clone + 'static,
    F: Fn(DbConnection) -> B + Send + Sync + 'static,
  {
    self.auth_installer = Some(Box::new(
      move |router, db_conn, rate_limit_per_user, token_lookup| {
        Box::pin(async move {
          if db_conn.get_database_backend() != DbBackend::Sqlite {
            warn!("Authentication currently only supports SQLite session store out-of-the-box.");
            return router;
          }

          let pool = db_conn.inner().get_sqlite_connection_pool();
          let session_store = SqliteStore::new(pool.clone());

          if let Err(e) = session_store.migrate().await {
            warn!("Failed to migrate sessions table: {}", e);
          }

          let session_layer = SessionManagerLayer::new(session_store)
            .with_secure(false)
            .with_same_site(tower_sessions::cookie::SameSite::Lax)
            .with_expiry(Expiry::OnInactivity(
              tower_sessions::cookie::time::Duration::days(30),
            ));

          let backend = backend_factory(db_conn.clone());
          let auth_layer = AuthManagerLayerBuilder::new(backend.clone(), session_layer).build();

          let router = if let Some(ref lookup) = token_lookup {
            let token_layer = TokenAuthLayer::new(db_conn.clone(), backend, lookup.clone());
            router.layer(token_layer).layer(auth_layer)
          } else {
            router.layer(auth_layer)
          };

          if let Some(n) = rate_limit_per_user {
            let burst = n.max(1);
            let mut builder = GovernorConfigBuilder::default();
            builder.per_second(1).burst_size(burst);
            let mut builder2 =
              builder.key_extractor(crate::rate_limit::RequesterOrgKeyExtractor::<B>::new());
            let conf = builder2.finish().expect("GovernorConfigBuilder per-user");
            router.layer(GovernorLayer::new(Arc::new(conf)))
          } else {
            router
          }
        })
      },
    ));
    self
  }

  /// Enable API token (Bearer) authentication alongside session auth. The closure receives (db, raw_token)
  /// and returns the user id if the token is valid (e.g. lookup by token hash in api_tokens table).
  pub fn with_token_auth<F, Fut>(mut self, lookup: F) -> Self
  where
    F: Fn(DbConnection, String) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Option<uuid::Uuid>> + Send + 'static,
  {
    let lookup = Arc::new(move |db: DbConnection, token: String| {
      Box::pin(lookup(db, token)) as BoxFuture<'static, Option<uuid::Uuid>>
    });
    self.token_lookup = Some(lookup);
    self
  }

  /// Get a reference to the application configuration.
  pub fn config(&self) -> &ForgeConfig {
    &self.config
  }

  /// Get a reference to the database connection (if available).
  pub fn db(&self) -> Option<&DbConnection> {
    self.db.as_ref()
  }

  /// Add a GET route to the application.
  pub fn route<H, T>(mut self, path: &str, handler: H) -> Self
  where
    H: Handler<T, DbConnection>,
    T: 'static,
  {
    self.router = self.router.route(path, get(handler));
    self
  }

  /// Add a POST route to the application (e.g. for /api/auth/register, /api/auth/login).
  pub fn post_route<H, T>(mut self, path: &str, handler: H) -> Self
  where
    H: Handler<T, DbConnection>,
    T: 'static,
  {
    self.router = self.router.route(path, post(handler));
    self
  }

  /// Add a DELETE route to the application (e.g. for /api/auth/tokens/:id).
  pub fn delete_route<H, T>(mut self, path: &str, handler: H) -> Self
  where
    H: Handler<T, DbConnection>,
    T: 'static,
  {
    self.router = self.router.route(path, delete(handler));
    self
  }

  /// Register a cron task that runs on the given schedule (in-process when [`.serve`](Self::serve) is used).
  /// The task receives the app's database connection. No external cron library; uses Interval / Hourly / Daily.
  pub fn with_cron<F, Fut>(mut self, name: &str, schedule: CronSchedule, f: F) -> Self
  where
    F: Fn(DbConnection) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>>
      + Send
      + 'static,
  {
    let task: CronTaskBox = Box::new(move |db| Box::pin(f(db)));
    self.cron_tasks.push((name.to_string(), schedule, task));
    self
  }

  /// Consumes the App and returns the underlying Axum router and optionally a cron runner.
  /// When cron tasks are registered, the second element is `Some((db, runner))` so [`.serve`](Self::serve) can spawn them.
  /// For tests that only need the router, use `let (router, _) = app.into_router().await`.
  /// The returned router has state applied and is `Router<()>`, so it can be used with `into_make_service_with_connect_info`.
  pub async fn into_router(self) -> (Router<()>, Option<(DbConnection, CronRunner)>) {
    // Initialize tracing subscriber if not already initialized
    Self::init_tracing();

    let App {
      mut router,
      config,
      db: _,
      migrator,
      seeder,
      auth_installer,
      rate_limit_per_ip,
      rate_limit_per_user,
      token_lookup,
      cron_tasks,
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
    let db_conn = DbConnection::from(db_conn);

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

    // Install authentication if configured (and optional per-user rate limit and token auth)
    if let Some(installer) = auth_installer {
      info!("Installing authentication middleware...");
      router = installer(router, db_conn.clone(), rate_limit_per_user, token_lookup).await;
    }

    // Health endpoints: excluded from rate limiting (merged without the layer)
    let health_routes = Router::new()
      .route("/healthz", get(crate::health::healthz))
      .route("/livez", get(crate::health::livez))
      .route("/readyz", get(crate::health::readyz));

    router = if let Some(conf) = rate_limit_per_ip {
      let limited = router.layer(GovernorLayer::new(conf));
      limited.merge(health_routes)
    } else {
      router.merge(health_routes)
    };

    // OWASP-aligned security headers on all responses (after merge so health routes are included)
    router = router.layer(tower::util::MapResponseLayer::new(
      security_headers::add_security_headers,
    ));

    // OpenTelemetry: W3C trace context propagation and request spans (017).
    // Propagation layer innermost so traceparent is attached just before handlers (not overwritten by OtelAxumLayer).
    router = router
      .layer(observability::TraceContextPropagationLayer)
      .layer(OtelInResponseLayer)
      .layer(OtelAxumLayer::default());

    // Optional application cache: inject into request extensions for handlers (Option<Arc<AppCache>>)
    let app_cache = config
      .cache
      .as_ref()
      .and_then(cache::AppCache::from_config)
      .map(Arc::new);
    if app_cache.is_some() {
      info!("Application cache enabled");
    }
    let cache_ext = app_cache.clone();
    router = router.layer(tower::util::MapRequestLayer::new(
      move |mut req: axum::extract::Request| {
        req.extensions_mut().insert(cache_ext.clone());
        req
      },
    ));

    // Inject database connection into state
    let mut router = router.with_state(db_conn.clone());

    // HTTP response cache: cache full GET responses (layer wraps router)
    if let Some(cache_cfg) = config.cache.as_ref()
      && let Some(response_cache_layer) =
        cache_http_layer::HttpResponseCacheLayer::from_config(cache_cfg)
    {
      info!("HTTP response cache enabled");
      router = router.layer(response_cache_layer);
    }

    let cron_runner = if cron_tasks.is_empty() {
      None
    } else {
      Some((
        db_conn,
        CronRunner {
          tasks: cron_tasks,
          job_pool_url: config.database.url.clone(),
        },
      ))
    };
    (router, cron_runner)
  }

  /// Start the server and serve the application.
  pub async fn serve(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let host = self.config.server.host.clone();
    let port = self.config.server.port;
    let addr = format!("{}:{}", host, port);

    let (router, cron_runner) = self.into_router().await;
    if let Some((db, runner)) = cron_runner {
      runner.spawn(db);
    }

    match Self::bind_listener(&addr).await {
      Ok(listener) => {
        Self::log_server_start(&host, port);

        let maker = router.into_make_service_with_connect_info::<std::net::SocketAddr>();
        match axum::serve(listener, maker)
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

  /// Initialize the tracing subscriber for logging and OpenTelemetry (017).
  fn init_tracing() {
    observability::init_otel();
    let _ = tracing_subscriber::registry()
      .with(observability::otel_layer())
      .with(observability::env_filter())
      .with(tracing_subscriber::fmt::layer())
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

    #[tokio::test]
    async fn healthz_livez_readyz_return_200_minimal_body() {
      let temp_dir = tempfile::tempdir().unwrap();
      let original_cwd = std::env::current_dir().unwrap();
      std::env::set_current_dir(temp_dir.path()).unwrap();
      setup_test_config(temp_dir.path(), "health_test");

      let app = App::new();
      let (router, _) = app.into_router().await;
      let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
      let port = listener.local_addr().unwrap().port();
      let maker = router.into_make_service_with_connect_info::<std::net::SocketAddr>();
      tokio::spawn(async move {
        let _ = axum::serve(listener, maker).await;
      });

      tokio::time::sleep(std::time::Duration::from_millis(100)).await;

      let client = reqwest::Client::new();
      let base = format!("http://127.0.0.1:{}", port);
      for path in ["/healthz", "/livez", "/readyz"] {
        let resp = client
          .get(format!("{}{}", base, path))
          .send()
          .await
          .unwrap();
        assert!(resp.status().is_success(), "{}: {}", path, resp.status());
        let body = resp.text().await.unwrap();
        assert_eq!(body.trim(), "ok", "{} body: {:?}", path, body);
        assert!(
          !body.contains("components") && !body.contains("database"),
          "no component disclosure: {:?}",
          body
        );
      }

      std::env::set_current_dir(original_cwd).unwrap();
    }
  }
}
