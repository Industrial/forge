//! E2E test for Forge Inertia (018): Vite+React frontend, HTML shell, i18n locale, auth routes, WebSocket.
//! Run via `bin/test-e2e`. Requires frontend built with bun (bin/test-e2e runs bun install + bun run build).

use std::time::Duration;

use forge_e2e_lib::cli;
use futures_util::{SinkExt, StreamExt};

/// Single E2E test: prebuilt has frontend (018), GET / returns Inertia HTML shell, SPA routes 200,
/// Accept-Language sets locale in props, /ws WebSocket echo works.
#[tokio::test]
async fn e2e_prebuilt_inertia_shell_i18n_auth_ws() {
  let project_root = cli::prebuilt_project_root();
  assert!(
    project_root.exists(),
    "prebuilt project not found at {} — run bin/test-e2e first",
    project_root.display()
  );
  cli::assert_project_layout(&project_root);

  // 018 layout: frontend with package.json and main.tsx
  let frontend_pkg = project_root.join("frontend/package.json");
  let frontend_main = project_root.join("frontend/src/main.tsx");
  assert!(
    frontend_pkg.exists(),
    "018 frontend/package.json missing at {}",
    frontend_pkg.display()
  );
  assert!(
    frontend_main.exists(),
    "018 frontend/src/main.tsx missing at {}",
    frontend_main.display()
  );

  let base = cli::e2e_base_url().expect("run e2e via bin/test-e2e (E2E_BASE_URL not set)");
  let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(5))
    .redirect(reqwest::redirect::Policy::none())
    .build()
    .unwrap();

  // GET / -> Inertia HTML shell with app root and script
  let home = client.get(base.clone()).send().await.expect("GET /");
  assert!(
    home.status().is_success(),
    "GET / must succeed; got {}",
    home.status()
  );
  let home_body = home.text().await.expect("body");
  assert!(
    home_body.contains("id=\"app\"") || home_body.contains("id='app'"),
    "GET / should return Inertia app root; got {:?}",
    home_body
  );
  assert!(
    home_body.contains("Pages/Home"),
    "GET / should contain Pages/Home component; got {:?}",
    home_body
  );

  // GET / with Accept-Language -> locale in page props
  let en_resp = client
    .get(base.clone())
    .header("Accept-Language", "en-US,en;q=0.9")
    .send()
    .await
    .expect("GET / en");
  let en_body = en_resp.text().await.expect("body");
  assert!(
    en_body.contains("en-US"),
    "Accept-Language en-US should set locale in props; got {:?}",
    en_body
  );

  // SPA routes return 200 with Inertia HTML
  for path in ["/login", "/register", "/dashboard", "/ws-demo"] {
    let resp = client
      .get(format!("{}{}", base, path))
      .send()
      .await
      .expect(format!("GET {}", path).as_str());
    assert!(
      resp.status().is_success(),
      "GET {} must succeed; got {}",
      path,
      resp.status()
    );
    let body = resp.text().await.expect("body");
    assert!(
      body.contains("id=\"app\"") || body.contains("id='app'"),
      "GET {} should return Inertia shell; got {:?}",
      path,
      body
    );
  }

  // WebSocket /ws echo
  let ws_url = base
    .replace("http://", "ws://")
    .replace("https://", "wss://");
  let ws_full = format!("{}/ws", ws_url);
  let (mut ws, _) = tokio_tungstenite::connect_async(&ws_full)
    .await
    .expect("WebSocket connect to /ws");
  ws.send(tokio_tungstenite::tungstenite::Message::Text("ping".into()))
    .await
    .expect("ws send");
  let msg = ws.next().await.expect("ws message").expect("ws recv error");
  let text = msg.to_text().unwrap_or_default();
  assert!(
    text == "ping",
    "WebSocket echo should return 'ping'; got {:?}",
    text
  );
}
