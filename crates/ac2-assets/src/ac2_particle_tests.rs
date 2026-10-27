//! Real-data checks of the AC2 particle records. They run only when `AC2_DAT_DIR` points
//! at an AC2 DAT directory, so ordinary test runs stay hermetic.

use std::collections::BTreeSet;
use std::sync::Arc;

use ac2_core::DataId;
use ac2_dat::{DatDataset, DatFileKind};
use ac2_db::{CachedResource, ResourcePackedDecode, ResourceStore, TypedResourceRef};

use crate::ac2_fx::Ac2FxTable;
use crate::{resolve_ac2_visual_particles, Ac2ParticleSystemDesc};

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
