//! From a transcript, a valid plan and the user's choices to what each
//! output frame draws (TS §19, §40.5). A scene is built once; `frame_at` is
//! then a pure function of the scene and a program time. Nothing here
//! touches a GPU, a pixel or a decoder, reads a clock, parses an item's
//! number (D-121; a caption word is read only to know whether it shows one,
//! D-169), or moves a timestamp except through the time map (D-126).

pub mod anim;
pub mod captions;
mod captions_draw;
pub mod display_list;
pub mod easing;
pub mod events;
pub mod fonts;
pub mod framing;
pub mod hit;
pub mod layout;
pub mod motion;
pub mod placement;
pub mod readability;
pub mod safe_area;
pub mod styles;
pub mod summary;
pub mod theme;
mod video;
pub mod watermark;

use offcut_plan::ValidPlan;
use offcut_types::{
    ChangeSummary, ClipInfo, DurMs, EditState, ExportProfile, OUTPUT_FPS, OutMs, SrcPos, TimeMap,
    Transcript, WordIdx, WordRange,
};
use thiserror::Error;

use crate::captions::Chunk;
pub use crate::display_list::{
    Affine, CropRect, DisplayList, DrawCmd, FontKey, ImageKey, PathEl, PositionedGlyph, Rect, Rgba,
    Stroke, VideoSlot,
};
use crate::events::EventTrack;
pub use crate::fonts::{FontFace, FontSet};
pub use crate::hit::Hit;
use crate::styles::StyleSpec;
pub use crate::theme::{FontRole, Role, Theme};

/// Why a scene cannot be built. All are `E_INTERNAL` to the caller: the fonts
/// are part of the program, and the time map is the program's own.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum SceneError {
    #[error("an embedded font could not be read")]
    Font,
    #[error("text could not be laid out")]
    Layout,
    #[error("an item's span has no program time")]
    TimeMap,
}

pub struct SceneInput<'a> {
    pub transcript: &'a Transcript,
    pub plan: &'a ValidPlan,
    pub edit: &'a EditState,
    pub clip: &'a ClipInfo,
    pub profile: &'a ExportProfile,
    pub time_map: &'a TimeMap,
    pub theme: &'a Theme,
    pub fonts: &'a FontSet,
}

/// Everything that is drawn over one clip, laid out in logical pixels, on
/// program time.
#[derive(Clone, Debug, PartialEq)]
pub struct Scene {
    time_map: TimeMap,
    video: video::Track,
    /// The output canvas, in output pixels.
    canvas: (f32, f32),
    /// Output pixels per logical pixel.
    scale: f32,
    style: StyleSpec,
    theme: Theme,
    fonts: FontSet,
    /// The caption block of the clip's placement, in logical pixels.
    caption_block: Rect,
    chunks: Vec<Chunk>,
    emphasis: captions_draw::Emphasis,
    tracks: Vec<EventTrack>,
    /// Drawn over everything when the export profile asks (INV-9).
    watermark: Option<watermark::Track>,
}

