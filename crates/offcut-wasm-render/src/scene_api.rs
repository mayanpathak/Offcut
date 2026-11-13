//! The scene of a session (TS §19.6, §40.8): the rules plan of the detected
//! events, and the scene built from the transcript, a plan the binding
//! validates, the user's choices and the time map, summed up for the page. A
//! second `#[wasm_bindgen] impl RenderSession` block; JavaScript sees one
//! class. Moved from `session.rs` (v4implementation D-150, §11.1).

use std::collections::BTreeMap;

use offcut_plan::{Dropped, PlanConfig, RejectReason, from_rules, validate};
// The scene crate, through the renderer: this crate does not depend on it (D-61).
use offcut_render::scene::safe_area::{CANVAS_HEIGHT, CANVAS_WIDTH};
use offcut_render::scene::{FontSet, Hit, Rect, Rgba, Scene, SceneInput, Theme, build_scene};
// V6.2 (P209, DC-16): the video mirror.
use offcut_render::scene::{CropRect, DrawCmd, VideoSlot};
use offcut_types::Enhance;
use offcut_types::{
    CaptionPlacement, ClipInfo, CropOffset, DetectedEvent, EditPlan, EditState, ErrorCode, EventId,
    ExportProfile, OutMs, SrcPos, StyleId, TimeMap, Transcript, WordRange,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::session::{RenderSession, scene_error_name};
use crate::{failure, from_js, to_js, to_plain, word_edits};

/// The user's choices as JavaScript holds them: the keys of `word_edits` are
/// strings there. A record V3 stored has no `caption_position` and no
/// `highlight_numbers`: they take `EditState`'s defaults (D-169, D-171).
#[derive(Deserialize)]
struct EditInput {
    word_edits: BTreeMap<String, String>,
    event_overrides: BTreeMap<EventId, bool>,
    style: StyleId,
    crop_offset: CropOffset,
    #[serde(default = "CaptionPlacement::bottom")]
    caption_position: CaptionPlacement,
    #[serde(default = "highlighted")]
    highlight_numbers: bool,
    /// V6.2 (D-485): absent in every record before V6.2 and in every `v5()` one.
    #[serde(default = "Enhance::v5")]
    enhance: Enhance,
}

/// `EditState`'s default for `highlight_numbers`, read from its one home.
fn highlighted() -> bool {
    EditState::default().highlight_numbers
}

/// What `set_scene` takes: everything but the clip, which the session has.
#[derive(Deserialize)]
struct SceneInputValue {
    transcript: Transcript,
    plan: EditPlan,
    edit: EditInput,
    profile: ExportProfile,
    #[serde(rename = "timeMap")]
    time_map: TimeMap,
}

/// A chunk of the captions as the transcript editor reads it (D-182):
/// `{ start, end, at }`, its transcript words and the program time it is
/// shown from.
#[derive(Debug, PartialEq, Serialize)]
struct ChunkValue {
    start: u32,
    end: u32,
    at: u32,
}

/// What `set_scene` answers: the plan's items and fields it left out, and
/// why, and the chunks of the captions (`Scene::chunk_ranges`).
#[derive(Serialize)]
struct SceneReport {
    dropped: Vec<Dropped>,
    chunks: Vec<ChunkValue>,
}

fn chunks_of(scene: &Scene) -> Vec<ChunkValue> {
    let chunk = |(range, at): (WordRange, OutMs)| ChunkValue {
        start: range.start.get(),
        end: range.end.get(),
        at: at.get(),
    };
    scene.chunk_ranges().into_iter().map(chunk).collect()
}

/// Where the main video's source is, as JavaScript reads it:
/// `{ at: number }` or `{ held: number }`.
#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum SourcePos {
    At(u32),
    Held(u32),
}

