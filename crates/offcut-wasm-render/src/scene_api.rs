//! The scene of a session (TS §19.6): built from the transcript, the events
//! and the user's choices, and summed up for the page. A second
//! `#[wasm_bindgen] impl RenderSession` block; JavaScript sees one class.
//! Moved from `session.rs` (v4implementation D-150).

use std::collections::BTreeMap;

// The scene crate, through the renderer: this crate does not depend on it (D-61).
use offcut_render::scene::{Scene, SceneInput, build_scene};
use offcut_types::{
    CropOffset, DetectedEvent, EditState, ErrorCode, EventId, ExportProfile, StyleId, Transcript,
};
use serde::Deserialize;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::session::{RenderSession, scene_error_name};
use crate::{failure, from_js, to_plain, word_edits};

/// The user's choices as JavaScript holds them: the keys of `word_edits` are
/// strings there.
#[derive(Deserialize)]
struct EditInput {
    word_edits: BTreeMap<String, String>,
    event_overrides: BTreeMap<EventId, bool>,
    style: StyleId,
    crop_offset: CropOffset,
}

/// A `SceneInput` by value: everything but the clip, which the session has.
#[derive(Deserialize)]
struct SceneInputValue {
    transcript: Transcript,
    events: Vec<DetectedEvent>,
    edit: EditInput,
    profile: ExportProfile,
}

#[wasm_bindgen]
impl RenderSession {
    /// `input` is `{ transcript, events, edit, profile }`; the clip is the
    /// one the session was opened with. Builds the scene and keeps it in
    /// place of the one before. The renderer is not touched.
    pub fn set_scene(&mut self, input: JsValue) -> Result<(), JsValue> {
        let input: SceneInputValue = from_js(input, "SceneInput")?;
        let word_edits = word_edits(input.edit.word_edits)
            .ok_or_else(|| failure(ErrorCode::Internal, "SceneInput"))?;
        let edit = EditState {
            word_edits,
            event_overrides: input.edit.event_overrides,
            style: input.edit.style,
            crop_offset: input.edit.crop_offset,
        };
        let scene = build_scene(SceneInput {
            transcript: &input.transcript,
            events: &input.events,
            edit: &edit,
            clip: &self.clip,
            profile: &input.profile,
        })
        .map_err(|error| failure(ErrorCode::Internal, scene_error_name(error)))?;
        self.scene = Some(scene);
        Ok(())
    }

    /// The `ChangeSummary` of the scene; `null` before a scene is set.
    pub fn summary(&self) -> JsValue {
        let summary = self.scene.as_ref().map(Scene::summary);
        // Three plain fields always serialize.
        summary
            .and_then(|summary| to_plain(&summary).ok())
            .unwrap_or(JsValue::NULL)
    }
}
