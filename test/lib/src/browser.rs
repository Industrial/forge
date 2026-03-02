//! Browser (fantoccini) helpers for E2E. Requires chromedriver and E2E_WEBDRIVER_URL when used.
//! Chrome is always started in headless mode so no window is shown.

use std::time::Duration;

/// Max time to wait for WebDriver (e.g. chromedriver) to accept a connection.
/// Prevents indefinite hang when E2E_WEBDRIVER_URL is set but no driver is running.
const WEBDRIVER_CONNECT_TIMEOUT: Duration = Duration::from_secs(15);

/// Max time to wait for an element to appear (forms, dashboard, etc.).
const ELEMENT_WAIT_TIMEOUT: Duration = Duration::from_secs(30);

/// Interval for polling URL/source when waiting for route or content changes.
const POLL_INTERVAL_MS: u64 = 200;

/// Returns E2E_WEBDRIVER_URL env var or default `http://localhost:9515`.
fn webdriver_url() -> String {
  std::env::var("E2E_WEBDRIVER_URL").unwrap_or_else(|_| "http://localhost:9515".to_string())
}

/// Headless Chrome capabilities so no browser window is shown.
fn headless_chrome_capabilities() -> fantoccini::wd::Capabilities {
  serde_json::from_value(serde_json::json!({
    "browserName": "chrome",
    "goog:chromeOptions": {
      "args": ["--headless=new", "--disable-gpu", "--no-sandbox", "--disable-dev-shm-usage"]
    }
  }))
  .expect("valid headless chrome capabilities")
}

/// Connect to WebDriver (Chrome in headless mode). Caller must call `c.close().await?` when done.
/// Fails after WEBDRIVER_CONNECT_TIMEOUT if chromedriver is not reachable (avoids indefinite hang).
pub async fn connect() -> Result<fantoccini::Client, Box<dyn std::error::Error + Send + Sync>> {
  let url = webdriver_url();
  let caps = headless_chrome_capabilities();
  let connect_fut = async {
    fantoccini::ClientBuilder::native()
      .capabilities(caps)
      .connect(&url)
      .await
  };
  match tokio::time::timeout(WEBDRIVER_CONNECT_TIMEOUT, connect_fut).await {
    Ok(Ok(c)) => Ok(c),
    Ok(Err(e)) => Err(e.into()),
    Err(_) => Err(format!(
      "WebDriver connection to {} timed out after {:?} — is chromedriver running? (e.g. run bin/test-e2e or start chromedriver --port=9515)",
      url,
      WEBDRIVER_CONNECT_TIMEOUT
    )
    .into()),
  }
}

/// Connect to WebDriver, goto `base_url`, wait for `#root` (app root), then close.
pub async fn assert_app_root_loads(
  base_url: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let c = connect().await?;
  c.goto(base_url).await?;
  c.wait()
    .for_element(fantoccini::Locator::Css("#root"))
    .await?;
  c.close().await?;
  Ok(())
}

/// Goto `base_url` n times, wait for `#root` each time, then close.
pub async fn assert_app_root_loads_repeated(
  base_url: &str,
  n: u32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let c = connect().await?;
  let root_loc = fantoccini::Locator::Css("#root");
  for _ in 0..n {
    c.goto(base_url).await?;
    c.wait().for_element(root_loc).await?;
  }
  c.close().await?;
  Ok(())
}

/// Goto `base_url`, assert `#root` exists and page source contains `substring`, then close.
pub async fn assert_app_root_loads_and_contains(
  base_url: &str,
  substring: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let c = connect().await?;
  c.goto(base_url).await?;
  let _ = c.find(fantoccini::Locator::Css("#root")).await?;
  let body = c.source().await?;
  if !body.contains(substring) {
    c.close().await?;
    return Err(format!("page source should contain {:?}", substring).into());
  }
  c.close().await?;
  Ok(())
}

/// Goto `base_url + path`, assert `#root` exists, then close.
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
  let _ = c.find(fantoccini::Locator::Css("#root")).await?;
  c.close().await?;
  Ok(())
}