pub fn build_scene(input: SceneInput<'_>) -> Result<Scene, SceneError> {
    let crop = framing::crop_rect(input.clip, input.edit.crop_offset);
    let style = styles::spec(input.edit.style);
    let (theme, fonts, map) = (input.theme, input.fonts, input.time_map);

    // Where the captions sit, and the visuals on their free side (D-171).
    let block = placement::caption_block(input.edit.caption_position, &style);
    let event_zone = placement::event_zone(block);

    let words = captions::words(input.transcript, &input.edit.word_edits, map);
    let sentences = &input.transcript.sentences;
    let mut chunks = captions::chunk(&words, sentences, &style, theme, fonts, block)?;
    readability::hold_for_reading(&mut chunks, OutMs::new(map.out_duration().get()));

    let mut tracks = Vec::new();
    let mut emphasis = captions_draw::Emphasis {
        pops: Vec::new(),
        numbers: input.edit.highlight_numbers,
    };
    for item in input.plan.items() {
        // The user's choice for this item, when there is one; every V4 item
        // is shown by default (TS §10.8, D-145).
        let shown = input
            .edit
            .event_overrides
            .get(&item.id)
            .copied()
            .unwrap_or(true);
        if !shown {
            continue;
        }
        let span = map.out_span(item.span).ok_or(SceneError::TimeMap)?;
        let (when, t) = ((span, event_zone), input.transcript);
        let Some(track) = events::layout(item, when, t, map, &style, theme, fonts)? else {
            continue;
        };
        // A KeywordPop is drawn by the captions: its word, from its program
        // start (TS §19.5, D-169). A word the captions do not show cannot
        // be popped.
        if let EventTrack::KeywordPop(pop) = &track {
            if !words.iter().any(|shown| shown.word == pop.word) {
                continue;
            }
            emphasis.pops.push((pop.word, pop.start));
        }
        tracks.push(track);
    }

    let watermark = if input.profile.watermark {
        Some(watermark::layout(theme, fonts)?)
    } else {
        None
    };
    let (width, height) = (input.profile.width.get(), input.profile.height.get());
    let scale = width as f32 / safe_area::CANVAS_WIDTH;
    Ok(Scene {
        time_map: map.clone(),
        video: video::Track::fixed(crop),
        canvas: (width as f32, height as f32),
        scale,
        style,
        theme: theme.clone(),
        fonts: fonts.clone(),
        caption_block: block,
        chunks,
        emphasis,
        tracks,
        watermark,
    })
}

impl Scene {
    /// The length of the program: the time map's.
    pub fn duration(&self) -> DurMs {
        self.time_map.out_duration()
    }

    /// `ceil(duration_ms * OUTPUT_FPS / 1000)`: 2,700 for 90 s.
    pub fn frame_count(&self) -> u32 {
        let frames = (u64::from(self.duration().get()) * u64::from(OUTPUT_FPS)).div_ceil(1_000);
        u32::try_from(frames).unwrap_or(u32::MAX)
    }

    /// What is drawn at `t`, in pixels of the output canvas: the video, the
    /// caption chunk that is shown, every item that is shown, and the
    /// watermark when the profile asks.
    pub fn frame_at(&self, t: OutMs) -> DisplayList {
        let mut cmds = vec![self.video.cmd(t, self.canvas)];
        if let Some(chunk) = self.chunks.iter().find(|chunk| chunk.visible_at(t)) {
            let (style, theme) = (&self.style, &self.theme);
            captions_draw::draw_with(chunk, t, style, theme, &self.emphasis, &mut cmds);
        }
        for track in &self.tracks {
            events::draw(track, t, &mut cmds);
        }
        if let Some(mark) = &self.watermark {
            watermark::draw(mark, &mut cmds);
        }
        // The video is placed in output pixels already; the rest is scaled.
        for cmd in &mut cmds {
            cmd.scale(self.scale);
        }
        DisplayList { cmds }
    }

    /// What the point `(x, y)` of the canvas, in logical pixels, is on at
    /// `t` (D-174): a word of the chunk shown then, the caption block, or
    /// the video. `None` outside the canvas.
    pub fn hit(&self, t: OutMs, x: f32, y: f32) -> Option<Hit> {
        let shown = self.chunks.iter().find(|chunk| chunk.visible_at(t));
        hit::hit(shown, self.caption_block, x, y)
    }

    /// The glyph box of `word` in the chunk shown at `t`, in logical pixels:
    /// the box `hit` finds it by, which the page outlines (D-183). `None`
    /// when no chunk shown then holds it.
    pub fn word_box(&self, t: OutMs, word: WordIdx) -> Option<Rect> {
        let shown = self.chunks.iter().find(|chunk| chunk.visible_at(t))?;
        let mut boxes = hit::word_boxes(shown).into_iter();
        boxes.find(|(at, _)| *at == word).map(|(_, r)| r)
    }

