//! The user's choices for one clip. No field changes audio, duration or timing.

use std::collections::BTreeMap;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};
use ts_rs::TS;

use crate::enhance::Enhance;
use crate::ids::{EventId, WordIdx};

/// Caption style (PS §12.2).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum StyleId {
    #[default]
    Clean,
    Bold,
    Tech,
}

/// Horizontal crop position: -1.0 (left) ..= 1.0 (right); 0.0 is the centre.
/// Checked in `new` and when deserializing.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, TS)]
pub struct CropOffset(f32);

impl CropOffset {
    /// `None` when `v` is not finite or is outside `-1.0..=1.0`.
    pub fn new(v: f32) -> Option<Self> {
        (-1.0..=1.0).contains(&v).then_some(Self(v))
    }

    pub const fn get(self) -> f32 {
        self.0
    }
}

impl<'de> Deserialize<'de> for CropOffset {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let v = f32::deserialize(deserializer)?;
        Self::new(v)
            .ok_or_else(|| D::Error::custom("a crop offset must be a finite number in -1.0..=1.0"))
    }
}

/// Where the caption block's centre sits: 0.0 at the top of the band captions
/// may use, 1.0 at its bottom (v4implementation D-171). Checked in `new` and
/// when deserializing.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, TS)]
pub struct CaptionPlacement(f32);

impl CaptionPlacement {
    /// `None` when `v` is not finite or is outside `0.0..=1.0`.
    pub fn new(v: f32) -> Option<Self> {
        (0.0..=1.0).contains(&v).then_some(Self(v))
    }

    pub const fn get(self) -> f32 {
        self.0
    }

    /// The default: the bottom of the band, V3's caption zone.
    pub const fn bottom() -> Self {
        Self(1.0)
    }
}

impl<'de> Deserialize<'de> for CaptionPlacement {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let v = f32::deserialize(deserializer)?;
        Self::new(v).ok_or_else(|| {
            D::Error::custom("a caption placement must be a finite number in 0.0..=1.0")
        })
    }
}

/// The serde default of `highlight_numbers`.
fn yes() -> bool {
    true
}

/// REOPENED in V4 (D-169, D-171): the last two fields. Their serde defaults
/// make a record V3 wrote read as V4's default.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
pub struct EditState {
    /// Replacement text; "" hides the word; timing never changes.
    pub word_edits: BTreeMap<WordIdx, String>,
    /// The enabled flag chosen by the user.
    pub event_overrides: BTreeMap<EventId, bool>,
    pub style: StyleId,
    pub crop_offset: CropOffset,
    #[serde(default = "CaptionPlacement::bottom")]
    pub caption_position: CaptionPlacement,
    /// Numbers in the captions in the accent colour (D-169).
    #[serde(default = "yes")]
    pub highlight_numbers: bool,
    /// REOPENED in V6.2 (v6.2implementation D-485, DC-16): the Enhance
    /// settings. Left out of the JSON when it is `Enhance::v5()`, so a state
    /// without a choice serializes as V6.1 wrote it, and a record without the
    /// field reads as `v5()`.
    #[serde(default = "Enhance::v5", skip_serializing_if = "Enhance::is_v5")]
    #[ts(optional, as = "Option<Enhance>")]
    pub enhance: Enhance,
}

