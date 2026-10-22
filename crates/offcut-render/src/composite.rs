//! One render pass to the canvas: the source frame, then the overlay over it.
//! What a list may ask of the pass is V4's support rule (D-133); V6.5
//! replaces the rule with layers, and the signature of `render` stays.

use offcut_scene::{CropRect, DisplayList, DrawCmd, Rect, VideoSlot};

use crate::RenderError;
use crate::gpu::Gpu;
use crate::video_pass::{
    VideoPass, linear_sampler, sampler_entry, texture_entry, whole_target_pipeline,
};

/// The video of `list`, if V4 can draw the list on a canvas of `canvas`
/// pixels: exactly one `Video` command, the first, of the main slot, opaque,
/// with no mask, its `src` a part of the frame and its `dst` inside the
/// canvas; and no `Image`. Every other list is `Unsupported`.
pub(crate) fn video_of(
    list: &DisplayList,
    canvas: (u32, u32),
) -> Result<(CropRect, Rect), RenderError> {
    let mut video = None;
    for (index, cmd) in list.cmds.iter().enumerate() {
        match cmd {
            DrawCmd::Video {
                slot,
                src,
                dst,
                opacity,
                mask,
            } => {
                let VideoSlot::Main = slot;
                let drawable = index == 0
                    && *opacity == 1.0
                    && mask.is_none()
                    && is_part(src)
                    && inside(dst, canvas);
                if !drawable {
                    return Err(RenderError::Unsupported);
                }
                video = Some((*src, *dst));
            }
            DrawCmd::Image { .. } => return Err(RenderError::Unsupported),
            DrawCmd::FillRect { .. }
            | DrawCmd::FillPath { .. }
            | DrawCmd::GlyphRun { .. }
            | DrawCmd::PushLayer { .. }
            | DrawCmd::PopLayer => {}
        }
    }
    video.ok_or(RenderError::Unsupported)
}

/// A part of a frame: not empty, not before its top-left corner. Where it
/// ends is the frame's to say; the sampler holds its edge beyond it.
fn is_part(src: &CropRect) -> bool {
    [src.x, src.y, src.w, src.h].iter().all(|v| v.is_finite())
        && src.x >= 0.0
        && src.y >= 0.0
        && src.w > 0.0
        && src.h > 0.0
}

fn inside(dst: &Rect, canvas: (u32, u32)) -> bool {
    let (width, height) = (canvas.0 as f32, canvas.1 as f32);
    [dst.x, dst.y, dst.w, dst.h].iter().all(|v| v.is_finite())
        && dst.w > 0.0
        && dst.h > 0.0
        && dst.x >= 0.0
        && dst.y >= 0.0
        && dst.x + dst.w <= width
        && dst.y + dst.h <= height
}

pub(crate) struct Compositor {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    /// The overlay texture as the pipeline reads it. Made again when the
    /// overlay texture is.
    overlay: wgpu::BindGroup,
}

/// What the overlay's fragment shader writes is premultiplied by its alpha:
/// the colour is added to what is left of the frame under it.
const PREMULTIPLIED: wgpu::BlendState = wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING;

fn bind(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    overlay: &wgpu::TextureView,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("overlay"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(overlay),
            },
        ],
    })
}

impl Compositor {
    pub fn new(
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        target: wgpu::TextureFormat,
        overlay: &wgpu::TextureView,
    ) -> Compositor {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("overlay"),
            entries: &[sampler_entry(1), texture_entry(3)],
        });
        let sampler = linear_sampler(device);
        Compositor {
            pipeline: whole_target_pipeline(
                device,
                shader,
                &layout,
                "over",
                target,
                Some(PREMULTIPLIED),
            ),
            overlay: bind(device, &layout, &sampler, overlay),
            layout,
            sampler,
        }
    }

    /// Reads from another overlay texture: the one of a new size.
    pub fn set_overlay(&mut self, device: &wgpu::Device, overlay: &wgpu::TextureView) {
        self.overlay = bind(device, &self.layout, &self.sampler, overlay);
    }

    /// Draws the frame `video` holds, then the overlay when there is one to
    /// draw, to `target`, and submits the pass. Presenting or reading the
    /// target is the caller's.
    pub fn draw(&self, gpu: &Gpu, target: &wgpu::Texture, video: &VideoPass, with_overlay: bool) {
        let view = target.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("frame"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            video.draw(&mut pass);
            if with_overlay {
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &self.overlay, &[]);
                pass.draw(0..3, 0..1);
            }
        }
        gpu.queue.submit([encoder.finish()]);
    }
}

