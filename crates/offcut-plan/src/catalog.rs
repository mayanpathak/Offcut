//! [ONLY] What the renderer can draw, as a plan may ask for it (TS §40.3,
//! v4implementation D-123). The prompt, the output schema and the validator's
//! limits are all read from these rows; a change to them is a new
//! `CATALOG_VERSION`. Every number here is an assumption until V6.1 (E-11).

use offcut_types::EventKind;

/// The version of the catalog a plan is produced against (TS §40.4 V-1).
pub const CATALOG_VERSION: u32 = 1;

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

const LABEL: ParamTy = ParamTy::Label { max_chars: 32 };

/// The four kinds of V4, in the order of their codes (D-47, D-123).
const CATALOG: [ItemSpec; 4] = [
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
                EventKind::KeywordPop
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
            assert!(row.zone == EVENT_ZONE || row.zone == CAPTION_ZONE);
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
        assert!(catalog().iter().all(|row| row.producers == Producers::Both));
    }
}
