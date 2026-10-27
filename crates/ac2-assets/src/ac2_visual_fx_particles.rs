//! The FX particle nodes of an AC2 visual, resolved ahead of time on a loader thread so the frame
//! thread can run `FXManager::DoParticleFX` without reading: for every FX id the attachments
//! `FXTable::GetFXData` returns (the total `VisualDesc`'s override when the id has one, else the
//! static `DBFXTable` entry followed by its sub-tables'), each placed as
//! `PHYSICSOBJREP::CreateParticleSystem` places a startup system. The same walk as the sound
//! table in [`crate::ac2_visual_fx_sounds`], over the particle lists.

use std::collections::HashMap;
use std::sync::Arc;

use ac2_core::DataId;
use ac2_dat::DatFileKind;
use ac2_db::{ResourceResult, ResourceStore, TypedResourceRef};

use crate::ac2_drawn_groups::rest_named_bones;
use crate::ac2_fx::{Ac2FxParticleNode, Ac2FxTable};
use crate::ac2_visual_desc::Ac2VisualDesc;
use crate::ac2_visual_particles::{
    anchor, attaches, load, looping_flags, placement, Ac2ParticleAnchor, Ac2ParticleAttachment,
};
use ac2_anim::Ac2Setup;

const VISUAL_DESC_PREFIX: u32 = 0x1F00_0000;
const MAX_VISUAL_DESC_DEPTH: usize = 16;
const MAX_FX_TABLE_DEPTH: usize = 8;

/// A visual's FX id -> the particle systems and lights each starts (`FXManager::DoParticleFX`
/// and `FXManager::DoLightSourceFX`, one attachment per FX node).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Ac2VisualFxParticles {
    entries: HashMap<u32, Arc<[Ac2ParticleAttachment]>>,
    /// The startup FX ids (`m_startup_fx`), each once, in the order `PHYSICSOBJREP::InitFXManager`
    /// fills them.
    startup: Vec<u32>,
}

impl Ac2VisualFxParticles {
    /// A table over `entries`, for tests that build an object's FX without a DAT.
    pub fn from_entries(
        entries: impl IntoIterator<Item = (u32, Vec<Ac2ParticleAttachment>)>,
    ) -> Self {
        Self {
            entries: entries
                .into_iter()
                .map(|(id, list)| (id, list.into()))
                .collect(),
            startup: Vec::new(),
        }
    }

    /// The same table with `startup` as its startup FX ids (for tests).
    pub fn with_startup(mut self, startup: Vec<u32>) -> Self {
        self.startup = startup;
        self
    }

    /// The startup FX ids: `CPhysicsObj::InitObjectEnd` runs each one's `DoStartupFX` on the
    /// object it creates, which places the lights of the nodes that carry one.
    pub fn startup(&self) -> &[u32] {
        &self.startup
    }

    /// The nodes FX `fx_id` starts, `None` when it starts none.
    pub fn get(&self, fx_id: u32) -> Option<&Arc<[Ac2ParticleAttachment]>> {
        self.entries.get(&fx_id)
    }

    /// Every FX id and its systems, for the real-data checks.
    #[cfg(test)]
    pub(crate) fn entries_for_test(
        &self,
    ) -> impl Iterator<Item = (u32, &Arc<[Ac2ParticleAttachment]>)> + '_ {
        self.entries.iter().map(|(id, list)| (*id, list))
    }

    /// The bone name of every bone node in the table, once per node (`m_bone_name`), for checks
    /// that a visual's part groups have each of them (`CPartGroup::FindBoneType`).
    pub fn bone_names(&self) -> Vec<u32> {
        self.entries
            .values()
            .flat_map(|list| list.iter())
            .filter_map(|attachment| match attachment.anchor {
                Ac2ParticleAnchor::Bone { name, .. } => Some(name),
                _ => None,
            })
            .collect()
    }

    /// Whether the visual has any particle FX.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Resolves the FX particle nodes of an AC2 `VisualDesc` (`None` when it has none).
pub fn resolve_ac2_visual_fx_particles(
    store: &ResourceStore,
    visual: DataId,
) -> ResourceResult<Option<Ac2VisualFxParticles>> {
    if visual.raw() & 0xFF00_0000 != VISUAL_DESC_PREFIX {
        return Ok(None);
    }
    let mut chain = Vec::new();
    let mut current = Some(visual);
    while let Some(did) = current.filter(|_| chain.len() < MAX_VISUAL_DESC_DEPTH) {
        let Some(desc) = load::<Ac2VisualDesc>(store, did)? else {
            break;
        };
        current = desc.parent;
        chain.push(desc);
    }
    // `PartGroupDataDesc::ApplyAdditivePGDDesc`, root first: the first override per id and the
    // last non-zero table.
    let mut fx_table = None;
    let mut overrides = Vec::<(u32, Arc<[Ac2FxParticleNode]>)>::new();
    let mut startup = Vec::<u32>::new();
    for desc in chain.iter().rev() {
        fx_table = desc.fx.fx_table.or(fx_table);
        for (id, _) in desc.fx.startup.iter() {
            if !startup.contains(id) {
                startup.push(*id);
            }
        }
        for (id, nodes) in desc.fx.overrides.iter() {
            if !overrides.iter().any(|(known, _)| known == id) {
                overrides.push((*id, Arc::clone(nodes)));
            }
        }
    }
    let mut table_nodes = HashMap::<u32, Vec<Ac2FxParticleNode>>::new();
    if let Some(table) = fx_table {
        collect_table(store, table, 0, &mut table_nodes)?;
    }
    let mut nodes: HashMap<u32, Vec<Ac2FxParticleNode>> = table_nodes;
    for (id, list) in overrides {
        nodes.insert(id, list.iter().cloned().collect());
    }
    if nodes.is_empty() {
        return Ok(None);
    }
    let setup = match crate::resolve_ac2_visual_setup(store, visual)? {
        Some(setup) => load::<Ac2Setup>(store, setup)?,
        None => None,
    };
    let rest = setup
        .as_ref()
        .map(|setup| setup.rest_pose())
        .unwrap_or_default();
    let named = rest_named_bones(store, visual)?;
    let mut entries = HashMap::new();
    for (id, list) in nodes.into_iter().filter(|(_, list)| !list.is_empty()) {
        let mut attachments = Vec::with_capacity(list.len());
        for node in list {
            // A node whose bone the visual lacks starts no system but keeps its light.
            let particle_descs = if attaches(&named, &node) {
                node.particle_descs.clone()
            } else {
                Arc::from([])
            };
            let looping = looping_flags(store, &particle_descs)?;
            attachments.push(Ac2ParticleAttachment {
                placement: placement(setup.as_deref(), &rest, &named, &node),
                anchor: anchor(setup.as_deref(), &node),
                remove_rotation: node.remove_rotation,
                particle_descs,
                water_plane: node.water_plane,
                fire_and_forget: node.fire_and_forget,
                light: node.light,
                looping,
                object: node.object,
            });
        }
        entries.insert(id, Arc::from(attachments));
    }
    Ok(Some(Ac2VisualFxParticles { entries, startup }))
}

/// Every id of `table` and its sub-tables, each id's nodes in `DBFXTable::GetFXData` order.
fn collect_table(
    store: &ResourceStore,
    table: DataId,
    depth: usize,
    nodes: &mut HashMap<u32, Vec<Ac2FxParticleNode>>,
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
    for (id, list) in record.entries.iter() {
        nodes.entry(*id).or_default().extend(list.iter().cloned());
    }
    for sub in record.sub_tables.iter() {
        collect_table(store, *sub, depth + 1, nodes)?;
    }
    Ok(())
}