impl Default for EditState {
    /// No edits, no overrides, `StyleId::Clean`, offset `0.0`, captions at
    /// the bottom, numbers highlighted.
    fn default() -> Self {
        Self {
            word_edits: BTreeMap::new(),
            event_overrides: BTreeMap::new(),
            style: StyleId::Clean,
            crop_offset: CropOffset(0.0),
            caption_position: CaptionPlacement::bottom(),
            highlight_numbers: true,
            enhance: Enhance::v5(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crop_offset_accepts_only_minus_one_to_one() {
        for ok in [-1.0, -0.5, 0.0, 0.25, 1.0] {
            assert_eq!(CropOffset::new(ok).map(CropOffset::get), Some(ok));
        }
        for bad in [
            -1.001,
            1.001,
            2.0,
            f32::NAN,
            f32::INFINITY,
            f32::NEG_INFINITY,
        ] {
            assert_eq!(CropOffset::new(bad), None, "{bad} was accepted");
        }
    }

    #[test]
    fn crop_offset_deserializes_through_new() {
        let offset = CropOffset::new(-0.5).unwrap();
        assert_eq!(serde_json::to_string(&offset).unwrap(), "-0.5");
        assert_eq!(serde_json::from_str::<CropOffset>("-0.5").unwrap(), offset);
        assert!(serde_json::from_str::<CropOffset>("1.5").is_err());
        assert!(serde_json::from_str::<CropOffset>("-1.01").is_err());
    }

    #[test]
    fn style_id_serializes_as_snake_case_and_defaults_to_clean() {
        let cases = [
            (StyleId::Clean, "clean"),
            (StyleId::Bold, "bold"),
            (StyleId::Tech, "tech"),
        ];
        for (style, name) in cases {
            let json = serde_json::to_string(&style).unwrap();
            assert_eq!(json, format!("\"{name}\""));
            assert_eq!(serde_json::from_str::<StyleId>(&json).unwrap(), style);
        }
        assert_eq!(StyleId::default(), StyleId::Clean);
    }

    #[test]
    fn default_edit_state_has_no_edits() {
        let state = EditState::default();
        assert!(state.word_edits.is_empty());
        assert!(state.event_overrides.is_empty());
        assert_eq!(state.style, StyleId::Clean);
        assert_eq!(state.crop_offset.get(), 0.0);
        assert_eq!(
            serde_json::to_string(&state).unwrap(),
            r#"{"word_edits":{},"event_overrides":{},"style":"clean","crop_offset":0.0,"caption_position":1.0,"highlight_numbers":true}"#
        );
    }

    #[test]
    fn edit_state_round_trips_with_ids_as_map_keys() {
        let mut state = EditState::default();
        state
            .word_edits
            .insert(WordIdx::new(3), "ten thousand".to_owned());
        state.word_edits.insert(WordIdx::new(9), String::new());
        state.event_overrides.insert(EventId::new(u64::MAX), false);
        state.style = StyleId::Tech;
        state.crop_offset = CropOffset::new(0.25).unwrap();

        let value = serde_json::to_value(&state).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "word_edits": { "3": "ten thousand", "9": "" },
                "event_overrides": { "18446744073709551615": false },
                "style": "tech",
                "crop_offset": 0.25,
                "caption_position": 1.0,
                "highlight_numbers": true,
            })
        );
        assert_eq!(serde_json::from_value::<EditState>(value).unwrap(), state);
    }

    #[test]
    fn a_v3_record_reads_with_the_two_v4_defaults() {
        let v3 = r#"{"word_edits":{},"event_overrides":{},"style":"clean","crop_offset":0}"#;
        let state: EditState = serde_json::from_str(v3).unwrap();
        assert_eq!(state.caption_position.get(), 1.0);
        assert!(state.highlight_numbers);
        assert_eq!(state, EditState::default());
    }

