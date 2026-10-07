//! Tracing setup and redaction. This file is the only place that decides what
//! a request's log line may contain (TS §24.4).
//!
//! Logged per request: request id, method, route template, status, latency.
//! Never logged: email addresses, tokens, cookies, bodies, IP addresses, query
//! strings, or the raw path.

use axum::extract::MatchedPath;
use axum::http::Request;
use offcut_types::UserId;
use sha2::{Digest as _, Sha256};

/// Logged in place of a route when no route matched. The raw path is never logged.
const UNMATCHED: &str = "unmatched";

/// Starts JSON logging to stdout at `level`. Call once; a second call does nothing.
pub fn init(level: tracing::Level) {
    let subscriber = tracing_subscriber::fmt().json().with_max_level(level);
    // An error here means a logger is already installed, which is what was asked for.
    let _ = subscriber.try_init();
}

/// The span for one request, for `tower_http::trace`. It carries the request
/// id, the method and the route template; the trace layer adds status and latency.
pub fn request_span<B>(req: &Request<B>) -> tracing::Span {
    tracing::info_span!(
        "request",
        request_id = request_id(req),
        method = %req.method(),
        route = route(req),
    )
}

/// The route template, such as `/api/v1/notify-me`.
fn route<B>(req: &Request<B>) -> &str {
    let matched = req.extensions().get::<MatchedPath>();
    matched.map_or(UNMATCHED, MatchedPath::as_str)
}

/// The request id, if it looks like one. A client can send its own
/// `X-Request-Id`; text that is not a plain id is not logged.
fn request_id<B>(req: &Request<B>) -> &str {
    let id = req
        .headers()
        .get("x-request-id")
        .and_then(|id| id.to_str().ok());
    let plain = |id: &&str| {
        !id.is_empty()
            && id.len() <= 64
            && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    };
    id.filter(plain).unwrap_or("-")
}

/// How a user appears in logs: the first 8 hex characters of the SHA-256 of
/// the user id, written as a lower-case hyphenated UUID. No handler formats a
/// user id itself.
pub fn user_tag(user_id: UserId) -> String {
    let digest = Sha256::digest(user_id.get().hyphenated().to_string());
    digest
        .iter()
        .take(4)
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use axum::body::Body;

    use super::*;

    fn request(path: &str, id: Option<&str>) -> Request<Body> {
        let mut request = Request::builder().uri(path);
        if let Some(id) = id {
            request = request.header("x-request-id", id);
        }
        request.body(Body::empty()).unwrap()
    }

    #[test]
    fn user_tag_is_eight_hex_characters_of_the_sha256() {
        let id: UserId = serde_json::from_str("\"0190f3a2-7b1c-7def-8a55-0123456789ab\"").unwrap();
        let tag = user_tag(id);
        // printf '%s' 0190f3a2-7b1c-7def-8a55-0123456789ab | sha256sum
        assert_eq!(tag, "29af63ce");
        assert_eq!(tag, user_tag(id));

        let other: UserId =
            serde_json::from_str("\"0190f3a2-7b1c-7def-8a55-0123456789ac\"").unwrap();
        assert_ne!(tag, user_tag(other));
        assert!(!"0190f3a2-7b1c-7def-8a55-0123456789ab".contains(&tag));
    }

    #[test]
    fn an_unmatched_request_never_logs_its_path_or_query() {
        let req = request(
            "/api/v1/verify?token=secret-token&email=a@example.com",
            None,
        );
        assert_eq!(route(&req), UNMATCHED);
    }

    #[test]
    fn only_a_plain_request_id_is_logged() {
        let uuid = "0190f3a2-7b1c-7def-8a55-0123456789ab";
        assert_eq!(request_id(&request("/", Some(uuid))), uuid);
        assert_eq!(request_id(&request("/", None)), "-");
        for text in [
            "",
            "a@example.com",
            "two words",
            "x".repeat(65).as_str(),
            "id\"quote",
        ] {
            assert_eq!(request_id(&request("/", Some(text))), "-", "{text}");
        }
    }

    #[test]
    fn the_request_span_can_be_built_without_a_subscriber() {
        let span = request_span(&request("/api/v1/healthz?x=1", Some("abc-123")));
        drop(span);
        init(tracing::Level::INFO);
        init(tracing::Level::INFO);
    }
}
