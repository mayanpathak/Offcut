//! Groq's speech-to-text, `whisper-large-v3-turbo` (cloudimplementation
//! §7.4, D-342, D-359). The only file that names the provider. The response's
//! shape was read from real answers to the reference clip's audio (cloud CL
//! Entry 2; `server/tests/fixtures/groq/`), not from documents alone.
//! Neither the provider's text nor its error body reaches a log.

use std::time::Duration;

use async_trait::async_trait;
use axum::body::Bytes;
use offcut_api_types::CloudWord;
use reqwest::multipart::{Form, Part};
use serde::Deserialize;

use crate::config::Config;
use crate::transcribe::align::{Heard, align};
use crate::transcribe::provider::{CloudTranscript, TranscribeError, Transcriber};

/// The model asked for.
pub const MODEL: &str = "whisper-large-v3-turbo";
/// What the transcript records as its `model_version`.
pub const MODEL_VERSION: &str = "groq:whisper-large-v3-turbo";
/// The client gives the whole request 30 s (D-374).
const API_TIMEOUT: Duration = Duration::from_secs(25);
const MS_PER_SECOND: f64 = 1_000.0;

pub struct GroqTranscriber {
    client: reqwest::Client,
    url: String,
    key: String,
}

impl GroqTranscriber {
    /// A client that cannot be built stops the boot, as the merchant's does.
    pub fn new(config: &Config) -> Result<GroqTranscriber, TranscribeError> {
        GroqTranscriber::with_timeout(config, API_TIMEOUT)
    }

    /// As `new`, with another timeout: the tests use a short one.
    pub fn with_timeout(
        config: &Config,
        timeout: Duration,
    ) -> Result<GroqTranscriber, TranscribeError> {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|_| TranscribeError::Unavailable)?;
        Ok(GroqTranscriber {
            client,
            url: format!("{}/audio/transcriptions", config.transcribe_api_base_url),
            key: config.transcribe_api_key.clone(),
        })
    }
}

fn failed(kind: &'static str) -> TranscribeError {
    tracing::warn!(kind, "transcription request failed");
    TranscribeError::Unavailable
}

#[async_trait]
impl Transcriber for GroqTranscriber {
    fn enabled(&self) -> bool {
        true
    }

    async fn transcribe(
        &self,
        audio: Bytes,
        duration_ms: u32,
    ) -> Result<CloudTranscript, TranscribeError> {
        let length = u64::try_from(audio.len()).map_err(|_| TranscribeError::BadAudio)?;
        let file = Part::stream_with_length(reqwest::Body::from(audio), length)
            .file_name("audio.ogg")
            .mime_str("audio/ogg")
            .map_err(|_| failed("form"))?;
        let form = Form::new()
            .part("file", file)
            .text("model", MODEL)
            .text("response_format", "verbose_json")
            .text("timestamp_granularities[]", "word")
            .text("language", "en")
            .text("temperature", "0");
        let response = self
            .client
            .post(&self.url)
            .bearer_auth(&self.key)
            .multipart(form)
            .send()
            .await
            .map_err(|error| {
                failed(if error.is_timeout() {
                    "timeout"
                } else {
                    "network"
                })
            })?;
        let status = response.status().as_u16();
        if matches!(status, 400 | 413 | 415 | 422) {
            tracing::warn!(
                kind = "refused_audio",
                status,
                "transcription request failed"
            );
            return Err(TranscribeError::BadAudio);
        }
        if !response.status().is_success() {
            tracing::warn!(kind = "refused", status, "transcription request failed");
            return Err(TranscribeError::Unavailable);
        }
        let body = response.bytes().await.map_err(|_| failed("body"))?;
        parse(&body, duration_ms)
    }
}

/// The part of Groq's `verbose_json` the product uses. Everything else is ignored.
#[derive(Deserialize)]
struct Answer {
    text: String,
    #[serde(default)]
    words: Option<Vec<AnswerWord>>,
    #[serde(default)]
    duration: Option<f64>,
}

#[derive(Deserialize)]
struct AnswerWord {
    word: String,
    start: f64,
    end: f64,
}

/// Whole milliseconds from seconds; `None` for a time that is not one.
fn ms(seconds: f64) -> Option<u32> {
    let ms = (seconds * MS_PER_SECOND).round();
    if !ms.is_finite() || ms < 0.0 {
        return None;
    }
    // In range by now; a time past u32::MAX ms (49 days) is clamped.
    Some(if ms >= f64::from(u32::MAX) {
        u32::MAX
    } else {
        ms as u32
    })
}

/// A response body to the words of the clip. `sent_ms` is the audio's own
/// length, used when the answer gives none.
pub fn parse(body: &[u8], sent_ms: u32) -> Result<CloudTranscript, TranscribeError> {
    let answer: Answer = serde_json::from_slice(body).map_err(|_| failed("parse"))?;
    let mut dropped = 0usize;
    let heard: Vec<Heard> = answer
        .words
        .unwrap_or_default()
        .into_iter()
        .filter_map(|w| match (ms(w.start), ms(w.end)) {
            (Some(start_ms), Some(end_ms)) => Some(Heard {
                text: w.word,
                start_ms,
                end_ms,
            }),
            _ => {
                dropped += 1;
                None
            }
        })
        .collect();
    if dropped > 0 {
        tracing::warn!(dropped, "transcription words without a usable time");
    }
    let words = align(&answer.text, &heard)
        .into_iter()
        .map(|w| CloudWord {
            text: w.text,
            start_ms: w.start_ms,
            end_ms: w.end_ms,
        })
        .collect();
    let duration_ms = answer.duration.and_then(ms).unwrap_or(sent_ms);
    Ok(CloudTranscript {
        words,
        duration_ms,
        model: MODEL_VERSION.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seconds_become_whole_milliseconds() {
        assert_eq!(ms(0.0), Some(0));
        assert_eq!(ms(59.92), Some(59_920));
        assert_eq!(ms(1.2345), Some(1_235));
        assert_eq!(ms(-0.01), None);
        assert_eq!(ms(f64::NAN), None);
        assert_eq!(ms(f64::INFINITY), None);
        assert_eq!(ms(1e12), Some(u32::MAX));
    }

    #[test]
    fn an_answer_without_words_is_an_empty_transcript() {
        let t = parse(br#"{"text":" ."}"#, 3_000).unwrap();
        assert!(t.words.is_empty());
        assert_eq!(t.duration_ms, 3_000);
        assert_eq!(t.model, MODEL_VERSION);
    }

    #[test]
    fn a_body_that_is_not_the_answer_is_unavailable() {
        assert_eq!(
            parse(b"<html>busy</html>", 1).unwrap_err(),
            TranscribeError::Unavailable
        );
        assert_eq!(
            parse(br#"{"words":[]}"#, 1).unwrap_err(),
            TranscribeError::Unavailable
        );
    }

    #[test]
    fn a_word_without_a_usable_time_is_dropped_and_the_rest_kept() {
        let body = br#"{"text":"a b c","words":[
            {"word":"a","start":0.0,"end":0.1},
            {"word":"b","start":-1.0,"end":0.2},
            {"word":"c","start":0.3,"end":0.4}],"duration":1.0}"#;
        let t = parse(body, 9).unwrap();
        let texts: Vec<&str> = t.words.iter().map(|w| w.text.as_str()).collect();
        assert_eq!(texts, ["a", "c"]);
        assert_eq!(t.duration_ms, 1_000);
    }
}