/// A box of the canvas, as fractions of its width and height.
#[derive(Debug, PartialEq, Serialize)]
struct CanvasBox {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl CanvasBox {
    fn of(r: Rect) -> Self {
        Self {
            x: r.x / CANVAS_WIDTH,
            y: r.y / CANVAS_HEIGHT,
            w: r.w / CANVAS_WIDTH,
            h: r.h / CANVAS_HEIGHT,
        }
    }
}

/// What a point of the canvas is on, as JavaScript reads it (D-174):
/// `{ kind: "word", word: number, rect }`, `{ kind: "captions", rect }` or
/// `{ kind: "video" }`. `rect` is the word's glyph box or the caption
/// block, which the page outlines and drags (D-183).
#[derive(Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum HitValue {
    Word { word: u32, rect: CanvasBox },
    Captions { rect: CanvasBox },
    Video,
}

/// What the point `(x, y)` is on at `t_out_ms`, `x` and `y` fractions of the
/// canvas's width and height. `None` outside the canvas.
fn hit_of(scene: &Scene, t_out_ms: u32, x: f32, y: f32) -> Option<HitValue> {
    let at = OutMs::new(t_out_ms);
    Some(match scene.hit(at, x * CANVAS_WIDTH, y * CANVAS_HEIGHT)? {
        Hit::Word(word) => HitValue::Word {
            word: word.get(),
            rect: CanvasBox::of(scene.word_box(at, word)?),
        },
        Hit::Captions => HitValue::Captions {
            rect: CanvasBox::of(scene.caption_block()),
        },
        Hit::Video => HitValue::Video,
    })
}

/// The caption shown at a time as it is drawn, for the tests of development
/// builds (D-155): its words joined by spaces, and each word's fill as
/// `#rrggbbaa`.
#[derive(Debug, PartialEq, Serialize)]
struct CaptionMirror {
    text: String,
    colors: Vec<String>,
}

/// V6.2 (P209, DC-16): what the main video shows at a time, for development
/// and test builds: `src`, the part of the source the frame shows, and
/// `crop`, the clip's crop, which `src` equals with motion off.
#[derive(Debug, PartialEq, Serialize)]
struct VideoMirror {
    src: CropRect,
    crop: CropRect,
}

fn video_mirror_of(scene: &Scene, t_out_ms: u32) -> Option<VideoMirror> {
    let list = scene.frame_at(OutMs::new(t_out_ms));
    list.cmds.iter().find_map(|cmd| {
        if let DrawCmd::Video {
            slot: VideoSlot::Main,
            src,
            ..
        } = cmd
        {
            Some(VideoMirror {
                src: *src,
                crop: scene.video_crop(),
            })
        } else {
            None
        }
    })
}

fn mirror_of(scene: &Scene, t_out_ms: u32) -> Option<CaptionMirror> {
    let words = scene.caption_words(OutMs::new(t_out_ms));
    if words.is_empty() {
        return None;
    }
    let hex = |c: Rgba| format!("#{:02x}{:02x}{:02x}{:02x}", c.r, c.g, c.b, c.a);
    Some(CaptionMirror {
        text: words
            .iter()
            .map(|(text, _)| text.as_str())
            .collect::<Vec<_>>()
            .join(" "),
        colors: words.iter().map(|(_, fill)| hex(*fill)).collect(),
    })
}

/// The plan of the detector's events: the plan the page holds and hands back
/// to `set_scene` (D-124). `transcript` is a `Transcript`, `events` the
/// `DetectedEvent[]` of `detect` or `redetect_sentence`.
#[wasm_bindgen]
pub fn rules_plan(transcript: JsValue, events: JsValue) -> Result<JsValue, JsValue> {
    let transcript: Transcript = from_js(transcript, "Transcript")?;
    let events: Vec<DetectedEvent> = from_js(events, "DetectedEvent")?;
    to_js(&from_rules(&events, &transcript))
}

/// What `validate_plan` takes (V6.2, v6.2implementation §8.1): the
/// transcript, a plan and the user's word edits, keyed by strings as
/// JavaScript holds them.
#[derive(Deserialize)]
struct PlanCheckInput {
    transcript: Transcript,
    plan: EditPlan,
    #[serde(rename = "wordEdits")]
    word_edits: BTreeMap<String, String>,
}

