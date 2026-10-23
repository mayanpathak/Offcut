//! Exact display lists (D-157, §8.10, TS §19.7's acceptance), at 1080 wide:
//! each style with each kind's visual, before it, while all of it shows, and
//! after it; the watermark on and off; a landscape crop at -1, 0 and 1; a
//! word edit that changes a number; a disabled item. Every snapshot is read
//! before it is accepted (G item 5).
//!
//! Each kind's input is a small written transcript, detected and validated
//! as the pipeline does: the inputs Prompt 107's goldens draw.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

#[path = "common/scenes.rs"]
mod scenes;

use std::collections::BTreeMap;

use offcut_scene::{DisplayList, DrawCmd, FontSet, SceneInput, Theme, build_scene};
use offcut_types::{
    BitsPerSec, ClipInfo, CropOffset, DurMs, EventKind, ExportProfile, Hz, Orientation, OutMs,
    ProfileKind, Px, Rotation, StyleId, TimeMap, WordIdx,
};

/// Each kind's sentences, and its three instants (ms): before the visual,
/// while all of it shows, and after its fade.
struct Input {
    kind: EventKind,
    name: &'static str,
    sentences: &'static [&'static str],
    at: [u32; 3],
}

/// The sentence after each: room for the "after" instant, and no item.
const TAIL: &str = "Thanks for watching, and see you next week.";

const INPUTS: [Input; 4] = [
    // "3,000 users" (label "last month") 1,305-2,030 ms: counted up by
    // 1,805, held to 3,230, faded by 3,430.
    Input {
        kind: EventKind::NumberReveal,
        name: "number_reveal",
        sentences: &["We crossed 3,000 users last month.", TAIL],
        at: [1_200, 1_905, 3_530],
    },
    // "3 REASONS" from 1,215 ms, its items at 2,965, 4,670 and 6,465, held
    // to 9,360 and faded by 9,560.
    Input {
        kind: EventKind::ListReveal,
        name: "list_reveal",
        sentences: &[
            "Here are 3 reasons to ship.",
            "First, it is fast.",
            "Second, it is cheap.",
            "Third, it is simple.",
            TAIL,
        ],
        at: [1_100, 6_800, 9_660],
    },
    // "90 minutes" from 2,020 ms, "10 minutes" at its mark, 3,115; held to
    // 5,340, faded by 5,540.
    Input {
        kind: EventKind::FromTo,
        name: "from_to",
        sentences: &["Our build went from 90 minutes to 10 minutes.", TAIL],
        at: [1_900, 3_400, 5_640],
    },
    // "brutal" 2,020-2,410 ms: at its pop's peak at 2,110, then in its pop
    // colour at its own size.
    Input {
        kind: EventKind::KeywordPop,
        name: "keyword_pop",
        sentences: &["The first week felt *brutal* for everyone.", TAIL],
        at: [1_900, 2_110, 2_600],
    },
];

const STYLES: [StyleId; 3] = [StyleId::Clean, StyleId::Bold, StyleId::Tech];

fn style_name(style: StyleId) -> String {
    format!("{style:?}").to_lowercase()
}

/// The case of `input` in `style`, with `edits`; it holds one item, of the
/// input's kind.
fn case(input: &Input, style: StyleId, edits: BTreeMap<WordIdx, String>) -> scenes::Case {
    let case = scenes::written(input.name, input.sentences, edits, |edit, _| {
        edit.style = style;
    });
    let kinds: Vec<EventKind> = case.plan.items().iter().map(|item| item.kind).collect();
    assert_eq!(kinds, [input.kind], "{}", case.name);
    case
}

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
