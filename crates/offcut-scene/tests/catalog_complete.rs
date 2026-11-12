//! INV-23 (D-123, §7.8): the kinds of `offcut-plan`'s catalog and the kinds
//! the scene draws are one set. Every catalog kind is laid out and drawn in
//! the zone the catalog names for it, which is a `ZoneId`; every kind the
//! scene can draw is in the catalog. That each kind also has a golden frame
//! is `golden_frames.rs`' (V4 Prompt 107).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use std::collections::BTreeSet;

use offcut_plan::ValidItem;
use offcut_plan::catalog::{CAPTION_ZONE, EVENT_ZONE, catalog};
use offcut_scene::events::{self, EventTrack};
use offcut_scene::placement::{SNAP_BOTTOM, SNAP_MIDDLE, SNAP_TOP};
use offcut_scene::safe_area::{ZoneId, zone};
use offcut_scene::styles::spec;
use offcut_scene::{DrawCmd, FontSet, PathEl, Rect, Theme};
use offcut_types::{
    CaptionPlacement, Confidence, DurMs, EventId, EventKind, EventParams, ListItem, OutMs, OutSpan,
    Quantity, Span, StyleId, TimeMap, TimeMs, Transcript, Unit, WordIdx, WordRange,
};
// V6.2 (P206, D-440 (9)): the params of the five kinds.
use offcut_types::{EmojiId, EmojiPlace, EmphasisSize, IconId, Treatment, ZoomMotion, ZoomScale};

/// Every kind there is. The match in `item` has no wildcard: a new kind
/// does not compile until it is given an item here.
const KINDS: [EventKind; 9] = [
    EventKind::NumberReveal,
    EventKind::ListReveal,
    EventKind::FromTo,
    EventKind::KeywordPop,
    // V6.2 (P206, D-440 (9)).
    EventKind::Emphasis,
    EventKind::IconPop,
    EventKind::PunchIn,
    EventKind::Emoji,
    EventKind::HookTitle,
];

fn quantity(value: f64, display: &str) -> Quantity {
    Quantity {
        value,
        unit: Unit::Minutes,
        display: display.to_owned(),
    }
}

/// One item of `kind`, spoken from 1,000 to 3,000 ms.
fn item(kind: EventKind) -> ValidItem {
    let (params, marks) = match kind {
        EventKind::NumberReveal => (
            EventParams::NumberReveal {
                value: quantity(90.0, "90 minutes"),
                label: None,
            },
            Vec::new(),
        ),
        EventKind::ListReveal => (
            EventParams::ListReveal {
                count: 2,
                header: "2 STEPS".to_owned(),
                items: (1u8..=2)
                    .map(|ordinal| ListItem {
                        ordinal,
                        text: "write it down".to_owned(),
                        at: TimeMs::new(1_000 * u32::from(ordinal) + 500),
                    })
                    .collect(),
            },
            Vec::new(),
        ),
        EventKind::FromTo => (
            EventParams::FromTo {
                from: quantity(90.0, "90 minutes"),
                to: quantity(10.0, "10 minutes"),
                label: None,
            },
            vec![TimeMs::new(2_000)],
        ),
        EventKind::KeywordPop => (
            EventParams::KeywordPop {
                word: WordIdx::new(0),
            },
            Vec::new(),
        ),
        // V6.2's kinds (D-446), drawn from P202 to P205 (P206).
        EventKind::Emphasis => (
            EventParams::Emphasis {
                word: WordIdx::new(0),
                treatment: Treatment::Colour,
                size: EmphasisSize::Normal,
            },
            Vec::new(),
        ),
        EventKind::IconPop => (
            EventParams::IconPop {
                word: WordIdx::new(0),
                icon: IconId::Lightbulb,
            },
            Vec::new(),
        ),
        EventKind::PunchIn => (
            EventParams::PunchIn {
                scale: ZoomScale::Mild,
                motion: ZoomMotion::Snap,
            },
            Vec::new(),
        ),
        EventKind::Emoji => (
            EventParams::Emoji {
                word: WordIdx::new(0),
                emoji: EmojiId::Fire,
                place: EmojiPlace::Under,
            },
            Vec::new(),
        ),
        EventKind::HookTitle => (
            EventParams::HookTitle {
                text: "The one thing nobody tells you".to_owned(),
                accent: 3,
            },
            Vec::new(),
        ),
    };
    ValidItem {
        id: EventId::new(1),
        kind,
        anchors: WordRange {
            start: WordIdx::new(0),
            end: WordIdx::new(1),
        },
        span: Span::new(TimeMs::new(1_000), TimeMs::new(3_000)).unwrap(),
        params,
        marks,
        confidence: Some(Confidence::new(1.0).unwrap()),
    }
}

