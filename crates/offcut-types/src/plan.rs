//! The edit plan's wire format (TS §40.2, v4implementation D-119, D-120).
//! One shape, fixed at `PLAN_FORMAT = 1`, for every producer: the rules of
//! V4 and the model of V6.1 on. These are data only: what a plan means, and
//! whether it is valid, is decided by `offcut-plan`.
//!
//! Serde: every struct refuses unknown fields; an enum whose variants carry
//! data is tagged `"kind"`, in snake case, and an enum of names only is a
//! snake-case string; `ItemParams` is tagged `"kind"` with its fields under
//! `"params"`, so that `Detected(EventParams)`, tagged `"kind"` itself, does
//! not collide.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::events::{EventKind, EventParams};
use crate::ids::{EventId, WordIdx, WordRange};
use crate::units::{Confidence, DurMs};

/// The version of this wire format. A plan of another format is refused.
pub const PLAN_FORMAT: u16 = 1;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct EditPlan {
    /// `PLAN_FORMAT`.
    pub format: u16,
    /// The `CATALOG_VERSION` the plan was produced against.
    pub catalog: u32,
    pub source: PlanSource,
    /// Empty until V6.2.
    pub cuts: Vec<Cut>,
    pub items: Vec<PlanItem>,
    /// Empty until V6.1.
    pub chapters: Vec<Chapter>,
    /// Empty until V6.1.
    pub extras: TextExtras,
    /// `None` until V6.3.
    pub music: Option<MusicCue>,
}

