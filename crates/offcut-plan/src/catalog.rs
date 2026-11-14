//! [ONLY] What the renderer can draw, as a plan may ask for it (TS §40.3,
//! v4implementation D-123). The prompt, the output schema and the validator's
//! limits are all read from these rows; a change to them is a new
//! `CATALOG_VERSION`. Every number here is an assumption until V6.2 (E-11).

use offcut_types::{EmojiId, EventKind, IconId};

/// The version of the catalog a plan is produced against (TS §40.4 V-1).
pub const CATALOG_VERSION: u32 = 2;

/// Who may write an item of a kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Producers {
    Rules,
    Model,
    Both,
}

/// How many words an item's anchors span.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnchorRule {
    /// From `min` to `max` words, both included.
    Words { min: u32, max: u32 },
    /// Exactly one word.
    Point,
    /// The whole clip; no V4 kind.
    ClipWide,
}

/// What a parameter holds, with its limits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParamTy {
    /// Text of at most `max_chars` characters.
    Label {
        max_chars: u32,
    },
    /// A pointer to the words that say a number.
    QuantityRef,
    WordIdx,
    /// At most `max` points, each with a label of at most `label_chars`.
    ListPoints {
        max: u32,
        label_chars: u32,
    },
    Enum(&'static [&'static str]),
    Bool,
    /// V6.2 (D-448): a whole number from 0 to `max`, both included (the hook's accent word).
    Index {
        max: u32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParamSpec {
    pub name: &'static str,
    pub ty: ParamTy,
    pub required: bool,
}

/// One kind a plan may hold. `zone` is the name of the screen zone it is
/// drawn in, as `offcut-scene`'s `ZoneId` names it (`"event"`, `"caption"`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemSpec {
    pub kind: EventKind,
    pub name: &'static str,
    /// The sentence the prompt gives the model.
    pub purpose: &'static str,
    pub anchor: AnchorRule,
    pub params: &'static [ParamSpec],
    pub producers: Producers,
    pub zone: &'static str,
    pub max_per_clip: u8,
    pub min_gap_ms: u32,
}

/// The zone of the event visuals and the one of the captions.
pub const EVENT_ZONE: &str = "event";
pub const CAPTION_ZONE: &str = "caption";
/// V6.2 (D-448): the video itself (a zoom), never taken by another kind, and
/// the whole frame for the hook title's first 2 s.
pub const VIDEO_ZONE: &str = "video";
pub const HOOK_ZONE: &str = "hook";

const LABEL: ParamTy = ParamTy::Label { max_chars: 32 };

