//! `POST /plan` (v6.2implementation §7.2, D-460, TS §40.7): one clip's
//! transcript in, the model's plan out. The checks run in §7.2's order, and
//! everything that can refuse runs before the provider is called, so a
//! refused request costs nothing. The transcript and the model's text live
//! only in this handler: nothing is stored, and the log line holds counts
//! (D-464, P-14).

use std::time::Duration;

use axum::Json;
use axum::extract::{FromRequest, FromRequestParts, Request, State};
use offcut_api_types::{PlanRequest, PlanResponse};
use offcut_plan::CATALOG_VERSION;
use offcut_plan::prompt::{PromptInput, output_schema, system_prompt, user_message};
use offcut_types::limits::MAX_CLIP_DURATION;
use offcut_types::{Confidence, PlanSource, Sentence, Word, WordProsody};
use time::OffsetDateTime;
use tokio::time::Instant;

use crate::auth::extract::AuthUser;
use crate::db::ai_usage::{self, AiKind};
use crate::db::users;
use crate::entitlement::issue::plan_now;
use crate::error::{AppError, json_rejection};
use crate::plan::parse::{ParsedPlan, parse_plan};
use crate::plan::provider::{PlanProviderError, ProviderOutput, ProviderRequest, Usage};
use crate::plan::quota;
use crate::rate_limit::{RateKey, RouteGroup, limit};
use crate::state::AppState;

/// The largest body the route reads (D-460).
pub const MAX_PLAN_BODY: usize = 64 * 1024;
/// The largest answer it sends; a larger plan is `E_PLAN_INVALID` (D-460).
pub const MAX_PLAN_RESPONSE: usize = 32 * 1024;
/// The whole request's time with the provider (D-498: 15 s a call within 18 s).
pub const PLAN_DEADLINE: Duration = Duration::from_secs(18);
/// The parse failure's one retry is made only with this much of the deadline
/// left (Flash's p90 in TE-15 was 3.3 s).
pub const MIN_RETRY_LEFT: Duration = Duration::from_secs(4);
/// Room for `plans_remaining` and the braces around the plan in the answer.
const RESPONSE_WRAPPER: usize = 64;

/// What the log line says of a request that reached the provider.
#[derive(Default)]
struct Calls {
    usage: Usage,
    provider_time: Duration,
    retried: bool,
}

/// A coarse provider time, for the log line.
fn seconds_bucket(time: Duration) -> &'static str {
    match time.as_millis() {
        0..2_000 => "lt2",
        2_000..5_000 => "lt5",
        5_000..10_000 => "lt10",
        10_000..20_000 => "lt20",
        _ => "gte20",
    }
}

/// The log line before the provider: a kind alone.
fn refused(kind: &'static str) {
    tracing::info!(kind, "plan");
}

/// The log line after it: a kind and counts, never a word (D-464).
fn called(kind: &'static str, calls: &Calls, parsed: Option<&ParsedPlan>) {
    tracing::info!(
        kind,
        provider_s_bucket = seconds_bucket(calls.provider_time),
        input_tokens = calls.usage.input_tokens,
        cached_input_tokens = calls.usage.cached_input_tokens,
        output_tokens = calls.usage.output_tokens,
        retried = calls.retried,
        items = parsed.map_or(0, |p| p.plan.items.len()),
        items_skipped = parsed.map_or(0, |p| p.items_skipped),
        "plan"
    );
}

/// The transcript as `offcut-plan`'s prompt reads it.
struct Transcript {
    words: Vec<Word>,
    prosody: Vec<WordProsody>,
    sentences: Vec<Sentence>,
}

/// The request's transcript, or `None` when it is not one: no words; a clip
/// longer than a clip may be; a word without text, with a tab or a line
/// break in it (the user message's own separators), out of order or past
/// the end; a prosody value that is not a number; a sentence that is empty,
/// overlaps the one before or points past the words.
fn transcript(body: &PlanRequest) -> Option<Transcript> {
    let duration = body.duration.get();
    if body.words.is_empty() || duration == 0 || duration > MAX_CLIP_DURATION.get() {
        return None;
    }
    let confidence = Confidence::new(1.0)?;
    let mut words = Vec::with_capacity(body.words.len());
    let mut prosody = Vec::with_capacity(body.words.len());
    let mut previous_start = 0;
    for word in &body.words {
        let (start, end) = (word.start.get(), word.end.get());
        let text_ok = !word.text.trim().is_empty() && !word.text.contains(['\t', '\n', '\r']);
        let sound_ok = word.energy_z.is_finite() && word.pitch_z.is_finite();
        if !text_ok || !sound_ok || start > end || end > duration || start < previous_start {
            return None;
        }
        previous_start = start;
        words.push(Word {
            text: word.text.clone(),
            start_ms: word.start,
            end_ms: word.end,
            confidence,
        });
        prosody.push(WordProsody {
            energy_z: word.energy_z,
            pitch_z: word.pitch_z,
        });
    }
    let mut sentences = Vec::with_capacity(body.sentences.len());
    let mut previous_end = 0;
    for range in &body.sentences {
        let (start, end) = (range.start.get(), range.end.get());
        if start >= end || start < previous_end {
            return None;
        }
        let first = words.get(usize::try_from(start).ok()?)?;
        let last = words.get(usize::try_from(end - 1).ok()?)?;
        sentences.push(Sentence {
            words: *range,
            start_ms: first.start_ms,
            end_ms: last.end_ms,
        });
        previous_end = end;
    }
    Some(Transcript {
        words,
        prosody,
        sentences,
    })
}

