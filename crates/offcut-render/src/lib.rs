//! The compositor: a display list drawn in its order on a canvas, its video
//! command with the frame of its slot, turned and cropped (TS §19, D-133). It
//! draws what it is given. It does not know what a scene means, it detects
//! nothing, and it closes no `VideoFrame`: the caller does (TS §19.6).

mod composite;
mod gpu;
mod vello_backend;
mod video_pass;

/// The scene crate, for the binding crate: it builds a scene and hands the
/// renderer its display lists, and depends on this crate only (D-61).
pub use offcut_scene as scene;

use offcut_scene::{CropRect, DisplayList, FontSet, Rect};
use offcut_types::{Px, Rotation};
use thiserror::Error;

use crate::composite::Compositor;
use crate::gpu::{Frame, Gpu};
use crate::vello_backend::Overlay;
use crate::video_pass::VideoPass;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum RenderError {
    #[error("no graphics adapter")]
    NoAdapter,
    #[error("the adapter gave no device")]
    DeviceRequest,
    #[error("the graphics device was lost")]
    DeviceLost,
    #[error("the canvas cannot be drawn to")]
    Surface,
    #[error("the video frame could not be read")]
    FrameImport,
    #[error("the overlay could not be drawn")]
    Overlay,
    /// A list this version cannot draw (D-133): an image, a mask, a video
    /// that is not the first command or not opaque, or more than one.
    #[error("the display list asks for what this renderer does not draw")]
    Unsupported,
}

/// The frames a list's `Video` commands draw, by slot, each with the
/// rotation of its clip (D-133).
pub struct VideoSources<'a> {
    pub main: (&'a web_sys::VideoFrame, Rotation),
}

/// An image in memory, row by row from the top-left, four bytes a pixel:
/// sRGB-encoded with straight alpha. A source frame and a rendered frame of
/// `render_to_image` (D-134).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RgbaImage {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

/// One device, and the textures of one output size. Textures are made when
/// the renderer is, and again only when a size changes (TS §31).
pub struct Renderer {
    gpu: Gpu,
    video: VideoPass,
    overlay: Overlay,
    compositor: Compositor,
}

impl Renderer {
    pub async fn new(
        canvas: web_sys::OffscreenCanvas,
        width: Px,
        height: Px,
    ) -> Result<Renderer, RenderError> {
        Renderer::on(Gpu::new(canvas, width.get(), height.get()).await?)
    }

    /// The passes on `gpu`, at its size and in its format.
    fn on(gpu: Gpu) -> Result<Renderer, RenderError> {
        let (width, height) = gpu.size();
        let shader = video_pass::shader(&gpu.device);
        let video = VideoPass::new(&gpu.device, &shader, gpu.format());
        let overlay = Overlay::new(&gpu.device, width, height)?;
        let compositor = Compositor::new(&gpu.device, &shader, gpu.format(), overlay.view());
        Ok(Renderer {
            gpu,
            video,
            overlay,
            compositor,
        })
    }

    /// The output is `width` by `height` from now on. The same size again
    /// allocates nothing.
    pub fn resize(&mut self, width: Px, height: Px) -> Result<(), RenderError> {
        let size = (width.get(), height.get());
        if size == self.gpu.size() {
            return Ok(());
        }
        self.gpu.resize(size.0, size.1)?;
        self.overlay.resize(&self.gpu.device, size.0, size.1);
        self.compositor
            .set_overlay(&self.gpu.device, self.overlay.view());
        Ok(())
    }

    /// Draws to another canvas, on the same device: the canvas of an export
    /// after the canvas of the preview. TS §19.2 has no such call, and the
    /// session of TS §19.2 attaches a canvas more than once.
    pub fn set_canvas(
        &mut self,
        canvas: web_sys::OffscreenCanvas,
        width: Px,
        height: Px,
    ) -> Result<(), RenderError> {
        let before = self.gpu.size();
        self.gpu.set_canvas(canvas, width.get(), height.get())?;
        if before != self.gpu.size() {
            self.overlay
                .resize(&self.gpu.device, width.get(), height.get());
            self.compositor
                .set_overlay(&self.gpu.device, self.overlay.view());
        }
        Ok(())
    }

    /// Draws `list` in its order: its video, the part `src` of the frame of
    /// its slot turned by the clip's rotation, in `dst`, black around it;
    /// then every command after it, with glyphs set in `fonts`. The result is
    /// on the canvas. `src` is in display pixels of the source (TS §19.7). A
    /// list V4 does not draw is `Unsupported` (D-133).
    pub fn render(
        &mut self,
        sources: &VideoSources<'_>,
        list: &DisplayList,
        fonts: &FontSet,
    ) -> Result<(), RenderError> {
        let (src, dst, size) = self.video_of(list)?;
        let (frame, rotation) = sources.main;
        self.video.load(
            &self.gpu.device,
            &self.gpu.queue,
            frame,
            src,
            rotation,
            (dst, size),
        )?;
        let drawn = self.draw(list, fonts)?;
        self.gpu.present(drawn);
        Ok(())
    }

