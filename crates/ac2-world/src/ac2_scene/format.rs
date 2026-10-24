//! Packed records the AC2 scenery generator reads: `CSceneDesc`, the scene part
//! of `CTerrainDesc`, `SceneFile` (`0x02xxxxxx`) with its `SceneGroup`s, and the
//! scenery-relevant fields of `LandscapeDefs`.

use ac2_core::QualifiedDataId;
use ac2_dat::ArchiveCursor;
use ac2_db::{ResourceError, ResourcePackedDecode, ResourceResult, ResourceTypeId};

const MAX_ITEMS: usize = 1 << 14;
const MAX_GROUP_DEPTH: usize = 32;

fn malformed(
    qdid: QualifiedDataId,
    type_name: &'static str,
    context: &'static str,
) -> ResourceError {
    ResourceError::MalformedTypedObject {
        qdid,
        type_name,
        context,
    }
}

fn count(cursor: &mut ArchiveCursor<'_>) -> Result<usize, &'static str> {
    let value = word(cursor)? as usize;
    if value > MAX_ITEMS || value > cursor.remaining() {
        return Err("item count exceeded the record");
    }
    Ok(value)
}

fn word(cursor: &mut ArchiveCursor<'_>) -> Result<u32, &'static str> {
    cursor.read_u32().map_err(|_| "record ended early")
}

fn float(cursor: &mut ArchiveCursor<'_>) -> Result<f64, &'static str> {
    cursor
        .read_f32()
        .map(f64::from)
        .map_err(|_| "record ended early")
}

fn vector(cursor: &mut ArchiveCursor<'_>) -> Result<[f64; 3], &'static str> {
    Ok([float(cursor)?, float(cursor)?, float(cursor)?])
}

/// One `CSceneDesc` scene type: the frequency-weighted `SceneFile`s a terrain scene enum maps to.
#[derive(Debug)]
pub(super) struct Ac2SceneType {
    pub(super) scene_index: u32,
    pub(super) scenes: Vec<(u32, f64)>,
    pub(super) freq_count: f64,
}

/// `CSceneDesc`: version, DID, then counted scene types of (index, sound, counted
/// (scene DID, frequency, flags)).
#[derive(Debug)]
pub(super) struct Ac2SceneDesc {
    pub(super) types: Vec<Ac2SceneType>,
}

impl ResourcePackedDecode for Ac2SceneDesc {
    const TYPE_NAME: &'static str = "AC2 SceneDesc";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        let decode = |cursor: &mut ArchiveCursor<'_>| -> Result<Self, &'static str> {
            let _version = word(cursor)?;
            if word(cursor)? != qdid.data_id().raw() {
                return Err("embedded DID did not match DAT identity");
            }
            let mut types = Vec::new();
            for _ in 0..count(cursor)? {
                let scene_index = word(cursor)?;
                let _sound = word(cursor)?;
                let mut scenes = Vec::new();
                for _ in 0..count(cursor)? {
                    let did = word(cursor)?;
                    let freq = float(cursor)?;
                    let _flags = word(cursor)?;
                    scenes.push((did, freq));
                }
                let freq_count = scenes.iter().map(|(_, freq)| freq).sum();
                types.push(Ac2SceneType {
                    scene_index,
                    scenes,
                    freq_count,
                });
            }
            if cursor.remaining() != 0 {
                return Err("trailing bytes after SceneDesc");
            }
            Ok(Self { types })
        };
        decode(&mut ArchiveCursor::new(bytes))
            .map_err(|context| malformed(qdid, Self::TYPE_NAME, context))
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self
                .types
                .iter()
                .map(|t| 48 + t.scenes.len() * 16)
                .sum::<usize>()
    }
}

/// The scene vocabulary of one `TerrainType`: optional default `TerrainSceneInfo` and the
/// `(scene_enum, scene_index)` list.
#[derive(Debug)]
pub(super) struct Ac2TerrainScenes {
    pub(super) terrain_index: u32,
    pub(super) default_scene: Option<u32>,
    pub(super) scenes: Vec<(u32, u32)>,
}

/// `CTerrainDesc` read for its scene lists (the surface part is owned by `ac2_terrain_desc.rs`).
#[derive(Debug)]
pub(super) struct Ac2TerrainSceneDesc {
    pub(super) default: Option<Ac2TerrainScenes>,
    pub(super) terrains: Vec<Ac2TerrainScenes>,
}

impl Ac2TerrainSceneDesc {
    /// `CTerrainDesc::FindTerrainType`: first terrain with the index, else the default.
    pub(super) fn terrain(&self, index: u32) -> Option<&Ac2TerrainScenes> {
        self.terrains
            .iter()
            .find(|terrain| terrain.terrain_index == index)
            .or(self.default.as_ref())
    }
}

impl ResourcePackedDecode for Ac2TerrainSceneDesc {
    const TYPE_NAME: &'static str = "AC2 TerrainDesc scenes";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        let decode = |cursor: &mut ArchiveCursor<'_>| -> Result<Self, &'static str> {
            if word(cursor)? != qdid.data_id().raw() {
                return Err("embedded DID did not match DAT identity");
            }
            let _version = word(cursor)?;
            let default = if word(cursor)? != 0 {
                Some(terrain_scenes(cursor)?)
            } else {
                None
            };
            let terrains = (0..count(cursor)?)
                .map(|_| terrain_scenes(cursor))
                .collect::<Result<Vec<_>, _>>()?;
            if cursor.remaining() != 0 {
                return Err("trailing bytes after TerrainDesc");
            }
            Ok(Self { default, terrains })
        };
        decode(&mut ArchiveCursor::new(bytes))
            .map_err(|context| malformed(qdid, Self::TYPE_NAME, context))
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self
                .terrains
                .iter()
                .map(|t| 48 + t.scenes.len() * 8)
                .sum::<usize>()
    }
}

