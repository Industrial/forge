//! BDD tests for GET/POST /api/users and GET/POST/PATCH/DELETE /api/organizations/{id}/users.
//! Uses Bearer token and scope headers (X-Organization-Id, X-Role-Id).
//!
//! BDD-style tests focusing on behavior rather than implementation.
//! Tests are organized by feature/behavior area with descriptive names.

use axum::http::StatusCode;

fn scope_headers<'a>(org_id: &'a str, role_id: &'a str) -> [(&'static str, &'a str); 2] {
  [("X-Organization-Id", org_id), ("X-Role-Id", role_id)]
}

mod bdd_tests {
  use super::*;

  mod authentication_behavior {
    use super::*;

    #[tokio::test]
    async fn should_return_401_for_get_api_users_when_unauthenticated() {
      // Given: an unauthenticated request
      let client = app::test_client().await.expect("test_client");
      // When: requesting GET /api/users without a token
      let (status, _) = app::test_request(&client, "GET", "/api/users", None, None, None)
        .await
        .unwrap();
      // Then: should return 401 Unauthorized
      assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn should_return_403_for_get_api_users_when_viewer_without_scope() {
      // Given: an authenticated viewer user without scope headers
      let client = app::test_client().await.expect("test_client");
      let (token, _org_id, _role_id) =
        app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
          .await
          .expect("login");
      // When: requesting GET /api/users without X-Organization-Id / X-Role-Id
      let (status, _) = app::test_request(&client, "GET", "/api/users", Some(&token), None, None)
        .await
        .unwrap();
      // Then: should return 403 Forbidden
      assert_eq!(status, StatusCode::FORBIDDEN);
    }
  }

  mod get_api_users_behavior {
    use super::*;

    #[tokio::test]
    async fn should_return_200_with_users_array_when_viewer_has_scope() {
      // Given: an authenticated viewer user with scope headers
      let client = app::test_client().await.expect("test_client");
      let (token, org_id, role_id) =
        app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
          .await
          .expect("login");
      let scope = scope_headers(org_id.as_str(), role_id.as_str());
      // When: requesting GET /api/users with scope
      let (status, body) = app::test_request(
        &client,
        "GET",
        "/api/users",
        Some(&token),
        None,
        Some(&scope),
      )
      .await
      .unwrap();
      // Then: should return 200 OK with users array
      assert_eq!(status, StatusCode::OK);
      let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
      assert!(json.get("users").and_then(|u| u.as_array()).is_some());
    }

    #[tokio::test]
    async fn should_scope_viewer_to_default_org_only() {
      // Given: an authenticated viewer for default org with scope
      let client = app::test_client().await.expect("test_client");
      let (token, org_id, role_id) =
        app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
          .await
          .expect("login");
      let scope = scope_headers(org_id.as_str(), role_id.as_str());
      // When: requesting GET /api/users with that scope
      let (status, body) = app::test_request(
        &client,
        "GET",
        "/api/users",
        Some(&token),
        None,
        Some(&scope),
      )
      .await
      .unwrap();
      assert_eq!(status, StatusCode::OK);
      let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
      let users = json["users"].as_array().unwrap();
      let empty: &[app::serde_json::Value] = &[];
      // Then: returned users must not include CoolOrg in memberships
      for u in users {
        for m in u["memberships"]
          .as_array()
          .map(|v| v.as_slice())
          .unwrap_or(empty)
        {
          let org_name = m["org_name"].as_str().unwrap_or("");
          assert!(
            !org_name.eq_ignore_ascii_case("CoolOrg"),
            "viewer with Default org scope must not see CoolOrg in memberships"
          );
        }
      }
    }

    #[tokio::test]
    async fn should_scope_viewer_at_coolorg_to_coolorg_users_only() {
      // Given: an authenticated viewer for CoolOrg with scope
      let client = app::test_client().await.expect("test_client");
      let (token, org_id, role_id) =
        app::auth_with_profile(&client, "viewer@coolorg.org", app::SEED_PASSWORD)
          .await
          .expect("login");
      let scope = scope_headers(org_id.as_str(), role_id.as_str());
      // When: requesting GET /api/users with that scope
      let (status, body) = app::test_request(
        &client,
        "GET",
        "/api/users",
        Some(&token),
        None,
        Some(&scope),
      )
      .await
      .unwrap();
      assert_eq!(status, StatusCode::OK);
      let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
      let users = json["users"].as_array().unwrap();
      let empty: &[app::serde_json::Value] = &[];
      // Then: each returned user should have at least one membership in CoolOrg
      for u in users {
        let memberships = u["memberships"]
          .as_array()
          .map(|v| v.as_slice())
          .unwrap_or(empty);
        let has_coolorg = memberships.iter().any(|m| {
          m["org_name"]
            .as_str()
            .map(|s| s == "CoolOrg")
            .unwrap_or(false)
        });
        assert!(
          has_coolorg,
          "viewer@coolorg.org with CoolOrg scope: each returned user should have at least one membership in CoolOrg"
        );
      }
    }