    /// The chunks as the transcript editor shows them (D-182, "Gaps settled"
    /// item 6), in time order: the transcript words from the first of each
    /// chunk to after the first word of its last caption word (a number's
    /// other words follow it), and the program time it is shown from. A read
    /// of what `build_scene` computed: the page never chunks a second time.
    pub fn chunk_ranges(&self) -> Vec<(WordRange, OutMs)> {
        let range = |chunk: &Chunk| {
            let first = chunk.words.first()?.word;
            let last = chunk.words.last()?.word;
            Some((
                WordRange {
                    start: first,
                    end: WordIdx::new(last.get() + 1),
                },
                chunk.show.start,
            ))
        };
        self.chunks.iter().filter_map(range).collect()
    }

    /// Each word of the chunk shown at `t` as it is drawn, with its fill:
    /// what development builds of the page mirror for the tests (D-155).
    pub fn caption_words(&self, t: OutMs) -> Vec<(String, Rgba)> {
        let Some(chunk) = self.chunks.iter().find(|chunk| chunk.visible_at(t)) else {
            return Vec::new();
        };
        let drawn = captions_draw::lay_out(chunk, t, &self.style, &self.theme, &self.emphasis);
        let text = |word: &captions_draw::Drawn<'_>| {
            word.runs.iter().map(|(run, _)| run.text.as_str()).collect()
        };
        drawn.iter().map(|word| (text(word), word.fill)).collect()
    }

    /// Where the main video's source is at `t`.
    pub fn source_at(&self, t: OutMs) -> SrcPos {
        self.time_map.to_src(t)
    }

    /// The box the caption lines are centred in, for the clip's placement,
    /// in logical pixels (D-171).
    pub fn caption_block(&self) -> Rect {
        self.caption_block
    }

    /// The faces the glyph runs of every frame are drawn with (D-132).
    pub fn fonts(&self) -> &FontSet {
        &self.fonts
    }

    /// What was changed (D-144): the items drawn, by kind.
    pub fn summary(&self) -> ChangeSummary {
        summary::change_summary(&self.tracks)
    }
}

#[cfg(test)]
pub(crate) mod testing {
    use std::collections::BTreeMap;

    use offcut_plan::{PlanConfig, from_rules, validate};
    use offcut_text::{RawWord, normalize_transcript};
    use offcut_types::{
        BitsPerSec, Confidence, DetectedEvent, EventId, EventKind, EventParams, Orientation,
        ProfileKind, Px, Quantity, Rotation, Span, StyleId, TimeMs, WordIdx, WordRange,
    };

    use super::*;
    use crate::layout::shape;

    pub(crate) fn word(text: &str, start_ms: u32, end_ms: u32) -> RawWord {
        RawWord {
            text: text.to_owned(),
            start_ms,
            end_ms,
            confidence: 1.0,
        }
    }

    /// A transcript as the pipeline builds it, sentences and numbers included.
    pub(crate) fn transcript(words: &[RawWord]) -> Transcript {
        normalize_transcript(words.to_vec(), "test")
    }

    pub(crate) fn clip(width: u32, height: u32, duration_ms: u32) -> ClipInfo {
        ClipInfo {
            duration: DurMs::new(duration_ms),
            display_width: Px::new(width),
            display_height: Px::new(height),
            rotation: Rotation::R0,
            orientation: if width > height {
                Orientation::Landscape
            } else {
                Orientation::Portrait
            },
            is_vfr: false,
            video_codec_string: "avc1.640028".to_owned(),
            audio_codec_string: "mp4a.40.2".to_owned(),
            audio_sample_rate: offcut_types::Hz::new(48_000),
            audio_channels: 2,
        }
    }

    pub(crate) fn profile(width: u32) -> ExportProfile {
        ExportProfile {
            kind: ProfileKind::Creator,
            width: Px::new(width),
            height: Px::new(width * 16 / 9),
            watermark: false,
            video_bitrate: BitsPerSec::new(8_000_000),
            audio_bitrate: BitsPerSec::new(160_000),
        }
    }

