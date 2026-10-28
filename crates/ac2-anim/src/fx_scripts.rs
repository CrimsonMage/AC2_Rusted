//! The DAT's FX scripts (`FxScript`, DIDs `0x57xxxxxx`) that a body's `DoFXScript` plays. The
//! records are read once, when the client attaches its DAT (a few dozen small records), so the
//! physics step only looks them up (`FXTrack::FillCache` in the body's tracker).

use std::collections::HashMap;
use std::sync::Arc;

use crate::{load_ac2_fx_script, Ac2FxScript, FX_SCRIPT_PREFIX};
use ac2_dat::DatFileKind;
use ac2_db::ResourceStore;

/// Every FX script of the portal DAT, by DID.
#[derive(Debug, Clone, Default)]
pub struct FxScripts {
    scripts: HashMap<u32, Arc<Ac2FxScript>>,
}

impl FxScripts {
    /// The script `id` (`FXTrack::FillCache`), `None` when the DAT has no such record.
    pub fn get(&self, id: u32) -> Option<Arc<Ac2FxScript>> {
        self.scripts.get(&id).cloned()
    }

    /// How many scripts the library holds.
    pub fn len(&self) -> usize {
        self.scripts.len()
    }

    /// Whether the library holds no script.
    pub fn is_empty(&self) -> bool {
        self.scripts.is_empty()
    }
}

/// Reads every `0x57` FX script record of the portal DAT. A record that fails to decode is left
/// out (its `DoFXScript` then does nothing, as a missing record does).
pub fn load_fx_scripts(store: &ResourceStore) -> FxScripts {
    let ids: Vec<u32> = store
        .dataset()
        .ids_in_range(
            DatFileKind::Portal,
            FX_SCRIPT_PREFIX,
            FX_SCRIPT_PREFIX | 0x00FF_FFFF,
        )
        .collect();
    let scripts = ids
        .into_iter()
        .filter_map(|id| {
            let script = load_fx_script_record(store, id).ok().flatten()?;
            Some((id, script))
        })
        .collect();
    FxScripts { scripts }
}

/// One record, copied out of the DAT cache (the library keeps its own `Arc`).
fn load_fx_script_record(
    store: &ResourceStore,
    id: u32,
) -> ac2_db::ResourceResult<Option<Arc<Ac2FxScript>>> {
    Ok(load_ac2_fx_script(store, id)?.map(|cached| Arc::new((*cached).clone())))
}
