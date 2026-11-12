//! The hook title in a scene (v6.2implementation D-484, D-453; TS §47.3): a
//! model's plan with `extras.hook`, in the four styles. Over the program's
//! first 2 s the frame is the video and the hook alone: no caption is drawn
//! or hit; from 2 s every frame is the frame without the hook. A hook that
//! cannot fit, or one switched off, changes nothing and is not counted; a
//! clip shorter than 2 s shows it throughout.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

#[path = "common/hook.rs"]
mod hook;
#[path = "common/scenes.rs"]
mod scenes;

use std::collections::BTreeMap;

use offcut_scene::events::hook_title::{self, HOOK_DUR_MS};
use offcut_scene::{FontSet, Hit, Scene, Theme};
use offcut_types::{EventId, OutMs, StyleId, WordIdx};

const STYLES: [StyleId; 4] = [StyleId::Clean, StyleId::Bold, StyleId::Tech, StyleId::Glow];
const SENTENCES: [&str; 2] = [
    "The first week felt brutal for everyone.",
    "Thanks for watching, and see you next week.",
];
const HOOK: &str = "The one thing nobody tells you";

fn plain(style: StyleId, sentences: &[&str]) -> scenes::Case {
    scenes::written("hook", sentences, BTreeMap::new(), |edit, _| {
        edit.style = style;
    })
}

fn hooked(style: StyleId, sentences: &[&str], (text, accent): (&str, u8)) -> scenes::Case {
    let case = hook::with_hook(plain(style, sentences), text, accent, Vec::new());
    assert_eq!(case.plan.items().len(), 1, "{}: {text:?} passed", case.name);
    case
}

/// Every frame time of a scene at 30 fps, and its last millisecond.
fn frames(scene: &Scene) -> impl Iterator<Item = u32> {
    let end = scene.duration().get();
    (0..end * 30 / 1000).map(|n| n * 1000 / 30).chain([end - 1])
}

#[test]
fn for_2_s_the_frame_is_the_video_and_the_hook_then_it_is_the_frame_without_it() {
    for style in STYLES {
        for width in [1080, 540] {
            let with = hooked(style, &SENTENCES, (HOOK, 3));
            let none = plain(style, &SENTENCES);
            let (a, b) = (scenes::build(&with, width), scenes::build(&none, width));
            let track = hook_title::layout(
                HOOK,
                3,
                OutMs::new(with.length),
                &Theme::default_for(style),
                &FontSet::embedded(),
            )
            .unwrap()
            .unwrap();
            for t in frames(&a) {
                let at = OutMs::new(t);
                let (x, y) = (a.frame_at(at), b.frame_at(at));
                if t < HOOK_DUR_MS {
                    // The video, then the hook as `hook_title` draws it, scaled
                    // to the canvas: no caption, no other item.
                    let mut want = vec![y.cmds[0].clone()];
                    let mut drawn = Vec::new();
                    hook_title::draw(&track, at, &mut drawn);
                    for cmd in &mut drawn {
                        cmd.scale(width as f32 / 1080.0);
                    }
                    want.extend(drawn);
                    assert_eq!(x.cmds, want, "{style:?} {width} at {t}");
                    assert!(a.caption_words(at).is_empty(), "{style:?} at {t}");
                } else {
                    assert_eq!(x, y, "{style:?} {width} at {t}: as without");
                }
            }
        }
    }
}

#[test]
fn under_the_hook_no_caption_word_is_hit_or_boxed() {
    for style in STYLES {
        let with = scenes::build(&hooked(style, &SENTENCES, (HOOK, 3)), 1080);
        let none = scenes::build(&plain(style, &SENTENCES), 1080);
        let mut words = 0;
        for t in (0..HOOK_DUR_MS).step_by(10) {
            let at = OutMs::new(t);
            for w in 0..8 {
                if let Some(r) = none.word_box(at, WordIdx::new(w)) {
                    words += 1;
                    assert_eq!(with.word_box(at, WordIdx::new(w)), None, "{t}");
                    let hit = with.hit(at, r.x + r.w / 2.0, r.y + r.h / 2.0);
                    assert!(!matches!(hit, Some(Hit::Word(_))), "{style:?} {t}: {hit:?}");
                }
            }
        }
        assert!(words > 50, "{style:?}: {words}");
    }
}

#[test]
fn a_hook_that_cannot_fit_or_switched_off_changes_nothing_and_is_not_counted() {
    for style in STYLES {
        let none = scenes::build(&plain(style, &SENTENCES), 1080);
        // 39 characters, 2 words: valid, but its first word is too wide at 48 lp.
        let wide = hooked(
            style,
            &SENTENCES,
            ("Supercalifragilisticexpialidocious, wow", 1),
        );
        let mut off = hooked(style, &SENTENCES, (HOOK, 3));
        off.edit
            .event_overrides
            .insert(EventId::new(hook::HOOK_ID), false);
        let (wide, off) = (scenes::build(&wide, 1080), scenes::build(&off, 1080));
        for t in frames(&none) {
            let at = OutMs::new(t);
            assert_eq!(wide.frame_at(at), none.frame_at(at), "{style:?} at {t}");
            assert_eq!(off.frame_at(at), none.frame_at(at), "{style:?} at {t}");
        }
        assert_eq!(wide.summary().visual_moments, 0);
        assert_eq!(off.summary().visual_moments, 0);
        let on = scenes::build(&hooked(style, &SENTENCES, (HOOK, 3)), 1080).summary();
        assert_eq!((on.captions_emphasized, on.visual_moments), (0, 1));
    }
}

#[test]
fn a_clip_shorter_than_2_s_shows_the_hook_throughout() {
    for style in STYLES {
        let case = hooked(style, &["Go now."], ("Stop scrolling", 1));
        assert!(case.length < HOOK_DUR_MS, "{}", case.length);
        let scene = scenes::build(&case, 1080);
        for t in frames(&scene) {
            let at = OutMs::new(t);
            assert!(scene.caption_words(at).is_empty(), "{style:?} at {t}");
            let runs = scene
                .frame_at(at)
                .cmds
                .iter()
                .filter(|cmd| matches!(cmd, offcut_scene::DrawCmd::GlyphRun { .. }))
                .count();
            assert!(runs >= 4, "{style:?} at {t}: the hook and its shadow");
        }
    }
}
