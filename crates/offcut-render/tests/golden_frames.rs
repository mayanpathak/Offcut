//! Golden frames (v4implementation D-134, D-135, §9.4): display lists drawn
//! by the renderer on a software adapter, from a synthetic source, compared
//! with the committed images in `fixtures/golden/` by SSIM. A golden's source
//! is drawn here, never read from a recording.
//!
//! V4 Prompt 101 writes the two goldens whose list is one `Video` command and
//! nothing else, which the look of Prompts 105-107 cannot change; Prompt 107
//! adds the other fourteen. `OFFCUT_GOLDEN_UPDATE=1` writes the images (CI
//! never sets it); a golden that differs writes what was drawn to
//! `target/golden-out/`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

#[path = "common/ssim.rs"]
mod ssim;

use std::fs::File;
use std::future::Future;
use std::io::BufWriter;
use std::path::PathBuf;
use std::pin::pin;
use std::task::{Context, Poll, Waker};

use offcut_render::scene::{CropRect, DisplayList, DrawCmd, FontSet, Rect, VideoSlot};
use offcut_render::{Renderer, RgbaImage};
use offcut_types::{Px, Rotation};

/// The least similarity a frame may have to its golden (assumption, M1.4).
const GOLDEN_SSIM_MIN: f64 = 0.98;
/// The preview's size: every golden is drawn at it (D-135).
const WIDTH: u32 = 540;
const HEIGHT: u32 = 960;

/// Runs a future to its end on this thread. wgpu's native futures are ready
/// at once or after a poll; nothing here needs a runtime.
fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    let mut cx = Context::from_waker(Waker::noop());
    loop {
        if let Poll::Ready(value) = future.as_mut().poll(&mut cx) {
            return value;
        }
        std::thread::yield_now();
    }
}

/// The synthetic source (§9.4): a diagonal gradient, an 8 x 8 grid of lines
/// two pixels wide, and a filled disc at a third of the width.
fn pattern(width: u32, height: u32) -> RgbaImage {
    let (cell_w, cell_h) = (width / 8, height / 8);
    let (cx, cy) = (f64::from(width) / 3.0, f64::from(height) / 2.0);
    let radius = f64::from(width.min(height)) / 6.0;
    let mut data = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            let t = (f64::from(x) / f64::from(width) + f64::from(y) / f64::from(height)) / 2.0;
            let mut pixel = [(255.0 * t) as u8, (255.0 * (1.0 - t)) as u8, 128, 255];
            if x % cell_w < 2 || y % cell_h < 2 {
                pixel = [240, 240, 240, 255];
            }
            let (dx, dy) = (f64::from(x) + 0.5 - cx, f64::from(y) + 0.5 - cy);
            if dx * dx + dy * dy <= radius * radius {
                pixel = [230, 57, 70, 255];
            }
            data.extend_from_slice(&pixel);
        }
    }
    RgbaImage {
        width,
        height,
        data,
    }
}

/// A list of one `Video` command: `src` of the source on the whole canvas.
fn video_only(src: CropRect) -> DisplayList {
    DisplayList {
        cmds: vec![DrawCmd::Video {
            slot: VideoSlot::Main,
            src,
            dst: Rect {
                x: 0.0,
                y: 0.0,
                w: WIDTH as f32,
                h: HEIGHT as f32,
            },
            opacity: 1.0,
            mask: None,
        }],
    }
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn write_png(path: &PathBuf, image: &RgbaImage) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let file = BufWriter::new(File::create(path).unwrap());
    let mut encoder = png::Encoder::new(file, image.width, image.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .unwrap()
        .write_image_data(&image.data)
        .unwrap();
}

fn read_png(path: &PathBuf) -> Option<RgbaImage> {
    let file = std::io::BufReader::new(File::open(path).ok()?);
    let mut reader = png::Decoder::new(file).read_info().unwrap();
    let mut data = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut data).unwrap();
    assert_eq!(
        (info.color_type, info.bit_depth),
        (png::ColorType::Rgba, png::BitDepth::Eight),
        "{}",
        path.display()
    );
    data.truncate(info.buffer_size());
    Some(RgbaImage {
        width: info.width,
        height: info.height,
        data,
    })
}

/// Draws `list` over `source` on a software adapter and compares it with
/// the golden `name`, or writes the golden under `OFFCUT_GOLDEN_UPDATE=1`.
fn golden(name: &str, source: &RgbaImage, list: &DisplayList) {
    let mut renderer = block_on(Renderer::new_headless(Px::new(WIDTH), Px::new(HEIGHT)))
        .unwrap_or_else(|error| panic!("{name}: no software adapter: {error}"));
    let adapter = renderer.adapter_info();
    // A golden is a software adapter's: WARP or lavapipe (D-134).
    assert_eq!(
        adapter.device_type,
        wgpu::DeviceType::Cpu,
        "{name}: not a software adapter: {adapter:?}"
    );
    let drawn = renderer
        .render_to_image(source, Rotation::R0, list, &FontSet::embedded())
        .unwrap();
    assert_eq!((drawn.width, drawn.height), (WIDTH, HEIGHT));

    let path = root().join("fixtures/golden").join(format!("{name}.png"));
    if std::env::var("OFFCUT_GOLDEN_UPDATE").is_ok_and(|v| v == "1") {
        write_png(&path, &drawn);
        println!(
            "wrote {} on {} ({:?})",
            path.display(),
            adapter.name,
            adapter.backend
        );
        return;
    }
    let Some(want) = read_png(&path) else {
        panic!(
            "{name}: no {}: run with OFFCUT_GOLDEN_UPDATE=1 and review the image",
            path.display()
        );
    };
    let similarity = ssim::ssim(&want, &drawn);
    println!(
        "ssim {name} = {similarity:.4} on {} ({:?})",
        adapter.name, adapter.backend
    );
    if similarity < GOLDEN_SSIM_MIN {
        let out = root().join("target/golden-out").join(format!("{name}.png"));
        write_png(&out, &drawn);
        panic!(
            "{name}: ssim {similarity:.4} < {GOLDEN_SSIM_MIN} on {}; what was drawn is in {}",
            adapter.name,
            out.display()
        );
    }
}

/// A landscape clip at a crop offset of 0.5 (TS §19.3): the strip right of
/// the centre, `x = (1920 - 607.5) / 2 x 1.5`.
#[test]
fn crop_landscape_050() {
    let source = pattern(1920, 1080);
    let crop = CropRect {
        x: 984.375,
        y: 0.0,
        w: 607.5,
        h: 1080.0,
    };
    golden("crop_landscape_050", &source, &video_only(crop));
}

/// A punch-in of 1.1 on a portrait clip (D-135): the centre 1/1.1 of the frame.
#[test]
fn punch_in_110() {
    let source = pattern(1080, 1920);
    let (w, h) = (1080.0 / 1.1, 1920.0 / 1.1);
    let crop = CropRect {
        x: (1080.0 - w) / 2.0,
        y: (1920.0 - h) / 2.0,
        w,
        h,
    };
    golden("punch_in_110", &source, &video_only(crop));
}
