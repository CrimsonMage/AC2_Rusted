// One `unsafe` call, `Gpu::load_pipeline_cache`, is allowed where it is made.
#![deny(unsafe_code)]

//! A small wgpu backend for `ac2-render`, modelled on AC2's own D3D pipeline: textures uploaded
//! once per `RenderTexture`, a near-landscape pass whose layers blend the way
//! `LScapeMaterialInfo::CombineMaterials` stacks them, and an offscreen target that can be read
//! back for headless checks. Colour math stays in gamma space as the retail D3D blender did.

mod ao;
mod frame;
mod object;
mod object_draw;
mod object_pipelines;
mod object_state;
mod pixel_program;
mod portal_fader;
mod post;
mod quality;
mod scene;
mod screen_quad;
pub mod screenshot;
mod shading;
mod shadow_map;
mod sky;
mod terrain;
mod texture;
mod timing;
mod tonemap;
mod ui;
mod ui_scenes;

pub use ac2_render::scene::SceneObject;
pub use ao::AoCamera;
pub use frame::{acquire_frame, render_to, FrameTarget, COLOR_FORMAT, DEPTH_FORMAT};
pub use object::{sky_backdrop, DrawGroup, GpuObjectModel, ObjectInstance, ObjectRenderer};
pub use portal_fader::PortalFader;
pub use quality::{quality_level_for, suggested_quality_level};
pub use scene::SceneRenderer;
pub use screen_quad::ScreenQuad;
pub use shading::srgb_to_linear;
pub use shadow_map::{StaticShadowMap, SHADOW_MAP_FORMAT, SHADOW_MAP_SIZE};
pub use sky::{SkyLook, SkyRenderer, DEFAULT_CLOUD_COVER};
pub use terrain::{GpuFarTerrain, TerrainBlock, TerrainRenderer};
pub use texture::{uploaded_texture_bytes, TextureCache, TextureKey};
pub use tonemap::{HdrTarget, ScenePresenter, ToneMap, HDR_FORMAT};
pub use ui::{UiLayer, UiOverlay};
pub use ui_scenes::UiSceneRenderers;

/// The device and queue every other object is created on.
pub struct Gpu {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub adapter: wgpu::Adapter,
    pub instance: wgpu::Instance,
    /// The driver's compiled pipelines, kept across runs (`load_pipeline_cache`), when the
    /// device supports it (Vulkan).
    pipeline_cache: Option<wgpu::PipelineCache>,
    /// Per-pass GPU timing, on a device made by `headless_timed`.
    pass_timer: Option<timing::PassTimer>,
}

#[derive(Debug)]
pub enum GpuError {
    NoAdapter(wgpu::RequestAdapterError),
    Device(wgpu::RequestDeviceError),
}

impl std::fmt::Display for GpuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoAdapter(error) => write!(f, "no GPU adapter: {error}"),
            Self::Device(error) => write!(f, "GPU device request failed: {error}"),
        }
    }
}

impl std::error::Error for GpuError {}

impl Gpu {
    /// A device for offscreen rendering, or for a surface made from `instance` afterwards.
    pub fn new(
        compatible_surface: Option<&wgpu::Surface<'_>>,
        instance: wgpu::Instance,
    ) -> Result<Self, GpuError> {
        Self::create(compatible_surface, instance, false)
    }

