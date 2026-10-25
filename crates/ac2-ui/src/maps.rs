//! The input maps the client binds keys through: `CKeyMap` (`0x1D`, `DB_TYPE_KEYMAP`) and
//! `ActionMap` (`0x51`, `DB_TYPE_ACTIONMAP`).
//!
//! `CKeyMap::StreamPack`: the `DBObj` DID, the map's name, its `MapGUID` (`Data1`, `Data2`,
//! `Data3`, eight `Data4` bytes) and a `PList<ActionMapping>` of (action, input semantic, name).
//! `ActionMap::UnPack`: the DID, the string table, the toggle table (action -> toggle type), the
//! action table ((action, modifier bitfield) -> (class, enum value, extent)) and the
//! description table (action -> name and description string ids).

use std::collections::{BTreeMap, HashMap};

use crate::layout_media::pstring_bytes;
use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::{ArchiveCursor, DatFileKind};
use ac2_db::{
    CachedResource, ResourceError, ResourcePackedDecode, ResourceResult, ResourceStore,
    ResourceTypeId, TypedResourceRef,
};

/// `KEYMAP_GROUP_ENUM` and `DB_TYPE_KEYMAP` (`CInputManager_WIN32::PushKeyMap`).
pub const KEYMAP_GROUP: u32 = 0x4000_000F;
/// `gmClient::InitKeymap`'s fallback key maps when no `.keymap` file loads: main (no meta key),
/// control (left control, meta mode 4) and shift (left shift, meta mode 1).
pub const KEYMAP_MAIN: u32 = 0x4100_0001;
pub const KEYMAP_CONTROL: u32 = 0x4100_0002;
pub const KEYMAP_SHIFT: u32 = 0x4100_0003;
/// `ActionInterpreter::Startup`: `GetByEnum(0x40000002, group 0x4000000d, DB_TYPE_ACTIONMAP)`.
const ACTION_MAP_GROUP: u32 = 0x4000_000D;
const ACTION_MAP_ENUM: u32 = 0x4000_0002;
const MAX_ITEMS: usize = 1 << 16;

/// One `ActionMapping`: the action a key raises.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionMapping {
    pub action: u32,
    /// The input semantic (`0x81000400 | DIK scan code` for keys, `0x82000xxx` for the mouse).
    pub semantic: u32,
    /// The binding's name.
    pub name: String,
}

/// A `CKeyMap`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyMap {
    pub name: String,
    pub guid: [u8; 16],
    pub mappings: Vec<ActionMapping>,
}

/// `ActionMapValue`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ActionValue {
    pub class: u32,
    pub enum_value: u32,
    pub extent: f32,
}

/// An `ActionMap`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActionMap {
    pub string_table: u32,
    /// `m_toggle_table`: action -> toggle type (`0x40000001` hold, `0x40000002` toggle,
    /// `0x40000003` one-shot).
    pub toggles: HashMap<u32, u32>,
    /// `m_action_table`: (action, modifier bitfield) -> value.
    pub actions: BTreeMap<(u32, u32), ActionValue>,
    /// `m_descrip_table`: action -> (name, description) string ids.
    pub descriptions: HashMap<u32, (u32, u32)>,
}

impl ActionMap {
    /// `ActionMap::GetToggleType` (0 when the action is not in the table).
    pub fn toggle_type(&self, action: u32) -> u32 {
        self.toggles.get(&action).copied().unwrap_or(0)
    }

    /// `ActionMap::GetActionValues`.
    pub fn values(&self, action: u32, modifiers: u32) -> Option<ActionValue> {
        self.actions.get(&(action, modifiers)).copied()
    }
}

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

fn count(cursor: &mut ArchiveCursor<'_>, item: usize) -> Option<usize> {
    let n = cursor.read_u32().ok()? as usize;
    (n <= MAX_ITEMS && n * item <= cursor.remaining()).then_some(n)
}

