//! The rules producer (TS §40.2, v4implementation §7.7): the detector's
//! events as a plan. Each event becomes one item carrying the detector's own
//! reading (`ItemParams::Detected`, D-120); no value is changed.

use offcut_types::{DetectedEvent, EditPlan, ItemParams, PlanItem, Transcript};

use crate::catalog::CATALOG_VERSION;

/// One item per event, in the detector's order. `enabled` is not copied:
/// every item of V4 starts enabled, and the user's toggles stay in
/// `EditState.event_overrides` (D-145). `t` is unused in V4; V6.4 maps the
/// rules' kinds onto new ones with it (D-163).
pub fn from_rules(events: &[DetectedEvent], t: &Transcript) -> EditPlan {
    let _ = t;
    let mut plan = EditPlan::empty_rules(CATALOG_VERSION);
    plan.items = events
        .iter()
        .map(|event| PlanItem {
            id: event.id,
            kind: event.kind,
            anchors: event.anchors,
            params: ItemParams::Detected(event.params.clone()),
            confidence: Some(event.confidence),
        })
        .collect();
    plan
}

#[cfg(test)]
mod tests {
    use offcut_types::{
        Confidence, EventId, EventKind, EventParams, PLAN_FORMAT, PlanSource, Quantity, Span,
        TimeMs, Unit, WordIdx, WordRange,
    };

    use super::*;

    fn transcript() -> Transcript {
        Transcript {
            words: Vec::new(),
            sentences: Vec::new(),
            numbers: Vec::new(),
            model_version: "test".to_owned(),
        }
    }

    #[test]
    fn one_event_is_one_item_with_the_detectors_params() {
        let params = EventParams::NumberReveal {
            value: Quantity {
                value: 12_000.0,
                unit: Unit::Usd,
                display: "$12k".to_owned(),
            },
            label: Some("in sales".to_owned()),
        };
        let event = DetectedEvent {
            id: EventId::new(7),
            kind: EventKind::NumberReveal,
            span: Span::new(TimeMs::new(62_660), TimeMs::new(63_540)).unwrap(),
            anchors: WordRange {
                start: WordIdx::new(132),
                end: WordIdx::new(134),
            },
            params: params.clone(),
            confidence: Confidence::new(0.9).unwrap(),
            // A disabled event is still an item: the toggle is not the plan's.
            enabled: false,
        };
        let plan = from_rules(std::slice::from_ref(&event), &transcript());
        assert_eq!((plan.format, plan.catalog), (PLAN_FORMAT, CATALOG_VERSION));
        assert_eq!(plan.source, PlanSource::Rules);
        assert!(plan.cuts.is_empty() && plan.chapters.is_empty() && plan.music.is_none());
        assert_eq!(
            plan.items,
            [PlanItem {
                id: event.id,
                kind: event.kind,
                anchors: event.anchors,
                params: ItemParams::Detected(params),
                confidence: Some(event.confidence),
            }]
        );
        assert_eq!(
            from_rules(&[], &transcript()),
            EditPlan::empty_rules(CATALOG_VERSION)
        );
    }
}
