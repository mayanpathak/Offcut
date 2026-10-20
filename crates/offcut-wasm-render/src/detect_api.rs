//! The export of `offcut-detect`: the visual events of a transcript.

use std::collections::BTreeMap;

use offcut_detect::{DetectorConfig, detect as detect_events, redetect_sentence as redetect};
use offcut_types::{DetectedEvent, ErrorCode, Prosody, SentenceIdx, Transcript};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{failure, from_js, to_js, word_edits};

/// `DetectedEvent[]`, in the order the events are spoken. `edits` is the
/// user's word edits, `{ [word number]: text }`. The detector's settings are
/// its defaults: nothing outside the detector chooses a threshold.
#[wasm_bindgen]
pub fn detect(transcript: JsValue, prosody: JsValue, edits: JsValue) -> Result<JsValue, JsValue> {
    let transcript: Transcript = from_js(transcript, "Transcript")?;
    let prosody: Prosody = from_js(prosody, "Prosody")?;
    let edits: BTreeMap<String, String> = from_js(edits, "Edits")?;
    let edits = word_edits(edits).ok_or_else(|| failure(ErrorCode::Internal, "Edits"))?;
    let events = detect_events(&transcript, &prosody, &edits, &DetectorConfig::default());
    to_js(&events)
}

/// `DetectedEvent[]` after an edit in sentence `sentence`: the events of
/// `prev` outside its scope are kept as they are, and the scope is detected
/// again (TS §17.3, D-86). A sentence past the last returns `prev`. Nothing
/// calls it before V4's review flow.
#[wasm_bindgen]
pub fn redetect_sentence(
    transcript: JsValue,
    prosody: JsValue,
    edits: JsValue,
    sentence: u32,
    prev: JsValue,
) -> Result<JsValue, JsValue> {
    let transcript: Transcript = from_js(transcript, "Transcript")?;
    let prosody: Prosody = from_js(prosody, "Prosody")?;
    let edits: BTreeMap<String, String> = from_js(edits, "Edits")?;
    let edits = word_edits(edits).ok_or_else(|| failure(ErrorCode::Internal, "Edits"))?;
    let prev: Vec<DetectedEvent> = from_js(prev, "Events")?;
    let events = redetect(
        &transcript,
        &prosody,
        &edits,
        SentenceIdx::new(sentence),
        &prev,
        &DetectorConfig::default(),
    );
    to_js(&events)
}
