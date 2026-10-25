//! Asheron's Call 2 appearance atoms: the `AppearanceTable` (`0x20`) entries that swap meshes and
//! part groups, set opacity and retexture a mesh's material instances, and material resolution
//! under those atoms.
//!
//! `AppearanceTable::StreamPack`: DID, counted base-table DIDs, then a
//! `PHashTable<key, VisualModTable>` (entry count in the low half of its header word) whose
//! tables are counted `AtomCollection`s (`AtomCollection::StreamPack`: intensity, then the
//! material-modifier, opacity, mesh-replacement and part-group-replacement atom lists, the FX atom
//! set and the icon atom list), then the threshold-count hash. `AppearanceTable::GetAtoms` merges
//! base tables first, then the last collection whose intensity does not exceed the applied scalar
//! (`VisualModTable::UpdateAtomCollection`, through the `AddReplacement*Atom` rules).
//! `CDynamicMesh::ApplyMaterialModifier` then gives every material instance whose
//! `m_materialType` matches the atom its template and appends its modifier. FX and icon atoms are
//! decoded past and dropped.

use std::collections::HashMap;

use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::DatFileKind;
use ac2_dat::{ArchiveCursor, ArchiveError};
use ac2_db::{
    CachedResource, ResourceError, ResourcePackedDecode, ResourceResult, ResourceStore,
    ResourceTypeId, TypedResourceRef,
};

use crate::ac2_fx::{count as fx_count, float, read_fx_data, skip, word, Malformed};
use crate::ac2_material::{Ac2MaterialInstance, Ac2MaterialModifier, Ac2RenderMaterial};
use crate::ac2_material_instance::apply_modifier;
use crate::ac2_visual_desc::{part_group, Ac2AppearanceHash, Ac2PartGroup};

/// `EnumIDMap` of enum-mapper groups (`DBCache::GetDIDFromEnumStatic`).
const ENUM_GROUP_MAP_DID: DataId = DataId::new(0x2800_0000);
/// `UNIQUEDB_GROUP_ENUM` and `DungeonAppearanceTable_UniqueDBID` (`CEnvCell::BuildAndApplyAtomCollection`).
const UNIQUE_DB_GROUP: u32 = 0x4000_0002;
const DUNGEON_APPEARANCE_TABLE: u32 = 0x4000_0008;
const APPEARANCE_TABLE_PREFIX: u32 = 0x2000_0000;
const MATERIAL_INSTANCE_PREFIX: u32 = 0x3100_0000;
/// Atom mesh/material type that matches every mesh or material.
const ANY_TYPE: u32 = 1;
/// `OpacityAtom::Clear`'s marker: the opacity is the applied scalar.
const OPACITY_FROM_SCALAR: f32 = -999.0;
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

/// One `OpacityAtom`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ac2OpacityAtom {
    pub priority: f32,
    pub opacity: f32,
}

/// One `MeshReplacementAtom`: meshes of `mesh_type` (1: every mesh) become `mesh`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ac2MeshAtom {
    pub priority: f32,
    pub mesh_type: u32,
    pub mesh: DataId,
}

/// One `PGDReplacementAtom`: the part group it swaps in.
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2PartGroupAtom {
    pub priority: f32,
    pub group: Ac2PartGroup,
    pub appearance: Ac2AppearanceHash,
}

/// The atoms an `AtomCollection` gathers, in application order.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Ac2Atoms {
    pub materials: Vec<Ac2MaterialAtom>,
    pub opacities: Vec<Ac2OpacityAtom>,
    pub meshes: Vec<Ac2MeshAtom>,
    pub part_groups: Vec<Ac2PartGroupAtom>,
}

/// `AddReplacementOpacityAtom` / `AddReplacementMeshReplacementAtom` /
/// `AddReplacementPGDReplacementAtom`: the first held atom the new one replaces takes its value; a
/// held atom that replaces it drops it; an identical one fails the collection; otherwise it joins.
fn add_replacing<T: Clone + PartialEq>(
    list: &mut Vec<T>,
    atom: &T,
    replaces: impl Fn(&T, &T) -> bool,
) -> bool {
    for held in list.iter_mut() {
        if replaces(atom, held) {
            *held = atom.clone();
            return true;
        }
        if replaces(held, atom) {
            return true;
        }
    }
    if list.contains(atom) {
        return false;
    }
    list.push(atom.clone());
    true
}

impl Ac2Atoms {
    /// `AtomCollection::AddReplacementMatModAtom`: a held atom that replaces the new one drops
    /// it; otherwise every held atom it replaces is removed, and it joins unless already held.
    fn add_material(&mut self, atom: &Ac2MaterialAtom) -> bool {
        if self.materials.iter().any(|held| held.replaces(atom)) {
            return true;
        }
        self.materials.retain(|held| !atom.replaces(held));
        if self.materials.contains(atom) {
            return false;
        }
        self.materials.push(*atom);
        true
    }

