//! The reference transcript with all five V6.2 kinds (P206): `a01-scriptA`,
//! the reference clip's labeled transcript, with a model's plan of a hook,
//! four emphases (one a punch word), two emoji, two icons and two zooms,
//! every item kept by the validation, and Motion on. Used by
//! `determinism.rs`, `parity.rs` and `display_list_snapshots.rs`. Needs
//! `labeled.rs`, `scenes.rs` and `hook.rs` as the modules `labeled`,
//! `scenes` and `hook`.

use offcut_types::{
    EmojiId, EmojiPlace, EmphasisSize, EventId, EventKind, IconId, ItemParams, PlanItem, StyleId,
    Treatment, WordIdx, WordRange, ZoomMotion, ZoomScale,
};

use crate::scenes::Case;

/// The hook, its accent word ("EVENINGS"), and the items' words.
pub const HOOK: (&str, u8) = ("The edit that gave me my evenings back", 6);
fn item(id: u64, (start, end): (u32, u32), kind: EventKind, params: ItemParams) -> PlanItem {
    PlanItem {
        id: EventId::new(id),
        kind,
        anchors: WordRange {
            start: WordIdx::new(start),
            end: WordIdx::new(end),
        },
        params,
        confidence: None,
    }
}

fn emphasis(id: u64, word: u32, treatment: Treatment, size: EmphasisSize) -> PlanItem {
    let params = ItemParams::Emphasis {
        word: WordIdx::new(word),
        treatment,
        size,
    };
    item(id, (word, word + 1), EventKind::Emphasis, params)
}

/// The plan's items besides the hook, on `a01-scriptA`'s words.
pub fn items() -> Vec<PlanItem> {
    let emoji = |id, word: u32, emoji, place| {
        let params = ItemParams::Emoji {
            word: WordIdx::new(word),
            emoji,
            place,
        };
        item(id, (word, word + 1), EventKind::Emoji, params)
    };
    let icon = |id, word: u32, icon| {
        let params = ItemParams::IconPop {
            word: WordIdx::new(word),
            icon,
        };
        item(id, (word, word + 1), EventKind::IconPop, params)
    };
    let zoom = |id, words, motion| {
        let params = ItemParams::PunchIn {
            scale: ZoomScale::Strong,
            motion,
        };
        item(id, words, EventKind::PunchIn, params)
    };
    vec![
        emoji(1, 13, EmojiId::Eyes, EmojiPlace::Under),
        emphasis(2, 25, Treatment::Underline, EmphasisSize::Normal),
        emphasis(3, 30, Treatment::Highlight, EmphasisSize::Normal),
        emoji(4, 31, EmojiId::Fire, EmojiPlace::Beside),
        zoom(5, (31, 32), ZoomMotion::Snap),
        zoom(6, (50, 60), ZoomMotion::Push),
        emphasis(7, 65, Treatment::Colour, EmphasisSize::Normal),
        emphasis(8, 72, Treatment::Colour, EmphasisSize::Punch),
        icon(9, 86, IconId::BarChart),
        icon(10, 94, IconId::Laptop),
    ]
}

/// The reference transcript in `style`, Motion on, with the hook and
/// `items()`; every item kept.
pub fn reference(style: StyleId) -> Case {
    let case = crate::labeled::case("a01-scriptA", |edit, _| {
        edit.style = style;
        edit.enhance.motion = true;
    });
    let more = items();
    let count = more.len() + 1;
    let case = crate::hook::with_hook(case, HOOK.0, HOOK.1, more);
    assert_eq!(
        case.plan.items().len(),
        count,
        "{}: every item kept",
        case.name
    );
    case
}
