//! Images drawn by the overlay (v6.2implementation D-472, §9.2), read as
//! pixels on a software adapter: a file at 1x is its own colours blended
//! over what is under it, at full and at half opacity; a white icon scaled
//! between pixels has soft edges and no dark fringe (known issue 9: the
//! bundle's transparent pixels are 0, 0, 0, 0); a key the renderer does not
//! hold is `Unsupported`, never a blank.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use std::future::Future;
use std::pin::pin;
use std::task::{Context, Poll, Waker};

use offcut_render::scene::{
    CropRect, DisplayList, DrawCmd, FontSet, ImageKey, ImageSet, Rect, Rgba, VideoSlot,
};
use offcut_render::{RenderError, Renderer, RgbaImage};
use offcut_types::{EmojiId, IconId, Px, Rotation};

const WIDTH: u32 = 540;
const HEIGHT: u32 = 960;
/// The flat colour under every image here.
const GREY: u8 = 128;

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

fn software() -> Renderer {
    let renderer = block_on(Renderer::new_headless(Px::new(WIDTH), Px::new(HEIGHT)))
        .unwrap_or_else(|error| panic!("no software adapter: {error}"));
    assert_eq!(renderer.adapter_info().device_type, wgpu::DeviceType::Cpu);
    renderer
}

fn whole() -> Rect {
    Rect {
        x: 0.0,
        y: 0.0,
        w: WIDTH as f32,
        h: HEIGHT as f32,
    }
}

/// A black source under a flat grey rectangle over the whole canvas, then `images`.
fn over_grey(images: Vec<DrawCmd>) -> DisplayList {
    let mut cmds = vec![
        DrawCmd::Video {
            slot: VideoSlot::Main,
            src: CropRect {
                x: 0.0,
                y: 0.0,
                w: WIDTH as f32,
                h: HEIGHT as f32,
            },
            dst: whole(),
            opacity: 1.0,
            mask: None,
        },
        DrawCmd::FillRect {
            rect: whole(),
            radius: 0.0,
            color: Rgba {
                r: GREY,
                g: GREY,
                b: GREY,
                a: 255,
            },
        },
    ];
    cmds.extend(images);
    DisplayList { cmds }
}

fn image(image: ImageKey, (x, y, side): (f32, f32, f32), opacity: f32) -> DrawCmd {
    DrawCmd::Image {
        image,
        rect: Rect {
            x,
            y,
            w: side,
            h: side,
        },
        opacity,
    }
}

fn draw(renderer: &mut Renderer, list: &DisplayList) -> Result<RgbaImage, RenderError> {
    let source = RgbaImage {
        width: WIDTH,
        height: HEIGHT,
        data: [0, 0, 0, 255].repeat((WIDTH * HEIGHT) as usize),
    };
    renderer.render_to_image(&source, Rotation::R0, list, &FontSet::embedded())
}

/// The 160 x 160 straight-alpha pixels of a bundle file.
fn pixels(key: ImageKey) -> Vec<u8> {
    let file = ImageSet::embedded().get(key).unwrap();
    let mut reader = png::Decoder::new(std::io::Cursor::new(file))
        .read_info()
        .unwrap();
    let mut data = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut data).unwrap();
    data.truncate(info.buffer_size());
    data
}

fn at(image: &RgbaImage, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * image.width + x) * 4) as usize;
    image.data[i..i + 4].try_into().unwrap()
}

/// The largest channel difference between the frame at `(x0, y0)` and the
/// file of `key` blended over grey at `opacity`, in sRGB values as the
/// overlay blends.
fn worst_blend(frame: &RgbaImage, key: ImageKey, (x0, y0): (u32, u32), opacity: f64) -> f64 {
    let file = pixels(key);
    let mut worst = 0.0f64;
    for y in 0..ImageSet::SIZE {
        for x in 0..ImageSet::SIZE {
            let i = ((y * ImageSet::SIZE + x) * 4) as usize;
            let alpha = f64::from(file[i + 3]) / 255.0 * opacity;
            let drawn = at(frame, x0 + x, y0 + y);
            for c in 0..3 {
                let want = f64::from(file[i + c]) * alpha + f64::from(GREY) * (1.0 - alpha);
                worst = worst.max((f64::from(drawn[c]) - want).abs());
            }
        }
    }
    worst
}

