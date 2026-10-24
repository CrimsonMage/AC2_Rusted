//! Asheron's Call 2 `BehaviorTable` (`0x44`, DB type `0x40000040`) and `CharTemplate` (`0x2F`,
//! DB type `0x40000030`): which animation a behaviour plays in each (element, mode), how modes
//! connect, and the global behaviour flags.
//!
//! `BehaviorTable::StreamPack`: the DBObj identity, then `mModeTransitionRampTimes`
//! (`PHashTable<ModeTransitionKey, float>`), `mComboLookup` (`<ComboKey, ComboInfo>`),
//! `mTransitionLookup` (`<ModeTransitionKey, TransitionInfo>`), `mLayers` and `mTweekLayers`
//! (`PSmartArray`), `mModeTransitionTable` (`<ModeStateKey, PList<ModeStateKey>>`), `mModeList`
//! (`<ModeStateKey, int>`), `mBehaviorInfo` (`<ulong, BehaviorInfo>`), `mInitBehaviors`
//! (`PList<ulong>`) and `mInitMode`. Packed hash tables lead with a word whose low 16 bits are the
//! entry count; arrays and lists with a plain count.

use std::collections::{HashMap, HashSet};

use ac2_core::QualifiedDataId;
use ac2_dat::ArchiveCursor;
use ac2_db::{ResourceError, ResourcePackedDecode, ResourceResult, ResourceTypeId};

/// `ModeStateKey`: (element, mode).
pub type Ac2ModeState = (u32, u32);

/// `ModeTransitionKey`: (source element, source mode, destination element, destination mode).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ac2ModeTransition {
    pub src_element: u32,
    pub src_mode: u32,
    pub dst_element: u32,
    pub dst_mode: u32,
}

/// `ComboInfo`: the animation key a behaviour plays and the index of the layer it plays on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ac2ComboInfo {
    pub anim_key: u32,
    pub layer: u32,
}

/// `TransitionInfo`: the behaviour a mode transition plays and its start/stop effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Ac2TransitionInfo {
    pub behavior: u32,
    pub effect: u32,
    pub stop_effect: u32,
}

/// `BehaviorInfo`: a behaviour's ramp time and start effect.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ac2BehaviorInfo {
    pub ramp_time: f32,
    pub effect: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ac2BehaviorTable {
    pub ramp_times: HashMap<Ac2ModeTransition, f32>,
    pub combos: HashMap<(u32, u32, u32), Ac2ComboInfo>,
    pub transitions: HashMap<Ac2ModeTransition, Ac2TransitionInfo>,
    pub layers: Vec<u32>,
    pub tweak_layers: Vec<i32>,
    /// The modes each mode can move to, in table order.
    pub mode_transitions: HashMap<Ac2ModeState, Vec<Ac2ModeState>>,
    /// `mModeList`: every mode, with the flag the path search starts it with (non-zero modes are
    /// never stepped into).
    pub modes: HashMap<Ac2ModeState, i32>,
    pub behavior_info: HashMap<u32, Ac2BehaviorInfo>,
    pub init_behaviors: Vec<u32>,
    pub init_mode: u32,
}

impl Ac2BehaviorTable {
    /// `BehaviorTable::GetModeTransitionRampTime` (0 when unlisted).
    pub fn mode_transition_ramp_time(&self, key: &Ac2ModeTransition) -> f32 {
        self.ramp_times.get(key).copied().unwrap_or(0.0)
    }

    /// `BehaviorTable::GetComboInfo`: `None` is retail's invalid combo (`layerNum` `0xFFFFFFFF`).
    pub fn combo_info(&self, element: u32, mode: u32, behavior: u32) -> Option<Ac2ComboInfo> {
        self.combos.get(&(element, mode, behavior)).copied()
    }

    /// `BehaviorTable::GetBehaviorInfo` (ramp 0.25, no effect when unlisted).
    pub fn behavior_info(&self, behavior: u32) -> Ac2BehaviorInfo {
        self.behavior_info
            .get(&behavior)
            .copied()
            .unwrap_or(Ac2BehaviorInfo {
                ramp_time: 0.25,
                effect: 0,
            })
    }

    /// `BehaviorTable::GetTransitionInfo`.
    pub fn transition_info(&self, key: &Ac2ModeTransition) -> Option<Ac2TransitionInfo> {
        self.transitions.get(key).copied()
    }

