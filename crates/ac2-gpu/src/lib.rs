// One `unsafe` call, `Gpu::load_pipeline_cache`, is allowed where it is made.
#![deny(unsafe_code)]

//! A small wgpu backend for `ac2-render`, modelled on AC2's own D3D pipeline: textures uploaded
//! once per `RenderTexture`, a near-landscape pass whose layers blend the way
//! `LScapeMaterialInfo::CombineMaterials` stacks them, and an offscreen target that can be read
//! back for headless checks. Colour math stays in gamma space as the retail D3D blender did.

mod frame;
mod object;
mod object_draw;
mod object_pipelines;
mod object_state;
mod pixel_program;
mod scene;
mod screen_quad;
mod shading;
mod shadow_map;
mod sky;
mod terrain;
mod texture;
mod tonemap;
mod ui;

pub use frame::{acquire_frame, render_to, FrameTarget, COLOR_FORMAT, DEPTH_FORMAT};
pub use object::{sky_backdrop, DrawGroup, GpuObjectModel, ObjectInstance, ObjectRenderer};
pub use scene::{SceneObject, SceneRenderer};
pub use screen_quad::ScreenQuad;
pub use shading::srgb_to_linear;
pub use shadow_map::{StaticShadowMap, SHADOW_MAP_FORMAT, SHADOW_MAP_SIZE};
pub use sky::{SkyLook, SkyRenderer, DEFAULT_CLOUD_COVER};
pub use terrain::{GpuFarTerrain, TerrainBlock, TerrainRenderer};
pub use texture::{TextureCache, TextureKey};
pub use tonemap::{HdrTarget, ScenePresenter, ToneMap, HDR_FORMAT};
pub use ui::{UiLayer, UiOverlay};

/// The device and queue every other object is created on.
pub struct Gpu {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub adapter: wgpu::Adapter,
    pub instance: wgpu::Instance,
    /// The driver's compiled pipelines, kept across runs (`load_pipeline_cache`), when the
    /// device supports it (Vulkan).
    pipeline_cache: Option<wgpu::PipelineCache>,
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
            let required_features = adapter.features() & wgpu::Features::PIPELINE_CACHE;
            let (device, queue) = adapter
                .request_device(&wgpu::DeviceDescriptor {
                    label: Some("ac2-gpu"),
                    required_features,
                    ..Default::default()
                })
                .await
                .map_err(GpuError::Device)?;
            Ok(Self {
                device,
                queue,
                adapter,
                instance,
                pipeline_cache: None,
            })
        })
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
}
