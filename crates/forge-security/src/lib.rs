//! OWASP-aligned security headers middleware.
//! Sets X-Content-Type-Options, X-Frame-Options, Referrer-Policy, CSP (frame-ancestors),
//! Permissions-Policy, and Cross-Origin-Resource-Policy on all responses.

use axum::http::{HeaderMap, HeaderName, HeaderValue};
use axum::response::Response;

/// Injects OWASP-recommended security headers into a response.
/// Used with `tower::ServiceBuilder::map_response`.
pub fn add_security_headers<B>(mut res: Response<B>) -> Response<B> {
  add_security_headers_to_map(res.headers_mut());
  res
}

/// Inserts or removes OWASP-recommended security headers in the given map (used by [`add_security_headers`]).
fn add_security_headers_to_map(headers: &mut HeaderMap) {
  headers.remove(HeaderName::from_static("server"));
  headers.remove(HeaderName::from_static("x-powered-by"));

  headers.insert(
    HeaderName::from_static("x-content-type-options"),
    HeaderValue::from_static("nosniff"),
  );
  headers.insert(
    HeaderName::from_static("x-frame-options"),
    HeaderValue::from_static("DENY"),
  );
  headers.insert(
    HeaderName::from_static("referrer-policy"),
    HeaderValue::from_static("strict-origin-when-cross-origin"),
  );
  headers.insert(
    HeaderName::from_static("content-security-policy"),
    HeaderValue::from_static("default-src 'none'; frame-ancestors 'none'"),
  );
  headers.insert(
    HeaderName::from_static("permissions-policy"),
    HeaderValue::from_static("geolocation=(), camera=(), microphone=()"),
  );
  headers.insert(
    HeaderName::from_static("cross-origin-resource-policy"),
    HeaderValue::from_static("same-site"),
  );
  headers.insert(
    HeaderName::from_static("cross-origin-opener-policy"),
    HeaderValue::from_static("same-origin"),
  );
}

#[cfg(test)]
mod tests {
  use super::*;
  use axum::body::Body;
  use axum::response::Response;

  #[test]
  fn add_security_headers_sets_owasp_headers() {
    let res: Response<Body> = Response::builder().status(200).body(Body::empty()).unwrap();
    let res = add_security_headers(res);
    let headers = res.headers();
    assert_eq!(
      headers
        .get("x-content-type-options")
        .and_then(|v| v.to_str().ok()),
      Some("nosniff")
    );
    assert_eq!(
      headers.get("x-frame-options").and_then(|v| v.to_str().ok()),
      Some("DENY")
    );
    assert_eq!(
      headers.get("referrer-policy").and_then(|v| v.to_str().ok()),
      Some("strict-origin-when-cross-origin")
    );
    assert_eq!(
      headers
        .get("content-security-policy")
        .and_then(|v| v.to_str().ok()),
      Some("default-src 'none'; frame-ancestors 'none'")
    );
    assert_eq!(
      headers
        .get("permissions-policy")
        .and_then(|v| v.to_str().ok()),
      Some("geolocation=(), camera=(), microphone=()")
    );
    assert_eq!(
      headers
        .get("cross-origin-resource-policy")
        .and_then(|v| v.to_str().ok()),
      Some("same-site")
    );
    assert_eq!(
      headers
        .get("cross-origin-opener-policy")
        .and_then(|v| v.to_str().ok()),
      Some("same-origin")
    );
  }

  #[test]
  fn add_security_headers_removes_server_identifiers() {
    let res: Response<Body> = Response::builder()
      .status(200)
      .header("server", "SomeServer/1.0")
      .header("x-powered-by", "PHP")
      .body(Body::empty())
      .unwrap();
    let res = add_security_headers(res);
    let headers = res.headers();
    assert!(headers.get("server").is_none());
    assert!(headers.get("x-powered-by").is_none());
  }

  mod bdd_tests {
    use super::*;

    /// BDD-style tests focusing on behavior rather than implementation.
    /// Tests verify OWASP security headers, header removal, and response modification behavior.

    mod security_headers_addition_behavior {
      use super::*;

      #[test]
      fn should_add_x_content_type_options_header_when_processing_response() {
        // Given: a response without security headers
        let res: Response<Body> = Response::builder().status(200).body(Body::empty()).unwrap();

        // When: adding security headers
        let res = add_security_headers(res);

        // Then: should include X-Content-Type-Options header with nosniff value
        let headers = res.headers();
        assert_eq!(
          headers
            .get("x-content-type-options")
            .and_then(|v| v.to_str().ok()),
          Some("nosniff"),
          "Should set X-Content-Type-Options to nosniff"
        );
      }

