//! Asheron's Call 2 static particle sources: the startup particle systems of a landblock's placed
//! entities and procedural scenery. Retail creates a `CPhysicsObj` per entity, whose
//! `PHYSICSOBJREP::InitFXManager` fills the FX table and `ApplyStartupFX` starts each startup
//! particle system at its attachment (`ac2_assets::resolve_ac2_visual_particles`).
//! Client-only grass frills start no FX.

use std::collections::HashMap;
use std::sync::Arc;

use ac2_core::{CellId, DataId};
use ac2_math::{Frame, Vec3};
use ac2_assets::{resolve_ac2_visual_particles, Ac2VisualParticles};
use ac2_db::ResourceResult;

use crate::Ac2WorldSource;

/// One startup particle system placed in the world.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ac2ParticleSource {
    /// The `PSDesc` (`0x39`) this system runs. A node naming several picks one per source, as
    /// `FXNode::GetParticleID` picks one per firing.
    pub particle_desc: DataId,
    /// System position (`CPartArray::QueueParticleSystems`): the entity frame composed with the
    /// attachment placement, in world AC axes.
    pub world_frame: Frame,
    /// Entity scale (`m_vObjectScale`).
    pub object_scale: Vec3,
    /// Visual `m_particle_scale` (`CPhysicsObj::GetParticleScale`).
    pub particle_scale: f32,
    /// Stable per-source identity for deterministic emission randomness.
    pub seed: u32,
}

/// The startup particle systems of one AC2 landblock's entities.
pub fn ac2_block_particle_sources(
    world: &Ac2WorldSource,
    block_cell_id: CellId,
) -> ResourceResult<Arc<[Ac2ParticleSource]>> {
    let markers = world.entity_markers(block_cell_id)?;
    let store = world.resource_store();
    let mut visuals = HashMap::<DataId, Option<Arc<Ac2VisualParticles>>>::new();
    let mut sources = Vec::new();
    for (index, marker) in markers
        .iter()
        .enumerate()
        .filter(|(_, marker)| !marker.frill())
    {
        let particles = match visuals.get(&marker.visual()) {
            Some(cached) => cached.clone(),
            None => {
                let resolved = resolve_ac2_visual_particles(store, marker.visual())?.map(Arc::new);
                visuals.insert(marker.visual(), resolved.clone());
                resolved
            }
        };
        let Some(particles) = particles else {
            continue;
        };
        let frame = marker.world_frame();
        for (slot, attachment) in particles.attachments.iter().enumerate() {
            let seed = mix(block_cell_id.raw() ^ mix(index as u32) ^ mix(slot as u32 + 0x9E37));
            let pick = seed as usize % attachment.particle_descs.len().max(1);
            let Some(particle_desc) = attachment.particle_descs.get(pick).copied() else {
                continue;
            };
            let placement = attachment.placement;
            sources.push(Ac2ParticleSource {
                particle_desc,
                world_frame: Frame::new(
                    frame.local_to_global_point(placement.offset * marker.scale()),
                    (frame.orientation * placement.rotation).normalize(),
                ),
                object_scale: marker.scale(),
                particle_scale: particles.particle_scale,
                seed,
            });
        }
    }
    Ok(Arc::from(sources))
}

/// Landblocks whose square lies within `radius` metres of `center` (world AC axes).
pub fn ac2_blocks_within(world: &Ac2WorldSource, center: Vec3, radius: f32) -> Vec<CellId> {
    let block = world.land_lattice().cell_size() * crate::LAND_BLOCK_CELLS_PER_SIDE as f32;
    let range = |value: f32| {
        let low = ((value - radius) / block).floor().max(0.0) as u32;
        let high = ((value + radius) / block).floor().clamp(0.0, 254.0) as u32;
        low..=high.max(low)
    };
    let mut blocks = Vec::new();
    for x in range(center.x).filter(|x| *x <= 254) {
        for y in range(center.y).filter(|y| *y <= 254) {
            let nearest = Vec3::new(
                center.x.clamp(x as f32 * block, (x + 1) as f32 * block),
                center.y.clamp(y as f32 * block, (y + 1) as f32 * block),
                center.z,
            );
            if nearest.distance(center) <= radius {
                blocks.push(CellId::new((x << 24) | (y << 16) | 0xFFFF));
            }
        }
    }
    blocks
}

fn mix(value: u32) -> u32 {
    let mut x = value.wrapping_mul(0x9E37_79B9) ^ 0x85EB_CA6B;
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^ (x >> 13)
}
