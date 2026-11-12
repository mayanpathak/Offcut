//! The overlay: the commands of a display list after its video, drawn by
//! Vello into a texture as large as the target. This file reads them command
//! by command and knows nothing of what they mean. The video is the
//! compositor's (D-133); images are drawn here, in list order with the text
//! (v6.2implementation D-472).

use std::collections::BTreeMap;
use std::sync::Arc;

use offcut_scene::{Affine, DrawCmd, FontKey, FontSet, ImageKey, ImageSet, PathEl, Rect, Rgba};
use vello::kurbo;
use vello::peniko::{
    Blob, Color, Fill, FontData, ImageAlphaType, ImageBrush, ImageData, ImageFormat, ImageQuality,
    Mix,
};

use crate::RenderError;

/// What Vello renders into: this format, and written by a compute shader.
const OVERLAY_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// A face as Vello takes it: the file, and where on its axes the face
/// stands. `file` is the set's own data, by which a cached face is known to
/// still be the face of its key.
struct Face {
    file: Arc<[u8]>,
    data: FontData,
    coords: Vec<i16>,
}

pub(crate) struct Overlay {
    renderer: vello::Renderer,
    /// Kept between frames for its allocations; emptied before each.
    scene: vello::Scene,
    view: wgpu::TextureView,
    width: u32,
    height: u32,
    /// Each face is read on its first use, once per key (D-132).
    faces: BTreeMap<FontKey, Face>,
    /// The images handed over last, each decoded once (D-472).
    images: BTreeMap<ImageKey, ImageData>,
}

