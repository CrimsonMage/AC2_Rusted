//! The startup lights of a landblock's placed entities. Retail makes a `CPhysicsObj` of each
//! entity (lamp posts, braziers, camp fires), and `CPhysicsObj::InitObjectEnd` runs
//! `PHYSICSOBJREP::ApplyStartupFX`: every startup FX node that carries a light places it through
//! `CPartArray::CreateLightSource`, in the object's frame, whether or not the node also starts a
//! particle system. Those lights are dynamic ones of the scene (`SceneTool::m_aDynamicLights`),
//! so they light what moves near them and pool on the ground like any FX light.

use std::collections::HashMap;
use std::sync::Arc;

use ac2_assets::{resolve_ac2_visual_fx_particles, Ac2FxLight, Ac2VisualFxParticles};
use ac2_core::{CellId, DataId};
use ac2_math::Vec3;
use ac2_world::Ac2WorldSource;

use crate::fx_particles::startup_light_starts;

/// One startup light of a placed entity, in world AC axes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EntityLight {
    pub light: Ac2FxLight,
    /// Where the light stands: the entity's frame times the node's offset.
    pub origin: Vec3,
}

/// The startup lights of `block`'s entities (grass frills start no FX).
pub fn block_entity_lights(world: &Ac2WorldSource, block: CellId) -> Vec<EntityLight> {
    let Ok(markers) = world.entity_markers(block) else {
        return Vec::new();
    };
    let store = world.resource_store();
    let mut tables = HashMap::<DataId, Option<Arc<Ac2VisualFxParticles>>>::new();
    let mut lights = Vec::new();
    for marker in markers.iter().filter(|marker| !marker.frill()) {
        let table = tables.entry(marker.visual()).or_insert_with(|| {
            resolve_ac2_visual_fx_particles(store, marker.visual())
                .ok()
                .flatten()
                .map(Arc::new)
        });
        let Some(table) = table else {
            continue;
        };
        let frame = marker.world_frame();
        lights.extend(
            startup_light_starts(0, table)
                .into_iter()
                .map(|start| EntityLight {
                    light: start.light,
                    origin: frame.local_to_global_point(start.light.offset),
                }),
        );
    }
    lights
}