/// The track of `kind`'s item in the event zone of `placement`.
fn track(kind: EventKind, placement: f32) -> Option<EventTrack> {
    let item = item(kind);
    let span = OutSpan::new(OutMs::new(1_000), OutMs::new(3_000)).unwrap();
    let event_zone = zone(ZoneId::Event, CaptionPlacement::new(placement).unwrap());
    let transcript = Transcript {
        words: Vec::new(),
        sentences: Vec::new(),
        numbers: Vec::new(),
        model_version: "catalog".to_owned(),
    };
    let (style, theme) = (spec(StyleId::Clean), Theme::default_for(StyleId::Clean));
    let map = TimeMap::identity(DurMs::new(10_000));
    let fonts = FontSet::embedded();
    events::layout(
        &item,
        (span, event_zone),
        &transcript,
        &map,
        &style,
        &theme,
        &fonts,
    )
    .unwrap()
}

/// The `ZoneId` a catalog zone name names: its name in lower case.
fn zone_id(name: &str) -> ZoneId {
    // V6.2 (D-448): and the zones "video" and "hook".
    let ids = [
        ZoneId::Event,
        ZoneId::Caption,
        ZoneId::Full,
        ZoneId::Video,
        ZoneId::Hook,
    ];
    let found = ids
        .into_iter()
        .find(|id| format!("{id:?}").to_lowercase() == name);
    found.unwrap_or_else(|| panic!("no ZoneId is named {name:?}"))
}

/// The points a command reaches: each glyph run's baseline start, each
/// rectangle's corners, each path's points.
fn points(cmd: &DrawCmd) -> Vec<(f32, f32)> {
    match cmd {
        DrawCmd::GlyphRun { transform, .. } => vec![(transform.m[4], transform.m[5])],
        DrawCmd::FillRect { rect, .. } => {
            vec![(rect.x, rect.y), (rect.x + rect.w, rect.y + rect.h)]
        }
        DrawCmd::FillPath { path, .. } => path
            .iter()
            .filter_map(|el| match *el {
                PathEl::MoveTo(x, y) | PathEl::LineTo(x, y) => Some((x, y)),
                PathEl::QuadTo(_, _, x, y) | PathEl::CurveTo(_, _, _, _, x, y) => Some((x, y)),
                PathEl::Close => None,
            })
            .collect(),
        // V6.2 (P206): an emoji's or an icon's rectangle.
        DrawCmd::Image { rect, .. } => {
            vec![(rect.x, rect.y), (rect.x + rect.w, rect.y + rect.h)]
        }
        DrawCmd::PushLayer { .. } | DrawCmd::PopLayer | DrawCmd::Video { .. } => Vec::new(),
    }
}

fn within(r: Rect, (x, y): (f32, f32)) -> bool {
    x >= r.x - 0.01 && x <= r.x + r.w + 0.01 && y >= r.y - 0.01 && y <= r.y + r.h + 0.01
}

#[test]
fn the_catalog_kinds_are_the_kinds_the_scene_draws() {
    let catalogued: BTreeSet<EventKind> = catalog().iter().map(|row| row.kind).collect();
    let drawn: BTreeSet<EventKind> = KINDS
        .into_iter()
        .filter_map(|kind| track(kind, SNAP_BOTTOM))
        .map(|track| track.kind())
        .collect();
    assert_eq!(catalogued, drawn);
    // Each item is laid out as its own kind.
    for kind in KINDS {
        assert_eq!(track(kind, SNAP_BOTTOM).map(|t| t.kind()), Some(kind));
    }
}

#[test]
fn the_catalogs_zone_names_are_zone_ids() {
    assert_eq!(zone_id(EVENT_ZONE), ZoneId::Event);
    assert_eq!(zone_id(CAPTION_ZONE), ZoneId::Caption);
    for row in catalog() {
        zone_id(row.zone);
    }
}

#[test]
fn each_kind_is_drawn_in_the_zone_the_catalog_names() {
    for row in catalog() {
        for placement in [SNAP_TOP, SNAP_MIDDLE, SNAP_BOTTOM] {
            let track = track(row.kind, placement).unwrap();
            let mut out = Vec::new();
            // Every step in and resting: before the hold ends. V6.2 (P206,
            // D-440 (9)): and at 1,500 ms, while the hook (the program's
            // first 2 s) and an icon (its span, here) are shown.
            events::draw(&track, OutMs::new(1_500), &mut out);
            events::draw(&track, OutMs::new(3_300), &mut out);
            let id = zone_id(row.zone);
            match id {
                // The captions draw it: nothing of its own. V6.2: the video
                // track draws a zoom (D-478): nothing of its own either.
                ZoneId::Caption | ZoneId::Video => assert!(out.is_empty(), "{:?}", row.kind),
                ZoneId::Event | ZoneId::Full | ZoneId::Hook => {
                    let area = zone(id, CaptionPlacement::new(placement).unwrap());
                    assert!(!out.is_empty(), "{:?}", row.kind);
                    for point in out.iter().flat_map(points) {
                        assert!(
                            within(area, point),
                            "{:?} at {placement}: {point:?} outside {area:?}",
                            row.kind
                        );
                    }
                }
            }
        }
    }
}
