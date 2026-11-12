//! PunchIn in a scene (v6.2implementation D-477, D-478; TS §47.2): a model's
//! plan with one zoom, over written sentences, read at every frame. The zoom
//! changes only the video's `src`; it holds over its anchors and lets go in
//! 300 ms; it multiplies the camera's drift with motion on and stays with
//! motion off; switched off, it is gone; it counts as a visual moment.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

#[path = "common/model.rs"]
mod model;
#[path = "common/scenes.rs"]
mod scenes;

use std::collections::BTreeMap;

use offcut_scene::camera::for_clip;
use offcut_scene::events::punch_in::{STRONG, ZOOM_RELEASE_MS};
use offcut_scene::{CropRect, DrawCmd, Scene};
use offcut_types::{
    DurMs, EventId, EventKind, ItemParams, OutMs, PlanItem, TimeMap, WordIdx, WordRange,
    ZoomMotion, ZoomScale,
};

const ID: u64 = 9_100;

/// "We crossed 3,000 users last month." and a tail, with a model's plan of
/// one zoom on "3,000 users", and `motion` on or off.
fn case(motion: ZoomMotion, drift: bool) -> scenes::Case {
    let sentences = [
        "We crossed 3,000 users last month.",
        "Thanks for watching, and see you next week.",
    ];
    let case = scenes::written("punch", &sentences, BTreeMap::new(), |edit, _| {
        edit.enhance.motion = drift;
    });
    let words = &case.transcript.words;
    let three = words.iter().position(|w| w.text == "3,000").unwrap() as u32;
    let zoom = PlanItem {
        id: EventId::new(ID),
        kind: EventKind::PunchIn,
        anchors: WordRange {
            start: WordIdx::new(three),
            end: WordIdx::new(three + 2),
        },
        params: ItemParams::PunchIn {
            scale: ZoomScale::Strong,
            motion,
        },
        confidence: None,
    };
    let case = model::with_model_items(case, vec![zoom]);
    assert_eq!(case.plan.items().len(), 1, "the zoom passed validation");
    case
}

/// The zoom's program span (the identity map).
fn span(case: &scenes::Case) -> (u32, u32) {
    let item = &case.plan.items()[0];
    assert_eq!(item.kind, EventKind::PunchIn, "{}", case.name);
    (item.span.start.get(), item.span.end.get())
}

fn src(scene: &Scene, t: u32) -> CropRect {
    let list = scene.frame_at(OutMs::new(t));
    let DrawCmd::Video { src, .. } = &list.cmds[0] else {
        panic!("the video first");
    };
    *src
}

/// Every frame time of the case at 30 fps.
fn frames(case: &scenes::Case) -> impl Iterator<Item = u32> {
    (0..case.length * 30 / 1000).map(|n| n * 1000 / 30)
}

#[test]
fn a_snap_holds_over_its_anchors_and_lets_go_in_300_ms_with_motion_off() {
    let case = case(ZoomMotion::Snap, false);
    let (start, end) = span(&case);
    let scene = scenes::build(&case, 540);
    let crop = src(&scene, 0);
    assert_eq!(crop.w, 1080.0, "the whole portrait frame");
    for t in frames(&case) {
        let now = src(&scene, t);
        if t < start || t >= end + ZOOM_RELEASE_MS {
            assert_eq!(now, crop, "{t}: no zoom outside it");
        } else if t < end {
            assert!((now.w - crop.w / STRONG).abs() < 1e-3, "{t}: {now:?}");
        } else {
            assert!(now.w > crop.w / STRONG && now.w < crop.w, "{t}: releasing");
        }
        // About the centre.
        assert!((now.x + now.w / 2.0 - (crop.x + crop.w / 2.0)).abs() < 1e-3);
        assert!((now.y + now.h / 2.0 - (crop.y + crop.h / 2.0)).abs() < 1e-3);
    }
}

#[test]
fn a_push_rises_across_its_anchors() {
    let case = case(ZoomMotion::Push, false);
    let (start, end) = span(&case);
    let scene = scenes::build(&case, 540);
    let crop = src(&scene, 0);
    assert_eq!(src(&scene, start), crop);
    let mut before = crop.w;
    for t in (start..end).step_by(33) {
        let now = src(&scene, t).w;
        assert!(now <= before, "{t}: the crop shrinks as the zoom rises");
        before = now;
    }
    assert!((src(&scene, end).w - crop.w / STRONG).abs() < 1e-3);
}

#[test]
fn the_zoom_changes_only_the_videos_src_and_switched_off_it_is_gone() {
    let on = case(ZoomMotion::Snap, false);
    let mut off = case(ZoomMotion::Snap, false);
    off.edit.event_overrides.insert(EventId::new(ID), false);
    let (with, without) = (scenes::build(&on, 540), scenes::build(&off, 540));
    let crop = src(&without, 0);
    for t in frames(&on) {
        let (a, b) = (
            with.frame_at(OutMs::new(t)),
            without.frame_at(OutMs::new(t)),
        );
        assert_eq!(a.cmds[1..], b.cmds[1..], "{t}: only the video may differ");
        assert_eq!(src(&without, t), crop, "{t}: switched off, no zoom");
    }
    // The zoom is a visual moment of "What we changed"; switched off, it is not.
    assert_eq!(with.summary().visual_moments, 1);
    assert_eq!(without.summary().visual_moments, 0);
}

#[test]
fn with_motion_on_the_zoom_multiplies_the_drift() {
    let case = case(ZoomMotion::Snap, true);
    let (start, end) = span(&case);
    let scene = scenes::build(&case, 540);
    let map = TimeMap::identity(DurMs::new(case.length));
    let drift = for_clip(&case.transcript, &map);
    let crop = CropRect {
        x: 0.0,
        y: 0.0,
        w: 1080.0,
        h: 1920.0,
    };
    for t in frames(&case) {
        let at = OutMs::new(t);
        let zoom = if t >= start && t < end { STRONG } else { 1.0 };
        let now = src(&scene, t);
        if t < start || t >= end + ZOOM_RELEASE_MS || t < end {
            let want = drift.scale_at(at) * zoom;
            assert!(
                (now.w - crop.w / want).abs() < 1e-2,
                "{t}: {now:?} for {want}"
            );
        }
        assert!(now.w >= crop.w / 1.35 - 1e-3, "{t}: past the cap");
    }
}
