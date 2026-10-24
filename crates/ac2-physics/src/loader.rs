//! Loading a body's shape and the resident terrain from the AC2 DATs.

use ac2_assets::{resolve_ac2_visual_parts, Ac2SetupPhysics};
use ac2_core::{CellId, DataId};
use ac2_dat::DatFileKind;
use ac2_db::{ResourceResult, ResourceStore, TypedResourceRef};
use ac2_world::Ac2WorldSource;

use crate::physics_obj::BodyShape;
use crate::terrain::Terrain;
use crate::vector::{Sphere, Vector};

/// The collision shape of the body a `VisualDesc` (or bare `CSetup`) places: its root Setup's
/// `CSphere`s (`CPartArray::GetSphere`), height and step heights, scaled by the visual's root
/// z scale (`CPhysicsObj::GetTotalScale`).
pub fn load_body_shape(store: &ResourceStore, visual: DataId) -> ResourceResult<Option<BodyShape>> {
    let Some((setup, root)) = resolve_ac2_visual_parts(store, visual)?.first().copied() else {
        return Ok(None);
    };
    let Some(physics) = store.load_cached_packed::<Ac2SetupPhysics>(TypedResourceRef::new(
        DatFileKind::Portal,
        setup,
        0,
    ))?
    else {
        return Ok(None);
    };
    let scale = if root.scale.z.is_finite() && root.scale.z > 0.0 {
        root.scale.z
    } else {
        1.0
    };
    Ok(Some(BodyShape {
        spheres: physics
            .spheres
            .iter()
            .map(|sphere| Sphere::new(Vector::from(sphere.origin), sphere.radius))
            .collect(),
        height: physics.height,
        step_up_height: physics.step_up_height,
        step_down_height: physics.step_down_height,
        scale,
        allow_free_heading: physics.allow_free_heading,
    }))
}

/// Makes the landblock holding `block_cell_id` resident with its water plane; returns whether
/// the block exists.
pub fn load_terrain_block(
    world: &Ac2WorldSource,
    terrain: &mut Terrain,
    block_cell_id: CellId,
) -> ResourceResult<bool> {
    let Some(block) = world.load_landblock(block_cell_id)? else {
        return Ok(false);
    };
    terrain.insert_block(block_cell_id, &block);
    let water = world.water_surface_height(block_cell_id)?;
    terrain.set_block_water(block_cell_id.raw(), water);
    Ok(true)
}
