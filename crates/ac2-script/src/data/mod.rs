//! The DAT data the VM and glue read, served from an ac2-db [`ResourceStore`].
//!
//! [`DatScriptData`] implements [`ScriptData`] over the store: WStates, the enum tables
//! (`EnumMapper`, `EnumIDMap`), `FileArray::GetDIDFromEnum` through the master `EnumIDMap`, and
//! `MasterDBMap::DivineType` by DID range.
//!
//! Retail read the DATs synchronously inside the interpreter. Here the frame thread never
//! reads: a host that runs the VM on its frame thread first calls
//! [`DatScriptData::preload_tables`] and [`DatScriptData::preload_states`] on a worker (they
//! check the records out of the store, so they stay resident), then
//! [`DatScriptData::set_cache_only`]. From then on a lookup that is not resident records a
//! [`DataRequest`] and answers "not yet"; the efun that asked waits and runs again once a
//! worker has run [`DatScriptData::load`] (clones share their state, so the worker holds a
//! clone).
//!
//! `FileArray::GetDID` (file name -> DID) has no data in the Legions install: retail fills
//! `FileArray` from `READONLY_DIDName.dat`, which the DVD does not ship, and the DATs hold no
//! `DBFile2IDTable` records, so names never resolve, as in retail.

mod db_types;
pub mod entity;
mod enum_map;
mod places;
mod render;
mod visual;

pub use db_types::{db_type, DB_TYPE_DID_MAPPER, DB_TYPE_ENUM_MAPPER, DB_TYPE_STRING_TABLE};
pub use enum_map::{EnumIdMap, EnumMapper};
pub use places::{PlacesTable, PLACES_TABLE_ENUM, UNIQUEDB_GROUP_ENUM};
pub use render::{render_info, script_error};

use std::collections::{BTreeSet, HashMap, HashSet};
use std::ops::Deref;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use ac2_assets::{Ac2EntityDesc, Ac2VisualDesc};
use ac2_core::DataId;
use ac2_dat::{ArchiveCursor, DatDataset, DatFileKind};
use ac2_db::{
    install_ac2_property_types, load_string_table, read_ac2_property_collection, CachedResource,
    ResourcePackedDecode, ResourceStore, StringTable, TypedResourceRef,
};

use crate::vm::{DataRequest, EntityProperties, ScriptData, StateRef};
use crate::wstate::{load_wstate, WState, WSTATE_BASE_DID};

/// The portal.dat header's master map (`DBCache::m_MasterMapID`): the `EnumIDMap` of enum
/// groups.
pub const MASTER_MAP_DID: u32 = 0x2800_0000;
/// The enum group of enum mappers (`FileArray::GetDIDFromEnum(mapid, 0x40000001)`).
pub const EMAPPER_GROUP: u32 = 0x4000_0001;

/// The package WStates (`0x70000000 | package id`: singletons and class defaults), what the
/// interpreter itself loads between instructions (`ReferenceTable::FillPackage`,
/// `AllocatePackage` for `new`): 12,583 records, about 46 MB decoded, a third of a second.
pub const STATE_RANGE: (u32, u32) = (WSTATE_BASE_DID + 0x0100_0000, WSTATE_BASE_DID + 0x0100_FFFF);

/// The packages of the data WStates the skill queries read.
pub const SKILL_STATE_PACKAGES: [&str; 3] = ["AdvancementTable", "LevelMappingTable", "CraftSkill"];

/// Records held checked out of the store (preloads), and records known to be absent.
#[derive(Default)]
struct Resident {
    states: HashMap<u32, CachedResource<WState>>,
    mappers: HashMap<u32, CachedResource<EnumMapper>>,
    id_maps: HashMap<u32, CachedResource<EnumIdMap>>,
    places: Option<CachedResource<PlacesTable>>,
    string_state: Option<CachedResource<ac2_db::StringState>>,
    entities: HashMap<u32, EntityProperties>,
    visuals: HashMap<u32, Arc<ac2_weenie::Appearance>>,
    defaults: HashMap<u32, Arc<ac2_weenie::WeenieQualities>>,
    absent: HashSet<DataRequest>,
}

#[derive(Default)]
struct Shared {
    cache_only: AtomicBool,
    missed: AtomicBool,
    resident: Mutex<Resident>,
    requests: Mutex<BTreeSet<DataRequest>>,
}

/// [`ScriptData`] over the DATs. Clones share their resident records and requests.
#[derive(Clone)]
pub struct DatScriptData {
    dataset: Arc<DatDataset>,
    store: ResourceStore,
    shared: Arc<Shared>,
}

