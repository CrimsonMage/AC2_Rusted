//! Real-data checks of the AC2 particle records. They run only when `AC2_DAT_DIR` points
//! at an AC2 DAT directory, so ordinary test runs stay hermetic.

use std::collections::BTreeSet;
use std::sync::Arc;

use ac2_core::DataId;
use ac2_dat::{DatDataset, DatFileKind};
use ac2_db::{CachedResource, ResourcePackedDecode, ResourceStore, TypedResourceRef};

use crate::ac2_fx::Ac2FxTable;
use crate::{
    resolve_ac2_entity_visual, resolve_ac2_visual_fx_particles, resolve_ac2_visual_meshes,
    resolve_ac2_visual_particles, Ac2ParticleAnchor, Ac2ParticleSystemDesc,
};

fn ac2_dataset() -> Option<(Arc<DatDataset>, ResourceStore)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = ResourceStore::new(Arc::clone(&dataset));
    assert!(ac2_db::install_ac2_property_types(&store).expect("MasterProperty decodes"));
    Some((dataset, store))
}

fn portal_ids(dataset: &DatDataset, prefix: u32) -> Vec<u32> {
    dataset
        .ids_in_range(DatFileKind::Portal, prefix, prefix | 0x00FF_FFFF)
        .collect()
}

fn load<T: ResourcePackedDecode + Send + Sync + 'static>(
    store: &ResourceStore,
    did: u32,
) -> CachedResource<T> {
    store
        .load_cached_packed::<T>(TypedResourceRef::new(
            DatFileKind::Portal,
            DataId::new(did),
            0,
        ))
        .unwrap_or_else(|error| panic!("{} {did:08X}: {error}", T::TYPE_NAME))
        .expect("listed record exists")
}

#[test]
fn every_legions_particle_system_decodes_to_its_exact_length() {
    let Some((dataset, store)) = ac2_dataset() else {
        return;
    };
    let ids = portal_ids(&dataset, 0x3900_0000);
    assert_eq!(ids.len(), 1729);
    let (mut emitters, mut keyframes, mut world_space) = (0, 0, 0);
    for did in &ids {
        let desc = load::<Ac2ParticleSystemDesc>(&store, *did);
        assert!(
            desc.material.is_some_and(|m| m.raw() >> 24 == 0x31),
            "{did:08X} material"
        );
        assert!(
            desc.physics_time_step > 0.0 && desc.stop_fade_distance > 0.0,
            "{did:08X}"
        );
        emitters += desc.emitters.len();
        keyframes += desc
            .emitters
            .iter()
            .map(|e| e.keyframes.len())
            .sum::<usize>();
        world_space += usize::from(desc.world_space);
        for emitter in desc.emitters.iter() {
            assert!(
                matches!(emitter.particle_shape, 3 | 4),
                "{did:08X} particle shape"
            );
            assert!(emitter.shape <= 4, "{did:08X} emitter shape");
        }
    }
    assert_eq!((emitters, keyframes, world_space), (3067, 11799, 1029));
}

#[test]
fn every_legions_fx_table_decodes_to_its_exact_length() {
    let Some((dataset, store)) = ac2_dataset() else {
        return;
    };
    let ids = portal_ids(&dataset, 0x1E00_0000);
    assert_eq!(ids.len(), 312);
    let mut particle_nodes = 0;
    let mut sound_refs = 0;
    for did in &ids {
        let table = load::<Ac2FxTable>(&store, *did);
        for (_, nodes) in table.sounds.iter() {
            for node in nodes.iter() {
                sound_refs += node.sounds.len();
                assert!(
                    node.sounds.iter().all(|s| matches!(s >> 24, 0x2A | 0x2C)),
                    "{did:08X} sound list {:08X?}",
                    node.sounds
                );
                assert!(node.ramp_time >= 0.0 && node.ramp_time.is_finite());
            }
        }
        particle_nodes += table
            .entries
            .iter()
            .map(|(_, nodes)| nodes.len())
            .sum::<usize>();
        for sub in table.sub_tables.iter() {
            assert_eq!(sub.raw() >> 24, 0x1E, "{did:08X} sub-table");
        }
    }
    assert!(particle_nodes > 500, "{particle_nodes} particle FX nodes");
    assert!(sound_refs > 10_000, "{sound_refs} FX sound references");
}

