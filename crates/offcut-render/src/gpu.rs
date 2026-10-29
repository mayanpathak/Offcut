//! The device, the queue and what a frame is drawn to: a canvas of the page,
//! presented after each frame, or (natively, for tests) a texture in memory,
//! read back after each frame (v4implementation D-134). One adapter and one
//! device per renderer.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use web_sys::OffscreenCanvas;

use crate::RenderError;
#[cfg(not(target_arch = "wasm32"))]
use crate::RgbaImage;

/// What the frames are drawn to.
enum Target {
    /// A canvas: each frame is drawn to the surface's texture and presented.
    Canvas {
        surface: wgpu::Surface<'static>,
        config: wgpu::SurfaceConfiguration,
    },
    /// A texture in memory, and the buffer it is copied to for reading.
    #[cfg(not(target_arch = "wasm32"))]
    Image {
        texture: wgpu::Texture,
        readback: wgpu::Buffer,
    },
}

/// The texture one frame is drawn to.
pub(crate) enum Frame {
    Canvas(wgpu::SurfaceTexture),
    #[cfg(not(target_arch = "wasm32"))]
    Image(wgpu::Texture),
}

impl Frame {
    pub fn texture(&self) -> &wgpu::Texture {
        match self {
            Frame::Canvas(surface) => &surface.texture,
            #[cfg(not(target_arch = "wasm32"))]
            Frame::Image(texture) => texture,
        }
    }
}

pub(crate) struct Gpu {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    target: Target,
    /// Set by the device when it is lost; the next `render` reports it.
    lost: Arc<AtomicBool>,
}

/// The surface of a canvas. A canvas exists in a browser only.
#[cfg(target_arch = "wasm32")]
fn canvas_surface(
    instance: &wgpu::Instance,
    canvas: OffscreenCanvas,
) -> Result<wgpu::Surface<'static>, RenderError> {
    instance
        .create_surface(wgpu::SurfaceTarget::OffscreenCanvas(canvas))
        .map_err(|_| RenderError::Surface)
}

/// Natively a frame is drawn to a texture in memory (`new_headless`).
#[cfg(not(target_arch = "wasm32"))]
fn canvas_surface(
    _instance: &wgpu::Instance,
    _canvas: OffscreenCanvas,
) -> Result<wgpu::Surface<'static>, RenderError> {
    Err(RenderError::Surface)
}

/// How a surface of this size is set up. The format is one without a
/// conversion to sRGB on writing: what is drawn is sRGB-encoded already.
fn configuration(
    surface: &wgpu::Surface<'_>,
    adapter: &wgpu::Adapter,
    width: u32,
    height: u32,
) -> Result<wgpu::SurfaceConfiguration, RenderError> {
    if width == 0 || height == 0 {
        return Err(RenderError::Surface);
    }
    let mut config = surface
        .get_default_config(adapter, width, height)
        .ok_or(RenderError::Surface)?;
    let formats = surface.get_capabilities(adapter).formats;
    if let Some(plain) = formats.iter().find(|format| !format.is_srgb()) {
        config.format = *plain;
    }
    Ok(config)
}

/// The device of `adapter`, with the flag its loss sets.
async fn device(
    adapter: &wgpu::Adapter,
) -> Result<(wgpu::Device, wgpu::Queue, Arc<AtomicBool>), RenderError> {
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("offcut-render"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            ..Default::default()
        })
        .await
        .map_err(|_| RenderError::DeviceRequest)?;
    let lost = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&lost);
    device.set_device_lost_callback(move |_reason, _message| {
        flag.store(true, Ordering::Relaxed);
    });
    Ok((device, queue, lost))
}

/// What a texture in memory is drawn in: what a canvas takes, sRGB-encoded
/// values written as they are.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) const IMAGE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// A row of the readback buffer, padded to wgpu's copy alignment of 256 bytes.
#[cfg(not(target_arch = "wasm32"))]
fn padded_row(width: u32) -> u32 {
    (width * 4).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT
}

