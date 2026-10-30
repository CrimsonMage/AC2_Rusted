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
use crate::ac2_fx::{
    Ac2FxAttach, Ac2FxAttachPoint, Ac2FxObject, Ac2FxObjectAttach, Ac2FxParticleNode, Ac2FxTable,
    Ac2FxTerrainSet, FxSegments,
};
use crate::ac2_visual_desc::Ac2VisualDesc;
use crate::ac2_visual_particles::{
    anchor, attaches, load, looping_flags, placement, Ac2ParticleAnchor, Ac2ParticleAttachment,
};
use ac2_anim::{Ac2Setup, Ac2Transform};
use ac2_math::Quat;

const VISUAL_DESC_PREFIX: u32 = 0x1F00_0000;
const MAX_VISUAL_DESC_DEPTH: usize = 16;
const MAX_FX_TABLE_DEPTH: usize = 8;

/// A visual's FX id -> the particle systems and lights each starts (`FXManager::DoParticleFX`
/// and `FXManager::DoLightSourceFX`, one attachment per FX node).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Ac2VisualFxParticles {
    entries: HashMap<u32, Arc<[Ac2ParticleAttachment]>>,
    /// The lists of the FX ids whose data key terrain nodes, per (FX id, terrain)
    /// (`FXData::GetData(m_terrain)`); an id off its keyed terrains uses `entries`.
    terrains: HashMap<(u32, u32), Arc<[Ac2ParticleAttachment]>>,
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
            terrains: HashMap::new(),
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

    /// The nodes FX `fx_id` starts, `None` when it starts none (an object on terrain 0, the
    /// `FXManager::m_terrain` a new manager has).
    pub fn get(&self, fx_id: u32) -> Option<&Arc<[Ac2ParticleAttachment]>> {
        self.get_on(fx_id, 0)
    }

    /// The nodes FX `fx_id` starts for an object on `terrain` (`FXData::GetData(m_terrain)` of
    /// each `FXData` of the id): the terrain's node where the data keys one, else its default.
    pub fn get_on(&self, fx_id: u32, terrain: u32) -> Option<&Arc<[Ac2ParticleAttachment]>> {
        self.terrains
            .get(&(fx_id, terrain))
            .or_else(|| self.entries.get(&fx_id))
    }

    /// A table with `terrain`'s list for `fx_id` (for tests).
    pub fn with_terrain(
        mut self,
        fx_id: u32,
        terrain: u32,
        list: Vec<Ac2ParticleAttachment>,
    ) -> Self {
        self.terrains.insert((fx_id, terrain), list.into());
        self
    }

    /// Every FX id and its systems, for the real-data checks.
    #[cfg(test)]
    pub(crate) fn entries_for_test(
        &self,
    ) -> impl Iterator<Item = (u32, &Arc<[Ac2ParticleAttachment]>)> + '_ {
        self.entries.iter().map(|(id, list)| (*id, list))
    }

    /// The (FX id, terrain) pairs whose data key a terrain node, for the real-data checks.
    #[cfg(test)]
    pub(crate) fn terrain_keys_for_test(&self) -> Vec<(u32, u32)> {
        self.terrains.keys().copied().collect()
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
        self.entries.is_empty() && self.terrains.is_empty()
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
    type Override = (u32, Arc<[Ac2FxParticleNode]>, Arc<[Ac2FxTerrainSet]>);
    let mut overrides = Vec::<Override>::new();
    let mut startup = Vec::<u32>::new();
    for desc in chain.iter().rev() {
        fx_table = desc.fx.fx_table.or(fx_table);
        for (id, _) in desc.fx.startup.iter() {
            if !startup.contains(id) {
                startup.push(*id);
            }
        }
        for (id, nodes) in desc.fx.overrides.iter() {
            if !overrides.iter().any(|(known, ..)| known == id) {
                let terrain = desc
                    .fx
                    .terrain_overrides
                    .iter()
                    .find(|(t, _)| t == id)
                    .map_or_else(|| Arc::from([]), |(_, sets)| Arc::clone(sets));
                overrides.push((*id, Arc::clone(nodes), terrain));
            }
        }
    }
    // `PartGroupDataDesc::FillFXTable`: with FX at all (`HasFX`), the global appearance keys' FX
    // atoms join the table behind the overrides.
    if fx_table.is_some() || !overrides.is_empty() {
        for atom in crate::ac2_drawn_groups::global_fx_atoms(store, visual)? {
            for id in atom.ids.iter() {
                if !overrides.iter().any(|(known, ..)| known == id) {
                    overrides.push((*id, Arc::clone(&atom.particles), Arc::clone(&atom.terrains)));
                }
            }
        }
    }
    let mut segments = HashMap::<u32, FxSegments>::new();
    if let Some(table) = fx_table {
        collect_table(store, table, 0, &mut segments)?;
    }
    for (id, list, terrains) in overrides {
        let mut one = FxSegments::default();
        one.push(list, Arc::from([]), terrains);
        segments.insert(id, one);
    }
    if segments.is_empty() {
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
    let rest_bone = |index: usize| rest.get(index).copied().unwrap_or(Ac2Transform::IDENTITY);
    let attach_list =
        |list: Vec<Ac2FxParticleNode>| -> ResourceResult<Arc<[Ac2ParticleAttachment]>> {
            let mut attachments = Vec::with_capacity(list.len());
            for node in list {
                // A node whose bone the visual lacks starts no system but keeps its light.
                let particle_descs = if attaches(&named, &node) {
                    node.particle_descs.clone()
                } else {
                    Arc::from([])
                };
                let looping = looping_flags(store, &particle_descs)?;
                let first_looping = looping.first().copied().unwrap_or(false);
                attachments.push(Ac2ParticleAttachment {
                    placement: placement(setup.as_deref(), &rest, &named, &node),
                    anchor: anchor(setup.as_deref(), &node),
                    remove_rotation: node.remove_rotation,
                    particle_descs,
                    water_plane: node.water_plane,
                    fire_and_forget: node.fire_and_forget,
                    light: node.light,
                    looping,
                    object: Ac2FxObject {
                        attach: object_attach(setup.as_deref(), &rest_bone, &named, &node),
                        particle_looping: first_looping,
                        ..node.object
                    },
                });
            }
            Ok(Arc::from(attachments))
        };
    let mut entries = HashMap::new();
    let mut terrains = HashMap::new();
    for (id, segs) in segments {
        let list = segs.particles();
        if !list.is_empty() {
            entries.insert(id, attach_list(list)?);
        }
        for set in segs.terrain_sets() {
            if !set.particles.is_empty() {
                terrains.insert((id, set.terrain), attach_list(set.particles.to_vec())?);
            }
        }
    }
    Ok(Some(Ac2VisualFxParticles {
        entries,
        terrains,
        startup,
    }))
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
    for (id, list) in record.entries.iter() {
        let terrains = record
            .terrains
            .iter()
            .find(|(t, _)| t == id)
            .map_or_else(|| Arc::from([]), |(_, sets)| Arc::clone(sets));
        segments
            .entry(*id)
            .or_default()
            .push(Arc::clone(list), Arc::from([]), terrains);
    }
    for sub in record.sub_tables.iter() {
        collect_table(store, *sub, depth + 1, segments)?;
    }
    Ok(())
}

/// `FXManager::DoCreateObjectFX`: where a created object goes on its emitter. A holding point
/// (`PHYSICSOBJREP::GetHoldingLocation`) or a bone (`GetBoneTypeMatrix`) is the attach point, its
/// node offset composed in (`CPhysicsObj::FXSetParent`); the rotation is dropped when
/// `m_remove_rotation` says so. The point's transform is kept as a local frame so the object can
/// be placed on the emitter's posed bone each frame; `rest` is the same frame at the rest pose.
/// A point or bone the emitter lacks leaves the object unplaced; a node with neither stands at
/// the emitter, and a water-plane node does too (its branch never moves the object off that point).
fn object_attach(
    setup: Option<&Ac2Setup>,
    rest_bone: &dyn Fn(usize) -> Ac2Transform,
    named: &[(u32, Ac2Transform)],
    node: &Ac2FxParticleNode,
) -> Ac2FxObjectAttach {
    let offset = Ac2Transform {
        offset: node.bone_offset,
        ..Ac2Transform::IDENTITY
    };
    let (point, local, mut rest) = if node.holding_location != 0 {
        let point = setup.and_then(|setup| {
            setup
                .holding_points
                .iter()
                .find(|point| point.id == node.holding_location)
        });
        let Some(point) = point else {
            return Ac2FxObjectAttach::Unplaced;
        };
        let local = point.transform.compose(&offset);
        let rest = rest_bone(point.bone).compose(&local);
        (Ac2FxAttachPoint::Holding(point.bone), local, rest)
    } else if node.bone_name != 0 {
        let Some((_, bone)) = named.iter().find(|(name, _)| *name == node.bone_name) else {
            return Ac2FxObjectAttach::Unplaced;
        };
        (
            Ac2FxAttachPoint::Bone(node.bone_name),
            offset,
            bone.compose(&offset),
        )
    } else {
        return Ac2FxObjectAttach::Emitter;
    };
    if node.remove_rotation {
        rest.rotation = Quat::IDENTITY;
    }
    Ac2FxObjectAttach::Attached(Ac2FxAttach {
        point,
        local,
        remove_rotation: node.remove_rotation,
        rest,
    })
}
