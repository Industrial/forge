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
    let res: Response<Body> = Response::builder()
      .status(200)
      .body(Body::empty())
      .unwrap();
    let res = add_security_headers(res);
    let headers = res.headers();
    assert_eq!(
      headers.get("x-content-type-options").and_then(|v| v.to_str().ok()),
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
      headers.get("cross-origin-resource-policy").and_then(|v| v.to_str().ok()),
      Some("same-site")
    );
    assert_eq!(
      headers.get("cross-origin-opener-policy").and_then(|v| v.to_str().ok()),
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
}