fn portal(did: u32) -> TypedResourceRef {
    TypedResourceRef::new(DatFileKind::Portal, DataId::new(did), 0)
}

impl DatScriptData {
    pub fn new(dataset: Arc<DatDataset>, store: ResourceStore) -> Self {
        Self {
            dataset,
            store,
            shared: Arc::default(),
        }
    }

    pub fn store(&self) -> &ResourceStore {
        &self.store
    }

    /// From now on lookups only read what is resident; misses become [`DataRequest`]s.
    pub fn set_cache_only(&self, cache_only: bool) {
        self.shared.cache_only.store(cache_only, Ordering::Relaxed);
    }

    fn cache_only(&self) -> bool {
        self.shared.cache_only.load(Ordering::Relaxed)
    }

    fn resident(&self) -> std::sync::MutexGuard<'_, Resident> {
        self.shared
            .resident
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    }

    /// A lookup that found nothing resident: unknown records are requested.
    fn miss(&self, request: DataRequest) {
        if self.resident().absent.contains(&request) {
            return;
        }
        self.shared.missed.store(true, Ordering::Relaxed);
        self.shared
            .requests
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(request);
    }

    /// A record from the store: read when allowed, else resident or requested.
    fn get<T: ResourcePackedDecode>(
        &self,
        reference: TypedResourceRef,
        request: DataRequest,
        read: impl FnOnce() -> Option<CachedResource<T>>,
    ) -> Option<CachedResource<T>> {
        if !self.cache_only() {
            return read();
        }
        let found = self.store.try_peek_cached::<T>(reference);
        if found.is_none() {
            self.miss(request);
        }
        found
    }

    /// Worker side: reads `request` into the store (the next lookup finds it resident), or
    /// notes that the DATs do not hold it.
    pub fn load(&self, request: DataRequest) {
        let found = match request {
            DataRequest::WState(did) => load_wstate(&self.store, DataId::new(did))
                .ok()
                .flatten()
                .is_some(),
            DataRequest::EnumMapper(did) => self.read::<EnumMapper>(did).is_some(),
            DataRequest::EnumIdMap(did) => self.read::<EnumIdMap>(did).is_some(),
            DataRequest::StringTable(did) => load_string_table(&self.store, DataId::new(did))
                .ok()
                .flatten()
                .is_some(),
            DataRequest::PlacesTable => self.load_places(),
            DataRequest::EntityDesc(did) => match self.read_entity(did) {
                Some(properties) => {
                    self.resident().entities.insert(did, properties);
                    true
                }
                None => false,
            },
            DataRequest::VisualDesc(did) => match self.read::<Ac2VisualDesc>(did) {
                Some(desc) => {
                    let appearance = visual::appearance_of_dat(&desc);
                    self.resident().visuals.insert(did, Arc::new(appearance));
                    true
                }
                None => false,
            },
            DataRequest::ClassDefaults(did) => match self.read_defaults(did) {
                Some(defaults) => {
                    self.resident().defaults.insert(did, defaults);
                    true
                }
                None => false,
            },
        };
        if !found {
            self.resident().absent.insert(request);
        }
    }

    /// A qualities record's packed body (`DB_TYPE_QUALITIES`, the same layout as the player
    /// description's `CBaseQualities::UnPack`) as a weenie's qualities, read in full.
    fn read_defaults(&self, did: u32) -> Option<Arc<ac2_weenie::WeenieQualities>> {
        let packed = self
            .dataset
            .read_packed(DatFileKind::Portal, did)
            .ok()
            .flatten()?;
        let qualities = ac2_net::protocol::Qualities::from_bytes(&packed).ok()?;
        Some(Arc::new(ac2_weenie::WeenieQualities::from_unpacked(
            &qualities,
        )))
    }

    /// `DBObj::Get` of an entity description and its `PropertyCollection` (the MasterProperty
    /// table is installed from the same store first: the record's property collections are
    /// typed by it). A record without properties has none.
    fn read_entity(&self, did: u32) -> Option<EntityProperties> {
        // Installing is idempotent; without the table the record cannot be decoded.
        let _ = install_ac2_property_types(&self.store);
        let desc = self
            .store
            .load_cached_packed::<Ac2EntityDesc>(portal(did))
            .ok()
            .flatten()?;
        let Some(bytes) = desc.properties.as_ref() else {
            return Some(EntityProperties::default());
        };
        let mut cursor = ArchiveCursor::new(bytes);
        let properties = read_ac2_property_collection(&mut cursor).ok()?;
        Some(std::sync::Arc::new(properties))
    }

    fn read<T: ResourcePackedDecode>(&self, did: u32) -> Option<CachedResource<T>> {
        self.store
            .load_cached_packed::<T>(portal(did))
            .ok()
            .flatten()
    }

    /// Worker side: every enum mapper and DID mapper (105 small records) and the
    /// `PlacesTable`, held resident.
    pub fn preload_tables(&self) {
        let ranges = [
            (0x2300_0000, 0x23FF_FFFE, true),
            (0x2800_0000, 0x28FF_FFFE, false),
        ];
        for (first, last, mapper) in ranges {
            let ids: Vec<u32> = self
                .dataset
                .ids_in_range(DatFileKind::Portal, first, last)
                .collect();
            for did in ids {
                if mapper {
                    if let Some(r) = self.read::<EnumMapper>(did) {
                        self.resident().mappers.insert(did, r);
                    }
                } else if let Some(r) = self.read::<EnumIdMap>(did) {
                    self.resident().id_maps.insert(did, r);
                }
            }
        }
        if !self.load_places() {
            self.resident().absent.insert(DataRequest::PlacesTable);
        }
        // `StringStateInfo`'s state (number formats, plurals, meta letters).
        let did = self
            .enum_did(ac2_db::STRING_STATE_ENUM, ac2_db::STRING_STATE_ENUM_GROUP)
            .unwrap_or(ac2_db::STRING_STATE_FIRST);
        if let Ok(Some(state)) = ac2_db::load_string_state(&self.store, DataId::new(did)) {
            self.resident().string_state = Some(state);
        }
    }

    /// Worker side: the `PlacesTable`, held resident.
    fn load_places(&self) -> bool {
        let Some(did) = self.enum_did(PLACES_TABLE_ENUM, UNIQUEDB_GROUP_ENUM) else {
            return false;
        };
        match self.read::<PlacesTable>(did) {
            Some(table) => {
                self.resident().places = Some(table);
                true
            }
            None => false,
        }
    }

    /// Worker side: the WStates in [`STATE_RANGE`], held resident.
    pub fn preload_states(&self) {
        let (first, last) = STATE_RANGE;
        let ids: Vec<u32> = self
            .dataset
            .ids_in_range(DatFileKind::Portal, first, last)
            .collect();
        for did in ids {
            if let Ok(Some(state)) = load_wstate(&self.store, DataId::new(did)) {
                self.resident().states.insert(did, state);
            }
        }
    }

    /// Worker side: the WStates in `first..=last` saved from one of `packages` (the package id
    /// a WState record ends with), held resident. Only those records are decoded.
    pub fn preload_package_states(&self, first: u32, last: u32, packages: &[u32]) -> usize {
        let ids: Vec<u32> = self
            .dataset
            .ids_in_range(DatFileKind::Portal, first, last)
            .collect();
        let mut held = 0;
        for did in ids {
            let Ok(Some(bytes)) = self.dataset.read_packed(DatFileKind::Portal, did) else {
                continue;
            };
            let Some(tail) = bytes.len().checked_sub(4).map(|at| &bytes[at..]) else {
                continue;
            };
            let pid = u32::from_le_bytes([tail[0], tail[1], tail[2], tail[3]]);
            if !packages.contains(&pid) {
                continue;
            }
            if let Ok(Some(state)) = load_wstate(&self.store, DataId::new(did)) {
                if state.pid == Some(pid) {
                    self.resident().states.insert(did, state);
                    held += 1;
                }
            }
        }
        held
    }

    /// Worker side: the skill system's data WStates (`0x6F000000 | n`) the player's skill
    /// queries read (`gmCWInterface::GetSkillExperienceNeededToLevelByEnum` and kin): the
    /// packages in [`SKILL_STATE_PACKAGES`], held resident (about 230 records).
    pub fn preload_skill_states(&self, program: &crate::vm::Program) -> usize {
        let pids: Vec<u32> = SKILL_STATE_PACKAGES
            .iter()
            .filter_map(|name| program.package_id(name))
            .collect();
        self.preload_package_states(WSTATE_BASE_DID, WSTATE_BASE_DID + 0x00FF_FFFF, &pids)
    }

    fn table<T: ResourcePackedDecode>(
        &self,
        did: u32,
        resident: impl FnOnce(&Resident) -> Option<CachedResource<T>>,
        request: DataRequest,
    ) -> Option<Box<dyn Deref<Target = T>>> {
        if let Some(r) = resident(&self.resident()) {
            return Some(Box::new(r));
        }
        self.get(portal(did), request, || self.read::<T>(did))
            .map(|r| Box::new(r) as Box<dyn Deref<Target = T>>)
    }
}