/// The object facts of the FX nodes decode with the real `DBFXTable` layout: object-creating
/// nodes exist, each with a visual, a finite lifetime and colour, and hide or colour nodes are
/// found too. A misaligned read would scatter these out of range.
#[test]
fn fx_nodes_decode_their_object_facts() {
    let Some((dataset, store)) = ac2_dataset() else {
        return;
    };
    let ids = portal_ids(&dataset, 0x1E00_0000);
    let (mut vdesc, mut hides, mut colours, mut sample) = (0, 0, 0, None);
    for did in &ids {
        let table = load::<Ac2FxTable>(&store, *did);
        for (fx, nodes) in table.entries.iter() {
            for node in nodes.iter() {
                let object = node.object;
                assert!(object.lifetime.is_finite());
                assert!(object.ramp_time.is_finite() && object.ramp_time >= 0.0);
                if object.vdesc != 0 {
                    vdesc += 1;
                    sample.get_or_insert((*did, *fx, object.vdesc));
                }
                hides += usize::from(object.hide_target);
                colours += usize::from(object.color.is_some());
            }
        }
    }
    assert!(vdesc > 0, "object-creating FX nodes");
    assert!(colours > 0, "colour FX nodes");
    eprintln!("object facts: vdesc {vdesc}, hide {hides}, colour {colours}, sample {sample:08X?}");
}

#[test]
fn the_human_visual_resolves_its_footstep_sounds() {
    let Some((_, store)) = ac2_dataset() else {
        return;
    };
    // The human male (`0x1F000023`, table `0x1E000013`): walking raises FX 0x40000002 and
    // 0x40000003, each a node picking one of its step sounds.
    let sounds = crate::resolve_ac2_visual_fx_sounds(&store, DataId::new(0x1F00_0023))
        .unwrap()
        .expect("the human visual has FX");
    assert!(sounds.has_sounds());
    for fx in [0x4000_0002, 0x4000_0003] {
        let nodes = sounds
            .get(fx)
            .expect("the step FX resolves through the table");
        let step = &nodes[0];
        assert!(step.sounds.len() > 1, "{fx:08X} picks among several sounds");
        assert!(step.sounds.iter().all(|did| did >> 24 == 0x2A));
        assert!(!step.fire_and_forget && step.day_time && step.night_time);
    }
    // Its startup FX's sound is the player's only.
    assert!(sounds.get(0x8100_01C0).unwrap()[0].player_only);
}

#[test]
fn startup_particles_of_every_visual_name_decodable_particle_systems() {
    let Some((dataset, store)) = ac2_dataset() else {
        return;
    };
    let mut visuals_with_particles = 0;
    let mut referenced = BTreeSet::new();
    for did in portal_ids(&dataset, 0x1F00_0000) {
        let resolved = resolve_ac2_visual_particles(&store, DataId::new(did))
            .unwrap_or_else(|error| panic!("VisualDesc {did:08X}: {error}"));
        let Some(particles) = resolved else {
            continue;
        };
        visuals_with_particles += 1;
        assert!(particles.particle_scale > 0.0);
        for attachment in particles.attachments.iter() {
            assert!(attachment.placement.offset.is_finite());
            referenced.extend(attachment.particle_descs.iter().map(|d| d.raw()));
        }
    }
    for did in &referenced {
        load::<Ac2ParticleSystemDesc>(&store, *did);
    }
    eprintln!(
        "{visuals_with_particles} visuals start particles from {} PSDescs",
        referenced.len()
    );
    assert!(
        visuals_with_particles > 500,
        "{visuals_with_particles} visuals start particles"
    );
}

#[test]
fn the_visuals_fx_particles_resolve_per_fx_id() {
    let Some((dataset, store)) = ac2_dataset() else {
        return;
    };
    // Every visual's FX particle table: each id's systems name particle descs (`0x39`).
    let mut visuals = 0;
    let mut ids = 0;
    for did in portal_ids(&dataset, 0x1F00_0000) {
        let Some(fx) = crate::resolve_ac2_visual_fx_particles(&store, DataId::new(did))
            .unwrap_or_else(|e| panic!("{did:08X} FX particles: {e:?}"))
        else {
            continue;
        };
        visuals += 1;
        for (fx_id, attachments) in fx.entries_for_test() {
            ids += 1;
            assert!(!attachments.is_empty(), "{did:08X} {fx_id:08X}");
            for attachment in attachments.iter() {
                assert!(
                    attachment
                        .particle_descs
                        .iter()
                        .all(|d| d.raw() >> 24 == 0x39),
                    "{did:08X} {fx_id:08X} particle descs"
                );
            }
        }
    }
    assert!(
        visuals > 100 && ids > 500,
        "{visuals} visuals, {ids} FX ids"
    );
}

