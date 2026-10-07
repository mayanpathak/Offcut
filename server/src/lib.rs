//! The Offcut API. The modules live in this library so that the integration
//! tests under `tests/` can build the router; `main.rs` is the thin entry point.

pub mod client_ip;
pub mod config;
pub mod error;
pub mod headers;
pub mod log;
