//! Golden frames (v4implementation D-134, D-135, §9.4): display lists drawn
//! by the renderer on a software adapter, from a synthetic source, compared
//! with the committed images in `fixtures/golden/` by SSIM. A golden's source
//! is drawn here, never read from a recording.
//!
//! Sixteen goldens: the two whose list is one `Video` command (V4 Prompt 101);
//! each style with each kind's visual, at the instant `offcut-scene`'s
//! snapshots call "during" (a caption word is spoken, so Clean's pill shows),
//! one of them on a source turned 90 degrees; Clean's NumberReveal with and
//! without the watermark (V4 Prompt 107). And the parity case (D-172): each
//! style at 1080, box-filtered to 540, against the same frame drawn at 540.
//! `OFFCUT_GOLDEN_UPDATE=1` writes the images (CI never sets it); a golden
//! that differs writes what was drawn to `target/golden-out/`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

#[path = "../../offcut-scene/tests/common/inputs.rs"]
mod inputs;
#[path = "../../offcut-scene/tests/common/scenes.rs"]
mod scenes;
#[path = "common/ssim.rs"]
mod ssim;

use std::collections::BTreeMap;

use std::fs::File;
use std::future::Future;
use std::io::BufWriter;
use std::path::PathBuf;
use std::pin::pin;
use std::task::{Context, Poll, Waker};

use offcut_render::scene::{
    CropRect, DisplayList, DrawCmd, FontSet, Rect, SceneInput, Theme, VideoSlot, build_scene,
};
use offcut_render::{Renderer, RgbaImage};
use offcut_types::{
    BitsPerSec, ClipInfo, DurMs, EventKind, ExportProfile, Hz, Orientation, OutMs, ProfileKind, Px,
    Rotation, StyleId, TimeMap,
};

use inputs::{INPUTS, Input, STYLES, case, style_name};

/// The least similarity a frame may have to its golden (assumption, M1.4),
const GOLDEN_SSIM_MIN: f64 = 0.98;
/// and a frame drawn at 1080 and box-filtered to 540 to the same frame drawn
/// at 540 (D-172). Never lowered (§21.1).
const PARITY_SSIM_MIN: f64 = 0.99;
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

/// A renderer of `width` x `height` on a software adapter: WARP or lavapipe
/// (D-134).
fn software(name: &str, width: u32, height: u32) -> Renderer {
    let renderer = block_on(Renderer::new_headless(Px::new(width), Px::new(height)))
        .unwrap_or_else(|error| panic!("{name}: no software adapter: {error}"));
    let adapter = renderer.adapter_info();
    assert_eq!(
        adapter.device_type,
        wgpu::DeviceType::Cpu,
        "{name}: not a software adapter: {adapter:?}"
    );
    renderer
}

/// Draws `list` over `source`, turned by `rotation`, on a software adapter
/// and compares it with the golden `name`, or writes the golden under
/// `OFFCUT_GOLDEN_UPDATE=1`.
fn golden(name: &str, (source, rotation): (&RgbaImage, Rotation), list: &DisplayList) {
    let mut renderer = software(name, WIDTH, HEIGHT);
    let adapter = renderer.adapter_info();
    let drawn = renderer
        .render_to_image(source, rotation, list, &FontSet::embedded())
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
    golden(
        "crop_landscape_050",
        (&source, Rotation::R0),
        &video_only(crop),
    );
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
    golden("punch_in_110", (&source, Rotation::R0), &video_only(crop));
}

/// A kind's short name in a golden's name: `<style>_<kind>.png` (D-135).
fn kind_name(input: &Input) -> &'static str {
    match input.kind {
        EventKind::NumberReveal => "number",
        EventKind::ListReveal => "list",
        EventKind::FromTo => "from_to",
        EventKind::KeywordPop => "keyword",
    }
}

/// The frame of `input` in `style` at its middle instant, at the preview's
/// size, with the watermark when `watermark`, over a portrait pattern.
fn scene_frame(style: StyleId, input: &Input, watermark: bool) -> (RgbaImage, DisplayList) {
    let case = case(input, style, BTreeMap::new());
    let scene = scenes::build_with(&case, WIDTH, watermark);
    let list = scene.frame_at(OutMs::new(input.at[1]));
    (pattern(1080, 1920), list)
}

/// `<style>_<kind>`: the frame inside the visual, a caption word spoken.
fn kind_golden(style: StyleId, input: &Input) {
    let (source, list) = scene_frame(style, input, false);
    let name = format!("{}_{}", style_name(style), kind_name(input));
    golden(&name, (&source, Rotation::R0), &list);
}

#[test]
fn clean_number() {
    kind_golden(StyleId::Clean, &INPUTS[0]);
}

#[test]
fn clean_list() {
    kind_golden(StyleId::Clean, &INPUTS[1]);
}

#[test]
fn clean_from_to() {
    kind_golden(StyleId::Clean, &INPUTS[2]);
}

#[test]
fn clean_keyword() {
    kind_golden(StyleId::Clean, &INPUTS[3]);
}

#[test]
fn bold_number() {
    kind_golden(StyleId::Bold, &INPUTS[0]);
}

#[test]
fn bold_list() {
    kind_golden(StyleId::Bold, &INPUTS[1]);
}

