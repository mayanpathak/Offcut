//! Equal inputs draw byte-identical frames (INV-7, TS §19.6, §8.10): the six
//! scenes of the labeled set, each built twice, give equal scenes and, for
//! every frame, the same display list to the byte; and a frame is a function
//! of the scene and its time alone, asked in any order.

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

use offcut_scene::DisplayList;
use offcut_types::FrameIdx;

fn bytes(list: &DisplayList) -> Vec<u8> {
    serde_json::to_vec(list).unwrap()
}

#[test]
fn each_scene_built_twice_draws_the_same_bytes_at_every_frame() {
    let mut frames = 0;
    for case in labeled::six() {
        let (first, second) = (scenes::build(&case, 1080), scenes::build(&case, 1080));
        assert!(first == second, "{}: two builds differ", case.name);
        for frame in 0..first.frame_count() {
            let t = FrameIdx::new(frame).to_out_ms();
            let (a, b) = (bytes(&first.frame_at(t)), bytes(&second.frame_at(t)));
            assert!(a == b, "{} frame {frame}", case.name);
            frames += 1;
        }
    }
    assert!(frames > 5_000, "{frames}");
}

#[test]
fn a_frame_asked_out_of_order_is_the_frame_asked_in_order() {
    for case in labeled::six() {
        let scene = scenes::build(&case, 1080);
        let n = scene.frame_count();
        let at = |frame: u32| bytes(&scene.frame_at(FrameIdx::new(frame).to_out_ms()));
        let in_order: Vec<Vec<u8>> = (0..n).map(at).collect();
        // A stride that shares no factor with `n` visits every frame once,
        // jumping back and forth; then backwards.
        let stride = (7_919..).find(|s| gcd(*s, n) == 1).unwrap();
        for i in 0..n {
            let frame = (u64::from(i) * u64::from(stride) % u64::from(n)) as u32;
            assert!(
                at(frame) == in_order[frame as usize],
                "{} frame {frame}",
                case.name
            );
        }
        for frame in (0..n).rev() {
            assert!(
                at(frame) == in_order[frame as usize],
                "{} frame {frame}",
                case.name
            );
        }
    }
}

/// D-166's budget, measured natively in release, and ignored by default:
/// `cargo test -p offcut-scene --release --test determinism -- --ignored scene_timing --nocapture`.
/// The reference clip's labeled transcript (`a01-scriptA`), of which the 60 s
/// portrait fixture is the start, detected and validated as the pipeline
/// does; `build_scene` at most 1 s, `frame_at` at most 2 ms on average over
/// every frame. The other five scenes are printed beside it. The one clock
/// of this crate's tests; the crate itself reads no clock (TS §7).
#[test]
#[ignore = "a timing, release only: D-166"]
#[allow(clippy::disallowed_methods)]
fn scene_timing() {
    for case in labeled::six() {
        let started = std::time::Instant::now();
        let scene = scenes::build(&case, 1080);
        let build_ms = started.elapsed().as_secs_f64() * 1000.0;
        let frames = scene.frame_count();
        let started = std::time::Instant::now();
        for frame in 0..frames {
            std::hint::black_box(scene.frame_at(FrameIdx::new(frame).to_out_ms()));
        }
        let mean_ms = started.elapsed().as_secs_f64() * 1000.0 / f64::from(frames);
        println!(
            "scene_timing {}: build_scene {build_ms:.1} ms, frame_at mean {mean_ms:.3} ms over {frames} frames",
            case.name
        );
        if case.name == "a01-scriptA" {
            assert!(build_ms <= 1_000.0, "build_scene {build_ms:.1} ms");
            assert!(mean_ms <= 2.0, "frame_at mean {mean_ms:.3} ms");
        }
    }
}

fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 { a } else { gcd(b, a % b) }
}
