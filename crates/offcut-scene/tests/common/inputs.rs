//! Each kind's input (V4 Prompt 106): a small written transcript, detected
//! and validated as the pipeline does, holding one item of its kind, and the
//! three instants `display_list_snapshots.rs` draws it at. `offcut-render`'s
//! `golden_frames.rs` draws the same scenes at the middle instant, so the
//! snapshots and the goldens show one look. Needs `scenes.rs` as the module
//! `scenes`.

use std::collections::BTreeMap;

use offcut_types::{EventKind, StyleId, WordIdx};

use crate::scenes::{Case, written};

/// Each kind's sentences, and its three instants (ms): before the visual,
/// while all of it shows, and after its fade.
pub struct Input {
    pub kind: EventKind,
    pub name: &'static str,
    pub sentences: &'static [&'static str],
    pub at: [u32; 3],
}

/// The sentence after each: room for the "after" instant, and no item.
pub const TAIL: &str = "Thanks for watching, and see you next week.";

pub const INPUTS: [Input; 4] = [
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

pub const STYLES: [StyleId; 3] = [StyleId::Clean, StyleId::Bold, StyleId::Tech];

pub fn style_name(style: StyleId) -> String {
    format!("{style:?}").to_lowercase()
}

/// The case of `input` in `style`, with `edits`; it holds one item, of the
/// input's kind.
pub fn case(input: &Input, style: StyleId, edits: BTreeMap<WordIdx, String>) -> Case {
    let case = written(input.name, input.sentences, edits, |edit, _| {
        edit.style = style;
    });
    let kinds: Vec<EventKind> = case.plan.items().iter().map(|item| item.kind).collect();
    assert_eq!(kinds, [input.kind], "{}", case.name);
    case
}
