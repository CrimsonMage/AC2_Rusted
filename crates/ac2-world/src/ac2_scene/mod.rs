//! Asheron's Call 2 (Legions 7160) procedural scenery and grass ("frill") placement.
//!
//! `CLandBlock::GenerateSceneObjectList` walks the 17 x 17 landscape vertices. Each vertex's
//! cell info selects a terrain scene (`CLandBlockData::CacheNearValues`: `CTerrainDesc` terrain
//! type -> scene enum -> `CSceneDesc` scene type -> frequency-weighted `SceneFile`). The scene's
//! nearest degrade is expanded through nested `SceneGroup`s whose hashed frequency, location,
//! rotation and scale decide each object, rejected by bounds, roads, slope, height bands, a slope
//! radius and an object clearance list seeded with the block's `CLandBlockInfo` entities. A
//! `SceneFile`'s client frill group places grass the same way without clearances.

mod block;
mod clearance;
mod format;
mod generate;
mod geometry;
mod hash;
#[cfg(test)]
mod real_data_tests;
mod sound;

use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;

use ac2_assets::Ac2EntityDesc;
use ac2_core::{CellId, DataId};
use ac2_dat::DatFileKind;
use ac2_db::install_ac2_property_types;
use ac2_db::{CachedResource, ResourceError, ResourceResult, ResourceStore, TypedResourceRef};
use ac2_math::{Quat, Vec3};

use format::{Ac2SceneDesc, Ac2SceneFile, Ac2SceneLandDefs, Ac2TerrainSceneDesc};
use geometry::{Ac2SceneTerrain, SIDE};
pub use sound::{ac2_ambient_sound_points, Ac2AmbientSoundPoint};

const SCENE_DESC_DID: u32 = 0x1100_0000;
const TERRAIN_DESC_DID: u32 = 0x1200_0000;
const LAND_DEFS_DID: u32 = 0x3600_0000;
const ENTITY_DESC_PREFIX: u32 = 0x4700_0000;
const SETUP_PREFIX: u32 = 0x0400_0000;
const VISUAL_DESC_PREFIX: u32 = 0x1F00_0000;

/// One procedurally placed AC2 scenery object or frill.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ac2ScenePlacement {
    /// `EntityDesc` (`0x47`) or `Setup` (`0x04`) DID, as authored in the `SceneFile`.
    pub object: DataId,
    /// Block-local AC metres (x east, y north, z up).
    pub origin: Vec3,
    /// Local-to-block rotation.
    pub rotation: Quat,
    pub scale: Vec3,
    /// Client-only grass/frill (no clearance, no collision).
    pub frill: bool,
}

/// `SceneFile` lookup used by the generator.
trait Ac2SceneLookup {
    type File: Deref<Target = Ac2SceneFile>;
    fn scene_file(&self, did: u32) -> Option<Self::File>;
}

/// Store-backed lookup memoising checkouts for one generation and keeping the first error.
struct StoreLookup<'a> {
    store: &'a ResourceStore,
    files: RefCell<HashMap<u32, Option<CachedResource<Ac2SceneFile>>>>,
    error: RefCell<Option<ResourceError>>,
}

impl Ac2SceneLookup for StoreLookup<'_> {
    type File = CachedResource<Ac2SceneFile>;

    fn scene_file(&self, did: u32) -> Option<Self::File> {
        if let Some(file) = self.files.borrow().get(&did) {
            return file.clone();
        }
        let file = match portal::<Ac2SceneFile>(self.store, did) {
            Ok(file) => file,
            Err(error) => {
                self.error.borrow_mut().get_or_insert(error);
                None
            }
        };
        self.files.borrow_mut().insert(did, file.clone());
        file
    }
}

fn portal<T: ac2_db::ResourcePackedDecode>(
    store: &ResourceStore,
    did: u32,
) -> ResourceResult<Option<CachedResource<T>>> {
    store.load_cached_packed::<T>(TypedResourceRef::new(
        DatFileKind::Portal,
        DataId::new(did),
        0,
    ))
}