/// The four kinds of V4 and the five of V6.2 (v6.2implementation §6.2,
/// D-448), in the order of their codes (D-47, D-123, D-446).
const CATALOG: [ItemSpec; 9] = [
    ItemSpec {
        kind: EventKind::NumberReveal,
        name: "number_reveal",
        purpose: "A number the speaker says that matters: show it large while it is said.",
        anchor: AnchorRule::Words { min: 1, max: 6 },
        params: &[
            ParamSpec {
                name: "value",
                ty: ParamTy::QuantityRef,
                required: true,
            },
            ParamSpec {
                name: "label",
                ty: LABEL,
                required: false,
            },
        ],
        producers: Producers::Both,
        zone: EVENT_ZONE,
        max_per_clip: 8,
        min_gap_ms: 1_500,
    },
    ItemSpec {
        kind: EventKind::ListReveal,
        name: "list_reveal",
        purpose: "A list the speaker counts out: a header, then each item as it is spoken.",
        anchor: AnchorRule::Words { min: 2, max: 400 },
        params: &[
            ParamSpec {
                name: "header",
                ty: ParamTy::Label { max_chars: 24 },
                required: false,
            },
            ParamSpec {
                name: "items",
                ty: ParamTy::ListPoints {
                    max: 7,
                    label_chars: 32,
                },
                required: true,
            },
        ],
        producers: Producers::Both,
        zone: EVENT_ZONE,
        max_per_clip: 3,
        min_gap_ms: 10_000,
    },
    ItemSpec {
        kind: EventKind::FromTo,
        name: "from_to",
        purpose: "A change from one number to another: show both with bars.",
        anchor: AnchorRule::Words { min: 3, max: 40 },
        params: &[
            ParamSpec {
                name: "from",
                ty: ParamTy::QuantityRef,
                required: true,
            },
            ParamSpec {
                name: "to",
                ty: ParamTy::QuantityRef,
                required: true,
            },
            ParamSpec {
                name: "label",
                ty: LABEL,
                required: false,
            },
        ],
        producers: Producers::Both,
        zone: EVENT_ZONE,
        max_per_clip: 4,
        min_gap_ms: 3_000,
    },
    ItemSpec {
        kind: EventKind::KeywordPop,
        name: "keyword_pop",
        purpose: "One word the speaker stresses: pop it in the caption.",
        anchor: AnchorRule::Point,
        params: &[ParamSpec {
            name: "word",
            ty: ParamTy::WordIdx,
            required: true,
        }],
        producers: Producers::Both,
        zone: CAPTION_ZONE,
        max_per_clip: 12,
        min_gap_ms: 800,
    },
    // V6.2 (§6.2): a model's kinds only. A punch word also keeps 6 s from
    // the last and 4 a clip (D-452); a zoom one a sentence (D-450).
    ItemSpec {
        kind: EventKind::Emphasis,
        name: "emphasis",
        purpose: "One word that carries the sentence: colour, underline or highlight it in the caption; `punch` shows it alone, large, for the line that lands hardest.",
        anchor: AnchorRule::Point,
        params: &[
            ParamSpec {
                name: "word",
                ty: ParamTy::WordIdx,
                required: true,
            },
            ParamSpec {
                name: "treatment",
                ty: ParamTy::Enum(&["colour", "underline", "highlight"]),
                required: true,
            },
            ParamSpec {
                name: "size",
                ty: ParamTy::Enum(&["normal", "punch"]),
                required: true,
            },
        ],
        producers: Producers::Model,
        zone: CAPTION_ZONE,
        max_per_clip: 12,
        min_gap_ms: 800,
    },
    ItemSpec {
        kind: EventKind::IconPop,
        name: "icon_pop",
        purpose: "A word that names a thing an icon shows plainly: pop the icon above the captions while it is said.",
        anchor: AnchorRule::Point,
        params: &[
            ParamSpec {
                name: "word",
                ty: ParamTy::WordIdx,
                required: true,
            },
            ParamSpec {
                name: "icon",
                ty: ParamTy::Enum(&IconId::NAMES),
                required: true,
            },
        ],
        producers: Producers::Model,
        zone: EVENT_ZONE,
        max_per_clip: 6,
        min_gap_ms: 2_500,
    },
    ItemSpec {
        kind: EventKind::PunchIn,
        name: "punch_in",
        purpose: "Zoom on the claim, the number or the punchline, never on filler: `snap` on one strong word, `push` across the few words that build to it.",
        anchor: AnchorRule::Words { min: 1, max: 40 },
        params: &[
            ParamSpec {
                name: "scale",
                ty: ParamTy::Enum(&["mild", "strong"]),
                required: true,
            },
            ParamSpec {
                name: "motion",
                ty: ParamTy::Enum(&["snap", "push"]),
                required: true,
            },
        ],
        producers: Producers::Model,
        zone: VIDEO_ZONE,
        max_per_clip: 8,
        min_gap_ms: 1_500,
    },
    ItemSpec {
        kind: EventKind::Emoji,
        name: "emoji",
        purpose: "A word an emoji says better: put it under the word, or beside it.",
        anchor: AnchorRule::Point,
        params: &[
            ParamSpec {
                name: "word",
                ty: ParamTy::WordIdx,
                required: true,
            },
            ParamSpec {
                name: "emoji",
                ty: ParamTy::Enum(&EmojiId::NAMES),
                required: true,
            },
            ParamSpec {
                name: "place",
                ty: ParamTy::Enum(&["beside", "under"]),
                required: true,
            },
        ],
        producers: Producers::Model,
        zone: CAPTION_ZONE,
        max_per_clip: 6,
        min_gap_ms: 2_500,
    },
    ItemSpec {
        kind: EventKind::HookTitle,
        name: "hook_title",
        purpose: "Always, with the hook: a title over the first two seconds, written in `extras.hook`; `accent` is the index of its strongest word.",
        anchor: AnchorRule::ClipWide,
        params: &[ParamSpec {
            name: "accent",
            ty: ParamTy::Index { max: 8 },
            required: true,
        }],
        producers: Producers::Model,
        zone: HOOK_ZONE,
        max_per_clip: 1,
        min_gap_ms: 0,
    },
];

/// Every kind a plan may hold, in code order.
pub fn catalog() -> &'static [ItemSpec] {
    &CATALOG
}