    /// `BehaviorTable::GetLayerNum`.
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// `BehaviorTable::GetNextTransitionAlongModePath`: search the mode graph from the source to
    /// the destination (a stack of paths, each mode taken once, staying within the source and
    /// destination elements) and set `key`'s destination to the first step. `false` when the
    /// destination is not a mode or no path exists.
    pub fn next_transition_along_mode_path(&self, key: &mut Ac2ModeTransition) -> bool {
        let src = (key.src_element, key.src_mode);
        let dst = (key.dst_element, key.dst_mode);
        if !self.modes.contains_key(&dst) {
            return false;
        }
        let (src_element, dst_element) = (key.src_element, key.dst_element);
        let allowed = |element: u32| {
            if src_element == dst_element {
                element == src_element
            } else {
                element == src_element || element == dst_element
            }
        };
        let mut visited: HashSet<Ac2ModeState> = self
            .modes
            .iter()
            .filter(|(_, &flag)| flag != 0)
            .map(|(&state, _)| state)
            .collect();
        let mut stack: Vec<Vec<Ac2ModeState>> = vec![vec![src]];
        while let Some(path) = stack.pop() {
            let last = *path.last().expect("paths are never empty");
            let Some(nexts) = self.mode_transitions.get(&last) else {
                continue;
            };
            for &next in nexts {
                if path.contains(&next)
                    || !allowed(next.0)
                    || !self.modes.contains_key(&next)
                    || !visited.insert(next)
                {
                    continue;
                }
                let mut extended = path.clone();
                extended.push(next);
                if next == dst {
                    if !extended.iter().all(|state| allowed(state.0)) {
                        return false;
                    }
                    let first = extended.get(1).copied().unwrap_or(next);
                    key.dst_element = first.0;
                    key.dst_mode = first.1;
                    return true;
                }
                stack.push(extended);
            }
        }
        false
    }
}

type Malformed = &'static str;

fn word(cursor: &mut ArchiveCursor<'_>) -> Result<u32, Malformed> {
    cursor.read_u32().map_err(|_| "record ended early")
}

fn float(cursor: &mut ArchiveCursor<'_>) -> Result<f32, Malformed> {
    cursor.read_f32().map_err(|_| "record ended early")
}

/// A count that `entry_bytes`-sized entries must fit in.
fn count(
    cursor: &mut ArchiveCursor<'_>,
    mask: u32,
    entry_bytes: usize,
) -> Result<usize, Malformed> {
    let count = (word(cursor)? & mask) as usize;
    if count * entry_bytes > cursor.remaining() {
        return Err("entry count exceeded the record");
    }
    Ok(count)
}

fn transition_key(cursor: &mut ArchiveCursor<'_>) -> Result<Ac2ModeTransition, Malformed> {
    Ok(Ac2ModeTransition {
        src_element: word(cursor)?,
        src_mode: word(cursor)?,
        dst_element: word(cursor)?,
        dst_mode: word(cursor)?,
    })
}

fn decode_table(cursor: &mut ArchiveCursor<'_>, did: u32) -> Result<Ac2BehaviorTable, Malformed> {
    if word(cursor)? != did {
        return Err("embedded DID did not match DAT identity");
    }
    let mut ramp_times = HashMap::new();
    for _ in 0..count(cursor, 0xFFFF, 20)? {
        let key = transition_key(cursor)?;
        ramp_times.insert(key, float(cursor)?);
    }
    let mut combos = HashMap::new();
    for _ in 0..count(cursor, 0xFFFF, 20)? {
        let key = (word(cursor)?, word(cursor)?, word(cursor)?);
        let anim_key = word(cursor)?;
        let layer = word(cursor)?;
        combos.insert(key, Ac2ComboInfo { anim_key, layer });
    }
    let mut transitions = HashMap::new();
    for _ in 0..count(cursor, 0xFFFF, 28)? {
        let key = transition_key(cursor)?;
        let info = Ac2TransitionInfo {
            behavior: word(cursor)?,
            effect: word(cursor)?,
            stop_effect: word(cursor)?,
        };
        transitions.insert(key, info);
    }
    let layers = (0..count(cursor, u32::MAX, 4)?)
        .map(|_| word(cursor))
        .collect::<Result<Vec<_>, _>>()?;
    let tweak_layers = (0..count(cursor, u32::MAX, 4)?)
        .map(|_| word(cursor).map(|value| value as i32))
        .collect::<Result<Vec<_>, _>>()?;
    let mut mode_transitions = HashMap::new();
    for _ in 0..count(cursor, 0xFFFF, 12)? {
        let key = (word(cursor)?, word(cursor)?);
        let list = (0..count(cursor, u32::MAX, 8)?)
            .map(|_| Ok((word(cursor)?, word(cursor)?)))
            .collect::<Result<Vec<_>, Malformed>>()?;
        mode_transitions.insert(key, list);
    }
    let mut modes = HashMap::new();
    for _ in 0..count(cursor, 0xFFFF, 12)? {
        let key = (word(cursor)?, word(cursor)?);
        modes.insert(key, word(cursor)? as i32);
    }
    let mut behavior_info = HashMap::new();
    for _ in 0..count(cursor, 0xFFFF, 12)? {
        let key = word(cursor)?;
        let ramp_time = float(cursor)?;
        let effect = word(cursor)?;
        behavior_info.insert(key, Ac2BehaviorInfo { ramp_time, effect });
    }
    let init_behaviors = (0..count(cursor, u32::MAX, 4)?)
        .map(|_| word(cursor))
        .collect::<Result<Vec<_>, _>>()?;
    let init_mode = word(cursor)?;
    if cursor.remaining() != 0 {
        return Err("trailing bytes after BehaviorTable");
    }
    Ok(Ac2BehaviorTable {
        ramp_times,
        combos,
        transitions,
        layers,
        tweak_layers,
        mode_transitions,
        modes,
        behavior_info,
        init_behaviors,
        init_mode,
    })
}

