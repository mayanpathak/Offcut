//! The Enhance settings of one clip (TS §47.5; v6.2implementation D-485):
//! what each enhancement does, switchable while reviewing. V6.2 has two
//! fields; `picture` and `sfx` join with V6.6a and V6.7, each with
//! `#[serde(default)]`, so a record of an earlier build still reads.
//!
//! `Enhance::v5()` is V5's output: the full voice chain and no camera motion.
//! `EditState` leaves the field out of its JSON when it is `v5()`, so every
//! stored record, the render-cache key and the pinned JSON stay as they were
//! (DC-16).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// What the voice stage does.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum VoiceMode {
    /// V5's chain: high-pass, denoise, compression, loudness and the limiter.
    #[default]
    Clean,
    /// Loudness to the target and the limiter only (D-487).
    Level,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
pub struct Enhance {
    pub voice: VoiceMode,
    /// The camera drift and the rules' zooms (TS §47.2).
    pub motion: bool,
}

impl Enhance {
    /// V5's frames and samples: the clean chain, no motion.
    pub const fn v5() -> Enhance {
        Enhance {
            voice: VoiceMode::Clean,
            motion: false,
        }
    }

    /// True when this is `v5()`; `EditState` then leaves the field out.
    pub fn is_v5(&self) -> bool {
        *self == Enhance::v5()
    }

    /// A new clip's settings: every shipped enhancement on. Motion follows
    /// its flag (off in production until V6.5, D-497).
    pub const fn defaults(motion_flag: bool) -> Enhance {
        Enhance {
            voice: VoiceMode::Clean,
            motion: motion_flag,
        }
    }
}

impl Default for Enhance {
    fn default() -> Self {
        Enhance::v5()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v5_is_the_clean_chain_without_motion() {
        let v5 = Enhance::v5();
        assert_eq!((v5.voice, v5.motion), (VoiceMode::Clean, false));
        assert!(v5.is_v5());
        assert_eq!(Enhance::default(), v5);
        assert_eq!(Enhance::defaults(false), v5);
        assert!(!Enhance::defaults(true).is_v5());
        assert!(
            !Enhance {
                voice: VoiceMode::Level,
                motion: false
            }
            .is_v5()
        );
    }

    #[test]
    fn enhance_serializes_with_snake_case_names() {
        let level = Enhance {
            voice: VoiceMode::Level,
            motion: true,
        };
        let json = serde_json::to_string(&level).unwrap();
        assert_eq!(json, r#"{"voice":"level","motion":true}"#);
        assert_eq!(serde_json::from_str::<Enhance>(&json).unwrap(), level);
        assert_eq!(
            serde_json::to_string(&VoiceMode::Clean).unwrap(),
            r#""clean""#
        );
    }

    #[test]
    fn a_later_field_is_read_past() {
        // V6.6a and V6.7 append `picture` and `sfx`; a V6.2 build reads such
        // a record without them (no `deny_unknown_fields`).
        let later = r#"{"voice":"clean","motion":true,"picture":true,"sfx":false}"#;
        let read: Enhance = serde_json::from_str(later).unwrap();
        assert_eq!(read, Enhance::defaults(true));
    }
}