    #[test]
    fn caption_placement_accepts_only_zero_to_one() {
        for ok in [0.0, 0.15, 0.55, 1.0] {
            assert_eq!(
                CaptionPlacement::new(ok).map(CaptionPlacement::get),
                Some(ok)
            );
        }
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -0.01, 1.01] {
            assert_eq!(CaptionPlacement::new(bad), None, "{bad} was accepted");
        }
        assert_eq!(CaptionPlacement::bottom().get(), 1.0);
    }

    #[test]
    fn caption_placement_deserializes_through_new() {
        let at = CaptionPlacement::new(0.55).unwrap();
        assert_eq!(serde_json::to_string(&at).unwrap(), "0.55");
        assert_eq!(
            serde_json::from_str::<CaptionPlacement>("0.55").unwrap(),
            at
        );
        // JSON has no NaN or infinity; the nearest a record can come is a
        // number outside the range, or a value that is not a number.
        for bad in ["-0.01", "1.01", "1e39", "null", "\"1\""] {
            assert!(
                serde_json::from_str::<CaptionPlacement>(bad).is_err(),
                "{bad}"
            );
        }
        let json = r#"{"word_edits":{},"event_overrides":{},"style":"clean","crop_offset":0,"caption_position":1.5}"#;
        let error = serde_json::from_str::<EditState>(json)
            .unwrap_err()
            .to_string();
        assert!(error.contains("caption placement"), "{error}");
        let off = r#"{"word_edits":{},"event_overrides":{},"style":"clean","crop_offset":0,"caption_position":0.15,"highlight_numbers":false}"#;
        let state: EditState = serde_json::from_str(off).unwrap();
        assert_eq!(
            (state.caption_position.get(), state.highlight_numbers),
            (0.15, false)
        );
    }

    // V6.2 (D-485, DC-16): `enhance`.

    /// `default_edit_state_has_no_edits`' literal at `b73fea5`.
    const V61_DEFAULT: &str = r#"{"word_edits":{},"event_overrides":{},"style":"clean","crop_offset":0.0,"caption_position":1.0,"highlight_numbers":true}"#;

    #[test]
    fn a_v5_state_serializes_as_v6_1_wrote_it() {
        let state = EditState::default();
        assert!(state.enhance.is_v5());
        assert_eq!(serde_json::to_string(&state).unwrap(), V61_DEFAULT);
        let edited = EditState {
            style: StyleId::Bold,
            enhance: Enhance::v5(),
            ..EditState::default()
        };
        let json = serde_json::to_string(&edited).unwrap();
        assert!(!json.contains("enhance"), "{json}");
    }

    #[test]
    fn a_record_without_the_field_reads_v5() {
        let state: EditState = serde_json::from_str(V61_DEFAULT).unwrap();
        assert_eq!(state.enhance, Enhance::v5());
        assert_eq!(state, EditState::default());
        let v3 = r#"{"word_edits":{},"event_overrides":{},"style":"clean","crop_offset":0}"#;
        let state: EditState = serde_json::from_str(v3).unwrap();
        assert_eq!(state.enhance, Enhance::v5());
    }

    #[test]
    fn motion_on_round_trips() {
        let mut state = EditState {
            enhance: Enhance::defaults(true),
            ..EditState::default()
        };
        let value = serde_json::to_value(&state).unwrap();
        assert_eq!(
            value["enhance"],
            serde_json::json!({ "voice": "clean", "motion": true })
        );
        assert_eq!(serde_json::from_value::<EditState>(value).unwrap(), state);
        state.enhance = Enhance {
            voice: crate::enhance::VoiceMode::Level,
            motion: false,
        };
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(serde_json::from_str::<EditState>(&json).unwrap(), state);
    }

    /// A V6.1 build (rollback, §14.4) reads a V6.2 record that carries the
    /// field: `EditState` at `b73fea5` does not deny unknown fields.
    #[test]
    fn a_v6_1_reader_accepts_a_v6_2_record() {
        #[derive(Deserialize)]
        struct EditStateV61 {
            word_edits: BTreeMap<WordIdx, String>,
            event_overrides: BTreeMap<EventId, bool>,
            style: StyleId,
            crop_offset: CropOffset,
            #[serde(default = "CaptionPlacement::bottom")]
            caption_position: CaptionPlacement,
            #[serde(default = "yes")]
            highlight_numbers: bool,
        }
        let state = EditState {
            style: StyleId::Tech,
            enhance: Enhance::defaults(true),
            ..EditState::default()
        };
        let json = serde_json::to_string(&state).unwrap();
        assert!(
            json.contains(r#""enhance":{"voice":"clean","motion":true}"#),
            "{json}"
        );
        let old: EditStateV61 = serde_json::from_str(&json).unwrap();
        assert_eq!(
            (
                old.word_edits.len(),
                old.event_overrides.len(),
                old.style,
                old.crop_offset.get(),
                old.caption_position.get(),
                old.highlight_numbers,
            ),
            (0, 0, StyleId::Tech, 0.0, 1.0, true)
        );
    }

    #[test]
    fn the_typescript_field_is_optional() {
        let ts = EditState::decl(&ts_rs::Config::default());
        assert!(ts.contains("enhance?: Enhance"), "{ts}");
    }

    #[test]
    fn edit_state_with_a_bad_crop_offset_is_rejected() {
        let json = r#"{"word_edits":{},"event_overrides":{},"style":"clean","crop_offset":3.0}"#;
        assert!(serde_json::from_str::<EditState>(json).is_err());
    }
}
