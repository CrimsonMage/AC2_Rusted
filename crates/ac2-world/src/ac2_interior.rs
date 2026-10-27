//! Static-world queries over Asheron's Call 2 interiors: env cells, their cell-mesh structure,
//! the landblock's interior facts and static lights, cell containment and the collision source
//! of a cell placed in world space. Decoders live in `ac2_env_cell`, `ac2_cell_struct`,
//! `ac2_landblock_interior` and `ac2_light_info`.

use ac2_core::{CellId, DataId};
use ac2_dat::DatFileKind;
use ac2_db::{
    CachedResource, ResourcePackedDecode, ResourceResult, ResourceStore, TypedResourceRef,
};
use ac2_math::{Frame, Vec3};

use crate::ac2_cell_struct::{Ac2CellBspNode, Ac2CellStruct};
use crate::ac2_env_cell::Ac2EnvCell;
use crate::ac2_landblock_interior::{Ac2LandBlockInterior, AC2_FIRST_ENV_CELL};
use crate::ac2_light_info::Ac2LightInfo;
use crate::Ac2WorldSource;

const CELL_MESH_PREFIX: u32 = 0x0100_0000;
const ENTITY_DESC_PREFIX: u32 = 0x4700_0000;
const VISUAL_DESC_PREFIX: u32 = 0x1F00_0000;
const LIGHT_INFO_SUFFIX: u32 = 0xFFFC;
const LANDBLOCK_INFO_SUFFIX: u32 = 0xFFFE;

/// The collision source of one AC2 env cell: its world frame, the cell-mesh structure (physics
/// polygons and `PHYSICS_BSP`, cell-local) and every physics polygon fanned into world-space
/// triangles (AC axes). Physics consumes this through the active static scope.
#[derive(Debug, Clone)]
pub struct Ac2EnvCellCollision {
    pub cell_id: CellId,
    pub world_frame: Frame,
    pub structure: CachedResource<Ac2CellStruct>,
    pub world_triangles: Box<[[Vec3; 3]]>,
}

impl Ac2EnvCellCollision {
    pub fn physics_bsp(&self) -> Option<&Ac2CellBspNode> {
        self.structure.physics_bsp.as_deref()
    }
}

/// One static entity of an env cell (`CEnvCell` entity group), resolved to the `VisualDesc` it
/// draws and placed in world AC axes. Cell entity frames share the cell position's landblock
/// space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ac2CellEntityPlacement {
    pub visual: DataId,
    pub world_frame: Frame,
    pub scale: Vec3,
    /// The entity's blocking physics-type bits (`ac2_landblock_info::entity_ethereal_mask`).
    pub ethereal: u64,
}

impl Ac2WorldSource {
    /// Landblock edge in metres (`LandDefs` cell size x 8 cells).
    pub fn ac2_block_size(&self) -> f32 {
        self.land_lattice().cell_size() * crate::LAND_BLOCK_CELLS_PER_SIDE as f32
    }

    /// World origin (AC axes) of the landblock holding `cell_id`.
    pub fn ac2_block_origin(&self, cell_id: CellId) -> Vec3 {
        let size = self.ac2_block_size();
        Vec3::new(
            f32::from(((cell_id.raw() >> 24) & 0xFF) as u8) * size,
            f32::from(((cell_id.raw() >> 16) & 0xFF) as u8) * size,
            0.0,
        )
    }

    /// The decoded env cell `xxyy0100+`, or `None` when the cell DAT has no such record.
    /// The static objects of an env cell (`CEnvCell::InitStaticObjects`) as placed visuals in
    /// world AC axes.
    pub fn env_cell_markers(&self, cell: &Ac2EnvCell) -> Vec<crate::Ac2OutdoorEntityMarker> {
        crate::ac2_landblock_info::ac2_env_cell_markers(
            self.resource_store(),
            self.ac2_block_origin(cell.cell_id),
            &cell.entities,
        )
    }

    pub fn env_cell(&self, cell_id: CellId) -> ResourceResult<Option<CachedResource<Ac2EnvCell>>> {
        if cell_id.raw() & 0xFFFF < AC2_FIRST_ENV_CELL
            || cell_id.raw() & 0xFFFF >= LIGHT_INFO_SUFFIX
        {
            return Ok(None);
        }
        load(
            self.resource_store(),
            DatFileKind::Cell,
            DataId::new(cell_id.raw()),
        )
    }

    /// The env cell's frame in world AC axes.
    pub fn env_cell_world_frame(&self, cell: &Ac2EnvCell) -> Frame {
        Frame::new(
            self.ac2_block_origin(cell.cell_id) + cell.frame.origin,
            cell.frame.orientation,
        )
    }

    /// The static structure (collision polygons, physics and cell trees, portal polygons) of a
    /// `CCellMesh`.
    pub fn cell_struct(
        &self,
        cell_mesh: DataId,
    ) -> ResourceResult<Option<CachedResource<Ac2CellStruct>>> {
        if cell_mesh.raw() & 0xFF00_0000 != CELL_MESH_PREFIX {
            return Ok(None);
        }
        load(self.resource_store(), DatFileKind::Portal, cell_mesh)
    }