impl ResourcePackedDecode for Ac2BehaviorTable {
    const TYPE_NAME: &'static str = "AC2 BehaviorTable";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        decode_table(&mut ArchiveCursor::new(bytes), qdid.data_id().raw()).map_err(|context| {
            ResourceError::MalformedTypedObject {
                qdid,
                type_name: Self::TYPE_NAME,
                context,
            }
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.ramp_times.len() * 24
            + self.combos.len() * 20
            + self.transitions.len() * 28
            + self
                .mode_transitions
                .values()
                .map(|list| 8 + list.len() * 8)
                .sum::<usize>()
            + self.modes.len() * 8
            + self.behavior_info.len() * 12
    }
}

/// `Behavior::m_Flags`: action, hold, dynamic first-person camera.
pub const AC2_BEHAVIOR_ACTION: u64 = 0x1;
pub const AC2_BEHAVIOR_HOLD: u64 = 0x2;
pub const AC2_BEHAVIOR_DYNAMIC_FP_CAMERA: u64 = 0x4;

/// `CharTemplate` (`CharacterTemplate_AnimSysID`): every behaviour's flags, and the default element
/// and mode. `CharTemplate::StreamPack`: identity, `behaviors` (`PHashTable<ulong, Behavior>`,
/// 64-bit flags), `sliders` and `modes`, the defaults, `userControlledSliders`.
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2CharTemplate {
    pub behaviors: HashMap<u32, u64>,
    pub default_element: u32,
    pub default_mode: u32,
}

impl Ac2CharTemplate {
    /// `CharTemplate::GetBehavior`: the behaviour's flags, when it is defined.
    pub fn behavior(&self, id: u32) -> Option<u64> {
        self.behaviors.get(&id).copied()
    }
}

fn decode_template(cursor: &mut ArchiveCursor<'_>, did: u32) -> Result<Ac2CharTemplate, Malformed> {
    if word(cursor)? != did {
        return Err("embedded DID did not match DAT identity");
    }
    let mut behaviors = HashMap::new();
    for _ in 0..count(cursor, 0xFFFF, 12)? {
        let key = word(cursor)?;
        let low = u64::from(word(cursor)?);
        let high = u64::from(word(cursor)?);
        behaviors.insert(key, low | high << 32);
    }
    // `Slider` (controller enum, 4 bytes) and `Mode` (`isCombat`, 4 bytes).
    for _ in 0..2 {
        for _ in 0..count(cursor, 0xFFFF, 8)? {
            word(cursor)?;
            word(cursor)?;
        }
    }
    let default_element = word(cursor)?;
    let default_mode = word(cursor)?;
    for _ in 0..count(cursor, u32::MAX, 4)? {
        word(cursor)?;
    }
    if cursor.remaining() != 0 {
        return Err("trailing bytes after CharTemplate");
    }
    Ok(Ac2CharTemplate {
        behaviors,
        default_element,
        default_mode,
    })
}

impl ResourcePackedDecode for Ac2CharTemplate {
    const TYPE_NAME: &'static str = "AC2 CharTemplate";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        decode_template(&mut ArchiveCursor::new(bytes), qdid.data_id().raw()).map_err(|context| {
            ResourceError::MalformedTypedObject {
                qdid,
                type_name: Self::TYPE_NAME,
                context,
            }
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.behaviors.len() * 12
    }
}

#[cfg(test)]
mod tests;