    /// A NumberReveal on the word `word`. `validate` takes its span from
    /// that word's times (D-121); `start` and `end` are V3's own span.
    pub(crate) fn number_event(value: Quantity, word: u32, start: u32, end: u32) -> DetectedEvent {
        DetectedEvent {
            id: EventId::new(u64::from(start)),
            kind: EventKind::NumberReveal,
            span: Span::new(TimeMs::new(start), TimeMs::new(end)).unwrap(),
            anchors: WordRange {
                start: WordIdx::new(word),
                end: WordIdx::new(word + 1),
            },
            params: EventParams::NumberReveal { value, label: None },
            confidence: Confidence::new(1.0).unwrap(),
            enabled: true,
        }
    }

    /// The rules plan of `events`, validated: what the pipeline draws.
    /// Every event is kept.
    pub(crate) fn plan(transcript: &Transcript, events: &[DetectedEvent]) -> ValidPlan {
        let rules = from_rules(events, transcript);
        let edits = BTreeMap::new();
        let (plan, dropped) = validate(rules, transcript, &edits, &PlanConfig::default()).unwrap();
        assert_eq!(dropped, [], "the test's events are all drawable");
        plan
    }

    /// A scene of a 10 s portrait clip, with no edits.
    pub(crate) fn build(transcript: &Transcript, events: &[DetectedEvent], width: u32) -> Scene {
        build_scene(SceneInput {
            transcript,
            plan: &plan(transcript, events),
            edit: &EditState::default(),
            clip: &clip(1080, 1920, 10_000),
            profile: &profile(width),
            time_map: &TimeMap::identity(DurMs::new(10_000)),
            theme: &Theme::default_for(StyleId::Clean),
            fonts: &FontSet::embedded(),
        })
        .unwrap()
    }

    /// The box a command draws in, on the canvas. For a glyph run it is the
    /// box of its line (ascent to descent, start to end) and its stroke. The
    /// video is under everything, on the whole canvas: no box.
    pub(crate) fn bounds(cmd: &DrawCmd) -> Option<Rect> {
        let corners: Vec<(f32, f32)> = match cmd {
            DrawCmd::GlyphRun {
                font,
                size,
                stroke,
                transform,
                text,
                ..
            } => {
                let runs = shape(text, *font, *size, &FontSet::embedded()).unwrap();
                assert_eq!(runs.len(), 1, "{text:?}");
                let pad = stroke.map_or(0.0, |stroke| stroke.width / 2.0);
                let (left, right) = (-pad, runs[0].width + pad);
                let (top, bottom) = (-runs[0].ascent - pad, runs[0].descent + pad);
                [(left, top), (right, top), (left, bottom), (right, bottom)]
                    .map(|(x, y)| transform.apply(x, y))
                    .to_vec()
            }
            DrawCmd::FillRect { rect, .. } => {
                vec![(rect.x, rect.y), (rect.x + rect.w, rect.y + rect.h)]
            }
            DrawCmd::FillPath { .. } => panic!("V2 draws no path"),
            DrawCmd::Image { .. } => panic!("V4 draws no image"),
            DrawCmd::PushLayer { .. } | DrawCmd::PopLayer | DrawCmd::Video { .. } => return None,
        };
        let fold = |pick: fn(&(f32, f32)) -> f32, by: fn(f32, f32) -> f32, from: f32| {
            corners.iter().map(pick).fold(from, by)
        };
        let (x, y) = (
            fold(|c| c.0, f32::min, f32::MAX),
            fold(|c| c.1, f32::min, f32::MAX),
        );
        let (right, bottom) = (
            fold(|c| c.0, f32::max, f32::MIN),
            fold(|c| c.1, f32::max, f32::MIN),
        );
        Some(Rect {
            x,
            y,
            w: right - x,
            h: bottom - y,
        })
    }
}

#[cfg(test)]
mod tests {
    use offcut_types::{
        Confidence, DetectedEvent, EventId, EventKind, EventParams, FrameIdx, Quantity, Span,
        StyleId, TimeMs, Unit, WordIdx, WordRange,
    };

