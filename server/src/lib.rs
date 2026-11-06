//! The Offcut API. The modules live in this library so that the integration
//! tests under `tests/` can build the router; `main.rs` is the thin entry point.

pub mod account;
pub mod analytics;
pub mod auth;
pub mod billing;
pub mod client_ip;
pub mod config;
pub mod db;
pub mod entitlement;
pub mod error;
pub mod headers;
pub mod log;
pub mod mail;
pub mod rate_limit;
pub mod router;
pub mod state;
pub mod transcribe;
pub mod usage;