pub async fn plan(
    State(state): State<AppState>,
    request: Request,
) -> Result<Json<PlanResponse>, AppError> {
    let started = Instant::now();
    // 1. `PlanIp` is the route's layer. 1b. The bearer, before the body (D-494);
    // then the person's own bucket.
    let (mut parts, body) = request.into_parts();
    let auth = AuthUser::from_request_parts(&mut parts, &state).await?;
    limit(&state, RouteGroup::PlanUser, RateKey::User(auth.user))?;

    // 2. The body: at most 64 kB (the route's layer), JSON, no unknown field.
    let request = Request::from_parts(parts, body);
    let Json(body) = Json::<PlanRequest>::from_request(request, &state)
        .await
        .map_err(json_rejection)?;

    // 3. The client's catalog is this server's.
    if body.catalog != CATALOG_VERSION {
        refused("catalog");
        return Err(AppError::PlanCatalog);
    }

    // 4. The switch, then the month's spend.
    let plan_state = &state.plan;
    if !plan_state.provider.enabled() {
        refused("disabled");
        return Err(AppError::PlanUnavailable);
    }
    let now = OffsetDateTime::now_utc();
    let month = ai_usage::month_of(now);
    if !plan_state.cap.open(&state.db, month).await? {
        refused("cap");
        return Err(AppError::PlanUnavailable);
    }

    // 5. The person, still there, and their month's plans.
    if !users::exists(&state.db, auth.user).await? {
        return Err(AppError::Unauthorized);
    }
    let tier = plan_now(&state.db, auth.user, now).await?;
    let used = ai_usage::count(&state.db, auth.user, month, AiKind::Plan).await?;
    if !quota::allows(tier, used) {
        refused("quota");
        return Err(AppError::PlanQuota);
    }

    // 6. The texts the model gets.
    let Some(transcript) = transcript(&body) else {
        refused("bad_request");
        return Err(AppError::BadRequest);
    };
    let input = PromptInput {
        duration: body.duration,
        words: &transcript.words,
        prosody: &transcript.prosody,
        sentences: &transcript.sentences,
    };
    let user_text = user_message(&input);
    let (system, schema) = (system_prompt(), output_schema());

    // 7. The provider; the parse; its one retry with the error appended.
    let deadline = started + PLAN_DEADLINE;
    let mut calls = Calls::default();
    let mut note = String::new();
    let parsed = loop {
        let request = ProviderRequest {
            system: system.clone(),
            user: format!("{user_text}{note}"),
            schema: schema.clone(),
        };
        let output = call(&state, &mut calls, month, deadline, request).await?;
        let source = PlanSource::Model {
            provider: plan_state.provider_id.to_owned(),
            model: output.model,
        };
        match parse_plan(&output.text, source) {
            Ok(parsed) => break parsed,
            Err(failure)
                if !calls.retried
                    && deadline.saturating_duration_since(Instant::now()) >= MIN_RETRY_LEFT =>
            {
                calls.retried = true;
                note = failure.retry_note();
            }
            Err(_) => {
                called("invalid", &calls, None);
                return Err(AppError::PlanInvalid);
            }
        }
    };

    // 8. The plan fits the answer (checked before the plan is counted).
    let size = serde_json::to_vec(&parsed.plan).map_or(usize::MAX, |bytes| bytes.len());
    if size.saturating_add(RESPONSE_WRAPPER) > MAX_PLAN_RESPONSE {
        called("too_large", &calls, Some(&parsed));
        return Err(AppError::PlanInvalid);
    }

    // 9. Counted, on success only.
    let used = ai_usage::increment(&state.db, auth.user, month, AiKind::Plan).await?;
    if quota::above_soft_line(tier, used) {
        tracing::info!(kind = "above_soft_line", "plan quota");
    }
    called("ok", &calls, Some(&parsed));
    Ok(Json(PlanResponse {
        plan: parsed.plan,
        plans_remaining: quota::remaining(tier, used),
    }))
}