    #[tokio::test]
    async fn should_allow_admin_to_get_all_users_without_scope() {
      // Given: an authenticated admin user (global)
      let client = app::test_client().await.expect("test_client");
      let (token, _org_id, _role_id) =
        app::auth_with_profile(&client, "admin@admin.com", app::SEED_PASSWORD)
          .await
          .expect("login");
      // When: requesting GET /api/users without scope headers
      let (status, body) =
        app::test_request(&client, "GET", "/api/users", Some(&token), None, None)
          .await
          .unwrap();
      // Then: should return 200 OK and include users with CoolOrg (seed data)
      assert_eq!(status, StatusCode::OK);
      let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
      let users = json["users"].as_array().expect("response has users array");
      let empty: &[app::serde_json::Value] = &[];
      let has_coolorg = users.iter().any(|u| {
        u["memberships"]
          .as_array()
          .map(|v| v.as_slice())
          .unwrap_or(empty)
          .iter()
          .any(|m| {
            m["org_name"]
              .as_str()
              .map(|s| s.eq_ignore_ascii_case("CoolOrg"))
              .unwrap_or(false)
          })
      });
      assert!(
        has_coolorg,
        "admin GET /api/users (global) must return users that include at least one with CoolOrg (seed data)"
      );
    }
  }

  mod post_org_users_behavior {
    use super::*;

    #[tokio::test]
    async fn should_return_403_when_viewer_posts_org_user() {
      // Given: an authenticated viewer with scope
      let client = app::test_client().await.expect("test_client");
      let (token, org_id, role_id) =
        app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
          .await
          .expect("login");
      let scope = scope_headers(org_id.as_str(), role_id.as_str());
      let body = format!(r#"{{"email":"new@example.com","password":"password123"}}"#);
      // When: posting a new user to the organization
      let (status, _) = app::test_request(
        &client,
        "POST",
        &format!("/api/organizations/{}/users", org_id),
        Some(&token),
        Some(&body),
        Some(&scope),
      )
      .await
      .unwrap();
      // Then: should return 403 Forbidden
      assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn should_return_201_when_editor_posts_org_user() {
      // Given: an authenticated editor with scope
      let client = app::test_client().await.expect("test_client");
      let (token, org_id, role_id) =
        app::auth_with_profile(&client, "editor@default.org", app::SEED_PASSWORD)
          .await
          .expect("login");
      let scope = scope_headers(org_id.as_str(), role_id.as_str());
      let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
      let body = format!(
        r#"{{"email":"newuser-{}@example.com","password":"password123"}}"#,
        unique
      );
      // When: posting a new user to the organization
      let (status, _) = app::test_request(
        &client,
        "POST",
        &format!("/api/organizations/{}/users", org_id),
        Some(&token),
        Some(&body),
        Some(&scope),
      )
      .await
      .unwrap();
      // Then: should return 201 Created
      assert_eq!(status, StatusCode::CREATED);
    }

    #[tokio::test]
    async fn should_return_403_when_posting_to_other_org_than_scope() {
      // Given: an authenticated viewer scoped to default org
      let client = app::test_client().await.expect("test_client");
      let (token, org_id, role_id) =
        app::auth_with_profile(&client, "viewer@default.org", app::SEED_PASSWORD)
          .await
          .expect("login");
      let scope = scope_headers(org_id.as_str(), role_id.as_str());
      let other_org_id = "00000000-0000-0000-0000-000000000001";
      let body = r#"{"email":"x@example.com","password":"password123"}"#;
      // When: posting to a different organization ID
      let (status, _) = app::test_request(
        &client,
        "POST",
        &format!("/api/organizations/{}/users", other_org_id),
        Some(&token),
        Some(body),
        Some(&scope),
      )
      .await
      .unwrap();
      // Then: should return 403 Forbidden
      assert_eq!(status, StatusCode::FORBIDDEN);
    }
  }

  mod get_org_users_behavior {
    use super::*;

    #[tokio::test]
    async fn should_return_200_with_roles_per_user_when_multi_profile_uses_scope() {
      // Given: a user with multiple org profiles (e.g. multi@email.com)
      let client = app::test_client().await.expect("test_client");
      let (token, _first_org_id, _first_role_id) =
        app::auth_with_profile(&client, "multi@email.com", app::SEED_PASSWORD)
          .await
          .expect("auth with profile");
      let (_, profiles_body) = app::test_request(
        &client,
        "GET",
        "/api/auth/profiles",
        Some(&token),
        None,
        None,
      )
      .await
      .unwrap();
      let json: app::serde_json::Value = app::serde_json::from_slice(&profiles_body).unwrap();
      let profiles = json["profiles"].as_array().unwrap();
      let coolorg = profiles
        .iter()
        .find(|p| p["org_name"].as_str() == Some("CoolOrg"))
        .expect("multi has CoolOrg");
      let org_id = coolorg["org_id"].as_str().unwrap();
      let role_id = coolorg["role_id"].as_str().unwrap();
      let scope = [("X-Organization-Id", org_id), ("X-Role-Id", role_id)];
      // When: requesting GET /api/organizations/{id}/users with CoolOrg scope
      let (status, body) = app::test_request(
        &client,
        "GET",
        &format!("/api/organizations/{}/users", org_id),
        Some(&token),
        None,
        Some(&scope),
      )
      .await
      .unwrap();
      // Then: should return 200 OK and each user has roles
      assert_eq!(status, StatusCode::OK);
      let users_json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
      let users = users_json["users"].as_array().expect("users array");
      for u in users {
        assert!(
          u.get("roles").and_then(|r| r.as_array()).is_some(),
          "each user has roles"
        );
      }
    }
  }
}
