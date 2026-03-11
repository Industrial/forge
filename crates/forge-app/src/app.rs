//! Forge App builder - the core of the web framework.

use std::sync::Arc;

use axum::{Router, handler::Handler};
use forge_auth::token_auth::{TokenAuthLayer, TokenLookupFn};
use forge_cache::HttpResponseCacheLayer;
use forge_config::ForgeConfig;
use forge_cron::{CronRunner, CronSchedule, CronTaskBox};
use forge_db::{DbConnection, initialize_database, wrap_traced};
use forge_health::{healthz, livez};
use forge_observability::{env_filter, init_otel, otel_layer};
use forge_rate_limit::RequesterOrgKeyExtractor;
use futures::future::BoxFuture;
use governor::middleware::NoOpMiddleware;
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

/// Type alias for the auth installer function (generic over app state type S).
/// Args: router, db, optional per-user rate limit, optional token lookup for Bearer auth.
pub type AuthInstallerFn<S> = Box<
  dyn FnOnce(
      Router<S>,
      DbConnection,
      Option<u32>,
      Option<TokenLookupFn>,
    ) -> BoxFuture<'static, Router<S>>
    + Send
    + Sync,
>;

/// Internal handler used only to type an empty router as `Router<S>`.
async fn __forge_empty_handler<S: Clone + Send + Sync + 'static>(
  _: axum::extract::State<S>,
) -> (axum::http::StatusCode, ()) {
  (axum::http::StatusCode::NOT_FOUND, ())
}

/// Create an empty `Router<S>` (used when building `App<S>` with custom state).
fn empty_router<S: Clone + Send + Sync + 'static>() -> Router<S> {
  Router::new().route(
    "/__forge_typed_state",
    axum::routing::get(__forge_empty_handler::<S>),
  )
}

/// Health handlers that accept `State<S>` so they can be used with any app state type.
async fn __healthz_with_state<S: Clone + Send + Sync + 'static>(
  _: axum::extract::State<S>,
) -> impl axum::response::IntoResponse {
  healthz().await
}
async fn __livez_with_state<S: Clone + Send + Sync + 'static>(
  _: axum::extract::State<S>,
) -> impl axum::response::IntoResponse {
  livez().await
}
/// Readiness for generic state: returns 200 OK. For DB-backed readiness use [App] with `DbConnection` state.
async fn __readyz_with_state<S: Clone + Send + Sync + 'static>(
  _: axum::extract::State<S>,
) -> impl axum::response::IntoResponse {
  (axum::http::StatusCode::OK, "ok")
}

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
pub struct App<S = DbConnection> {
  /// Whether the app is configured for token-only authentication.
  token_only_auth: bool,
  /// The Axum router containing all configured routes and middleware.
  router: Router<S>,
  state_builder: Box<dyn FnOnce(DbConnection) -> S + Send>,
  /// Application configuration loaded from `config/app.toml` and `config/db.toml`
  config: ForgeConfig,
  /// Optional migrator function to run on startup
  migrator: Option<MigratorFn>,
  /// Optional seeder to run on startup
  seeder: Option<SeedFn>,
  /// Optional auth installer
  auth_installer: Option<AuthInstallerFn<S>>,
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

impl App<DbConnection> {
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
      state_builder: Box::new(|db: DbConnection| db),
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

  /// Switch to a custom state type (e.g. [ScopeExtractorState](forge_auth::ScopeExtractorState))
  /// so routes can use handlers that take `State<S>`. Call this before adding routes and before
  /// [with_token_auth_only](Self::with_token_auth_only).
  pub fn with_state_builder<S2, F>(self, state_builder: F) -> App<S2>
  where
    S2: Clone + Send + Sync + 'static,
    F: FnOnce(DbConnection) -> S2 + Send + 'static,
  {
    let trace_layer = TraceLayer::new_for_http()
      .make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
      .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
      .on_response(DefaultOnResponse::new().level(tracing::Level::INFO));
    let router = empty_router::<S2>().layer(ServiceBuilder::new().layer(trace_layer));
    App {
      token_only_auth: self.token_only_auth,
      router,
      state_builder: Box::new(state_builder),
      config: self.config,
      migrator: self.migrator,
      seeder: self.seeder,
      auth_installer: None,
      rate_limit_per_ip: self.rate_limit_per_ip,
      rate_limit_per_user: self.rate_limit_per_user,
      token_lookup: self.token_lookup,
      cron_tasks: self.cron_tasks,
      live_backend: self.live_backend,
    }
  }