/// What `validate_plan` answers: how many items the plan keeps and what it
/// left out, `{ kept, dropped, extras }`, or why it was refused whole, `{ refused }`.
#[derive(Debug, PartialEq, Serialize)]
#[serde(untagged)]
enum PlanCheck {
    Valid {
        kept: u32,
        dropped: Vec<Dropped>,
        extras: ExtrasValue,
    },
    Refused {
        refused: &'static str,
    },
}

/// V6.2 (P194, D-454): the text extras that passed, for the text extras
/// panel: "the panel shows only what passed". `ValidExtras` as JavaScript reads it.
#[derive(Debug, Default, PartialEq, Serialize)]
struct ExtrasValue {
    hook: Option<String>,
    post_caption: Option<String>,
    hashtags: Vec<String>,
}

/// `validate` alone, without a scene: what `set_scene` would keep of `input`'s
/// plan. `Err` is a detail of `E_INTERNAL`: the page built every part.
fn check_plan(input: PlanCheckInput) -> Result<PlanCheck, &'static str> {
    let edits = word_edits(input.word_edits).ok_or("PlanCheckInput")?;
    match validate(
        input.plan,
        &input.transcript,
        &edits,
        &PlanConfig::default(),
    ) {
        Ok((plan, dropped)) => {
            let kept = u32::try_from(plan.items().len()).map_err(|_| "PlanCheckInput")?;
            let valid = plan.extras();
            let extras = ExtrasValue {
                hook: valid.hook.clone(),
                post_caption: valid.post_caption.clone(),
                hashtags: valid.hashtags.clone(),
            };
            Ok(PlanCheck::Valid {
                kept,
                dropped,
                extras,
            })
        }
        Err(rejected) => Ok(PlanCheck::Refused {
            refused: reject_name(rejected.reason),
        }),
    }
}

/// V6.2 (v6.2implementation D-467, §8.1): what the plan validation would keep
/// of a model's plan, before the page chooses it over the rules plan. The
/// session and its scene are not touched. `input` is
/// `{ transcript, plan, wordEdits }`; the answer `{ kept, dropped }` or
/// `{ refused: "PlanFormat" | "PlanCatalog" }`.
#[wasm_bindgen]
pub fn validate_plan(input: JsValue) -> Result<JsValue, JsValue> {
    let input: PlanCheckInput = from_js(input, "PlanCheckInput")?;
    let check = check_plan(input).map_err(|detail| failure(ErrorCode::Internal, detail))?;
    to_js(&check)
}

/// The variant's name: the `detail` of a refused plan.
fn reject_name(reason: RejectReason) -> &'static str {
    match reason {
        RejectReason::Format => "PlanFormat",
        RejectReason::Catalog => "PlanCatalog",
    }
}

/// The scene of `input` on `clip`, and what its plan left out. Every failure
/// is `E_INTERNAL`, with the detail returned here: the page built every part
/// of the input.
fn scene_of(
    input: SceneInputValue,
    clip: &ClipInfo,
) -> Result<(Scene, Vec<Dropped>), &'static str> {
    let word_edits = word_edits(input.edit.word_edits).ok_or("SceneInput")?;
    // V4 cuts nothing and holds nothing (D-164).
    if !input.time_map.is_identity() {
        return Err("TimeMapNotIdentity");
    }
    let edit = EditState {
        word_edits,
        event_overrides: input.edit.event_overrides,
        style: input.edit.style,
        crop_offset: input.edit.crop_offset,
        caption_position: input.edit.caption_position,
        highlight_numbers: input.edit.highlight_numbers,
        enhance: input.edit.enhance,
    };
    let (plan, dropped) = validate(
        input.plan,
        &input.transcript,
        &edit.word_edits,
        &PlanConfig::default(),
    )
    .map_err(|rejected| reject_name(rejected.reason))?;
    let theme = Theme::default_for(edit.style);
    let fonts = FontSet::embedded();
    let scene = build_scene(SceneInput {
        transcript: &input.transcript,
        plan: &plan,
        edit: &edit,
        clip,
        profile: &input.profile,
        time_map: &input.time_map,
        theme: &theme,
        fonts: &fonts,
    })
    .map_err(scene_error_name)?;
    Ok((scene, dropped))
}