/// `TerrainType::unpack`: PString name (4-aligned), surface info, index, hard-edge angle, optional
/// default `TerrainSceneInfo`, counted `TerrainSceneInfo`s.
fn terrain_scenes(cursor: &mut ArchiveCursor<'_>) -> Result<Ac2TerrainScenes, &'static str> {
    let mut len = usize::from(cursor.read_u16().map_err(|_| "record ended early")?);
    if len == 0xFFFF {
        len = word(cursor)? as usize;
    }
    cursor
        .read_bytes(len)
        .map_err(|_| "terrain name exceeded the record")?;
    cursor.align_to(4).map_err(|_| "record ended early")?;
    let _surface = word(cursor)?;
    let terrain_index = word(cursor)?;
    let _hard_edge = word(cursor)?;
    let default_scene = if word(cursor)? != 0 {
        let scene_enum = word(cursor)?;
        let _scene_index = word(cursor)?;
        Some(scene_enum)
    } else {
        None
    };
    let scenes = (0..count(cursor)?)
        .map(|_| Ok((word(cursor)?, word(cursor)?)))
        .collect::<Result<Vec<_>, &'static str>>()?;
    Ok(Ac2TerrainScenes {
        terrain_index,
        default_scene,
        scenes,
    })
}

/// One `SceneEntity`: an object DID (`EntityDesc` 0x47 or `Setup` 0x04), a nested group, or a
/// nested `SceneFile`.
#[derive(Debug)]
pub(super) enum Ac2SceneEntity {
    Object(u32),
    Group(Box<Ac2SceneGroup>),
    Scene(u32),
}

/// `SceneGroup::StreamPack`.
#[derive(Debug, Default)]
pub(super) struct Ac2SceneGroup {
    pub(super) min_loc: [f64; 3],
    pub(super) max_loc: [f64; 3],
    pub(super) min_rot: f64,
    pub(super) max_rot: f64,
    pub(super) freq: f64,
    pub(super) min_scale: [f64; 3],
    pub(super) max_scale: [f64; 3],
    pub(super) min_slope: f64,
    pub(super) max_slope: f64,
    pub(super) align: bool,
    pub(super) orient: bool,
    pub(super) min_above_dry: f64,
    pub(super) max_above_dry: f64,
    pub(super) min_above_water: f64,
    pub(super) max_above_water: f64,
    pub(super) min_below_water: f64,
    pub(super) max_below_water: f64,
    pub(super) min_water_depth: f64,
    pub(super) max_water_depth: f64,
    pub(super) min_below_ground: f64,
    pub(super) max_below_ground: f64,
    pub(super) clearance_radius: f64,
    pub(super) check_slope_radius: f64,
    pub(super) scale_check_radius: bool,
    pub(super) min_slope_in_radius: f64,
    pub(super) max_slope_in_radius: f64,
    pub(super) align_max_heading: f64,
    pub(super) max_objects: u32,
    pub(super) scale_loc_sub: bool,
    pub(super) scale_size_sub: bool,
    pub(super) rot_sub_loc: bool,
    pub(super) rot_sub_orient: bool,
    pub(super) scale_clearance: bool,
    pub(super) no_legal_check: bool,
    pub(super) use_3d: bool,
    pub(super) height_relative: u32,
    pub(super) scale_type: u32,
    pub(super) above_dry_type: u32,
    pub(super) above_water_type: u32,
    pub(super) below_water_type: u32,
    pub(super) water_depth_type: u32,
    pub(super) below_ground_type: u32,
    pub(super) subs: Vec<Ac2SceneEntity>,
}

fn scene_group(
    cursor: &mut ArchiveCursor<'_>,
    depth: usize,
) -> Result<Ac2SceneGroup, &'static str> {
    if depth > MAX_GROUP_DEPTH {
        return Err("SceneGroup nesting too deep");
    }
    let flag = |cursor: &mut ArchiveCursor<'_>| word(cursor).map(|value| value != 0);
    let mut g = Ac2SceneGroup {
        min_loc: vector(cursor)?,
        max_loc: vector(cursor)?,
        min_rot: float(cursor)?,
        max_rot: float(cursor)?,
        freq: float(cursor)?,
        min_scale: vector(cursor)?,
        max_scale: vector(cursor)?,
        ..Ac2SceneGroup::default()
    };
    let _scale_depth = float(cursor)?;
    g.min_slope = float(cursor)?;
    g.max_slope = float(cursor)?;
    g.align = flag(cursor)?;
    g.orient = flag(cursor)?;
    g.min_above_dry = float(cursor)?;
    g.max_above_dry = float(cursor)?;
    g.min_above_water = float(cursor)?;
    g.max_above_water = float(cursor)?;
    g.min_below_water = float(cursor)?;
    g.max_below_water = float(cursor)?;
    g.min_water_depth = float(cursor)?;
    g.max_water_depth = float(cursor)?;
    g.min_below_ground = float(cursor)?;
    g.max_below_ground = float(cursor)?;
    g.clearance_radius = float(cursor)?;
    g.check_slope_radius = float(cursor)?;
    g.scale_check_radius = flag(cursor)?;
    g.min_slope_in_radius = float(cursor)?;
    g.max_slope_in_radius = float(cursor)?;
    g.align_max_heading = float(cursor)?;
    g.max_objects = word(cursor)?;
    let _day_freq = float(cursor)?;
    g.scale_loc_sub = flag(cursor)?;
    g.scale_size_sub = flag(cursor)?;
    g.rot_sub_loc = flag(cursor)?;
    g.rot_sub_orient = flag(cursor)?;
    g.scale_clearance = flag(cursor)?;
    g.no_legal_check = flag(cursor)?;
    g.use_3d = flag(cursor)?;
    let _choose_one = word(cursor)?;
    let _ignore_clearance = word(cursor)?;
    g.height_relative = word(cursor)?;
    g.scale_type = word(cursor)?;
    g.above_dry_type = word(cursor)?;
    g.above_water_type = word(cursor)?;
    g.below_water_type = word(cursor)?;
    g.water_depth_type = word(cursor)?;
    g.below_ground_type = word(cursor)?;
    g.subs = entities(cursor, depth + 1)?;
    Ok(g)
}

