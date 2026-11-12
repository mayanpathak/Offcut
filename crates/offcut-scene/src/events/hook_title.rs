//! HookTitle: the plan's hook over the clip's first 2 s, as a lockup that
//! fills the width (v6.2implementation D-484, TS §47.3). It is laid out in
//! the safe area above the watermark's band (the catalog's zone "hook"),
//! each line centred, in the theme's text colour with the accent word in
//! the accent, over a soft drop shadow grown with the size. It enters at
//! 0.9 of its size and reaches all of it in 150 ms on `OUT`, and is shown
//! over `[0, HOOK_DUR_MS)` of program time, or the whole of a shorter clip;
//! the captions of that time are not drawn (`Scene::frame_at`). A hook that
//! cannot fit has no track: it is not drawn and not counted.

use offcut_types::OutMs;

use crate::SceneError;
use crate::display_list::{Affine, DrawCmd, Rect, Rgba};
use crate::easing::{OUT, bezier};
use crate::fonts::FontSet;
use crate::layout::ShapedRun;
use crate::lockup::lay_out;
use crate::placement::WATERMARK_BAND;
use crate::safe_area::SAFE_AREA;
use crate::theme::{Role, Theme};

/// How long the hook is shown from the start of the program (D-484).
pub const HOOK_DUR_MS: u32 = 2_000;
/// Its entry: from this share of its size to all of it, over this long.
pub const HOOK_ENTER_FROM: f32 = 0.9;
pub const HOOK_ENTER_MS: u32 = 150;
/// The drop shadow's fall, per lp of a word's size: Clean's 4 lp at its
/// 72 lp (D-170), at 45% (assumption, read at CP 3).
pub const HOOK_SHADOW_EM: f32 = 4.0 / 72.0;
pub const HOOK_SHADOW_OPACITY: f32 = 0.45;

/// The room the lockup is laid out and centred in: the safe area above the
/// watermark's band.
pub fn room() -> Rect {
    Rect {
        h: SAFE_AREA.h - WATERMARK_BAND,
        ..SAFE_AREA
    }
}

/// A run of the lockup on the canvas: where its baseline starts, its fill,
/// and how far its shadow falls.
#[derive(Clone, Debug, PartialEq)]
pub struct Piece {
    pub run: ShapedRun,
    pub x: f32,
    pub baseline: f32,
    pub fill: Rgba,
    pub drop: f32,
}

/// The lockup placed, the point it enters about, and when it ends.
#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    pub pieces: Vec<Piece>,
    pub shadow: Rgba,
    pub centre: (f32, f32),
    pub until: OutMs,
}

/// The hook `text`, its word `accent` the accent, over the first
/// `HOOK_DUR_MS` of a program `duration` long; `None` when its lockup
/// cannot fit `room()`.
pub fn layout(
    text: &str,
    accent: u8,
    duration: OutMs,
    theme: &Theme,
    fonts: &FontSet,
) -> Result<Option<Track>, SceneError> {
    let room = room();
    let Some(lockup) = lay_out(text, usize::from(accent), (room.w, room.h), fonts)? else {
        return Ok(None);
    };
    let top = room.y + (room.h - lockup.height) / 2.0;
    let mut pieces = Vec::new();
    for line in &lockup.lines {
        let left = room.x + (room.w - line.width) / 2.0;
        for word in &line.words {
            let fill = theme.color(if word.accent {
                Role::Accent
            } else {
                Role::Text
            });
            let mut x = left + word.x;
            for run in &word.runs {
                pieces.push(Piece {
                    run: run.clone(),
                    x,
                    baseline: top + line.baseline,
                    fill,
                    drop: HOOK_SHADOW_EM * word.size,
                });
                x += run.width;
            }
        }
    }
    Ok(Some(Track {
        pieces,
        shadow: theme.color(Role::Shadow),
        centre: (room.x + room.w / 2.0, top + lockup.height / 2.0),
        until: OutMs::new(duration.get().min(HOOK_DUR_MS)),
    }))
}

/// The lockup's scale `since_ms` after the program starts.
pub fn enter_scale(since_ms: u32) -> f32 {
    let share = (since_ms as f32 / HOOK_ENTER_MS as f32).min(1.0);
    HOOK_ENTER_FROM + (1.0 - HOOK_ENTER_FROM) * bezier(OUT, share)
}

