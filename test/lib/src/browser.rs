//! Browser (fantoccini) helpers for E2E. Requires chromedriver and E2E_WEBDRIVER_URL when used.

/// Returns E2E_WEBDRIVER_URL env var or default `http://localhost:9515`.
fn webdriver_url() -> String {
  std::env::var("E2E_WEBDRIVER_URL").unwrap_or_else(|_| "http://localhost:9515".to_string())
}

/// Connect to WebDriver. Caller must call `c.close().await?` when done.
pub async fn connect() -> Result<fantoccini::Client, Box<dyn std::error::Error + Send + Sync>> {
  let c = fantoccini::ClientBuilder::native()
    .connect(&webdriver_url())
    .await?;
  Ok(c)
}

/// Connect to WebDriver, goto `base_url`, assert `#app` exists (Inertia root), close.
pub async fn assert_app_root_loads(
  base_url: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let c = connect().await?;
  c.goto(base_url).await?;
  let _ = c.find(fantoccini::Locator::Css("#app")).await?;
  c.close().await?;
  Ok(())
}

/// Goto `base_url` n times, assert `#app` exists each time, then close.
pub async fn assert_app_root_loads_repeated(
  base_url: &str,
  n: u32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let c = connect().await?;
  for _ in 0..n {
    c.goto(base_url).await?;
    let _ = c.find(fantoccini::Locator::Css("#app")).await?;
  }
  c.close().await?;
  Ok(())
}

/// Goto `base_url`, assert `#app` exists and page source contains `substring`, then close.
pub async fn assert_app_root_loads_and_contains(
  base_url: &str,
  substring: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let c = connect().await?;
  c.goto(base_url).await?;
  let _ = c.find(fantoccini::Locator::Css("#app")).await?;
  let body = c.source().await?;
  if !body.contains(substring) {
    c.close().await?;
    return Err(format!("page source should contain {:?}", substring).into());
  }
  c.close().await?;
  Ok(())
}

/// Goto `base_url + path`, assert `#app` exists, then close.
pub async fn goto_path_and_assert_app(
  base_url: &str,
  path: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let base = base_url.trim_end_matches('/');
  let url = if path.starts_with('/') {
    format!("{}{}", base, path)
  } else {
    format!("{}/{}", base, path)
  };
  let c = connect().await?;
  c.goto(&url).await?;
  let _ = c.find(fantoccini::Locator::Css("#app")).await?;
  c.close().await?;
  Ok(())
}

/// Fill and submit the register form at `/register`. Uses input[type=email], input[type=password], button[type=submit].
pub async fn register_via_browser(
  c: &fantoccini::Client,
  base_url: &str,
  email: &str,
  password: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let url = format!("{}/register", base_url.trim_end_matches('/'));
  c.goto(&url).await?;
  c.find(fantoccini::Locator::Css("input[type=email]"))
    .await?
    .send_keys(email)
    .await?;
  c.find(fantoccini::Locator::Css("input[type=password]"))
    .await?
    .send_keys(password)
    .await?;
  c.find(fantoccini::Locator::Css("button[type=submit]"))
    .await?
    .click()
    .await?;
  tokio::time::sleep(std::time::Duration::from_millis(500)).await;
  Ok(())
}

/// Fill and submit the login form at `/login`.
pub async fn login_via_browser(
  c: &fantoccini::Client,
  base_url: &str,
  email: &str,
  password: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let url = format!("{}/login", base_url.trim_end_matches('/'));
  c.goto(&url).await?;
  c.find(fantoccini::Locator::Css("input[type=email]"))
    .await?
    .send_keys(email)
    .await?;
  c.find(fantoccini::Locator::Css("input[type=password]"))
    .await?
    .send_keys(password)
    .await?;
  c.find(fantoccini::Locator::Css("button[type=submit]"))
    .await?
    .click()
    .await?;
  tokio::time::sleep(std::time::Duration::from_millis(500)).await;
  Ok(())
}

/// Goto /dashboard, assert #app and that page source contains "Welcome" or "dashboard".
pub async fn assert_dashboard_visible(
  c: &fantoccini::Client,
  base_url: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let url = format!("{}/dashboard", base_url.trim_end_matches('/'));
  c.goto(&url).await?;
  let _ = c.find(fantoccini::Locator::Css("#app")).await?;
  let body = c.source().await?;
  if !body.contains("Welcome") && !body.contains("dashboard") && !body.contains("Dashboard") {
    return Err(
      format!(
        "dashboard page should contain Welcome or dashboard; got {:?}",
        &body[..body.len().min(400)]
      )
      .into(),
    );
  }
  Ok(())
}

/// Goto /dashboard unauthed; assert current URL contains "login" (redirect to login page).
pub async fn assert_dashboard_redirects_to_login(
  c: &fantoccini::Client,
  base_url: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let dashboard_url = format!("{}/dashboard", base_url.trim_end_matches('/'));
  c.goto(&dashboard_url).await?;
  tokio::time::sleep(std::time::Duration::from_millis(300)).await;
  let url = c.current_url().await?;
  let s = url.as_str();
  if !s.contains("login") {
    return Err(
      format!(
        "unauthed /dashboard should redirect to login; current url: {}",
        s
      )
      .into(),
    );
  }
  Ok(())
}

/// Connect to WebDriver, goto health URL, assert body contains "ok" and does not disclose components.
pub async fn assert_health_ok_in_browser(
  health_url: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let c = connect().await?;
  c.goto(health_url).await?;
  let body = c.source().await?;
  if !body.contains("ok") {
    c.close().await?;
    return Err("health body should contain 'ok'".into());
  }
  if body.contains("components") || body.contains("database") {
    c.close().await?;
    return Err("health response must not disclose components".into());
  }
  c.close().await?;
  Ok(())
}

/// Goto /ws-demo, type `msg` in the input, click Send, assert message appears in page source.
pub async fn assert_ws_demo_echo(
  base_url: &str,
  msg: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let c = connect().await?;
  let url = base_url.trim_end_matches('/');
  let ws_demo_url = format!("{}/ws-demo", url);
  c.goto(&ws_demo_url).await?;
  let _ = c.find(fantoccini::Locator::Css("#app")).await?;
  c.find(fantoccini::Locator::Css("input"))
    .await?
    .send_keys(msg)
    .await?;
  c.find(fantoccini::Locator::Css("button"))
    .await?
    .click()
    .await?;
  tokio::time::sleep(std::time::Duration::from_secs(1)).await;
  let body = c.source().await?;
  c.close().await?;
  if !body.contains(msg) {
    return Err(format!("ws-demo page source should contain {:?} after send", msg).into());
  }
  Ok(())
}
