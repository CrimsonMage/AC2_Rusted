//! The particle systems an AC2 visual starts on its own: `PHYSICSOBJREP::InitFXManager` fills the
//! FX table from the total `VisualDesc` (`VisualDesc::GetTotalVisualDescInternal` applies the parent
//! chain root first; `PartGroupDataDesc::ApplyAdditivePGDDesc` keeps the first startup scalar and
//! FX override per id and the last non-zero `DBFXTable`), then `ApplyStartupFX` fires every startup
//! FX id. `FXTable::GetFXData` uses the override list when the id has one and otherwise the static
//! table, whose sub-tables are searched after it (`DBFXTable::GetFXData`). Each particle node
//! becomes a system placed by `PHYSICSOBJREP::CreateParticleSystem`.

use std::sync::Arc;

use ac2_core::DataId;
use ac2_dat::DatFileKind;
use ac2_db::{
    CachedResource, ResourcePackedDecode, ResourceResult, ResourceStore, TypedResourceRef,
};
use ac2_math::{Quat, Vec3};

use crate::ac2_drawn_groups::rest_named_bones;
use crate::ac2_fx::{Ac2FxLight, Ac2FxObject, Ac2FxParticleNode, Ac2FxTable};
use crate::ac2_visual_desc::Ac2VisualDesc;
use ac2_anim::{Ac2Setup, Ac2Transform};

const VISUAL_DESC_PREFIX: u32 = 0x1F00_0000;
const MAX_VISUAL_DESC_DEPTH: usize = 16;
const MAX_FX_TABLE_DEPTH: usize = 8;

/// Where a particle system attaches on its object, named against the visual's root Setup. The
/// bone and holding transforms are kept unposed: the renderer composes them with the object's
/// posed bones each frame (`CPartArray::QueueParticleSystems`, `FXManager::FireParticleFX`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ac2ParticleAnchor {
    /// Neither a holding point nor a bone: `m_bone_offset` in the object's space.
    Object { offset: Vec3 },
    /// `m_bone_name`: a bone of the object's part groups, found by name the way
    /// `CPartGroup::FindBoneType` searches them (the render side poses it). A name no group has
    /// is not an attachment. `m_bone_offset` is in that bone's space.
    Bone { name: u32, offset: Vec3 },
    /// `m_holding_location`: the Setup's holding point, its bone and location transform. `None`
    /// when the Setup has no such point.
    Holding {
        location: Option<(usize, Ac2Transform)>,
    },
}

/// One startup particle system of a visual.
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2ParticleAttachment {
    /// `FXNode::m_psdesc_list`; retail picks one entry at random per firing.
    pub particle_descs: Arc<[DataId]>,
    /// Object-local placement (AC axes) at the rest pose: holding point or bone rest pose, then
    /// the bone offset. Static world sources (`ac2_particle_sources`) use it.
    pub placement: Ac2Transform,
    /// The node's attachment, to be posed per frame (see [`Ac2ParticleAnchor`]).
    pub anchor: Ac2ParticleAnchor,
    /// `m_remove_rotation`: the attachment keeps only its translation.
    pub remove_rotation: bool,
    /// The node rides its cell's water plane (`FXManager::FireParticleFX`).
    pub water_plane: bool,
    /// `m_fire_and_forget` (see [`Ac2FxParticleNode::fire_and_forget`]).
    pub fire_and_forget: bool,
    /// The node's light, if it places one.
    pub light: Option<Ac2FxLight>,
    /// `FXManager::ParticleIsLooping` of each `particle_descs` entry (the same order).
    pub looping: Arc<[bool]>,
    /// The node's object facts (`FXNode`'s vdesc, hide, colour; see [`Ac2FxObject`]).
    pub object: Ac2FxObject,
}

/// The startup particle systems of a visual and its `m_particle_scale`.
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2VisualParticles {
    pub attachments: Arc<[Ac2ParticleAttachment]>,
    pub particle_scale: f32,
}

