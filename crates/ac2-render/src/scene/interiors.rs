//! The interior cells a landblock shows to the outside (`CLandBlockInfo`'s outside stab list),
//! as `WorldRender` draws visible env cells from outdoors: each cell mesh with its baked static
//! light and up to four point lights, under the block's inside environment.

use std::collections::HashMap;
use std::sync::Arc;

use ac2_core::{CellId, DataId};
use ac2_db::ResourceStore;
use ac2_math::{BBox, Mat4, Vec3};
use ac2_world::{Ac2CellStruct, Ac2EnvCell, Ac2WorldSource};

use crate::interior_cull::{compute_cell_groups, BlockCull, CullCell, CullPortal};

use crate::object::{
    load_cell_model, object_point_lights, CellLighting, ObjectEnvironment, ObjectModel, PointLight,
    RenderSettings, SceneLight,
};

/// `CLandBlockInfo::m_InteriorFogType` linear.
const FOG_LINEAR: u32 = 0x4000_0002;
/// `CLandBlockInfo::m_InteriorFogType` none.
const FOG_NONE: u32 = 0x4000_0001;

/// One block's visible interiors.
#[derive(Debug, Clone)]
pub struct BlockInteriors {
    pub environment: ObjectEnvironment,
    /// Each cell's model (in world space) and the point lights cached for it.
    pub cells: Vec<(ObjectModel, Vec<PointLight>)>,
    /// The cells' static objects (`CEnvCell::InitStaticObjects`): visual, world matrix, and the
    /// point lights of the cell they stand in.
    pub objects: Vec<(DataId, Mat4, Vec<PointLight>)>,
    /// The env cell each of `cells` is, and the one each of `objects` stands in.
    pub cell_ids: Vec<u32>,
    pub object_cells: Vec<u32>,
    /// What decides which of them draw (`CEnvCell::CullCells`).
    pub cull: Arc<BlockCull>,
}

/// `CEnvCell::m_boundingBox`: the cell mesh's bounds turned into the scene's space.
fn cell_bounds(structure: &Ac2CellStruct, world: Mat4) -> BBox {
    let (lo, hi) = (structure.bounds_min, structure.bounds_max);
    let mut min = Vec3::splat(f32::MAX);
    let mut max = Vec3::splat(f32::MIN);
    for corner in 0..8 {
        let point = Vec3::new(
            if corner & 1 == 0 { lo.x } else { hi.x },
            if corner & 2 == 0 { lo.y } else { hi.y },
            if corner & 4 == 0 { lo.z } else { hi.z },
        );
        let world_point = world.transform_point3(point);
        min = min.min(world_point);
        max = max.max(world_point);
    }
    BBox::new(min, max)
}

/// The cull's view of one env cell: its portals' polygons from the cell mesh, its bounds and
/// frame in the scene.
fn cull_cell(cell: &Ac2EnvCell, structure: &Arc<Ac2CellStruct>, world: Mat4) -> CullCell {
    let portals = cell
        .portals
        .iter()
        .map(|portal| {
            let polygon = usize::try_from(portal.polygon)
                .ok()
                .and_then(|slot| structure.portal_polygons.get(slot))
                .and_then(|p| structure.polygons.get(usize::from(p.polygon)))
                .map(|polygon| {
                    polygon
                        .vertex_ids
                        .iter()
                        .filter_map(|&id| structure.positions.get(usize::from(id)).copied())
                        .collect()
                })
                .unwrap_or_default();
            CullPortal {
                other_cell: portal.other_cell.map(|id| id.raw()),
                polygon,
            }
        })
        .collect();
    CullCell {
        id: cell.cell_id.raw(),
        bbox: cell_bounds(structure, world),
        world,
        portals,
        no_cull: cell.no_cull,
        draw_sky: cell.draw_sky,
        shared_cells: cell.shared_cells.iter().map(|id| id.raw()).collect(),
        structure: Some(Arc::clone(structure)),
        inverse: world.inverse(),
    }
}

/// Loads the outside-visible cells of `block`, or `None` when it shows none.
pub fn load_block_interiors(
    world: &Ac2WorldSource,
    store: &ResourceStore,
    block: CellId,
    settings: &RenderSettings,
) -> Option<BlockInteriors> {
    let interior = world.landblock_interior(block).ok().flatten()?;
    // Every env cell of the block loads (`CLandBlockInfo::m_cells`); which of them draw is the
    // cull's decision each frame.
    interior.cell_ids(block).next()?;
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
    let mut objects = Vec::new();
    let mut cell_ids = Vec::new();
    let mut object_cells = Vec::new();
    let mut cull_cells = HashMap::new();
    for id in interior.cell_ids(block) {
        let Some(cell) = world.env_cell(id).ok().flatten() else {
            continue;
        };
        let frame = world.env_cell_world_frame(&cell);
        if let Some(structure) = world.cell_struct(cell.cell_mesh).ok().flatten() {
            let culled = cull_cell(&cell, &structure.arc(), frame.to_mat4());
            cull_cells.insert(culled.id, culled);
        }
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
        for marker in world.env_cell_markers(&cell) {
            let world_matrix = marker.world_frame().to_mat4() * Mat4::from_scale(marker.scale());
            objects.push((marker.visual(), world_matrix, point_lights.clone()));
            object_cells.push(cell.cell_id.raw());
        }
        cells.push((model, point_lights));
        cell_ids.push(cell.cell_id.raw());
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
    // `CEnvCell::CullCells`: the interior fog's far distance is the culling plane unless the
    // block has no interior fog (`CLandBlockInfo::m_InteriorFogType` 0x40000001).
    let culling_plane = match (interior.fog_type, interior.fog_far) {
        (Some(FOG_NONE), _) | (_, None) => -1.0,
        (_, Some(far)) => far,
    };
    let groups = compute_cell_groups(&cull_cells);
    let cull = BlockCull {
        cells: cull_cells,
        groups,
        outside_stab: interior
            .outside_stab_cells
            .iter()
            .map(|id| id.raw())
            .collect(),
        culling_plane,
    };
    Some(BlockInteriors {
        environment: ObjectEnvironment::inside(ambient, fog),
        cells,
        objects,
        cell_ids,
        object_cells,
        cull: Arc::new(cull),
    })
}
