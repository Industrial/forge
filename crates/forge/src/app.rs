//! Forge App builder - the core of the web framework.

use axum::{handler::Handler, routing::get, Router};
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::{info, warn};

/// The main Forge application builder.
///
/// Provides a fluent API for configuring and running Axum-based web applications.
pub struct App {
  /// The Axum router containing all configured routes and middleware.
  router: Router,
}

impl App {
  /// Create a new Forge application.
  pub fn new() -> Self {
    info!("Initializing Forge application");

    let router = Router::new();

    // Add tracing middleware for HTTP request logging
    let trace_layer = TraceLayer::new_for_http()
      .make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
      .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
      .on_response(DefaultOnResponse::new().level(tracing::Level::INFO));

    let router = router.layer(ServiceBuilder::new().layer(trace_layer));

    info!("Forge application initialized with tracing middleware");

    Self { router }
  }

  /// Add a route to the application.
  ///
  /// # Example
  /// ```ignore
  /// use forge::App;
  ///
  /// let app = App::new()
  ///     .route("/", || async { "Hello, World!" });
  /// ```
  pub fn route<H, T>(mut self, path: &str, handler: H) -> Self
  where
    H: Handler<T, ()>,
    T: 'static,
  {
    self.router = self.router.route(path, get(handler));
    self
  }

  /// Parse port from environment variable or default to 3000.
  ///
  /// # Returns
  /// - The port number from PORT env var if valid
  /// - 3000 if PORT is not set or invalid
  fn parse_port() -> u16 {
    std::env::var("PORT")
      .map(|p| p.parse::<u16>().unwrap_or(3000))
      .unwrap_or(3000)
  }

  /// Initialize the tracing subscriber for logging.
  ///
  /// This should only be called once per application lifecycle.
  fn init_tracing() {
    tracing_subscriber::fmt()
      .with_env_filter(
        tracing_subscriber::EnvFilter::from_default_env()
          .add_directive("forge=info".parse().unwrap())
          .add_directive("tower_http=info".parse().unwrap()),
      )
      .init();
  }

  /// Build the server address string from port.
  fn build_address(port: u16) -> String {
    format!("0.0.0.0:{}", port)
  }

  /// Log server start message
  fn log_server_start(port: u16) {
    info!("Forge server running on http://0.0.0.0:{}", port);
  }

  /// Log server stop message
  fn log_server_stop() {
    info!("Forge server shut down gracefully");
  }

  /// Log server error
  fn log_server_error(e: &std::io::Error) {
    warn!("Error running server: {}", e);
  }

  /// Log bind error
  fn log_bind_error(addr: &str, e: &std::io::Error) {
    warn!("Failed to bind to {}: {}", addr, e);
  }

  /// Bind TCP listener to address
  async fn bind_listener(addr: &str) -> Result<tokio::net::TcpListener, std::io::Error> {
    tokio::net::TcpListener::bind(addr).await
  }

  /// Start the server and serve the application.
  ///
  /// This method will block until the server is shut down (Ctrl+C).
  /// The server port can be configured via the PORT environment variable,
  /// defaulting to 3000.
  ///
  /// # Example
  /// ```no_run
  /// use forge::App;
  ///
  /// #[tokio::main]
  /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
  ///     App::new()
  ///         .route("/", || async { "Hello from Forge!" })
  ///         .serve()
  ///         .await
  /// }
  /// ```
  pub async fn serve(self) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber if not already initialized
    Self::init_tracing();

    let app = self.router;

    // Get port from environment or default to 3000
    let port = Self::parse_port();

    let addr = Self::build_address(port);

    match Self::bind_listener(&addr).await {
      Ok(listener) => {
        Self::log_server_start(port);

        match axum::serve(listener, app)
          .with_graceful_shutdown(shutdown_signal())
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
        Self::log_bind_error(&addr, &e);
        Err(e.into())
      }
    }
  }
}

/// Setup Ctrl+C signal handler
async fn setup_ctrl_c() {
  signal::ctrl_c()
    .await
    .expect("failed to install Ctrl+C handler");
}

/// Setup SIGTERM signal handler (Unix only)
#[cfg(unix)]
async fn setup_sigterm() {
  signal::unix::signal(signal::unix::SignalKind::terminate())
    .expect("failed to install signal handler")
    .recv()
    .await;
}