      #[test]
      fn should_add_x_frame_options_header_when_processing_response() {
        // Given: a response without security headers
        let res: Response<Body> = Response::builder().status(200).body(Body::empty()).unwrap();

        // When: adding security headers
        let res = add_security_headers(res);

        // Then: should include X-Frame-Options header with DENY value
        let headers = res.headers();
        assert_eq!(
          headers.get("x-frame-options").and_then(|v| v.to_str().ok()),
          Some("DENY"),
          "Should set X-Frame-Options to DENY"
        );
      }

      #[test]
      fn should_add_referrer_policy_header_when_processing_response() {
        // Given: a response without security headers
        let res: Response<Body> = Response::builder().status(200).body(Body::empty()).unwrap();

        // When: adding security headers
        let res = add_security_headers(res);

        // Then: should include Referrer-Policy header with strict-origin-when-cross-origin value
        let headers = res.headers();
        assert_eq!(
          headers.get("referrer-policy").and_then(|v| v.to_str().ok()),
          Some("strict-origin-when-cross-origin"),
          "Should set Referrer-Policy to strict-origin-when-cross-origin"
        );
      }

      #[test]
      fn should_add_content_security_policy_header_when_processing_response() {
        // Given: a response without security headers
        let res: Response<Body> = Response::builder().status(200).body(Body::empty()).unwrap();

        // When: adding security headers
        let res = add_security_headers(res);

        // Then: should include Content-Security-Policy header with frame-ancestors directive
        let headers = res.headers();
        assert_eq!(
          headers
            .get("content-security-policy")
            .and_then(|v| v.to_str().ok()),
          Some("default-src 'none'; frame-ancestors 'none'"),
          "Should set Content-Security-Policy with frame-ancestors"
        );
      }

      #[test]
      fn should_add_permissions_policy_header_when_processing_response() {
        // Given: a response without security headers
        let res: Response<Body> = Response::builder().status(200).body(Body::empty()).unwrap();

        // When: adding security headers
        let res = add_security_headers(res);

        // Then: should include Permissions-Policy header restricting geolocation, camera, microphone
        let headers = res.headers();
        assert_eq!(
          headers.get("permissions-policy").and_then(|v| v.to_str().ok()),
          Some("geolocation=(), camera=(), microphone=()"),
          "Should set Permissions-Policy to restrict geolocation, camera, microphone"
        );
      }

      #[test]
      fn should_add_cross_origin_resource_policy_header_when_processing_response() {
        // Given: a response without security headers
        let res: Response<Body> = Response::builder().status(200).body(Body::empty()).unwrap();

        // When: adding security headers
        let res = add_security_headers(res);

        // Then: should include Cross-Origin-Resource-Policy header with same-site value
        let headers = res.headers();
        assert_eq!(
          headers
            .get("cross-origin-resource-policy")
            .and_then(|v| v.to_str().ok()),
          Some("same-site"),
          "Should set Cross-Origin-Resource-Policy to same-site"
        );
      }

      #[test]
      fn should_add_cross_origin_opener_policy_header_when_processing_response() {
        // Given: a response without security headers
        let res: Response<Body> = Response::builder().status(200).body(Body::empty()).unwrap();

        // When: adding security headers
        let res = add_security_headers(res);

        // Then: should include Cross-Origin-Opener-Policy header with same-origin value
        let headers = res.headers();
        assert_eq!(
          headers
            .get("cross-origin-opener-policy")
            .and_then(|v| v.to_str().ok()),
          Some("same-origin"),
          "Should set Cross-Origin-Opener-Policy to same-origin"
        );
      }

      #[test]
      fn should_add_all_owasp_recommended_headers_when_processing_response() {
        // Given: a response without security headers
        let res: Response<Body> = Response::builder().status(200).body(Body::empty()).unwrap();

        // When: adding security headers
        let res = add_security_headers(res);

        // Then: should include all OWASP-recommended security headers
        let headers = res.headers();
        assert!(
          headers.get("x-content-type-options").is_some(),
          "Should include X-Content-Type-Options"
        );
        assert!(headers.get("x-frame-options").is_some(), "Should include X-Frame-Options");
        assert!(
          headers.get("referrer-policy").is_some(),
          "Should include Referrer-Policy"
        );
        assert!(
          headers.get("content-security-policy").is_some(),
          "Should include Content-Security-Policy"
        );
        assert!(
          headers.get("permissions-policy").is_some(),
          "Should include Permissions-Policy"
        );
        assert!(
          headers.get("cross-origin-resource-policy").is_some(),
          "Should include Cross-Origin-Resource-Policy"
        );
        assert!(
          headers.get("cross-origin-opener-policy").is_some(),
          "Should include Cross-Origin-Opener-Policy"
        );
      }
    }

    mod server_identifier_removal_behavior {
      use super::*;