    /// `new`, with per-pass GPU timing when `pass_timing` and the adapter has timestamp queries.
    /// Only measuring tools ask for it: the feature alone costs the NVIDIA driver about 130 MB of
    /// resident memory, queries or not.
    fn create(
        compatible_surface: Option<&wgpu::Surface<'_>>,
        instance: wgpu::Instance,
        pass_timing: bool,
    ) -> Result<Self, GpuError> {
        pollster::block_on(async {
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface,
                    force_fallback_adapter: false,
                    apply_limit_buckets: false,
                })
                .await
                .map_err(GpuError::NoAdapter)?;
            // A pipeline cache where the driver keeps one; BC textures where the GPU samples them
            // (the scene then loads DXT levels as blocks, `ac2_render::set_gpu_block_compression`);
            // timestamp queries when pass timing was asked for.
            let timing = if pass_timing {
                wgpu::Features::TIMESTAMP_QUERY
            } else {
                wgpu::Features::empty()
            };
            let required_features = adapter.features()
                & (wgpu::Features::PIPELINE_CACHE
                    | wgpu::Features::TEXTURE_COMPRESSION_BC
                    | timing);
            let (device, queue) = adapter
                .request_device(&wgpu::DeviceDescriptor {
                    label: Some("ac2-gpu"),
                    required_features,
                    ..Default::default()
                })
                .await
                .map_err(GpuError::Device)?;
            let pass_timer = required_features
                .contains(wgpu::Features::TIMESTAMP_QUERY)
                .then(|| timing::PassTimer::new(&device, &queue));
            Ok(Self {
                device,
                queue,
                adapter,
                instance,
                pipeline_cache: None,
                pass_timer,
            })
        })
    }

    /// Submits the buffer and texture writes queued since the last submit (`Queue::write_buffer`,
    /// `write_texture`) with no commands. Their staging memory is held until a submit, so a frame
    /// that updates the UI but draws nothing (the window is occluded or its surface is lost) must
    /// flush them, or they pile up every frame.
    pub fn flush_writes(&self) {
        self.queue.submit(std::iter::empty());
    }

    /// Whether the device samples BC1-BC3 textures.
    pub fn block_compression(&self) -> bool {
        self.device
            .features()
            .contains(wgpu::Features::TEXTURE_COMPRESSION_BC)
    }

    /// The timestamp writes for a render pass reported under `label`, while timing is on.
    pub fn pass_timestamps(
        &self,
        label: &'static str,
    ) -> Option<wgpu::RenderPassTimestampWrites<'_>> {
        self.pass_timer.as_ref()?.writes(label)
    }

    /// The GPU milliseconds of each pass label since the last call, summed per label in
    /// first-use order (empty while timing is off). Waits for the GPU.
    pub fn read_pass_times(&self) -> Vec<(&'static str, f64)> {
        self.pass_timer
            .as_ref()
            .map_or_else(Vec::new, |timer| timer.read(&self.device, &self.queue))
    }

    /// GPU memory the device's allocator holds, `(allocated, reserved)` bytes, on the backends
    /// that report it (Vulkan; `None` on GL and Metal).
    pub fn allocator_bytes(&self) -> Option<(u64, u64)> {
        let report = self.device.generate_allocator_report()?;
        Some((report.total_allocated_bytes, report.total_reserved_bytes))
    }

    /// The pipeline cache new pipelines compile through, once loaded.
    pub fn pipeline_cache(&self) -> Option<&wgpu::PipelineCache> {
        self.pipeline_cache.as_ref()
    }

    /// The cache file for this adapter and driver in `dir`, when the device keeps a cache.
    fn pipeline_cache_file(&self, dir: &std::path::Path) -> Option<std::path::PathBuf> {
        if !self
            .device
            .features()
            .contains(wgpu::Features::PIPELINE_CACHE)
        {
            return None;
        }
        let key = wgpu::util::pipeline_cache_key(&self.adapter.get_info())?;
        Some(dir.join(key))
    }

    /// Starts the device's pipeline cache from `dir`'s file for this adapter (empty when there is
    /// none, or when the driver rejects it). Call before the renderers are made, so their
    /// pipelines compile through it.
    #[allow(unsafe_code)]
    pub fn load_pipeline_cache(&mut self, dir: &std::path::Path) {
        let Some(file) = self.pipeline_cache_file(dir) else {
            return;
        };
        let data = std::fs::read(&file).ok();
        // SAFETY: `data` is only what `save_pipeline_cache` wrote from `PipelineCache::get_data`,
        // in a file named by `pipeline_cache_key` for this adapter; wgpu checks the header (its
        // version, the adapter and a checksum) and with `fallback` starts empty on a mismatch.
        let cache = unsafe {
            self.device
                .create_pipeline_cache(&wgpu::PipelineCacheDescriptor {
                    label: Some("ac2 pipelines"),
                    data: data.as_deref(),
                    fallback: true,
                })
        };
        self.pipeline_cache = Some(cache);
    }

    /// Writes the pipeline cache to `dir` for the next run (through a temporary file, so a
    /// crash mid-write leaves the old one).
    pub fn save_pipeline_cache(&self, dir: &std::path::Path) -> std::io::Result<()> {
        let (Some(cache), Some(file)) = (&self.pipeline_cache, self.pipeline_cache_file(dir))
        else {
            return Ok(());
        };
        let Some(data) = cache.get_data() else {
            return Ok(());
        };
        std::fs::create_dir_all(dir)?;
        let temporary = file.with_extension("tmp");
        std::fs::write(&temporary, data)?;
        std::fs::rename(temporary, file)
    }

    /// An offscreen device on the default backends.
    pub fn headless() -> Result<Self, GpuError> {
        Self::new(None, wgpu::Instance::default())
    }

    /// `headless` with per-pass GPU timing where the adapter has timestamp queries (benches):
    /// `read_pass_times` must then be called every frame.
    pub fn headless_timed() -> Result<Self, GpuError> {
        Self::create(None, wgpu::Instance::default(), true)
    }
}
