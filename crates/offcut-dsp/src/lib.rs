//! Audio signal processing: the resampler of V2, and the voice chain and the
//! prosody measure of V3. Nothing here detects silence, classifies a pause,
//! cuts, crossfades or changes a length (TS §36, INV-5, INV-10).

pub mod chain;
pub mod compressor;
pub mod config;
pub mod denoise;
pub mod highpass;
pub mod limiter;
pub mod loudness;
pub mod prosody;
pub mod resample;

pub use chain::{ChainOutput, Flow, run_chain};
pub use config::ChainConfig;
pub use prosody::measure_prosody;

use thiserror::Error;

/// Why audio cannot be processed: `run_chain` refuses empty or non-finite input.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum DspError {
    #[error("no samples")]
    Empty,
    #[error("a sample is not a finite number")]
    NonFinite,
}
