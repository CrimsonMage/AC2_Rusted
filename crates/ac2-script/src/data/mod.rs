//! The DAT data the VM and glue read, served from an ac2-db [`ResourceStore`].
//!
//! [`DatScriptData`] implements [`ScriptData`] over the store: WStates, the enum tables
//! (`EnumMapper`, `EnumIDMap`), `FileArray::GetDIDFromEnum` through the master `EnumIDMap`, and
//! `MasterDBMap::DivineType` by DID range. Loads happen on first use (the store decodes and
//! caches); a host that runs the VM on its frame thread calls
//! [`DatScriptData::preload_tables`] (and loads the WStates it needs) on a worker first.
//!
//! `FileArray::GetDID` (file name -> DID) has no data in the Legions install: retail fills
//! `FileArray` from `READONLY_DIDName.dat`, which the DVD does not ship, and the DATs hold no
//! `DBFile2IDTable` records, so names never resolve, as in retail.

mod db_types;
mod enum_map;

pub use db_types::{db_type, DB_TYPE_DID_MAPPER, DB_TYPE_ENUM_MAPPER, DB_TYPE_STRING_TABLE};
pub use enum_map::{EnumIdMap, EnumMapper};

use std::ops::Deref;
use std::sync::Arc;

use ac2_core::DataId;
use ac2_dat::{DatDataset, DatFileKind};
use ac2_db::{
    load_string_table, ResourcePackedDecode, ResourceStore, StringTable, TypedResourceRef,
};

use crate::vm::{ScriptData, StateRef};
use crate::wstate::load_wstate;

/// The portal.dat header's master map (`DBCache::m_MasterMapID`): the `EnumIDMap` of enum
/// groups.
pub const MASTER_MAP_DID: u32 = 0x2800_0000;
/// The enum group of enum mappers (`FileArray::GetDIDFromEnum(mapid, 0x40000001)`).
pub const EMAPPER_GROUP: u32 = 0x4000_0001;

/// [`ScriptData`] over the DATs.
pub struct DatScriptData {
    dataset: Arc<DatDataset>,
    store: ResourceStore,
}

impl DatScriptData {
    pub fn new(dataset: Arc<DatDataset>, store: ResourceStore) -> Self {
        Self { dataset, store }
    }

    pub fn store(&self) -> &ResourceStore {
        &self.store
    }

    fn load<T: ResourcePackedDecode>(&self, did: u32) -> Option<Box<dyn Deref<Target = T>>> {
        self.store
            .load_cached_packed::<T>(TypedResourceRef::new(
                DatFileKind::Portal,
                DataId::new(did),
                0,
            ))
            .ok()
            .flatten()
            .map(|r| Box::new(r) as Box<dyn Deref<Target = T>>)
    }

    /// Decodes every enum mapper and DID mapper (105 small records) into the store's cache.
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
                    self.load::<EnumMapper>(did);
                } else {
                    self.load::<EnumIdMap>(did);
                }
            }
        }
    }
}

impl ScriptData for DatScriptData {
    fn wstate(&self, did: u32) -> Option<StateRef> {
        load_wstate(&self.store, DataId::new(did))
            .ok()
            .flatten()
            .map(|state| Box::new(state) as StateRef)
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
        self.load::<EnumMapper>(did)
    }

    fn enum_id_map(&self, did: u32) -> Option<Box<dyn Deref<Target = EnumIdMap>>> {
        self.load::<EnumIdMap>(did)
    }

    /// `DBObj::Get(QualifiedDataID(did, DB_TYPE_STRING_TABLE))` through ac2-db's decoder.
    fn string_table(&self, did: u32) -> Option<Box<dyn Deref<Target = StringTable>>> {
        load_string_table(&self.store, DataId::new(did))
            .ok()
            .flatten()
            .map(|t| Box::new(t) as Box<dyn Deref<Target = StringTable>>)
    }
}
