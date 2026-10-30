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
//! `m_materialType` matches the atom its template and appends its modifier. The FX atoms
//! (`PartGroupDataDesc::FillFXTable` adds them to the FX table) and the icon atoms
//! (`IconDesc::ApplyAdditiveIconLayer` is what the icon composition runs them through) are kept
//! with their `Replaces` rules (`AtomCollection::AddReplacementFXAtom` / `AddReplacementIconModAtom`).
//! The key and threshold tables answer `AppearanceTable::HasAppearanceKey`,
//! `InqTsysAppearanceKeyModCount` and `InqAppearanceKeyModList`.

use std::collections::HashMap;
use std::sync::Arc;

use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::DatFileKind;
use ac2_dat::{ArchiveCursor, ArchiveError};
use ac2_db::{
    CachedResource, ResourceError, ResourcePackedDecode, ResourceResult, ResourceStore,
    ResourceTypeId, TypedResourceRef,
};

use crate::ac2_fx::{
    count as fx_count, float, read_fx_data, word, Ac2FxParticleNode, Ac2FxSoundNode,
    Ac2FxTerrainSet, FxDataParts, Malformed,
};
use crate::ac2_icon::Ac2IconLayer;
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

/// One `FXAtom`: the FX ids it answers and their `FXData` (reduced to the particle and sound
/// views, as the FX tables are).
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2FxAtom {
    /// `m_fx_ids`; the first one orders the atom in its set and decides what it replaces.
    pub ids: Arc<[u32]>,
    pub particles: Arc<[Ac2FxParticleNode]>,
    pub sounds: Arc<[Ac2FxSoundNode]>,
    /// The lists per terrain the atom's `FXData` key (`FXData::GetData`).
    pub terrains: Arc<[Ac2FxTerrainSet]>,
}

impl Ac2FxAtom {
    /// `FXAtom::Replaces`: the same first FX id.
    fn replaces(&self, other: &Self) -> bool {
        matches!((self.ids.first(), other.ids.first()), (Some(a), Some(b)) if a == b)
    }
}

/// The atoms an `AtomCollection` gathers, in application order.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Ac2Atoms {
    pub materials: Vec<Ac2MaterialAtom>,
    pub opacities: Vec<Ac2OpacityAtom>,
    pub meshes: Vec<Ac2MeshAtom>,
    pub part_groups: Vec<Ac2PartGroupAtom>,
    /// `m_fx`, a `std::set<FXAtom>` ordered by first FX id.
    pub fx: Vec<Ac2FxAtom>,
    /// `m_iconModifiers`, in insertion order.
    pub icons: Vec<Ac2IconLayer>,
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
        for atom in &collection.meshes {
            if !add_replacing(&mut self.meshes, atom, |a, b| {
                a.mesh_type == b.mesh_type && b.priority <= a.priority
            }) {
                return false;
            }
        }
        for atom in &collection.fx {
            self.add_fx(atom);
        }
        collection.icons.iter().all(|atom| self.add_icon(atom))
    }

    /// `AtomCollection::AddReplacementFXAtom`: the first held atom the new one replaces leaves
    /// the set, then the new atom joins at its place in the set's order (an atom with the same
    /// first id cannot be there, so the insert is unique). Always answers 1.
    fn add_fx(&mut self, atom: &Ac2FxAtom) {
        if let Some(at) = self.fx.iter().position(|held| atom.replaces(held)) {
            self.fx.remove(at);
        }
        // `FXAtom::operator<`: an atom with no ids sorts before everything.
        let at = self
            .fx
            .iter()
            .position(|held| match (atom.ids.first(), held.ids.first()) {
                (Some(new), Some(held)) => new < held,
                _ => true,
            })
            .unwrap_or(self.fx.len());
        self.fx.insert(at, atom.clone());
    }

    /// `AtomCollection::AddReplacementIconModAtom`: the first held atom the new one replaces is
    /// removed; an equal atom still held refuses it; otherwise it joins at the end.
    fn add_icon(&mut self, atom: &Ac2IconLayer) -> bool {
        if let Some(at) = self.icons.iter().position(|held| atom.replaces(held)) {
            self.icons.remove(at);
        }
        if self.icons.contains(atom) {
            return false;
        }
        self.icons.push(*atom);
        true
    }
}

