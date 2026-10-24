//! Asheron's Call 2 `MotionInterpDesc` (`0x32`, DB type `0x40000034`): the movement limits a
//! creature's motion interpreter reads for its current element and mode.
//!
//! Retail `MotionInterpDesc::StreamPack` reads the DBObj identity, a
//! `PHashTable<ModeStateKey, ulong>` from (element, mode) to an index into a `PSmartArray<MSList>`,
//! then the sea-floor and water multipliers. Each `MSList` (`MSList::StreamPack`) holds a terrain
//! hash of `TerrainList`s, a global-modifier hash and its default `MotionValues`
//! (`MotionValues::StreamPack`, 22 words). `MotionInterpDesc::GetMotionValues` picks the global
//! modifier's values, else the terrain's (optionally a terrain modifier's), else the default.

use std::collections::HashMap;

use ac2_dat::ArchiveCursor;
use ac2_dat::DatFileKind;
use ac2_core::{DataId, QualifiedDataId};
use ac2_db::{
    CachedResource, ResourceError, ResourcePackedDecode, ResourceResult, ResourceStore,
    ResourceTypeId, TypedResourceRef,
};

/// The element a body moves in (`CPhysicsObj::CalcElement`). Each
/// keys its own `MotionValues` list: the human desc gives the ground list its locomotion, the
/// air list no acceleration (no air control), water buoyancy and the sea floor a wading pace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ac2Element {
    Ground,
    Air,
    Water,
    SeaFloor,
    WalkOnAir,
}

impl Ac2Element {
    pub const fn raw(self) -> u32 {
        match self {
            Self::Ground => 0x4000_0001,
            Self::Air => 0x4000_0002,
            Self::Water => 0x4000_0003,
            Self::SeaFloor => 0x4000_0004,
            Self::WalkOnAir => 0x4000_0007,
        }
    }
}

/// The `MotionValues` one body uses in every element for one mode and terrain
/// (`CMotionInterp::GetMotionValuesForElement`), with the desc's depth multipliers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ac2ElementMotionValues {
    pub ground: Ac2MotionValues,
    pub air: Ac2MotionValues,
    pub water: Ac2MotionValues,
    pub sea_floor: Ac2MotionValues,
    pub walk_on_air: Ac2MotionValues,
    /// `mWaterMultiplier`: swimming starts this fraction of the body height under water.
    pub water_multiplier: f32,
    /// `mSeaFloorMultiplier`: wading starts this fraction of the body height under water.
    pub sea_floor_multiplier: f32,
}

impl Ac2ElementMotionValues {
    pub const fn values(&self, element: Ac2Element) -> &Ac2MotionValues {
        match element {
            Ac2Element::Ground => &self.ground,
            Ac2Element::Air => &self.air,
            Ac2Element::Water => &self.water,
            Ac2Element::SeaFloor => &self.sea_floor,
            Ac2Element::WalkOnAir => &self.walk_on_air,
        }
    }

    /// One set of values for every element; the multipliers are retail's human defaults.
    pub fn uniform(values: Ac2MotionValues) -> Self {
        Self {
            ground: values,
            air: values,
            water: values,
            sea_floor: values,
            walk_on_air: values,
            water_multiplier: 0.75,
            sea_floor_multiplier: 0.1,
        }
    }
}
/// The default mode; `CMotionInterp::GetDesiredMotionValues` falls back to it.
pub const AC2_MODE_DEFAULT: u32 = 0x4000_0001;
/// `CMotionInterp::mTerrainType`'s constructor value.
pub const AC2_TERRAIN_DEFAULT: u32 = 0x8000_0001;

const MAX_ENTRIES: usize = 4096;
/// `MotionValues::StreamPack` writes 22 words.
const MOTION_VALUES_BYTES: usize = 22 * 4;

/// Retail `MotionValues`: linear speeds in metres per second, turn rates in degrees per second,
/// accelerations per second. The `*_brake` fields are retail's `m*SB` ("stop/brake") rates used
/// when the target velocity is zero or reverses direction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ac2MotionValues {
    pub lift: f32,
    pub drag: f32,
    pub terminal_velocity: f32,
    pub gravity: bool,
    pub sink_offset: f32,
    pub hover_height: f32,
    pub jump_impulse: f32,
    pub max_forward_velocity: f32,
    pub max_backward_velocity: f32,
    pub max_strafe_velocity: f32,
    pub still_turn_velocity: f32,
    pub running_turn_velocity: f32,
    pub max_vertical_velocity: f32,
    pub forward_brake: f32,
    pub backward_brake: f32,
    pub strafe_brake: f32,
    pub turn_brake: f32,
    pub forward_acceleration: f32,
    pub backward_acceleration: f32,
    pub strafe_acceleration: f32,
    pub turn_acceleration: f32,
    pub vertical_acceleration: f32,
}