/// The row of `kind`, if this build draws it.
pub fn spec(kind: EventKind) -> Option<&'static ItemSpec> {
    CATALOG.iter().find(|row| row.kind == kind)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn the_rows_are_in_code_order_once_each() {
        let kinds: Vec<EventKind> = catalog().iter().map(|row| row.kind).collect();
        assert_eq!(
            kinds,
            [
                EventKind::NumberReveal,
                EventKind::ListReveal,
                EventKind::FromTo,
                EventKind::KeywordPop,
                // V6.2 (D-440 (1)): the five rows follow V4's four.
                EventKind::Emphasis,
                EventKind::IconPop,
                EventKind::PunchIn,
                EventKind::Emoji,
                EventKind::HookTitle,
            ]
        );
        let names: BTreeSet<&str> = catalog().iter().map(|row| row.name).collect();
        assert_eq!(names.len(), catalog().len());
        for row in catalog() {
            // The name on the wire is the kind's serde name.
            assert_eq!(
                serde_json::to_string(&row.kind).unwrap(),
                format!("\"{}\"", row.name)
            );
            assert_eq!(spec(row.kind), Some(row));
            let zones = [EVENT_ZONE, CAPTION_ZONE, VIDEO_ZONE, HOOK_ZONE];
            assert!(zones.contains(&row.zone), "{}", row.name);
        }
    }

    #[test]
    fn the_rows_hold_the_numbers_of_the_plan() {
        let row = |kind| spec(kind).unwrap();
        let limits = |kind| {
            (
                row(kind).max_per_clip,
                row(kind).min_gap_ms,
                row(kind).anchor,
            )
        };
        assert_eq!(
            limits(EventKind::NumberReveal),
            (8, 1_500, AnchorRule::Words { min: 1, max: 6 })
        );
        assert_eq!(
            limits(EventKind::ListReveal),
            (3, 10_000, AnchorRule::Words { min: 2, max: 400 })
        );
        assert_eq!(
            limits(EventKind::FromTo),
            (4, 3_000, AnchorRule::Words { min: 3, max: 40 })
        );
        assert_eq!(limits(EventKind::KeywordPop), (12, 800, AnchorRule::Point));
        assert_eq!(row(EventKind::KeywordPop).zone, CAPTION_ZONE);
        let v4 = catalog().iter().take(4);
        assert!(v4.clone().all(|row| row.producers == Producers::Both));
        assert_eq!(v4.count(), 4);
        // V6.2 (D-440 (1)): the five rows of §6.2, a model's only.
        assert_eq!(limits(EventKind::Emphasis), (12, 800, AnchorRule::Point));
        assert_eq!(limits(EventKind::IconPop), (6, 2_500, AnchorRule::Point));
        assert_eq!(
            limits(EventKind::PunchIn),
            (8, 1_500, AnchorRule::Words { min: 1, max: 40 })
        );
        assert_eq!(limits(EventKind::Emoji), (6, 2_500, AnchorRule::Point));
        assert_eq!(limits(EventKind::HookTitle), (1, 0, AnchorRule::ClipWide));
        let zones: Vec<&str> = catalog().iter().skip(4).map(|row| row.zone).collect();
        assert_eq!(
            zones,
            [
                CAPTION_ZONE,
                EVENT_ZONE,
                VIDEO_ZONE,
                CAPTION_ZONE,
                HOOK_ZONE
            ]
        );
        assert!(
            catalog()
                .iter()
                .skip(4)
                .all(|row| row.producers == Producers::Model)
        );
    }

    /// Every `Enum` param's names are the serde names of its type, so the model
    /// is offered exactly what deserializes (D-447).
    #[test]
    fn the_enum_params_name_what_the_types_read() {
        let names = |kind, param: &str| {
            let row = spec(kind).unwrap();
            let found = row.params.iter().find(|p| p.name == param).unwrap();
            match found.ty {
                ParamTy::Enum(values) => values.to_vec(),
                other @ (ParamTy::Label { .. }
                | ParamTy::QuantityRef
                | ParamTy::WordIdx
                | ParamTy::ListPoints { .. }
                | ParamTy::Bool
                | ParamTy::Index { .. }) => panic!("{other:?}"),
            }
        };
        let read = |values: Vec<&str>, check: &dyn Fn(&str) -> bool| {
            assert!(values.iter().all(|v| check(v)), "{values:?}");
        };
        read(names(EventKind::Emphasis, "treatment"), &|v| {
            serde_json::from_value::<offcut_types::Treatment>(serde_json::json!(v)).is_ok()
        });
        read(names(EventKind::Emphasis, "size"), &|v| {
            serde_json::from_value::<offcut_types::EmphasisSize>(serde_json::json!(v)).is_ok()
        });
        read(names(EventKind::PunchIn, "scale"), &|v| {
            serde_json::from_value::<offcut_types::ZoomScale>(serde_json::json!(v)).is_ok()
        });
        read(names(EventKind::PunchIn, "motion"), &|v| {
            serde_json::from_value::<offcut_types::ZoomMotion>(serde_json::json!(v)).is_ok()
        });
        read(names(EventKind::Emoji, "place"), &|v| {
            serde_json::from_value::<offcut_types::EmojiPlace>(serde_json::json!(v)).is_ok()
        });
        assert_eq!(names(EventKind::IconPop, "icon"), IconId::NAMES);
        assert_eq!(names(EventKind::Emoji, "emoji"), EmojiId::NAMES);
        assert_eq!(names(EventKind::Emphasis, "treatment").len(), 3);
        assert_eq!(names(EventKind::PunchIn, "motion"), ["snap", "push"]);
    }
}
