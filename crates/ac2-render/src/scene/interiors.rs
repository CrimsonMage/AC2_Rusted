//! The interior cells a landblock shows to the outside (`CLandBlockInfo`'s outside stab list),
//! as `WorldRender` draws visible env cells from outdoors: each cell mesh with its baked static
//! light and up to four point lights, under the block's inside environment.

use ac2_core::CellId;
use ac2_db::ResourceStore;
use ac2_math::Vec3;
use ac2_world::Ac2WorldSource;

use crate::object::{
    load_cell_model, object_point_lights, CellLighting, ObjectEnvironment, ObjectModel, PointLight,
    RenderSettings, SceneLight,
};

/// `CLandBlockInfo::m_InteriorFogType` linear.
const FOG_LINEAR: u32 = 0x4000_0002;

/// One block's visible interiors.
#[derive(Debug, Clone)]
pub struct BlockInteriors {
    pub environment: ObjectEnvironment,
    /// Each cell's model (in world space) and the point lights cached for it.
    pub cells: Vec<(ObjectModel, Vec<PointLight>)>,
}

/// Loads the outside-visible cells of `block`, or `None` when it shows none.
pub fn load_block_interiors(
    world: &Ac2WorldSource,
    store: &ResourceStore,
    block: CellId,
    settings: &RenderSettings,
) -> Option<BlockInteriors> {
    let interior = world.landblock_interior(block).ok().flatten()?;
    if interior.outside_stab_cells.is_empty() {
        return None;
    }
    let lights = world.light_info(block).ok().flatten();
    let ambient = interior.ambient.unwrap_or([0.0; 3]);
    let lighting = CellLighting {
        light_info: lights.as_deref(),
        interior_ambient: ambient,
    };
    let block_origin = world.ac2_block_origin(block);
    let scene_lights: Vec<SceneLight> = lights
        .iter()
        .flat_map(|info| info.lights.iter())
        .map(|light| SceneLight {
            light: light.clone(),
            origin: block_origin + light.origin,
            indoor: light.cell.raw() & 0xFFFF >= 0x100,
        })
        .collect();
    let mut cells = Vec::new();
    for id in interior.outside_stab_cells.iter() {
        let Some(cell) = world.env_cell(*id).ok().flatten() else {
            continue;
        };
        let frame = world.env_cell_world_frame(&cell);
        let Some(model) = load_cell_model(store, &cell, frame, lighting, settings)
            .ok()
            .flatten()
        else {
            continue;
        };
        // `CEnvCell::DrawCellMesh`: lights cached at the cell for 1.5 x its bounding radius.
        let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
        for vertex in model.parts.iter().flat_map(|part| part.vertices.iter()) {
            lo = lo.min(Vec3::from_array(vertex.position));
            hi = hi.max(Vec3::from_array(vertex.position));
        }
        let radius = (hi - lo).length() * 0.5 * 1.5;
        let point_lights = object_point_lights(&scene_lights, frame.origin, radius, true);
        cells.push((model, point_lights));
    }
    let fog = (interior.fog_type == Some(FOG_LINEAR)).then(|| {
        (
            interior.fog_color.unwrap_or([0.0; 3]),
            (
                interior.fog_near.unwrap_or(0.0),
                interior.fog_far.unwrap_or(1000.0),
            ),
        )
    });
    Some(BlockInteriors {
        environment: ObjectEnvironment::inside(ambient, fog),
        cells,
    })
}
