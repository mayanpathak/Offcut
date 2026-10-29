//! [ONLY] The limits `validate` applies (v4implementation D-165): per kind,
//! read from the catalog, and the cut and chapter limits of TS §40.4, which
//! nothing reads until V6.2 and V6.6.

use std::collections::BTreeMap;

use offcut_types::{DurMs, EventKind};

use crate::catalog::{ParamTy, catalog};

/// The limits of one kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KindLimits {
    pub max_per_clip: u8,
    /// The least time from the end of a kept item to the start of the next of its kind.
    pub min_gap: DurMs,
    /// The longest label of the kind (a list point's label too), in characters; 0 when it has none.
    pub label_chars: u32,
    /// The longest header (ListReveal), in characters; 0 when it has none.
    pub header_chars: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanConfig {
    pub per_kind: BTreeMap<EventKind, KindLimits>,
    /// A shortened pause keeps at least this much (V6.2).
    pub min_keep_gap: DurMs,
    /// At most this share of the clip is removed, in thousandths (V6.2).
    pub max_cut_share_permille: u32,
    /// At most this many chapters (V6.6).
    pub max_chapters: u32,
}

/// The longest label among the params of a kind, ignoring the header.
fn limits(row: &crate::catalog::ItemSpec) -> KindLimits {
    let (mut label_chars, mut header_chars) = (0, 0);
    for param in row.params {
        match param.ty {
            ParamTy::Label { max_chars } if param.name == "header" => header_chars = max_chars,
            ParamTy::Label { max_chars } => label_chars = label_chars.max(max_chars),
            ParamTy::ListPoints {
                label_chars: chars, ..
            } => label_chars = label_chars.max(chars),
            ParamTy::QuantityRef | ParamTy::WordIdx | ParamTy::Enum(_) | ParamTy::Bool => {}
        }
    }
    KindLimits {
        max_per_clip: row.max_per_clip,
        min_gap: DurMs::new(row.min_gap_ms),
        label_chars,
        header_chars,
    }
}

impl Default for PlanConfig {
    /// The catalog's numbers; the cut fields at TS §40.4's 150 ms, 40% and 6.
    fn default() -> Self {
        PlanConfig {
            per_kind: catalog()
                .iter()
                .map(|row| (row.kind, limits(row)))
                .collect(),
            min_keep_gap: DurMs::new(150),
            max_cut_share_permille: 400,
            max_chapters: 6,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_reads_the_catalog() {
        let cfg = PlanConfig::default();
        assert_eq!(cfg.per_kind.len(), 4);
        let limits = |kind| *cfg.per_kind.get(&kind).unwrap();
        assert_eq!(
            limits(EventKind::NumberReveal),
            KindLimits {
                max_per_clip: 8,
                min_gap: DurMs::new(1_500),
                label_chars: 32,
                header_chars: 0
            }
        );
        assert_eq!(
            limits(EventKind::ListReveal),
            KindLimits {
                max_per_clip: 3,
                min_gap: DurMs::new(10_000),
                label_chars: 32,
                header_chars: 24
            }
        );
        assert_eq!(limits(EventKind::FromTo).max_per_clip, 4);
        assert_eq!(limits(EventKind::FromTo).min_gap, DurMs::new(3_000));
        assert_eq!(limits(EventKind::KeywordPop).label_chars, 0);
        assert_eq!(limits(EventKind::KeywordPop).min_gap, DurMs::new(800));
        assert_eq!(
            (
                cfg.min_keep_gap,
                cfg.max_cut_share_permille,
                cfg.max_chapters
            ),
            (DurMs::new(150), 400, 6)
        );
    }
}