/// An FX node's object is an `EntityDesc` (`0x47`): `EntityDesc::GetVisualDescID` follows its
/// template to a `VisualDesc` or `CSetup`, and that visual has meshes to draw. Every object DID
/// the FX tables name must resolve so.
#[test]
fn every_fx_objects_entity_resolves_to_a_drawable_visual() {
    let Some((dataset, store)) = ac2_dataset() else {
        return;
    };
    let mut objects = BTreeSet::new();
    for did in portal_ids(&dataset, 0x1E00_0000) {
        let table = load::<Ac2FxTable>(&store, did);
        for (_, nodes) in table.entries.iter() {
            objects.extend(
                nodes
                    .iter()
                    .map(|node| node.object.vdesc)
                    .filter(|vdesc| *vdesc != 0),
            );
        }
    }
    let entities: Vec<u32> = objects
        .iter()
        .copied()
        .filter(|vdesc| vdesc >> 24 == 0x47)
        .collect();
    assert!(!entities.is_empty(), "FX objects are EntityDescs");
    for entity in &entities {
        let visual = resolve_ac2_entity_visual(&store, DataId::new(*entity))
            .unwrap_or_else(|e| panic!("EntityDesc {entity:08X}: {e}"));
        assert!(
            matches!(visual.raw() >> 24, 0x1F | 0x04),
            "{entity:08X} resolves to {visual:?}, not a visual"
        );
        let meshes = resolve_ac2_visual_meshes(&store, visual)
            .unwrap_or_else(|e| panic!("{entity:08X} meshes: {e}"));
        assert!(!meshes.is_empty(), "{entity:08X} draws no mesh");
    }
    eprintln!(
        "{} FX object EntityDescs resolve to drawable visuals",
        entities.len()
    );
}

/// Every FX particle node of the human-scale visuals, by how it attaches: a posed bone, a holding
/// point (found, or missing), the object's offset, and the water-plane and removed-rotation flags.
/// Each kind the DAT uses must resolve in the real data.
#[test]
fn fx_particle_nodes_attach_to_bones_holding_points_and_water_in_the_dataset() {
    let Some((dataset, store)) = ac2_dataset() else {
        return;
    };
    let (mut bone, mut holding, mut missing_holding) = (0, 0, 0);
    let (mut object, mut water, mut removed, mut fire) = (0, 0, 0, 0);
    let mut unstarted = 0;
    for did in portal_ids(&dataset, 0x1F00_0000) {
        let Some(table) = resolve_ac2_visual_fx_particles(&store, DataId::new(did))
            .unwrap_or_else(|error| panic!("VisualDesc {did:08X}: {error}"))
        else {
            continue;
        };
        for (_, attachments) in table.entries_for_test() {
            for attachment in attachments.iter() {
                // A node with no system (a bone the visual lacks, or a light-only node) starts
                // nothing, so only the nodes that start a system count.
                if attachment.particle_descs.is_empty() {
                    unstarted += 1;
                    continue;
                }
                match attachment.anchor {
                    Ac2ParticleAnchor::Object { .. } => object += 1,
                    Ac2ParticleAnchor::Bone { .. } => bone += 1,
                    Ac2ParticleAnchor::Holding { location: Some(_) } => holding += 1,
                    Ac2ParticleAnchor::Holding { location: None } => missing_holding += 1,
                }
                water += usize::from(attachment.water_plane);
                removed += usize::from(attachment.remove_rotation);
                fire += usize::from(attachment.fire_and_forget);
            }
        }
    }
    eprintln!(
        "FX particle nodes: object {object}, bone {bone}, holding \
         {holding} (missing {missing_holding}), water {water}, removed rotation {removed}, \
         fire-and-forget {fire}, no system {unstarted}"
    );
    assert!(bone > 0, "some FX attach to a posed bone");
    assert!(holding > 0, "some FX attach to a holding point");
    assert!(object > 0);
}

/// Every FX node that creates an object (`FXNode::m_vdesc_id`), by where the object is placed on
/// its emitter (`FXManager::DoCreateObjectFX`): the emitter itself, a bone or holding point, or
/// nowhere when the emitter lacks the point.
#[test]
fn fx_created_objects_attach_to_their_emitters_points_in_the_dataset() {
    use crate::Ac2FxObjectAttach;
    let Some((dataset, store)) = ac2_dataset() else {
        return;
    };
    let (mut emitter, mut attached, mut unplaced) = (0, 0, 0);
    for did in portal_ids(&dataset, 0x1F00_0000) {
        let Some(table) = resolve_ac2_visual_fx_particles(&store, DataId::new(did))
            .unwrap_or_else(|error| panic!("VisualDesc {did:08X}: {error}"))
        else {
            continue;
        };
        for (_, attachments) in table.entries_for_test() {
            for attachment in attachments.iter().filter(|a| a.object.vdesc != 0) {
                match attachment.object.attach {
                    Ac2FxObjectAttach::Emitter => emitter += 1,
                    Ac2FxObjectAttach::Attached(_) => attached += 1,
                    Ac2FxObjectAttach::Unplaced => unplaced += 1,
                }
            }
        }
    }
    eprintln!(
        "FX-created objects: at the emitter {emitter}, attached {attached}, unplaced {unplaced}"
    );
    assert!(
        emitter + attached + unplaced > 0,
        "the dataset has FX objects"
    );
}

