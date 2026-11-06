//! The route table, the body limits and the layers every request passes.

use std::any::Any;

use axum::extract::{DefaultBodyLimit, Request, State};
use axum::http::header::{CONTENT_TYPE, HeaderName};
use axum::http::{HeaderValue, Method};
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
use crate::transcribe::route::{AUDIO_TYPE, MAX_AUDIO_BYTES};
use crate::{account, analytics, auth, billing, entitlement, headers, log, transcribe, usage};
use offcut_api_types::HealthResponse;

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

    // The table of TS §22.1, in its order.
    let api = Router::new()
        .route("/healthz", get(healthz).layer(ip(RouteGroup::Healthz)))
        // `MagicLinkEmail`, the other limit of this route, is checked in its
        // handler: the key is the address it reads.
        .route(
            "/auth/magic-link",
            post(auth::request_magic_link).layer((body(KB), ip(RouteGroup::MagicLinkIp))),
        )
        .route(
            "/auth/verify",
            post(auth::verify).layer((body(KB), ip(RouteGroup::Verify))),
        )
        .route(
            "/auth/refresh",
            post(auth::refresh).layer((body(0), ip(RouteGroup::Refresh))),
        )
        .route(
            "/auth/logout",
            post(auth::logout).layer((body(0), ip(RouteGroup::Logout))),
        )
        // `Me` is limited per user in its handler.
        .route("/me", get(auth::me))
        // These two are limited per user in their handlers.
        .route("/entitlement", get(entitlement::get_entitlement))
        .route("/usage/receipts", post(usage::post_receipt).layer(body(KB)))
        // These two are limited per user in their handlers.
        .route("/billing/checkout", post(billing::checkout).layer(body(KB)))
        .route("/billing/portal", get(billing::portal))
        .route(
            "/billing/webhook",
            post(billing::webhook::handle).layer((body(64 * KB), ip(RouteGroup::Webhook))),
        )
        .route(
            "/events",
            post(analytics::ingest).layer(ip(RouteGroup::Events)),
        )
        .route(
            "/notify-me",
            post(account::notify::notify_me).layer((body(KB), ip(RouteGroup::NotifyMe))),
        )
        // The one route that takes media (DC-14): `audio/ogg`, at most 1 MiB.
        // `TranscribeUser`, its other limit, is checked in its handler.
        .route(
            "/transcribe",
            post(transcribe::route::transcribe)
                .layer((body(MAX_AUDIO_BYTES), ip(RouteGroup::TranscribeIp))),
        )
        // These two are limited per user in their handlers.
        .route(
            "/account/delete",
            post(account::delete::delete_account).layer(body(0)),
        )
        .route("/account/export", get(account::export::export_account));

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
/// It also says whether cloud transcription will try (cloudimplementation
/// D-355): the client offers it only after a server that says so.
async fn healthz(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        ok: true,
        version: state.config.git_sha.clone(),
        cloud_transcribe: state.transcriber.enabled(),
    })
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

/// Layer 5. The API takes no media and no file upload on any route (INV-4),
/// with one exception (DC-14): such a request gets 415 before a handler sees it.
async fn media_guard(req: Request, next: Next) -> Result<Response, AppError> {
    let content_type = req.headers().get(CONTENT_TYPE);
    let content_type = content_type.and_then(|value| value.to_str().ok());
    if content_type.is_some_and(is_media) && !is_transcribe_upload(&req, content_type) {
        return Err(AppError::UnsupportedMediaType);
    }
    Ok(next.run(req).await)
}

/// The one exception to INV-4 (DC-14): exactly `POST /api/v1/transcribe`
/// with `audio/ogg`.
fn is_transcribe_upload(req: &Request, content_type: Option<&str>) -> bool {
    req.method() == Method::POST
        && req.uri().path() == "/api/v1/transcribe"
        && content_type.is_some_and(|value| media_type(value) == AUDIO_TYPE)
}

/// A content type's media type: lower case, without its parameters.
pub(crate) fn media_type(content_type: &str) -> String {
    let media_type = content_type.split(';').next().unwrap_or(content_type);
    media_type.trim().to_ascii_lowercase()
}

fn is_media(content_type: &str) -> bool {
    let media_type = media_type(content_type);
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

    #[test]
    fn the_media_type_is_lower_case_without_parameters() {
        assert_eq!(media_type("AUDIO/OGG; codecs=opus"), "audio/ogg");
        assert_eq!(media_type(" audio/ogg "), "audio/ogg");
        assert_eq!(
            media_type("application/json; charset=utf-8"),
            "application/json"
        );
    }

    #[test]
    fn only_post_audio_ogg_on_the_transcribe_path_is_the_exception() {
        let req = |method: Method, path: &str| {
            Request::builder()
                .method(method)
                .uri(path)
                .body(Body::empty())
                .unwrap()
        };
        let path = "/api/v1/transcribe";
        assert!(is_transcribe_upload(
            &req(Method::POST, path),
            Some("audio/ogg")
        ));
        assert!(is_transcribe_upload(
            &req(Method::POST, path),
            Some("AUDIO/OGG; codecs=opus")
        ));
        for other in [
            "audio/wav",
            "audio/webm",
            "video/mp4",
            "multipart/form-data; boundary=x",
            "application/octet-stream",
        ] {
            assert!(
                !is_transcribe_upload(&req(Method::POST, path), Some(other)),
                "{other}"
            );
        }
        assert!(!is_transcribe_upload(&req(Method::POST, path), None));
        assert!(!is_transcribe_upload(
            &req(Method::GET, path),
            Some("audio/ogg")
        ));
        for other_path in [
            "/api/v1/events",
            "/api/v1/transcribe/",
            "/api/v1/transcribex",
            "/transcribe",
        ] {
            assert!(
                !is_transcribe_upload(&req(Method::POST, other_path), Some("audio/ogg")),
                "{other_path}"
            );
        }
    }

    #[tokio::test]
    async fn audio_ogg_is_refused_on_any_other_path() {
        let request = request(Method::POST, "/ok").header(CONTENT_TYPE, "audio/ogg");
        let response = send(request.body(Body::from("x")).unwrap()).await;
        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
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
