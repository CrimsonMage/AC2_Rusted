#![forbid(unsafe_code)]

//! The Asheron's Call 2 world as data: `LandscapeDefs` and the land lattice, landblocks and their
//! terrain, region sky and game time, landblock entities, procedural scenery and grass, water,
//! env cells and cell meshes, static lights and motion interpretation descriptions.
//!
//! Positions are cell-local (a cell id and AC-axis coordinates relative to its landblock); see
//! `coordinates` for conversions to global AC and render axes.

mod ac2_cell_struct;
mod ac2_env_cell;
mod ac2_interior;
#[cfg(test)]
mod ac2_interior_tests;
mod ac2_land_defs;
mod ac2_landblock_info;
mod ac2_landblock_interior;
mod ac2_light_info;
mod ac2_motion_interp;
#[cfg(test)]
mod ac2_particle_source_tests;
mod ac2_particle_sources;
#[cfg(test)]
mod ac2_real_data_tests;
mod ac2_scene;
mod ac2_terrain_desc;
mod ac2_water;
mod ac2_world;
mod cell;
mod cell_info;
mod cell_query;
mod color;
mod coordinates;
mod land_block_data;
mod land_blocks;
mod land_lattice;
mod landblock;
mod quad_tree;
mod region;
mod surface_desc;
mod surface_info;
mod terrain_cut;
mod vertex_normals;

pub use ac2_cell_struct::{Ac2CellBspNode, Ac2CellPolygon, Ac2CellPortalPolygon, Ac2CellStruct};
pub use ac2_env_cell::{Ac2EnvCell, Ac2EnvCellPortal};
pub use ac2_landblock_info::{load_entity_ethereal_mask, Ac2OutdoorEntityMarker};
pub use ac2_light_info::{Ac2Corona, Ac2LightInfo, Ac2LightKind, Ac2StaticLight};
pub use ac2_motion_interp::{
    load_ac2_motion_interp_desc, Ac2Element, Ac2ElementMotionValues, Ac2MotionInterpDesc,
    Ac2MotionValues, AC2_MODE_DEFAULT, AC2_TERRAIN_DEFAULT,
};
pub use ac2_particle_sources::{ac2_block_particle_sources, ac2_blocks_within, Ac2ParticleSource};
pub use ac2_scene::{
    ac2_ambient_sound_points, ac2_block_async_load_dids, ac2_scene_object_visual,
    ac2_scene_placements, Ac2AmbientSoundPoint,
};
pub use ac2_water::{
    load_ac2_block_map, Ac2BlockMap, Ac2BlockWater, Ac2WaterSource, WaterLightInfo,
};
pub use ac2_world::Ac2WorldSource;
pub use cell::{
    adjust_to_outside_cell_id, classify_cell, culling_plane_check,
    outdoor_cell_id_at_render_position, same_outdoor_block, CellKind, LAND_BLOCK_CELLS_PER_SIDE,
};
pub use cell_info::{CellInfoField, CellInfoLayout, LandBlockCellInfo};
pub use cell_query::{outdoor_cell_ids_for_moving_sphere, OutdoorCellCandidates};
pub use color::Rgba;
pub use coordinates::{
    cell_local_origin_from_world_ac_position, cell_local_origin_from_world_render_position,
    cell_local_render_position_relative_to_landblock, retail_position_offset,
    world_render_position_from_cell_local,
};
pub use land_blocks::{
    block_id_at_offset, blockpart_to_blockcoord, get_block_gid, inbound_valid_blockid, interpolate,
    within_block,
};
pub use land_lattice::{
    land_block_size, land_cell_size, land_lattice, set_land_lattice, LandLattice,
};
pub use landblock::LandBlockDesc;
pub use quad_tree::{BoundingType, LandBlockQuadTree, PhysObjBrush};
pub use region::{
    ac2_detail_map, ac2_map_note_desc, ac2_region_detail_map_table, ac2_region_game_map,
    ac2_region_info, ac2_region_sound_desc, color_corner_interp, float_corner_interp,
    integer_corner_interp, vector_corner_interp, BlockFogCorners, BlockProperty, CelestialPosition,
    DetailMap, DetailMapRow, DidLookup, EncounterDesc, EncounterElem, EncounterExcept,
    EncounterRow, FogCombine, FogCombineType, FogDesc, FogInfo, FogType, GameClock, GameDateTime,
    InfoList, LensFlare, LensFlarePoint, LightColors, LightFogInfo, MapNoteDesc, MapNoteRow,
    PropertyDesc, RegionDesc, RegionFog, RegionInfo, SkyEnvironmentCadence, SkyEnvironmentSample,
    SkyFogDesc, SkyLightingDesc, TimeStampCalendar, LSCAPE_FOG_LINEAR,
};
pub use surface_desc::Ac2FarTerrain;
pub use surface_info::Ac2SurfaceInfo;
pub use terrain_cut::terrain_polygon_cut;