impl Default for Ac2MotionValues {
    /// `MotionValues::MotionValues`: a motionless body that still falls.
    fn default() -> Self {
        Self {
            lift: 0.0,
            drag: 0.0,
            terminal_velocity: 50.0,
            gravity: true,
            sink_offset: 0.0,
            hover_height: 0.0,
            jump_impulse: 0.0,
            max_forward_velocity: 0.0,
            max_backward_velocity: 0.0,
            max_strafe_velocity: 0.0,
            still_turn_velocity: 0.0,
            running_turn_velocity: 0.0,
            max_vertical_velocity: 0.0,
            forward_brake: 1.0,
            backward_brake: 1.0,
            strafe_brake: 1.0,
            turn_brake: 180.0,
            forward_acceleration: 0.0,
            backward_acceleration: 0.0,
            strafe_acceleration: 0.0,
            turn_acceleration: 0.0,
            vertical_acceleration: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct TerrainList {
    default: Ac2MotionValues,
    modifiers: HashMap<u32, Ac2MotionValues>,
}

#[derive(Debug, Clone, PartialEq)]
struct ModeStateList {
    default: Ac2MotionValues,
    terrains: HashMap<u32, TerrainList>,
    global_modifiers: HashMap<u32, Ac2MotionValues>,
}

/// A decoded `MotionInterpDesc`.
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2MotionInterpDesc {
    /// (element, mode) -> index into `lists`.
    states: HashMap<(u32, u32), usize>,
    lists: Vec<ModeStateList>,
    sea_floor_multiplier: f32,
    water_multiplier: f32,
}

impl Ac2MotionInterpDesc {
    /// `MotionInterpDesc::GetMotionValues`. `terrain_modifier` is the creature's
    /// registered modifier for `terrain` (`CMotionInterp::AddTerrainModifier`), if any; a
    /// registered modifier the terrain does not list yields `None`, as retail returns null.
    pub fn motion_values(
        &self,
        element: u32,
        mode: u32,
        global_modifier: u32,
        terrain: u32,
        terrain_modifier: Option<u32>,
    ) -> Option<&Ac2MotionValues> {
        let list = &self.lists[*self.states.get(&(element, mode))?];
        if let Some(values) = list.global_modifiers.get(&global_modifier) {
            return Some(values);
        }
        let Some(terrain) = list.terrains.get(&terrain) else {
            return Some(&list.default);
        };
        match terrain_modifier {
            None => Some(&terrain.default),
            Some(modifier) => terrain.modifiers.get(&modifier),
        }
    }

    /// `CMotionInterp::GetDesiredMotionValues`: the mode's values, else the
    /// element's default-mode values, else retail's static default `MotionValues`.
    pub fn desired_motion_values(
        &self,
        element: u32,
        mode: u32,
        global_modifier: u32,
        terrain: u32,
        terrain_modifier: Option<u32>,
    ) -> Ac2MotionValues {
        self.motion_values(element, mode, global_modifier, terrain, terrain_modifier)
            .or_else(|| {
                self.motion_values(
                    element,
                    AC2_MODE_DEFAULT,
                    global_modifier,
                    terrain,
                    terrain_modifier,
                )
            })
            .copied()
            .unwrap_or_default()
    }

    /// `GetDesiredMotionValues` for every element at one mode and terrain.
    pub fn element_values(
        &self,
        mode: u32,
        global_modifier: u32,
        terrain: u32,
        terrain_modifier: Option<u32>,
    ) -> Ac2ElementMotionValues {
        let values = |element: Ac2Element| {
            self.desired_motion_values(
                element.raw(),
                mode,
                global_modifier,
                terrain,
                terrain_modifier,
            )
        };
        Ac2ElementMotionValues {
            ground: values(Ac2Element::Ground),
            air: values(Ac2Element::Air),
            water: values(Ac2Element::Water),
            sea_floor: values(Ac2Element::SeaFloor),
            walk_on_air: values(Ac2Element::WalkOnAir),
            water_multiplier: self.water_multiplier,
            sea_floor_multiplier: self.sea_floor_multiplier,
        }
    }

    /// `MotionInterpDesc::mSeaFloorMultiplier`.
    pub fn sea_floor_multiplier(&self) -> f32 {
        self.sea_floor_multiplier
    }

    /// `MotionInterpDesc::mWaterMultiplier`.
    pub fn water_multiplier(&self) -> f32 {
        self.water_multiplier
    }
}

impl ResourcePackedDecode for Ac2MotionInterpDesc {
    const TYPE_NAME: &'static str = "AC2 MotionInterpDesc";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        decode(&mut ArchiveCursor::new(bytes), qdid).map_err(|context| {
            ResourceError::MalformedTypedObject {
                qdid,
                type_name: Self::TYPE_NAME,
                context,
            }
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        let values = std::mem::size_of::<Ac2MotionValues>();
        let lists: usize = self
            .lists
            .iter()
            .map(|list| {
                let terrains: usize = list
                    .terrains
                    .values()
                    .map(|terrain| (1 + terrain.modifiers.len()) * values)
                    .sum();
                (1 + list.global_modifiers.len()) * values + terrains
            })
            .sum();
        std::mem::size_of::<Self>() + self.states.len() * 16 + lists
    }
}

/// Loads a `MotionInterpDesc` (a `VisualDesc`'s `m_midesc_did`) from the portal DAT.
pub fn load_ac2_motion_interp_desc(
    store: &ResourceStore,
    did: DataId,
) -> ResourceResult<Option<CachedResource<Ac2MotionInterpDesc>>> {
    store.load_cached_packed::<Ac2MotionInterpDesc>(TypedResourceRef::new(
        DatFileKind::Portal,
        did,
        0,
    ))
}

type Malformed = &'static str;

fn decode(
    cursor: &mut ArchiveCursor<'_>,
    qdid: QualifiedDataId,
) -> Result<Ac2MotionInterpDesc, Malformed> {
    if word(cursor)? != qdid.data_id().raw() {
        return Err("embedded DID did not match DAT identity");
    }
    let mut states = HashMap::new();
    for _ in 0..hash_count(cursor, 12)? {
        let element = word(cursor)?;
        let mode = word(cursor)?;
        states.insert((element, mode), word(cursor)? as usize);
    }
    let count = word(cursor)? as usize;
    if count > MAX_ENTRIES || count * MOTION_VALUES_BYTES > cursor.remaining() {
        return Err("MSList count exceeded the record");
    }
    let mut lists = Vec::with_capacity(count);
    for _ in 0..count {
        let mut terrains = HashMap::new();
        for _ in 0..hash_count(cursor, 4)? {
            let terrain = word(cursor)?;
            // `TerrainList::StreamPack`: modifier hash, then its default values.
            let modifiers = values_hash(cursor)?;
            let default = motion_values(cursor)?;
            terrains.insert(terrain, TerrainList { default, modifiers });
        }
        let global_modifiers = values_hash(cursor)?;
        let default = motion_values(cursor)?;
        lists.push(ModeStateList {
            default,
            terrains,
            global_modifiers,
        });
    }
    if states.values().any(|&index| index >= lists.len()) {
        return Err("mode state names a missing MSList");
    }
    let sea_floor_multiplier = float(cursor)?;
    let water_multiplier = float(cursor)?;
    if cursor.remaining() != 0 {
        return Err("trailing bytes after MotionInterpDesc");
    }
    Ok(Ac2MotionInterpDesc {
        states,
        lists,
        sea_floor_multiplier,
        water_multiplier,
    })
}

fn values_hash(cursor: &mut ArchiveCursor<'_>) -> Result<HashMap<u32, Ac2MotionValues>, Malformed> {
    let count = hash_count(cursor, 4 + MOTION_VALUES_BYTES)?;
    let mut values = HashMap::with_capacity(count);
    for _ in 0..count {
        let key = word(cursor)?;
        values.insert(key, motion_values(cursor)?);
    }
    Ok(values)
}

/// `MotionValues::StreamPack` field order.
fn motion_values(cursor: &mut ArchiveCursor<'_>) -> Result<Ac2MotionValues, Malformed> {
    Ok(Ac2MotionValues {
        lift: float(cursor)?,
        drag: float(cursor)?,
        terminal_velocity: float(cursor)?,
        gravity: word(cursor)? != 0,
        sink_offset: float(cursor)?,
        hover_height: float(cursor)?,
        jump_impulse: float(cursor)?,
        max_forward_velocity: float(cursor)?,
        max_backward_velocity: float(cursor)?,
        max_strafe_velocity: float(cursor)?,
        still_turn_velocity: float(cursor)?,
        running_turn_velocity: float(cursor)?,
        max_vertical_velocity: float(cursor)?,
        forward_brake: float(cursor)?,
        backward_brake: float(cursor)?,
        strafe_brake: float(cursor)?,
        turn_brake: float(cursor)?,
        forward_acceleration: float(cursor)?,
        backward_acceleration: float(cursor)?,
        strafe_acceleration: float(cursor)?,
        turn_acceleration: float(cursor)?,
        vertical_acceleration: float(cursor)?,
    })
}

/// Packed hash tables lead with a word whose low 16 bits are the entry count.
fn hash_count(cursor: &mut ArchiveCursor<'_>, entry_bytes: usize) -> Result<usize, Malformed> {
    let count = (word(cursor)? & 0xFFFF) as usize;
    if count * entry_bytes > cursor.remaining() {
        return Err("hash entry count exceeded the record");
    }
    Ok(count)
}

fn word(cursor: &mut ArchiveCursor<'_>) -> Result<u32, Malformed> {
    cursor.read_u32().map_err(|_| "record ended early")
}

fn float(cursor: &mut ArchiveCursor<'_>) -> Result<f32, Malformed> {
    cursor.read_f32().map_err(|_| "record ended early")
}

#[cfg(test)]
#[path = "ac2_motion_interp_tests.rs"]
mod tests;
