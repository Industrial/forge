//! Forge App builder - the core of the web framework.

use std::sync::Arc;

use axum::{Router, handler::Handler};
use forge_auth::token_auth::{TokenAuthLayer, TokenLookupFn};
use forge_cache::HttpResponseCacheLayer;
use forge_config::ForgeConfig;
use forge_cron::{CronRunner, CronSchedule, CronTaskBox};
use forge_db::{DbConnection, initialize_database, wrap_traced};
use forge_health::{healthz, livez, readyz};
use forge_observability::{env_filter, init_otel, otel_layer};
use forge_rate_limit::RequesterOrgKeyExtractor;
use futures::future::BoxFuture;
use governor::middleware::NoOpMiddleware;
use sea_orm::{ConnectionTrait, DbBackend};
use sea_orm_migration::MigratorTrait;
use tokio::signal;
use tower::ServiceBuilder;
use tower_governor::{
  GovernorLayer,
  governor::{GovernorConfig, GovernorConfigBuilder},
  key_extractor::PeerIpKeyExtractor,
};
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::{info, warn};
use tracing_subscriber::prelude::*;

#[cfg(feature = "opentelemetry")]
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};

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
    + Send
    + Sync,
>;

/// Initialize the tracing subscriber for logging and OpenTelemetry (017).
/// Call this at the start of `main` when using `into_router_before_state()` so that
/// server start and other logs are visible.
pub fn init_tracing() {
  init_tracing_impl();
}

/// Internal helper that sets up OpenTelemetry and the tracing subscriber.
/// When [FORGE_SQL_DEBUG] is set, set `RUST_LOG=sqlx=debug` (e.g. in devenv) to see SQL statements.
fn init_tracing_impl() {
  init_otel();
  let _ = tracing_subscriber::registry()
    .with(otel_layer())
    .with(env_filter())
    .with(tracing_subscriber::fmt::layer())
    .try_init();
  let _ = tracing_log::LogTracer::init();
}

/// The main Forge application builder.
///
/// Provides a fluent API for configuring and running Axum-based web applications.
pub struct App {
  /// Whether the app is configured for token-only authentication.
  token_only_auth: bool,
  /// The Axum router containing all configured routes and middleware.
  router: Router<DbConnection>,
  /// Application configuration loaded from `config/app.toml` and `config/db.toml`
  config: ForgeConfig,
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
  /// Optional Live Query backend for real-time broadcast (e.g. [forge_live::InMemoryLiveBackend]).
  live_backend: Option<Arc<forge_live::InMemoryLiveBackend>>,
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