#[wasm_bindgen]
impl RenderSession {
    /// `input` is `{ transcript, plan, edit, profile, timeMap }`; the clip is
    /// the one the session was opened with. Validates the plan against the
    /// transcript as edited, builds the scene and keeps it in place of the
    /// one before, and answers `{ dropped }`. The renderer is not touched.
    pub fn set_scene(&mut self, input: JsValue) -> Result<JsValue, JsValue> {
        let input: SceneInputValue = from_js(input, "SceneInput")?;
        let (scene, dropped) =
            scene_of(input, &self.clip).map_err(|detail| failure(ErrorCode::Internal, detail))?;
        let chunks = chunks_of(&scene);
        self.scene = Some(scene);
        to_js(&SceneReport { dropped, chunks })
    }

    /// Where the main video's source is at the program time `t_out_ms`;
    /// `null` before a scene is set.
    pub fn source_at(&self, t_out_ms: u32) -> JsValue {
        let at = self
            .scene
            .as_ref()
            .map(|scene| match scene.source_at(OutMs::new(t_out_ms)) {
                SrcPos::At(t) => SourcePos::At(t.get()),
                SrcPos::Held(t) => SourcePos::Held(t.get()),
            });
        // One number in one field always serializes.
        at.and_then(|at| to_plain(&at).ok())
            .unwrap_or(JsValue::NULL)
    }

    /// What the point `(x, y)` of the canvas is on at the program time
    /// `t_out_ms` (D-174), `x` and `y` fractions of its width and height:
    /// `{ kind: "word", word, rect }`, `{ kind: "captions", rect }`,
    /// `{ kind: "video" }`; `null` outside the canvas or before a scene is
    /// set. The worker's `hitTest`.
    pub fn hit(&self, t_out_ms: u32, x: f32, y: f32) -> JsValue {
        let hit = self
            .scene
            .as_ref()
            .and_then(|scene| hit_of(scene, t_out_ms, x, y));
        // A tag, a number and four more always serialize.
        hit.and_then(|hit| to_plain(&hit).ok())
            .unwrap_or(JsValue::NULL)
    }

    /// `{ text, colors }` of the caption shown at `t_out_ms`; `null` when
    /// none is shown or before a scene is set. Only development and test
    /// builds of the page ask (D-155).
    pub fn caption_mirror(&self, t_out_ms: u32) -> JsValue {
        let mirror = self
            .scene
            .as_ref()
            .and_then(|scene| mirror_of(scene, t_out_ms));
        // A string and a list of strings always serialize.
        mirror
            .and_then(|mirror| to_plain(&mirror).ok())
            .unwrap_or(JsValue::NULL)
    }

