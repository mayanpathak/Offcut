//! A model's plan with the hook (V6.2, P205): a `HookTitle` over the whole
//! clip, `extras.hook` its text, and other items, validated as the pipeline
//! validates. Its own file, as `model.rs` is, so that the test binaries
//! that do not use it do not warn. Needs `scenes.rs` as the module `scenes`.

use offcut_plan::{CATALOG_VERSION, PlanConfig, validate};
use offcut_types::{
    EditPlan, EventId, EventKind, ItemParams, Label, PlanItem, PlanSource, WordIdx, WordRange,
};

use crate::scenes::Case;

/// The hook item's id in every case.
pub const HOOK_ID: u64 = 9_600;

/// `case` with a model's plan of the hook `text` (its word `accent` the
/// accent) and `more` items, validated against its transcript and edits.
pub fn with_hook(mut case: Case, text: &str, accent: u8, more: Vec<PlanItem>) -> Case {
    let words = u32::try_from(case.transcript.words.len()).unwrap();
    let hook = PlanItem {
        id: EventId::new(HOOK_ID),
        kind: EventKind::HookTitle,
        anchors: WordRange {
            start: WordIdx::new(0),
            end: WordIdx::new(words),
        },
        params: ItemParams::HookTitle { accent },
        confidence: None,
    };
    let mut plan = EditPlan::empty_rules(CATALOG_VERSION);
    plan.source = PlanSource::Model {
        provider: "deepseek".to_owned(),
        model: "test".to_owned(),
    };
    plan.items = std::iter::once(hook).chain(more).collect();
    let label: Label = serde_json::from_value(serde_json::json!(text)).unwrap();
    plan.extras.hook = Some(label);
    let (valid, _) = validate(
        plan,
        &case.transcript,
        &case.edit.word_edits,
        &PlanConfig::default(),
    )
    .unwrap();
    case.plan = valid;
    case
}