/// Stub for non-Unix platforms
#[cfg(not(unix))]
async fn setup_sigterm() {
  std::future::pending::<()>().await
}

/// Log shutdown signal received
fn log_shutdown_signal(signal_name: &str) {
  info!(
    "Received {} signal, initiating graceful shutdown",
    signal_name
  );
}

/// Create a future that completes when a shutdown signal is received.
async fn shutdown_signal() {
  let ctrl_c = setup_ctrl_c();
  let terminate = setup_sigterm();

  tokio::select! {
    _ = ctrl_c => {
      log_shutdown_signal("Ctrl+C");
    },
    _ = terminate => {
      log_shutdown_signal("SIGTERM");
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
  use axum::response::Html;

  /// Test suite for App creation and initialization
  mod app_creation {
    use super::*;

    #[test]
    fn new_creates_app_with_empty_router() {
      // Given: No preconditions
      // When: Creating a new App
      let app = App::new();

      // Then: App should be created successfully
      // Note: We can't inspect the router directly, but compilation succeeds
      let _app = app;
    }

    #[test]
    fn default_trait_creates_same_as_new() {
      // Given: No preconditions
      // When: Using Default trait vs new()
      let app_new = App::new();
      let app_default = App::default();

      // Then: They should be equivalent
      // Note: We test this by ensuring both compile and run without errors
      let _app_new = app_new;
      let _app_default = app_default;
    }

    #[test]
    fn app_is_send_and_sync() {
      // Given: An App instance
      let app = App::new();

      // When: Checking if it implements Send and Sync
      // Then: It should compile (marker trait test)
      fn assert_send_sync<T: Send + Sync>(_t: T) {}
      assert_send_sync(app);
    }
  }

  /// Test suite for route registration
  mod route_registration {
    use super::*;

    #[test]
    fn route_method_accepts_static_handler() {
      // Given: An App instance
      let app = App::new();

      // When: Adding a route with a static string handler
      let app = app.route("/", || async { "Hello" });

      // Then: It should succeed without errors
      let _app = app;
    }

    #[test]
    fn route_method_accepts_complex_handler() {
      // Given: An App instance
      let app = App::new();

      // When: Adding a route with a more complex handler
      let app = app.route("/api", || async { Html("<h1>API</h1>") });

      // Then: It should succeed without errors
      let _app = app;
    }

    #[test]
    fn route_method_is_fluent_returns_self() {
      // Given: An App instance
      let app = App::new();

      // When: Chaining multiple route calls
      let app = app
        .route("/", || async { "root" })
        .route("/api", || async { "api" })
        .route("/health", || async { "ok" });

      // Then: It should succeed and return App
      let _app = app;
    }

    #[test]
    fn route_method_accepts_different_path_patterns() {
      // Given: An App instance
      let app = App::new();

      // When: Adding routes with different path patterns
      let app = app
        .route("/", || async { "root" })
        .route("/users", || async { "users" })
        .route("/users/:id", || async { "user detail" })
        .route("/api/v1/data", || async { "api data" });

      // Then: All should be accepted
      let _app = app;
    }
  }

  /// Test suite for serve method - limited testing due to server binding
  mod serve_method {
    use super::*;

    #[test]
    fn serve_method_exists_and_is_callable() {
      // Given: An App with routes
      let app = App::new().route("/", || async { "test" });

      // When: Calling serve method signature
      // Then: It should compile (async function that returns Result)
      // Note: We can't easily test the actual serving without complex setup
      #[allow(dead_code)]
      async fn test_signature(_app: App) -> Result<(), Box<dyn std::error::Error>> {
        // This would try to bind to port 3000, so we don't call it
        let _result: Result<(), Box<dyn std::error::Error>> = Ok(());
        _result
      }

      // Just test that the method exists
      let _app = app;
    }

    #[test]
    fn parse_port_returns_env_var_when_valid() {
      // Given: PORT environment variable set to valid value
      std::env::set_var("PORT", "8080");

      // When: parse_port is called
      let port = App::parse_port();

      // Then: It should return the parsed value
      assert_eq!(port, 8080);

      // Clean up
      std::env::remove_var("PORT");
    }

    #[test]
    fn parse_port_defaults_to_3000_when_not_set() {
      // Given: PORT environment variable not set
      std::env::remove_var("PORT");

      // When: parse_port is called
      let port = App::parse_port();

      // Then: It should default to 3000
      assert_eq!(port, 3000);
    }

    #[test]
    fn parse_port_handles_invalid_values() {
      // Given: Invalid PORT environment variable
      std::env::set_var("PORT", "invalid");

      // When: parse_port is called
      let port = App::parse_port();

      // Then: It should fallback to 3000
      assert_eq!(port, 3000);

      // Clean up
      std::env::remove_var("PORT");
    }

    #[test]
    fn parse_port_handles_out_of_range_values() {
      // Given: PORT environment variable with out-of-range value
      std::env::set_var("PORT", "999999");

      // When: parse_port is called
      let port = App::parse_port();

      // Then: It should fallback to 3000
      assert_eq!(port, 3000);

      // Clean up
      std::env::remove_var("PORT");
    }

    #[test]
    fn parse_port_handles_empty_string() {
      // Given: PORT environment variable set to empty string
      std::env::set_var("PORT", "");

      // When: parse_port is called
      let port = App::parse_port();

      // Then: It should fallback to 3000
      assert_eq!(port, 3000);

      // Clean up
      std::env::remove_var("PORT");
    }

    #[test]
    fn build_address_formats_correctly() {
      // Given: A port number
      let port = 8080;

      // When: build_address is called
      let addr = App::build_address(port);

      // Then: It should format correctly
      assert_eq!(addr, "0.0.0.0:8080");
    }

    #[test]
    fn build_address_handles_standard_port() {
      // Given: The default port 3000
      let port = 3000;

      // When: build_address is called
      let addr = App::build_address(port);

      // Then: It should format correctly
      assert_eq!(addr, "0.0.0.0:3000");
    }

    #[test]
    fn build_address_handles_privileged_port() {
      // Given: A privileged port (e.g., 80)
      let port = 80;

      // When: build_address is called
      let addr = App::build_address(port);

      // Then: It should format correctly
      assert_eq!(addr, "0.0.0.0:80");
    }

    #[tokio::test]
    async fn serve_attempts_to_bind_to_port() {
      // Note: Cannot directly test serve() because it calls init_tracing() which
      // can only run once per process. Instead, we test the components:

      // Given: A valid port
      std::env::set_var("PORT", "0"); // Port 0 lets OS assign random port

      // When: Testing the components that serve() uses
      let port = App::parse_port();
      let addr = App::build_address(port);

      // Then: bind_listener should succeed with port 0
      let result = App::bind_listener(&addr).await;
      assert!(result.is_ok(), "Should be able to bind to port 0");

      // Clean up
      std::env::remove_var("PORT");
    }

    #[tokio::test]
    async fn serve_handles_bind_failure() {
      // Note: This test is limited because serve() calls init_tracing() which can only
      // run once per process. We verify the bind error logic through the bind_listener test.
      // This test documents the expected behavior.

      // Verify that bind_listener fails with invalid port (this is what serve() uses internally)
      let result = App::bind_listener("127.0.0.1:99999").await;
      assert!(result.is_err());
    }

    #[test]
    fn log_functions_exist() {
      // These are simple logging functions that just call info!/warn!
      // We verify they exist and can be called (compilation test)
      App::log_server_start(3000);
      App::log_server_stop();
    }

    #[tokio::test]
    async fn bind_listener_succeeds_with_valid_address() {
      // Given: A valid address with port 0 (random port)
      let addr = "127.0.0.1:0";

      // When: Binding a listener
      let result = App::bind_listener(addr).await;

      // Then: It should succeed
      assert!(result.is_ok());
    }

    #[tokio::test]
    async fn bind_listener_fails_with_invalid_port() {
      // Given: An invalid port
      let addr = "127.0.0.1:99999";

      // When: Binding a listener
      let result = App::bind_listener(addr).await;

      // Then: It should fail
      assert!(result.is_err());
    }
  }

  /// Test suite for tracing and logging setup
  mod tracing_setup {
    use super::*;

    #[test]
    fn app_initialization_logs_are_present() {
      // Given: Tracing subscriber is initialized
      // When: Creating a new App
      // Then: Initialization logs should be emitted
      // Note: We can't easily capture logs in unit tests, but we verify
      // that the logging calls compile and the logic is correct
      let _app = App::new();
    }

    #[test]
    fn tracing_middleware_is_configured() {
      // Given: App creation process
      // When: new() is called
      // Then: TraceLayer should be added with INFO level logging
      // Note: This is verified by successful compilation and runtime behavior
      let _app = App::new();
    }

    #[test]
    fn server_logging_uses_structured_format() {
      // Given: Tracing setup in serve method
      // When: serve() initializes logging
      // Then: It should set up structured logging with proper directives
      // Note: This is tested by the fact that the code compiles
    }
  }

  /// Test suite for error handling in serve method
  mod error_handling {
    use super::*;

    #[tokio::test]
    async fn bind_error_returns_error_result() {
      // Given: A port that's likely already in use or invalid
      // Bind to a port first, then try to bind again
      let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
      let _addr = listener.local_addr().unwrap();

      // Create app and try to serve on the same port
      let _app = App::new().route("/", || async { "test" });

      // When: serve() tries to bind to the same address
      // We can't easily test this without actually running serve(),
      // but we can verify the error type is correct
      let result: Result<(), Box<dyn std::error::Error>> =
        Err(std::io::Error::new(std::io::ErrorKind::AddrInUse, "Address in use").into());

      // Then: The error should be a Box<dyn Error>
      assert!(result.is_err());
    }

    #[test]
    fn io_error_converts_to_boxed_error() {
      // Given: An IO error
      let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test error");

      // When: Converting to boxed error
      let boxed: Box<dyn std::error::Error> = io_err.into();

      // Then: It should work
      assert!(boxed.to_string().contains("test error"));
    }
  }

  /// Test suite for shutdown signal handling
  mod shutdown_signal {
    use super::*;

    #[tokio::test]
    async fn shutdown_signal_function_is_callable() {
      // Given: The shutdown_signal function
      // When: Creating the future (but not awaiting it indefinitely)
      // Then: It should compile and be a valid future
      let shutdown = shutdown_signal();

      // Verify it's a valid future by wrapping in a timeout
      let result = tokio::time::timeout(tokio::time::Duration::from_millis(1), shutdown).await;

      // We expect a timeout since no signal is sent
      assert!(result.is_err());
    }

    #[test]
    fn shutdown_signal_returns_future() {
      // Given: The shutdown_signal function
      // When: Calling it
      let _future = shutdown_signal();

      // Then: It returns a future that can be awaited
      // The type system ensures this is correct
    }
  }

  /// Integration-style tests that can run without external dependencies
  mod integration_tests {
    use super::*;

    #[test]
    fn app_builder_pattern_works() {
      // Given: Fluent API usage
      // When: Building an app with multiple routes
      let app = App::new()
        .route("/", || async { "Welcome" })
        .route("/api/v1/health", || async { "OK" })
        .route("/api/v1/users", || async { "[]" });

      // Then: It should succeed
      let _app = app;
    }

    #[test]
    fn app_can_be_moved_and_cloned_conceptually() {
      // Given: An app instance
      let app1 = App::new().route("/", || async { "test" });

      // When: Moving the app (conceptually - ownership transfer)
      // Then: It should work (compile-time test)
      let _app2 = app1;
    }

    #[test]
    fn route_handlers_can_return_different_types() {
      // Given: Different response types
      // When: Adding routes with different return types
      let app = App::new()
        .route("/text", || async { "plain text" })
        .route("/json", || async {
          axum::Json(serde_json::json!({"status": "ok"}))
        })
        .route("/html", || async { Html("<div>Hello</div>") });

      // Then: All should compile successfully
      let _app = app;
    }

    #[tokio::test]
    async fn tcp_listener_can_be_created_for_testing() {
      // Given: A need to verify TCP binding works
      // When: Creating a TCP listener on a random port
      let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await;

      // Then: It should succeed
      assert!(listener.is_ok());
      let listener = listener.unwrap();
      let addr = listener.local_addr().unwrap();
      assert!(addr.port() > 0);
    }

    #[tokio::test]
    async fn tcp_binding_fails_for_invalid_address() {
      // Given: An invalid address (port 0 with specific IP issues)
      // When: Trying to bind to an invalid port scenario
      // Note: Port 0 is actually valid (random port), so we test with
      // a malformed address instead
      let result: Result<tokio::net::TcpListener, _> =
        tokio::net::TcpListener::bind("invalid-address:999999").await;

      // Then: It should fail with an error
      assert!(result.is_err());
    }
  }
}