    /// V6.2 (P209, DC-16): `{ src, crop }` of the main video at `t_out_ms`
    /// (`src` equals `crop` with motion off); `null` before a scene is set.
    /// Only development and test builds of the page ask (D-155).
    pub fn video_mirror(&self, t_out_ms: u32) -> JsValue {
        let mirror = self
            .scene
            .as_ref()
            .and_then(|scene| video_mirror_of(scene, t_out_ms));
        // Two plain rectangles always serialize.
        mirror
            .and_then(|mirror| to_plain(&mirror).ok())
            .unwrap_or(JsValue::NULL)
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

#[cfg(test)]
mod tests {
    use offcut_types::{BitsPerSec, DurMs, Hz, Orientation, ProfileKind, Px, Rotation};
    use serde_json::{Value, json};

    use super::*;

    fn clip() -> ClipInfo {
        ClipInfo {
            duration: DurMs::new(10_000),
            display_width: Px::new(1080),
            display_height: Px::new(1920),
            rotation: Rotation::R0,
            orientation: Orientation::Portrait,
            is_vfr: false,
            video_codec_string: "avc1.640028".to_owned(),
            audio_codec_string: "mp4a.40.2".to_owned(),
            audio_sample_rate: Hz::new(48_000),
            audio_channels: 2,
        }
    }

    /// The input the page sends, as JSON, with `change` made to it.
    fn input(change: impl FnOnce(&mut Value)) -> SceneInputValue {
        let profile = ExportProfile {
            kind: ProfileKind::Preview,
            width: Px::new(540),
            height: Px::new(960),
            watermark: false,
            video_bitrate: BitsPerSec::new(4_000_000),
            audio_bitrate: BitsPerSec::new(128_000),
        };
        let transcript = Transcript {
            words: Vec::new(),
            sentences: Vec::new(),
            numbers: Vec::new(),
            model_version: "test".to_owned(),
        };
        let plan = from_rules(&[], &transcript);
        let mut value = json!({
            "transcript": transcript,
            "plan": plan,
            "edit": { "word_edits": {}, "event_overrides": {}, "style": "clean", "crop_offset": 0.0 },
            "profile": profile,
            "timeMap": TimeMap::identity(DurMs::new(10_000)),
        });
        change(&mut value);
        serde_json::from_value(value).unwrap()
    }

    fn refused(input: SceneInputValue) -> &'static str {
        match scene_of(input, &clip()) {
            Ok(_) => panic!("the input was not refused"),
            Err(detail) => detail,
        }
    }

    #[test]
    fn the_rules_plan_on_the_identity_map_builds_a_scene() {
        let (scene, dropped) = scene_of(input(|_| {}), &clip()).unwrap();
        assert_eq!(dropped, []);
        assert_eq!(scene.duration(), DurMs::new(10_000));
        assert_eq!(
            scene.source_at(OutMs::new(33)),
            SrcPos::At(offcut_types::TimeMs::new(33))
        );
        let chunks = chunks_of(&scene);
        let report = serde_json::to_value(SceneReport { dropped, chunks }).unwrap();
        // No words, no chunks.
        assert_eq!(report, json!({ "dropped": [], "chunks": [] }));
        let chunk = ChunkValue {
            start: 3,
            end: 7,
            at: 1_250,
        };
        assert_eq!(
            serde_json::to_value(chunk).unwrap(),
            json!({ "start": 3, "end": 7, "at": 1_250 })
        );
        let at = serde_json::to_value(SourcePos::At(33)).unwrap();
        assert_eq!(at, json!({ "at": 33 }));
    }

    #[test]
    fn a_map_that_is_not_the_identity_and_a_plan_of_another_build_are_refused() {
        // A cut between 1 s and 2 s: V4 constructs none outside tests (D-164).
        let cut = json!({ "segments": [
            { "kind": "source", "src": { "start": 0, "end": 1_000 } },
            { "kind": "source", "src": { "start": 2_000, "end": 10_000 } },
        ] });
        assert_eq!(refused(input(|v| v["timeMap"] = cut)), "TimeMapNotIdentity");
        assert_eq!(
            refused(input(|v| v["plan"]["format"] = json!(2))),
            "PlanFormat"
        );
        assert_eq!(
            // V6.2 (D-449): catalog 2 is this build's; a newer one is refused.
            refused(input(|v| v["plan"]["catalog"] = json!(3))),
            "PlanCatalog"
        );
        assert_eq!(
            refused(input(|v| v["edit"]["word_edits"] = json!({ "01": "x" }))),
            "SceneInput"
        );
    }

    #[test]
    fn a_field_v4_does_not_apply_is_dropped_and_reported() {
        // V6.2 keeps a valid hook (D-454, D-440 (2)); the music cue is still not applied (D-455).
        let music = input(|v| v["plan"]["music"] = json!({ "fits": true, "energy": "calm" }));
        let (_, dropped) = scene_of(music, &clip()).unwrap();
        let report = serde_json::to_value(SceneReport {
            dropped,
            chunks: Vec::new(),
        })
        .unwrap();
        assert_eq!(
            report,
            json!({ "dropped": [{ "kind": null, "reason": "not_in_this_build" }], "chunks": [] })
        );
    }

    #[test]
    fn a_point_is_read_as_fractions_of_the_canvas_and_answered_as_a_tagged_object() {
        let (scene, _) = scene_of(input(|_| {}), &clip()).unwrap();
        // No words: the caption block's middle is the video, and no caption
        // is mirrored; off the canvas, nothing.
        assert_eq!(hit_of(&scene, 0, 0.5, 0.5), Some(HitValue::Video));
        assert_eq!(hit_of(&scene, 0, 1.01, 0.5), None);
        assert_eq!(hit_of(&scene, 0, 0.5, -0.01), None);
        assert_eq!(mirror_of(&scene, 0), None);
        let as_json = |hit: HitValue| serde_json::to_value(hit).unwrap();
        let half = || {
            CanvasBox::of(Rect {
                x: 540.0,
                y: 960.0,
                w: 540.0,
                h: 960.0,
            })
        };
        assert_eq!(
            as_json(HitValue::Word {
                word: 7,
                rect: half()
            }),
            json!({ "kind": "word", "word": 7, "rect": { "x": 0.5, "y": 0.5, "w": 0.5, "h": 0.5 } })
        );
        assert_eq!(
            as_json(HitValue::Captions { rect: half() }),
            json!({ "kind": "captions", "rect": { "x": 0.5, "y": 0.5, "w": 0.5, "h": 0.5 } })
        );
        assert_eq!(as_json(HitValue::Video), json!({ "kind": "video" }));
        let mirror = CaptionMirror {
            text: "WE MADE IT".to_owned(),
            colors: vec!["#ffffffff".to_owned()],
        };
        assert_eq!(
            serde_json::to_value(mirror).unwrap(),
            json!({ "text": "WE MADE IT", "colors": ["#ffffffff"] })
        );
    }

    /// "We grew ten times." as a transcript of four timed words, one sentence.
    fn four_words() -> Transcript {
        serde_json::from_value(json!({
            "words": [
                { "text": "We", "start_ms": 500, "end_ms": 800, "confidence": 1.0 },
                { "text": "grew", "start_ms": 900, "end_ms": 1_200, "confidence": 1.0 },
                { "text": "ten", "start_ms": 1_300, "end_ms": 1_600, "confidence": 1.0 },
                { "text": "times.", "start_ms": 1_700, "end_ms": 2_000, "confidence": 1.0 },
            ],
            "sentences": [{ "words": { "start": 0, "end": 4 }, "start_ms": 500, "end_ms": 2_000 }],
            "numbers": [],
            "model_version": "test",
        }))
        .unwrap()
    }

    /// A model's plan of emphasis items, one on each word given.
    fn model_plan(words: &[u32]) -> EditPlan {
        use offcut_types::{
            EmphasisSize, EventKind, ItemParams, PlanItem, PlanSource, Treatment, WordIdx,
        };
        let mut plan = from_rules(&[], &four_words());
        plan.source = PlanSource::Model {
            provider: "deepseek".to_owned(),
            model: "test".to_owned(),
        };
        plan.items = (1u64..)
            .zip(words)
            .map(|(id, &word)| PlanItem {
                id: EventId::new(id),
                kind: EventKind::Emphasis,
                anchors: WordRange {
                    start: WordIdx::new(word),
                    end: WordIdx::new(word + 1),
                },
                params: ItemParams::Emphasis {
                    word: WordIdx::new(word),
                    treatment: Treatment::Colour,
                    size: EmphasisSize::Normal,
                },
                confidence: None,
            })
            .collect();
        plan
    }

    fn check(plan: EditPlan, edits: Value) -> Result<PlanCheck, &'static str> {
        let input = json!({ "transcript": four_words(), "plan": plan, "wordEdits": edits });
        check_plan(serde_json::from_value(input).unwrap())
    }