#[test]
fn bold_from_to() {
    kind_golden(StyleId::Bold, &INPUTS[2]);
}

#[test]
fn bold_keyword() {
    kind_golden(StyleId::Bold, &INPUTS[3]);
}

#[test]
fn tech_number() {
    kind_golden(StyleId::Tech, &INPUTS[0]);
}

/// Tech's list on a clip recorded 1280 x 720 and turned 90 degrees, shown
/// 720 x 1280 as `ok_rotated_90.mov` is (§18): the pattern drawn in its
/// coded orientation, turned by the video pass.
#[test]
fn tech_list() {
    let input = &INPUTS[1];
    let case = case(input, StyleId::Tech, BTreeMap::new());
    let turned = ClipInfo {
        duration: DurMs::new(case.length),
        display_width: Px::new(720),
        display_height: Px::new(1280),
        rotation: Rotation::R90,
        orientation: Orientation::Portrait,
        is_vfr: false,
        video_codec_string: "avc1.640028".to_owned(),
        audio_codec_string: "mp4a.40.2".to_owned(),
        audio_sample_rate: Hz::new(48_000),
        audio_channels: 2,
    };
    let scene = build_scene(SceneInput {
        transcript: &case.transcript,
        plan: &case.plan,
        edit: &case.edit,
        clip: &turned,
        profile: &ExportProfile {
            kind: ProfileKind::Creator,
            width: Px::new(WIDTH),
            height: Px::new(HEIGHT),
            watermark: false,
            video_bitrate: BitsPerSec::new(8_000_000),
            audio_bitrate: BitsPerSec::new(160_000),
        },
        time_map: &TimeMap::identity(DurMs::new(case.length)),
        theme: &Theme::default_for(StyleId::Tech),
        fonts: &FontSet::embedded(),
    })
    .unwrap();
    let list = scene.frame_at(OutMs::new(input.at[1]));
    // The whole turned frame, from its display size.
    let DrawCmd::Video { src, .. } = &list.cmds[0] else {
        panic!("{}: the video first", case.name);
    };
    assert_eq!((src.x, src.y, src.w, src.h), (0.0, 0.0, 720.0, 1_280.0));
    golden("tech_list", (&pattern(1280, 720), Rotation::R90), &list);
}

#[test]
fn tech_from_to() {
    kind_golden(StyleId::Tech, &INPUTS[2]);
}

#[test]
fn tech_keyword() {
    kind_golden(StyleId::Tech, &INPUTS[3]);
}

/// Clean's NumberReveal with the watermark of a Free export (INV-9).
#[test]
fn clean_number_watermark() {
    let (source, list) = scene_frame(StyleId::Clean, &INPUTS[0], true);
    golden("clean_number_watermark", (&source, Rotation::R0), &list);
}

/// The same frame without it: what a Creator export and the preview draw.
#[test]
fn clean_number_nowatermark() {
    let (source, list) = scene_frame(StyleId::Clean, &INPUTS[0], false);
    golden("clean_number_nowatermark", (&source, Rotation::R0), &list);
}

/// `image` at half its size: each output pixel the mean of a 2 x 2 block,
/// channel by channel, rounded.
fn box_filter(image: &RgbaImage) -> RgbaImage {
    let (width, height) = (image.width / 2, image.height / 2);
    let mut data = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            for channel in 0..4 {
                let at = |dx: u32, dy: u32| {
                    let i = (((2 * y + dy) * image.width + 2 * x + dx) * 4 + channel) as usize;
                    u32::from(image.data[i])
                };
                let sum = at(0, 0) + at(1, 0) + at(0, 1) + at(1, 1);
                data.push(((sum + 2) / 4) as u8);
            }
        }
    }
    RgbaImage {
        width,
        height,
        data,
    }
}

/// The preview is the export (D-172): each style's from-to frame, drawn at
/// 1080 x 1920 and box-filtered to 540 x 960, against the same frame drawn
/// at 540 x 960.
#[test]
fn parity_1080_box_filtered_is_540() {
    let input = &INPUTS[2];
    let source = pattern(1080, 1920);
    let mut full = software("parity", 1080, 1920);
    let mut half = software("parity", WIDTH, HEIGHT);
    let adapter = half.adapter_info();
    for style in STYLES {
        let case = case(input, style, BTreeMap::new());
        let at = OutMs::new(input.at[1]);
        let fonts = FontSet::embedded();
        let large = scenes::build(&case, 1080).frame_at(at);
        let small = scenes::build(&case, WIDTH).frame_at(at);
        let large = full
            .render_to_image(&source, Rotation::R0, &large, &fonts)
            .unwrap();
        let small = half
            .render_to_image(&source, Rotation::R0, &small, &fonts)
            .unwrap();
        let similarity = ssim::ssim(&box_filter(&large), &small);
        println!(
            "ssim parity {} {} = {similarity:.4} on {} ({:?})",
            style_name(style),
            case.name,
            adapter.name,
            adapter.backend
        );
        assert!(
            similarity >= PARITY_SSIM_MIN,
            "{} in {style:?}: ssim {similarity:.4} < {PARITY_SSIM_MIN}",
            case.name
        );
    }
}
