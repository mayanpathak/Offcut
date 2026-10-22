//! The edit plan (TS §40.2-§40.4, v4implementation §7): the catalog of the
//! kinds a plan may hold, the prompt and output schema generated from it, the
//! rules producer `from_rules`, and `validate`, the only constructor of a
//! `ValidPlan`. A pure crate: no clock, randomness, browser or GPU (D-122).

pub mod catalog;
pub mod config;
pub mod from_rules;
#[cfg(feature = "prompt")]
pub mod prompt;
pub mod resolve;
pub mod validate;
mod validate_params;

pub use catalog::{AnchorRule, CATALOG_VERSION, ItemSpec, ParamSpec, ParamTy, Producers, catalog};
pub use config::PlanConfig;
pub use from_rules::from_rules;
pub use validate::{
    DropReason, Dropped, PlanRejected, RejectReason, ValidItem, ValidPlan, validate,
};
