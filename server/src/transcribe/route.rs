//! `POST /transcribe` (cloudimplementation §6, §7.3, D-352): one clip's
//! compressed sound in, its words out. The checks run in order and all of
//! them before the body is read, so a refused request costs neither
//! bandwidth nor budget. The audio and the words live only in this handler;
//! nothing is stored, and the log line holds a kind, a length bucket and
//! the status (D-347, D-364).

use std::net::SocketAddr;

use axum::Json;
use axum::extract::{ConnectInfo, FromRequestParts, Request, State};
use axum::http::HeaderMap;
use axum::http::header::CONTENT_TYPE;
use offcut_api_types::TranscribeResponse;
use offcut_types::{MAX_CLIP_DURATION, MAX_SOURCE_WINDOWS};
use time::OffsetDateTime;

use crate::auth::extract::AuthUser;
use crate::client_ip::client_ip;
use crate::error::AppError;
use crate::rate_limit::{RateKey, RouteGroup, limit};
use crate::router::media_type;
use crate::state::AppState;
use crate::transcribe::ogg;
use crate::transcribe::provider::TranscribeError;

/// The largest body the route reads: about 90 s at the client's 32 kbps,
/// with room to spare (D-352).
pub const MAX_AUDIO_BYTES: usize = 1024 * 1024;
/// The longest audio the route takes: a clip's limit and 2 s of slack.
pub const MAX_AUDIO_MS: u32 = MAX_CLIP_DURATION.get() + 2_000;
/// The one media type the route takes (DC-14).
pub const AUDIO_TYPE: &str = "audio/ogg";

// 92 s at the client's 32 kbps, with 10% for Ogg's pages, fits the body limit.
const _: () = assert!(MAX_AUDIO_MS == 92_000);
const _: () = assert!(92 * 32_000 / 8 * 11 / 10 < MAX_AUDIO_BYTES);

/// A coarse length, for the log line.
fn length_bucket(duration_ms: u32) -> &'static str {
    match duration_ms {
        0..30_000 => "lt30",
        30_000..60_000 => "lt60",
        _ => "lte92",
    }
}

/// The log line: a kind, a length bucket and whether it was a window of a
/// long source (V6.1, D-411; never the window's index).
fn logged_as(window: bool, kind: &'static str, duration_ms: Option<u32>) {
    match duration_ms {
        Some(ms) => tracing::info!(
            kind,
            duration_s_bucket = length_bucket(ms),
            window,
            "transcribe"
        ),
        None => tracing::info!(kind, window, "transcribe"),
    }
}

/// A window header that is not `<index>/<total>` within the limits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BadWindow;

/// The name of the header that marks a window of a long source (V6.1, D-411).
pub const WINDOW_HEADER: &str = "x-offcut-window";

/// `X-Offcut-Window: <index>/<total>`, with `0 <= index < total <= MAX_SOURCE_WINDOWS`.
/// `Ok(None)` when the header is absent: the request is a clip's, as before V6.1.
pub fn window_of(headers: &HeaderMap) -> Result<Option<(u32, u32)>, BadWindow> {
    let Some(value) = headers.get(WINDOW_HEADER) else {
        return Ok(None);
    };
    let text = value.to_str().map_err(|_| BadWindow)?;
    let (index, total) = text.split_once('/').ok_or(BadWindow)?;
    let digits = |part: &str| -> Result<u32, BadWindow> {
        if part.is_empty() || part.len() > 3 || !part.bytes().all(|b| b.is_ascii_digit()) {
            return Err(BadWindow);
        }
        part.parse().map_err(|_| BadWindow)
    };
    let (index, total) = (digits(index)?, digits(total)?);
    if index < total && total <= MAX_SOURCE_WINDOWS {
        Ok(Some((index, total)))
    } else {
        Err(BadWindow)
    }
}

