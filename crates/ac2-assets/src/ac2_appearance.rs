//! Asheron's Call 2 appearance material atoms: the `AppearanceTable` (`0x20`) entries that
//! retexture a mesh's material instances, and material resolution under those atoms.
//!
//! `AppearanceTable::StreamPack`: DID, counted base-table DIDs, then a
//! `PHashTable<key, VisualModTable>` (entry count in the low half of its header word) whose
//! tables are counted `AtomCollection`s (`AtomCollection::StreamPack`:
//! intensity, then the material-modifier, opacity, mesh, part-group, FX and icon atom lists),
//! then the threshold-count hash. `AppearanceTable::GetAtoms` merges base tables first,
//! then the last collection whose intensity does not exceed the applied scalar
//! (`VisualModTable::UpdateAtomCollection`), through `AddReplacementMatModAtom`.
//! `CDynamicMesh::ApplyMaterialModifier` then gives every material
//! instance whose `m_materialType` matches the atom its template and appends its modifier.
//!
//! Only material-modifier atoms are decoded: tables carrying any other atom kind are rejected.
//! The dungeon table, the one `CEnvCell` applies, carries only material modifiers.

use std::collections::HashMap;

use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::DatFileKind;
use ac2_dat::{ArchiveCursor, ArchiveError};
use ac2_db::{
    CachedResource, ResourceError, ResourcePackedDecode, ResourceResult, ResourceStore,
    ResourceTypeId, TypedResourceRef,
};

use crate::ac2_material::{Ac2MaterialInstance, Ac2MaterialModifier, Ac2RenderMaterial};
use crate::ac2_material_instance::apply_modifier;

/// `EnumIDMap` of enum-mapper groups (`DBCache::GetDIDFromEnumStatic`).
const ENUM_GROUP_MAP_DID: DataId = DataId::new(0x2800_0000);
/// `UNIQUEDB_GROUP_ENUM` and `DungeonAppearanceTable_UniqueDBID` (`CEnvCell::BuildAndApplyAtomCollection`).
const UNIQUE_DB_GROUP: u32 = 0x4000_0002;
const DUNGEON_APPEARANCE_TABLE: u32 = 0x4000_0008;
const APPEARANCE_TABLE_PREFIX: u32 = 0x2000_0000;
const MATERIAL_INSTANCE_PREFIX: u32 = 0x3100_0000;
/// Atom mesh/material type that matches every mesh or material.
const ANY_TYPE: u32 = 1;
const MAX_ITEMS: usize = 1 << 16;
const MAX_BASE_DEPTH: usize = 8;

/// One `MaterialModifierAtom`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ac2MaterialAtom {
    pub priority: f32,
    pub template: Option<DataId>,
    pub modifier: Option<DataId>,
    pub mesh_type: u32,
    pub material_type: u32,
}

impl Ac2MaterialAtom {
    /// `MaterialModifierAtom::Replaces`.
    fn replaces(&self, other: &Self) -> bool {
        other.priority < self.priority
            && (self.mesh_type == ANY_TYPE || self.mesh_type == other.mesh_type)
            && self.material_type == other.material_type
    }
}

#[derive(Debug)]
struct Ac2AppearanceTable {
    base: Vec<DataId>,
    /// Appearance key -> (intensity, material atoms) collections in stream order.
    atoms: HashMap<u32, Vec<(f32, Vec<Ac2MaterialAtom>)>>,
}

impl ResourcePackedDecode for Ac2AppearanceTable {
    const TYPE_NAME: &'static str = "AC2 AppearanceTable";
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
        let base = (0..count(&mut cursor, 4)?)
            .map(|_| cursor.read_u32().map(DataId::new))
            .collect::<Result<Vec<_>, _>>()?;
        let entries = (cursor.read_u32()? & 0xFFFF) as usize;
        let mut atoms = HashMap::with_capacity(entries);
        for _ in 0..entries {
            let key = cursor.read_u32()?;
            let mut collections = Vec::new();
            for _ in 0..count(&mut cursor, 28)? {
                let intensity = cursor.read_f32()?;
                let mut list = Vec::new();
                for _ in 0..count(&mut cursor, 20)? {
                    let priority = cursor.read_f32()?;
                    let did = |word: u32| Some(DataId::new(word)).filter(|did| !did.is_null());
                    list.push(Ac2MaterialAtom {
                        priority,
                        template: did(cursor.read_u32()?),
                        modifier: did(cursor.read_u32()?),
                        mesh_type: cursor.read_u32()?,
                        material_type: cursor.read_u32()?,
                    });
                }
                // Opacity, mesh, part-group, FX and icon atom lists.
                for _ in 0..5 {
                    if cursor.read_u32()? != 0 {
                        return Err(malformed("appearance atoms other than material modifiers"));
                    }
                }
                collections.push((intensity, list));
            }
            atoms.insert(key, collections);
        }
        let thresholds = (cursor.read_u32()? & 0xFFFF) as usize;
        cursor.read_bytes(thresholds * 8)?;
        if cursor.remaining() != 0 {
            return Err(malformed("trailing bytes after AppearanceTable"));
        }
        Ok(Self { base, atoms })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.base.len() * 4
            + self
                .atoms
                .values()
                .flatten()
                .map(|(_, list)| 8 + list.len() * std::mem::size_of::<Ac2MaterialAtom>())
                .sum::<usize>()
    }
}

/// The leading `m_EnumToID` table of an `EnumIDMap` (`EnumIDMap::StreamPack`).
#[derive(Debug)]
struct Ac2EnumIdMap {
    ids: HashMap<u32, u32>,
}