/// Fill and submit the register form at `/register`. Uses native inputs (Spectrum wraps in divs). Clicks submit so SPA fetch runs.
pub async fn register(
  c: &fantoccini::Client,
  base_url: &str,
  email: &str,
  password: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let url = format!("{}/register", base_url.trim_end_matches('/'));
  c.goto(&url).await?;
  let form_loc = fantoccini::Locator::Css("[data-testid=register-form]");
  let email_input = fantoccini::Locator::Css("[data-testid=register-form] input[type=email]");
  let password_input = fantoccini::Locator::Css("[data-testid=register-form] input[type=password]");
  let submit_loc = fantoccini::Locator::Css("[data-testid=register-submit]");
  c.wait().for_element(form_loc).await?;
  let email_el = c.find(email_input).await?;
  email_el.clear().await?;
  email_el.send_keys(email).await?;
  let pw_el = c.find(password_input).await?;
  pw_el.clear().await?;
  pw_el.send_keys(password).await?;
  c.find(submit_loc).await?.click().await?;
  // Wait for SPA to navigate to /login after successful register.
  let login_url_substr = "/login";
  let deadline = std::time::Instant::now() + ELEMENT_WAIT_TIMEOUT;
  while std::time::Instant::now() < deadline {
    let url = c.current_url().await?;
    if url.as_str().contains(login_url_substr) {
      return Ok(());
    }
    tokio::time::sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
  }
  Err("register: timed out waiting for redirect to /login".into())
}

/// Fill and submit the login form at `/login`. Uses native inputs (Spectrum wraps in divs). Clicks submit so SPA fetch runs.
pub async fn login(
  c: &fantoccini::Client,
  base_url: &str,
  email: &str,
  password: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let url = format!("{}/login", base_url.trim_end_matches('/'));
  c.goto(&url).await?;
  let form_loc = fantoccini::Locator::Css("[data-testid=login-form]");
  let email_input = fantoccini::Locator::Css("[data-testid=login-form] input[type=email]");
  let password_input = fantoccini::Locator::Css("[data-testid=login-form] input[type=password]");
  let submit_loc = fantoccini::Locator::Css("[data-testid=login-submit]");
  c.wait().for_element(form_loc).await?;
  let email_el = c.find(email_input).await?;
  email_el.clear().await?;
  email_el.send_keys(email).await?;
  let password_el = c.find(password_input).await?;
  password_el.clear().await?;
  password_el.send_keys(password).await?;
  c.find(submit_loc).await?.click().await?;
  // Wait for post-login: URL contains /dashboard or dashboard heading appears.
  let dashboard_path = "/dashboard";
  let heading_loc = fantoccini::Locator::Css("[data-testid=dashboard-heading]");
  let mut seen_dashboard = false;
  let deadline = std::time::Instant::now() + ELEMENT_WAIT_TIMEOUT;
  while std::time::Instant::now() < deadline {
    let url = c.current_url().await?;
    if url.as_str().contains(dashboard_path) {
      seen_dashboard = true;
      break;
    }
    if let Ok(el) = c.find(heading_loc).await
      && let Ok(text) = el.text().await
      && text.contains("Dashboard")
    {
      seen_dashboard = true;
      break;
    }
    tokio::time::sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
  }
  if !seen_dashboard {
    let current = c.current_url().await?;
    return Err(
      format!(
        "login: redirect to /dashboard or dashboard heading did not appear (current URL: {})",
        current
      )
      .into(),
    );
  }
  let heading = fantoccini::Locator::Css("[data-testid=dashboard-heading]");
  c.wait().for_element(heading).await?;
  let el = c.find(heading).await?;
  let text = el.text().await?;
  if !text.contains("Dashboard") {
    return Err(
      format!(
        "login: dashboard page heading should contain 'Dashboard'; got {:?}",
        text
      )
      .into(),
    );
  }
  Ok(())
}

/// Goto /dashboard, wait for the dashboard heading (client-rendered), then assert it.
pub async fn assert_dashboard_visible(
  c: &fantoccini::Client,
  base_url: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let url = format!("{}/dashboard", base_url.trim_end_matches('/'));
  c.goto(&url).await?;
  let heading = fantoccini::Locator::Css("[data-testid=dashboard-heading]");
  c.wait().for_element(heading).await?;
  let el = c.find(heading).await?;
  let text = el.text().await?;
  if !text.contains("Dashboard") {
    return Err(
      format!(
        "dashboard page heading should contain 'Dashboard'; got {:?}",
        text
      )
      .into(),
    );
  }
  Ok(())
}

/// Goto /dashboard unauthed; wait for SPA to redirect to login (URL contains "login").
pub async fn assert_dashboard_redirects_to_login(
  c: &fantoccini::Client,
  base_url: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let dashboard_url = format!("{}/dashboard", base_url.trim_end_matches('/'));
  c.goto(&dashboard_url).await?;
  let deadline = std::time::Instant::now() + ELEMENT_WAIT_TIMEOUT;
  while std::time::Instant::now() < deadline {
    let url = c.current_url().await?;
    if url.as_str().contains("login") {
      return Ok(());
    }
    tokio::time::sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
  }
  let url = c.current_url().await?;
  Err(
    format!(
      "unauthed /dashboard should redirect to login; current url: {}",
      url.as_str()
    )
    .into(),
  )
}

