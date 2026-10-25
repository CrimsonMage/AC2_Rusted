//! The FX sound nodes of an AC2 visual, resolved ahead of time on a loader thread so the frame
//! thread can run `FXManager::DoFX` without reading: the total `VisualDesc`'s FX table and
//! overrides (as `PHYSICSOBJREP::InitFXManager` fills `FXTable`), and for every FX id the node
//! list `FXTable::GetFXData` returns (the override when the id has one, else the static
//! `DBFXTable` entry followed by its sub-tables', `DBFXTable::GetFXData`).

use std::collections::HashMap;
use std::sync::Arc;

use ac2_core::DataId;
use ac2_dat::DatFileKind;
use ac2_db::{ResourceResult, ResourceStore, TypedResourceRef};

use crate::ac2_fx::{Ac2FxSoundNode, Ac2FxTable};
use crate::ac2_visual_desc::Ac2VisualDesc;

const VISUAL_DESC_PREFIX: u32 = 0x1F00_0000;
const MAX_VISUAL_DESC_DEPTH: usize = 16;
const MAX_FX_TABLE_DEPTH: usize = 8;

/// A visual's FX id -> sound nodes.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Ac2VisualFxSounds {
    entries: HashMap<u32, Arc<[Ac2FxSoundNode]>>,
}

impl Ac2VisualFxSounds {
    /// `FXTable::GetFXData(fx_id)`'s nodes.
    pub fn get(&self, fx_id: u32) -> Option<&Arc<[Ac2FxSoundNode]>> {
        self.entries.get(&fx_id)
    }

    /// Every sound any FX of the visual can play.
    pub fn sound_ids(&self) -> std::collections::BTreeSet<u32> {
        self.entries
            .values()
            .flat_map(|nodes| nodes.iter().flat_map(|node| node.sounds.iter().copied()))
            .collect()
    }

    /// Whether any FX of the visual can play a sound.
    pub fn has_sounds(&self) -> bool {
        self.entries
            .values()
            .any(|nodes| nodes.iter().any(|node| !node.sounds.is_empty()))
    }
}

/// Resolves the FX sound nodes of an AC2 `VisualDesc` (`None` when it has no FX at all).
pub fn resolve_ac2_visual_fx_sounds(
    store: &ResourceStore,
    visual: DataId,
) -> ResourceResult<Option<Ac2VisualFxSounds>> {
    if visual.raw() & 0xFF00_0000 != VISUAL_DESC_PREFIX {
        return Ok(None);
    }
    let load_desc = |did| {
        store.load_cached_packed::<Ac2VisualDesc>(TypedResourceRef::new(
            DatFileKind::Portal,
            did,
            0,
        ))
    };
    let mut chain = Vec::new();
    let mut current = Some(visual);
    while let Some(did) = current.filter(|_| chain.len() < MAX_VISUAL_DESC_DEPTH) {
        let Some(desc) = load_desc(did)? else {
            break;
        };
        current = desc.parent;
        chain.push(desc);
    }
    // `PartGroupDataDesc::ApplyAdditivePGDDesc`, root first: the first override per id and the
    // last non-zero table.
    let mut fx_table = None;
    let mut entries = HashMap::new();
    for desc in chain.iter().rev() {
        fx_table = desc.fx.fx_table.or(fx_table);
        for (id, nodes) in desc.fx.sound_overrides.iter() {
            entries.entry(*id).or_insert_with(|| Arc::clone(nodes));
        }
    }
    let mut table_nodes = HashMap::<u32, Vec<Ac2FxSoundNode>>::new();
    if let Some(table) = fx_table {
        collect_table(store, table, 0, &mut table_nodes)?;
    }
    for (id, nodes) in table_nodes {
        entries.entry(id).or_insert_with(|| Arc::from(nodes));
    }
    if entries.is_empty() {
        return Ok(None);
    }
    Ok(Some(Ac2VisualFxSounds { entries }))
}

/// Every id of `table` and its sub-tables, each id's nodes in `DBFXTable::GetFXData` order.
fn collect_table(
    store: &ResourceStore,
    table: DataId,
    depth: usize,
    nodes: &mut HashMap<u32, Vec<Ac2FxSoundNode>>,
) -> ResourceResult<()> {
    if depth >= MAX_FX_TABLE_DEPTH {
        return Ok(());
    }
    let Some(record) = store.load_cached_packed::<Ac2FxTable>(TypedResourceRef::new(
        DatFileKind::Portal,
        table,
        0,
    ))?
    else {
        return Ok(());
    };
    for (id, list) in record.sounds.iter() {
        nodes.entry(*id).or_default().extend(list.iter().cloned());
    }
    for sub in record.sub_tables.iter() {
        collect_table(store, *sub, depth + 1, nodes)?;
    }
    Ok(())
}