fn texture_view(device: &wgpu::Device, width: u32, height: u32) -> wgpu::TextureView {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("overlay"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: OVERLAY_FORMAT,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}

fn color(c: Rgba) -> Color {
    Color::from_rgba8(c.r, c.g, c.b, c.a)
}

fn affine(a: &Affine) -> kurbo::Affine {
    kurbo::Affine::new(a.m.map(f64::from))
}

fn rect(r: &Rect) -> kurbo::Rect {
    let (x, y) = (f64::from(r.x), f64::from(r.y));
    kurbo::Rect::new(x, y, x + f64::from(r.w), y + f64::from(r.h))
}

/// A PNG of the bundle as Vello takes it: 8-bit RGBA, straight alpha, as the
/// file holds it. Vello premultiplies each texel before it filters (its
/// `fine.wgsl`, `maybe_premul_alpha`), so nothing is premultiplied here
/// (D-472's note of P183). `Overlay` for a file that is not an 8-bit RGBA PNG.
pub(crate) fn decode(file: &[u8]) -> Result<ImageData, RenderError> {
    let decoder = png::Decoder::new(std::io::Cursor::new(file));
    let mut reader = decoder.read_info().map_err(|_| RenderError::Overlay)?;
    let mut data = vec![0; reader.output_buffer_size().ok_or(RenderError::Overlay)?];
    let info = reader
        .next_frame(&mut data)
        .map_err(|_| RenderError::Overlay)?;
    if (info.color_type, info.bit_depth) != (png::ColorType::Rgba, png::BitDepth::Eight) {
        return Err(RenderError::Overlay);
    }
    data.truncate(info.buffer_size());
    Ok(ImageData {
        data: Blob::from(data),
        format: ImageFormat::Rgba8,
        alpha_type: ImageAlphaType::Alpha,
        width: info.width,
        height: info.height,
    })
}

/// Where an image of `size` pixels lands to fill `r`: scaled, then moved.
fn placed(r: &Rect, size: (u32, u32)) -> kurbo::Affine {
    let (sx, sy) = (
        f64::from(r.w) / f64::from(size.0),
        f64::from(r.h) / f64::from(size.1),
    );
    kurbo::Affine::translate((f64::from(r.x), f64::from(r.y)))
        * kurbo::Affine::scale_non_uniform(sx, sy)
}

fn path(elements: &[PathEl]) -> kurbo::BezPath {
    let point = |x: f32, y: f32| kurbo::Point::new(f64::from(x), f64::from(y));
    let mut path = kurbo::BezPath::new();
    for element in elements {
        match *element {
            PathEl::MoveTo(x, y) => path.move_to(point(x, y)),
            PathEl::LineTo(x, y) => path.line_to(point(x, y)),
            PathEl::QuadTo(a, b, x, y) => path.quad_to(point(a, b), point(x, y)),
            PathEl::CurveTo(a, b, c, d, x, y) => {
                path.curve_to(point(a, b), point(c, d), point(x, y));
            }
            PathEl::Close => path.close_path(),
        }
    }
    path
}

impl Overlay {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Result<Overlay, RenderError> {
        let options = vello::RendererOptions {
            use_cpu: false,
            // The one method `render` asks for.
            antialiasing_support: vello::AaSupport::area_only(),
            num_init_threads: None,
            pipeline_cache: None,
        };
        let renderer = vello::Renderer::new(device, options).map_err(|_| RenderError::Overlay)?;
        Ok(Overlay {
            renderer,
            scene: vello::Scene::new(),
            view: texture_view(device, width, height),
            width,
            height,
            faces: BTreeMap::new(),
            images: BTreeMap::new(),
        })
    }

    /// Holds the images of `images`, each decoded once, in place of those
    /// held before. On an error the images held before are kept.
    pub fn set_images(&mut self, images: &ImageSet) -> Result<(), RenderError> {
        let decoded = images
            .iter()
            .map(|(key, file)| Ok((key, decode(file)?)))
            .collect::<Result<BTreeMap<_, _>, RenderError>>()?;
        self.images = decoded;
        Ok(())
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    /// A texture of the new size. The caller asks only when the size changed.
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.view = texture_view(device, width, height);
        self.width = width;
        self.height = height;
    }

    /// The file and the axis coordinates of the face `key` of `fonts`, read
    /// once. A key whose face is another file than the one read is read again.
    fn face<'a>(
        faces: &'a mut BTreeMap<FontKey, Face>,
        key: FontKey,
        fonts: &FontSet,
    ) -> Result<&'a Face, RenderError> {
        let wanted = fonts.get(key).ok_or(RenderError::Overlay)?;
        let held = faces
            .get(&key)
            .is_some_and(|face| Arc::ptr_eq(&face.file, &wanted.data));
        if !held {
            let file = Arc::clone(&wanted.data);
            let data = FontData::new(Blob::new(Arc::new(Arc::clone(&file))), 0);
            let coords = wanted.coords.clone();
            faces.insert(key, Face { file, data, coords });
        }
        faces.get(&key).ok_or(RenderError::Overlay)
    }

    /// Draws `cmds` into the overlay texture, over nothing: where they draw
    /// nothing the texture is transparent. Glyph runs are set in the faces of
    /// `fonts`; an image is one `set_images` handed over, and a key it did not
    /// is `Unsupported`, never a blank (§9.2).
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        cmds: &[DrawCmd],
        fonts: &FontSet,
    ) -> Result<(), RenderError> {
        self.scene.reset();
        let whole = kurbo::Rect::new(0.0, 0.0, f64::from(self.width), f64::from(self.height));
        let mut layers = 0u32;
        for cmd in cmds {
            match cmd {
                DrawCmd::FillRect {
                    rect: r,
                    radius,
                    color: c,
                } => {
                    let shape = kurbo::RoundedRect::from_rect(rect(r), f64::from(*radius));
                    self.scene.fill(
                        Fill::NonZero,
                        kurbo::Affine::IDENTITY,
                        color(*c),
                        None,
                        &shape,
                    );
                }
                DrawCmd::FillPath { path: p, color: c } => {
                    let shape = path(p);
                    self.scene.fill(
                        Fill::NonZero,
                        kurbo::Affine::IDENTITY,
                        color(*c),
                        None,
                        &shape,
                    );
                }
                DrawCmd::GlyphRun {
                    font,
                    size,
                    glyphs,
                    fill,
                    stroke,
                    transform,
                    text: _,
                } => {
                    let face = Self::face(&mut self.faces, *font, fonts)?;
                    let glyphs = || {
                        glyphs.iter().map(|glyph| vello::Glyph {
                            id: glyph.id,
                            x: glyph.x,
                            y: glyph.y,
                        })
                    };
                    // The stroke first and the fill over it (D-68): a stroke
                    // lies half inside the outline, and drawn last it
                    // would cover that much of each letter.
                    if let Some(stroke) = stroke {
                        let style = kurbo::Stroke::new(f64::from(stroke.width));
                        self.scene
                            .draw_glyphs(&face.data)
                            .font_size(*size)
                            .normalized_coords(&face.coords)
                            .transform(affine(transform))
                            .brush(color(stroke.color))
                            .draw(&style, glyphs());
                    }
                    self.scene
                        .draw_glyphs(&face.data)
                        .font_size(*size)
                        .normalized_coords(&face.coords)
                        .transform(affine(transform))
                        .brush(color(*fill))
                        .draw(Fill::NonZero, glyphs());
                }
                DrawCmd::PushLayer { opacity, clip } => {
                    let clip = clip.as_ref().map_or(whole, rect);
                    self.scene.push_layer(
                        Fill::NonZero,
                        Mix::Normal,
                        *opacity,
                        kurbo::Affine::IDENTITY,
                        &clip,
                    );
                    layers = layers.saturating_add(1);
                }
                DrawCmd::PopLayer => {
                    // A pop without a push would close a layer that is not there.
                    if layers > 0 {
                        self.scene.pop_layer();
                        layers -= 1;
                    }
                }
                DrawCmd::Image {
                    image,
                    rect: r,
                    opacity,
                } => {
                    let data = self.images.get(image).ok_or(RenderError::Unsupported)?;
                    if ![r.x, r.y, r.w, r.h, *opacity].iter().all(|v| v.is_finite()) {
                        return Err(RenderError::Unsupported);
                    }
                    // An empty rectangle or no opacity draws nothing: a pop
                    // that starts from zero.
                    if r.w > 0.0 && r.h > 0.0 && *opacity > 0.0 {
                        let brush = ImageBrush::new(data.clone())
                            .with_quality(ImageQuality::Medium)
                            .with_alpha(opacity.min(1.0));
                        let transform = placed(r, (data.width, data.height));
                        self.scene.draw_image(&brush, transform);
                    }
                }
                // The compositor draws the video.
                DrawCmd::Video { .. } => {}
            }
        }
        // A list that leaves a layer open is closed here, so that it cannot
        // reach into the next frame.
        for _ in 0..layers {
            self.scene.pop_layer();
        }

        let params = vello::RenderParams {
            base_color: Color::TRANSPARENT,
            width: self.width,
            height: self.height,
            antialiasing_method: vello::AaConfig::Area,
        };
        self.renderer
            .render_to_texture(device, queue, &self.scene, &self.view, &params)
            .map_err(|_| RenderError::Overlay)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bundle_file_decodes_to_its_own_straight_alpha_pixels() {
        let set = ImageSet::embedded();
        let (key, file) = set.iter().next().unwrap();
        assert_eq!(key, ImageKey(0));
        let image = decode(file).unwrap();
        assert_eq!(
            (image.width, image.height),
            (ImageSet::SIZE, ImageSet::SIZE)
        );
        assert_eq!(image.format, ImageFormat::Rgba8);
        assert_eq!(image.alpha_type, ImageAlphaType::Alpha);
        let bytes = image.data.data();
        assert_eq!(bytes.len(), 160 * 160 * 4);
        // Straight alpha: a soft edge pixel keeps its full colour.
        let soft = bytes.chunks(4).find(|p| p[3] > 0 && p[3] < 128).unwrap();
        assert!(soft[..3].iter().any(|c| *c > soft[3]), "{soft:?}");
    }

    #[test]
    fn a_file_that_is_not_an_rgba_png_is_refused() {
        assert_eq!(decode(b"not a png"), Err(RenderError::Overlay));
        let mut rgb = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut rgb, 2, 2);
            encoder.set_color(png::ColorType::Rgb);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().unwrap();
            writer.write_image_data(&[0; 12]).unwrap();
        }
        assert_eq!(decode(&rgb), Err(RenderError::Overlay));
    }

    #[test]
    fn an_image_is_scaled_into_its_rectangle_then_moved() {
        let r = Rect {
            x: 10.0,
            y: 20.0,
            w: 80.0,
            h: 240.0,
        };
        let t = placed(&r, (160, 160));
        assert_eq!(
            t * kurbo::Point::new(0.0, 0.0),
            kurbo::Point::new(10.0, 20.0)
        );
        assert_eq!(
            t * kurbo::Point::new(160.0, 160.0),
            kurbo::Point::new(90.0, 260.0)
        );
    }
}