fn entities(
    cursor: &mut ArchiveCursor<'_>,
    depth: usize,
) -> Result<Vec<Ac2SceneEntity>, &'static str> {
    (0..count(cursor)?)
        .map(|_| match word(cursor)? {
            1 => Ok(Ac2SceneEntity::Object(word(cursor)?)),
            2 => Ok(Ac2SceneEntity::Group(Box::new(scene_group(cursor, depth)?))),
            3 => Ok(Ac2SceneEntity::Scene(word(cursor)?)),
            _ => Err("unknown SceneEntity type"),
        })
        .collect()
}

/// `SceneFile::StreamPack`: version, DID, allow-rotate, optional client frill `SceneGroup`, then
/// counted degrades of (min, ideal, max distance, counted `SceneEntity`s).
#[derive(Debug)]
pub(super) struct Ac2SceneFile {
    pub(super) frill: Option<Ac2SceneGroup>,
    pub(super) degrades: Vec<Vec<Ac2SceneEntity>>,
}

impl ResourcePackedDecode for Ac2SceneFile {
    const TYPE_NAME: &'static str = "AC2 SceneFile";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        let decode = |cursor: &mut ArchiveCursor<'_>| -> Result<Self, &'static str> {
            let _version = word(cursor)?;
            if word(cursor)? != qdid.data_id().raw() {
                return Err("embedded DID did not match DAT identity");
            }
            let _allow_rotate = word(cursor)?;
            let frill = if word(cursor)? != 0 {
                Some(scene_group(cursor, 0)?)
            } else {
                None
            };
            let mut degrades = Vec::new();
            for _ in 0..count(cursor)? {
                let _distances = (float(cursor)?, float(cursor)?, float(cursor)?);
                degrades.push(entities(cursor, 0)?);
            }
            if cursor.remaining() != 0 {
                return Err("trailing bytes after SceneFile");
            }
            Ok(Self { frill, degrades })
        };
        decode(&mut ArchiveCursor::new(bytes))
            .map_err(|context| malformed(qdid, Self::TYPE_NAME, context))
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + bytes_hint(self.frill.as_ref(), &self.degrades)
    }
}

