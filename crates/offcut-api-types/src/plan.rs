//! The AI plan (TS §40.7, v6.2implementation §7.1): what the client sends to
//! `POST /plan` and what it gets back. The request holds the ASR words, never
//! the person's edits (TS §40.7); the server forwards them to the AI provider
//! and keeps none of them (P-14).

use offcut_types::{DurMs, EditPlan, TimeMs, WordRange};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// One word of the transcript, with how it was said.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct PlanWord {
    pub text: String,
    pub start: TimeMs,
    pub end: TimeMs,
    /// How much louder than the speaker's average, in standard deviations.
    pub energy_z: f32,
    /// How much higher than the speaker's average, in standard deviations.
    pub pitch_z: f32,
}

/// The body of `POST /plan`: one clip's transcript.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct PlanRequest {
    /// The client's `CATALOG_VERSION`; another than the server's is refused (409).
    pub catalog: u32,
    pub duration: DurMs,
    pub words: Vec<PlanWord>,
    /// Each sentence's words, in order.
    pub sentences: Vec<WordRange>,
}

/// The answer of `POST /plan`: the model's plan as the server read it, not
/// yet validated (the client validates it, INV-21).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct PlanResponse {
    pub plan: EditPlan,
    /// The plans left this month on a quota that blocks (Free); `None` on Creator.
    pub plans_remaining: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_request_reads_and_refuses_an_unknown_field() {
        let good = r#"{"catalog":2,"duration":1000,"words":[{"text":"Hi.","start":0,"end":300,"energy_z":0.5,"pitch_z":-0.25}],"sentences":[{"start":0,"end":1}]}"#;
        let parsed: PlanRequest = serde_json::from_str(good).unwrap();
        assert_eq!(serde_json::to_string(&parsed).unwrap(), good);
        let edited = good.replace(r#""sentences""#, r#""edits":{},"sentences""#);
        assert!(serde_json::from_str::<PlanRequest>(&edited).is_err());
        let extra_word = good.replace(r#""pitch_z":-0.25"#, r#""pitch_z":-0.25,"hidden":true"#);
        assert!(serde_json::from_str::<PlanRequest>(&extra_word).is_err());
    }
}