    use super::testing::*;
    use super::*;
    use crate::safe_area::SAFE_AREA;
    use crate::styles::spec;

    /// Captions with a long and a wide word, a pause, a number that counts
    /// up through texts wider than its end, and one that scales in.
    fn busy() -> (Transcript, Vec<DetectedEvent>) {
        let texts = [
            "we",
            "spent",
            "three",
            "long",
            "months",
            "building",
            "something",
            "WWWWWWWWWWWWWWWWWW",
            "internationalization",
            "nobody",
            "wanted.",
        ];
        let mut words: Vec<_> = (0u32..)
            .zip(texts)
            .map(|(i, text)| word(text, 500 + i * 300, 750 + i * 300))
            .collect();
        words.push(word("It", 6_000, 6_200));
        words.push(word("made", 6_250, 6_500));
        words.push(word("$250,000", 6_550, 7_000));
        words.push(word("here.", 7_050, 7_300));
        let quantity = |value: f64, unit: Unit, display: &str| Quantity {
            value,
            unit,
            display: display.to_owned(),
        };
        // Each on a word of its own: `validate` takes an item's span from its
        // words ("three" 1,100-1,350, "wanted." 3,500-3,750, "$250,000"
        // 6,550-7,000), and two items on one word would be too close (V-6).
        let events = vec![
            number_event(quantity(40.0, Unit::Percent, "40%"), 2, 1_000, 1_400),
            number_event(quantity(3.0, Unit::None, "3"), 10, 3_500, 3_800),
            number_event(quantity(250_000.0, Unit::Usd, "$250k"), 13, 6_550, 7_000),
        ];
        (transcript(&words), events)
    }

    fn frames(scene: &Scene) -> Vec<DisplayList> {
        // A frame index is a program time (D-125).
        (0..scene.frame_count())
            .map(|frame| scene.frame_at(FrameIdx::new(frame).to_out_ms()))
            .collect()
    }

    #[test]
    fn a_60_000_ms_clip_has_1_800_frames_and_a_20_033_ms_clip_601() {
        let empty = transcript(&[]);
        let no_items = plan(&empty, &[]);
        let (theme, fonts) = (Theme::default_for(StyleId::Clean), FontSet::embedded());
        let count = |duration_ms: u32| {
            let scene = build_scene(SceneInput {
                transcript: &empty,
                plan: &no_items,
                edit: &EditState::default(),
                clip: &clip(1080, 1920, duration_ms),
                profile: &profile(1080),
                time_map: &TimeMap::identity(DurMs::new(duration_ms)),
                theme: &theme,
                fonts: &fonts,
            })
            .unwrap();
            assert_eq!(scene.duration(), DurMs::new(duration_ms));
            scene.frame_count()
        };
        assert_eq!(count(60_000), 1_800);
        assert_eq!(count(20_033), 601);
        assert_eq!(count(90_000), 2_700);
        assert_eq!(count(74_705), 2_242);
        assert_eq!(count(0), 0);
        assert_eq!(count(1), 1);
    }

    #[test]
    fn every_command_of_every_frame_is_inside_the_safe_area() {
        let (transcript, events) = busy();
        let scene = build(&transcript, &events, 1080);
        assert_eq!(scene.summary().visual_moments, 3);
        let (right, bottom) = (SAFE_AREA.x + SAFE_AREA.w, SAFE_AREA.y + SAFE_AREA.h);
        let mut drawn = 0;
        for (frame, list) in frames(&scene).iter().enumerate() {
            for cmd in &list.cmds {
                let Some(b) = bounds(cmd) else { continue };
                drawn += 1;
                let inside = b.x >= SAFE_AREA.x - 0.01
                    && b.y >= SAFE_AREA.y - 0.01
                    && b.x + b.w <= right + 0.01
                    && b.y + b.h <= bottom + 0.01;
                assert!(inside, "frame {frame}: {b:?} of {cmd:?}");
            }
        }
        assert!(drawn > 500, "{drawn}");
    }