/// Trigger logout by GET /api/auth/logout. Call when already logged in; session is cleared.
pub async fn logout(
  c: &fantoccini::Client,
  base_url: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let url = format!("{}/api/auth/logout", base_url.trim_end_matches('/'));
  c.goto(&url).await?;
  Ok(())
}

/// Submit login form; assert login fails (no redirect to dashboard). Use for wrong password / unknown user.
/// Uses native inputs inside the form so Spectrum wrapper divs don't cause "invalid element state".
pub async fn login_fails(
  c: &fantoccini::Client,
  base_url: &str,
  email: &str,
  password: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let url = format!("{}/login", base_url.trim_end_matches('/'));
  let form_loc = fantoccini::Locator::Css("[data-testid=login-form]");
  let email_input = fantoccini::Locator::Css("[data-testid=login-form] input[type=email]");
  let password_input = fantoccini::Locator::Css("[data-testid=login-form] input[type=password]");
  let submit_locator = fantoccini::Locator::Css("[data-testid=login-submit]");
  c.goto(&url).await?;
  c.wait().for_element(form_loc).await?;
  let email_el = c.find(email_input).await?;
  email_el.clear().await?;
  email_el.send_keys(email).await?;
  let pw_el = c.find(password_input).await?;
  pw_el.clear().await?;
  pw_el.send_keys(password).await?;
  c.find(submit_locator).await?.click().await?;
  // Wait for route/content: either error appears (success) or redirect to dashboard (failure).
  let error_loc = fantoccini::Locator::Css("[data-testid=login-error]");
  let deadline = std::time::Instant::now() + ELEMENT_WAIT_TIMEOUT;
  while std::time::Instant::now() < deadline {
    let url = c.current_url().await?;
    if url.as_str().contains("/dashboard") {
      return Err(
        "login_fails: expected to stay on login page, but redirected to dashboard".into(),
      );
    }
    if c.find(error_loc).await.is_ok() {
      return Ok(());
    }
    tokio::time::sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
  }
  let url = c.current_url().await?;
  if url.as_str().contains("/dashboard") {
    return Err("login_fails: expected to stay on login page, but redirected to dashboard".into());
  }
  Err("login_fails: timed out waiting for login error element".into())
}

/// Goto /api/auth/admin (must be logged in as global admin). Waits for response body to contain "access granted".
pub async fn assert_admin_endpoint_granted(
  c: &fantoccini::Client,
  base_url: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let url = format!("{}/api/auth/admin", base_url.trim_end_matches('/'));
  c.goto(&url).await?;
  let deadline = std::time::Instant::now() + ELEMENT_WAIT_TIMEOUT;
  while std::time::Instant::now() < deadline {
    let body = c.source().await?;
    if body.contains("access granted") {
      return Ok(());
    }
    tokio::time::sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
  }
  let body = c.source().await?;
  Err(format!("admin endpoint should grant access; body: {:?}", body).into())
}

/// Goto /api/auth/admin (must be logged in as non-admin). Waits for response body to contain "Forbidden".
pub async fn assert_admin_endpoint_denied(
  c: &fantoccini::Client,
  base_url: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let url = format!("{}/api/auth/admin", base_url.trim_end_matches('/'));
  c.goto(&url).await?;
  let deadline = std::time::Instant::now() + ELEMENT_WAIT_TIMEOUT;
  while std::time::Instant::now() < deadline {
    let body = c.source().await?;
    if body.contains("Forbidden") {
      return Ok(());
    }
    tokio::time::sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
  }
  let body = c.source().await?;
  Err(
    format!(
      "admin endpoint should deny access (Forbidden); body: {:?}",
      body
    )
    .into(),
  )
}

/// Connect to WebDriver, goto health URL, assert body contains "ok" and does not disclose components.
pub async fn assert_health_ok(
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
  let _ = c.find(fantoccini::Locator::Css("#root")).await?;
  c.find(fantoccini::Locator::Css("input"))
    .await?
    .send_keys(msg)
    .await?;
  c.find(fantoccini::Locator::Css("button"))
    .await?
    .click()
    .await?;
  let deadline = std::time::Instant::now() + ELEMENT_WAIT_TIMEOUT;
  while std::time::Instant::now() < deadline {
    let body = c.source().await?;
    if body.contains(msg) {
      c.close().await?;
      return Ok(());
    }
    tokio::time::sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
  }
  let body = c.source().await?;
  c.close().await?;
  Err(
    format!(
      "ws-demo page source should contain {:?} after send; body: {:?}",
      msg, body
    )
    .into(),
  )
}
