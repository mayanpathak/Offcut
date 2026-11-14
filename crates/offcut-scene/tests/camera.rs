//! The camera drift (v6.2implementation D-476, D-477, §10.1; TS §47.2): its
//! segments from the changes and the sentences, the alternation, the held
//! short segment, and in a scene: with `Enhance.motion` off every frame is
//! today's, and on, only the video's `src` differs (DC-16).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

#[path = "common/labeled.rs"]
mod labeled;
#[path = "common/scenes.rs"]
mod scenes;

use offcut_scene::DrawCmd;
use offcut_scene::camera::{CameraTrack, DRIFT, MAX_SEG_MS, MIN_SEG_MS, build};
use offcut_scene::easing::{STANDARD, bezier};
use offcut_types::{OutMs, OutSpan};

fn ms(v: u32) -> OutMs {
    OutMs::new(v)
}

/// Sentences starting at `starts`, each running to the next (the last one second long).
fn sentences(starts: &[u32]) -> Vec<OutSpan> {
    starts
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let end = starts.get(i + 1).copied().unwrap_or(s + 1_000);
            OutSpan::new(ms(*s), ms(end)).unwrap()
        })
        .collect()
}

/// The segments' bounds: each start, then the last end.
fn bounds(track: &CameraTrack) -> Vec<u32> {
    let segs = track.segs();
    let mut out: Vec<u32> = segs.iter().map(|s| s.start.get()).collect();
    out.extend(segs.last().map(|s| s.end.get()));
    out
}

#[test]
fn none_is_one_at_every_time() {
    let none = CameraTrack::none();
    assert!(none.segs().is_empty());
    for t in [0, 1, 1_999, 60_000, u32::MAX] {
        assert_eq!(none.scale_at(ms(t)), 1.0);
    }
}

#[test]
fn a_segment_up_to_4_s_is_not_split() {
    let track = build(&[], &sentences(&[0, 1_000, 2_000, 3_000]), ms(MAX_SEG_MS));
    assert_eq!(bounds(&track), [0, MAX_SEG_MS]);
}

#[test]
fn a_longer_one_is_split_at_the_sentence_start_nearest_its_middle_until_short() {
    // [0, 10 s): middle 5,000, nearest start 4,800. [0, 4.8 s): middle
    // 2,400, nearest 2,000. [4.8 s, 10 s): middle 7,400, nearest 7,000.
    let track = build(&[], &sentences(&[0, 2_000, 4_800, 7_000]), ms(10_000));
    assert_eq!(bounds(&track), [0, 2_000, 4_800, 7_000, 10_000]);
    // Of two starts as near the middle, the earlier.
    let tie = build(&[], &sentences(&[0, 2_000, 4_000]), ms(6_000));
    assert_eq!(bounds(&tie), [0, 2_000, 6_000]);
}

#[test]
fn a_long_stretch_with_no_sentence_start_inside_stays_whole() {
    let track = build(&[], &sentences(&[0]), ms(9_000));
    assert_eq!(bounds(&track), [0, 9_000]);
    let seg = track.segs()[0];
    assert_eq!((seg.from, seg.to), (1.0, 1.0 + DRIFT));
}

#[test]
fn a_change_is_always_a_bound_and_one_outside_the_program_is_ignored() {
    let track = build(&[ms(0), ms(3_000), ms(7_000), ms(9_000)], &[], ms(7_000));
    assert_eq!(bounds(&track), [0, 3_000, 7_000]);
}

#[test]
fn segments_alternate_in_and_out_and_join_without_a_jump() {
    let starts: Vec<u32> = (0..20).map(|i| i * 3_100).collect();
    let track = build(&[], &sentences(&starts), ms(62_000));
    let segs = track.segs();
    assert!(segs.len() > 10, "{}", segs.len());
    assert_eq!(segs[0].start, ms(0));
    assert_eq!(segs[0].from, 1.0);
    let mut moving_up = Vec::new();
    for (i, seg) in segs.iter().enumerate() {
        assert!(seg.end > seg.start);
        assert!(seg.end.get() - seg.start.get() <= MAX_SEG_MS, "{seg:?}");
        for scale in [seg.from, seg.to] {
            assert!(scale == 1.0 || scale == 1.0 + DRIFT, "{seg:?}");
        }
        if i > 0 {
            assert_eq!(seg.start, segs[i - 1].end, "segments end to end");
            assert_eq!(seg.from, segs[i - 1].to, "no jump at {seg:?}");
        }
        if seg.from != seg.to {
            moving_up.push(seg.to > seg.from);
        }
    }
    assert!(moving_up.windows(2).all(|w| w[0] != w[1]), "{moving_up:?}");
}

