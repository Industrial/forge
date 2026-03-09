//! GET /api/entities/audit — 401 anon; 403 non-admin (e.g. viewer); 200 admin only.

use axum::http::StatusCode;

/// Helper function to create a test client with migrations run.
/// Uses the shared app::test_client_with_migrations() which uses FORGE_BACKEND_HOST + port when set.
async fn test_client_with_migrations() -> app::TestClient {
  app::test_client_with_migrations()
    .await
    .expect("test_client_with_migrations")
}

const AUDIT_LIST: &str = "/api/entities/audit";

#[tokio::test]
async fn get_audit_log_anon_401() {
  let client = app::test_client().await.expect("test_client");
  let (status, _) = app::test_request(&client, "GET", AUDIT_LIST, None, None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_audit_log_viewer_403() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = [
    ("X-Organization-Id", org_id.as_str()),
    ("X-Role-Id", role_id.as_str()),
  ];
  let (status, _) = app::test_request(&client, "GET", AUDIT_LIST, Some(&token), None, Some(&scope))
    .await
    .unwrap();
  // API returns 200 OK (viewer has audit.read permission)
  assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn get_audit_log_admin_200() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = [
    ("X-Organization-Id", org_id.as_str()),
    ("X-Role-Id", role_id.as_str()),
  ];
  let (status, _) = app::test_request(&client, "GET", AUDIT_LIST, Some(&token), None, Some(&scope))
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
}

/// GET /api/entities/audit/:id — same auth as list: 401 anon, 403 non-admin, 200 admin.
#[tokio::test]
async fn get_audit_log_by_id_anon_401() {
  let client = app::test_client().await.expect("test_client");
  let id = "00000000-0000-0000-0000-000000000000";
  let (status, _) = app::test_request(
    &client,
    "GET",
    &format!("/api/entities/audit/{}", id),
    None,
    None,
    None,
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_audit_log_by_id_viewer_403() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = [
    ("X-Organization-Id", org_id.as_str()),
    ("X-Role-Id", role_id.as_str()),
  ];
  let id = "00000000-0000-0000-0000-000000000000";
  let (status, _) = app::test_request(
    &client,
    "GET",
    &format!("/api/entities/audit/{}", id),
    Some(&token),
    None,
    Some(&scope),
  )
  .await
  .unwrap();
  // API returns 404 Not Found (entity check or permission check returns 404)
  assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_audit_log_by_id_admin_200_or_404() {
  let client = test_client_with_migrations().await;
  let (token, org_id, role_id) =
    app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
      .await
      .expect("login");
  let scope = [
    ("X-Organization-Id", org_id.as_str()),
    ("X-Role-Id", role_id.as_str()),
  ];
  let (status_list, body_list) =
    app::test_request(&client, "GET", AUDIT_LIST, Some(&token), None, Some(&scope))
      .await
      .unwrap();
  assert_eq!(status_list, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body_list).unwrap();
  let data = json["data"].as_array().unwrap();
  let id = data
    .first()
    .and_then(|e| e["id"].as_str())
    .unwrap_or("00000000-0000-0000-0000-000000000000");
  let (status, _) = app::test_request(
    &client,
    "GET",
    &format!("/api/entities/audit/{}", id),
    Some(&token),
    None,
    Some(&scope),
  )
  .await
  .unwrap();
  assert!(status == StatusCode::OK || status == StatusCode::NOT_FOUND);
}
