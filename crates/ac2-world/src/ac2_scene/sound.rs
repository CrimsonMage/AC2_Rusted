//! A landblock's ambient sound points (`CLandBlock::CreateAmbientSoundPoints`): every landscape
//! vertex whose terrain and scene name a sound type (`CTerrainDesc::GetSoundType` ->
//! `TerrainType::GetSceneType` -> `CSceneDesc::GetSoundType`) becomes a point at the vertex.

use ac2_core::CellId;
use ac2_db::{ResourceResult, ResourceStore};

use super::format::{Ac2SceneDesc, Ac2SceneLandDefs, Ac2TerrainSceneDesc};
use super::geometry::{BLOCK_LENGTH, SIDE};
use super::{portal, LAND_DEFS_DID, SCENE_DESC_DID, TERRAIN_DESC_DID};

/// `AmbientSoundPoint`: a sound type and where it is in its block.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ac2AmbientSoundPoint {
    /// `m_soundType`: a `CSoundDesc` key.
    pub sound_type: u32,
    /// `m_origin`, block-local.
    pub origin: [f32; 3],
}

/// The ambient sound points of the landblock holding `block_cell_id`, in vertex order (empty
/// where the block or the region's descriptions are missing).
pub fn ac2_ambient_sound_points(
    store: &ResourceStore,
    block_cell_id: CellId,
) -> ResourceResult<Vec<Ac2AmbientSoundPoint>> {
    let (Some(land), Some(scenes), Some(terrains)) = (
        portal::<Ac2SceneLandDefs>(store, LAND_DEFS_DID)?,
        portal::<Ac2SceneDesc>(store, SCENE_DESC_DID)?,
        portal::<Ac2TerrainSceneDesc>(store, TERRAIN_DESC_DID)?,
    ) else {
        return Ok(Vec::new());
    };
    // The block's one `CLandBlockData` (`CLandBlock` derives from it), shared with its terrain
    // and scenery.
    let Some(record) = crate::land_block_data::load_land_block_data(store, block_cell_id)? else {
        return Ok(Vec::new());
    };
    // `LandDefs::block_length / side_vertex_count`.
    let spacing = BLOCK_LENGTH / SIDE as f64;
    let mut points = Vec::new();
    for vertex in 0..SIDE * SIDE {
        let (x, y) = (vertex / SIDE, vertex % SIDE);
        let info = record.cell_info[x * SIDE + y];
        let Some(sound_type) = sound_type(&scenes, &terrains, info) else {
            continue;
        };
        let height = land.heights[usize::from(record.heights[x * SIDE + y])];
        points.push(Ac2AmbientSoundPoint {
            sound_type,
            origin: [
                (x as f64 * spacing) as f32,
                (y as f64 * spacing) as f32,
                height as f32,
            ],
        });
    }
    Ok(points)
}

/// `CTerrainDesc::GetSoundType`: terrain bits 0-6 pick the terrain type, scene bits 7-11 its
/// scene enum, whose `CSceneDesc` scene type carries the sound (zero is none).
fn sound_type(scenes: &Ac2SceneDesc, terrains: &Ac2TerrainSceneDesc, info: u32) -> Option<u32> {
    let terrain = terrains.terrain(info & 0x7F)?;
    let scene_index = (info >> 7) & 0x1F;
    let scene_enum = terrain
        .scenes
        .iter()
        .find(|(_, index)| *index == scene_index)
        .map(|(scene_enum, _)| *scene_enum)
        .or(terrain.default_scene)?;
    let scene = scenes.types.iter().find(|t| t.scene_index == scene_enum)?;
    (scene.sound != 0).then_some(scene.sound)
}