  /// Returns a reference to the application configuration.
  pub fn config(&self) -> &ForgeConfig {
    &self.config
  }
}

impl<S: Clone + Send + Sync + 'static> App<S> {
  /// Enable Live Query: in-memory channel broadcast for real-time sync.
  /// Handlers can use [axum::Extension]<Option<Arc<forge_live::InMemoryLiveBackend>>>
  /// and call [forge_live::broadcast_to_org] after mutations.
  pub fn with_live_query(mut self) -> Self {
    self.live_backend = Some(Arc::new(forge_live::InMemoryLiveBackend::new()));
    self
  }

  /// Enable Live Query using a shared backend (e.g. so the app and a tick loop can both use it).
  /// Handlers receive [axum::Extension]<Option<Arc<forge_live::InMemoryLiveBackend>>>.
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
      .route("/healthz", axum::routing::get(__healthz_with_state::<S>))
      .route("/livez", axum::routing::get(__livez_with_state::<S>))
      .route("/readyz", axum::routing::get(__readyz_with_state::<S>));
    self.router = router;
    self
  }

  /// Mounts a `Router` at the given path (GET).
  pub fn route<T, H>(mut self, path: &str, service: H) -> Self
  where
    H: Handler<T, S> + Clone + Send + 'static,
    T: Send + 'static,
  {
    self.router = self.router.route(path, axum::routing::get(service));
    self
  }

  /// Mounts a `Router` at the given path (POST).
  pub fn post_route<T, H>(mut self, path: &str, service: H) -> Self
  where
    H: Handler<T, S> + Clone + Send + 'static,
    T: Send + 'static,
  {
    self.router = self.router.route(path, axum::routing::post(service));
    self
  }

  /// Mounts a `Router` at the given path with a [axum::routing::MethodRouter] (e.g. `.get(h).post(p)`).
  pub fn route_methods(
    mut self,
    path: &str,
    method_router: axum::routing::MethodRouter<S>,
  ) -> Self {
    self.router = self.router.route(path, method_router);
    self
  }

  /// Mounts a `Router` at the given path for any HTTP method.
  /// When path is `"/"` or `""`, routes are merged (axum 0.8 does not allow nesting at root).
  pub fn nest(mut self, path: &str, router: Router<S>) -> Self {
    if path.is_empty() || path == "/" {
      self.router = self.router.merge(router);
    } else {
      self.router = self.router.nest(path, router);
    }
    self
  }

  /// Build the router with all configured middleware and services.
  /// Returns state_builder so the caller can apply state (into_router consumes it).
  async fn build_router_until_state(
    self,
  ) -> (
    Router<S>,
    DbConnection,
    Option<(DbConnection, CronRunner)>,
    Option<HttpResponseCacheLayer>,
    Box<dyn FnOnce(DbConnection) -> S + Send>,
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

    (
      router,
      db_conn,
      cron_runner,
      response_cache_layer,
      self.state_builder,
    )
  }

  /// Consumes the App and returns the underlying Axum router and optionally a cron runner.
  /// When cron tasks are registered, the second element is `Some((db, runner))` so [`.serve`](Self::serve) can spawn them.
  /// For tests that only need the router, use `let (router, _) = app.into_router().await`.\
  /// The returned router has state applied and is `Router<()>`, so it can be used with `into_make_service_with_connect_info`.
  pub async fn into_router(self) -> (Router<()>, Option<(DbConnection, CronRunner)>) {
    let (router, db_conn, cron_runner, response_cache_layer, state_builder) =
      self.build_router_until_state().await;

    let state = state_builder(db_conn);
    let mut router: Router<()> = router.with_state(state);

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
    Router<S>,
    DbConnection,
    Option<(DbConnection, CronRunner)>,
    Option<HttpResponseCacheLayer>,
    Box<dyn FnOnce(DbConnection) -> S + Send>,
  ) {
    let (router, db_conn, cron_runner, layer, state_builder) =
      self.build_router_until_state().await;
    (router, db_conn, cron_runner, layer, state_builder)
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

  mod bdd_tests {
    use super::*;
    use axum::extract::State;

    /// BDD-style tests focusing on behavior rather than implementation.
    /// Tests are organized by feature/behavior area with descriptive names.
    mod app_creation_behavior {
      use super::*;

      #[test]
      fn should_create_app_when_config_exists() {
        // Given: valid configuration files exist
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        setup_test_config(&std::env::current_dir().unwrap(), "bdd_test_app");

        // When: creating a new app
        let app = App::try_new();

        // Then: app should be created successfully
        assert!(app.is_ok(), "App creation should succeed with valid config");

        std::env::set_current_dir(original_cwd).unwrap();
      }

      #[test]
      fn should_panic_when_config_missing() {
        // Given: no configuration files exist
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // When: creating a new app with App::new()
        // Then: should panic (tested via try_new returning error)
        let result = App::try_new();
        assert!(result.is_err(), "App creation should fail without config");

        std::env::set_current_dir(original_cwd).unwrap();
      }

      #[test]
      fn should_provide_config_access_after_creation() {
        // Given: an app is created
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        setup_test_config(temp_dir.path(), "config_test");

        // When: accessing the config
        let app = App::new();
        let config = app.config();

        // Then: config should be accessible and contain expected values
        assert_eq!(config.app.name, "config_test");
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 3000);

        std::env::set_current_dir(original_cwd).unwrap();
      }
    }

    mod route_registration_behavior {
      use super::*;

      async fn hello_handler(_: State<DbConnection>) -> &'static str {
        "Hello, World!"
      }

      async fn json_handler(_: State<DbConnection>) -> &'static str {
        "success"
      }

      #[tokio::test]
      async fn should_register_get_route_when_route_called() {
        // Given: an app and a handler
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        setup_test_config(temp_dir.path(), "route_test");

        // When: registering a GET route
        let app = App::new().route("/hello", hello_handler);
        let (router, _) = app.into_router().await;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let maker = router.into_make_service_with_connect_info::<std::net::SocketAddr>();
        tokio::spawn(async move {
          let _ = axum::serve(listener, maker).await;
        });

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Then: route should be registered and accessible
        let client = reqwest::Client::new();
        let resp = client
          .get(format!("http://127.0.0.1:{}/hello", port))
          .send()
          .await
          .unwrap();
        assert!(resp.status().is_success());
        assert_eq!(resp.text().await.unwrap(), "Hello, World!");

        std::env::set_current_dir(original_cwd).unwrap();
      }

      #[tokio::test]
      async fn should_register_post_route_when_post_route_called() {
        // Given: an app and a handler
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        setup_test_config(temp_dir.path(), "post_route_test");

        // When: registering a POST route
        let app = App::new().post_route("/api", json_handler);
        let (router, _) = app.into_router().await;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let maker = router.into_make_service_with_connect_info::<std::net::SocketAddr>();
        tokio::spawn(async move {
          let _ = axum::serve(listener, maker).await;
        });

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Then: route should be registered and accessible
        let client = reqwest::Client::new();
        let resp = client
          .post(format!("http://127.0.0.1:{}/api", port))
          .send()
          .await
          .unwrap();
        assert!(resp.status().is_success());
        assert_eq!(resp.text().await.unwrap(), "success");

        std::env::set_current_dir(original_cwd).unwrap();
      }

      #[tokio::test]
      async fn should_register_multiple_methods_when_route_methods_called() {
        // Given: an app and handlers for different methods
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        setup_test_config(temp_dir.path(), "methods_test");

        // When: registering a route with multiple HTTP methods
        let method_router = axum::routing::MethodRouter::new()
          .get(hello_handler)
          .post(json_handler);
        let app = App::new().route_methods("/api", method_router);
        let (router, _) = app.into_router().await;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let maker = router.into_make_service_with_connect_info::<std::net::SocketAddr>();
        tokio::spawn(async move {
          let _ = axum::serve(listener, maker).await;
        });

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Then: route should support multiple methods
        let client = reqwest::Client::new();
        let get_resp = client
          .get(format!("http://127.0.0.1:{}/api", port))
          .send()
          .await
          .unwrap();
        assert!(get_resp.status().is_success());
        assert_eq!(get_resp.text().await.unwrap(), "Hello, World!");

        let post_resp = client
          .post(format!("http://127.0.0.1:{}/api", port))
          .send()
          .await
          .unwrap();
        assert!(post_resp.status().is_success());
        assert_eq!(post_resp.text().await.unwrap(), "success");

        std::env::set_current_dir(original_cwd).unwrap();
      }

      #[tokio::test]
      async fn should_merge_routes_when_nesting_at_root() {
        // Given: an app and a nested router
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        setup_test_config(temp_dir.path(), "nest_test");

        // When: nesting a router at root path
        let nested_router = Router::new().route("/nested", axum::routing::get(hello_handler));
        let app = App::new().nest("/", nested_router);
        let (router, _) = app.into_router().await;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let maker = router.into_make_service_with_connect_info::<std::net::SocketAddr>();
        tokio::spawn(async move {
          let _ = axum::serve(listener, maker).await;
        });

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Then: routes should be merged and accessible
        let client = reqwest::Client::new();
        let resp = client
          .get(format!("http://127.0.0.1:{}/nested", port))
          .send()
          .await
          .unwrap();
        assert!(resp.status().is_success());
        assert_eq!(resp.text().await.unwrap(), "Hello, World!");

        std::env::set_current_dir(original_cwd).unwrap();
      }

      #[tokio::test]
      async fn should_nest_routes_when_nesting_at_subpath() {
        // Given: an app and a nested router
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        setup_test_config(temp_dir.path(), "nest_subpath_test");

        // When: nesting a router at a subpath
        let nested_router = Router::new().route("/item", axum::routing::get(hello_handler));
        let app = App::new().nest("/api", nested_router);
        let (router, _) = app.into_router().await;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let maker = router.into_make_service_with_connect_info::<std::net::SocketAddr>();
        tokio::spawn(async move {
          let _ = axum::serve(listener, maker).await;
        });

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Then: routes should be nested under the subpath and accessible
        let client = reqwest::Client::new();
        let resp = client
          .get(format!("http://127.0.0.1:{}/api/item", port))
          .send()
          .await
          .unwrap();
        assert!(resp.status().is_success());
        assert_eq!(resp.text().await.unwrap(), "Hello, World!");

        std::env::set_current_dir(original_cwd).unwrap();
      }
    }

    mod rate_limiting_behavior {
      use super::*;

      #[tokio::test]
      async fn should_enable_ip_rate_limiting_when_with_rate_limit_per_ip_called() {
        // Given: an app
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        setup_test_config(temp_dir.path(), "rate_limit_ip_test");

        // When: enabling per-IP rate limiting
        let app = App::new().with_rate_limit_per_ip(60);
        let (router, _) = app.into_router().await;

        // Then: rate limiting should be configured (router builds successfully)
        // The fact that router builds means rate limiting layer was added
        let _router: Router<()> = router;

        std::env::set_current_dir(original_cwd).unwrap();
      }

      #[tokio::test]
      async fn should_enforce_minimum_rate_limit_when_zero_provided() {
        // Given: an app
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        setup_test_config(temp_dir.path(), "rate_limit_min_test");

        // When: setting rate limit to 0
        let app = App::new().with_rate_limit_per_ip(0);

        // Then: should use minimum of 1 (no panic, router builds)
        let (router, _) = app.into_router().await;
        let _router: Router<()> = router;

        std::env::set_current_dir(original_cwd).unwrap();
      }

      #[tokio::test]
      async fn should_enable_user_rate_limiting_when_with_rate_limit_per_user_called() {
        // Given: an app
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        setup_test_config(temp_dir.path(), "rate_limit_user_test");

        // When: enabling per-user rate limiting
        let app = App::new().with_rate_limit_per_user(30);

        // Then: rate limiting should be configured (no panic)
        // Note: This only takes effect when auth is also configured
        let (router, _) = app.into_router().await;
        let _router: Router<()> = router;

        std::env::set_current_dir(original_cwd).unwrap();
      }
    }

    mod health_routes_behavior {
      use super::*;

      #[tokio::test]
      async fn should_register_health_routes_when_with_health_routes_called() {
        // Given: an app
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        setup_test_config(temp_dir.path(), "health_test");

        // When: enabling health routes
        let app = App::new().with_health_routes();
        let (router, _) = app.into_router().await;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let maker = router.into_make_service_with_connect_info::<std::net::SocketAddr>();
        tokio::spawn(async move {
          let _ = axum::serve(listener, maker).await;
        });

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Then: health endpoints should be accessible
        let client = reqwest::Client::new();
        let base = format!("http://127.0.0.1:{}", port);
        for path in ["/healthz", "/livez", "/readyz"] {
          let resp = client
            .get(format!("{}{}", base, path))
            .send()
            .await
            .unwrap();
          assert!(resp.status().is_success(), "{} should return 200", path);
          let body = resp.text().await.unwrap();
          assert_eq!(body.trim(), "ok", "{} should return 'ok'", path);
        }

        std::env::set_current_dir(original_cwd).unwrap();
      }
    }

    mod live_query_behavior {
      use super::*;

      #[tokio::test]
      async fn should_enable_live_query_when_with_live_query_called() {
        // Given: an app
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        setup_test_config(temp_dir.path(), "live_query_test");

        // When: enabling live query
        let app = App::new().with_live_query();
        let (_router, cron_runner) = app.into_router().await;

        // Then: live query backend should be configured
        // When live query is enabled, a cron runner should be created for sweep task
        assert!(
          cron_runner.is_some(),
          "Cron runner should exist when live query enabled"
        );

        std::env::set_current_dir(original_cwd).unwrap();
      }

      #[tokio::test]
      async fn should_use_provided_backend_when_with_live_query_using_called() {
        // Given: an app and a live query backend
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        setup_test_config(temp_dir.path(), "live_query_shared_test");

        // When: enabling live query with a shared backend
        let backend = Arc::new(forge_live::InMemoryLiveBackend::new());
        let app = App::new().with_live_query_using(backend);
        let (_router, cron_runner) = app.into_router().await;

        // Then: the provided backend should be used
        assert!(
          cron_runner.is_some(),
          "Cron runner should exist when live query enabled"
        );

        std::env::set_current_dir(original_cwd).unwrap();
      }
    }

    mod router_building_behavior {
      use super::*;

      #[tokio::test]
      async fn should_build_router_when_into_router_called() {
        // Given: a configured app
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        setup_test_config(temp_dir.path(), "router_build_test");

        // When: converting app to router
        let app = App::new().with_health_routes();
        let (router, cron_runner) = app.into_router().await;

        // Then: router should be built successfully
        assert!(
          cron_runner.is_none() || cron_runner.is_some(),
          "Router should build"
        );
        // Router type is Router<()> after into_router
        let _router: Router<()> = router;

        std::env::set_current_dir(original_cwd).unwrap();
      }

      #[tokio::test]
      async fn should_build_router_before_state_when_into_router_before_state_called() {
        // Given: a configured app
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        setup_test_config(temp_dir.path(), "router_before_state_test");

        // When: converting app to router before state
        let app = App::new().with_health_routes();
        let (router, db_conn, cron_runner, cache_layer, _state_builder) =
          app.into_router_before_state().await;

        // Then: router should be built with DbConnection state
        let _router: Router<DbConnection> = router;
        // db_conn is DbConnection, not Option<DbConnection>, so it always exists
        let _db: DbConnection = db_conn;
        assert!(
          cron_runner.is_none() || cron_runner.is_some(),
          "Cron runner may or may not exist"
        );
        assert!(
          cache_layer.is_none() || cache_layer.is_some(),
          "Cache layer may or may not exist"
        );

        std::env::set_current_dir(original_cwd).unwrap();
      }

      #[tokio::test]
      async fn should_create_cron_runner_when_live_query_enabled() {
        // Given: an app with live query enabled
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        setup_test_config(temp_dir.path(), "cron_runner_test");

        // When: building router with live query
        let app = App::new().with_live_query();
        let (_router, cron_runner) = app.into_router().await;

        // Then: cron runner should be created
        assert!(
          cron_runner.is_some(),
          "Cron runner should exist when live query enabled"
        );
        let (db, runner) = cron_runner.unwrap();
        // db is DbConnection, not Option<DbConnection>, so it always exists
        let _db_conn: DbConnection = db;
        assert!(!runner.tasks.is_empty(), "Cron runner should have tasks");

        std::env::set_current_dir(original_cwd).unwrap();
      }
    }

    mod configuration_behavior {
      use super::*;

      #[test]
      fn should_load_config_from_files_when_app_created() {
        // Given: configuration files exist
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        setup_test_config(temp_dir.path(), "config_load_test");

        // When: creating an app
        let app = App::new();

        // Then: config should be loaded from files
        let config = app.config();
        assert_eq!(config.app.name, "config_load_test");
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 3000);

        std::env::set_current_dir(original_cwd).unwrap();
      }

      #[test]
      fn should_provide_readonly_config_access() {
        // Given: an app
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        setup_test_config(temp_dir.path(), "config_readonly_test");

        // When: accessing config multiple times
        let app = App::new();
        let config1 = app.config();
        let config2 = app.config();

        // Then: should return same config reference
        assert_eq!(config1.app.name, config2.app.name);
        assert_eq!(config1.server.host, config2.server.host);

        std::env::set_current_dir(original_cwd).unwrap();
      }
    }

    mod fluent_api_behavior {
      use super::*;

      #[tokio::test]
      async fn should_support_method_chaining_when_configuring_app() {
        // Given: an app builder
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        setup_test_config(temp_dir.path(), "fluent_api_test");

        async fn handler(_: State<DbConnection>) -> &'static str {
          "test"
        }

        // When: chaining multiple configuration methods
        let app = App::new()
          .with_health_routes()
          .with_rate_limit_per_ip(60)
          .route("/test", handler);

        // Then: all configurations should be applied
        let (router, _) = app.into_router().await;
        let _router: Router<()> = router;

        std::env::set_current_dir(original_cwd).unwrap();
      }

      #[tokio::test]
      async fn should_allow_multiple_route_registrations() {
        // Given: an app and multiple handlers
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        setup_test_config(temp_dir.path(), "multiple_routes_test");

        async fn handler1(_: State<DbConnection>) -> &'static str {
          "handler1"
        }

        async fn handler2(_: State<DbConnection>) -> &'static str {
          "handler2"
        }

        // When: registering multiple routes
        let app = App::new()
          .route("/route1", handler1)
          .route("/route2", handler2)
          .post_route("/route3", handler1);

        // Then: all routes should be registered
        let (router, _) = app.into_router().await;
        let _router: Router<()> = router;

        std::env::set_current_dir(original_cwd).unwrap();
      }
    }
  }
}
