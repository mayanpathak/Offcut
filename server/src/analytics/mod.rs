//! `POST /events`: analytics ingestion (TS §22.6). The allowlist itself is the
//! `AnalyticsEvent` type; a body that is not on it never reaches `ingest`.

use axum::Json;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use offcut_api_types::{
    AnalyticsEvent, EventsBatch, MAX_COUNT, MAX_DURATION_MS, MAX_EVENTS_PER_BATCH,
};
use serde_json::Value;

use crate::db::analytics_events;
use crate::error::{AppError, json_rejection};
use crate::state::AppState;

pub mod retention;

/// Stores a batch whole or rejects it whole with 400. A row holds the
/// anonymous id, the event's name, its props and the server's receive time.
/// The IP address, the user agent and any client timestamp are neither stored
/// nor logged.
pub async fn ingest(
    State(state): State<AppState>,
    body: Result<Json<EventsBatch>, JsonRejection>,
) -> Result<StatusCode, AppError> {
    let Json(batch) = body.map_err(json_rejection)?;
    if batch.events.is_empty() || batch.events.len() > MAX_EVENTS_PER_BATCH {
        return Err(AppError::BadRequest);
    }
    if !batch.events.iter().all(within_bounds) {
        return Err(AppError::BadRequest);
    }
    let rows: Vec<(&'static str, Value)> = batch
        .events
        .iter()
        .map(|event| Ok((event.name(), props(event)?)))
        .collect::<Result<_, AppError>>()?;
    analytics_events::insert_batch(&state.db, batch.anon_id, &rows).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// The integer caps of TS §22.6: a duration is at most `MAX_DURATION_MS`, a
/// count at most `MAX_COUNT`. An event without integers is always within bounds.
fn within_bounds(event: &AnalyticsEvent) -> bool {
    let ms = |ms: &u32| *ms <= MAX_DURATION_MS;
    let count = |n: &u32| *n <= MAX_COUNT;
    match event {
        AnalyticsEvent::ModelDownload { duration_ms, .. }
        | AnalyticsEvent::StageTiming { duration_ms, .. } => ms(duration_ms),
        AnalyticsEvent::PipelineDone {
            total_ms,
            n_number,
            n_list,
            n_from_to,
            n_keyword,
        } => {
            ms(total_ms)
                && [n_number, n_list, n_from_to, n_keyword]
                    .into_iter()
                    .all(count)
        }
        AnalyticsEvent::ExportDone {
            total_ms,
            word_edits,
            events_kept,
            events_disabled,
            profile: _,
            style: _,
            crop_adjusted: _,
            from_cache: _,
        } => {
            ms(total_ms)
                && [word_edits, events_kept, events_disabled]
                    .into_iter()
                    .all(count)
        }
        AnalyticsEvent::LandingView { .. }
        | AnalyticsEvent::CapabilityCheck { .. }
        | AnalyticsEvent::ClipAccepted { .. }
        | AnalyticsEvent::ClipRejected { .. }
        | AnalyticsEvent::PreviewPlayed
        | AnalyticsEvent::ReviewAction { .. }
        | AnalyticsEvent::ExportStarted { .. }
        | AnalyticsEvent::ExportFailed { .. }
        | AnalyticsEvent::JobCancelled { .. }
        | AnalyticsEvent::PostExportAnswer { .. }
        | AnalyticsEvent::UpgradePrompt { .. }
        | AnalyticsEvent::SigninStep { .. }
        | AnalyticsEvent::CheckoutStep { .. }
        | AnalyticsEvent::ClientError { .. }
        | AnalyticsEvent::LocalDataCleared => true,
    }
}

/// The value of the `props` column: the `props` member of the event's JSON,
/// or `{}` for an event that has none.
fn props(event: &AnalyticsEvent) -> Result<Value, AppError> {
    let Ok(Value::Object(mut json)) = serde_json::to_value(event) else {
        return Err(AppError::Internal);
    };
    let props = json.remove("props");
    Ok(props.unwrap_or_else(|| Value::Object(serde_json::Map::new())))
}

#[cfg(test)]
mod tests {
    use offcut_api_types::{DownloadOutcome, ExportedAs, HeroVariant};
    use offcut_types::{PipelineStage, StyleId};
    use serde_json::json;

    use super::*;

    fn pipeline_done(total_ms: u32, counts: [u32; 4]) -> AnalyticsEvent {
        let [n_number, n_list, n_from_to, n_keyword] = counts;
        AnalyticsEvent::PipelineDone {
            total_ms,
            n_number,
            n_list,
            n_from_to,
            n_keyword,
        }
    }

    fn export_done(total_ms: u32, counts: [u32; 3]) -> AnalyticsEvent {
        let [word_edits, events_kept, events_disabled] = counts;
        AnalyticsEvent::ExportDone {
            total_ms,
            profile: ExportedAs::Free,
            word_edits,
            events_kept,
            events_disabled,
            style: StyleId::Clean,
            crop_adjusted: false,
            from_cache: false,
        }
    }

    #[test]
    fn a_duration_may_be_at_most_one_hour() {
        let download = |duration_ms| AnalyticsEvent::ModelDownload {
            outcome: DownloadOutcome::Ok,
            duration_ms,
            resumed: false,
        };
        let stage = |duration_ms| AnalyticsEvent::StageTiming {
            stage: PipelineStage::Asr,
            duration_ms,
            asr_backend: None,
        };
        for at_limit in [
            download(MAX_DURATION_MS),
            stage(MAX_DURATION_MS),
            pipeline_done(MAX_DURATION_MS, [0; 4]),
            export_done(MAX_DURATION_MS, [0; 3]),
        ] {
            assert!(within_bounds(&at_limit), "{at_limit:?}");
        }
        for over in [
            download(MAX_DURATION_MS + 1),
            stage(MAX_DURATION_MS + 1),
            pipeline_done(MAX_DURATION_MS + 1, [0; 4]),
            export_done(MAX_DURATION_MS + 1, [0; 3]),
            download(u32::MAX),
        ] {
            assert!(!within_bounds(&over), "{over:?}");
        }
    }

    #[test]
    fn each_count_may_be_at_most_ten_thousand() {
        assert!(within_bounds(&pipeline_done(0, [MAX_COUNT; 4])));
        assert!(within_bounds(&export_done(0, [MAX_COUNT; 3])));
        // One count over the cap is enough, whichever it is.
        for position in 0..4 {
            let mut counts = [0; 4];
            counts[position] = MAX_COUNT + 1;
            assert!(!within_bounds(&pipeline_done(0, counts)), "{position}");
        }
        for position in 0..3 {
            let mut counts = [0; 3];
            counts[position] = MAX_COUNT + 1;
            assert!(!within_bounds(&export_done(0, counts)), "{position}");
        }
    }

    #[test]
    fn props_are_the_props_member_or_an_empty_object() {
        let landing = AnalyticsEvent::LandingView {
            hero_variant: HeroVariant::Outcome,
        };
        assert_eq!(props(&landing).unwrap(), json!({"hero_variant": "outcome"}));
        assert_eq!(props(&AnalyticsEvent::PreviewPlayed).unwrap(), json!({}));
        assert_eq!(props(&AnalyticsEvent::LocalDataCleared).unwrap(), json!({}));
        // An optional prop that is absent is left out, not stored as null.
        let stage = AnalyticsEvent::StageTiming {
            stage: PipelineStage::Asr,
            duration_ms: 1200,
            asr_backend: None,
        };
        assert_eq!(
            props(&stage).unwrap(),
            json!({"stage": "asr", "duration_ms": 1200})
        );
    }
}
