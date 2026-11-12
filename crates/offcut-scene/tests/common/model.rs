//! A model's plan for a test case (V6.2, P200): the kinds only a model
//! writes (`PunchIn`, and P201-P205's), validated as the pipeline validates.
//! Needs `scenes.rs` as the module `scenes`.

use offcut_plan::{CATALOG_VERSION, PlanConfig, validate};
use offcut_types::{EditPlan, PlanItem, PlanSource};

use crate::scenes::Case;

/// `case` with a model's plan of `items` in place of its rules plan,
/// validated against its transcript and word edits. Items the validation
/// drops are not in the plan.
pub fn with_model_items(mut case: Case, items: Vec<PlanItem>) -> Case {
    let mut plan = EditPlan::empty_rules(CATALOG_VERSION);
    plan.source = PlanSource::Model {
        provider: "deepseek".to_owned(),
        model: "test".to_owned(),
    };
    plan.items = items;
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
