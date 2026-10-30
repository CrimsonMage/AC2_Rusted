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

use crate::ac2_fx::{Ac2FxSoundNode, Ac2FxTable, Ac2FxTerrainSet, FxSegments};
use crate::ac2_visual_desc::Ac2VisualDesc;

const VISUAL_DESC_PREFIX: u32 = 0x1F00_0000;
const MAX_VISUAL_DESC_DEPTH: usize = 16;
const MAX_FX_TABLE_DEPTH: usize = 8;

/// A visual's FX id -> sound nodes.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Ac2VisualFxSounds {
    entries: HashMap<u32, Arc<[Ac2FxSoundNode]>>,
    /// The lists of the ids whose data key terrain nodes, per (FX id, terrain).
    terrains: HashMap<(u32, u32), Arc<[Ac2FxSoundNode]>>,
}

impl Ac2VisualFxSounds {
    /// `FXTable::GetFXData(fx_id)`'s nodes.
    pub fn get(&self, fx_id: u32) -> Option<&Arc<[Ac2FxSoundNode]>> {
        self.get_on(fx_id, 0)
    }

    /// `FXTable::GetFXData(fx_id)`'s nodes for an object on `terrain` (`FXData::GetData` of each
    /// `FXData`: the terrain's node where the data keys one, else its default).
    pub fn get_on(&self, fx_id: u32, terrain: u32) -> Option<&Arc<[Ac2FxSoundNode]>> {
        self.terrains
            .get(&(fx_id, terrain))
            .or_else(|| self.entries.get(&fx_id))
    }

    /// The (FX id, terrain) pairs whose data key a terrain node, for the real-data checks.
    #[cfg(test)]
    pub(crate) fn terrain_keys_for_test(&self) -> Vec<(u32, u32)> {
        self.terrains.keys().copied().collect()
    }

    /// Every sound any FX of the visual can play.
    pub fn sound_ids(&self) -> std::collections::BTreeSet<u32> {
        self.entries
            .values()
            .chain(self.terrains.values())
            .flat_map(|nodes| nodes.iter().flat_map(|node| node.sounds.iter().copied()))
            .collect()
    }

    /// Whether any FX of the visual can play a sound.
    pub fn has_sounds(&self) -> bool {
        self.entries
            .values()
            .chain(self.terrains.values())
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
    type Override = (Arc<[Ac2FxSoundNode]>, Arc<[Ac2FxTerrainSet]>);
    let mut overrides = HashMap::<u32, Override>::new();
    for desc in chain.iter().rev() {
        fx_table = desc.fx.fx_table.or(fx_table);
        for (id, nodes) in desc.fx.sound_overrides.iter() {
            overrides.entry(*id).or_insert_with(|| {
                let terrain = desc
                    .fx
                    .terrain_overrides
                    .iter()
                    .find(|(t, _)| t == id)
                    .map_or_else(|| Arc::from([]), |(_, sets)| Arc::clone(sets));
                (Arc::clone(nodes), terrain)
            });
        }
    }
    // `PartGroupDataDesc::FillFXTable`: with FX at all (`HasFX`), the global appearance keys' FX
    // atoms join the table behind the overrides.
    if fx_table.is_some() || !overrides.is_empty() {
        for atom in crate::ac2_drawn_groups::global_fx_atoms(store, visual)? {
            for id in atom.ids.iter() {
                overrides
                    .entry(*id)
                    .or_insert_with(|| (Arc::clone(&atom.sounds), Arc::clone(&atom.terrains)));
            }
        }
    }
    let mut segments = HashMap::<u32, FxSegments>::new();
    if let Some(table) = fx_table {
        collect_table(store, table, 0, &mut segments)?;
    }
    for (id, (nodes, terrains)) in overrides {
        let mut one = FxSegments::default();
        one.push(Arc::from([]), nodes, terrains);
        segments.insert(id, one);
    }
    if segments.is_empty() {
        return Ok(None);
    }
    let mut entries = HashMap::new();
    let mut terrains = HashMap::new();
    for (id, segs) in segments {
        entries.insert(id, Arc::from(segs.sounds()));
        for set in segs.terrain_sets() {
            terrains.insert((id, set.terrain), set.sounds);
        }
    }
    Ok(Some(Ac2VisualFxSounds { entries, terrains }))
}

/// Every id of `table` and its sub-tables, each id's nodes in `DBFXTable::GetFXData` order.
fn collect_table(
    store: &ResourceStore,
    table: DataId,
    depth: usize,
    segments: &mut HashMap<u32, FxSegments>,
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
        let terrains = record
            .terrains
            .iter()
            .find(|(t, _)| t == id)
            .map_or_else(|| Arc::from([]), |(_, sets)| Arc::clone(sets));
        segments
            .entry(*id)
            .or_default()
            .push(Arc::from([]), Arc::clone(list), terrains);
    }
    for sub in record.sub_tables.iter() {
        collect_table(store, *sub, depth + 1, segments)?;
    }
    Ok(())
}
