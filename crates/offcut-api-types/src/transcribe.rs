//! Cloud transcription (cloudimplementation §6.2): the answer of
//! `POST /transcribe`, and the body of `GET /healthz`, which says whether the
//! route will try.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// One word as the provider heard it, with its written form ("year,", "3000").
/// Times are milliseconds from the start of the audio sent.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct CloudWord {
    pub text: String,
    pub start_ms: u32,
    pub end_ms: u32,
}

/// The answer of `POST /transcribe`. The server keeps none of it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct TranscribeResponse {
    pub words: Vec<CloudWord>,
    /// The duration of the audio the provider heard, in milliseconds.
    pub duration_ms: u32,
    /// The provider and model, such as "groq:whisper-large-v3-turbo". It
    /// becomes the transcript's `model_version`.
    pub model: String,
}

/// The body of `GET /healthz`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct HealthResponse {
    pub ok: bool,
    pub version: String,
    /// Whether `POST /transcribe` will try. Absent from a server older than
    /// the cloud transcription release, which reads as `false`.
    #[serde(default)]
    pub cloud_transcribe: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_older_servers_health_body_reads_as_no_cloud() {
        let old: HealthResponse = serde_json::from_str(r#"{"ok":true,"version":"abc"}"#).unwrap();
        assert!(!old.cloud_transcribe);
        let new: HealthResponse =
            serde_json::from_str(r#"{"ok":true,"version":"abc","cloud_transcribe":true}"#).unwrap();
        assert!(new.cloud_transcribe);
    }

    #[test]
    fn a_response_with_an_unknown_field_is_refused() {
        let good = r#"{"words":[{"text":"Hi.","start_ms":0,"end_ms":300}],"duration_ms":1000,"model":"m"}"#;
        let parsed: TranscribeResponse = serde_json::from_str(good).unwrap();
        assert_eq!(serde_json::to_string(&parsed).unwrap(), good);
        let extra = r#"{"words":[],"duration_ms":1000,"model":"m","text":"Hi."}"#;
        assert!(serde_json::from_str::<TranscribeResponse>(extra).is_err());
    }
}
