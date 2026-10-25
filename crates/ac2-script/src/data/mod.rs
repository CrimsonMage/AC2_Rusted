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
mod enum_map;

pub use db_types::{db_type, DB_TYPE_DID_MAPPER, DB_TYPE_ENUM_MAPPER, DB_TYPE_STRING_TABLE};
pub use enum_map::{EnumIdMap, EnumMapper};

use std::collections::{BTreeSet, HashMap, HashSet};
use std::ops::Deref;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use ac2_core::DataId;
use ac2_dat::{DatDataset, DatFileKind};
use ac2_db::{
    load_string_table, CachedResource, ResourcePackedDecode, ResourceStore, StringTable,
    TypedResourceRef,
};

use crate::vm::{DataRequest, ScriptData, StateRef};
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

/// Records held checked out of the store (preloads), and records known to be absent.
#[derive(Default)]
struct Resident {
    states: HashMap<u32, CachedResource<WState>>,
    mappers: HashMap<u32, CachedResource<EnumMapper>>,
    id_maps: HashMap<u32, CachedResource<EnumIdMap>>,
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
        };
        if !found {
            self.resident().absent.insert(request);
        }
    }

    fn read<T: ResourcePackedDecode>(&self, did: u32) -> Option<CachedResource<T>> {
        self.store
            .load_cached_packed::<T>(portal(did))
            .ok()
            .flatten()
    }

    /// Worker side: every enum mapper and DID mapper (105 small records), held resident.
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
