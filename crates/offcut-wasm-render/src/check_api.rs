//! The development check of TE-30 (v6.2implementation D-474, §9.1 step 3):
//! the browser draws `image_basic`'s frame from `offcut_render::check`, the
//! definition the native golden asserts is its own, so the page can compare
//! its canvas with `fixtures/golden/image_basic.png`. Only a development
//! build's render worker calls these (`workers/render/image-check.ts`); the
//! product never does.

use offcut_render::check;
use offcut_render::scene::{FontSet, ImageSet};
use offcut_render::{Renderer, VideoSources};
use offcut_types::{Px, Rotation};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::{OffscreenCanvas, VideoFrame};

use crate::session::render_error_value;

/// A drawn check. It holds its renderer, so that the canvas keeps the frame
/// until the page has read it; `free()` gives both back.
#[wasm_bindgen]
pub struct ImageCheck {
    _renderer: Renderer,
    decode_ms: f64,
}

#[wasm_bindgen]
impl ImageCheck {
    /// How long handing the 72 images over took: each PNG decoded once.
    pub fn decode_ms(&self) -> f64 {
        self.decode_ms
    }
}

/// The source the check is drawn over, RGBA, `check::SOURCE` in size.
#[wasm_bindgen]
pub fn check_source() -> Vec<u8> {
    let (width, height) = check::SOURCE;
    check::pattern(width, height).data
}

/// Draws `image_basic`'s list on `canvas` (`check::CANVAS` in size) over
/// `frame`, which shows `check_source()`, with a renderer of its own that
/// holds the bundle's images as a session's does. The frame stays the
/// caller's. JavaScript gets a promise.
#[wasm_bindgen]
pub async fn draw_image_check(
    canvas: OffscreenCanvas,
    frame: VideoFrame,
) -> Result<ImageCheck, JsValue> {
    let (width, height) = check::CANVAS;
    let mut renderer = Renderer::new(canvas, Px::new(width), Px::new(height))
        .await
        .map_err(render_error_value)?;
    let start = js_sys::Date::now();
    renderer
        .set_images(&ImageSet::embedded())
        .map_err(render_error_value)?;
    let decode_ms = js_sys::Date::now() - start;
    let sources = VideoSources {
        main: (&frame, Rotation::R0),
    };
    renderer
        .render(&sources, &check::image_basic(), &FontSet::embedded())
        .map_err(render_error_value)?;
    Ok(ImageCheck {
        _renderer: renderer,
        decode_ms,
    })
}