#[test]
fn a_segment_under_0_8_s_holds_where_the_last_one_ended() {
    // [0, 4.5 s) splits at 4,000: a 4 s drift up, then 500 ms held at the top.
    let track = build(&[], &sentences(&[0, 4_000]), ms(4_500));
    let segs = track.segs();
    assert_eq!(bounds(&track), [0, 4_000, 4_500]);
    assert!(segs[1].end.get() - segs[1].start.get() < MIN_SEG_MS);
    assert_eq!((segs[0].from, segs[0].to), (1.0, 1.0 + DRIFT));
    assert_eq!((segs[1].from, segs[1].to), (1.0 + DRIFT, 1.0 + DRIFT));
    for t in [4_000, 4_250, 4_499] {
        assert_eq!(track.scale_at(ms(t)), 1.0 + DRIFT);
    }
}

#[test]
fn the_scale_follows_the_standard_curve_and_stays_in_range() {
    let track = build(&[], &[], ms(3_000));
    let seg = track.segs()[0];
    assert_eq!(track.scale_at(ms(0)), seg.from);
    let half = track.scale_at(ms(1_500));
    let want = seg.from + (seg.to - seg.from) * bezier(STANDARD, 0.5);
    assert!((half - want).abs() < 1e-6, "{half} {want}");
    let mut before = track.scale_at(ms(0));
    for t in (0..3_000).step_by(33) {
        let now = track.scale_at(ms(t));
        assert!((1.0..=1.0 + DRIFT).contains(&now), "{t}: {now}");
        assert!(now >= before, "an upward drift falls at {t}");
        before = now;
    }
    // Past the program, nothing.
    assert_eq!(track.scale_at(ms(3_000)), 1.0);
}

#[test]
fn the_same_input_gives_the_same_track() {
    let input = sentences(&[0, 1_700, 5_200, 9_900, 14_000]);
    assert_eq!(
        build(&[ms(0)], &input, ms(20_000)),
        build(&[ms(0)], &input, ms(20_000))
    );
}

/// The labeled set's six cases (three styles, the band's ends, a word edit,
/// a visual turned off), each with `motion` off as stored and then on: every
/// frame of each, at 30 fps.
#[test]
fn motion_off_is_todays_scene_and_on_changes_only_the_videos_src() {
    for mut case in labeled::six() {
        // Today's choices leave `enhance` at v5(), so motion off.
        assert!(!case.edit.enhance.motion, "{}", case.name);
        let off = scenes::build(&case, 540);
        case.edit.enhance.motion = true;
        let on = scenes::build(&case, 540);
        compare(&case, &off, &on);
    }
}

fn compare(case: &scenes::Case, off: &offcut_scene::Scene, on: &offcut_scene::Scene) {
    let frames = off.frame_count();
    assert!(frames > 150, "{}: {frames}", case.name);
    let (mut moved, mut first_src) = (0, None);
    for n in 0..frames {
        let t = ms(n * 1_000 / 30);
        let (a, b) = (off.frame_at(t), on.frame_at(t));
        assert_eq!(a.cmds.len(), b.cmds.len());
        assert_eq!(
            a.cmds[1..],
            b.cmds[1..],
            "{}: only the video may differ at {t:?}",
            case.name
        );
        let (
            DrawCmd::Video {
                src: sa, dst: da, ..
            },
            DrawCmd::Video {
                src: sb, dst: db, ..
            },
        ) = (&a.cmds[0], &b.cmds[0])
        else {
            panic!("the video first");
        };
        assert_eq!(da, db);
        // Off: the same crop at every frame.
        assert_eq!(*first_src.get_or_insert(*sa), *sa);
        // On: inside the crop, and smaller by at most the drift.
        assert!(sb.x >= sa.x && sb.y >= sa.y, "{t:?}");
        assert!(sb.x + sb.w <= sa.x + sa.w + 1e-3 && sb.y + sb.h <= sa.y + sa.h + 1e-3);
        assert!(sb.w >= sa.w / (1.0 + DRIFT) - 1e-3, "{t:?}");
        if sb != sa {
            moved += 1;
        }
    }
    assert!(moved > frames / 2, "{moved} of {frames} frames drift");
}