/// The shadow, in one layer, then the lockup, while it is shown.
pub fn draw(track: &Track, t: OutMs, out: &mut Vec<DrawCmd>) {
    if t >= track.until {
        return;
    }
    let k = enter_scale(t.get());
    let (cx, cy) = track.centre;
    let at = |piece: &Piece, fall: f32| {
        let x = cx + (piece.x - cx) * k;
        let y = cy + (piece.baseline + fall - cy) * k;
        Affine::scale_then_translate(k, x, y)
    };
    let run = |piece: &Piece, fill: Rgba, transform: Affine| DrawCmd::GlyphRun {
        font: piece.run.font,
        size: piece.run.size,
        glyphs: piece.run.glyphs.clone(),
        fill,
        stroke: None,
        transform,
        text: piece.run.text.clone(),
    };
    out.push(DrawCmd::PushLayer {
        opacity: HOOK_SHADOW_OPACITY,
        clip: None,
    });
    for piece in &track.pieces {
        out.push(run(piece, track.shadow, at(piece, piece.drop)));
    }
    out.push(DrawCmd::PopLayer);
    for piece in &track.pieces {
        out.push(run(piece, piece.fill, at(piece, 0.0)));
    }
}

#[cfg(test)]
mod tests {
    use offcut_types::StyleId;

    use super::*;

    fn track(text: &str, accent: u8, duration: u32) -> Option<Track> {
        let theme = Theme::default_for(StyleId::Clean);
        layout(
            text,
            accent,
            OutMs::new(duration),
            &theme,
            &FontSet::embedded(),
        )
        .unwrap()
    }

    #[test]
    fn the_hook_is_centred_in_the_room_in_its_colours_for_2_s_or_a_shorter_clip() {
        let hook = track("The one thing nobody tells you", 3, 30_000).unwrap();
        let theme = Theme::default_for(StyleId::Clean);
        assert_eq!(hook.until, OutMs::new(HOOK_DUR_MS));
        assert_eq!(
            track("The one thing nobody tells you", 3, 1_500)
                .unwrap()
                .until,
            OutMs::new(1_500)
        );
        // The accent word, "NOBODY", in the accent; the rest in the text colour.
        for piece in &hook.pieces {
            let want = if piece.run.text == "NOBODY" {
                theme.accent
            } else {
                theme.text
            };
            assert_eq!(piece.fill, want, "{}", piece.run.text);
        }
        let room = room();
        assert!((hook.centre.0 - (room.x + room.w / 2.0)).abs() < 1e-3);
        assert!((hook.centre.1 - (room.y + room.h / 2.0)).abs() < 1e-3);
    }

    #[test]
    fn it_enters_from_0_9_on_out_with_its_shadow_first_and_is_gone_at_its_end() {
        assert_eq!(enter_scale(0), HOOK_ENTER_FROM);
        assert!((enter_scale(HOOK_ENTER_MS) - 1.0).abs() < 1e-6);
        assert_eq!(enter_scale(1_900), 1.0);
        let mut was = enter_scale(0);
        for t in 1..=HOOK_ENTER_MS {
            assert!(enter_scale(t) >= was, "{t}");
            was = enter_scale(t);
        }
        let hook = track("Stop scrolling right now", 0, 30_000).unwrap();
        let n = hook.pieces.len();
        let mut out = Vec::new();
        draw(&hook, OutMs::new(500), &mut out);
        assert_eq!(out.len(), 2 * n + 2);
        assert_eq!(
            out[0],
            DrawCmd::PushLayer {
                opacity: HOOK_SHADOW_OPACITY,
                clip: None
            }
        );
        assert_eq!(out[n + 1], DrawCmd::PopLayer);
        // At rest, each run where the lockup put it; its shadow below it.
        if let (
            DrawCmd::GlyphRun {
                transform: s,
                fill: sf,
                ..
            },
            DrawCmd::GlyphRun { transform: w, .. },
        ) = (&out[1], &out[n + 2])
        {
            let piece = &hook.pieces[0];
            assert_eq!(w.m, [1.0, 0.0, 0.0, 1.0, piece.x, piece.baseline]);
            assert_eq!(s.m[5], piece.baseline + piece.drop);
            assert_eq!(*sf, hook.shadow);
        } else {
            panic!("{out:?}");
        }
        for t in [HOOK_DUR_MS, 9_000] {
            let mut none = Vec::new();
            draw(&hook, OutMs::new(t), &mut none);
            assert!(none.is_empty(), "{t}");
        }
    }

    #[test]
    fn a_hook_that_cannot_fit_has_no_track() {
        // One word is no lockup; nor is a word too wide at 48 lp.
        assert!(track("Hello", 0, 30_000).is_none());
        assert!(
            track(
                "Pneumonoultramicroscopicsilicovolcanoconiosis now",
                1,
                30_000
            )
            .is_none()
        );
    }
}
