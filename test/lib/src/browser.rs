//! Browser (fantoccini) helpers for E2E. Requires chromedriver and E2E_WEBDRIVER_URL when used.
//! Chrome is always started in headless mode so no window is shown.

use std::time::Duration;

/// Max time to wait for WebDriver (e.g. chromedriver) to accept a connection.
/// Prevents indefinite hang when E2E_WEBDRIVER_URL is set but no driver is running.
const WEBDRIVER_CONNECT_TIMEOUT: Duration = Duration::from_secs(15);

/// Max time to wait for an element to appear (forms, dashboard, etc.).
const ELEMENT_WAIT_TIMEOUT: Duration = Duration::from_secs(30);

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

/// Connect to WebDriver, goto `base_url`, assert `#root` exists (app root), close.
pub async fn assert_app_root_loads(
  base_url: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let c = connect().await?;
  c.goto(base_url).await?;
  let _ = c.find(fantoccini::Locator::Css("#root")).await?;
  c.close().await?;
  Ok(())
}

/// Goto `base_url` n times, assert `#root` exists each time, then close.
pub async fn assert_app_root_loads_repeated(
  base_url: &str,
  n: u32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let c = connect().await?;
  for _ in 0..n {
    c.goto(base_url).await?;
    let _ = c.find(fantoccini::Locator::Css("#root")).await?;
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

/// Fill and submit the register form at `/register`. Uses form set_by_name then click submit.
pub async fn register(
  c: &fantoccini::Client,
  base_url: &str,
  email: &str,
  password: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let url = format!("{}/register", base_url.trim_end_matches('/'));
  c.goto(&url).await?;
  let form_loc = fantoccini::Locator::Css("[data-testid=register-form]");
  c.wait()
    .at_most(ELEMENT_WAIT_TIMEOUT)
    .for_element(form_loc)
    .await?;
  let form = c.form(form_loc).await?;
  form.set_by_name("email", email).await?;
  form.set_by_name("password", password).await?;
  c.find(fantoccini::Locator::Css("[data-testid=register-submit]")).await?.click().await?;
  // Wait for server to create user and redirect to /login (template register returns Redirect::to("/login")).
  let login_url_substr = "/login";
  for _ in 0..(ELEMENT_WAIT_TIMEOUT.as_secs() * 2) {
    let url = c.current_url().await?;
    if url.as_str().contains(login_url_substr) {
      return Ok(());
    }
    tokio::time::sleep(Duration::from_millis(500)).await;
  }
  Err("register: timed out waiting for redirect to /login".into())
}

/// Fill and submit the login form at `/login`. Form uses native submit; fill via send_keys so required/validation pass, then form.submit().
pub async fn login(
  c: &fantoccini::Client,
  base_url: &str,
  email: &str,
  password: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let url = format!("{}/login", base_url.trim_end_matches('/'));
  c.goto(&url).await?;
  let form_loc = fantoccini::Locator::Css("[data-testid=login-form]");
  c.wait()
    .at_most(ELEMENT_WAIT_TIMEOUT)
    .for_element(form_loc)
    .await?;
  let email_el = c.find(fantoccini::Locator::Css("[data-testid=login-email]")).await?;
  email_el.clear().await?;
  email_el.send_keys(email).await?;
  let password_el = c.find(fantoccini::Locator::Css("[data-testid=login-password]")).await?;
  password_el.clear().await?;
  password_el.send_keys(password).await?;
  let form = c.form(form_loc).await?;
  form.submit().await?;
  // Wait for post-login (browser follows 302 to /dashboard): either URL contains /dashboard or dashboard heading appears.
  let dashboard_path = "/dashboard";
  let heading_loc = fantoccini::Locator::Css("[data-testid=dashboard-heading]");
  let mut seen_dashboard = false;
  for _ in 0..(ELEMENT_WAIT_TIMEOUT.as_secs() * 2) {
    let url = c.current_url().await?;
    if url.as_str().contains(dashboard_path) {
      seen_dashboard = true;
      break;
    }
    if let Ok(el) = c.find(heading_loc).await {
      if let Ok(text) = el.text().await {
        if text.contains("Dashboard") {
          seen_dashboard = true;
          break;
        }
      }
    }
    tokio::time::sleep(Duration::from_millis(500)).await;
  }
  if !seen_dashboard {
    let current = c.current_url().await?;
    return Err(format!(
      "login: redirect to /dashboard or dashboard heading did not appear (current URL: {})",
      current
    )
    .into());
  }
  let heading = fantoccini::Locator::Css("[data-testid=dashboard-heading]");
  c.wait().at_most(ELEMENT_WAIT_TIMEOUT).for_element(heading).await?;
  let el = c.find(heading).await?;
  let text = el.text().await?;
  if !text.contains("Dashboard") {
    return Err(format!("login: dashboard page heading should contain 'Dashboard'; got {:?}", text).into());
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
  c.wait()
    .at_most(ELEMENT_WAIT_TIMEOUT)
    .for_element(heading)
    .await?;
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

/// Trigger logout by GET /api/auth/logout. Call when already logged in; session is cleared.
pub async fn logout(
  c: &fantoccini::Client,
  base_url: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let url = format!("{}/api/auth/logout", base_url.trim_end_matches('/'));
  c.goto(&url).await?;
  tokio::time::sleep(std::time::Duration::from_millis(200)).await;
  Ok(())
}

/// Submit login form; assert login fails (no redirect to dashboard). Use for wrong password / unknown user.
pub async fn login_fails(
  c: &fantoccini::Client,
  base_url: &str,
  email: &str,
  password: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let url = format!("{}/login", base_url.trim_end_matches('/'));
  let email_locator = fantoccini::Locator::Css("[data-testid=login-email]");
  let password_locator = fantoccini::Locator::Css("[data-testid=login-password]");
  let submit_locator = fantoccini::Locator::Css("[data-testid=login-submit]");
  c.goto(&url).await?;
  c.wait()
    .at_most(ELEMENT_WAIT_TIMEOUT)
    .for_element(email_locator)
    .await?;
  let email_el = c.find(email_locator).await?;
  email_el.clear().await?;
  email_el.send_keys(email).await?;
  let pw_el = c.find(password_locator).await?;
  pw_el.clear().await?;
  pw_el.send_keys(password).await?;
  c.find(submit_locator).await?.click().await?;
  // Wait a bit; we must NOT end up on dashboard.
  tokio::time::sleep(Duration::from_secs(2)).await;
  let url = c.current_url().await?;
  if url.as_str().contains("/dashboard") {
    return Err("login_fails: expected to stay on login page, but redirected to dashboard".into());
  }
  Ok(())
}

/// Goto /api/auth/admin (must be logged in as global admin). Asserts response body contains "access granted".
pub async fn assert_admin_endpoint_granted(
  c: &fantoccini::Client,
  base_url: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let url = format!("{}/api/auth/admin", base_url.trim_end_matches('/'));
  c.goto(&url).await?;
  tokio::time::sleep(std::time::Duration::from_millis(300)).await;
  let body = c.source().await?;
  if !body.contains("access granted") {
    return Err(format!("admin endpoint should grant access; body: {:?}", body).into());
  }
  Ok(())
}

/// Goto /api/auth/admin (must be logged in as non-admin). Asserts response body contains "Forbidden".
pub async fn assert_admin_endpoint_denied(
  c: &fantoccini::Client,
  base_url: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let url = format!("{}/api/auth/admin", base_url.trim_end_matches('/'));
  c.goto(&url).await?;
  tokio::time::sleep(std::time::Duration::from_millis(300)).await;
  let body = c.source().await?;
  if !body.contains("Forbidden") {
    return Err(format!("admin endpoint should deny access (Forbidden); body: {:?}", body).into());
  }
  Ok(())
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
  tokio::time::sleep(std::time::Duration::from_secs(1)).await;
  let body = c.source().await?;
  c.close().await?;
  if !body.contains(msg) {
    return Err(format!("ws-demo page source should contain {:?} after send", msg).into());
  }
  Ok(())
}