    /// `AtomCollection::UpdateAtomCollection` of `collection` at `intensity`: every kind but the
    /// part-group swaps, or only those. Stops at the first atom a rule refuses.
    fn update(&mut self, collection: &Ac2Atoms, intensity: f32, only_part_groups: bool) -> bool {
        if only_part_groups {
            return collection.part_groups.iter().all(|atom| {
                add_replacing(&mut self.part_groups, atom, |a, b| {
                    a.group.key == b.group.key && b.priority <= a.priority
                })
            });
        }
        for atom in &collection.materials {
            if !self.add_material(atom) {
                return false;
            }
        }
        for atom in &collection.opacities {
            let mut atom = *atom;
            if atom.opacity == OPACITY_FROM_SCALAR {
                atom.opacity = intensity;
            }
            if !add_replacing(&mut self.opacities, &atom, |a, b| b.priority <= a.priority) {
                return false;
            }
        }
        collection.meshes.iter().all(|atom| {
            add_replacing(&mut self.meshes, atom, |a, b| {
                a.mesh_type == b.mesh_type && b.priority <= a.priority
            })
        })
    }
}

#[derive(Debug)]
pub(crate) struct Ac2AppearanceTable {
    pub(crate) base: Vec<DataId>,
    /// Appearance key -> (intensity, atoms) collections in stream order.
    pub(crate) atoms: HashMap<u32, Vec<(f32, Ac2Atoms)>>,
}

/// `AtomCollection::StreamPack`.
fn read_collection(cursor: &mut ArchiveCursor<'_>) -> Result<(f32, Ac2Atoms), Malformed> {
    let intensity = float(cursor)?;
    let mut atoms = Ac2Atoms::default();
    let did = |word: u32| Some(DataId::new(word)).filter(|did| !did.is_null());
    for _ in 0..fx_count(cursor, 20)? {
        atoms.materials.push(Ac2MaterialAtom {
            priority: float(cursor)?,
            template: did(word(cursor)?),
            modifier: did(word(cursor)?),
            mesh_type: word(cursor)?,
            material_type: word(cursor)?,
        });
    }
    for _ in 0..fx_count(cursor, 8)? {
        atoms.opacities.push(Ac2OpacityAtom {
            priority: float(cursor)?,
            opacity: float(cursor)?,
        });
    }
    for _ in 0..fx_count(cursor, 12)? {
        atoms.meshes.push(Ac2MeshAtom {
            priority: float(cursor)?,
            mesh_type: word(cursor)?,
            mesh: DataId::new(word(cursor)?),
        });
    }
    for _ in 0..fx_count(cursor, 8)? {
        let priority = float(cursor)?;
        let decoded = part_group(cursor)?;
        atoms.part_groups.push(Ac2PartGroupAtom {
            priority,
            group: decoded.group,
            appearance: decoded.appearance,
        });
    }
    // `STREAMPACK_STL_SET` of `FXAtom`s: their FX ids, then their `FXData`.
    for _ in 0..fx_count(cursor, 8)? {
        let ids = fx_count(cursor, 4)?;
        skip(cursor, ids * 4)?;
        for _ in 0..fx_count(cursor, 8)? {
            read_fx_data(cursor)?;
        }
    }
    // `IconModAtom`s: an `IconLayerDesc` each (layer id, image DID, shift colour).
    let icons = fx_count(cursor, 12)?;
    skip(cursor, icons * 12)?;
    Ok((intensity, atoms))
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
            let collections = (0..count(&mut cursor, 28)?)
                .map(|_| read_collection(&mut cursor))
                .collect::<Result<Vec<_>, _>>()
                .map_err(malformed)?;
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
                .map(|(_, atoms)| {
                    40 + atoms.materials.len() * std::mem::size_of::<Ac2MaterialAtom>()
                        + atoms.opacities.len() * 8
                        + atoms.meshes.len() * 12
                        + atoms.part_groups.len() * 96
                })
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
    let mut atoms = Ac2Atoms::default();
    ac2_appearance_atoms(store, table, key, scalar, false, &mut atoms)?;
    Ok(atoms.materials)
}

/// `AppearanceTable::GetAtoms(applied key, atoms, onlyGetPGDatoms)`: the base tables' atoms
/// first, then the table's own collection for `scalar`, merged into `atoms`. A missing table
/// adds nothing.
pub fn ac2_appearance_atoms(
    store: &ResourceStore,
    table: DataId,
    key: u32,
    scalar: f32,
    only_part_groups: bool,
    atoms: &mut Ac2Atoms,
) -> ResourceResult<()> {
    collect_atoms(store, table, key, scalar, only_part_groups, atoms, 0)
}

fn collect_atoms(
    store: &ResourceStore,
    table: DataId,
    key: u32,
    scalar: f32,
    only_part_groups: bool,
    atoms: &mut Ac2Atoms,
    depth: usize,
) -> ResourceResult<()> {
    if depth > MAX_BASE_DEPTH {
        return Ok(());
    }
    let Some(table) = load::<Ac2AppearanceTable>(store, table)? else {
        return Ok(());
    };
    for base in &table.base {
        collect_atoms(
            store,
            *base,
            key,
            scalar,
            only_part_groups,
            atoms,
            depth + 1,
        )?;
    }
    let Some(collections) = table.atoms.get(&key) else {
        return Ok(());
    };
    // `VisualModTable::UpdateAtomCollection`: the last collection whose intensity the scalar
    // reaches.
    if let Some((_, chosen)) = collections
        .iter()
        .take_while(|(intensity, _)| scalar >= *intensity)
        .last()
    {
        atoms.update(chosen, scalar, only_part_groups);
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