/// Resolves the startup particle systems of an AC2 `VisualDesc`. Returns `None` when the visual
/// starts none.
pub fn resolve_ac2_visual_particles(
    store: &ResourceStore,
    visual: DataId,
) -> ResourceResult<Option<Ac2VisualParticles>> {
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
    let mut fx_table = None;
    let mut particle_scale = None;
    let mut startup = Vec::<u32>::new();
    let mut overrides = Vec::<(u32, Arc<[Ac2FxParticleNode]>)>::new();
    for desc in chain.iter().rev() {
        fx_table = desc.fx.fx_table.or(fx_table);
        particle_scale = desc.particle_scale.or(particle_scale);
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
    // `PartGroupDataDesc::FillFXTable`: with FX at all (`HasFX`), the global appearance keys' FX
    // atoms join the table behind the overrides.
    if fx_table.is_some() || !overrides.is_empty() {
        for atom in crate::ac2_drawn_groups::global_fx_atoms(store, visual)? {
            for id in atom.ids.iter() {
                if !overrides.iter().any(|(known, _)| known == id) {
                    overrides.push((*id, Arc::clone(&atom.particles)));
                }
            }
        }
    }
    let mut nodes = Vec::new();
    for fx_id in startup {
        match overrides.iter().find(|(id, _)| *id == fx_id) {
            Some((_, list)) => nodes.extend(list.iter().cloned()),
            None => {
                if let Some(table) = fx_table {
                    collect_table_nodes(store, table, fx_id, 0, &mut nodes)?;
                }
            }
        }
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
    // Startup systems are the particle nodes; a light-only node starts none.
    let mut attachments = Vec::new();
    for node in nodes
        .into_iter()
        .filter(|node| !node.particle_descs.is_empty())
        .filter(|node| attaches(&named, node))
    {
        let looping = looping_flags(store, &node.particle_descs)?;
        attachments.push(Ac2ParticleAttachment {
            placement: placement(setup.as_deref(), &rest, &named, &node),
            anchor: anchor(setup.as_deref(), &node),
            remove_rotation: node.remove_rotation,
            particle_descs: node.particle_descs,
            water_plane: node.water_plane,
            fire_and_forget: node.fire_and_forget,
            light: node.light,
            looping,
            object: node.object,
        });
    }
    let attachments: Arc<[_]> = attachments.into();
    Ok(Some(Ac2VisualParticles {
        attachments,
        particle_scale: particle_scale.unwrap_or(1.0),
    }))
}

/// `DBFXTable::GetFXData`: this table's entry, then every sub-table.
fn collect_table_nodes(
    store: &ResourceStore,
    table: DataId,
    fx_id: u32,
    depth: usize,
    nodes: &mut Vec<Ac2FxParticleNode>,
) -> ResourceResult<()> {
    if depth >= MAX_FX_TABLE_DEPTH {
        return Ok(());
    }
    let Some(record) = load::<Ac2FxTable>(store, table)? else {
        return Ok(());
    };
    if let Some(list) = record.entry(fx_id) {
        nodes.extend(list.iter().cloned());
    }
    for sub in record.sub_tables.iter() {
        collect_table_nodes(store, *sub, fx_id, depth + 1, nodes)?;
    }
    Ok(())
}

/// Whether a node starts a system at all. `PHYSICSOBJREP::CreateParticleSystem` finds its bone
/// first (`CPartGroup::FindBoneType`) and returns zero when no part group has it, so a
/// bone-anchored node whose bone the visual lacks starts nothing. A fire-and-forget node is not
/// filtered: `FXManager::FireParticleFX` places it where its object stands.
pub(crate) fn attaches(named: &[(u32, Ac2Transform)], node: &Ac2FxParticleNode) -> bool {
    if node.fire_and_forget || node.holding_location != 0 || node.bone_name == 0 {
        return true;
    }
    named.iter().any(|(name, _)| *name == node.bone_name)
}

/// `PHYSICSOBJREP::CreateParticleSystem` and `CPartArray::QueueParticleSystems`: a holding point
/// places the system at its location transform on its bone (the bone offset is unused there); a
/// bone name offsets the system in that bone's rest pose, found through the part groups by name
/// (`named`, see [`rest_named_bones`]); otherwise the offset is object-local. `m_remove_rotation`
/// keeps only the composed translation.
pub(crate) fn placement(
    setup: Option<&Ac2Setup>,
    rest: &[Ac2Transform],
    named: &[(u32, Ac2Transform)],
    node: &Ac2FxParticleNode,
) -> Ac2Transform {
    let offset = Ac2Transform {
        offset: node.bone_offset,
        ..Ac2Transform::IDENTITY
    };
    let bone = |index: usize| rest.get(index).copied().unwrap_or(Ac2Transform::IDENTITY);
    let mut placed = match setup {
        Some(setup) if node.holding_location != 0 => setup
            .holding_points
            .iter()
            .find(|point| point.id == node.holding_location)
            .map_or(Ac2Transform::IDENTITY, |point| {
                bone(point.bone).compose(&point.transform)
            }),
        Some(_) if node.bone_name != 0 => named
            .iter()
            .find(|(name, _)| *name == node.bone_name)
            .map_or(offset, |(_, bone)| bone.compose(&offset)),
        _ => offset,
    };
    if node.remove_rotation {
        placed.rotation = Quat::IDENTITY;
        placed.scale = ac2_math::Vec3::ONE;
    }
    placed
}

/// The unposed attachment of a node (see [`Ac2ParticleAnchor`]), by the same precedence as
/// [`placement`]: a holding point, then a bone, then the object's offset.
pub(crate) fn anchor(setup: Option<&Ac2Setup>, node: &Ac2FxParticleNode) -> Ac2ParticleAnchor {
    if node.holding_location != 0 {
        let location = setup.and_then(|setup| {
            setup
                .holding_points
                .iter()
                .find(|point| point.id == node.holding_location)
                .map(|point| (point.bone, point.transform))
        });
        return Ac2ParticleAnchor::Holding { location };
    }
    match setup {
        Some(_) if node.bone_name != 0 => Ac2ParticleAnchor::Bone {
            name: node.bone_name,
            offset: node.bone_offset,
        },
        _ => Ac2ParticleAnchor::Object {
            offset: node.bone_offset,
        },
    }
}

/// `FXManager::ParticleIsLooping` for each of a node's systems, read on the loader thread (the
/// frame thread never reads a `PSDesc`). A missing system is not looping.
pub(crate) fn looping_flags(
    store: &ResourceStore,
    descs: &[DataId],
) -> ResourceResult<Arc<[bool]>> {
    let mut flags = Vec::with_capacity(descs.len());
    for &did in descs {
        let looping =
            load::<crate::Ac2ParticleSystemDesc>(store, did)?.is_some_and(|desc| desc.is_looping());
        flags.push(looping);
    }
    Ok(flags.into())
}

pub(crate) fn load<T: ResourcePackedDecode + Send + Sync + 'static>(
    store: &ResourceStore,
    did: DataId,
) -> ResourceResult<Option<CachedResource<T>>> {
    store.load_cached_packed::<T>(TypedResourceRef::new(DatFileKind::Portal, did, 0))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use ac2_core::DataId;
    use ac2_math::Vec3;

    use super::*;
    use crate::ac2_fx::Ac2FxObject;

    /// The named bones of a visual whose part groups have bones called `names`.
    fn named(names: &[u32]) -> Vec<(u32, Ac2Transform)> {
        names
            .iter()
            .map(|name| (*name, Ac2Transform::IDENTITY))
            .collect()
    }

    fn node(bone_name: u32) -> Ac2FxParticleNode {
        Ac2FxParticleNode {
            particle_descs: Arc::from([DataId::new(9)]),
            fire_and_forget: false,
            light: None,
            holding_location: 0,
            bone_name,
            bone_offset: Vec3::ZERO,
            remove_rotation: false,
            water_plane: false,
            object: Ac2FxObject::default(),
        }
    }

    /// `CPartGroup::FindBoneType` finds no bone, so `CreateParticleSystem` starts nothing; a
    /// node whose bone some group has, or that names no bone, still starts its system.
    #[test]
    fn a_node_whose_bone_no_part_group_has_starts_no_system() {
        let bones = named(&[5]);
        assert!(!attaches(&bones, &node(7)));
        assert!(attaches(&bones, &node(5)));
        assert!(attaches(&bones, &node(0)));
        assert!(!attaches(&[], &node(7)), "no part group, nothing to search");
    }

    /// `FXManager::FireParticleFX` places a fire-and-forget system without its bone, and a
    /// holding-point node never reads the bone name.
    #[test]
    fn fire_and_forget_and_holding_nodes_are_not_filtered_by_their_bone() {
        let bones = named(&[5]);
        let mut fired = node(7);
        fired.fire_and_forget = true;
        assert!(attaches(&bones, &fired));
        let mut holding = node(7);
        holding.holding_location = 3;
        assert!(attaches(&bones, &holding));
    }

    /// `CPartArray::QueueParticleSystems` takes the bone's matrix times the node's offset, so the
    /// placement is the found bone's rest transform with the offset composed in it.
    #[test]
    fn placement_composes_the_offset_in_the_found_bone() {
        let setup = Ac2Setup {
            meshes: Arc::from([]),
            bones: Arc::from([]),
            placements: Arc::from([]),
            holding_points: Arc::from([]),
            connection_points: Arc::from([]),
            degrade: Default::default(),
            bounds_min: Vec3::ZERO,
            bounds_max: Vec3::ZERO,
        };
        let mut bone = named(&[5]);
        bone[0].1.offset = Vec3::new(0.0, 0.0, 2.0);
        let mut n = node(5);
        n.bone_offset = Vec3::new(1.0, 0.0, 0.0);
        let placed = placement(Some(&setup), &[], &bone, &n);
        assert_eq!(placed.offset, Vec3::new(1.0, 0.0, 2.0));
    }
}