fn bytes_hint(frill: Option<&Ac2SceneGroup>, degrades: &[Vec<Ac2SceneEntity>]) -> usize {
    fn list(entities: &[Ac2SceneEntity]) -> usize {
        entities
            .iter()
            .map(|entity| match entity {
                Ac2SceneEntity::Group(group) => group_bytes(group),
                _ => std::mem::size_of::<Ac2SceneEntity>(),
            })
            .sum()
    }
    fn group_bytes(group: &Ac2SceneGroup) -> usize {
        std::mem::size_of::<Ac2SceneGroup>() + list(&group.subs)
    }
    frill.map_or(0, group_bytes) + degrades.iter().map(|degrade| list(degrade)).sum::<usize>()
}

/// `LandscapeDefs` fields the generator needs: the road half-width and the height table.
#[derive(Debug)]
pub(super) struct Ac2SceneLandDefs {
    pub(super) road_width: f64,
    pub(super) heights: Vec<f64>,
}

impl ResourcePackedDecode for Ac2SceneLandDefs {
    const TYPE_NAME: &'static str = "AC2 LandDefs scenery";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        let decode = |cursor: &mut ArchiveCursor<'_>| -> Result<Self, &'static str> {
            if word(cursor)? != qdid.data_id().raw() {
                return Err("embedded DID did not match DAT identity");
            }
            let (cell, cells_per_block, sub) = (float(cursor)?, word(cursor)?, word(cursor)?);
            if cell != 20.0 || cells_per_block != 8 || sub != 2 {
                return Err("LandDefs lattice was not the retail 20/8/2 layout");
            }
            let _max_object_height = float(cursor)?;
            let _sky_height = float(cursor)?;
            let road_width = float(cursor)?;
            for _ in 0..6 {
                word(cursor)?;
            }
            let heights = (0..256)
                .map(|_| float(cursor))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Self {
                road_width,
                heights,
            })
        };
        decode(&mut ArchiveCursor::new(bytes))
            .map_err(|context| malformed(qdid, Self::TYPE_NAME, context))
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.heights.len() * 8
    }
}