    #[test]
    fn two_builds_of_one_input_draw_the_same_frames() {
        let (transcript, events) = busy();
        let (first, second) = (
            build(&transcript, &events, 1080),
            build(&transcript, &events, 1080),
        );
        assert_eq!(first, second);
        assert_eq!(frames(&first), frames(&second));
        // A frame is a function of the scene and the time: asked again, and
        // asked out of order, it is the same.
        let t = OutMs::new(6_700);
        let before = first.frame_at(t);
        first.frame_at(OutMs::new(100));
        assert_eq!(first.frame_at(t), before);
        assert!(!before.cmds.is_empty());
    }

    #[test]
    fn at_a_profile_width_of_540_every_coordinate_is_half() {
        let (transcript, events) = busy();
        let full = frames(&build(&transcript, &events, 1080));
        let half = frames(&build(&transcript, &events, 540));
        assert_eq!(full.len(), half.len());
        let mut runs = 0;
        for (full, half) in full.iter().zip(&half) {
            assert_eq!(full.cmds.len(), half.cmds.len());
            for pair in full.cmds.iter().zip(&half.cmds) {
                match pair {
                    (
                        DrawCmd::GlyphRun {
                            font,
                            size,
                            glyphs,
                            fill,
                            stroke,
                            transform,
                            text,
                        },
                        DrawCmd::GlyphRun {
                            font: font_h,
                            size: size_h,
                            glyphs: glyphs_h,
                            fill: fill_h,
                            stroke: stroke_h,
                            transform: transform_h,
                            text: text_h,
                        },
                    ) => {
                        runs += 1;
                        assert_eq!((font, fill, text), (font_h, fill_h, text_h));
                        assert_eq!(*size_h, size / 2.0);
                        assert_eq!(stroke_h.map(|s| s.width), stroke.map(|s| s.width / 2.0));
                        assert_eq!(glyphs.len(), glyphs_h.len());
                        for (glyph, glyph_h) in glyphs.iter().zip(glyphs_h) {
                            assert_eq!(glyph_h.id, glyph.id);
                            // Where the glyph lands on the canvas.
                            let at = transform.apply(glyph.x, glyph.y);
                            let at_h = transform_h.apply(glyph_h.x, glyph_h.y);
                            assert!((at_h.0 - at.0 / 2.0).abs() < 0.001, "{at:?} {at_h:?}");
                            assert!((at_h.1 - at.1 / 2.0).abs() < 0.001, "{at:?} {at_h:?}");
                        }
                    }
                    (
                        DrawCmd::PushLayer { opacity, clip },
                        DrawCmd::PushLayer {
                            opacity: opacity_h,
                            clip: clip_h,
                        },
                    ) => {
                        assert_eq!(opacity, opacity_h);
                        // The sliding pill's clip (D-170): half the box.
                        let half = clip.map(|r| [r.x / 2.0, r.y / 2.0, r.w / 2.0, r.h / 2.0]);
                        let got = clip_h.map(|r| [r.x, r.y, r.w, r.h]);
                        assert_eq!(half.is_some(), got.is_some());
                        for (a, b) in half.iter().flatten().zip(got.iter().flatten()) {
                            assert!((a - b).abs() < 0.001, "{clip:?} {clip_h:?}");
                        }
                    }
                    (DrawCmd::PopLayer, DrawCmd::PopLayer) => {}
                    // Clean's pill (D-170): half the box and the radius.
                    (
                        DrawCmd::FillRect {
                            rect,
                            radius,
                            color,
                        },
                        DrawCmd::FillRect {
                            rect: rect_h,
                            radius: radius_h,
                            color: color_h,
                        },
                    ) => {
                        assert_eq!(color, color_h);
                        let (full, half) = (
                            [rect.x, rect.y, rect.w, rect.h, *radius],
                            [rect_h.x, rect_h.y, rect_h.w, rect_h.h, *radius_h],
                        );
                        for (a, b) in full.iter().zip(half) {
                            assert!((b - a / 2.0).abs() < 0.001, "{full:?} {half:?}");
                        }
                    }
                    // The video is placed on the output canvas: the whole of
                    // it at either width, from the same part of the source.
                    (
                        DrawCmd::Video {
                            slot,
                            src,
                            dst,
                            opacity,
                            mask,
                        },
                        DrawCmd::Video {
                            slot: slot_h,
                            src: src_h,
                            dst: dst_h,
                            opacity: opacity_h,
                            mask: mask_h,
                        },
                    ) => {
                        assert_eq!(
                            (slot, src, opacity, mask),
                            (slot_h, src_h, opacity_h, mask_h)
                        );
                        assert_eq!((dst.w, dst.h), (1080.0, 1920.0));
                        assert_eq!((dst_h.w, dst_h.h), (540.0, 960.0));
                    }
                    other => panic!("{other:?}"),
                }
            }
        }
        assert!(runs > 500, "{runs}");
    }