    let config = forge_config::load_config()?;
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
      migrator: None,
      seeder: None,
      auth_installer: None,
      rate_limit_per_ip: None,
      rate_limit_per_user: None,
      token_lookup: None,
      cron_tasks: Vec::new(),
      live_backend: None,
      token_only_auth: false,
    })
  }

  /// Returns a reference to the application configuration.
  pub fn config(&self) -> &ForgeConfig {
    &self.config
  }

  /// Enable Live Query: in-memory channel broadcast for real-time sync.
  /// Handlers can use [Extension]<Option<Arc<forge_live::InMemoryLiveBackend>>>
  /// and call [forge_live::broadcast_to_org] after mutations.
  pub fn with_live_query(mut self) -> Self {
    self.live_backend = Some(Arc::new(forge_live::InMemoryLiveBackend::new()));
    self
  }

  /// Enable Live Query using a shared backend (e.g. so the app and a tick loop can both use it).
  /// Handlers receive [Extension]<Option<Arc<forge_live::InMemoryLiveBackend>>>.
  pub fn with_live_query_using(mut self, backend: Arc<forge_live::InMemoryLiveBackend>) -> Self {
    self.live_backend = Some(backend);
    self
  }

  /// Enable per-requester (per user, per organization) rate limiting.
  /// Only has effect when [`.with_auth`](Self::with_auth) is also used. Limits are applied
  /// per (organization_id, user_id) so individual users in an org can be throttled.
  /// Unauthenticated requests share a single rate-limit bucket so register/login work.
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
        let config = forge_config::load_config()?;
        let db = initialize_database(&config.database).await?;
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

  /// Register authentication (token/Bearer only).
  pub fn with_auth<B, F>(mut self, backend_factory: F) -> Self
  where
    B: axum_login::AuthnBackend + Send + Sync + Clone + 'static,
    B::User: forge_auth::AuthzContext<RequesterId = uuid::Uuid, SubjectId = uuid::Uuid>
      + Send
      + Clone
      + 'static,
    B::User: axum_login::AuthUser<Id = uuid::Uuid>,
    F: Fn(DbConnection) -> B + Send + Sync + 'static,
  {
    self.auth_installer = Some(Box::new(
      move |router, db_conn, rate_limit_per_user, token_lookup| {
        Box::pin(async move {
          if db_conn.get_database_backend() != DbBackend::Sqlite {
            warn!(
              "Authentication currently only
            supported with Sqlite. Skipping auth setup."
            );
            return router;
          }

          let auth_backend = backend_factory(db_conn.clone());

          let router = if let Some(lookup) = token_lookup {
            router.layer(TokenAuthLayer::new(db_conn.clone(), auth_backend, lookup))
          } else {
            router
          };

          if let Some(rpm) = rate_limit_per_user {
            let mut builder =
              GovernorConfigBuilder::default().key_extractor(RequesterOrgKeyExtractor::<B>::new());
            builder.per_second(1).burst_size(rpm.max(1));
            let conf = builder.finish().expect("GovernorConfig per-user");
            router.layer(GovernorLayer::new(Arc::new(conf)))
          } else {
            router
          }
        })
      },
    ));
    self
  }

  /// Enables token (Bearer) authentication for the application.
  pub fn with_token_auth_only<B, F>(
    mut self,
    backend_factory: F,
    token_lookup: TokenLookupFn,
  ) -> Self
  where
    B: axum_login::AuthnBackend + Send + Sync + Clone + 'static,
    B::User: forge_auth::AuthzContext<RequesterId = uuid::Uuid, SubjectId = uuid::Uuid>
      + Send
      + Clone
      + 'static,
    B::User: axum_login::AuthUser<Id = uuid::Uuid>,
    F: Fn(DbConnection) -> B + Send + Sync + 'static,
  {
    self.token_only_auth = true;
    self.token_lookup = Some(token_lookup);
    self.with_auth(backend_factory)
  }

  /// Configures the application to include health check endpoints: /healthz, /livez, /readyz.
  /// These endpoints are excluded from IP-based rate limiting.
  pub fn with_health_routes(mut self) -> Self {
    let router = self
      .router
      .route("/healthz", axum::routing::get(healthz))
      .route("/livez", axum::routing::get(livez))
      .route("/readyz", axum::routing::get(readyz));
    self.router = router;
    self
  }

  /// Mounts a `Router` at the given path (GET).
  pub fn route<T, H>(mut self, path: &str, service: H) -> Self
  where
    H: Handler<T, DbConnection> + Clone + Send + 'static,
    T: Send + 'static,
  {
    self.router = self.router.route(path, axum::routing::get(service));
    self
  }

  /// Mounts a `Router` at the given path (POST).
  pub fn post_route<T, H>(mut self, path: &str, service: H) -> Self
  where
    H: Handler<T, DbConnection> + Clone + Send + 'static,
    T: Send + 'static,
  {
    self.router = self.router.route(path, axum::routing::post(service));
    self
  }

  /// Mounts a `Router` at the given path with a [axum::routing::MethodRouter] (e.g. `.get(h).post(p)`).
  pub fn route_methods(
    mut self,
    path: &str,
    method_router: axum::routing::MethodRouter<DbConnection>,
  ) -> Self {
    self.router = self.router.route(path, method_router);
    self
  }

  /// Mounts a `Router` at the given path for any HTTP method.
  /// When path is `"/"` or `""`, routes are merged (axum 0.8 does not allow nesting at root).
  pub fn nest(mut self, path: &str, router: Router<DbConnection>) -> Self {
    if path.is_empty() || path == "/" {
      self.router = self.router.merge(router);
    } else {
      self.router = self.router.nest(path, router);
    }
    self
  }

  /// Build the router with all configured middleware and services.
  async fn build_router_until_state(
    self,
  ) -> (
    Router<DbConnection>,
    DbConnection,
    Option<(DbConnection, CronRunner)>,
    Option<HttpResponseCacheLayer>,
  ) {
    let db_raw = initialize_database(&self.config.database)
      .await
      .unwrap_or_else(|e| {
        eprintln!("Error initializing database: {}", e);
        std::process::exit(1);
      });

    if let Some(migrator) = self.migrator {
      info!("Running migrations...");
      migrator().await.unwrap_or_else(|e| {
        eprintln!("Error running migrations: {}", e);
        std::process::exit(1);
      });
      info!("Migrations completed.");
    }

    let db_conn = wrap_traced(db_raw);

    if let Some(seeder) = self.seeder {
      info!("Running seed function...");
      seeder(db_conn.clone()).await.unwrap_or_else(|e| {
        eprintln!("Error running seed function: {}", e);
        std::process::exit(1);
      });
      info!("Seed function completed.");
    }

    let mut router = self.router;

    // OpenTelemetry layers (017)
    #[cfg(feature = "opentelemetry")]
    {
      router = router
        .layer(OtelAxumLayer::default())
        .layer(OtelInResponseLayer);
    }

    // IP-based rate limiting (Governor)
    if let Some(config) = self.rate_limit_per_ip {
      router = router.layer(GovernorLayer::new(config));
    }

    let auth_installer = self.auth_installer;
    let token_lookup = self.token_lookup;
    let rate_limit_per_user = self.rate_limit_per_user;

    if let Some(installer) = auth_installer {
      router = installer(router, db_conn.clone(), rate_limit_per_user, token_lookup).await;
    }

    let mut cron_tasks = self.cron_tasks;
    if let Some(live_backend) = self.live_backend {
      router = router.layer(axum::Extension(Some(live_backend)));
      cron_tasks.push((
        "forge-live-sweep".to_string(),
        CronSchedule::Interval(std::time::Duration::from_secs(60)),
        Box::new(|db_conn| {
          Box::pin(async move {
            forge_live::sweep_expired_connections(db_conn).await;
            Ok(())
          })
        }),
      ));
    }

    let response_cache_layer = self
      .config
      .cache
      .as_ref()
      .and_then(HttpResponseCacheLayer::from_config);

    let cron_runner = if cron_tasks.is_empty() {
      None
    } else {
      Some((
        db_conn.clone(),
        CronRunner {
          tasks: cron_tasks,
          job_pool_url: self.config.database.url.clone(),
        },
      ))
    };

    (router, db_conn, cron_runner, response_cache_layer)
  }

  /// Consumes the App and returns the underlying Axum router and optionally a cron runner.
  /// When cron tasks are registered, the second element is `Some((db, runner))` so [`.serve`](Self::serve) can spawn them.
  /// For tests that only need the router, use `let (router, _) = app.into_router().await`.\
  /// The returned router has state applied and is `Router<()>`, so it can be used with `into_make_service_with_connect_info`.
  pub async fn into_router(self) -> (Router<()>, Option<(DbConnection, CronRunner)>) {
    let (router, db_conn, cron_runner, response_cache_layer) =
      self.build_router_until_state().await;

    let mut router: Router<()> = router.with_state(db_conn);

    if let Some(layer) = response_cache_layer {
      router = router.layer(layer);
    }

    (router, cron_runner)
  }

  /// Returns the router **before** state and HTTP response cache are applied,
  /// so you can apply custom state (e.g. `(DbConnection, InertiaConfig)` for [Inertia](https://docs.rs/axum-inertia) apps).
  ///
  /// Use this when you need combined app state (e.g. DB + Inertia).
  /// Then apply state with `router.with_state((db_conn, inertia_config))`, then apply the returned cache layer if `Some`,
  /// then run the server (or use your own `serve` flow).
  ///
  /// # Example (Inertia)
  ///
  /// ```ignore
  /// let (router, db_conn, cron_runner, response_cache) = app.into_router_before_state().await;
  /// let inertia = build_inertia_config(&config); // vite dev or prod
  /// let router = router.with_state((db_conn.clone(), inertia));
  /// if let Some(cache) = response_cache {
  ///     router = router.layer(cache);
  /// }
  /// // run server with router; spawn cron_runner if desired
  /// ```
  pub async fn into_router_before_state(
    self,
  ) -> (
    Router<DbConnection>,
    DbConnection,
    Option<(DbConnection, CronRunner)>,
    Option<HttpResponseCacheLayer>,
  ) {
    self.build_router_until_state().await
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

  /// Log server start message.
  fn log_server_start(host: &str, port: u16) {
    info!("HTTP server listening on http://{}:{}", host, port);
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
    shutdown_signal_future().await;
  }
}

