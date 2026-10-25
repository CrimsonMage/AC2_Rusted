//! [`StringTables`] sources: a [`ResourceStore`] that reads on demand (worker threads) and a
//! preloaded set a frame-thread caller can hold.

use std::collections::HashMap;
use std::sync::Arc;

use ac2_core::DataId;

use crate::string_info::{StringStateRef, StringTableRef, StringTables};
use crate::string_state::{load_string_state, StringState};
use crate::string_table::{load_string_table, StringTable};
use crate::{CachedResource, ResourceStore};

/// String tables read from the local DAT through a [`ResourceStore`] on first use (the
/// store caches them). Reads files: use it off the frame thread.
pub struct StoreStringTables {
    store: ResourceStore,
    state: Option<CachedResource<StringState>>,
}

impl StoreStringTables {
    /// `StringStateInfo::Startup`: the `StringState` `state_did` is read once (0 or a missing
    /// record: no state). The Legions DAT's is 0x58000000, the DID enum
    /// [`crate::STRING_STATE_ENUM`] names in group [`crate::STRING_STATE_ENUM_GROUP`].
    pub fn new(store: ResourceStore, state_did: u32) -> Self {
        let state = (state_did != 0)
            .then(|| {
                load_string_state(&store, DataId::new(state_did))
                    .ok()
                    .flatten()
            })
            .flatten();
        Self { store, state }
    }

    pub fn store(&self) -> &ResourceStore {
        &self.store
    }
}

impl StringTables for StoreStringTables {
    fn table(&self, did: u32) -> Option<StringTableRef<'_>> {
        let table = load_string_table(&self.store, DataId::new(did)).ok()??;
        Some(Box::new(table))
    }

    fn string_state(&self) -> Option<StringStateRef<'_>> {
        let state: &StringState = self.state.as_deref()?;
        Some(Box::new(state))
    }
}

/// Tables loaded ahead of time; a table not in the set is a failed lookup. Cheap to clone.
#[derive(Debug, Clone, Default)]
pub struct PreloadedStringTables {
    tables: HashMap<u32, Arc<StringTable>>,
    state: Option<Arc<StringState>>,
}

impl PreloadedStringTables {
    pub fn insert_table(&mut self, did: u32, table: Arc<StringTable>) {
        self.tables.insert(did, table);
    }

    pub fn set_state(&mut self, state: Option<Arc<StringState>>) {
        self.state = state;
    }

    pub fn contains(&self, did: u32) -> bool {
        self.tables.contains_key(&did)
    }

    /// Reads `dids` and the private tables their entries name, plus the state `state_did`
    /// (0: none), from `store`. Reads files: call it on a worker.
    pub fn load(store: &ResourceStore, dids: &[u32], state_did: u32) -> Self {
        let mut out = Self::default();
        let mut pending: Vec<u32> = dids.to_vec();
        while let Some(did) = pending.pop() {
            if out.tables.contains_key(&did) {
                continue;
            }
            let Some(table) = load_string_table(store, DataId::new(did)).ok().flatten() else {
                continue;
            };
            let table = table.value_arc();
            let mut private: Vec<u32> = table.strings.values().map(|e| e.table).collect();
            private.retain(|&t| t != 0 && t != did);
            private.sort_unstable();
            private.dedup();
            pending.extend(private);
            out.tables.insert(did, table);
        }
        if state_did != 0 {
            out.state = load_string_state(store, DataId::new(state_did))
                .ok()
                .flatten()
                .map(|s| s.value_arc());
        }
        out
    }
}

impl StringTables for PreloadedStringTables {
    fn table(&self, did: u32) -> Option<StringTableRef<'_>> {
        let table: &StringTable = self.tables.get(&did)?;
        Some(Box::new(table))
    }

    fn string_state(&self) -> Option<StringStateRef<'_>> {
        let state: &StringState = self.state.as_deref()?;
        Some(Box::new(state))
    }
}