    #[test]
    fn a_number_is_in_accent_when_highlighted_and_a_keyword_pop_from_its_start() {
        let spoken = [
            word("It", 1_000, 1_200),
            word("made", 1_250, 1_500),
            word("$5", 1_550, 1_800),
            word("now.", 1_850, 2_200),
        ];
        let t = transcript(&spoken);
        let pop = DetectedEvent {
            id: EventId::new(1),
            kind: EventKind::KeywordPop,
            span: Span::new(TimeMs::new(1_250), TimeMs::new(1_500)).unwrap(),
            anchors: WordRange {
                start: WordIdx::new(1),
                end: WordIdx::new(2),
            },
            params: EventParams::KeywordPop {
                word: WordIdx::new(1),
            },
            confidence: Confidence::new(1.0).unwrap(),
            enabled: true,
        };
        let plan = plan(&t, &[pop]);
        let theme = Theme::default_for(StyleId::Clean);
        let scene = |highlight_numbers| {
            build_scene(SceneInput {
                transcript: &t,
                plan: &plan,
                edit: &EditState {
                    highlight_numbers,
                    ..EditState::default()
                },
                clip: &clip(1080, 1920, 10_000),
                profile: &profile(1080),
                time_map: &TimeMap::identity(DurMs::new(10_000)),
                theme: &theme,
                fonts: &FontSet::embedded(),
            })
            .unwrap()
        };
        let fill = |scene: &Scene, at: u32, text: &str| {
            let list = scene.frame_at(OutMs::new(at));
            let fills: Vec<Rgba> = list
                .cmds
                .iter()
                .filter_map(|cmd| match cmd {
                    // The run over its shadow (D-170).
                    DrawCmd::GlyphRun {
                        fill, text: run, ..
                    } if run == text && *fill != theme.shadow => Some(*fill),
                    DrawCmd::GlyphRun { .. }
                    | DrawCmd::FillRect { .. }
                    | DrawCmd::FillPath { .. }
                    | DrawCmd::PushLayer { .. }
                    | DrawCmd::PopLayer
                    | DrawCmd::Video { .. }
                    | DrawCmd::Image { .. } => None,
                })
                .collect();
            assert_eq!(fills.len(), 1, "{text:?} at {at}");
            fills[0]
        };
        let (on, off) = (scene(true), scene(false));
        // One chunk: "It made $5 now." fits one line.
        // "It" is spoken: "made" is not popped yet; "$5" is a number.
        assert_eq!(fill(&on, 1_100, "made"), theme.text);
        assert_eq!(fill(&on, 1_100, "$5"), theme.accent);
        assert_eq!(fill(&off, 1_100, "$5"), theme.text);
        // "now." is spoken: "made" has popped and stays in `pop`.
        let pop = theme.color(spec(StyleId::Clean).pop);
        assert_eq!(fill(&on, 2_000, "made"), pop);
        assert_eq!(fill(&off, 2_000, "made"), pop);
        // The spoken word is in ink on Clean's pill (D-170).
        assert_eq!(fill(&on, 2_000, "now."), theme.on_accent);
        assert_eq!(fill(&on, 2_000, "It"), theme.text);
        assert_eq!(fill(&off, 2_000, "$5"), theme.text);
    }