    #[test]
    fn validate_plan_counts_the_items_kept_and_reports_the_drops_without_a_session() {
        // V6.2 (P193): one item on a word, one anchored past the last word.
        let checked = check(model_plan(&[1, 9]), json!({})).unwrap();
        let PlanCheck::Valid { kept, dropped, .. } = &checked else {
            panic!("refused: {checked:?}");
        };
        assert_eq!(*kept, 1);
        assert_eq!(dropped.len(), 1);
        assert_eq!(dropped[0].kind, Some(offcut_types::EventKind::Emphasis));
        assert_eq!(
            serde_json::to_value(&checked).unwrap(),
            json!({ "kept": 1, "dropped": [{ "kind": "emphasis", "reason": serde_json::to_value(dropped[0].reason).unwrap() }], "extras": { "hook": null, "post_caption": null, "hashtags": [] } })
        );
        // Every item dropped is still an answer: kept 0.
        let none = check(model_plan(&[9]), json!({})).unwrap();
        assert!(matches!(none, PlanCheck::Valid { kept: 0, .. }), "{none:?}");
        // The rules plan of no events keeps nothing and drops nothing.
        let rules = check(from_rules(&[], &four_words()), json!({})).unwrap();
        assert_eq!(
            serde_json::to_value(rules).unwrap(),
            json!({ "kept": 0, "dropped": [], "extras": { "hook": null, "post_caption": null, "hashtags": [] } })
        );
    }