/// Who wrote the plan: set by the producer, never by a model.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PlanSource {
    Rules,
    /// `provider` is a provider id (V6.1 decides the list).
    Model {
        provider: String,
        model: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Cut {
    /// The silence before the first word, shortened to `keep`.
    Lead {
        keep: DurMs,
    },
    /// The silence after the last word, shortened to `keep`.
    Trail {
        keep: DurMs,
    },
    /// The pause between `after` and the next word, shortened to `keep`.
    Gap {
        after: WordIdx,
        keep: DurMs,
    },
    /// An intentional pause: no `Gap` cut may touch it.
    KeepGap {
        after: WordIdx,
    },
    Words {
        range: WordRange,
        why: CutWhy,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum CutWhy {
    Filler,
    FalseStart,
}

/// One visual of the plan. `kind` is the event kind (D-118); `params` repeat
/// it as their tag, and validation checks that the two agree.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct PlanItem {
    pub id: EventId,
    pub kind: EventKind,
    pub anchors: WordRange,
    pub params: ItemParams,
    /// `Some` for an item of the rules (INV-6), `None` for one of a model.
    pub confidence: Option<Confidence>,
}

/// What an item shows. The four kinds point at words; `Detected` carries the
/// detector's own reading and is written by the rules producer only (D-120).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    content = "params",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum ItemParams {
    NumberReveal {
        value: QuantityRef,
        label: Option<Label>,
    },
    ListReveal {
        header: Option<Label>,
        items: Vec<ListPoint>,
    },
    FromTo {
        from: QuantityRef,
        to: QuantityRef,
        label: Option<Label>,
    },
    KeywordPop {
        word: WordIdx,
    },
    Detected(EventParams),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct ListPoint {
    pub at: WordIdx,
    pub label: Label,
}

/// A number as a pointer to the words that say it; `validate` reads the
/// number from them (D-121). One field, so that a later form can be added
/// beside `words` without breaking format 1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct QuantityRef {
    pub words: WordRange,
}

/// Text a plan shows. Its length and digit rules are `validate`'s (TS §40.4 V-4).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(transparent)]
pub struct Label(String);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct Chapter {
    pub starts_at: WordIdx,
    pub title: Label,
}

/// Text around the video. Every field may be left out (empty until V6.1).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(default, deny_unknown_fields)]
pub struct TextExtras {
    pub hook: Option<Label>,
    pub post_caption: Option<String>,
    pub hashtags: Vec<Label>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum MusicEnergy {
    Calm,
    Steady,
    Upbeat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct MusicCue {
    pub fits: bool,
    pub energy: MusicEnergy,
}

impl Label {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl EditPlan {
    /// A plan of the rules with nothing in it, against `catalog`.
    pub fn empty_rules(catalog: u32) -> EditPlan {
        EditPlan {
            format: PLAN_FORMAT,
            catalog,
            source: PlanSource::Rules,
            cuts: Vec::new(),
            items: Vec::new(),
            chapters: Vec::new(),
            extras: TextExtras::default(),
            music: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::ListItem;
    use crate::transcript::{Quantity, Unit};
    use crate::units::TimeMs;

    fn words(start: u32, end: u32) -> WordRange {
        WordRange {
            start: WordIdx::new(start),
            end: WordIdx::new(end),
        }
    }

    fn label(text: &str) -> Label {
        Label(text.to_owned())
    }

    fn item(id: u64, kind: EventKind, anchors: WordRange, params: ItemParams) -> PlanItem {
        PlanItem {
            id: EventId::new(id),
            kind,
            anchors,
            params,
            confidence: Confidence::new(0.9),
        }
    }

    /// A plan with every arm of every enum and every optional field set.
    fn full() -> EditPlan {
        let quantity = |value: f64, display: &str| Quantity {
            value,
            unit: Unit::Minutes,
            display: display.to_owned(),
        };
        let detected = EventParams::ListReveal {
            count: 2,
            header: "2 STEPS".to_owned(),
            items: vec![ListItem {
                ordinal: 1,
                text: "plan".to_owned(),
                at: TimeMs::new(4_000),
            }],
        };
        let from_to = EventParams::FromTo {
            from: quantity(90.0, "90 minutes"),
            to: quantity(10.0, "10 minutes"),
            label: Some("a rough cut".to_owned()),
        };
        EditPlan {
            format: PLAN_FORMAT,
            catalog: 1,
            source: PlanSource::Model {
                provider: "provider-a".to_owned(),
                model: "model-1".to_owned(),
            },
            cuts: vec![
                Cut::Lead {
                    keep: DurMs::new(200),
                },
                Cut::Trail {
                    keep: DurMs::new(300),
                },
                Cut::Gap {
                    after: WordIdx::new(4),
                    keep: DurMs::new(150),
                },
                Cut::KeepGap {
                    after: WordIdx::new(9),
                },
                Cut::Words {
                    range: words(12, 13),
                    why: CutWhy::Filler,
                },
                Cut::Words {
                    range: words(20, 22),
                    why: CutWhy::FalseStart,
                },
            ],
            items: vec![
                item(
                    1,
                    EventKind::NumberReveal,
                    words(3, 5),
                    ItemParams::NumberReveal {
                        value: QuantityRef { words: words(3, 5) },
                        label: Some(label("in sales")),
                    },
                ),
                item(
                    2,
                    EventKind::ListReveal,
                    words(6, 20),
                    ItemParams::ListReveal {
                        header: None,
                        items: vec![ListPoint {
                            at: WordIdx::new(8),
                            label: label("captions"),
                        }],
                    },
                ),
                item(
                    3,
                    EventKind::FromTo,
                    words(21, 27),
                    ItemParams::FromTo {
                        from: QuantityRef {
                            words: words(22, 24),
                        },
                        to: QuantityRef {
                            words: words(25, 27),
                        },
                        label: None,
                    },
                ),
                PlanItem {
                    confidence: None,
                    ..item(
                        4,
                        EventKind::KeywordPop,
                        words(30, 31),
                        ItemParams::KeywordPop {
                            word: WordIdx::new(30),
                        },
                    )
                },
                item(
                    5,
                    EventKind::ListReveal,
                    words(40, 50),
                    ItemParams::Detected(detected),
                ),
                item(
                    6,
                    EventKind::FromTo,
                    words(51, 57),
                    ItemParams::Detected(from_to),
                ),
            ],
            chapters: vec![Chapter {
                starts_at: WordIdx::new(0),
                title: label("Why"),
            }],
            extras: TextExtras {
                hook: Some(label("I almost quit")),
                post_caption: Some("How I got my evenings back.".to_owned()),
                hashtags: vec![label("editing")],
            },
            music: Some(MusicCue {
                fits: true,
                energy: MusicEnergy::Steady,
            }),
        }
    }

    fn round_trips<T>(value: &T)
    where
        T: Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug,
    {
        let json = serde_json::to_string(value).unwrap();
        assert_eq!(&serde_json::from_str::<T>(&json).unwrap(), value, "{json}");
    }

    #[test]
    fn a_plan_with_every_arm_round_trips() {
        let plan = full();
        round_trips(&plan);
        round_trips(&PlanSource::Rules);
        for energy in [MusicEnergy::Calm, MusicEnergy::Steady, MusicEnergy::Upbeat] {
            round_trips(&energy);
        }
        round_trips(&EditPlan::empty_rules(7));
        assert_eq!(plan.items.len(), 6);
    }

    #[test]
    fn the_names_on_the_wire() {
        let json = |value: &dyn erased::Json| value.json();
        assert_eq!(json(&PlanSource::Rules), r#"{"kind":"rules"}"#);
        assert_eq!(
            json(&Cut::Gap {
                after: WordIdx::new(4),
                keep: DurMs::new(150)
            }),
            r#"{"kind":"gap","after":4,"keep":150}"#
        );
        assert_eq!(
            json(&Cut::KeepGap {
                after: WordIdx::new(9)
            }),
            r#"{"kind":"keep_gap","after":9}"#
        );
        assert_eq!(json(&CutWhy::FalseStart), r#""false_start""#);
        assert_eq!(json(&MusicEnergy::Upbeat), r#""upbeat""#);
        assert_eq!(json(&label("in sales")), r#""in sales""#);
        assert_eq!(label("in sales").as_str(), "in sales");
        assert_eq!(
            json(&QuantityRef { words: words(3, 5) }),
            r#"{"words":{"start":3,"end":5}}"#
        );
        assert_eq!(
            json(&TextExtras::default()),
            r#"{"hook":null,"post_caption":null,"hashtags":[]}"#
        );
    }

    /// `serde_json::to_string` behind one trait object, for the table above.
    mod erased {
        pub trait Json {
            fn json(&self) -> String;
        }
        impl<T: serde::Serialize> Json for T {
            fn json(&self) -> String {
                serde_json::to_string(self).unwrap()
            }
        }
    }

    /// The JSON of a one-item plan of the rules, exactly. A contract with
    /// V6.1's server and with stored plans: a change here is a new
    /// `PLAN_FORMAT`, never an edit of this string.
    #[test]
    fn the_json_of_a_one_item_rules_plan_is_pinned() {
        let mut plan = EditPlan::empty_rules(1);
        plan.items.push(PlanItem {
            id: EventId::new(42),
            kind: EventKind::NumberReveal,
            anchors: words(132, 134),
            params: ItemParams::Detected(EventParams::NumberReveal {
                value: Quantity {
                    value: 12_000.0,
                    unit: Unit::Usd,
                    display: "$12k".to_owned(),
                },
                label: Some("in sales".to_owned()),
            }),
            confidence: Confidence::new(1.0),
        });
        let json = serde_json::to_string(&plan).unwrap();
        assert_eq!(
            json,
            concat!(
                r#"{"format":1,"catalog":1,"source":{"kind":"rules"},"cuts":[],"items":[{"id":"42","#,
                r#""kind":"number_reveal","anchors":{"start":132,"end":134},"params":{"kind":"detected","#,
                r#""params":{"kind":"number_reveal","value":{"value":12000.0,"unit":"usd","display":"$12k"},"#,
                r#""label":"in sales"}},"confidence":1.0}],"chapters":[],"extras":{"hook":null,"#,
                r#""post_caption":null,"hashtags":[]},"music":null}"#
            )
        );
        assert_eq!(serde_json::from_str::<EditPlan>(&json).unwrap(), plan);
    }

    #[test]
    fn detected_params_keep_their_own_kind_inside_the_item_tag() {
        let params = ItemParams::Detected(EventParams::KeywordPop {
            word: WordIdx::new(31),
        });
        let json = serde_json::to_string(&params).unwrap();
        assert_eq!(
            json,
            r#"{"kind":"detected","params":{"kind":"keyword_pop","word":31}}"#
        );
        assert_eq!(serde_json::from_str::<ItemParams>(&json).unwrap(), params);
        let pointer = ItemParams::KeywordPop {
            word: WordIdx::new(31),
        };
        assert_eq!(
            serde_json::to_string(&pointer).unwrap(),
            r#"{"kind":"keyword_pop","params":{"word":31}}"#
        );
    }

    #[test]
    fn an_unknown_field_is_refused_at_every_level() {
        let refused = |json: &str, why: &str| {
            let error = serde_json::from_str::<EditPlan>(json)
                .unwrap_err()
                .to_string();
            assert!(error.contains("unknown field"), "{why}: {error}");
        };
        let plan = |items: &str, extras: &str, music: &str| {
            format!(
                r#"{{"format":1,"catalog":1,"source":{{"kind":"rules"}},"cuts":[],"items":[{items}],"chapters":[],"extras":{extras},"music":{music}}}"#
            )
        };
        let good_item = r#"{"id":"1","kind":"number_reveal","anchors":{"start":3,"end":5},"params":{"kind":"number_reveal","params":{"value":{"words":{"start":3,"end":5}},"label":null}},"confidence":null}"#;
        // The good plan reads, so each refusal below is the one extra field.
        assert!(serde_json::from_str::<EditPlan>(&plan(good_item, "{}", "null")).is_ok());

        let with_extra_field =
            plan(good_item, "{}", "null").replacen("{\"format\"", "{\"x\":0,\"format\"", 1);
        refused(&with_extra_field, "the plan");
        refused(
            &plan(
                &good_item.replacen("{\"id\"", "{\"x\":0,\"id\"", 1),
                "{}",
                "null",
            ),
            "an item",
        );
        refused(
            &plan(
                &good_item.replacen("\"label\":null", "\"label\":null,\"x\":0", 1),
                "{}",
                "null",
            ),
            "a param block",
        );
        refused(
            &plan(
                &good_item.replacen("{\"words\"", "{\"x\":0,\"words\"", 1),
                "{}",
                "null",
            ),
            "a quantity ref",
        );
        refused(&plan(good_item, r#"{"x":0}"#, "null"), "the text extras");
        refused(
            &plan(good_item, "{}", r#"{"fits":true,"energy":"calm","x":0}"#),
            "the music cue",
        );
        // The source with fields. Serde cannot refuse an extra field beside
        // the tag of a field-less variant (`{"kind":"rules","x":0}` reads as
        // `Rules`); the producer sets the source, never a model (TS §40.2).
        refused(
            &plan(good_item, "{}", "null").replacen(
                r#"{"kind":"rules"}"#,
                r#"{"kind":"model","provider":"a","model":"b","x":0}"#,
                1,
            ),
            "the source",
        );
    }
}
