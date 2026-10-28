use std::sync::Arc;

use ac2_core::{CellId, DataId, QualifiedDataId};
use ac2_dat::DatFileKind;
use ac2_db::{CachedResource, ResourceResult, ResourceStore};

use crate::ac2_land_defs::load_ac2_land_defs;
use crate::ac2_terrain_desc::{load_ac2_terrain_desc, Ac2TerrainDesc};
use crate::cell_info::CellInfoField;
use crate::land_block_data::{
    land_block_did, load_land_block_data, Ac2LandBlockData, LAND_BLOCK_SAMPLES,
};
use crate::surface_desc::{load_ac2_surface_desc, Ac2SurfaceDesc};
use crate::surface_info::{vertex_info, Ac2SurfaceInfo};
use crate::LandBlockDesc;

/// AC2 `LandDefs::side_vertex_count`: 8 cells per block x 2 pixels per cell + 1.
const AC2_SIDE_VERTICES: usize = 17;
const AC2_SAMPLES: usize = LAND_BLOCK_SAMPLES;
/// `CLandBlockMesh::ConstructPolygons` and `LScapeTerrainMesh::UpdateNearTerrain` drop every
/// terrain quad touching a vertex of this terrain type, opening the landscape over cave mouths.
const AC2_HOLE_TERRAIN: u32 = 0x1E;

/// Per-vertex samples (17 x 17, x-major) of one landblock: metric heights, the terrain and road
/// `SurfaceInfo` (`CLandBlockData::m_vertexSurfaceInfo`, `m_vertexRoadInfo`) and the hole flag.
#[derive(Debug, PartialEq)]
pub struct Ac2TerrainSamples {
    height_bits: Vec<u32>,
    /// `CLandBlockData::m_vNormals` (`CacheFarValues`).
    normals: Vec<[f32; 3]>,
    surface: Vec<Ac2SurfaceInfo>,
    road: Vec<Ac2SurfaceInfo>,
    /// Vertices of the landscape-hole terrain type (`0x1E`), where cave and dungeon mouths open.
    holes: Vec<bool>,
    /// `CLandBlockData::GetColor` per vertex: the shadow-light bits the vertex is lit by.
    colours: Vec<u32>,
    /// The physics material per vertex: its road's, else its terrain's, by the vertex's slope band
    /// (`CLandBlock::GetPhysicsMaterial` over `CSurfaceDesc::GetPhysicsMaterial` and
    /// `GetRoadPhysicsMaterial`). `None` where the band names none.
    physics: Vec<Option<DataId>>,
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

    pub(crate) fn physics(&self, index: usize) -> Option<DataId> {
        self.physics.get(index).copied().flatten()
    }

    pub(crate) fn colour(&self, index: usize) -> Option<u32> {
        self.colours.get(index).copied()
    }

    pub(crate) fn normal(&self, index: usize) -> Option<[f32; 3]> {
        self.normals.get(index).copied()
    }

    pub(crate) fn surface(&self, index: usize) -> Option<&Ac2SurfaceInfo> {
        self.surface.get(index)
    }