    #[test]
    fn validate_plan_answers_a_refused_plan_and_refuses_a_bad_input() {
        let mut newer = model_plan(&[1]);
        newer.catalog = offcut_plan::CATALOG_VERSION + 1;
        assert_eq!(
            serde_json::to_value(check(newer, json!({})).unwrap()).unwrap(),
            json!({ "refused": "PlanCatalog" })
        );
        let mut other = serde_json::to_value(model_plan(&[1])).unwrap();
        other["format"] = json!(2);
        assert_eq!(
            check(serde_json::from_value(other).unwrap(), json!({})).unwrap(),
            PlanCheck::Refused {
                refused: "PlanFormat"
            }
        );
        assert_eq!(
            check(model_plan(&[1]), json!({ "01": "x" })),
            Err("PlanCheckInput")
        );
    }

    #[test]
    fn validate_plan_answers_the_text_extras_that_passed_and_only_those() {
        // V6.2 (P194, D-454): a valid hook and caption kept, a bad hashtag left out.
        let mut plan = model_plan(&[1]);
        plan.extras.hook = Some(serde_json::from_value(json!("We grew ten times")).unwrap());
        plan.extras.post_caption = Some("How we grew, in one minute.".to_owned());
        plan.extras.hashtags = vec![
            serde_json::from_value(json!("#growth")).unwrap(),
            serde_json::from_value(json!("#not a tag")).unwrap(),
        ];
        let checked = serde_json::to_value(check(plan, json!({})).unwrap()).unwrap();
        assert_eq!(
            checked["extras"],
            json!({ "hook": "We grew ten times", "post_caption": "How we grew, in one minute.", "hashtags": ["#growth"] })
        );
        // D-440 (23), P206c: the valid hook with no hook item gains one (D-503).
        assert_eq!(checked["kept"], json!(2));
        assert_eq!(checked["dropped"].as_array().map(Vec::len), Some(1));
    }

    /// V6.2 (P209, DC-16): the video mirror. With motion off every frame's
    /// main video shows the clip's crop; with it on, the camera's drift moves
    /// `src` off the crop, and the crop answered stays the clip's.
    #[test]
    fn the_video_mirror_shows_the_crop_with_motion_off_and_the_drift_with_it_on() {
        let times = (0..10_000).step_by(250);
        let (off, _) = scene_of(input(|_| {}), &clip()).unwrap();
        let crop = off.video_crop();
        for t in times.clone() {
            let mirror = video_mirror_of(&off, t).unwrap();
            assert_eq!((mirror.src, mirror.crop), (crop, crop), "{t}");
        }
        let (on, _) = scene_of(
            input(|v| v["edit"]["enhance"] = json!({ "voice": "clean", "motion": true })),
            &clip(),
        )
        .unwrap();
        let moved = times
            .filter(|&t| video_mirror_of(&on, t).is_some_and(|m| m.src != crop && m.crop == crop))
            .count();
        assert!(moved > 0, "the drift moved no frame");
        assert_eq!(
            serde_json::to_value(video_mirror_of(&off, 0).unwrap()).unwrap()["crop"]["w"],
            json!(1080.0)
        );
    }
}
