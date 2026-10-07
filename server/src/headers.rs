//! Headers added to every API response (TS §24.3). The API emits no CORS
//! header: the web app calls it on its own origin, through a rewrite (TS §22.2).

use axum::http::HeaderValue;
use axum::http::header::{CACHE_CONTROL, X_CONTENT_TYPE_OPTIONS};
use tower::layer::util::Stack;
use tower_http::set_header::SetResponseHeaderLayer;

pub type HeadersLayer =
    Stack<SetResponseHeaderLayer<HeaderValue>, SetResponseHeaderLayer<HeaderValue>>;

/// Sets `Cache-Control: no-store` and `X-Content-Type-Options: nosniff`,
/// replacing any value a handler set.
pub fn layer() -> HeadersLayer {
    Stack::new(
        SetResponseHeaderLayer::overriding(CACHE_CONTROL, HeaderValue::from_static("no-store")),
        SetResponseHeaderLayer::overriding(
            X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ),
    )
}

#[cfg(test)]
mod tests {
    use axum::Router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::routing::get;
    use tower::ServiceExt as _;

    use super::*;

    #[tokio::test]
    async fn every_response_gets_both_headers_and_no_cors_header() {
        let cached = || async { ([(CACHE_CONTROL, "public, max-age=3600")], "ok") };
        let app = Router::new()
            .route("/plain", get(|| async { "ok" }))
            .route("/cached", get(cached))
            .layer(layer());

        for path in ["/plain", "/cached", "/missing"] {
            let request = Request::builder().uri(path).body(Body::empty()).unwrap();
            let response = app.clone().oneshot(request).await.unwrap();
            let headers = response.headers();

            assert_eq!(headers.get_all(CACHE_CONTROL).iter().count(), 1, "{path}");
            assert_eq!(headers.get(CACHE_CONTROL).unwrap(), "no-store", "{path}");
            assert_eq!(
                headers.get(X_CONTENT_TYPE_OPTIONS).unwrap(),
                "nosniff",
                "{path}"
            );
            let cors = headers
                .keys()
                .any(|name| name.as_str().starts_with("access-control-"));
            assert!(!cors, "{path} has a CORS header");
            if path == "/missing" {
                assert_eq!(response.status(), StatusCode::NOT_FOUND);
            }
        }
    }
}