#[test]
fn an_image_at_1x_is_its_file_blended_at_full_and_half_opacity() {
    let mut renderer = software();
    renderer.set_images(&ImageSet::embedded()).unwrap();
    let laughing = ImageKey::of_emoji(EmojiId::Laughing);
    let heart_eyes = ImageKey::of_emoji(EmojiId::HeartEyes);
    let list = over_grey(vec![
        image(laughing, (100.0, 100.0, 160.0), 1.0),
        image(heart_eyes, (100.0, 400.0, 160.0), 0.5),
    ]);
    let frame = draw(&mut renderer, &list).unwrap();
    let full = worst_blend(&frame, laughing, (100, 100), 1.0);
    let half = worst_blend(&frame, heart_eyes, (100, 400), 0.5);
    println!("worst channel difference: {full:.2} at 1.0, {half:.2} at 0.5");
    assert!(full <= 2.0, "at opacity 1.0: {full}");
    assert!(half <= 2.0, "at opacity 0.5: {half}");
    // Outside the images, the grey alone.
    assert_eq!(at(&frame, 50, 50), [GREY, GREY, GREY, 255]);
    assert_eq!(at(&frame, 300, 700), [GREY, GREY, GREY, 255]);
}

#[test]
fn a_white_icon_between_pixels_has_soft_edges_and_no_dark_fringe() {
    let mut renderer = software();
    renderer.set_images(&ImageSet::embedded()).unwrap();
    let check = ImageKey::of_icon(IconId::Check);
    // 0.5x, 1.37x and 1.5x, each starting between pixels.
    let list = over_grey(vec![
        image(check, (20.3, 20.7, 80.0), 1.0),
        image(check, (120.25, 20.5, 219.2), 1.0),
        image(check, (20.5, 300.5, 240.0), 1.0),
    ]);
    let frame = draw(&mut renderer, &list).unwrap();
    let (mut soft, mut white) = (0u32, 0u32);
    for p in frame.data.chunks(4) {
        // White over grey stays grey or lighter, and neutral.
        assert!(p[..3].iter().all(|c| *c >= GREY - 1), "dark fringe: {p:?}");
        let spread = p[..3].iter().max().unwrap() - p[..3].iter().min().unwrap();
        assert!(spread <= 1, "not neutral: {p:?}");
        if p[0] > GREY + 1 && p[0] < 250 {
            soft += 1;
        }
        if p[0] >= 250 {
            white += 1;
        }
    }
    println!("{soft} soft edge pixels, {white} white");
    assert!(soft > 1_000, "{soft} soft edge pixels");
    assert!(white > 10_000, "{white} white pixels");
}

#[test]
fn a_key_the_renderer_does_not_hold_is_unsupported() {
    let mut renderer = software();
    let fire = ImageKey::of_emoji(EmojiId::Fire);
    let list = |key, side: f32| over_grey(vec![image(key, (10.0, 10.0, side), 1.0)]);
    // Before any set is handed over, every image.
    assert_eq!(
        draw(&mut renderer, &list(fire, 80.0)),
        Err(RenderError::Unsupported)
    );
    renderer.set_images(&ImageSet::embedded()).unwrap();
    assert!(draw(&mut renderer, &list(fire, 80.0)).is_ok());
    // A key between the two sets, and one past the icons.
    for key in [ImageKey(48), ImageKey(999), ImageKey(1024)] {
        assert_eq!(
            draw(&mut renderer, &list(key, 80.0)),
            Err(RenderError::Unsupported),
            "{key:?}"
        );
    }
    // A rectangle that is not a number; an empty one draws nothing.
    assert_eq!(
        draw(&mut renderer, &list(fire, f32::NAN)),
        Err(RenderError::Unsupported)
    );
    let empty = draw(&mut renderer, &list(fire, 0.0)).unwrap();
    assert_eq!(at(&empty, 10, 10), [GREY, GREY, GREY, 255]);
    // An empty set takes the images away again.
    renderer.set_images(&ImageSet::default()).unwrap();
    assert_eq!(
        draw(&mut renderer, &list(fire, 80.0)),
        Err(RenderError::Unsupported)
    );
}
