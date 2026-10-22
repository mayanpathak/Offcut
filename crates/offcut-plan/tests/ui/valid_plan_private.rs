//! A `ValidPlan` is made by `offcut_plan::validate` and nothing else (INV-21):
//! outside the crate its fields cannot be set, so it cannot be built.

use offcut_plan::ValidPlan;
use offcut_types::PlanSource;

fn main() {
    let _plan = ValidPlan {
        source: PlanSource::Rules,
        items: Vec::new(),
    };
}
