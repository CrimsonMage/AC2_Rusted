//! The decals the FX project (`DecalEffectManager`): each one takes the ground under its origin
//! when it starts, from the resident blocks' terrain (`LScapeRender::GetTerrainGeometry` at the
//! nine polygons around the centre, `DecalEffect::ConstructLandscapeDecal`), then thinks and draws
//! each frame as its material's geometry, merged into the frame's particle batches.

use ac2_math::Vec3;
use ac2_world::land_block_size;

use super::loader::BlockKey;
use super::{ParticleBatch, WorldScene};
use crate::decal::{DecalSpec, GroundTriangle};

impl WorldScene {
    /// `DecalEffectManager::CreateDecalEffect` for an FX decal: its material loads off the frame
    /// thread (like a streak's), and the decal starts now on the resident ground under it.
    pub fn start_object_decal(&mut self, spec: DecalSpec) {
        self.request_material(spec.material);
        let ground = self.decal_ground(spec.origin);
        self.decals.create(spec, self.particles.now(), &ground);
    }

    /// The ground the decal at `origin` lands on: the two triangles of each of the nine polygons
    /// around it (the centre and its eight neighbours, one polygon apart), as
    /// `LScapeRender::GetTerrainGeometry` gives them. A polygon whose block is not resident gives
    /// nothing.
    pub(super) fn decal_ground(&self, origin: Vec3) -> Vec<GroundTriangle> {
        let block_size = land_block_size();
        let Some(pitch) = self
            .heights_at(origin.x, origin.y, block_size)
            .map(|h| h.polygon_size())
        else {
            return Vec::new();
        };
        let mut ground = Vec::new();
        for dy in [0.0, -pitch, pitch] {
            for dx in [0.0, -pitch, pitch] {
                let (x, y) = (origin.x + dx, origin.y + dy);
                if let Some(heights) = self.heights_at(x, y, block_size) {
                    if let Some(triangles) = heights.polygon_triangles(x, y) {
                        ground.extend(triangles);
                    }
                }
            }
        }
        ground
    }

    /// The resident terrain under world `(x, y)`, when its block is resident.
    fn heights_at(
        &self,
        x: f32,
        y: f32,
        block_size: f32,
    ) -> Option<&crate::terrain::TerrainHeights> {
        let bx = (x / block_size).floor();
        let by = (y / block_size).floor();
        if !(0.0..256.0).contains(&bx) || !(0.0..256.0).contains(&by) {
            return None;
        }
        let key: BlockKey = (bx as u8, by as u8);
        self.heights.get(&key).map(|heights| heights.as_ref())
    }

    /// The decals' geometry this frame, after they think: one batch per material.
    pub(super) fn decal_batches(&mut self) -> Vec<ParticleBatch> {
        self.decals.think(self.particles.now());
        self.decals
            .geometry()
            .into_iter()
            .map(|(material, geometry)| ParticleBatch { material, geometry })
            .collect()
    }
}