    /// The part of the source and the rectangle of the list's video, on a
    /// device that is not lost, with the target's size.
    fn video_of(&self, list: &DisplayList) -> Result<(CropRect, Rect, (u32, u32)), RenderError> {
        if self.gpu.is_lost() {
            return Err(RenderError::DeviceLost);
        }
        let size = self.gpu.size();
        let (src, dst) = composite::video_of(list, size)?;
        Ok((src, dst, size))
    }

    /// The rest of a frame, once its source is loaded: the overlay of the
    /// commands after the video, then one pass to the target.
    fn draw(&mut self, list: &DisplayList, fonts: &FontSet) -> Result<Frame, RenderError> {
        // The video is the first command; a frame with nothing over it costs
        // no overlay pass.
        let over = list.cmds.get(1..).unwrap_or_default();
        let with_overlay = !over.is_empty();
        if with_overlay {
            self.overlay
                .render(&self.gpu.device, &self.gpu.queue, over, fonts)?;
        }
        let frame = self.gpu.frame()?;
        self.compositor
            .draw(&self.gpu, frame.texture(), &self.video, with_overlay);
        Ok(frame)
    }
}

/// The renderer without a browser (D-134): on a software adapter, from an
/// image in memory, to an image in memory. Every step but the source and the
/// target is `render`'s.
#[cfg(not(target_arch = "wasm32"))]
impl Renderer {
    /// `NoAdapter` when the machine has no software adapter.
    pub async fn new_headless(width: Px, height: Px) -> Result<Renderer, RenderError> {
        Renderer::on(Gpu::new_headless(width.get(), height.get(), dx12_compiler()).await?)
    }

    /// What the adapter says it is: its name, its backend, and whether it is
    /// a software one (`DeviceType::Cpu`).
    pub fn adapter_info(&self) -> wgpu::AdapterInfo {
        self.gpu.adapter_info()
    }

    /// `render` with `source` as the main video, turned by `rotation`; the
    /// frame drawn, read back.
    pub fn render_to_image(
        &mut self,
        source: &RgbaImage,
        rotation: Rotation,
        list: &DisplayList,
        fonts: &FontSet,
    ) -> Result<RgbaImage, RenderError> {
        let (src, dst, size) = self.video_of(list)?;
        self.video.load_image(
            &self.gpu.device,
            &self.gpu.queue,
            source,
            src,
            rotation,
            (dst, size),
        )?;
        let drawn = self.draw(list, fonts)?;
        self.gpu.read(&drawn)
    }
}

/// The DX12 shader compiler of a headless renderer: wgpu's own variable
/// `WGPU_DX12_COMPILER` when it is set, else the newest Windows SDK's
/// `dxcompiler.dll` when there is one, else wgpu's choice. WARP draws the
/// overlay only with DXC: with FXC, Vello's shaders crash it (v4changelog
/// known issue 3, Entry 11). No other backend reads it.
#[cfg(not(target_arch = "wasm32"))]
fn dx12_compiler() -> wgpu::Dx12Compiler {
    if let Some(chosen) = wgpu::Dx12Compiler::from_env() {
        return chosen;
    }
    windows_sdk_dxc().map_or(wgpu::Dx12Compiler::Auto, |dxc_path| {
        wgpu::Dx12Compiler::DynamicDxc { dxc_path }
    })
}

/// `%ProgramFiles(x86)%/Windows Kits/10/bin/<version>/x64/dxcompiler.dll` of the highest
/// version. Off Windows there is no `ProgramFiles(x86)`, and so none.
#[cfg(not(target_arch = "wasm32"))]
fn windows_sdk_dxc() -> Option<String> {
    let root = std::path::PathBuf::from(std::env::var_os("ProgramFiles(x86)")?)
        .join("Windows Kits")
        .join("10")
        .join("bin");
    let version = |name: &str| -> Option<Vec<u32>> {
        name.split('.').map(|part| part.parse().ok()).collect()
    };
    let mut found: Vec<(Vec<u32>, std::path::PathBuf)> = std::fs::read_dir(root)
        .ok()?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let number = version(entry.file_name().to_str()?)?;
            let dll = entry.path().join("x64").join("dxcompiler.dll");
            dll.is_file().then_some((number, dll))
        })
        .collect();
    found.sort();
    found.pop()?.1.to_str().map(str::to_owned)
}
