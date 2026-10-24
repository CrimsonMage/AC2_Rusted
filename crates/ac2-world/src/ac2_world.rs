use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use ac2_dat::ArchiveCursor;
use ac2_dat::DatFileKind;
use ac2_core::{CellId, DataId, QualifiedDataId};
use ac2_assets::resolve_ac2_material_texture_chain;
use ac2_db::{
    CachedResource, ResourceError, ResourcePackedDecode, ResourceResult, ResourceStore,
    ResourceTypeId, TypedResourceRef,
};

use crate::ac2_land_defs::load_ac2_land_defs;
use crate::ac2_surface_defs::{load_ac2_surface_defs, Ac2SurfaceDefs};
use crate::ac2_terrain_desc::{load_ac2_terrain_desc, Ac2TerrainDesc};
use crate::LandBlockDesc;

/// AC2 `LandDefs::side_vertex_count`: 8 cells per block x 2 pixels per cell + 1.
const AC2_SIDE_VERTICES: usize = 17;
const AC2_SAMPLES: usize = AC2_SIDE_VERTICES * AC2_SIDE_VERTICES;
/// `CLandBlockData` terrain bits (AC2 `LandscapeDefs` terrain_bits = 7 at offset 0).
const AC2_TERRAIN_MASK: u32 = 0x7F;
/// `CLandBlockMesh::ConstructPolygons` and `LScapeTerrainMesh::UpdateNearTerrain` drop every
/// terrain quad touching a vertex of this terrain type, opening the landscape over cave mouths.
const AC2_HOLE_TERRAIN: u32 = 0x1E;

/// Per-vertex samples (17 x 17, x-major) of one landblock: metric heights, the slope-selected
/// surface texture chain and the hole flag of every vertex.
#[derive(Debug, PartialEq, Eq)]
pub struct Ac2TerrainSamples {
    height_bits: Vec<u32>,
    surface_textures: Vec<u32>,
    /// Vertices of the landscape-hole terrain type (`0x1E`), where cave and dungeon mouths open.
    holes: Vec<bool>,
}

impl Ac2TerrainSamples {
    pub(crate) fn vertex_side(&self) -> usize {
        AC2_SIDE_VERTICES
    }

    pub(crate) fn height(&self, index: usize) -> Option<f32> {
        self.height_bits
            .get(index)
            .map(|bits| f32::from_bits(*bits))
    }

    pub(crate) fn is_hole(&self, index: usize) -> bool {
        self.holes.get(index).copied().unwrap_or(false)
    }

    pub(crate) fn surface_texture(&self, index: usize) -> Option<DataId> {
        self.surface_textures
            .get(index)
            .filter(|did| **did != 0)
            .map(|did| DataId::new(*did))
    }
}

/// The Asheron's Call 2 static-world source over one AC2 dataset. Owns the landscape vocabulary
/// (`LandscapeDefs`, `CTerrainDesc`, surface definitions, region and water) needed to turn a
/// `CLandBlockData` record into a [`LandBlockDesc`].
pub struct Ac2WorldSource {
    store: Arc<ResourceStore>,
    height_table: Vec<f32>,
    vertex_spacing: f32,
    /// `LandscapeDefs` block edge in metres (cells per block x cell size).
    block_size: f32,
    lattice: crate::LandLattice,
    terrain: CachedResource<Ac2TerrainDesc>,
    surfaces: CachedResource<Ac2SurfaceDefs>,
    region: Option<Arc<crate::RegionDesc>>,
    water: Option<crate::ac2_water::Ac2WaterSource>,
    texture_by_material: Mutex<HashMap<u32, Option<DataId>>>,
}

impl std::fmt::Debug for Ac2WorldSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ac2WorldSource")
            .field("vertex_spacing", &self.vertex_spacing)
            .finish_non_exhaustive()
    }
}

