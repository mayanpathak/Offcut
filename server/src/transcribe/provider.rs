//! What the server asks of a transcription provider (cloudimplementation
//! §7.1, D-357). Only `groq.rs` names a provider; the route and its tests
//! meet this trait alone.

use async_trait::async_trait;
use axum::body::Bytes;
use offcut_api_types::CloudWord;

/// The words of one clip's audio. Held only until the response is written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CloudTranscript {
    pub words: Vec<CloudWord>,
    pub duration_ms: u32,
    /// The provider and model, such as "groq:whisper-large-v3-turbo".
    pub model: String,
}

/// Holds no text from the provider: it could quote what was said.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum TranscribeError {
    #[error("the transcription provider cannot be used")]
    Unavailable,
    #[error("the provider refused the audio")]
    BadAudio,
}

#[async_trait]
pub trait Transcriber: Send + Sync {
    /// Whether the route should try at all. `/healthz` reports it.
    fn enabled(&self) -> bool;
    /// The words of `audio`, one Ogg Opus file of `duration_ms`.
    async fn transcribe(
        &self,
        audio: Bytes,
        duration_ms: u32,
    ) -> Result<CloudTranscript, TranscribeError>;
}

/// The provider when `TRANSCRIBE_ENABLED=false`: the route refuses, and
/// nothing is ever sent anywhere.
pub struct UnconfiguredTranscriber;

#[async_trait]
impl Transcriber for UnconfiguredTranscriber {
    fn enabled(&self) -> bool {
        false
    }

    async fn transcribe(
        &self,
        _audio: Bytes,
        _duration_ms: u32,
    ) -> Result<CloudTranscript, TranscribeError> {
        Err(TranscribeError::Unavailable)
    }
}