pub async fn transcribe(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    request: Request,
) -> Result<Json<TranscribeResponse>, AppError> {
    // 0. V6.1 (D-411): a window of a long source says so; a malformed or out-of-range
    // header is refused here, before anything else is read.
    let Ok(window) = window_of(request.headers()) else {
        logged_as(true, "bad_window", None);
        return Err(AppError::BadRequest);
    };
    let is_window = window.is_some();
    let logged =
        |kind: &'static str, duration_ms: Option<u32>| logged_as(is_window, kind, duration_ms);

    // 1. The address's bucket, first, as the route's layer had it before V6.1: an
    // unauthenticated request still takes the address's tokens. Windows have their own.
    let ip = client_ip(
        request.headers(),
        peer.ip(),
        state.config.trusted_proxy_hops,
    );
    let ip_group = if is_window {
        RouteGroup::TranscribeWindowIp
    } else {
        RouteGroup::TranscribeIp
    };
    limit(&state, ip_group, RateKey::Ip(ip))?;

    // The bearer, after the address's bucket and before the body.
    let (mut parts, body) = request.into_parts();
    let auth = AuthUser::from_request_parts(&mut parts, &state).await?;
    let request = Request::from_parts(parts, body);

    // 2. The person's own bucket; a window counts against the window group (D-411).
    let user_group = if is_window {
        RouteGroup::TranscribeWindowUser
    } else {
        RouteGroup::TranscribeUser
    };
    limit(&state, user_group, RateKey::User(auth.user))?;

    // 3. The kill switch.
    if !state.transcriber.enabled() {
        logged("disabled", None);
        return Err(AppError::TranscribeUnavailable);
    }

    // 4. Only an Ogg file passes `media_guard` on this path; this says so again.
    let content_type = request.headers().get(CONTENT_TYPE);
    let content_type = content_type.and_then(|value| value.to_str().ok());
    if content_type.map(media_type).as_deref() != Some(AUDIO_TYPE) {
        return Err(AppError::UnsupportedMediaType);
    }

    // 5. The body, now, and at most 1 MiB of it.
    let audio = axum::body::to_bytes(request.into_body(), MAX_AUDIO_BYTES)
        .await
        .map_err(|_| AppError::PayloadTooLarge)?;

    // 6. One mono Opus stream of an allowed length.
    let info = ogg::read(&audio).map_err(|_| {
        logged("bad_audio", None);
        AppError::BadRequest
    })?;
    if info.duration_ms > MAX_AUDIO_MS {
        logged("bad_audio", None);
        return Err(AppError::BadRequest);
    }

    // 7. The budget, charged before the provider is called.
    let now = OffsetDateTime::now_utc().unix_timestamp();
    if !state.transcribe_budget.charge(info.duration_ms, now) {
        logged("budget", Some(info.duration_ms));
        return Err(AppError::TranscribeUnavailable);
    }

    // 8. The provider.
    match state.transcriber.transcribe(audio, info.duration_ms).await {
        Ok(transcript) => {
            logged("ok", Some(info.duration_ms));
            Ok(Json(TranscribeResponse {
                words: transcript.words,
                duration_ms: transcript.duration_ms,
                model: transcript.model,
            }))
        }
        Err(TranscribeError::Unavailable) => {
            logged("unavailable", Some(info.duration_ms));
            Err(AppError::TranscribeUnavailable)
        }
        Err(TranscribeError::BadAudio) => {
            logged("bad_audio", Some(info.duration_ms));
            Err(AppError::BadRequest)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers(value: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(WINDOW_HEADER, value.parse().unwrap());
        headers
    }

    #[test]
    fn the_window_header_is_index_slash_total_within_the_45_minute_count() {
        assert_eq!(window_of(&HeaderMap::new()), Ok(None));
        assert_eq!(window_of(&headers("0/34")), Ok(Some((0, 34))));
        assert_eq!(window_of(&headers("33/34")), Ok(Some((33, 34))));
        assert_eq!(window_of(&headers("34/35")), Ok(Some((34, 35))));
        for bad in [
            "34/34", "0/36", "a/b", "-1/3", "1", "/3", "3/", "0/0", " 1/3", "1/3 ", "+1/3",
            "0001/3",
        ] {
            assert_eq!(window_of(&headers(bad)), Err(BadWindow), "{bad}");
        }
    }

    #[test]
    fn lengths_are_logged_in_three_buckets() {
        assert_eq!(length_bucket(0), "lt30");
        assert_eq!(length_bucket(29_999), "lt30");
        assert_eq!(length_bucket(30_000), "lt60");
        assert_eq!(length_bucket(60_000), "lte92");
        assert_eq!(length_bucket(92_000), "lte92");
    }
}