impl Ac2WorldSource {
    /// Opens the AC2 landscape vocabulary from an AC2 resource store. Returns `None` when the store
    /// has no AC2 `LandDefs`, i.e. it is not an AC2 dataset.
    pub fn open(store: Arc<ResourceStore>) -> ResourceResult<Option<Arc<Self>>> {
        let Some(land) = load_ac2_land_defs(&store)? else {
            return Ok(None);
        };
        let (Some(terrain), Some(surfaces)) = (
            load_ac2_terrain_desc(&store)?,
            load_ac2_surface_defs(&store)?,
        ) else {
            return Ok(None);
        };
        let vertex_spacing = land.logical_cell_size() / land.subdivisions_per_cell() as f32;
        let region = crate::region::load_ac2_region(&store)?.map(Arc::new);
        // `EntityDesc` and FX records carry property collections typed by `MasterProperty`.
        ac2_db::install_ac2_property_types(&store)?;
        let water = match crate::region::ac2_region_water_desc(&store)? {
            Some(desc) => crate::ac2_water::Ac2WaterSource::load(&store, desc)?,
            None => None,
        };
        Ok(Some(Arc::new(Self {
            height_table: land.height_table().to_vec(),
            vertex_spacing,
            block_size: land.logical_cell_size() * land.block_cells() as f32,
            lattice: crate::LandLattice::new(
                land.logical_cell_size(),
                land.subdivisions_per_cell(),
            ),
            store,
            terrain,
            surfaces,
            region,
            water,
            texture_by_material: Mutex::new(HashMap::new()),
        })))
    }

    /// AC2's `LandscapeDefs` lattice (20 m cells, two terrain vertices per cell side). The
    /// composition root installs it with `set_land_lattice` before building world geometry.
    pub fn land_lattice(&self) -> crate::LandLattice {
        self.lattice
    }

    /// The AC2 region environment (`CRegionDesc` sky, days and `GameTime`) as the engine region.
    pub fn region(&self) -> Option<Arc<crate::RegionDesc>> {
        self.region.clone()
    }

    /// The landblock's water (`CLandBlockData::CacheFarValues`), when its lowest vertex lies below
    /// the mapped water surface.
    pub fn block_water(
        &self,
        block_cell_id: CellId,
        landblock: &LandBlockDesc,
    ) -> Option<crate::Ac2BlockWater> {
        let min_height = landblock.height_grid().into_iter().fold(f32::MAX, f32::min);
        self.water
            .as_ref()?
            .block_water(block_cell_id, &self.height_table, min_height)
    }

    /// Height (AC z) of the water surface over one landblock: the plane `CPhysicsObj::CalcElement`
    /// reads through `CLandCell::InqWaterPlane` -> `CLandBlockMesh::InqWaterPlane`.
    /// `None` when the block holds no water. Reads only the cached
    /// height record, so a physics quantum can ask every step.
    pub fn water_surface_height(&self, block_cell_id: CellId) -> ResourceResult<Option<f32>> {
        let did = DataId::new((block_cell_id.raw() & 0xFFFF_0000) | 0x0000_FFFF);
        let Some(record) =
            self.store
                .load_cached_packed::<Ac2LandBlockRecord>(TypedResourceRef::new(
                    DatFileKind::Cell,
                    did,
                    0,
                ))?
        else {
            return Ok(None);
        };
        let min_height = record
            .heights
            .iter()
            .filter_map(|index| self.height_table.get(usize::from(*index)).copied())
            .fold(f32::MAX, f32::min);
        Ok(self
            .water
            .as_ref()
            .and_then(|water| water.block_water(block_cell_id, &self.height_table, min_height))
            .map(|water| water.height))
    }

    /// Placed `CLandBlockInfo` entities of one landblock.
    pub fn entity_markers(
        &self,
        block_cell_id: CellId,
    ) -> ResourceResult<Arc<[crate::Ac2OutdoorEntityMarker]>> {
        crate::ac2_landblock_info::ac2_entity_markers(&self.store, self.block_size, block_cell_id)
    }

