//! Audio signal processing: the resampler of V2, and the voice chain and the
//! prosody measure of V3. Nothing here detects silence, classifies a pause,
//! cuts, crossfades or changes a length (TS §36, INV-5, INV-10).

pub mod compressor;
pub mod config;
pub mod highpass;
pub mod limiter;
pub mod loudness;
pub mod resample;

pub use config::ChainConfig;

use thiserror::Error;

/// Why audio cannot be processed. Unused until the voice chain of V3.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum DspError {
    #[error("no samples")]
    Empty,
    #[error("a sample is not a finite number")]
    NonFinite,
}
