//! OWASP-aligned security headers middleware.
//! Sets X-Content-Type-Options, X-Frame-Options, Referrer-Policy, CSP (frame-ancestors),
//! Permissions-Policy, and Cross-Origin-Resource-Policy on all responses.

use axum::http::{HeaderMap, HeaderName, HeaderValue, Response};

/// Injects OWASP-recommended security headers into a response.
/// Used with `tower::ServiceBuilder::map_response`.
pub fn add_security_headers<B>(mut res: Response<B>) -> Response<B> {
  add_security_headers_to_map(res.headers_mut());
  res
}

/// Inserts or removes OWASP-recommended security headers in the given map (used by [`add_security_headers`]).
fn add_security_headers_to_map(headers: &mut HeaderMap) {
  // OWASP: reduce fingerprinting — remove or genericize server-identifying headers
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