    /// AC2 store used for AC2 textures, materials and landblocks.
    pub fn resource_store(&self) -> &Arc<ResourceStore> {
        &self.store
    }

    /// Whether the AC2 cell DAT holds this landblock's `CLandBlockData` record (no decode).
    pub fn has_landblock(&self, block_cell_id: CellId) -> bool {
        let did = DataId::new((block_cell_id.raw() & 0xFFFF_0000) | 0x0000_FFFF);
        self.store
            .has_typed_resource(ac2_db::TypedResourceRef::from_resource_type(
                DatFileKind::Cell,
                did,
                ac2_db::ResourceTypeId::LandBlock,
            ))
    }

    /// `CLandBlockData` heights, holes and per-vertex surface textures of one landblock.
    pub fn load_landblock(
        &self,
        block_cell_id: CellId,
    ) -> ResourceResult<Option<Arc<LandBlockDesc>>> {
        let did = DataId::new((block_cell_id.raw() & 0xFFFF_0000) | 0x0000_FFFF);
        let Some(record) =
            self.store
                .load_cached_packed::<Ac2LandBlockRecord>(TypedResourceRef::new(
                    DatFileKind::Cell,
                    did,
                    0,
                ))?
        else {
            return Ok(None);
        };
        let samples = self.samples(&record)?;
        Ok(Some(Arc::new(LandBlockDesc::new(
            QualifiedDataId::new(did, 0),
            Arc::new(samples),
        ))))
    }

    fn samples(&self, record: &Ac2LandBlockRecord) -> ResourceResult<Ac2TerrainSamples> {
        let metres = |x: usize, y: usize| {
            let index =
                x.min(AC2_SIDE_VERTICES - 1) * AC2_SIDE_VERTICES + y.min(AC2_SIDE_VERTICES - 1);
            self.height_table
                .get(usize::from(record.heights[index]))
                .copied()
                .unwrap_or(0.0)
        };
        let mut height_bits = vec![0_u32; AC2_SAMPLES];
        let mut surface_textures = vec![0_u32; AC2_SAMPLES];
        let mut holes = vec![false; AC2_SAMPLES];
        for x in 0..AC2_SIDE_VERTICES {
            for y in 0..AC2_SIDE_VERTICES {
                let engine_index = x * AC2_SIDE_VERTICES + y;
                height_bits[engine_index] = metres(x, y).to_bits();
                // `CLandBlockData::CacheNearValues`: slope^2 = normal.x^2 + normal.y^2 of the unit
                // vertex normal, computed on AC2's own vertex lattice.
                let dx = (metres(x + 1, y) - metres(x.saturating_sub(1), y))
                    / (self.vertex_spacing
                        * (1 + usize::from(x > 0 && x + 1 < AC2_SIDE_VERTICES)) as f32);
                let dy = (metres(x, y + 1) - metres(x, y.saturating_sub(1)))
                    / (self.vertex_spacing
                        * (1 + usize::from(y > 0 && y + 1 < AC2_SIDE_VERTICES)) as f32);
                let inv_len = 1.0 / (dx * dx + dy * dy + 1.0).sqrt();
                let slope_squared = (dx * inv_len).powi(2) + (dy * inv_len).powi(2);
                let terrain = record.cell_info[x * AC2_SIDE_VERTICES + y] & AC2_TERRAIN_MASK;
                holes[engine_index] = terrain == AC2_HOLE_TERRAIN;
                surface_textures[engine_index] = self
                    .surface_texture(terrain, slope_squared)?
                    .map_or(0, DataId::raw);
            }
        }
        Ok(Ac2TerrainSamples {
            height_bits,
            surface_textures,
            holes,
        })
    }

