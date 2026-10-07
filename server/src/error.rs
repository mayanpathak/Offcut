//! Every error a handler can return, and the one mapping from it to an HTTP
//! response. The response body is an `ApiError` and never echoes the request.

use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::http::header::RETRY_AFTER;
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use offcut_api_types::{ApiError, ApiErrorCode};

#[derive(Debug)]
pub enum AppError {
    BadRequest,
    NotFound,
    UnsupportedMediaType,
    PayloadTooLarge,
    RateLimited { retry_after_secs: u32 },
    Db(sqlx::Error),
    Internal,
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        Self::Db(error)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, retry_after_secs) = match self {
            Self::BadRequest => (StatusCode::BAD_REQUEST, ApiErrorCode::BadRequest, None),
            Self::NotFound => (StatusCode::NOT_FOUND, ApiErrorCode::NotFound, None),
            Self::UnsupportedMediaType => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                ApiErrorCode::UnsupportedMediaType,
                None,
            ),
            Self::PayloadTooLarge => (
                StatusCode::PAYLOAD_TOO_LARGE,
                ApiErrorCode::PayloadTooLarge,
                None,
            ),
            Self::RateLimited { retry_after_secs } => (
                StatusCode::TOO_MANY_REQUESTS,
                ApiErrorCode::RateLimited,
                Some(retry_after_secs),
            ),
            Self::Db(error) => {
                // The kind only. The error's own text can hold values from the query.
                tracing::error!(kind = db_error_kind(&error), "database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ApiErrorCode::Internal,
                    None,
                )
            }
            Self::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiErrorCode::Internal,
                None,
            ),
        };

        let body = Json(ApiError {
            code,
            retry_after_secs,
        });
        let mut response = (status, body).into_response();
        if let Some(secs) = retry_after_secs {
            response
                .headers_mut()
                .insert(RETRY_AFTER, HeaderValue::from(secs));
        }
        response
    }
}

/// A fixed word for the kind of database failure. `sqlx::Error` may gain
/// variants, so this asks about the kinds worth telling apart and calls the
/// rest "other".
pub fn db_error_kind(error: &sqlx::Error) -> &'static str {
    if error.as_database_error().is_some() {
        "database"
    } else if matches!(error, sqlx::Error::PoolTimedOut) {
        "pool_timed_out"
    } else if matches!(error, sqlx::Error::PoolClosed) {
        "pool_closed"
    } else if matches!(error, sqlx::Error::Io(_)) {
        "io"
    } else if matches!(error, sqlx::Error::Tls(_)) {
        "tls"
    } else if matches!(error, sqlx::Error::RowNotFound) {
        "row_not_found"
    } else {
        "other"
    }
}

/// Maps a rejected JSON body. A body over the size limit is `PayloadTooLarge`,
/// a wrong content type is `UnsupportedMediaType`, and every parse or
/// validation failure (unknown field, unknown event name, wrong type) is
/// `BadRequest`. Handlers take `Result<Json<T>, JsonRejection>`, so this is
/// the only path from a bad body to a response.
pub fn json_rejection(rejection: JsonRejection) -> AppError {
    let status = rejection.status();
    if status == StatusCode::PAYLOAD_TOO_LARGE {
        AppError::PayloadTooLarge
    } else if status == StatusCode::UNSUPPORTED_MEDIA_TYPE {
        AppError::UnsupportedMediaType
    } else {
        AppError::BadRequest
    }
}

#[cfg(test)]
mod tests {
    use axum::body::{Body, to_bytes};
    use axum::extract::FromRequest;
    use axum::http::Request;
    use axum::http::header::CONTENT_TYPE;
    use offcut_api_types::NotifyMeRequest;

    use super::*;

    async fn parts(error: AppError) -> (StatusCode, Option<HeaderValue>, serde_json::Value) {
        let response = error.into_response();
        let status = response.status();
        let retry_after = response.headers().get(RETRY_AFTER).cloned();
        let body = to_bytes(response.into_body(), 1024).await.unwrap();
        (status, retry_after, serde_json::from_slice(&body).unwrap())
    }

