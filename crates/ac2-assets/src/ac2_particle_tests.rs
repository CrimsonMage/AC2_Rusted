//! Real-data checks of the AC2 particle records. They run only when `AC2_DAT_DIR` points
//! at an AC2 DAT directory, so ordinary test runs stay hermetic.

use std::collections::BTreeSet;
use std::sync::Arc;

use ac2_dat::{DatDataset, DatFileKind, DatOpenOptions};
use ac2_core::DataId;
use ac2_db::{
    CachedResource, ResourcePackedDecode, ResourceStore, TypedResourceRef,
};

use crate::ac2_fx::Ac2FxTable;
use crate::{resolve_ac2_visual_particles, Ac2ParticleSystemDesc};

fn ac2_dataset() -> Option<(Arc<DatDataset>, ResourceStore)> {
    let root = std::env::var_os("AC2_DAT_DIR")?;
    let dataset = DatDataset::open(DatOpenOptions::new(root)).expect("AC2 DATs mount");
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
    for did in &ids {
        let table = load::<Ac2FxTable>(&store, *did);
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
