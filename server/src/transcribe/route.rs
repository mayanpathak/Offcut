//! `POST /transcribe` (cloudimplementation §6, §7.3, D-352): one clip's
//! compressed sound in, its words out. The checks run in order and all of
//! them before the body is read, so a refused request costs neither
//! bandwidth nor budget. The audio and the words live only in this handler;
//! nothing is stored, and the log line holds a kind, a length bucket and
//! the status (D-347, D-364).

use axum::Json;
use axum::extract::{Request, State};
use axum::http::header::CONTENT_TYPE;
use offcut_api_types::TranscribeResponse;
use offcut_types::MAX_CLIP_DURATION;
use time::OffsetDateTime;

use crate::auth::extract::AuthUser;
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

fn logged(kind: &'static str, duration_ms: Option<u32>) {
    match duration_ms {
        Some(ms) => tracing::info!(kind, duration_s_bucket = length_bucket(ms), "transcribe"),
        None => tracing::info!(kind, "transcribe"),
    }
}

pub async fn transcribe(
    State(state): State<AppState>,
    auth: AuthUser,
    request: Request,
) -> Result<Json<TranscribeResponse>, AppError> {
    // 2. The person's own bucket (the IP's is the route's layer).
    limit(&state, RouteGroup::TranscribeUser, RateKey::User(auth.user))?;

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

    #[test]
    fn lengths_are_logged_in_three_buckets() {
        assert_eq!(length_bucket(0), "lt30");
        assert_eq!(length_bucket(29_999), "lt30");
        assert_eq!(length_bucket(30_000), "lt60");
        assert_eq!(length_bucket(60_000), "lte92");
        assert_eq!(length_bucket(92_000), "lte92");
    }
}
