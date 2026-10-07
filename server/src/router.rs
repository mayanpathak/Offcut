//! The route table, the body limits and the layers every request passes.

use std::any::Any;

use axum::extract::{DefaultBodyLimit, Request, State};
use axum::http::HeaderValue;
use axum::http::header::{CONTENT_TYPE, HeaderName};
use axum::middleware::{Next, from_fn, from_fn_with_state};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use tower::ServiceBuilder;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::trace::{DefaultOnResponse, TraceLayer};
use uuid::Uuid;

use crate::error::AppError;
use crate::rate_limit::{RouteGroup, by_ip};
use crate::state::AppState;
use crate::{account, analytics, headers, log};

const KB: usize = 1024;
/// Every route, unless its row in the table says otherwise (TS §22.1).
const DEFAULT_BODY_LIMIT: usize = 16 * KB;

const X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

pub fn build(state: AppState) -> Router {
    // Layer 7, per route: the rate limit of one group, by client IP.
    let ip = |group: RouteGroup| {
        let limit =
            move |state: State<AppState>, req: Request, next: Next| by_ip(group, state, req, next);
        from_fn_with_state(state.clone(), limit)
    };
    // Layer 6, per route: a body limit other than the default.
    let body = DefaultBodyLimit::max;

    // The table of TS §22.1, in its order. The twelve routes of V6 are
    // comments: route, body limit, rate limit group.
    let api = Router::new()
        .route("/healthz", get(healthz).layer(ip(RouteGroup::Healthz)))
        // POST /auth/magic-link    1 kB     MagicLinkEmail, MagicLinkIp
        // POST /auth/verify        1 kB     Verify
        // POST /auth/refresh       0        Refresh
        // POST /auth/logout        0        Logout
        // GET  /me                 none     Me
        // GET  /entitlement        none     Entitlement
        // POST /usage/receipts     1 kB     UsageReceipts
        // POST /billing/checkout   1 kB     Checkout
        // GET  /billing/portal     none     Portal
        // POST /billing/webhook    64 kB    Webhook
        .route(
            "/events",
            post(analytics::ingest).layer(ip(RouteGroup::Events)),
        )
        .route(
            "/notify-me",
            post(account::notify::notify_me).layer((body(KB), ip(RouteGroup::NotifyMe))),
        );
    // POST /account/delete     0        AccountDelete
    // GET  /account/export     none     AccountExport

    let app = Router::new()
        .nest("/api/v1", api)
        .fallback(not_found)
        .method_not_allowed_fallback(not_found);
    with_layers(app).with_state(state)
}

/// Layers 1 to 6 of v1implementation §10.11, outermost first. They are added
/// with `Router::layer`, so they run after routing and the trace layer knows
/// the route template.
fn with_layers<S: Clone + Send + Sync + 'static>(router: Router<S>) -> Router<S> {
    let trace = TraceLayer::new_for_http()
        .make_span_with(log::request_span)
        // The default level is DEBUG, which `LOG_LEVEL=info` would hide.
        .on_response(DefaultOnResponse::new().level(tracing::Level::INFO));
    let layers = ServiceBuilder::new()
        .layer(from_fn(request_id))
        .layer(trace)
        .layer(CatchPanicLayer::custom(panic_response))
        .layer(headers::layer())
        .layer(from_fn(media_guard))
        .layer(DefaultBodyLimit::max(DEFAULT_BODY_LIMIT));
    router.layer(layers)
}

/// Liveness. Touches no database, so it answers while the database is down.
async fn healthz(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "ok": true, "version": state.config.git_sha }))
}

async fn not_found() -> AppError {
    AppError::NotFound
}

/// Layer 1. Gives the request a fresh id and returns it as `X-Request-Id`.
/// An id sent by the client is replaced, so the logged id is always ours.
async fn request_id(mut req: Request, next: Next) -> Response {
    let id = HeaderValue::try_from(Uuid::new_v4().to_string()).ok();
    match &id {
        Some(id) => req.headers_mut().insert(X_REQUEST_ID, id.clone()),
        None => req.headers_mut().remove(X_REQUEST_ID),
    };
    let mut response = next.run(req).await;
    if let Some(id) = id {
        response.headers_mut().insert(X_REQUEST_ID, id);
    }
    response
}

/// Layer 3. A panic in a handler becomes 500 `internal`; the process goes on
/// (TS §11.3). The panic's message is not logged here: it may quote a request.
fn panic_response(_: Box<dyn Any + Send + 'static>) -> Response {
    tracing::error!("handler panicked");
    AppError::Internal.into_response()
}

/// Layer 5. The API takes no media and no file upload on any route (INV-4):
/// such a request gets 415 before a handler sees it.
async fn media_guard(req: Request, next: Next) -> Result<Response, AppError> {
    let content_type = req.headers().get(CONTENT_TYPE);
    let content_type = content_type.and_then(|value| value.to_str().ok());
    if content_type.is_some_and(is_media) {
        return Err(AppError::UnsupportedMediaType);
    }
    Ok(next.run(req).await)
}

