//! What the viewer draws: the shared scene (`ac2_render::scene::WorldScene`, loading on its own
//! thread) around the camera's block, uploaded and drawn by `ac2_gpu::SceneRenderer`, lit by the
//! region sky at the viewer's time of day.

use std::sync::Arc;
use std::time::Duration;

use ac2_gpu::{Gpu, SceneRenderer};
use ac2_math::Vec3;
use ac2_render::scene::WorldScene;
use ac2_render::Camera;
use ac2_world::Ac2WorldSource;

pub use ac2_render::scene::DAY_SECONDS;

/// Frame-thread time a frame spends taking finished loads (`DBCache::UseTime`'s pump).
const LOAD_BUDGET: Duration = Duration::from_millis(4);

pub struct Scene {
    pub world: Arc<Ac2WorldSource>,
    scene: WorldScene,
    renderer: SceneRenderer,
    pub time_of_day: f64,
}

impl Scene {
    pub fn new(
        world: Arc<Ac2WorldSource>,
        gpu: &Gpu,
        format: wgpu::TextureFormat,
        radius: i32,
    ) -> Self {
        Self {
            scene: WorldScene::new(Arc::clone(&world), radius),
            renderer: SceneRenderer::new(gpu, format),
            world,
            time_of_day: 0.0,
        }
    }

    /// The landblock under a world AC position.
    pub fn block_at(&self, position: Vec3) -> (u8, u8) {
        let size = self.world.block_size();
        let clamp = |v: f32| (v / size).floor().clamp(0.0, 254.0) as u8;
        (clamp(position.x), clamp(position.y))
    }

    /// Highest terrain vertex of a block, for placing the camera above it (read once at
    /// start-up, before any frame).
    pub fn block_top(&self, block: (u8, u8)) -> f32 {
        let id = ac2_core::CellId::new((u32::from(block.0) << 24) | (u32::from(block.1) << 16) | 1);
        self.world
            .load_landblock(id)
            .ok()
            .flatten()
            .map_or(0.0, |landblock| {
                landblock.height_grid().into_iter().fold(0.0, f32::max)
            })
    }

    /// Centres the resident ring on `center` and uploads the loads that finished.
    pub fn update_residency(&mut self, gpu: &Gpu, center: (u8, u8)) {
        self.scene.set_center(center);
        let update = self.scene.poll(LOAD_BUDGET);
        self.renderer.apply(gpu, update);
    }

    /// Waits for every outstanding load, including the sky objects seen from `camera`
    /// (headless snapshots).
    pub fn settle(&mut self, gpu: &Gpu, camera: &Camera) {
        self.scene.time_of_day = self.time_of_day;
        let update = self.scene.wait();
        self.renderer.apply(gpu, update);
        self.scene.frame(camera, 16.0 / 9.0);
        let update = self.scene.wait();
        self.renderer.apply(gpu, update);
    }

    /// Starts at the brightest daylight of the region's day.
    pub fn set_daytime(&mut self) {
        self.scene.set_daytime();
        self.time_of_day = self.scene.time_of_day;
    }

    /// Lays out the frame; `time` is client time in seconds for animated materials.
    pub fn prepare(&mut self, gpu: &Gpu, camera: &Camera, aspect: f32, time: f64) {
        self.scene.time_of_day = self.time_of_day;
        let frame = self.scene.frame(camera, aspect);
        self.renderer.prepare(gpu, camera, aspect, time, frame);
    }

    /// Records the frame into a pass cleared to `SceneTool::backgroundColor` (black).
    pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        self.renderer.draw(pass);
    }
}