/// A texture of `width` by `height` to draw to and copy from, and the buffer
/// it is read through.
#[cfg(not(target_arch = "wasm32"))]
fn image_target(device: &wgpu::Device, width: u32, height: u32) -> Target {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("image"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: IMAGE_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("readback"),
        size: u64::from(padded_row(width)) * u64::from(height),
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    Target::Image { texture, readback }
}

impl Gpu {
    pub async fn new(canvas: OffscreenCanvas, width: u32, height: u32) -> Result<Gpu, RenderError> {
        // A build without a graphics backend for its platform has no adapter.
        if wgpu::Instance::enabled_backend_features().is_empty() {
            return Err(RenderError::NoAdapter);
        }
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let surface = canvas_surface(&instance, canvas)?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                // No preference for one GPU over another: the browser picks
                // the adapter, as it does for the rest of the page.
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .map_err(|_| RenderError::NoAdapter)?;
        let (device, queue, lost) = device(&adapter).await?;

        let config = configuration(&surface, &adapter, width, height)?;
        surface.configure(&device, &config);
        Ok(Gpu {
            instance,
            adapter,
            device,
            queue,
            target: Target::Canvas { surface, config },
            lost,
        })
    }

    /// A device on a software adapter (`force_fallback_adapter`: WARP through
    /// DX12 on Windows, lavapipe through Vulkan elsewhere), drawing to a
    /// texture in memory of `width` by `height` (D-134). `NoAdapter` when the
    /// machine has none.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn new_headless(
        width: u32,
        height: u32,
        dx12: wgpu::Dx12Compiler,
    ) -> Result<Gpu, RenderError> {
        if width == 0 || height == 0 {
            return Err(RenderError::Surface);
        }
        // The native backends are enabled by the dev-dependency only (D-134).
        if wgpu::Instance::enabled_backend_features().is_empty() {
            return Err(RenderError::NoAdapter);
        }
        let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
        descriptor.backends = if cfg!(windows) {
            wgpu::Backends::DX12
        } else {
            wgpu::Backends::VULKAN
        };
        descriptor.backend_options.dx12.shader_compiler = dx12;
        let instance = wgpu::Instance::new(descriptor);
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                force_fallback_adapter: true,
                compatible_surface: None,
                ..Default::default()
            })
            .await
            .map_err(|_| RenderError::NoAdapter)?;
        let (device, queue, lost) = device(&adapter).await?;
        let target = image_target(&device, width, height);
        Ok(Gpu {
            instance,
            adapter,
            device,
            queue,
            target,
            lost,
        })
    }

    /// What the adapter says it is: a test asks it whether it is a software one.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn adapter_info(&self) -> wgpu::AdapterInfo {
        self.adapter.get_info()
    }

    pub fn is_lost(&self) -> bool {
        self.lost.load(Ordering::Relaxed)
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        match &self.target {
            Target::Canvas { config, .. } => config.format,
            #[cfg(not(target_arch = "wasm32"))]
            Target::Image { .. } => IMAGE_FORMAT,
        }
    }

    pub fn size(&self) -> (u32, u32) {
        match &self.target {
            Target::Canvas { config, .. } => (config.width, config.height),
            #[cfg(not(target_arch = "wasm32"))]
            Target::Image { texture, .. } => (texture.width(), texture.height()),
        }
    }

    /// Sets the target up again when the size is another one.
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), RenderError> {
        if (width, height) == self.size() {
            return Ok(());
        }
        if width == 0 || height == 0 {
            return Err(RenderError::Surface);
        }
        match &mut self.target {
            Target::Canvas { surface, config } => {
                config.width = width;
                config.height = height;
                surface.configure(&self.device, config);
            }
            #[cfg(not(target_arch = "wasm32"))]
            Target::Image { .. } => self.target = image_target(&self.device, width, height),
        }
        Ok(())
    }

    /// Another canvas on the same device. Its surface must take the format
    /// the pipelines were built for. A renderer that draws to memory has no
    /// canvas to change.
    pub fn set_canvas(
        &mut self,
        canvas: OffscreenCanvas,
        width: u32,
        height: u32,
    ) -> Result<(), RenderError> {
        let surface = canvas_surface(&self.instance, canvas)?;
        let config = configuration(&surface, &self.adapter, width, height)?;
        if config.format != self.format() {
            return Err(RenderError::Surface);
        }
        surface.configure(&self.device, &config);
        self.target = Target::Canvas { surface, config };
        Ok(())
    }

    /// The texture the next frame is drawn to.
    pub fn frame(&self) -> Result<Frame, RenderError> {
        match &self.target {
            Target::Canvas { surface, .. } => match surface.get_current_texture() {
                wgpu::CurrentSurfaceTexture::Success(texture)
                | wgpu::CurrentSurfaceTexture::Suboptimal(texture) => Ok(Frame::Canvas(texture)),
                wgpu::CurrentSurfaceTexture::Lost => Err(RenderError::DeviceLost),
                wgpu::CurrentSurfaceTexture::Timeout
                | wgpu::CurrentSurfaceTexture::Occluded
                | wgpu::CurrentSurfaceTexture::Outdated
                | wgpu::CurrentSurfaceTexture::Validation => Err(RenderError::Surface),
            },
            #[cfg(not(target_arch = "wasm32"))]
            Target::Image { texture, .. } => Ok(Frame::Image(texture.clone())),
        }
    }

    /// Shows a drawn frame on its canvas. A frame in memory is read instead.
    pub fn present(&self, frame: Frame) {
        match frame {
            Frame::Canvas(texture) => self.queue.present(texture),
            #[cfg(not(target_arch = "wasm32"))]
            Frame::Image(_) => {}
        }
    }

    /// The pixels of a drawn frame in memory, row by row: the texture is
    /// copied to the readback buffer with each row padded to 256 bytes, and
    /// the padding is left out on reading.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn read(&self, frame: &Frame) -> Result<RgbaImage, RenderError> {
        let Target::Image { readback, .. } = &self.target else {
            return Err(RenderError::Surface);
        };
        let texture = frame.texture();
        let (width, height) = (texture.width(), texture.height());
        let padded = padded_row(width);
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("readback"),
            });
        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit([encoder.finish()]);

        let slice = readback.slice(..);
        let (sent, mapped) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            // The receiver waits below; a closed channel means nobody reads.
            let _ = sent.send(result);
        });
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|_| RenderError::DeviceLost)?;
        mapped
            .recv()
            .map_err(|_| RenderError::Surface)?
            .map_err(|_| RenderError::Surface)?;

        let row = usize::try_from(width * 4).map_err(|_| RenderError::Surface)?;
        let step = usize::try_from(padded).map_err(|_| RenderError::Surface)?;
        let mut data = Vec::with_capacity(row * usize::try_from(height).unwrap_or(0));
        {
            let view = slice.get_mapped_range().map_err(|_| RenderError::Surface)?;
            for padded_row in view.chunks(step) {
                data.extend_from_slice(padded_row.get(..row).ok_or(RenderError::Surface)?);
            }
        }
        readback.unmap();
        Ok(RgbaImage {
            width,
            height,
            data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_row_is_padded_to_256_bytes() {
        // 540 x 4 = 2,160 bytes: nine blocks of 256.
        assert_eq!(padded_row(540), 2_304);
        assert_eq!(padded_row(64), 256);
        assert_eq!(padded_row(1), 256);
        assert_eq!(padded_row(1_080), 4_352);
    }
}
