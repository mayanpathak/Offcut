//! Exact display lists (D-157, §8.10, TS §19.7's acceptance), at 1080 wide:
//! each style with each kind's visual, before it, while all of it shows, and
//! after it; the watermark on and off; a landscape crop at -1, 0 and 1; a
//! word edit that changes a number; a disabled item. Every snapshot is read
//! before it is accepted (G item 5).
//!
//! Each kind's input is a small written transcript, detected and validated
//! as the pipeline does (`common/inputs.rs`): the inputs `offcut-render`'s
//! goldens draw.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

#[path = "common/inputs.rs"]
mod inputs;
#[path = "common/scenes.rs"]
mod scenes;
// V6.2 (P202): a model's plan for the emphasis snapshots.
#[path = "common/model.rs"]
mod model;
// V6.2 (P205): a model's plan with the hook.
#[path = "common/hook.rs"]
mod hook;

use std::collections::BTreeMap;

use offcut_scene::{DisplayList, DrawCmd, FontSet, SceneInput, Theme, build_scene};
use offcut_types::{
    BitsPerSec, ClipInfo, CropOffset, DurMs, ExportProfile, Hz, Orientation, OutMs, ProfileKind,
    Px, Rotation, StyleId, TimeMap, WordIdx,
};

use inputs::{INPUTS, STYLES, case, style_name};