impl ScriptData for DatScriptData {
    fn wstate(&self, did: u32) -> Option<StateRef> {
        if let Some(state) = self.resident().states.get(&did).cloned() {
            return Some(Box::new(state));
        }
        let (first, last) = STATE_RANGE;
        if self.cache_only() && (first..=last).contains(&did) && !self.resident().states.is_empty()
        {
            // The range is resident: a state not held does not exist.
            return None;
        }
        self.get(portal(did), DataRequest::WState(did), || {
            load_wstate(&self.store, DataId::new(did)).ok().flatten()
        })
        .map(|state| Box::new(state) as StateRef)
    }

    fn take_missed(&self) -> bool {
        self.shared.missed.swap(false, Ordering::Relaxed)
    }

    fn entity_properties(&self, did: u32) -> Option<EntityProperties> {
        if let Some(properties) = self.resident().entities.get(&did).cloned() {
            return Some(properties);
        }
        if !self.cache_only() {
            let properties = self.read_entity(did)?;
            self.resident().entities.insert(did, properties.clone());
            return Some(properties);
        }
        self.miss(DataRequest::EntityDesc(did));
        None
    }

    fn class_defaults(&self, did: u32) -> Option<Arc<ac2_weenie::WeenieQualities>> {
        if let Some(defaults) = self.resident().defaults.get(&did).cloned() {
            return Some(defaults);
        }
        if !self.cache_only() {
            let defaults = self.read_defaults(did)?;
            self.resident().defaults.insert(did, Arc::clone(&defaults));
            return Some(defaults);
        }
        self.miss(DataRequest::ClassDefaults(did));
        None
    }