/// The streak FX nodes of the real FX tables decode to plausible `StreakFX` values: a colour in
/// `[0, 1]`, finite points, a material instance and a texture scale. A misaligned stream reads
/// garbage here, and the nodes after a streak would decode wrong too.
#[test]
fn streak_fx_nodes_decode_from_real_fx_tables() {
    let Some((dataset, store)) = ac2_dataset() else {
        return;
    };
    // Streak nodes sit in the FX tables (`0x1E`) and in each visual's own FX overrides (`0x1F`).
    let mut streaks = Vec::new();
    for did in portal_ids(&dataset, 0x1E00_0000) {
        let table = load::<Ac2FxTable>(&store, did);
        for (fx, nodes) in table.entries.iter() {
            for node in nodes.iter() {
                if let Some(streak) = node.object.streak {
                    streaks.push((did, *fx, streak, node.object.start_streak));
                }
            }
        }
    }
    for did in portal_ids(&dataset, 0x1F00_0000) {
        let visual = load::<crate::ac2_visual_desc::Ac2VisualDesc>(&store, did);
        for (fx, nodes) in visual.fx.overrides.iter() {
            for node in nodes.iter() {
                if let Some(streak) = node.object.streak {
                    streaks.push((did, *fx, streak, node.object.start_streak));
                }
            }
        }
    }
    println!("streak nodes: {}", streaks.len());
    assert!(
        !streaks.is_empty(),
        "the FX tables or visuals carry streak nodes"
    );
    for (did, fx, streak, _) in &streaks {
        assert!(
            streak
                .color
                .iter()
                .all(|c| c.is_finite() && (0.0..=1.0).contains(c)),
            "{did:08X} fx {fx:08X} colour {:?}",
            streak.color
        );
        for point in [streak.start, streak.end] {
            assert!(point.x.is_finite() && point.y.is_finite() && point.z.is_finite());
        }
        assert_ne!(streak.material, 0, "{did:08X} fx {fx:08X} has a material");
        assert!(
            streak.texture_scale.is_finite() && streak.texture_scale >= 0.0,
            "{did:08X} fx {fx:08X} texture scale {}",
            streak.texture_scale
        );
        assert!(streak.trail_duration.is_finite() && streak.trail_duration >= 0.0);
        assert!(streak.lifespan.is_finite() && streak.lifespan >= 0.0);
    }
}

/// The decal FX nodes of the real FX tables decode to plausible `DecalFX` values: a colour in
/// `[0, 1]`, a material, finite origin and lifespan, and waveforms with finite bases. The node's
/// stream is read to its end by the table decode, so a misaligned decal reads garbage here.
#[test]
fn decal_fx_nodes_decode_from_real_fx_tables() {
    let Some((dataset, store)) = ac2_dataset() else {
        return;
    };
    let mut decals = Vec::new();
    for did in portal_ids(&dataset, 0x1E00_0000) {
        let table = load::<Ac2FxTable>(&store, did);
        for (fx, nodes) in table.entries.iter() {
            for node in nodes.iter() {
                if let Some(decal) = node.object.decal {
                    decals.push((did, *fx, decal));
                }
            }
        }
    }
    for did in portal_ids(&dataset, 0x1F00_0000) {
        let visual = load::<crate::ac2_visual_desc::Ac2VisualDesc>(&store, did);
        for (fx, nodes) in visual.fx.overrides.iter() {
            for node in nodes.iter() {
                if let Some(decal) = node.object.decal {
                    decals.push((did, *fx, decal));
                }
            }
        }
    }
    println!("decal nodes: {}", decals.len());
    for (did, fx, decal) in &decals {
        assert!(
            decal
                .color
                .iter()
                .all(|c| c.is_finite() && (0.0..=1.0).contains(c)),
            "{did:08X} fx {fx:08X} colour {:?}",
            decal.color
        );
        assert_ne!(decal.material, 0, "{did:08X} fx {fx:08X} has a material");
        assert!(
            decal.origin.x.is_finite() && decal.origin.y.is_finite() && decal.origin.z.is_finite()
        );
        assert!(decal.lifespan.is_finite() && decal.lifespan >= 0.0);
        for waveform in [decal.size, decal.rotation, decal.position_radius] {
            assert!(waveform.base.is_finite() && waveform.amplitude.is_finite());
        }
    }
}
