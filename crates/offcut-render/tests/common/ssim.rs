//! The structural similarity of two images (v4implementation §9.4, D-135):
//! on BT.709 luma, over the 8 x 8 windows that tile the image, with the
//! standard constants, the mean over windows. Written here: no crate.

use offcut_render::RgbaImage;

/// The side of a window, in pixels.
pub const WINDOW: u32 = 8;
/// `(0.01 x 255)²` and `(0.03 x 255)²`.
const C1: f64 = (0.01 * 255.0) * (0.01 * 255.0);
const C2: f64 = (0.03 * 255.0) * (0.03 * 255.0);

/// BT.709 luma of a pixel's encoded values, 0 to 255.
fn luma(pixel: &[u8]) -> f64 {
    match pixel {
        [r, g, b, _] => 0.2126 * f64::from(*r) + 0.7152 * f64::from(*g) + 0.0722 * f64::from(*b),
        _ => panic!("a pixel is four bytes"),
    }
}

/// The SSIM of `a` and `b`, which are the same size: 1.0 for equal images.
/// Pixels past the last whole window, right or bottom, are not compared.
pub fn ssim(a: &RgbaImage, b: &RgbaImage) -> f64 {
    assert_eq!((a.width, a.height), (b.width, b.height), "two sizes");
    let (columns, rows) = (a.width / WINDOW, a.height / WINDOW);
    assert!(columns > 0 && rows > 0, "an image smaller than a window");
    let y = |image: &RgbaImage, x: u32, y: u32| {
        let at = ((y * image.width + x) * 4) as usize;
        luma(&image.data[at..at + 4])
    };
    let n = f64::from(WINDOW * WINDOW);
    let mut total = 0.0;
    for row in 0..rows {
        for column in 0..columns {
            let (mut sa, mut sb, mut saa, mut sbb, mut sab) = (0.0, 0.0, 0.0, 0.0, 0.0);
            for dy in 0..WINDOW {
                for dx in 0..WINDOW {
                    let (px, py) = (column * WINDOW + dx, row * WINDOW + dy);
                    let (va, vb) = (y(a, px, py), y(b, px, py));
                    sa += va;
                    sb += vb;
                    saa += va * va;
                    sbb += vb * vb;
                    sab += va * vb;
                }
            }
            let (ma, mb) = (sa / n, sb / n);
            let (va, vb) = (saa / n - ma * ma, sbb / n - mb * mb);
            let cov = sab / n - ma * mb;
            total += ((2.0 * ma * mb + C1) * (2.0 * cov + C2))
                / ((ma * ma + mb * mb + C1) * (va + vb + C2));
        }
    }
    total / f64::from(columns * rows)
}

#[test]
fn an_image_is_like_itself_and_unlike_its_inverse() {
    let (width, height) = (64, 48);
    let data: Vec<u8> = (0..width * height)
        .flat_map(|i| {
            let (x, y) = (i % width, i / width);
            let v = ((x * 4 + y * 3) % 256) as u8;
            [v, v.wrapping_mul(3), 255 - v, 255]
        })
        .collect();
    let image = RgbaImage {
        width,
        height,
        data,
    };
    assert_eq!(ssim(&image, &image), 1.0);
    let inverse = RgbaImage {
        data: image
            .data
            .chunks(4)
            .flat_map(|p| [255 - p[0], 255 - p[1], 255 - p[2], p[3]])
            .collect(),
        ..image.clone()
    };
    let s = ssim(&image, &inverse);
    assert!(s < 0.5, "{s}");
}

#[test]
fn a_small_change_keeps_the_similarity_high_and_a_darker_image_lowers_it() {
    let (width, height) = (64, 64);
    let data: Vec<u8> = (0..width * height)
        .flat_map(|i| {
            let v = ((i % width) * 4) as u8;
            [v, v, v, 255]
        })
        .collect();
    let image = RgbaImage {
        width,
        height,
        data,
    };
    let mut nudged = image.clone();
    nudged.data[0] = nudged.data[0].saturating_add(1);
    assert!(ssim(&image, &nudged) > 0.999);
    let darker = RgbaImage {
        data: image
            .data
            .chunks(4)
            .flat_map(|p| [p[0] / 2, p[1] / 2, p[2] / 2, p[3]])
            .collect(),
        ..image.clone()
    };
    assert!(ssim(&image, &darker) < 0.98);
}