    pub(crate) fn road(&self, index: usize) -> Option<&Ac2SurfaceInfo> {
        self.road.get(index)
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
    surfaces: CachedResource<Ac2SurfaceDesc>,
    /// The fields of a vertex's cell-info word. The colour field's bits name the shadow lights a
    /// vertex is in (`CLandBlockData::GetColor`, `LScapeTerrainMesh::UpdateNearTerrain`'s alpha).
    cell_info: crate::cell_info::CellInfoLayout,
    region: Option<Arc<crate::RegionDesc>>,
    water: Option<crate::ac2_water::Ac2WaterSource>,
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
            load_ac2_surface_desc(&store)?,
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
            cell_info: land.cell_info_layout(),
            terrain,
            surfaces,
            region,
            water,
        })))
    }

    /// The far-landscape parameters of the region's current surface info
    /// (`CRegionDesc::GetCurrentSurfaceInfo`, read by `LScapeTerrainMesh::InitFarMesh`).
    pub fn far_terrain(&self) -> crate::Ac2FarTerrain {
        crate::Ac2FarTerrain {
            texture: self.surfaces.far_texture,
            texture_scale: self.surfaces.far_texture_scale,
            water_material: self.surfaces.far_water_material,
            cloud_texture: self.surfaces.cloud_texture,
            cloud_texture_scale: self.surfaces.cloud_texture_scale,
            cloud_scroll: self.surfaces.cloud_scroll,
        }
    }

    /// `LandscapeDefs` block edge in metres (cells per block x cell size).
    pub fn block_size(&self) -> f32 {
        self.block_size
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
        Ok(self
            .resident_block_water(block_cell_id)?
            .map(|water| water.height))
    }

    /// The physics material of the block's water (`CLandBlock::GetWaterPhysicsMaterial`:
    /// `CWaterDesc::GetPhysicsMaterial` of the water type the block selects). `None` where the
    /// block holds no water or its type names no material.
    pub fn water_physics_material(&self, block_cell_id: CellId) -> ResourceResult<Option<DataId>> {
        Ok(self
            .resident_block_water(block_cell_id)?
            .map(|water| water.physics_material)
            .filter(|did| did.raw() != 0))
    }

    /// The block's water as its record and water description give it, `None` when it has none.
    fn resident_block_water(
        &self,
        block_cell_id: CellId,
    ) -> ResourceResult<Option<crate::Ac2BlockWater>> {
        let Some(record) = load_land_block_data(&self.store, block_cell_id)? else {
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
            .and_then(|water| water.block_water(block_cell_id, &self.height_table, min_height)))
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

    /// `CLandBlockData` heights, holes and per-vertex terrain and road `SurfaceInfo` of one block.
    pub fn load_landblock(
        &self,
        block_cell_id: CellId,
    ) -> ResourceResult<Option<Arc<LandBlockDesc>>> {
        let Some(record) = load_land_block_data(&self.store, block_cell_id)? else {
            return Ok(None);
        };
        let samples = self.samples(&record, block_cell_id);
        Ok(Some(Arc::new(LandBlockDesc::new(
            QualifiedDataId::new(land_block_did(block_cell_id), 0),
            Arc::new(samples),
        ))))
    }

    fn samples(&self, record: &Ac2LandBlockData, block_cell_id: CellId) -> Ac2TerrainSamples {
        let side = AC2_SIDE_VERTICES as i32;
        let polygons = side - 1;
        let block_x = ((block_cell_id.raw() >> 24) & 0xFF) as i32;
        let block_y = ((block_cell_id.raw() >> 16) & 0xFF) as i32;
        // The block and its eight neighbours, for edge normals.
        let mut grid: [Option<CachedResource<Ac2LandBlockData>>; 9] = Default::default();
        for (slot, (dx, dy)) in (-1..=1)
            .flat_map(|dx| (-1..=1).map(move |dy| (dx, dy)))
            .enumerate()
        {
            if (dx, dy) != (0, 0) {
                grid[slot] = self.neighbour_record(block_x + dx, block_y + dy);
            }
        }
        let record_at = |dx: i32, dy: i32| -> Option<&Ac2LandBlockData> {
            if (dx, dy) == (0, 0) {
                return Some(record);
            }
            grid[((dx + 1) * 3 + dy + 1) as usize].as_deref()
        };
        let metres = |record: &Ac2LandBlockData, x: i32, y: i32| {
            self.height_table
                .get(usize::from(record.heights[(x * side + y) as usize]))
                .copied()
                .unwrap_or(0.0)
        };
        // `CLandBlockData::GetNeighborHeightValue`: -1 reads the previous block's vertex 15, 17
        // the next block's vertex 1; a missing block reads 0.
        let height = |x: i32, y: i32| {
            let step = |i: i32| match i {
                i if i < 0 => (-1, side - 2),
                i if i >= side => (1, 1),
                i => (0, i),
            };
            let ((dx, x), (dy, y)) = (step(x), step(y));
            record_at(dx, dy).map_or(0.0, |r| metres(r, x, y))
        };
        // `CLandBlockData::GetNeighborIsSWtoNECut`: -1 is the previous block's polygon 15, 16 and
        // 17 the next block's polygons 0 and 1; a missing block is not cut.
        let cut = |x: i32, y: i32| {
            let step = |i: i32| match i {
                i if i < 0 => (-1, polygons - 1),
                i if i >= polygons => (1, i - polygons),
                i => (0, i),
            };
            let ((dx, x), (dy, y)) = (step(x), step(y));
            record_at(dx, dy).is_some()
                && crate::terrain_cut::ac2_sw_to_ne_cut(
                    (block_x + dx) * polygons + x,
                    (block_y + dy) * polygons + y,
                )
        };
        let normals = crate::vertex_normals::vertex_normals(
            AC2_SIDE_VERTICES,
            self.vertex_spacing,
            height,
            cut,
        );
        // Global vertex coordinates seed the per-vertex material picks
        // (`CLandBlockData::CacheFarValues`: `(side_vertex_count - 1) * block + i`).
        let (base_x, base_y) = (block_x * polygons, block_y * polygons);
        let mut samples = Ac2TerrainSamples {
            height_bits: vec![0; AC2_SAMPLES],
            normals,
            surface: vec![Ac2SurfaceInfo::default(); AC2_SAMPLES],
            road: vec![Ac2SurfaceInfo::default(); AC2_SAMPLES],
            holes: vec![false; AC2_SAMPLES],
            colours: vec![0; AC2_SAMPLES],
            physics: vec![None; AC2_SAMPLES],
        };
        for x in 0..AC2_SIDE_VERTICES {
            for y in 0..AC2_SIDE_VERTICES {
                let index = x * AC2_SIDE_VERTICES + y;
                samples.height_bits[index] = metres(record, x as i32, y as i32).to_bits();
                // `pitchSq = normal.x^2 + normal.y^2` of the vertex normal.
                let [nx, ny, _] = samples.normals[index];
                let pitch_squared = nx * nx + ny * ny;
                let word = record.cell_info[index];
                let terrain = self.cell_info.get(CellInfoField::Terrain, word);
                let road = self.cell_info.get(CellInfoField::Road, word);
                samples.holes[index] = terrain == AC2_HOLE_TERRAIN;
                samples.colours[index] = self.cell_info.get(CellInfoField::Color, word);
                let (gx, gy) = (base_x + x as i32, base_y + y as i32);
                // `CSurfaceDesc::GetVertexInfo` for the terrain's surface type, `GetRoadInfo` for
                // the road bits; a miss keeps `SurfaceInfo::SetToDefault`.
                if let Some(info) = self
                    .terrain
                    .surface_type(terrain)
                    .and_then(|surface| self.surfaces.surface(surface))
                    .and_then(|surface| vertex_info(surface, pitch_squared, gx, gy))
                {
                    samples.surface[index] = info;
                }
                if let Some(info) = self
                    .surfaces
                    .road(road)
                    .and_then(|surface| vertex_info(surface, pitch_squared, gx, gy))
                {
                    samples.road[index] = info;
                }
                // `CSurfaceDesc::GetRoadPhysicsMaterial` then `GetPhysicsMaterial`: the road's
                // material where the vertex has a road, else the terrain's; each band's
                // `LScapeSurfaceType::GetPhysicsMaterial` is the first band the pitch falls in.
                let road_material = self
                    .surfaces
                    .road(road)
                    .and_then(|surface| surface.group_for_pitch(pitch_squared))
                    .map(|group| group.physics_material);
                let terrain_material = self
                    .terrain
                    .surface_type(terrain)
                    .and_then(|id| self.surfaces.surface(id))
                    .and_then(|surface| surface.group_for_pitch(pitch_squared))
                    .map(|group| group.physics_material);
                samples.physics[index] = [road_material, terrain_material]
                    .into_iter()
                    .flatten()
                    .find(|did| did.raw() != 0);
            }
        }
        samples
    }

    /// The `CLandBlockData` of block (`x`, `y`), `None` off the map or where none exists.
    fn neighbour_record(&self, x: i32, y: i32) -> Option<CachedResource<Ac2LandBlockData>> {
        if !(0..=0xFF).contains(&x) || !(0..=0xFF).contains(&y) {
            return None;
        }
        let cell = CellId::new(((x as u32) << 24) | ((y as u32) << 16) | 0xFFFF);
        load_land_block_data(&self.store, cell).ok().flatten()
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
            normals: vec![[0.0, 0.0, 1.0]; AC2_SAMPLES],
            surface: vec![Default::default(); AC2_SAMPLES],
            road: vec![Default::default(); AC2_SAMPLES],
            holes,
            colours: vec![0; AC2_SAMPLES],
            physics: vec![None; AC2_SAMPLES],
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
