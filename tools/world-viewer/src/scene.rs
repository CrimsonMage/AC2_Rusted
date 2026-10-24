//! What the viewer draws: the near landscape of the blocks around the camera, lit by the region
//! sky at the viewer's time of day.

use std::collections::HashMap;
use std::sync::Arc;

use ac2_core::CellId;
use ac2_gpu::{Gpu, TerrainBlock, TerrainRenderer, TextureKey};
use ac2_math::Vec3;
use ac2_render::terrain::build_terrain_mesh;
use ac2_render::{load_material_image, load_texture_image, SceneLighting};
use ac2_world::Ac2WorldSource;

/// `GameTime` day length in seconds for the Legions region.
pub const DAY_SECONDS: f64 = 7347.0;

pub struct Scene {
    pub world: Arc<Ac2WorldSource>,
    pub renderer: TerrainRenderer,
    blocks: HashMap<(u8, u8), Option<TerrainBlock>>,
    pub radius: i32,
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
            world,
            renderer: TerrainRenderer::new(gpu, format),
            blocks: HashMap::new(),
            radius,
            time_of_day: 0.0,
        }
    }

    /// The landblock under a world AC position.
    pub fn block_at(&self, position: Vec3) -> (u8, u8) {
        let size = self.world.block_size();
        let clamp = |v: f32| (v / size).floor().clamp(0.0, 254.0) as u8;
        (clamp(position.x), clamp(position.y))
    }

    /// Highest terrain vertex of a block, for placing the camera above it.
    pub fn block_top(&self, block: (u8, u8)) -> f32 {
        self.world
            .load_landblock(block_id(block))
            .ok()
            .flatten()
            .map_or(0.0, |landblock| {
                landblock.height_grid().into_iter().fold(0.0, f32::max)
            })
    }

    /// Loads the blocks within `radius` of `center` and drops the rest
    /// (`LScapeTerrainMesh::UpdateNearTerrain` over the near ring).
    pub fn update_residency(&mut self, gpu: &Gpu, center: (u8, u8)) {
        let radius = self.radius;
        let near = |block: &(u8, u8)| {
            (i32::from(block.0) - i32::from(center.0)).abs() <= radius
                && (i32::from(block.1) - i32::from(center.1)).abs() <= radius
        };
        self.blocks.retain(|block, _| near(block));
        let store = self.world.resource_store().clone();
        let mut load = |key: TextureKey| match key {
            TextureKey::Material(did) => load_material_image(&store, did).ok().flatten(),
            TextureKey::Mask(did) => load_texture_image(&store, did).ok().flatten(),
        };
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                let (x, y) = (i32::from(center.0) + dx, i32::from(center.1) + dy);
                let (Ok(x), Ok(y)) = (u8::try_from(x), u8::try_from(y)) else {
                    continue;
                };
                if self.blocks.contains_key(&(x, y)) {
                    continue;
                }
                let id = block_id((x, y));
                let block = self
                    .world
                    .load_landblock(id)
                    .ok()
                    .flatten()
                    .map(|landblock| {
                        let mesh = build_terrain_mesh(id, &landblock, self.world.block_size());
                        self.renderer
                            .upload_block(gpu, &mesh, self.world.block_size(), &mut load)
                    });
                self.blocks.insert((x, y), block);
            }
        }
    }

    pub fn lighting(&self) -> SceneLighting {
        self.world
            .region()
            .and_then(|region| region.sample_sky_environment(self.time_of_day))
            .map(|sample| SceneLighting::from_sky(&sample))
            .unwrap_or_default()
    }

    /// Starts at the brightest daylight of the region's day.
    pub fn set_daytime(&mut self) {
        let Some(region) = self.world.region() else {
            return;
        };
        let brightness = |time: f64| {
            region
                .sample_sky_environment(time)
                .map(|sample| {
                    SceneLighting::from_sky(&sample)
                        .fog_color
                        .iter()
                        .sum::<f32>()
                })
                .unwrap_or(0.0)
        };
        self.time_of_day = (0..48)
            .map(|step| f64::from(step) * DAY_SECONDS / 48.0)
            .max_by(|a, b| brightness(*a).total_cmp(&brightness(*b)))
            .unwrap_or(0.0);
    }

    pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        self.renderer.draw(pass, self.blocks.values().flatten());
    }
}

pub fn block_id((x, y): (u8, u8)) -> CellId {
    CellId::new((u32::from(x) << 24) | (u32::from(y) << 16) | 1)
}