    /// The landblock's env cell count, interior lighting/fog and outside stab list.
    pub fn landblock_interior(
        &self,
        block_cell_id: CellId,
    ) -> ResourceResult<Option<CachedResource<Ac2LandBlockInterior>>> {
        let did = DataId::new((block_cell_id.raw() & 0xFFFF_0000) | LANDBLOCK_INFO_SUFFIX);
        load(self.resource_store(), DatFileKind::Cell, did)
    }

    /// The landblock's static lights (`CLightInfo`, `xxyyFFFC`).
    pub fn light_info(
        &self,
        block_cell_id: CellId,
    ) -> ResourceResult<Option<CachedResource<Ac2LightInfo>>> {
        let did = DataId::new((block_cell_id.raw() & 0xFFFF_0000) | LIGHT_INFO_SUFFIX);
        load(self.resource_store(), DatFileKind::Cell, did)
    }

    /// Whether the world point (AC axes) lies inside `cell` (`CEnvCell::point_in_cell` through the
    /// cell mesh's `CELL_BSP`).
    pub fn env_cell_contains(&self, cell: &Ac2EnvCell, world_point: Vec3) -> ResourceResult<bool> {
        let Some(structure) = self.cell_struct(cell.cell_mesh)? else {
            return Ok(false);
        };
        let local = self
            .env_cell_world_frame(cell)
            .global_to_local_point(world_point);
        Ok(structure.point_in_cell(local))
    }

    /// The first of `candidates` containing the world point.
    pub fn find_env_cell(
        &self,
        world_point: Vec3,
        candidates: impl IntoIterator<Item = CellId>,
    ) -> ResourceResult<Option<CachedResource<Ac2EnvCell>>> {
        for candidate in candidates {
            let Some(cell) = self.env_cell(candidate)? else {
                continue;
            };
            if self.env_cell_contains(&cell, world_point)? {
                return Ok(Some(cell));
            }
        }
        Ok(None)
    }

    /// The cell's static entities resolved to visuals; instances name a template `EntityDesc`
    /// whose data is the `VisualDesc`, and scale composes instance over template.
    pub fn env_cell_entities(
        &self,
        cell: &Ac2EnvCell,
    ) -> ResourceResult<Vec<Ac2CellEntityPlacement>> {
        let origin = self.ac2_block_origin(cell.cell_id);
        let mut placements = Vec::with_capacity(cell.entities.len());
        for entity in cell.entities.iter() {
            let Some(data) = entity.data else {
                continue;
            };
            let (visual, template_scale) = match data.raw() & 0xFF00_0000 {
                VISUAL_DESC_PREFIX => (data, Vec3::ONE),
                ENTITY_DESC_PREFIX => {
                    let Some(template) = load::<ac2_assets::Ac2EntityDesc>(
                        self.resource_store(),
                        DatFileKind::Portal,
                        data,
                    )
                    .ok()
                    .flatten() else {
                        continue;
                    };
                    match template.data {
                        Some(visual) if visual.raw() & 0xFF00_0000 == VISUAL_DESC_PREFIX => {
                            (visual, template.scale)
                        }
                        _ => continue,
                    }
                }
                _ => continue,
            };
            placements.push(Ac2CellEntityPlacement {
                visual,
                world_frame: Frame::new(origin + entity.origin, entity.rotation),
                scale: entity.scale * template_scale,
                ethereal: crate::ac2_landblock_info::entity_ethereal_mask(entity),
            });
        }
        Ok(placements)
    }

    /// The collision source of an env cell placed in world space.
    pub fn env_cell_collision(
        &self,
        cell_id: CellId,
    ) -> ResourceResult<Option<Ac2EnvCellCollision>> {
        let Some(cell) = self.env_cell(cell_id)? else {
            return Ok(None);
        };
        let Some(structure) = self.cell_struct(cell.cell_mesh)? else {
            return Ok(None);
        };
        let world_frame = self.env_cell_world_frame(&cell);
        let mut world_triangles = Vec::new();
        for polygon in structure.polygons.iter() {
            let points = polygon
                .vertex_ids
                .iter()
                .filter_map(|id| structure.positions.get(usize::from(*id)))
                .map(|point| world_frame.local_to_global_point(*point))
                .collect::<Vec<_>>();
            for index in 1..points.len().saturating_sub(1) {
                world_triangles.push([points[0], points[index], points[index + 1]]);
            }
        }
        Ok(Some(Ac2EnvCellCollision {
            cell_id,
            world_frame,
            structure,
            world_triangles: world_triangles.into_boxed_slice(),
        }))
    }
}

fn load<T: ResourcePackedDecode + Send + Sync + 'static>(
    store: &ResourceStore,
    kind: DatFileKind,
    did: DataId,
) -> ResourceResult<Option<CachedResource<T>>> {
    store.load_cached_packed::<T>(TypedResourceRef::new(kind, did, 0))
}