    #[tokio::test]
    async fn each_error_maps_to_its_status_and_code() {
        let cases = [
            (AppError::BadRequest, 400, "bad_request"),
            (AppError::NotFound, 404, "not_found"),
            (
                AppError::UnsupportedMediaType,
                415,
                "unsupported_media_type",
            ),
            (AppError::PayloadTooLarge, 413, "payload_too_large"),
            (AppError::Db(sqlx::Error::PoolTimedOut), 500, "internal"),
            (AppError::Internal, 500, "internal"),
        ];
        for (error, status, code) in cases {
            let (got, retry_after, body) = parts(error).await;
            assert_eq!(got.as_u16(), status);
            assert_eq!(retry_after, None);
            assert_eq!(
                body,
                serde_json::json!({ "code": code, "retry_after_secs": null })
            );
        }
    }

    #[tokio::test]
    async fn rate_limited_carries_the_wait_in_the_body_and_the_header() {
        let error = AppError::RateLimited {
            retry_after_secs: 37,
        };
        let (status, retry_after, body) = parts(error).await;
        assert_eq!(status.as_u16(), 429);
        assert_eq!(retry_after.unwrap(), "37");
        assert_eq!(
            body,
            serde_json::json!({ "code": "rate_limited", "retry_after_secs": 37 })
        );
    }

    #[test]
    fn a_database_error_becomes_a_db_error() {
        let error = AppError::from(sqlx::Error::RowNotFound);
        assert!(matches!(error, AppError::Db(sqlx::Error::RowNotFound)));
        assert_eq!(db_error_kind(&sqlx::Error::RowNotFound), "row_not_found");
        assert_eq!(db_error_kind(&sqlx::Error::PoolTimedOut), "pool_timed_out");
        assert_eq!(db_error_kind(&sqlx::Error::WorkerCrashed), "other");
    }

    /// Runs the JSON extractor on a request and maps what it rejects.
    async fn rejected(content_type: Option<&str>, body: String) -> AppError {
        let mut request = Request::builder().method("POST").uri("/");
        if let Some(content_type) = content_type {
            request = request.header(CONTENT_TYPE, content_type);
        }
        let request = request.body(Body::from(body)).unwrap();
        let result = Json::<NotifyMeRequest>::from_request(request, &()).await;
        json_rejection(result.unwrap_err())
    }

    #[tokio::test]
    async fn a_body_that_does_not_parse_or_validate_is_a_bad_request() {
        let json = Some("application/json");
        let bodies = [
            "",
            "{",
            "[]",
            r#"{"email":"a@example.com"}"#,
            r#"{"email":"a@example.com","wanted":"edge"}"#,
            r#"{"email":"a@example.com","wanted":"launch","extra":1}"#,
            r#"{"email":7,"wanted":"launch"}"#,
        ];
        for body in bodies {
            let error = rejected(json, body.to_owned()).await;
            assert!(matches!(error, AppError::BadRequest), "{body}: {error:?}");
        }
    }

    #[tokio::test]
    async fn a_wrong_content_type_is_unsupported_media_type() {
        let body = r#"{"email":"a@example.com","wanted":"launch"}"#;
        for content_type in [
            None,
            Some("text/plain"),
            Some("application/x-www-form-urlencoded"),
        ] {
            let error = rejected(content_type, body.to_owned()).await;
            assert!(
                matches!(error, AppError::UnsupportedMediaType),
                "{content_type:?}"
            );
        }
    }

    #[tokio::test]
    async fn a_body_over_the_limit_is_payload_too_large() {
        // Without a route limit the extractor's own limit applies: 2 MiB.
        let body = format!(
            r#"{{"email":"{}","wanted":"launch"}}"#,
            "a".repeat(3 * 1024 * 1024)
        );
        let error = rejected(Some("application/json"), body).await;
        assert!(matches!(error, AppError::PayloadTooLarge), "{error:?}");
    }
}