    /// The busy scene with `edit` and the time map `map`.
    fn busy_with(edit: &EditState, map: &TimeMap) -> Result<Scene, SceneError> {
        let (transcript, events) = busy();
        build_scene(SceneInput {
            transcript: &transcript,
            plan: &plan(&transcript, &events),
            edit,
            clip: &clip(1080, 1920, 10_000),
            profile: &profile(1080),
            time_map: map,
            theme: &Theme::default_for(StyleId::Clean),
            fonts: &FontSet::embedded(),
        })
    }

    #[test]
    fn the_summary_counts_the_items_drawn_and_not_a_disabled_one() {
        let identity = TimeMap::identity(DurMs::new(10_000));
        let counts = |edit: &EditState| {
            let s = busy_with(edit, &identity).unwrap().summary();
            (s.voice_cleaned, s.captions_emphasized, s.visual_moments)
        };
        assert_eq!(counts(&EditState::default()), (true, 0, 3));
        let (transcript, events) = busy();
        let first = plan(&transcript, &events).items()[0].id;
        let overrides = |pairs: &[(EventId, bool)]| EditState {
            event_overrides: pairs.iter().copied().collect(),
            ..EditState::default()
        };
        assert_eq!(counts(&overrides(&[(first, false)])), (true, 0, 2));
        // Turned back on it is drawn; an override of an id the plan no
        // longer has is ignored (D-145).
        let gone = EventId::new(999_999);
        assert_eq!(
            counts(&overrides(&[(first, true), (gone, false)])),
            (true, 0, 3)
        );
    }

    #[test]
    fn an_item_the_program_does_not_play_is_an_error() {
        // A map made by hand (D-164: in tests only) that leaves out 1,000 to
        // 1,500 ms of the recording, where the first number is said.
        let cut: TimeMap = serde_json::from_str(
            r#"{"segments":[{"kind":"source","src":{"start":0,"end":1000}},{"kind":"source","src":{"start":1500,"end":10000}}]}"#,
        )
        .unwrap();
        let result = busy_with(&EditState::default(), &cut);
        assert_eq!(result.err(), Some(SceneError::TimeMap));
    }

    #[test]
    fn the_chunk_ranges_are_the_chunks_in_order_and_a_hidden_word_starts_none() {
        let identity = TimeMap::identity(DurMs::new(10_000));
        let ranges = busy_with(&EditState::default(), &identity)
            .unwrap()
            .chunk_ranges();
        assert!(ranges.len() >= 3, "{ranges:?}");
        assert_eq!(ranges[0].0.start, WordIdx::new(0));
        for pair in ranges.windows(2) {
            let ((a, at_a), (b, at_b)) = (pair[0], pair[1]);
            // In time order, one after another, never overlapping.
            assert!(
                a.start < a.end && a.end <= b.start && at_a < at_b,
                "{pair:?}"
            );
        }
        // Each chunk is shown from its time, starting with its first word (a
        // number shows its display, "$250k" for "$250,000").
        let scene = busy_with(&EditState::default(), &identity).unwrap();
        let (transcript, _) = busy();
        for (range, at) in &ranges {
            let shown = scene.caption_words(*at);
            let first = &transcript.words[range.start.get() as usize].text;
            let same = shown[0].0.eq_ignore_ascii_case(first) || first.starts_with('$');
            assert!(same, "{shown:?} {first}");
        }
        // Hidden, the first word starts no chunk: the next one does.
        let hidden = EditState {
            word_edits: [(WordIdx::new(0), String::new())].into_iter().collect(),
            ..EditState::default()
        };
        let ranges = busy_with(&hidden, &identity).unwrap().chunk_ranges();
        assert_eq!(ranges[0].0.start, WordIdx::new(1));
    }
}