      #[test]
      fn should_remove_server_header_when_present() {
        // Given: a response with server header
        let res: Response<Body> = Response::builder()
          .status(200)
          .header("server", "SomeServer/1.0")
          .body(Body::empty())
          .unwrap();

        // When: adding security headers
        let res = add_security_headers(res);

        // Then: server header should be removed
        let headers = res.headers();
        assert!(
          headers.get("server").is_none(),
          "Should remove server header to prevent server fingerprinting"
        );
      }

      #[test]
      fn should_remove_x_powered_by_header_when_present() {
        // Given: a response with x-powered-by header
        let res: Response<Body> = Response::builder()
          .status(200)
          .header("x-powered-by", "PHP/8.0")
          .body(Body::empty())
          .unwrap();

        // When: adding security headers
        let res = add_security_headers(res);

        // Then: x-powered-by header should be removed
        let headers = res.headers();
        assert!(
          headers.get("x-powered-by").is_none(),
          "Should remove x-powered-by header to prevent technology disclosure"
        );
      }

      #[test]
      fn should_remove_both_server_identifiers_when_both_present() {
        // Given: a response with both server and x-powered-by headers
        let res: Response<Body> = Response::builder()
          .status(200)
          .header("server", "Nginx/1.20")
          .header("x-powered-by", "Express")
          .body(Body::empty())
          .unwrap();

        // When: adding security headers
        let res = add_security_headers(res);

        // Then: both headers should be removed
        let headers = res.headers();
        assert!(
          headers.get("server").is_none(),
          "Should remove server header"
        );
        assert!(
          headers.get("x-powered-by").is_none(),
          "Should remove x-powered-by header"
        );
      }

      #[test]
      fn should_handle_missing_server_identifiers_gracefully() {
        // Given: a response without server identifiers
        let res: Response<Body> = Response::builder().status(200).body(Body::empty()).unwrap();

        // When: adding security headers
        let res = add_security_headers(res);

        // Then: should not error and should still add security headers
        let headers = res.headers();
        assert!(
          headers.get("x-content-type-options").is_some(),
          "Should still add security headers even without server identifiers"
        );
        assert!(
          headers.get("server").is_none(),
          "Server header should remain absent"
        );
        assert!(
          headers.get("x-powered-by").is_none(),
          "X-Powered-By header should remain absent"
        );
      }
    }

    mod response_modification_behavior {
      use super::*;

      #[test]
      fn should_preserve_response_status_code() {
        // Given: a response with specific status code
        let res: Response<Body> = Response::builder().status(404).body(Body::empty()).unwrap();

        // When: adding security headers
        let res = add_security_headers(res);

        // Then: status code should be preserved
        assert_eq!(res.status(), 404, "Should preserve original status code");
      }

      #[test]
      fn should_preserve_existing_headers_not_related_to_security() {
        // Given: a response with custom headers
        let res: Response<Body> = Response::builder()
          .status(200)
          .header("custom-header", "custom-value")
          .header("another-header", "another-value")
          .body(Body::empty())
          .unwrap();

        // When: adding security headers
        let res = add_security_headers(res);

        // Then: custom headers should be preserved
        let headers = res.headers();
        assert_eq!(
          headers.get("custom-header").and_then(|v| v.to_str().ok()),
          Some("custom-value"),
          "Should preserve custom headers"
        );
        assert_eq!(
          headers.get("another-header").and_then(|v| v.to_str().ok()),
          Some("another-value"),
          "Should preserve other headers"
        );
      }

      #[test]
      fn should_overwrite_existing_security_headers() {
        // Given: a response with existing security headers
        let res: Response<Body> = Response::builder()
          .status(200)
          .header("x-frame-options", "SAMEORIGIN")
          .header("x-content-type-options", "sniff")
          .body(Body::empty())
          .unwrap();

        // When: adding security headers
        let res = add_security_headers(res);

        // Then: security headers should be overwritten with correct values
        let headers = res.headers();
        assert_eq!(
          headers.get("x-frame-options").and_then(|v| v.to_str().ok()),
          Some("DENY"),
          "Should overwrite X-Frame-Options with DENY"
        );
        assert_eq!(
          headers
            .get("x-content-type-options")
            .and_then(|v| v.to_str().ok()),
          Some("nosniff"),
          "Should overwrite X-Content-Type-Options with nosniff"
        );
      }

      #[test]
      fn should_return_response_with_same_body_type() {
        // Given: a response with body
        let res: Response<Body> = Response::builder()
          .status(200)
          .body(Body::empty())
          .unwrap();

        // When: adding security headers
        let res = add_security_headers(res);

        // Then: response should still have body (type preserved)
        // Body type is preserved through the transformation
        assert_eq!(res.status(), 200, "Response should be valid");
        assert!(
          res.headers().get("x-content-type-options").is_some(),
          "Security headers should be added"
        );
      }
    }
  }
}