    /// `CTerrainDesc::GetSurfaceType` then the surface's slope band, resolved to a texture chain.
    fn surface_texture(&self, terrain: u32, slope_squared: f32) -> ResourceResult<Option<DataId>> {
        let Some(surface) = self.terrain.surface_type(terrain) else {
            return Ok(None);
        };
        let Some(material) = self
            .surfaces
            .material_for_slope(surface as u8, slope_squared)
        else {
            return Ok(None);
        };
        if let Some(cached) = self
            .texture_by_material
            .lock()
            .expect("AC2 material cache lock poisoned")
            .get(&material.raw())
        {
            return Ok(*cached);
        }
        let chain = resolve_ac2_material_texture_chain(&self.store, material)?;
        self.texture_by_material
            .lock()
            .expect("AC2 material cache lock poisoned")
            .insert(material.raw(), chain);
        Ok(chain)
    }
}

/// AC2 `CLandBlockData::StreamPack`: lbi id, path-map id, then counted height bytes and counted
/// cell-info words (terrain/scene/encounter/color/road/replace bit fields), 4-byte aligned.
#[derive(Debug)]
struct Ac2LandBlockRecord {
    heights: Vec<u8>,
    cell_info: Vec<u32>,
}

impl ResourcePackedDecode for Ac2LandBlockRecord {
    const TYPE_NAME: &'static str = "AC2 LandBlockData";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        let malformed = |context| ResourceError::MalformedTypedObject {
            qdid,
            type_name: Self::TYPE_NAME,
            context,
        };
        let mut cursor = ArchiveCursor::new(bytes);
        if cursor.read_u32()? != qdid.data_id().raw() {
            return Err(malformed("embedded DID did not match DAT identity"));
        }
        let _landblock_info_id = cursor.read_u32()?;
        let _path_map_id = cursor.read_u32()?;
        if cursor.read_u32()? as usize != AC2_SAMPLES {
            return Err(malformed("AC2 landblock height count was not 17 x 17"));
        }
        let heights = (0..AC2_SAMPLES)
            .map(|_| cursor.read_u8())
            .collect::<Result<Vec<_>, _>>()?;
        if cursor.read_u32()? as usize != AC2_SAMPLES {
            return Err(malformed("AC2 landblock cell-info count was not 17 x 17"));
        }
        let cell_info = (0..AC2_SAMPLES)
            .map(|_| cursor.read_u32())
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { heights, cell_info })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.heights.len() + self.cell_info.len() * 4
    }
}

#[cfg(test)]
mod hole_tests {
    use std::sync::Arc;

    use ac2_core::{DataId, QualifiedDataId};

    use super::{Ac2TerrainSamples, AC2_SAMPLES, AC2_SIDE_VERTICES};
    use crate::LandBlockDesc;

    fn block_with_hole_at(x: usize, y: usize) -> LandBlockDesc {
        let mut holes = vec![false; AC2_SAMPLES];
        holes[x * AC2_SIDE_VERTICES + y] = true;
        let samples = Ac2TerrainSamples {
            height_bits: vec![0; AC2_SAMPLES],
            surface_textures: vec![0; AC2_SAMPLES],
            holes,
        };
        LandBlockDesc::new(
            QualifiedDataId::new(DataId::new(0x4592_FFFF), 0),
            Arc::new(samples),
        )
    }

    #[test]
    fn every_quad_touching_a_hole_vertex_is_removed() {
        let block = block_with_hole_at(5, 7);
        // The four 10 m quads sharing vertex (5, 7) are holes; their neighbours are not.
        for (x, y) in [(4, 6), (5, 6), (4, 7), (5, 7)] {
            assert!(block.ac2_polygon_is_hole(x, y, 1), "quad ({x}, {y})");
        }
        for (x, y) in [(3, 7), (6, 7), (5, 5), (5, 8)] {
            assert!(!block.ac2_polygon_is_hole(x, y, 1), "quad ({x}, {y})");
        }
        // A coarse polygon spanning the vertex is a hole too.
        assert!(block.ac2_polygon_is_hole(4, 6, 2));
    }
}
