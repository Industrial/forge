//! E2E tests for Forge rate limiting (009).
//!
//! Covers: per-IP rate limiting returns 429 when limit exceeded;
//! health endpoints are excluded from rate limiting.

use std::fs;
use std::time::Duration;

/// E2E: per-IP rate limit — when limit exceeded, returns 429.
#[tokio::test]
async fn per_ip_rate_limit_returns_429_when_exceeded() {
  let temp_dir = tempfile::tempdir().unwrap();
  let original_cwd = std::env::current_dir().unwrap();
  std::env::set_current_dir(temp_dir.path()).unwrap();

  let config_dir = temp_dir.path().join("config");
  fs::create_dir_all(&config_dir).unwrap();
  fs::write(
    config_dir.join("app.toml"),
    r#"[app]
name = "rate_limit_e2e"
environment = "test"

[server]
host = "127.0.0.1"
port = 0
"#,
  )
  .unwrap();
  fs::write(
    config_dir.join("db.toml"),
    r#"[database]
url = "sqlite::memory:"
"#,
  )
  .unwrap();

  // Limit: 2 requests burst, 1 replenished per second — 3rd request within window gets 429
  let app = forge::App::new()
    .with_rate_limit_per_ip(2)
    .route("/api/ping", || async { "pong" });

  let (router, _) = app.into_router().await;
  let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
  let port = listener.local_addr().unwrap().port();
  tokio::spawn(async move {
    axum::serve(
      listener,
      router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
  });

  tokio::time::sleep(Duration::from_millis(100)).await;

  let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(5))
    .build()
    .unwrap();
  let base = format!("http://127.0.0.1:{}", port);

  // First two requests succeed
  let r1 = client
    .get(format!("{}/api/ping", base))
    .send()
    .await
    .unwrap();
  let r2 = client
    .get(format!("{}/api/ping", base))
    .send()
    .await
    .unwrap();
  assert_eq!(r1.status().as_u16(), 200, "first request should be 200");
  assert_eq!(r2.status().as_u16(), 200, "second request should be 200");

  // Third request is rate limited
  let r3 = client
    .get(format!("{}/api/ping", base))
    .send()
    .await
    .unwrap();
  assert_eq!(
    r3.status().as_u16(),
    429,
    "third request should be 429 Too Many Requests"
  );

  std::env::set_current_dir(original_cwd).unwrap();
}

/// E2E: health endpoints are not rate limited — many requests all return 200.
#[tokio::test]
async fn health_endpoints_excluded_from_rate_limiting() {
  let temp_dir = tempfile::tempdir().unwrap();
  let original_cwd = std::env::current_dir().unwrap();
  std::env::set_current_dir(temp_dir.path()).unwrap();

  let config_dir = temp_dir.path().join("config");
  fs::create_dir_all(&config_dir).unwrap();
  fs::write(
    config_dir.join("app.toml"),
    r#"[app]
name = "rate_limit_e2e"
environment = "test"

[server]
host = "127.0.0.1"
port = 0
"#,
  )
  .unwrap();
  fs::write(
    config_dir.join("db.toml"),
    r#"[database]
url = "sqlite::memory:"
"#,
  )
  .unwrap();

  let app = forge::App::new()
    .with_rate_limit_per_ip(2)
    .route("/api/ping", || async { "pong" });

  let (router, _) = app.into_router().await;
  let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
  let port = listener.local_addr().unwrap().port();
  tokio::spawn(async move {
    axum::serve(
      listener,
      router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
  });

  tokio::time::sleep(Duration::from_millis(100)).await;

  let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(5))
    .build()
    .unwrap();
  let base = format!("http://127.0.0.1:{}", port);

  // Exhaust rate limit on /api/ping
  let _ = client
    .get(format!("{}/api/ping", base))
    .send()
    .await
    .unwrap();
  let _ = client
    .get(format!("{}/api/ping", base))
    .send()
    .await
    .unwrap();
  let r3 = client
    .get(format!("{}/api/ping", base))
    .send()
    .await
    .unwrap();
  assert_eq!(r3.status().as_u16(), 429);

  // Health endpoints remain 200 regardless of rate limit
  for _ in 0..5 {
    let r = client
      .get(format!("{}/healthz", base))
      .send()
      .await
      .unwrap();
    assert_eq!(r.status().as_u16(), 200, "healthz must not be rate limited");
    let r = client.get(format!("{}/livez", base)).send().await.unwrap();
    assert_eq!(r.status().as_u16(), 200, "livez must not be rate limited");
    let r = client.get(format!("{}/readyz", base)).send().await.unwrap();
    assert_eq!(r.status().as_u16(), 200, "readyz must not be rate limited");
  }

  std::env::set_current_dir(original_cwd).unwrap();
}
