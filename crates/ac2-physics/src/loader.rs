//! Loading a body's shape, the resident terrain and a landblock's static objects from the AC2
//! DATs.

use std::collections::HashMap;
use std::sync::Arc;

use ac2_assets::{
    resolve_ac2_visual_meshes, resolve_ac2_visual_parts, resolve_ac2_visual_scale, Ac2PhysicsMesh,
    Ac2SetupPhysics,
};
use ac2_core::{CellId, DataId};
use ac2_dat::DatFileKind;
use ac2_db::{ResourceResult, ResourceStore, TypedResourceRef};
use ac2_world::{Ac2CellBspNode, Ac2CellStruct, Ac2WorldSource};

use crate::bsp::{BspTree, Polygon};
use crate::envcell::{CellBsp, CellBspNode, CellPortal, CellStructure, EnvCell};
use crate::fpu::{ld, st};
use crate::frame::Frame;
use crate::landdefs::{adjust_to_outside, BLOCK_LENGTH};
use crate::objects::{ObjectShape, StaticObject};
use crate::physics_obj::BodyShape;
use crate::position::Position;
use crate::shapes::cylsphere::CylSphere;
use crate::terrain::Terrain;
use crate::vector::{Plane, Quaternion, Sphere, Vector};

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

/// The collision shape of a static object's visual: the physics BSP of every mesh it draws
/// (`CPartArray::FindObjCollisions` collides each part's `CStaticMesh::physics_bsp`) and the
/// root Setup's cylinders and spheres.
pub fn load_object_shape(
    store: &ResourceStore,
    visual: DataId,
) -> ResourceResult<Option<ObjectShape>> {
    let Some((setup, _)) = resolve_ac2_visual_parts(store, visual)?.first().copied() else {
        return Ok(None);
    };
    let mut bsps = Vec::new();
    for (mesh, _) in resolve_ac2_visual_meshes(store, visual)? {
        let physics = store.load_cached_packed::<Ac2PhysicsMesh>(TypedResourceRef::new(
            DatFileKind::Portal,
            mesh,
            0,
        ))?;
        if let Some(tree) = physics.as_deref().and_then(BspTree::from_mesh) {
            bsps.push(Arc::new(tree));
        }
    }
    let physics = store.load_cached_packed::<Ac2SetupPhysics>(TypedResourceRef::new(
        DatFileKind::Portal,
        setup,
        0,
    ))?;
    let (cylspheres, spheres) = physics
        .as_deref()
        .map_or((Vec::new(), Vec::new()), |setup| {
            (
                setup
                    .cylspheres
                    .iter()
                    .map(|c| CylSphere {
                        low: Vector::from(c.low_point),
                        height: c.height,
                        radius: c.radius,
                    })
                    .collect(),
                setup
                    .spheres
                    .iter()
                    .map(|s| Sphere::new(Vector::from(s.origin), s.radius))
                    .collect(),
            )
        });
    if bsps.is_empty() && cylspheres.is_empty() && spheres.is_empty() {
        return Ok(None);
    }
    Ok(Some(ObjectShape::new(bsps, cylspheres, spheres)))
}

/// `CLandBlock::CreateStaticObjects`: the colliding entities and scenery of one landblock as
/// static objects in the world (positions from block-relative frames through
/// `LandDefs::adjust_to_outside`). Returns how many were added.
pub fn load_block_objects(
    world: &Ac2WorldSource,
    terrain: &mut Terrain,
    block_cell_id: CellId,
) -> ResourceResult<usize> {
    let store = world.resource_store();
    let markers = world.entity_markers(block_cell_id)?;
    let block = block_cell_id.raw() & 0xFFFF_0000;
    let base = Vector::new(
        ((block >> 24) & 0xFF) as f32 * BLOCK_LENGTH,
        ((block >> 16) & 0xFF) as f32 * BLOCK_LENGTH,
        0.0,
    );
    let mut shapes: HashMap<u32, Option<Arc<ObjectShape>>> = HashMap::new();
    let mut added = 0;
    for marker in markers.iter().filter(|marker| marker.collides()) {
        let visual = marker.visual();
        let shape = match shapes.get(&visual.raw()) {
            Some(shape) => shape.clone(),
            None => {
                let shape = load_object_shape(store, visual)?.map(Arc::new);
                shapes.insert(visual.raw(), shape.clone());
                shape
            }
        };
        let Some(shape) = shape else {
            continue;
        };
        let frame = marker.world_frame();
        let mut origin = Vector::new(
            st(ld(frame.origin.x) - ld(base.x)),
            st(ld(frame.origin.y) - ld(base.y)),
            frame.origin.z,
        );
        let mut cell_id = block | 1;
        if !adjust_to_outside(&mut cell_id, &mut origin) {
            continue;
        }
        let q = frame.orientation;
        let position = Position::new(
            cell_id,
            Frame::new(origin, Quaternion::new(q.w, q.x, q.y, q.z)),
        );
        let vdesc = Vector::from(resolve_ac2_visual_scale(store, visual)?);
        let s = marker.scale();
        let scale = Vector::new(
            st(ld(vdesc.x) * ld(s.x)),
            st(ld(vdesc.y) * ld(s.y)),
            st(ld(vdesc.z) * ld(s.z)),
        );
        terrain.insert_object(StaticObject::new(position, scale, shape));
        added += 1;
    }
    Ok(added)
}