impl ResourcePackedDecode for Ac2EnumIdMap {
    const TYPE_NAME: &'static str = "AC2 EnumIDMap";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        let mut cursor = ArchiveCursor::new(bytes);
        if cursor.read_u32()? != qdid.data_id().raw() {
            return Err(ResourceError::MalformedTypedObject {
                qdid,
                type_name: Self::TYPE_NAME,
                context: "embedded DID did not match DAT identity",
            });
        }
        let entries = (cursor.read_u32()? & 0xFFFF) as usize;
        let mut ids = HashMap::with_capacity(entries);
        for _ in 0..entries {
            let key = cursor.read_u32()?;
            ids.insert(key, cursor.read_u32()?);
        }
        Ok(Self { ids })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.ids.len() * 8
    }
}

fn count(cursor: &mut ArchiveCursor<'_>, min_item_bytes: usize) -> Result<usize, ArchiveError> {
    let n = cursor.read_u32_count()?;
    if n > MAX_ITEMS || n * min_item_bytes > cursor.remaining() {
        return Err(ArchiveError::UnexpectedEof {
            position: cursor.position(),
            requested: n * min_item_bytes,
            len: cursor.bytes().len(),
        });
    }
    Ok(n)
}

fn load<T: ResourcePackedDecode + Send + Sync + 'static>(
    store: &ResourceStore,
    did: DataId,
) -> ResourceResult<Option<CachedResource<T>>> {
    store.load_cached_packed::<T>(TypedResourceRef::new(DatFileKind::Portal, did, 0))
}

/// `DBObj::GetByEnum(DungeonAppearanceTable_UniqueDBID, UNIQUEDB_GROUP_ENUM, DB_TYPE_APPEARANCE)`.
pub fn ac2_dungeon_appearance_table(store: &ResourceStore) -> ResourceResult<Option<DataId>> {
    let Some(groups) = load::<Ac2EnumIdMap>(store, ENUM_GROUP_MAP_DID)? else {
        return Ok(None);
    };
    let Some(map) = groups.ids.get(&UNIQUE_DB_GROUP).copied() else {
        return Ok(None);
    };
    let Some(map) = load::<Ac2EnumIdMap>(store, DataId::new(map))? else {
        return Ok(None);
    };
    Ok(map
        .ids
        .get(&DUNGEON_APPEARANCE_TABLE)
        .copied()
        .filter(|did| did & 0xFF00_0000 == APPEARANCE_TABLE_PREFIX)
        .map(DataId::new))
}

/// `AppearanceTable::GetAtoms` for an applied key and scalar: the material atoms in application
/// order after `AddReplacementMatModAtom` replacement.
pub fn ac2_appearance_material_atoms(
    store: &ResourceStore,
    table: DataId,
    key: u32,
    scalar: f32,
) -> ResourceResult<Vec<Ac2MaterialAtom>> {
    let mut atoms = Vec::new();
    collect_atoms(store, table, key, scalar, &mut atoms, 0)?;
    Ok(atoms)
}

fn collect_atoms(
    store: &ResourceStore,
    table: DataId,
    key: u32,
    scalar: f32,
    atoms: &mut Vec<Ac2MaterialAtom>,
    depth: usize,
) -> ResourceResult<()> {
    if depth > MAX_BASE_DEPTH {
        return Ok(());
    }
    let Some(table) = load::<Ac2AppearanceTable>(store, table)? else {
        return Ok(());
    };
    for base in &table.base {
        collect_atoms(store, *base, key, scalar, atoms, depth + 1)?;
    }
    let Some(collections) = table.atoms.get(&key) else {
        return Ok(());
    };
    let chosen = collections
        .iter()
        .take_while(|(intensity, _)| scalar >= *intensity)
        .last();
    for atom in chosen.map(|(_, list)| list.as_slice()).unwrap_or(&[]) {
        if atoms.iter().any(|existing| existing.replaces(atom)) {
            continue;
        }
        atoms.retain(|existing| !atom.replaces(existing));
        if !atoms.contains(atom) {
            atoms.push(*atom);
        }
    }
    Ok(())
}

/// A `MaterialInstance` resolved with appearance atoms applied: matching atoms
/// (`m_materialType`, or the any-type `1`) replace the template and append their modifiers
/// after the instance's own. Other materials resolve as [`crate::load_ac2_material`] does.
pub fn load_ac2_material_with_atoms(
    store: &ResourceStore,
    material_did: DataId,
    mesh_type: u32,
    atoms: &[Ac2MaterialAtom],
) -> ResourceResult<Option<Ac2RenderMaterial>> {
    if atoms.is_empty() || material_did.raw() & 0xFF00_0000 != MATERIAL_INSTANCE_PREFIX {
        return crate::load_ac2_material(store, material_did);
    }
    let Some(instance) = load::<Ac2MaterialInstance>(store, material_did)? else {
        return Ok(None);
    };
    let mut template = instance.material;
    let mut modifiers = instance.modifiers.clone();
    for atom in atoms {
        let mesh_matches = atom.mesh_type == ANY_TYPE || atom.mesh_type == mesh_type;
        let material_matches =
            atom.material_type == ANY_TYPE || atom.material_type == instance.material_type;
        if !(mesh_matches && material_matches) {
            continue;
        }
        if let Some(new_template) = atom.template {
            template = new_template;
        }
        if let Some(modifier) = atom.modifier {
            modifiers.push(modifier);
        }
    }
    let Some(template) = load::<Ac2RenderMaterial>(store, template)? else {
        return Ok(None);
    };
    let mut material = (*template).clone();
    for modifier in modifiers {
        if let Some(modifier) = load::<Ac2MaterialModifier>(store, modifier)? {
            apply_modifier(&mut material, &template, &modifier);
        }
    }
    Ok(Some(material))
}