/// Public shutdown signal for custom server loops (e.g. Inertia apps using `into_router_before_state`).
/// Completes when Ctrl+C or SIGTERM is received.
pub async fn shutdown_signal_future() {
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

impl Default for App {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use axum::extract::State;
  use axum::response::Html;
  use std::fs;
  use std::path::Path;

  fn setup_test_config(dir: &Path, project_name: &str) {
    let config_dir = dir.join("config");
    fs::create_dir_all(&config_dir).unwrap();
    let app_toml_content = format!(
      r#"[app]
name = "{}"

[server]
host = "127.0.0.1"
port = 3000

[frontend]
port = 3000
"#,
      project_name
    );
    fs::write(config_dir.join("app.toml"), app_toml_content).unwrap();

    let db_toml_content = r#"[database]
url = "sqlite::memory:"
auto_migrate = true
auto_seed = false
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

  async fn test_handler_hello(_: State<DbConnection>) -> &'static str {
    "Hello"
  }

  async fn test_handler_api(_: State<DbConnection>) -> Html<&'static str> {
    Html("<h1>API</h1>")
  }

  mod route_registration {
    use super::*;

    #[test]
    fn route_method_accepts_static_handler() {
      let temp_dir = tempfile::tempdir().unwrap();
      let original_cwd = std::env::current_dir().unwrap();
      std::env::set_current_dir(temp_dir.path()).unwrap();
      setup_test_config(temp_dir.path(), "test_app");

      let router = Router::new().route("/", axum::routing::get(test_handler_hello));
      let app = App::new().nest("/", router);
      let _ = app;

      std::env::set_current_dir(original_cwd).unwrap();
    }

    #[test]
    fn route_method_accepts_complex_handler() {
      let temp_dir = tempfile::tempdir().unwrap();
      let original_cwd = std::env::current_dir().unwrap();
      std::env::set_current_dir(temp_dir.path()).unwrap();
      setup_test_config(temp_dir.path(), "test_app");

      let router = Router::new().route("/api", axum::routing::get(test_handler_api));
      let app = App::new().nest("/", router);
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

[server]
host = "127.0.0.1"
port = 0

[frontend]
port = 3000
"#;
      fs::write(config_dir.join("app.toml"), app_toml_content).unwrap();

      let db_toml_content = r#"[database]
url = "sqlite::memory:"
auto_migrate = true
auto_seed = false
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

      let app = App::new().with_health_routes();
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
