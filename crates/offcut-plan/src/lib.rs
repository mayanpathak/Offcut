//! The edit plan (TS §40.2-§40.4, v4implementation §7): the catalog of the
//! kinds a plan may hold, the prompt and output schema generated from it, the
//! rules producer `from_rules`, and `validate`, the only constructor of a
//! `ValidPlan`. A pure crate: no clock, randomness, browser or GPU (D-122).
