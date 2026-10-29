//! The breaking water on the near landscape's shores (`LScapeTerrainMesh::GenerateShore`, made
//! again when the near blocks change; `DrawShore(true)` at `LandscapeShoreEffects` 2), drawn with
//! the shore material through the frame's particle batches.

use std::sync::Arc;

use ac2_core::Random;

use super::loader::BlockKey;
use super::{ParticleBatch, WorldScene};
use crate::particles::ParticleGeometry;
use crate::terrain::generate_shore;

/// `DBObj::GetByEnum(0x40000017, ..)`: the shore's material.
pub(crate) const SHORE_MATERIAL_ENUM: u32 = 0x4000_0017;

/// The strips made for one set of near blocks around one centre.
pub(super) struct ShoreCache {
    blocks: Vec<BlockKey>,
    centre: Option<BlockKey>,
    geometry: Arc<ParticleGeometry>,
}

impl WorldScene {
    /// The frame's shore batch over the near terrain `blocks`, made again when they or the centre
    /// changed; none below `LandscapeShoreEffects` 2 or before the material loads.
    pub(super) fn shore_batch(&mut self, blocks: &[BlockKey]) -> Option<ParticleBatch> {
        if self.shore_effects < 2 {
            return None;
        }
        let material = self.shore_material.filter(|m| m.raw() != 0)?;
        let stale = self
            .shore
            .as_ref()
            .is_none_or(|cache| cache.blocks != blocks || cache.centre != self.center);
        if stale {
            let near: Vec<_> = blocks
                .iter()
                .filter_map(|key| {
                    let heights = self.heights.get(key)?;
                    Some((heights.water?, heights.all_triangles()))
                })
                .collect();
            let centre = self
                .center
                .and_then(|key| self.heights.get(&key))
                .and_then(|heights| heights.water);
            let shore = generate_shore(&near, centre, &mut Random::default());
            self.shore = Some(ShoreCache {
                blocks: blocks.to_vec(),
                centre: self.center,
                geometry: Arc::new(shore.breaking_waves()),
            });
        }
        let geometry = self.shore.as_ref()?.geometry.as_ref().clone();
        (!geometry.is_empty()).then_some(ParticleBatch { material, geometry })
    }
}