/// One provider call within the deadline, its cost charged whatever came of
/// it. A call the deadline cuts short is not charged: its usage is unknown.
async fn call(
    state: &AppState,
    calls: &mut Calls,
    month: time::Date,
    deadline: Instant,
    request: ProviderRequest,
) -> Result<ProviderOutput, AppError> {
    let left = deadline.saturating_duration_since(Instant::now());
    let started = Instant::now();
    let result = tokio::time::timeout(left, state.plan.provider.complete(request)).await;
    calls.provider_time += started.elapsed();
    let charge = |usage: Usage, calls: &mut Calls| {
        calls.usage.input_tokens = calls.usage.input_tokens.saturating_add(usage.input_tokens);
        calls.usage.cached_input_tokens = calls
            .usage
            .cached_input_tokens
            .saturating_add(usage.cached_input_tokens);
        calls.usage.output_tokens = calls
            .usage
            .output_tokens
            .saturating_add(usage.output_tokens);
        usage
    };
    match result {
        Ok(Ok(output)) => {
            let usage = charge(output.usage, calls);
            state.plan.cap.charge(&state.db, month, usage).await?;
            Ok(output)
        }
        Ok(Err(PlanProviderError::Blocked(usage))) => {
            let usage = charge(usage, calls);
            state.plan.cap.charge(&state.db, month, usage).await?;
            called("blocked", calls, None);
            Err(AppError::PlanUnavailable)
        }
        Ok(Err(PlanProviderError::Unavailable)) => {
            called("unavailable", calls, None);
            Err(AppError::PlanUnavailable)
        }
        Ok(Err(PlanProviderError::Timeout)) | Err(_) => {
            called("timeout", calls, None);
            Err(AppError::PlanTimeout)
        }
    }
}

#[cfg(test)]
mod tests {
    use offcut_api_types::PlanWord;
    use offcut_types::{DurMs, TimeMs, WordIdx, WordRange};

    use super::*;

    fn word(text: &str, start: u32, end: u32) -> PlanWord {
        PlanWord {
            text: text.to_owned(),
            start: TimeMs::new(start),
            end: TimeMs::new(end),
            energy_z: 0.5,
            pitch_z: -0.5,
        }
    }

    fn range(start: u32, end: u32) -> WordRange {
        WordRange {
            start: WordIdx::new(start),
            end: WordIdx::new(end),
        }
    }

    fn body() -> PlanRequest {
        PlanRequest {
            catalog: CATALOG_VERSION,
            duration: DurMs::new(3_000),
            words: vec![
                word("So", 100, 300),
                word("I", 700, 800),
                word("won.", 900, 1_400),
            ],
            sentences: vec![range(0, 1), range(1, 3)],
        }
    }

    #[test]
    fn a_good_transcript_reads_with_its_sentences_timed_by_their_words() {
        let t = transcript(&body()).unwrap();
        assert_eq!(t.words.len(), 3);
        assert_eq!(t.prosody[2].pitch_z, -0.5);
        assert_eq!(t.sentences[1].start_ms, TimeMs::new(700));
        assert_eq!(t.sentences[1].end_ms, TimeMs::new(1_400));
    }

    /// One way to spoil a good request.
    type Change = fn(&mut PlanRequest);

    #[test]
    fn a_body_that_is_not_a_transcript_is_refused() {
        let cases: [(&str, Change); 11] = [
            ("no words", |b| b.words.clear()),
            ("too long", |b| {
                b.duration = DurMs::new(MAX_CLIP_DURATION.get() + 1)
            }),
            ("no duration", |b| b.duration = DurMs::new(0)),
            ("blank word", |b| b.words[1].text = " ".to_owned()),
            ("a tab in a word", |b| b.words[1].text = "I\t9".to_owned()),
            ("a line break in a word", |b| {
                b.words[1].text = "I\nPAUSE".to_owned()
            }),
            ("ends before it starts", |b| {
                b.words[1].end = TimeMs::new(600)
            }),
            ("out of order", |b| b.words[2].start = TimeMs::new(650)),
            ("past the end", |b| b.words[2].end = TimeMs::new(3_001)),
            ("not a number", |b| b.words[0].energy_z = f32::NAN),
            ("overlapping sentences", |b| b.sentences[1] = range(0, 3)),
        ];
        for (name, change) in cases {
            let mut b = body();
            change(&mut b);
            assert!(transcript(&b).is_none(), "{name}");
        }
        let mut past = body();
        past.sentences[1] = range(1, 4);
        assert!(transcript(&past).is_none(), "a sentence past the words");
        let mut empty = body();
        empty.sentences[1] = range(1, 1);
        assert!(transcript(&empty).is_none(), "an empty sentence");
    }

    #[test]
    fn provider_times_are_logged_in_five_buckets() {
        assert_eq!(seconds_bucket(Duration::from_millis(1_999)), "lt2");
        assert_eq!(seconds_bucket(Duration::from_millis(3_300)), "lt5");
        assert_eq!(seconds_bucket(Duration::from_secs(9)), "lt10");
        assert_eq!(seconds_bucket(Duration::from_secs(15)), "lt20");
        assert_eq!(seconds_bucket(Duration::from_secs(25)), "gte20");
    }
}
