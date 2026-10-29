//! Audio signal processing: the resampler of V2, the voice chain and the
//! prosody measure of V3, and V4's program audio (TS §40.6). Nothing here
//! classifies a pause, cuts or crossfades (TS §36, INV-5). `program::assemble`
//! is the one function that may change a length (INV-10), and in V4 it does
//! not; `silence::silence_profile` measures and decides nothing.

pub mod chain;
pub mod compressor;
pub mod config;
pub mod denoise;
pub mod highpass;
pub mod limiter;
pub mod loudness;
pub mod program;
pub mod prosody;
pub mod resample;
pub mod silence;

pub use chain::{ChainOutput, Flow, run_chain};
pub use config::ChainConfig;
pub use program::{MusicInput, ProgramAudio, ProgramConfig, assemble};
pub use prosody::measure_prosody;
pub use silence::silence_profile;

use thiserror::Error;

/// Why audio cannot be processed: `run_chain` refuses empty or non-finite
/// input; `assemble` refuses empty input, and in V4 any map but the identity
/// and any music.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum DspError {
    #[error("no samples")]
    Empty,
    #[error("a sample is not a finite number")]
    NonFinite,
    #[error("this build cannot assemble that program")]
    Unsupported,
}
