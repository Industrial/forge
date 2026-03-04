//! Dashboard: GET/POST/PATCH/DELETE /api/dashboard/roles — 401 anon; GET 200 any auth with read; POST/PATCH/DELETE 403 viewer, 200 editor+.

use axum::http::StatusCode;

#[tokio::test]
async fn get_roles_anon_401() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let (status, _) = app::test_request(&router, "GET", "/api/dashboard/roles", None, None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

fn scope_headers(org_id: &str, role_id: &str) -> [(&'static str, &str); 2] {
  [("X-Organization-Id", org_id), ("X-Role-Id", role_id)]
}

#[tokio::test]
async fn get_roles_viewer_200() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let (token, org_id, role_id) = app::auth_with_profile(&router, "viewer@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let scope = scope_headers(org_id.as_str(), role_id.as_str());
  let (status, body) = app::test_request(&router, "GET", "/api/dashboard/roles", Some(&token), None, Some(&scope))
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  assert!(json["roles"].as_array().is_some());
}

#[tokio::test]
async fn post_roles_anon_401() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let body = r#"{"name":"custom","display_name":"Custom"}"#;
  let (status, _) = app::test_request(&router, "POST", "/api/dashboard/roles", None, Some(body), None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn post_roles_viewer_403() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let (token, org_id, role_id) = app::auth_with_profile(&router, "viewer@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let scope = scope_headers(org_id.as_str(), role_id.as_str());
  let body = r#"{"name":"custom","display_name":"Custom"}"#;
  let (status, _) = app::test_request(
    &router,
    "POST",
    "/api/dashboard/roles",
    Some(&token),
    Some(body),
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn post_roles_editor_201() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let (token, org_id, role_id) = app::auth_with_profile(&router, "editor@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let scope = scope_headers(org_id.as_str(), role_id.as_str());
  let body = r#"{"name":"testrole123","display_name":"Test Role"}"#;
  let (status, _) = app::test_request(
    &router,
    "POST",
    "/api/dashboard/roles",
    Some(&token),
    Some(body),
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn patch_roles_viewer_403() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let (token, org_id, role_id) = app::auth_with_profile(&router, "viewer@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let scope = scope_headers(org_id.as_str(), role_id.as_str());
  let (_, body) = app::test_request(&router, "GET", "/api/dashboard/roles", Some(&token), None, Some(&scope))
    .await
    .unwrap();
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let roles = json["roles"].as_array().unwrap();
  let role_id = roles.first().and_then(|r| r["id"].as_str()).unwrap_or("00000000-0000-0000-0000-000000000000");
  let patch_body = format!(r#"{{"id":"{}","display_name":"Updated"}}"#, role_id);
  let (status, _) = app::test_request(
    &router,
    "PATCH",
    "/api/dashboard/roles",
    Some(&token),
    Some(&patch_body),
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn patch_roles_editor_200() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let (token, org_id, role_id) = app::auth_with_profile(&router, "editor@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let scope = scope_headers(org_id.as_str(), role_id.as_str());
  let (_, body) = app::test_request(&router, "GET", "/api/dashboard/roles", Some(&token), None, Some(&scope))
    .await
    .unwrap();
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let roles = json["roles"].as_array().unwrap();
  let role_id = roles.first().and_then(|r| r["id"].as_str()).unwrap();
  let patch_body = format!(r#"{{"id":"{}","display_name":"Updated Display"}}"#, role_id);
  let (status, _) = app::test_request(
    &router,
    "PATCH",
    "/api/dashboard/roles",
    Some(&token),
    Some(&patch_body),
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn delete_roles_viewer_403() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let (token, org_id, role_id) = app::auth_with_profile(&router, "viewer@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let scope = scope_headers(org_id.as_str(), role_id.as_str());
  let body = r#"{"id":"00000000-0000-0000-0000-000000000000"}"#;
  let (status, _) = app::test_request(
    &router,
    "DELETE",
    "/api/dashboard/roles",
    Some(&token),
    Some(body),
    Some(&scope),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::FORBIDDEN);
}
