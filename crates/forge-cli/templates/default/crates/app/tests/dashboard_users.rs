//! Integration tests: GET/POST/PATCH/DELETE /api/dashboard/users (authz and scope).

use axum::http::StatusCode;

#[tokio::test]
async fn get_dashboard_users_anonymous_401() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let (status, _) = app::test_request(&router, "GET", "/api/dashboard/users", None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_dashboard_users_as_viewer_default_200() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = app::login_as_seed_user(&router, "viewer@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let (status, _) = app::test_request(&router, "GET", "/api/dashboard/users", Some(&cookie), None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
}

/// Org-scoped user (viewer@default.org) sees only Default org users; no memberships for "Other" org.
#[tokio::test]
async fn get_dashboard_users_viewer_default_only_sees_default_org() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = app::login_as_seed_user(&router, "viewer@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let (status, body) = app::test_request(&router, "GET", "/api/dashboard/users", Some(&cookie), None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let users = json["users"].as_array().unwrap();
  for u in users {
    let memberships = u["memberships"].as_array().unwrap();
    for m in memberships {
      let org_name = m["org_name"].as_str().unwrap();
      assert!(
        !org_name.eq_ignore_ascii_case("Other"),
        "org-scoped viewer must not see Other org in list (only current org)"
      );
    }
  }
}

#[tokio::test]
async fn post_dashboard_users_viewer_403() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = app::login_as_seed_user(&router, "viewer@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let (status, body) = app::test_request(&router, "GET", "/api/auth/session", Some(&cookie), None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let org_id = json["user"]["current_org_id"].as_str().unwrap();
  let body_post = format!(
    r#"{{"email":"new@example.com","password":"password123","org_id":"{}","role_ids":[]}}"#,
    org_id
  );
  let (status2, _) = app::test_request(
    &router,
    "POST",
    "/api/dashboard/users",
    Some(&cookie),
    Some(&body_post),
  )
  .await
  .unwrap();
  assert_eq!(status2, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn post_dashboard_users_editor_201() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = app::login_as_seed_user(&router, "editor@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let (_, profiles_body) = app::test_request(&router, "GET", "/api/auth/profiles", Some(&cookie), None)
    .await
    .unwrap();
  let profiles: app::serde_json::Value = app::serde_json::from_slice(&profiles_body).unwrap();
  let first = profiles["profiles"]
    .as_array()
    .and_then(|a| a.first())
    .expect("editor has at least one profile");
  let org_id = first["org_id"].as_str().expect("profile has org_id");
  let role_id = first["role_id"].as_str().expect("profile has role_id");
  let unique = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_millis();
  let body_post = format!(
    r#"{{"email":"newuser-{}@example.com","password":"password123","org_id":"{}","role_ids":["{}"]}}"#,
    unique, org_id, role_id
  );
  let (status, _) = app::test_request(
    &router,
    "POST",
    "/api/dashboard/users",
    Some(&cookie),
    Some(&body_post),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn patch_dashboard_users_viewer_403() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = app::login_as_seed_user(&router, "viewer@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let (_, body) = app::test_request(&router, "GET", "/api/dashboard/users", Some(&cookie), None)
    .await
    .unwrap();
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let user_id = json["users"]
    .as_array()
    .and_then(|a| a.first())
    .and_then(|u| u["id"].as_str())
    .unwrap_or("00000000-0000-0000-0000-000000000000");
  let patch_body = format!(r#"{{"id":"{}","is_active":true}}"#, user_id);
  let (status, _) = app::test_request(
    &router,
    "PATCH",
    "/api/dashboard/users",
    Some(&cookie),
    Some(&patch_body),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn patch_dashboard_users_editor_200() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = app::login_as_seed_user(&router, "editor@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let (_, body) = app::test_request(&router, "GET", "/api/dashboard/users", Some(&cookie), None)
    .await
    .unwrap();
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let user_id = json["users"]
    .as_array()
    .and_then(|a| a.first())
    .and_then(|u| u["id"].as_str())
    .expect("at least one user");
  let patch_body = format!(r#"{{"id":"{}","is_active":true}}"#, user_id);
  let (status, _) = app::test_request(
    &router,
    "PATCH",
    "/api/dashboard/users",
    Some(&cookie),
    Some(&patch_body),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn delete_dashboard_users_viewer_403() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = app::login_as_seed_user(&router, "viewer@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let (_, body) = app::test_request(&router, "GET", "/api/dashboard/users", Some(&cookie), None)
    .await
    .unwrap();
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let user_id = json["users"]
    .as_array()
    .and_then(|a| a.first())
    .and_then(|u| u["id"].as_str())
    .unwrap_or("00000000-0000-0000-0000-000000000000");
  let del_body = format!(r#"{{"id":"{}"}}"#, user_id);
  let (status, _) = app::test_request(
    &router,
    "DELETE",
    "/api/dashboard/users",
    Some(&cookie),
    Some(&del_body),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn get_users_viewer_default_only_default_org_in_memberships() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = app::login_as_seed_user(&router, "viewer@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let (status, body) = app::test_request(&router, "GET", "/api/dashboard/users", Some(&cookie), None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let users = json["users"].as_array().unwrap();
  for u in users {
    let memberships = u["memberships"].as_array().unwrap();
    for m in memberships {
      let org_name = m["org_name"].as_str().unwrap();
      assert!(!org_name.eq_ignore_ascii_case("Other"), "viewer@default.org should not see Other org in memberships");
    }
  }
}

#[tokio::test]
async fn get_users_viewer_other_only_other_org() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = app::login_as_seed_user(&router, "viewer@other.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let (status, body) = app::test_request(&router, "GET", "/api/dashboard/users", Some(&cookie), None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let users = json["users"].as_array().unwrap();
  for u in users {
    let memberships = u["memberships"].as_array().unwrap();
    for m in memberships {
      let org_name = m["org_name"].as_str().unwrap();
      assert_eq!(org_name, "Other", "viewer@other.org should only see Other org");
    }
  }
}

/// Global-scope admin (admin@admin.com has platform_admin via user_global_role) GET /api/dashboard/users
/// returns users from all orgs, including the "Other" organization.
#[tokio::test]
async fn get_dashboard_users_admin_sees_all_orgs_including_other() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = app::login_as_seed_user(&router, "admin@admin.com", app::SEED_PASSWORD)
    .await
    .expect("login");
  let (status, body) = app::test_request(&router, "GET", "/api/dashboard/users", Some(&cookie), None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let users = json["users"].as_array().unwrap();
  let has_other_org_membership = users.iter().any(|u| {
    u["memberships"]
      .as_array()
      .unwrap()
      .iter()
      .any(|m| m["org_name"].as_str().map(|s| s.eq_ignore_ascii_case("Other")).unwrap_or(false))
  });
  assert!(
    has_other_org_membership,
    "admin GET /api/dashboard/users must return users with membership in Other org (global-scope)"
  );
}

#[tokio::test]
async fn post_users_org_id_other_as_viewer_default_403() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = app::login_as_seed_user(&router, "viewer@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let body_post = r#"{"email":"x@example.com","password":"password123","org_id":"00000000-0000-0000-0000-000000000001","role_ids":[]}"#;
  let (status, _) = app::test_request(
    &router,
    "POST",
    "/api/dashboard/users",
    Some(&cookie),
    Some(body_post),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::FORBIDDEN);
}