/// The collision structure of a `CCellMesh`: its physics BSP over the physics polygons, its cell
/// BSP and its portal polygons (`CCellMesh::StreamPack`).
fn cell_structure(structure: &Ac2CellStruct) -> CellStructure {
    let polygons: Vec<Polygon> = structure
        .polygons
        .iter()
        .map(|polygon| {
            Polygon::new(
                polygon
                    .vertex_ids
                    .iter()
                    .filter_map(|&id| structure.positions.get(usize::from(id)))
                    .map(|&p| Vector::from(p))
                    .collect(),
            )
        })
        .collect();
    let portal_polygons = structure
        .portal_polygons
        .iter()
        .filter_map(|portal| polygons.get(usize::from(portal.polygon)).cloned())
        .collect();
    let physics = structure
        .physics_bsp
        .as_deref()
        .map(|root| Arc::new(BspTree::from_cell(root, polygons)));
    let mut cell_bsp = CellBsp::default();
    if let Some(root) = structure.cell_bsp.as_deref() {
        push_cell_node(&mut cell_bsp, root);
    }
    CellStructure {
        physics,
        cell_bsp,
        portal_polygons,
    }
}

fn push_cell_node(tree: &mut CellBsp, node: &Ac2CellBspNode) -> usize {
    let index = tree.nodes.len();
    tree.nodes.push(CellBspNode {
        plane: Plane::new(Vector::from(node.plane.normal), node.plane.d),
        pos: None,
        neg: None,
    });
    let pos = node.pos.as_deref().map(|child| push_cell_node(tree, child));
    let neg = node.neg.as_deref().map(|child| push_cell_node(tree, child));
    tree.nodes[index].pos = pos;
    tree.nodes[index].neg = neg;
    index
}

/// A landblock's interior cells (`CEnvCell`s `xxyy0100` and up) with their static entities
/// (`CEnvCell::InitStaticObjects`), then the land cells' touching cells
/// (`CLandBlock::InitCellConnections`). Load the block's terrain first. Returns how many cells
/// became resident.
pub fn load_block_interiors(
    world: &Ac2WorldSource,
    terrain: &mut Terrain,
    block_cell_id: CellId,
) -> ResourceResult<usize> {
    let Some(interior) = world.landblock_interior(block_cell_id)? else {
        return Ok(0);
    };
    let store = world.resource_store();
    let mut structures: HashMap<u32, Arc<CellStructure>> = HashMap::new();
    let mut cells = Vec::new();
    for cell_id in interior.cell_ids(block_cell_id) {
        let Some(cell) = world.env_cell(cell_id)? else {
            continue;
        };
        let structure = match structures.get(&cell.cell_mesh.raw()) {
            Some(structure) => Arc::clone(structure),
            None => {
                let Some(source) = world.cell_struct(cell.cell_mesh)? else {
                    continue;
                };
                let structure = Arc::new(cell_structure(&source));
                structures.insert(cell.cell_mesh.raw(), Arc::clone(&structure));
                structure
            }
        };
        let portals = cell
            .portals
            .iter()
            .filter_map(|portal| {
                let polygon = usize::try_from(portal.polygon).ok()?;
                Some(CellPortal {
                    other_cell: portal.other_cell.map(|other| other.raw()),
                    polygon: structure.portal_polygons.get(polygon)?.clone(),
                })
            })
            .collect();
        let q = cell.frame.orientation;
        let position = Position::new(
            cell_id.raw(),
            Frame::new(
                Vector::from(cell.frame.origin),
                Quaternion::new(q.w, q.x, q.y, q.z),
            ),
        );
        terrain.insert_env_cell(EnvCell {
            cell_id: cell_id.raw(),
            position,
            structure,
            portals,
            seen_outside: cell.seen_outside,
            objects: Vec::new(),
        });
        cells.push(cell);
    }
    let origin = world.ac2_block_origin(block_cell_id);
    let mut shapes: HashMap<u32, Option<Arc<ObjectShape>>> = HashMap::new();
    for cell in &cells {
        for entity in world.env_cell_entities(cell)? {
            let shape = match shapes.get(&entity.visual.raw()) {
                Some(shape) => shape.clone(),
                None => {
                    let shape = load_object_shape(store, entity.visual)?.map(Arc::new);
                    shapes.insert(entity.visual.raw(), shape.clone());
                    shape
                }
            };
            let Some(shape) = shape else {
                continue;
            };
            let frame = entity.world_frame;
            let local = frame.origin - origin;
            let q = frame.orientation;
            let position = Position::new(
                cell.cell_id.raw(),
                Frame::new(Vector::from(local), Quaternion::new(q.w, q.x, q.y, q.z)),
            );
            let vdesc = Vector::from(resolve_ac2_visual_scale(store, entity.visual)?);
            let scale = Vector::new(
                st(ld(vdesc.x) * ld(entity.scale.x)),
                st(ld(vdesc.y) * ld(entity.scale.y)),
                st(ld(vdesc.z) * ld(entity.scale.z)),
            );
            terrain.insert_object(StaticObject::new(position, scale, shape));
        }
    }
    let stab: Vec<u32> = interior
        .outside_stab_cells
        .iter()
        .map(|id| id.raw())
        .collect();
    terrain.connect_block(block_cell_id.raw(), &stab);
    Ok(cells.len())
}