#[cfg(test)]
mod tests {
    use offcut_scene::{Affine, FontKey, ImageKey, PathEl, Rgba};

    use super::*;

    const CANVAS: (u32, u32) = (540, 960);

    fn strip() -> CropRect {
        CropRect {
            x: 656.25,
            y: 0.0,
            w: 607.5,
            h: 1080.0,
        }
    }

    fn whole() -> Rect {
        Rect {
            x: 0.0,
            y: 0.0,
            w: 540.0,
            h: 960.0,
        }
    }

    fn video() -> DrawCmd {
        DrawCmd::Video {
            slot: VideoSlot::Main,
            src: strip(),
            dst: whole(),
            opacity: 1.0,
            mask: None,
        }
    }

    fn text() -> DrawCmd {
        DrawCmd::GlyphRun {
            font: FontKey(0),
            size: 32.0,
            glyphs: Vec::new(),
            fill: Rgba {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
            stroke: None,
            transform: Affine::IDENTITY,
            text: "hi".to_owned(),
        }
    }

    fn list(cmds: Vec<DrawCmd>) -> DisplayList {
        DisplayList { cmds }
    }

    #[test]
    fn a_scenes_list_is_drawn_with_its_video() {
        assert_eq!(
            video_of(&list(vec![video()]), CANVAS),
            Ok((strip(), whole()))
        );
        let with_text = list(vec![video(), text(), DrawCmd::PopLayer]);
        assert_eq!(video_of(&with_text, CANVAS), Ok((strip(), whole())));
        // A smaller rectangle inside the canvas: black around it.
        let inset = DrawCmd::Video {
            slot: VideoSlot::Main,
            src: strip(),
            dst: Rect {
                x: 10.0,
                y: 20.0,
                w: 270.0,
                h: 480.0,
            },
            opacity: 1.0,
            mask: None,
        };
        assert!(video_of(&list(vec![inset]), CANVAS).is_ok());
    }

    #[test]
    fn a_list_outside_v4s_rule_is_unsupported() {
        let unsupported = Err(RenderError::Unsupported);
        let image = DrawCmd::Image {
            image: ImageKey(0),
            rect: whole(),
            opacity: 1.0,
        };
        let with = |change: fn(&mut DrawCmd)| {
            let mut cmd = video();
            change(&mut cmd);
            list(vec![cmd])
        };
        let cases = [
            ("an image", list(vec![video(), image])),
            ("two videos", list(vec![video(), video()])),
            ("a video not first", list(vec![text(), video()])),
            ("no video", list(vec![text()])),
            (
                "a mask",
                with(|cmd| {
                    if let DrawCmd::Video { mask, .. } = cmd {
                        *mask = Some(vec![PathEl::MoveTo(0.0, 0.0), PathEl::Close]);
                    }
                }),
            ),
            (
                "half opaque",
                with(|cmd| {
                    if let DrawCmd::Video { opacity, .. } = cmd {
                        *opacity = 0.5;
                    }
                }),
            ),
            (
                "past the canvas",
                with(|cmd| {
                    if let DrawCmd::Video { dst, .. } = cmd {
                        dst.w = 541.0;
                    }
                }),
            ),
            (
                "an empty source",
                with(|cmd| {
                    if let DrawCmd::Video { src, .. } = cmd {
                        src.w = 0.0;
                    }
                }),
            ),
        ];
        for (name, list) in cases {
            assert_eq!(video_of(&list, CANVAS), unsupported, "{name}");
        }
    }
}