fn is_media(content_type: &str) -> bool {
    let media_type = content_type.split(';').next().unwrap_or(content_type);
    let media_type = media_type.trim().to_ascii_lowercase();
    ["multipart/", "video/", "audio/"]
        .iter()
        .any(|prefix| media_type.starts_with(prefix))
        || media_type == "application/octet-stream"
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Method, StatusCode};
    use offcut_api_types::{ApiError, ApiErrorCode};
    use tower::ServiceExt as _;

    use super::*;

    async fn panics() -> &'static str {
        panic!("a handler bug")
    }

    /// The six layers around two plain routes. No state, so no database.
    fn app() -> Router {
        let routes = Router::new()
            .route("/ok", post(|| async { "ok" }))
            .route("/panics", get(panics))
            .fallback(not_found)
            .method_not_allowed_fallback(not_found);
        with_layers(routes)
    }

    fn request(method: Method, path: &str) -> axum::http::request::Builder {
        Request::builder().method(method).uri(path)
    }

    async fn send(request: Request) -> Response {
        app().oneshot(request).await.unwrap()
    }

    async fn api_error(response: Response) -> ApiError {
        let body = axum::body::to_bytes(response.into_body(), 64 * KB).await;
        serde_json::from_slice(&body.unwrap()).unwrap()
    }

    #[test]
    fn media_and_upload_content_types_are_recognised() {
        let media = [
            "multipart/form-data",
            "multipart/form-data; boundary=x",
            "video/mp4",
            "VIDEO/MP4",
            "audio/wav",
            " audio/ogg ; codecs=opus",
            "application/octet-stream",
            "Application/Octet-Stream; charset=binary",
        ];
        for content_type in media {
            assert!(is_media(content_type), "{content_type:?}");
        }
        let not_media = [
            "application/json",
            "application/json; charset=utf-8",
            "text/plain",
            "application/octet-streamx",
            "",
        ];
        for content_type in not_media {
            assert!(!is_media(content_type), "{content_type:?}");
        }
    }

    #[tokio::test]
    async fn a_media_request_gets_415_on_a_route_and_off_one() {
        for path in ["/ok", "/nowhere"] {
            let request = request(Method::POST, path).header(CONTENT_TYPE, "video/mp4");
            let response = send(request.body(Body::from("x")).unwrap()).await;
            assert_eq!(
                response.status(),
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "{path}"
            );
            let error = api_error(response).await;
            assert_eq!(error.code, ApiErrorCode::UnsupportedMediaType);
        }
    }

    #[tokio::test]
    async fn a_panic_becomes_500_internal_with_a_request_id() {
        let response = send(request(Method::GET, "/panics").body(Body::empty()).unwrap()).await;
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(response.headers().contains_key(X_REQUEST_ID));
        let error = api_error(response).await;
        assert_eq!(error.code, ApiErrorCode::Internal);
        assert_eq!(error.retry_after_secs, None);
    }

    #[tokio::test]
    async fn an_unknown_path_and_a_wrong_method_are_404_not_found() {
        for (method, path) in [(Method::GET, "/nowhere"), (Method::GET, "/ok")] {
            let response = send(request(method, path).body(Body::empty()).unwrap()).await;
            assert_eq!(response.status(), StatusCode::NOT_FOUND, "{path}");
            assert_eq!(response.headers()["cache-control"], "no-store");
            assert_eq!(response.headers()["x-content-type-options"], "nosniff");
            assert_eq!(api_error(response).await.code, ApiErrorCode::NotFound);
        }
    }

    #[tokio::test]
    async fn the_request_id_is_new_for_each_request_and_replaces_the_clients() {
        let sent = "client-chosen-id";
        let mut seen = Vec::new();
        for _ in 0..2 {
            let request = request(Method::POST, "/ok").header(X_REQUEST_ID, sent);
            let response = send(request.body(Body::empty()).unwrap()).await;
            assert_eq!(response.status(), StatusCode::OK);
            let id = response.headers()[X_REQUEST_ID]
                .to_str()
                .unwrap()
                .to_owned();
            assert_ne!(id, sent);
            assert!(Uuid::parse_str(&id).is_ok(), "{id}");
            seen.push(id);
        }
        assert_ne!(seen[0], seen[1]);
    }

    #[tokio::test]
    async fn a_body_over_the_default_limit_is_refused_by_the_json_extractor() {
        use crate::error::json_rejection;
        use axum::extract::rejection::JsonRejection;

        async fn takes_json(
            body: Result<Json<serde_json::Value>, JsonRejection>,
        ) -> Result<StatusCode, AppError> {
            let _json = body.map_err(json_rejection)?;
            Ok(StatusCode::NO_CONTENT)
        }
        let app = with_layers(Router::new().route("/json", post(takes_json)));
        let send = |bytes: usize| {
            // A JSON string of exactly `bytes` bytes.
            let body = format!("\"{}\"", "a".repeat(bytes - 2));
            let request = request(Method::POST, "/json").header(CONTENT_TYPE, "application/json");
            app.clone().oneshot(request.body(Body::from(body)).unwrap())
        };
        let at_limit = send(DEFAULT_BODY_LIMIT).await.unwrap();
        assert_eq!(at_limit.status(), StatusCode::NO_CONTENT);
        let over = send(DEFAULT_BODY_LIMIT + 1).await.unwrap();
        assert_eq!(over.status(), StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(api_error(over).await.code, ApiErrorCode::PayloadTooLarge);
    }
}
