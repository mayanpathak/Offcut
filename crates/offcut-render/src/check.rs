//! The frame of TE-30 (v6.2implementation D-474, §9.1 step 3), defined once:
//! the synthetic source of the golden frames and the list of the
//! `image_basic` golden. `tests/golden_frames.rs` draws them natively and
//! asserts they are its own; the render bundle's development check draws
//! them in the browser, so the two sides compare the same frame.

use offcut_scene::{CropRect, DisplayList, DrawCmd, ImageKey, Rect, Rgba, VideoSlot};
use offcut_types::{EmojiId, IconId};

use crate::RgbaImage;

/// The canvas the check is drawn at: the preview's size, as every golden.
pub const CANVAS: (u32, u32) = (540, 960);
/// The portrait source the check is drawn over.
pub const SOURCE: (u32, u32) = (1080, 1920);

/// The synthetic source of the goldens (v4implementation §9.4): a diagonal
/// gradient, an 8 x 8 grid of lines two pixels wide, and a filled disc at a
/// third of the width. Pixel for pixel `golden_frames.rs`' `pattern`.
pub fn pattern(width: u32, height: u32) -> RgbaImage {
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

/// An image of the bundle at `(x, y)`, `side` pixels square, at `opacity`.
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

/// `image_basic`'s list on `CANVAS` (P196): the whole `SOURCE`, then three
/// emoji at 0.5x, 1x and 1.5x (the last between pixels, over the disc), one
/// at opacity 0.5, one across the right edge, a white icon, and a half-dark
/// rectangle after one emoji, which list order puts over it.
pub fn image_basic() -> DisplayList {
    let (canvas_w, canvas_h) = (CANVAS.0 as f32, CANVAS.1 as f32);
    let emoji = ImageKey::of_emoji;
    DisplayList {
        cmds: vec![
            DrawCmd::Video {
                slot: VideoSlot::Main,
                src: CropRect {
                    x: 0.0,
                    y: 0.0,
                    w: SOURCE.0 as f32,
                    h: SOURCE.1 as f32,
                },
                dst: Rect {
                    x: 0.0,
                    y: 0.0,
                    w: canvas_w,
                    h: canvas_h,
                },
                opacity: 1.0,
                mask: None,
            },
            image(emoji(EmojiId::Fire), (40.0, 60.0, 80.0), 1.0),
            image(emoji(EmojiId::Laughing), (170.0, 40.0, 160.0), 1.0),
            image(emoji(EmojiId::Rocket), (100.5, 330.25, 240.0), 1.0),
            image(emoji(EmojiId::HeartEyes), (360.0, 260.0, 160.0), 0.5),
            image(emoji(EmojiId::Trophy), (460.0, 700.0, 160.0), 1.0),
            image(ImageKey::of_icon(IconId::Check), (60.0, 700.0, 120.0), 1.0),
            DrawCmd::FillRect {
                rect: Rect {
                    x: 250.0,
                    y: 120.0,
                    w: 120.0,
                    h: 40.0,
                },
                radius: 0.0,
                color: Rgba {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 128,
                },
            },
        ],
    }
}