#[derive(Debug)]
pub(crate) struct Ac2AppearanceTable {
    pub(crate) base: Vec<DataId>,
    /// Appearance key -> (intensity, atoms) collections in stream order.
    pub(crate) atoms: HashMap<u32, Vec<(f32, Ac2Atoms)>>,
    /// `m_threshCountHash`: appearance key -> the count `InqTsysAppearanceKeyModCount` answers.
    pub(crate) threshold_counts: HashMap<u32, u32>,
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
    // `STREAMPACK_STL_SET` of `FXAtom`s: their FX ids, then their `FXData`. The set is rebuilt in
    // stream order, so each atom goes in at its place in the set's order.
    for _ in 0..fx_count(cursor, 8)? {
        let ids = (0..fx_count(cursor, 4)?)
            .map(|_| word(cursor))
            .collect::<Result<Vec<_>, _>>()?;
        let list = (0..fx_count(cursor, 8)?)
            .map(|_| read_fx_data(cursor))
            .collect::<Result<Vec<_>, _>>()?;
        let (particles, sounds, terrains) = FxDataParts::segment(list);
        let atom = Ac2FxAtom {
            ids: Arc::from(ids),
            particles,
            sounds,
            terrains,
        };
        let at = atoms
            .fx
            .iter()
            .position(|held| match (atom.ids.first(), held.ids.first()) {
                (Some(new), Some(held)) => new < held,
                _ => true,
            })
            .unwrap_or(atoms.fx.len());
        atoms.fx.insert(at, atom);
    }
    // `IconModAtom`s: an `IconLayerDesc` each (layer id, image DID, shift colour).
    for _ in 0..fx_count(cursor, 12)? {
        atoms.icons.push(Ac2IconLayer {
            layer_id: word(cursor)?,
            image: word(cursor)?,
            shift_color: word(cursor)?,
        });
    }
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
        let mut threshold_counts = HashMap::with_capacity(thresholds);
        for _ in 0..thresholds {
            let key = cursor.read_u32()?;
            threshold_counts.insert(key, cursor.read_u32()?);
        }
        if cursor.remaining() != 0 {
            return Err(malformed("trailing bytes after AppearanceTable"));
        }
        Ok(Self {
            base,
            atoms,
            threshold_counts,
        })
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
                        + atoms.fx.len() * 64
                        + atoms.icons.len() * 12
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

/// `DBObj::GetByEnum(value, group, type)`'s DID: the group's `EnumIDMap` (from the enum-mapper
/// group map `0x28000000`), then `value` in it.
pub fn ac2_enum_did(
    store: &ResourceStore,
    group: u32,
    value: u32,
) -> ResourceResult<Option<DataId>> {
    let Some(groups) = load::<Ac2EnumIdMap>(store, ENUM_GROUP_MAP_DID)? else {
        return Ok(None);
    };
    let Some(map) = groups.ids.get(&group).copied() else {
        return Ok(None);
    };
    let Some(map) = load::<Ac2EnumIdMap>(store, DataId::new(map))? else {
        return Ok(None);
    };
    Ok(map.ids.get(&value).copied().map(DataId::new))
}

/// `DBObj::GetByEnum(DungeonAppearanceTable_UniqueDBID, UNIQUEDB_GROUP_ENUM, DB_TYPE_APPEARANCE)`.
pub fn ac2_dungeon_appearance_table(store: &ResourceStore) -> ResourceResult<Option<DataId>> {
    Ok(
        ac2_enum_did(store, UNIQUE_DB_GROUP, DUNGEON_APPEARANCE_TABLE)?
            .filter(|did| did.raw() & 0xFF00_0000 == APPEARANCE_TABLE_PREFIX),
    )
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

/// `DBObj::Get` of the `AppearanceTable` at `table` (`AppearanceTableAdaptor::
/// InqAppearanceKeyModList`'s first step); `None` when it is missing.
pub fn load_ac2_appearance_table(
    store: &ResourceStore,
    table: DataId,
) -> ResourceResult<Option<AppearanceTable>> {
    Ok(load::<Ac2AppearanceTable>(store, table)?.map(AppearanceTable))
}

/// A loaded `AppearanceTable`, for [`AppearanceTable::key_mods`].
pub struct AppearanceTable(CachedResource<Ac2AppearanceTable>);

impl AppearanceTable {
    /// `AppearanceTable::InqAppearanceKeyModList(key)`: the intensities of the collections `key`
    /// holds, base tables first, each placed by `List<float>::insert_sort` (ascending, equal
    /// values after the held ones).
    pub fn key_mods(&self, store: &ResourceStore, key: u32) -> ResourceResult<Vec<f32>> {
        let mut list = Vec::new();
        key_mods(store, &self.0, key, &mut list, 0)?;
        Ok(list)
    }
}

fn key_mods(
    store: &ResourceStore,
    table: &Ac2AppearanceTable,
    key: u32,
    list: &mut Vec<f32>,
    depth: usize,
) -> ResourceResult<()> {
    if depth > MAX_BASE_DEPTH {
        return Ok(());
    }
    for base in &table.base {
        if let Some(base) = load::<Ac2AppearanceTable>(store, *base)? {
            key_mods(store, &base, key, list, depth + 1)?;
        }
    }
    for (intensity, _) in table.atoms.get(&key).into_iter().flatten() {
        let at = list
            .iter()
            .position(|held| intensity < held)
            .unwrap_or(list.len());
        list.insert(at, *intensity);
    }
    Ok(())
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

/// `AppearanceTable::HasAppearanceKey`: the key is in a base table (searched first, in order) or
/// in this one's own key hash. `None` is a table that does not load (the adaptor's miss).
pub fn ac2_has_appearance_key(
    store: &ResourceStore,
    table: DataId,
    key: u32,
) -> ResourceResult<Option<bool>> {
    let Some(table) = load::<Ac2AppearanceTable>(store, table)? else {
        return Ok(None);
    };
    Ok(Some(has_key(store, &table, key, 0)?))
}

fn has_key(
    store: &ResourceStore,
    table: &Ac2AppearanceTable,
    key: u32,
    depth: usize,
) -> ResourceResult<bool> {
    if depth <= MAX_BASE_DEPTH {
        for base in &table.base {
            if let Some(base) = load::<Ac2AppearanceTable>(store, *base)? {
                if has_key(store, &base, key, depth + 1)? {
                    return Ok(true);
                }
            }
        }
    }
    Ok(table.atoms.contains_key(&key))
}

/// `AppearanceTable::InqTsysAppearanceKeyModCount` (through
/// `AppearanceTableAdaptor::InqTsysAppearanceKeyModCount`): the count the table's own threshold
/// hash holds for `key` (base tables are not searched). The outer `None` is a table that does
/// not load.
pub fn ac2_appearance_key_mod_count(
    store: &ResourceStore,
    table: DataId,
    key: u32,
) -> ResourceResult<Option<Option<u32>>> {
    Ok(load::<Ac2AppearanceTable>(store, table)?
        .map(|table| table.threshold_counts.get(&key).copied()))
}

/// `AppearanceTable::InqAppearanceKeyModList` (through the adaptor, which first empties the
/// list): the intensities of the key's atom collections, the base tables' first, each inserted
/// with `List<float>::insert_sort` (before the first element it is less than, so ascending with
/// equal values kept in arrival order). `None` is a table that does not load.
pub fn ac2_appearance_key_mod_list(
    store: &ResourceStore,
    table: DataId,
    key: u32,
) -> ResourceResult<Option<Vec<f32>>> {
    let Some(table) = load::<Ac2AppearanceTable>(store, table)? else {
        return Ok(None);
    };
    let mut list = Vec::new();
    collect_mod_list(store, &table, key, &mut list, 0)?;
    Ok(Some(list))
}

fn collect_mod_list(
    store: &ResourceStore,
    table: &Ac2AppearanceTable,
    key: u32,
    list: &mut Vec<f32>,
    depth: usize,
) -> ResourceResult<()> {
    if depth <= MAX_BASE_DEPTH {
        for base in &table.base {
            if let Some(base) = load::<Ac2AppearanceTable>(store, *base)? {
                collect_mod_list(store, &base, key, list, depth + 1)?;
            }
        }
    }
    for (intensity, _) in table.atoms.get(&key).into_iter().flatten() {
        let at = list
            .iter()
            .position(|held| *intensity < *held)
            .unwrap_or(list.len());
        list.insert(at, *intensity);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn fx(ids: &[u32]) -> Ac2FxAtom {
        Ac2FxAtom {
            ids: Arc::from(ids),
            particles: Arc::from([]),
            sounds: Arc::from([]),
            terrains: Arc::from([]),
        }
    }

    #[test]
    fn an_fx_atom_replaces_the_held_atom_with_the_same_first_id_and_the_set_stays_ordered() {
        let mut atoms = Ac2Atoms::default();
        atoms.add_fx(&fx(&[30, 31]));
        atoms.add_fx(&fx(&[10]));
        atoms.add_fx(&fx(&[20]));
        let firsts = |atoms: &Ac2Atoms| atoms.fx.iter().map(|a| a.ids[0]).collect::<Vec<_>>();
        assert_eq!(firsts(&atoms), [10, 20, 30]);
        // The same first id replaces the held atom whatever its other ids are.
        atoms.add_fx(&fx(&[30, 99]));
        assert_eq!(firsts(&atoms), [10, 20, 30]);
        assert_eq!(atoms.fx[2].ids.as_ref(), [30, 99]);
    }

    #[test]
    fn an_icon_atom_replaces_a_matching_layer_and_an_equal_one_is_not_added_twice() {
        let layer = |layer_id, image, shift_color| Ac2IconLayer {
            layer_id,
            image,
            shift_color,
        };
        let mut atoms = Ac2Atoms::default();
        assert!(atoms.add_icon(&layer(1, 0x0600_0001, 0)));
        assert!(atoms.add_icon(&layer(2, 0, 0xFF00_00FF)));
        // Same layer, still an image and no colour: replaces the first.
        assert!(atoms.add_icon(&layer(1, 0x0600_0002, 0)));
        assert_eq!(
            atoms.icons,
            [layer(2, 0, 0xFF00_00FF), layer(1, 0x0600_0002, 0)]
        );
        // An image-less layer 1 does not replace the imaged one: both are held.
        assert!(atoms.add_icon(&layer(1, 0, 0xFF)));
        assert_eq!(atoms.icons.len(), 3);
        // Replacing then equal: the replaced atom leaves and the new one joins again.
        assert!(atoms.add_icon(&layer(1, 0x0600_0002, 0)));
        assert_eq!(atoms.icons.len(), 3);
    }

    #[test]
    fn update_merges_fx_and_icon_atoms_after_the_mesh_atoms() {
        let mut collection = Ac2Atoms::default();
        collection.fx.push(fx(&[5]));
        collection.icons.push(Ac2IconLayer {
            layer_id: 3,
            image: 0x0600_0003,
            shift_color: 0,
        });
        let mut all = Ac2Atoms::default();
        assert!(all.update(&collection, 0.5, false));
        assert_eq!((all.fx.len(), all.icons.len()), (1, 1));
        // Only the part-group pass leaves them out.
        let mut groups = Ac2Atoms::default();
        assert!(groups.update(&collection, 0.5, true));
        assert_eq!((groups.fx.len(), groups.icons.len()), (0, 0));
    }
}