impl ResourcePackedDecode for KeyMap {
    const TYPE_NAME: &'static str = "AC2 CKeyMap";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(qdid: QualifiedDataId, _version: u16, bytes: &[u8]) -> ResourceResult<Self> {
        let bad = |context| malformed(qdid, Self::TYPE_NAME, context);
        let mut cursor = ArchiveCursor::new(bytes);
        if cursor.read_u32()? != qdid.data_id().raw() {
            return Err(bad("embedded DID did not match DAT identity"));
        }
        let name = String::from_utf8_lossy(&pstring_bytes(&mut cursor)?).into_owned();
        let mut guid = [0_u8; 16];
        guid[..4].copy_from_slice(&cursor.read_u32()?.to_le_bytes());
        guid[4..6].copy_from_slice(&cursor.read_u16()?.to_le_bytes());
        guid[6..8].copy_from_slice(&cursor.read_u16()?.to_le_bytes());
        for byte in &mut guid[8..] {
            *byte = cursor.read_u8()?;
        }
        let n = count(&mut cursor, 12).ok_or_else(|| bad("mapping count"))?;
        let mut mappings = Vec::with_capacity(n);
        for _ in 0..n {
            mappings.push(ActionMapping {
                action: cursor.read_u32()?,
                semantic: cursor.read_u32()?,
                name: String::from_utf8_lossy(&pstring_bytes(&mut cursor)?).into_owned(),
            });
        }
        if cursor.remaining() != 0 {
            return Err(bad("trailing bytes after CKeyMap"));
        }
        Ok(Self {
            name,
            guid,
            mappings,
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.mappings.len() * 48
    }
}

impl ResourcePackedDecode for ActionMap {
    const TYPE_NAME: &'static str = "AC2 ActionMap";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(qdid: QualifiedDataId, _version: u16, bytes: &[u8]) -> ResourceResult<Self> {
        let bad = |context| malformed(qdid, Self::TYPE_NAME, context);
        let mut cursor = ArchiveCursor::new(bytes);
        if cursor.read_u32()? != qdid.data_id().raw() {
            return Err(bad("embedded DID did not match DAT identity"));
        }
        let mut map = ActionMap {
            string_table: cursor.read_u32()?,
            ..ActionMap::default()
        };
        for _ in 0..count(&mut cursor, 8).ok_or_else(|| bad("toggle count"))? {
            let action = cursor.read_u32()?;
            map.toggles.insert(action, cursor.read_u32()?);
        }
        for _ in 0..count(&mut cursor, 20).ok_or_else(|| bad("action count"))? {
            let key = (cursor.read_u32()?, cursor.read_u32()?);
            let value = ActionValue {
                class: cursor.read_u32()?,
                enum_value: cursor.read_u32()?,
                extent: cursor.read_f32()?,
            };
            map.actions.insert(key, value);
        }
        for _ in 0..count(&mut cursor, 12).ok_or_else(|| bad("description count"))? {
            let action = cursor.read_u32()?;
            map.descriptions
                .insert(action, (cursor.read_u32()?, cursor.read_u32()?));
        }
        if cursor.remaining() != 0 {
            return Err(bad("trailing bytes after ActionMap"));
        }
        Ok(map)
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + (self.toggles.len() + self.descriptions.len()) * 16
            + self.actions.len() * 32
    }
}

fn load<T: ResourcePackedDecode + Send + Sync + 'static>(
    store: &ResourceStore,
    did: DataId,
) -> ResourceResult<Option<CachedResource<T>>> {
    store.load_cached_packed::<T>(TypedResourceRef::new(DatFileKind::Portal, did, 0))
}

/// `CInputManager_WIN32::PushKeyMap(_actID, _metaKey)`'s map: `GetByEnum(_actID,
/// KEYMAP_GROUP_ENUM, DB_TYPE_KEYMAP)`.
pub fn load_key_map(
    store: &ResourceStore,
    enum_value: u32,
) -> ResourceResult<Option<CachedResource<KeyMap>>> {
    match ac2_assets::ac2_enum_did(store, KEYMAP_GROUP, enum_value)? {
        Some(did) => load(store, did),
        None => Ok(None),
    }
}

/// `ActionInterpreter::Startup`'s action map.
pub fn load_action_map(store: &ResourceStore) -> ResourceResult<Option<CachedResource<ActionMap>>> {
    match ac2_assets::ac2_enum_did(store, ACTION_MAP_GROUP, ACTION_MAP_ENUM)? {
        Some(did) => load(store, did),
        None => Ok(None),
    }
}
