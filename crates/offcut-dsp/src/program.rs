//! [ONLY place that changes duration] The program's audio: the cleaned voice
//! laid out on the time map, with music under it (TS §40.6, seam S-6). This
//! is the one function of `offcut-dsp` that may return another length than it
//! is given (INV-10). In V4 it never does: the only map is the identity and
//! there is no music, and anything else is refused (D-164).

use offcut_types::{Lufs, OutSpan, TimeMap};

use crate::DspError;
use crate::config::ChainConfig;
use crate::loudness::integrated;

/// How a program is put together. Empty in V4: V6.2 adds the cross-fade at a
/// join, V6.3 the ducking of music under speech.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ProgramConfig {}

/// A music bed (V6.3): mono 48 kHz, with its measured loudness.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MusicInput<'a> {
    pub pcm48: &'a [f32],
    pub loudness: Lufs,
}

/// The program's audio, mono 48 kHz, and its loudness by the chain's meter.
#[derive(Clone, Debug, PartialEq)]
pub struct ProgramAudio {
    pub samples: Vec<f32>,
    pub loudness: Lufs,
}

/// `out48` played as `map` says, with `music` under the `speech` spans. V4
/// takes the identity map and no music only, and returns `out48` unchanged,
/// with its loudness; any other map or a music input is `Unsupported`. The
/// map's length is not compared with `out48`'s: the identity plays the
/// recording as it is, whatever its length.
pub fn assemble(
    out48: &[f32],
    map: &TimeMap,
    speech: &[OutSpan],
    music: Option<MusicInput<'_>>,
    cfg: &ProgramConfig,
) -> Result<ProgramAudio, DspError> {
    // Where music ducks (V6.3), and how a join is smoothed (V6.2).
    let _ = (speech, cfg);
    if !map.is_identity() || music.is_some() {
        return Err(DspError::Unsupported);
    }
    if out48.is_empty() {
        return Err(DspError::Empty);
    }
    // The chain's meter, and its reading for a programme it cannot measure.
    let loudness = integrated(out48).unwrap_or(ChainConfig::default().absolute_silence);
    Ok(ProgramAudio {
        samples: out48.to_vec(),
        loudness,
    })
}

#[cfg(test)]
mod tests {
    use offcut_types::DurMs;

    use super::*;

    #[test]
    fn the_identity_gives_the_voice_back_and_nothing_else_is_done() {
        let voice: Vec<f32> = (0..48_000).map(|i| (i as f32 * 0.01).sin() * 0.3).collect();
        let identity = TimeMap::identity(DurMs::new(1_000));
        let program = assemble(&voice, &identity, &[], None, &ProgramConfig::default()).unwrap();
        assert_eq!(program.samples, voice);
        assert_eq!(program.loudness, integrated(&voice).unwrap());
        // Silence cannot be measured: the chain's floor.
        let quiet = assemble(
            &[0.0; 48_000],
            &identity,
            &[],
            None,
            &ProgramConfig::default(),
        );
        assert_eq!(
            quiet.unwrap().loudness,
            ChainConfig::default().absolute_silence
        );
        assert_eq!(
            assemble(&[], &identity, &[], None, &ProgramConfig::default()),
            Err(DspError::Empty)
        );
        let bed = MusicInput {
            pcm48: &voice,
            loudness: Lufs::new(-20.0).unwrap(),
        };
        assert_eq!(
            assemble(&voice, &identity, &[], Some(bed), &ProgramConfig::default()),
            Err(DspError::Unsupported)
        );
    }
}
