//! The shaping cache changes no output (D-173, INV-7): each of six scenes is
//! built with the cache cold (emptied), warm (just filled by the same build)
//! and turned off, and the three scenes and every frame they draw are equal.
//! Prints each build's time: `cargo test -p offcut-scene --release --test
//! scene_cache -- --nocapture`.

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

use std::time::{Duration, Instant};

use offcut_scene::layout::set_shape_cache;
use offcut_scene::{DisplayList, Scene};
use offcut_types::FrameIdx;

/// The one clock of this crate's tests: each build's time, printed, not
/// asserted (D-173; read it from a release build). The crate itself reads no
/// clock (TS §7).
#[allow(clippy::disallowed_methods)]
fn timed(build: impl FnOnce() -> Scene) -> (Scene, Duration) {
    let start = Instant::now();
    let scene = build();
    (scene, start.elapsed())
}

fn frames(scene: &Scene) -> Vec<DisplayList> {
    (0..scene.frame_count())
        .map(|frame| scene.frame_at(FrameIdx::new(frame).to_out_ms()))
        .collect()
}

#[test]
fn six_scenes_are_the_same_cold_warm_and_without_the_cache() {
    let ms = |d: Duration| d.as_secs_f64() * 1_000.0;
    let (mut cold_total, mut warm_total) = (0.0, 0.0);
    for case in labeled::six() {
        set_shape_cache(true);
        let (cold, cold_time) = timed(|| scenes::build(&case, 1080));
        let (warm, warm_time) = timed(|| scenes::build(&case, 1080));
        set_shape_cache(false);
        let (off, off_time) = timed(|| scenes::build(&case, 1080));
        set_shape_cache(true);

        assert_eq!(cold, warm, "{}", case.name);
        assert_eq!(cold, off, "{}", case.name);
        let (cold, warm, off) = (frames(&cold), frames(&warm), frames(&off));
        assert!(cold.len() > 300, "{}: {} frames", case.name, cold.len());
        assert_eq!(cold, warm, "{}", case.name);
        assert_eq!(cold, off, "{}", case.name);
        println!(
            "{}: {} frames; build cold {:.1} ms, warm {:.1} ms, without the cache {:.1} ms",
            case.name,
            cold.len(),
            ms(cold_time),
            ms(warm_time),
            ms(off_time)
        );
        cold_total += ms(cold_time);
        warm_total += ms(warm_time);
    }
    println!("six scenes: cold {cold_total:.1} ms, warm {warm_total:.1} ms");
}