/// Procedural scenery and frills of one AC2 landblock, in generation order. Empty when the store
/// is not an AC2 dataset or the landblock does not exist.
pub fn ac2_scene_placements(
    store: &ResourceStore,
    block_cell_id: CellId,
) -> ResourceResult<Vec<Ac2ScenePlacement>> {
    let (Some(land), Some(scene_desc), Some(terrain_desc)) = (
        portal::<Ac2SceneLandDefs>(store, LAND_DEFS_DID)?,
        portal::<Ac2SceneDesc>(store, SCENE_DESC_DID)?,
        portal::<Ac2TerrainSceneDesc>(store, TERRAIN_DESC_DID)?,
    ) else {
        return Ok(Vec::new());
    };
    let landblock_did = DataId::new((block_cell_id.raw() & 0xFFFF_0000) | 0xFFFF);
    let Some(record) = store.load_cached_packed::<block::Ac2SceneLandBlock>(
        TypedResourceRef::new(DatFileKind::Cell, landblock_did, 0),
    )?
    else {
        return Ok(Vec::new());
    };
    let terrain = Ac2SceneTerrain {
        block_x: (block_cell_id.raw() >> 24) & 0xFF,
        block_y: (block_cell_id.raw() >> 16) & 0xFF,
        heights: record
            .heights
            .iter()
            .map(|index| land.heights[usize::from(*index)])
            .collect(),
        cell_info: record.cell_info.clone(),
        road_width: land.road_width,
    };
    let min_height = terrain.heights.iter().copied().fold(f64::MAX, f64::min);
    let water = block::block_water_height(store, block_cell_id, &land.heights, min_height)?;
    // `CLandBlockInfo` entity descriptors carry `MasterProperty`-typed property collections.
    install_ac2_property_types(store)?;
    let clearances = block::load_clearances(store, record.landblock_info)?;

    let lookup = StoreLookup {
        store,
        files: RefCell::new(HashMap::new()),
        error: RefCell::new(None),
    };
    let mut generator = generate::Generator::new(&terrain, &lookup, water, clearances);
    generator.generate(|ix, iy| vertex_scene(&terrain, &scene_desc, &terrain_desc, ix, iy));
    let placements = std::mem::take(&mut generator.placements);
    if let Some(error) = lookup.error.into_inner() {
        return Err(error);
    }
    Ok(placements
        .iter()
        .map(|p| Ac2ScenePlacement {
            object: DataId::new(p.object),
            origin: Vec3::new(p.origin[0] as f32, p.origin[1] as f32, p.origin[2] as f32),
            rotation: Quat::from_xyzw(
                p.rotation[1] as f32,
                p.rotation[2] as f32,
                p.rotation[3] as f32,
                p.rotation[0] as f32,
            ),
            scale: Vec3::new(p.scale[0] as f32, p.scale[1] as f32, p.scale[2] as f32),
            frill: p.frill,
        })
        .collect())
}

/// `CLandBlockData::CacheNearValues` scene pick for vertex `(ix, iy)`: terrain bits 0-6 and scene
/// bits 7-11 select a `TerrainSceneInfo`, whose scene enum names a `CSceneDesc` type, from which
/// a hashed fraction picks one frequency-weighted `SceneFile`. Zero means no scene.
fn vertex_scene(
    terrain: &Ac2SceneTerrain,
    scenes: &Ac2SceneDesc,
    terrains: &Ac2TerrainSceneDesc,
    ix: usize,
    iy: usize,
) -> u32 {
    let info = terrain.cell_info[ix * SIDE + iy];
    let Some(terrain_type) = terrains.terrain(info & 0x7F) else {
        return 0;
    };
    let scene_index = (info >> 7) & 0x1F;
    let scene_enum = terrain_type
        .scenes
        .iter()
        .find(|(_, index)| *index == scene_index)
        .map(|(scene_enum, _)| *scene_enum)
        .or(terrain_type.default_scene)
        .unwrap_or(0);
    if scene_enum == 0 {
        return 0;
    }
    let Some(scene_type) = scenes.types.iter().find(|t| t.scene_index == scene_enum) else {
        return 0;
    };
    let Some((last, others)) = scene_type.scenes.split_last() else {
        return 0;
    };
    let target = hash::scene_rand(
        terrain.block_x * 16 + ix as u32,
        terrain.block_y * 16 + iy as u32,
    ) * scene_type.freq_count;
    let mut accumulated = 0.0;
    for (did, freq) in others {
        accumulated += freq;
        if target < accumulated {
            return *did;
        }
    }
    last.0
}

/// The `VisualDesc` (`0x1F`) a placement's object draws and the template scale to compose with the
/// placement scale. `EntityDesc` objects resolve through their data field; `Setup` DIDs pass
/// through unchanged with unit scale.
pub fn ac2_scene_object_visual(
    store: &ResourceStore,
    object: DataId,
) -> ResourceResult<Option<(DataId, Vec3)>> {
    match object.raw() & 0xFF00_0000 {
        SETUP_PREFIX | VISUAL_DESC_PREFIX => Ok(Some((object, Vec3::ONE))),
        ENTITY_DESC_PREFIX => {
            install_ac2_property_types(store)?;
            let Some(template) = store.load_cached_packed::<Ac2EntityDesc>(
                TypedResourceRef::new(DatFileKind::Portal, object, 0),
            )?
            else {
                return Ok(None);
            };
            Ok(template
                .data
                .filter(|data| data.raw() & 0xFF00_0000 == VISUAL_DESC_PREFIX)
                .map(|data| (data, template.scale)))
        }
        _ => Ok(None),
    }
}
