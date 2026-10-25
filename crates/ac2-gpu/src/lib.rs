#![forbid(unsafe_code)]

//! A small wgpu backend for `ac2-render`, modelled on AC2's own D3D pipeline: textures uploaded
//! once per `RenderTexture`, a near-landscape pass whose layers blend the way
//! `LScapeMaterialInfo::CombineMaterials` stacks them, and an offscreen target that can be read
//! back for headless checks. Colour math stays in gamma space as the retail D3D blender did.

mod frame;
mod object;
mod object_draw;
mod object_state;
mod pixel_program;
mod scene;
mod screen_quad;
mod terrain;
mod texture;
mod ui;

pub use frame::{acquire_frame, render_to, FrameTarget, COLOR_FORMAT, DEPTH_FORMAT};
pub use object::{DrawGroup, GpuObjectModel, ObjectInstance, ObjectRenderer};
pub use scene::{SceneObject, SceneRenderer};
pub use screen_quad::ScreenQuad;
pub use terrain::{GpuFarTerrain, TerrainBlock, TerrainRenderer};
pub use texture::{TextureCache, TextureKey};
pub use ui::{UiLayer, UiOverlay};

/// The device and queue every other object is created on.
pub struct Gpu {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub adapter: wgpu::Adapter,
    pub instance: wgpu::Instance,
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
            let (device, queue) = adapter
                .request_device(&wgpu::DeviceDescriptor {
                    label: Some("ac2-gpu"),
                    ..Default::default()
                })
                .await
                .map_err(GpuError::Device)?;
            Ok(Self {
                device,
                queue,
                adapter,
                instance,
            })
        })
    }

    /// An offscreen device on the default backends.
    pub fn headless() -> Result<Self, GpuError> {
        Self::new(None, wgpu::Instance::default())
    }
}