    fn visual_desc(&self, did: u32) -> Option<Arc<ac2_weenie::Appearance>> {
        if let Some(visual) = self.resident().visuals.get(&did).cloned() {
            return Some(visual);
        }
        if !self.cache_only() {
            let desc = self.read::<Ac2VisualDesc>(did)?;
            let visual = Arc::new(visual::appearance_of_dat(&desc));
            self.resident().visuals.insert(did, Arc::clone(&visual));
            return Some(visual);
        }
        self.miss(DataRequest::VisualDesc(did));
        None
    }

    fn take_requests(&self) -> Vec<DataRequest> {
        std::mem::take(
            &mut *self
                .shared
                .requests
                .lock()
                .unwrap_or_else(|e| e.into_inner()),
        )
        .into_iter()
        .collect()
    }

    /// `DBCache::GetDIDFromEnum`: the group's `EnumIDMap` from the master map, then the value.
    fn enum_did(&self, value: u32, group: u32) -> Option<u32> {
        let master = self.enum_id_map(MASTER_MAP_DID)?;
        let map = self.enum_id_map(master.enum_to_did(group)?)?;
        map.enum_to_did(value)
    }

    fn db_type(&self, did: u32) -> Option<u32> {
        db_type(did)
    }

    fn enum_mapper(&self, did: u32) -> Option<Box<dyn Deref<Target = EnumMapper>>> {
        self.table(
            did,
            |r| r.mappers.get(&did).cloned(),
            DataRequest::EnumMapper(did),
        )
    }

    fn enum_id_map(&self, did: u32) -> Option<Box<dyn Deref<Target = EnumIdMap>>> {
        self.table(
            did,
            |r| r.id_maps.get(&did).cloned(),
            DataRequest::EnumIdMap(did),
        )
    }

    /// `PlacesTable::InqPlace`'s `DBObj::GetByEnum`.
    fn places_table(&self) -> Option<Box<dyn Deref<Target = PlacesTable>>> {
        if let Some(table) = self.resident().places.clone() {
            return Some(Box::new(table));
        }
        if self.cache_only() {
            self.miss(DataRequest::PlacesTable);
            return None;
        }
        self.load_places();
        let table = self.resident().places.clone()?;
        Some(Box::new(table))
    }

    /// `StringTable::GetString` (vtable +0x80) through ac2-db's meta-language renderer.
    fn render_string(
        &self,
        table: &StringTable,
        info: &crate::native::StringInfo,
    ) -> Result<Vec<u16>, crate::native::GetStringError> {
        ac2_db::render_with_table(self, table, &render_info(info)).map_err(script_error)
    }

    /// `DBObj::Get(QualifiedDataID(did, DB_TYPE_STRING_TABLE))` through ac2-db's decoder.
    fn string_table(&self, did: u32) -> Option<Box<dyn Deref<Target = StringTable>>> {
        let reference = TypedResourceRef::new(DatFileKind::Local, DataId::new(did), 0);
        self.get(reference, DataRequest::StringTable(did), || {
            load_string_table(&self.store, DataId::new(did))
                .ok()
                .flatten()
        })
        .map(|t| Box::new(t) as Box<dyn Deref<Target = StringTable>>)
    }
}