/// The texts of a list's glyph runs.
fn texts(list: &DisplayList) -> Vec<String> {
    list.cmds
        .iter()
        .filter_map(|cmd| {
            if let DrawCmd::GlyphRun { text, .. } = cmd {
                Some(text.clone())
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn each_style_with_each_kind_before_during_and_after_its_visual() {
    for style in STYLES {
        for input in &INPUTS {
            let scene = scenes::build(&case(input, style, BTreeMap::new()), 1080);
            for (when, t) in ["before", "during", "after"].into_iter().zip(input.at) {
                let name = format!("{}_{}_{when}", style_name(style), input.name);
                insta::assert_json_snapshot!(name, scene.frame_at(OutMs::new(t)));
            }
        }
    }
}

#[test]
fn the_watermark_on_and_off() {
    let case = case(&INPUTS[0], StyleId::Clean, BTreeMap::new());
    let at = OutMs::new(INPUTS[0].at[1]);
    let (on, off) = (
        scenes::build_with(&case, 1080, true).frame_at(at),
        scenes::build_with(&case, 1080, false).frame_at(at),
    );
    // On, the same list with the watermark's layer last (INV-9).
    assert_eq!(on.cmds[..off.cmds.len()], off.cmds[..]);
    assert_eq!(on.cmds.last(), Some(&DrawCmd::PopLayer));
    insta::assert_json_snapshot!("watermark_on", on);
    insta::assert_json_snapshot!("watermark_off", off);
}

#[test]
fn a_landscape_clip_is_cropped_where_its_offset_says() {
    let case = case(&INPUTS[0], StyleId::Clean, BTreeMap::new());
    let landscape = ClipInfo {
        duration: DurMs::new(case.length),
        display_width: Px::new(1920),
        display_height: Px::new(1080),
        rotation: Rotation::R0,
        orientation: Orientation::Landscape,
        is_vfr: false,
        video_codec_string: "avc1.640028".to_owned(),
        audio_codec_string: "mp4a.40.2".to_owned(),
        audio_sample_rate: Hz::new(48_000),
        audio_channels: 2,
    };
    for (name, offset) in [
        ("crop_left", -1.0),
        ("crop_centre", 0.0),
        ("crop_right", 1.0),
    ] {
        let edit = offcut_types::EditState {
            crop_offset: CropOffset::new(offset).unwrap(),
            ..case.edit.clone()
        };
        let scene = build_scene(SceneInput {
            transcript: &case.transcript,
            plan: &case.plan,
            edit: &edit,
            clip: &landscape,
            profile: &ExportProfile {
                kind: ProfileKind::Creator,
                width: Px::new(1080),
                height: Px::new(1920),
                watermark: false,
                video_bitrate: BitsPerSec::new(8_000_000),
                audio_bitrate: BitsPerSec::new(160_000),
            },
            time_map: &TimeMap::identity(DurMs::new(case.length)),
            theme: &Theme::default_for(StyleId::Clean),
            fonts: &FontSet::embedded(),
        })
        .unwrap();
        // The video command: the strip of the source the offset names.
        insta::assert_json_snapshot!(name, scene.frame_at(OutMs::new(0)).cmds[0]);
    }
}

#[test]
fn a_word_edit_that_changes_a_number_shows_the_edited_value() {
    // "3,000" (word 2) edited to "5,000": the plan's value and the caption
    // both show it (INV-22).
    let edits = BTreeMap::from([(WordIdx::new(2), "5,000".to_owned())]);
    let case = case(&INPUTS[0], StyleId::Clean, edits);
    let list = scenes::build(&case, 1080).frame_at(OutMs::new(INPUTS[0].at[1]));
    let shown = texts(&list);
    assert!(shown.iter().any(|text| text == "5,000 users"), "{shown:?}");
    assert!(
        !shown.iter().any(|text| text.contains("3,000")),
        "{shown:?}"
    );
    insta::assert_json_snapshot!("word_edit_number", list);
}

#[test]
fn a_disabled_item_is_not_drawn() {
    let input = &INPUTS[0];
    let case = scenes::written(
        input.name,
        input.sentences,
        BTreeMap::new(),
        |edit, plan| {
            for item in plan.items() {
                edit.event_overrides.insert(item.id, false);
            }
        },
    );
    let scene = scenes::build(&case, 1080);
    assert_eq!(scene.summary().visual_moments, 0);
    // Only the captions are drawn: every run is at Clean's caption size
    // (the caption shows "3,000 users" as one word).
    let list = scene.frame_at(OutMs::new(input.at[1]));
    let caption_size = offcut_scene::styles::spec(StyleId::Clean).size;
    for cmd in &list.cmds {
        if let DrawCmd::GlyphRun { size, text, .. } = cmd {
            assert_eq!(*size, caption_size, "{text:?}");
        }
    }
    insta::assert_json_snapshot!("disabled_item", list);
}

#[test]
fn the_from_to_steps_to_its_second_value_at_its_mark() {
    // §8.7: "10 minutes" and the arrow come at the item's mark (the start of
    // "10", 3,115 ms), through the time map; "90 minutes" from the start.
    let input = &INPUTS[2];
    let case = case(input, StyleId::Clean, BTreeMap::new());
    let item = &case.plan.items()[0];
    let mark = item.marks[0].get();
    assert!(mark > item.span.start.get());
    let scene = scenes::build(&case, 1080);
    // The "to" value is the run drawn 1.15 times its size; the arrow the path.
    let parts = |t: u32| {
        let list = scene.frame_at(OutMs::new(t));
        let to = list.cmds.iter().any(|cmd| {
            matches!(cmd, DrawCmd::GlyphRun { transform, .. } if (transform.m[0] - 1.15).abs() < 1e-6)
        });
        let arrow = list
            .cmds
            .iter()
            .any(|cmd| matches!(cmd, DrawCmd::FillPath { .. }));
        let from = texts(&list).iter().any(|text| text == "90 minutes");
        (from, to, arrow)
    };
    // From its start, "90 minutes"; no "to" value and no arrow until the
    // mark.
    assert_eq!(parts(item.span.start.get()), (true, false, false));
    assert_eq!(parts(mark - 1), (true, false, false));
    assert_eq!(parts(mark), (true, true, true));
}

/// V6.2 (P201, D-480): the Glow style with each kind's visual, before it,
/// while it shows and after it. New files; the three styles' are unchanged.
#[test]
fn glow_with_each_kind_before_during_and_after_its_visual() {
    let style = StyleId::Glow;
    for input in &INPUTS {
        let scene = scenes::build(&case(input, style, BTreeMap::new()), 1080);
        for (when, t) in ["before", "during", "after"].into_iter().zip(input.at) {
            let name = format!("{}_{}_{when}", style_name(style), input.name);
            insta::assert_json_snapshot!(name, scene.frame_at(OutMs::new(t)));
        }
    }
}

/// V6.2 (P202, D-481): the KeywordPop input's sentences with a model's plan
/// of one `Emphasis` on "brutal" in each treatment, in the four styles,
/// while the word is said, and 90 ms after it in Clean and Bold, whose chunk
/// goes on after it. In Tech and Glow "brutal" ends its chunk, and with it
/// the mark (D-481). New files.
#[test]
fn emphasis_each_treatment_in_each_style_while_said_and_after() {
    use offcut_types::{
        EmphasisSize, EventId, EventKind, ItemParams, PlanItem, Treatment, WordRange,
    };
    let input = &INPUTS[3];
    let styles = [StyleId::Clean, StyleId::Bold, StyleId::Tech, StyleId::Glow];
    let treatments = [
        (Treatment::Colour, "colour"),
        (Treatment::Underline, "underline"),
        (Treatment::Highlight, "highlight"),
    ];
    for style in styles {
        for (treatment, treatment_name) in treatments {
            let case = case(input, style, BTreeMap::new());
            let words = &case.transcript.words;
            let brutal = words.iter().position(|w| w.text == "brutal").unwrap() as u32;
            let item = PlanItem {
                id: EventId::new(9_200),
                kind: EventKind::Emphasis,
                anchors: WordRange {
                    start: WordIdx::new(brutal),
                    end: WordIdx::new(brutal + 1),
                },
                params: ItemParams::Emphasis {
                    word: WordIdx::new(brutal),
                    treatment,
                    size: EmphasisSize::Normal,
                },
                confidence: None,
            };
            let case = model::with_model_items(case, vec![item]);
            assert_eq!(case.plan.items().len(), 1, "{}", case.name);
            let scene = scenes::build(&case, 1080);
            let after = case.transcript.words[brutal as usize].end_ms.get() + 90;
            let goes_on = matches!(style, StyleId::Clean | StyleId::Bold);
            let shown = |t: u32| {
                scene
                    .word_box(OutMs::new(t), WordIdx::new(brutal))
                    .is_some()
            };
            assert_eq!(shown(after), goes_on, "{style:?}");
            let mut instants = vec![("said", input.at[1])];
            if goes_on {
                instants.push(("after", after));
            }
            for (when, t) in instants {
                assert!(shown(t), "{style:?} {t}");
                let name = format!("emphasis_{treatment_name}_{}_{when}", style_name(style));
                insta::assert_json_snapshot!(name, scene.frame_at(OutMs::new(t)));
            }
        }
    }
}

/// V6.2 (P203, D-482): the KeywordPop input's sentences with a model's plan
/// of one punch word, "brutal" (colour), in the four styles: while it is
/// said, alone in capitals at 1.9 times the size, and when the next chunk
/// is shown. New files.
#[test]
fn punch_word_in_each_style_while_said_and_next() {
    use offcut_types::{
        EmphasisSize, EventId, EventKind, ItemParams, PlanItem, Treatment, WordRange,
    };
    let input = &INPUTS[3];
    for style in [StyleId::Clean, StyleId::Bold, StyleId::Tech, StyleId::Glow] {
        let case = case(input, style, BTreeMap::new());
        let words = &case.transcript.words;
        let brutal = words.iter().position(|w| w.text == "brutal").unwrap() as u32;
        let item = PlanItem {
            id: EventId::new(9_200),
            kind: EventKind::Emphasis,
            anchors: WordRange {
                start: WordIdx::new(brutal),
                end: WordIdx::new(brutal + 1),
            },
            params: ItemParams::Emphasis {
                word: WordIdx::new(brutal),
                treatment: Treatment::Colour,
                size: EmphasisSize::Punch,
            },
            confidence: None,
        };
        let case = model::with_model_items(case, vec![item]);
        assert_eq!(case.plan.items().len(), 1, "{}", case.name);
        let scene = scenes::build(&case, 1080);
        let said = scene.frame_at(OutMs::new(input.at[1]));
        assert_eq!(
            texts(&said).last().map(String::as_str),
            Some("BRUTAL"),
            "{style:?}"
        );
        let next = scene.frame_at(OutMs::new(input.at[2]));
        assert!(!texts(&next).iter().any(|t| t == "BRUTAL"), "{style:?}");
        insta::assert_json_snapshot!(format!("punch_word_{}_said", style_name(style)), said);
        insta::assert_json_snapshot!(format!("punch_word_{}_next", style_name(style)), next);
    }
}

/// V6.2 (P204, D-483): the KeywordPop input's sentences with a model's plan
/// of one emoji or one icon, in the four styles, 300 ms after its word
/// starts (at rest): fire under "brutal" at Middle and at Bottom (over the
/// lines: no room under them), fire beside "everyone.", a light bulb on a
/// disc for "brutal". New files.
#[test]
fn emoji_and_icon_in_each_style() {
    use offcut_types::{
        CaptionPlacement, EmojiId, EmojiPlace, EventId, EventKind, IconId, ItemParams, PlanItem,
        WordRange,
    };
    let input = &INPUTS[3];
    let item = |word: u32, kind: EventKind, params: ItemParams| PlanItem {
        id: EventId::new(9_300),
        kind,
        anchors: WordRange {
            start: WordIdx::new(word),
            end: WordIdx::new(word + 1),
        },
        params,
        confidence: None,
    };
    for style in [StyleId::Clean, StyleId::Bold, StyleId::Tech, StyleId::Glow] {
        let base = case(input, style, BTreeMap::new());
        let at = |text: &str| {
            base.transcript
                .words
                .iter()
                .position(|w| w.text == text)
                .unwrap() as u32
        };
        let (brutal, everyone) = (at("brutal"), at("everyone."));
        let emoji = |word: u32, place| {
            let params = ItemParams::Emoji {
                word: WordIdx::new(word),
                emoji: EmojiId::Fire,
                place,
            };
            item(word, EventKind::Emoji, params)
        };
        let icon = item(
            brutal,
            EventKind::IconPop,
            ItemParams::IconPop {
                word: WordIdx::new(brutal),
                icon: IconId::Lightbulb,
            },
        );
        let shots = [
            (
                "emoji_under",
                0.55,
                emoji(brutal, EmojiPlace::Under),
                brutal,
            ),
            ("emoji_over", 1.0, emoji(brutal, EmojiPlace::Under), brutal),
            (
                "emoji_beside",
                0.55,
                emoji(everyone, EmojiPlace::Beside),
                everyone,
            ),
            ("icon_pop", 1.0, icon.clone(), brutal),
        ];
        for (name, place, plan, word) in shots {
            let mut case = case(input, style, BTreeMap::new());
            case.edit.caption_position = CaptionPlacement::new(place).unwrap();
            let case = model::with_model_items(case, vec![plan]);
            assert_eq!(case.plan.items().len(), 1, "{}", case.name);
            let scene = scenes::build(&case, 1080);
            let t = case.transcript.words[word as usize].start_ms.get() + 300;
            let list = scene.frame_at(OutMs::new(t));
            let drawn = list
                .cmds
                .iter()
                .filter(|cmd| matches!(cmd, DrawCmd::Image { .. }))
                .count();
            assert_eq!(drawn, 1, "{name} {style:?}");
            insta::assert_json_snapshot!(format!("{name}_{}", style_name(style)), list);
        }
    }
}

/// V6.2 (P205, D-484): the KeywordPop input's sentences with a model's plan
/// of the hook "The one thing nobody tells you" ("NOBODY" the accent), in
/// the four styles: entering (0 ms) and at rest (1,000 ms), no caption under
/// it. New files.
#[test]
fn hook_in_each_style_entering_and_at_rest() {
    let input = &INPUTS[3];
    for style in [StyleId::Clean, StyleId::Bold, StyleId::Tech, StyleId::Glow] {
        let case = case(input, style, BTreeMap::new());
        let case = hook::with_hook(case, "The one thing nobody tells you", 3, Vec::new());
        assert_eq!(case.plan.items().len(), 1, "{}", case.name);
        let scene = scenes::build(&case, 1080);
        for (when, t) in [("enter", 0), ("rest", 1_000)] {
            let list = scene.frame_at(OutMs::new(t));
            let words = ["THE", "ONE", "THING", "NOBODY", "TELLS", "YOU"];
            assert!(
                texts(&list).iter().all(|t| words.contains(&t.as_str())),
                "{style:?}"
            );
            insta::assert_json_snapshot!(format!("hook_{}_{when}", style_name(style)), list);
        }
    }
}
