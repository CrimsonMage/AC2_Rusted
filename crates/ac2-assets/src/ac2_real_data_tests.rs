//! Real-data checks against an Asheron's Call 2 installation. They run only when
//! `AC2_DAT_DIR` points at an AC2 DAT directory, so ordinary test runs stay hermetic.

use std::sync::Arc;

use ac2_core::DataId;
use ac2_dat::{DatDataset, DatFileKind, DatOpenOptions};
use ac2_db::{ResourcePackedDecode, ResourceStore, TypedResourceRef};

use crate::ac2_material::{Ac2MaterialInstance, Ac2MaterialModifier, Ac2RenderMaterial};
use crate::{load_ac2_material, load_ac2_texture_asset, resolve_ac2_material_texture_chain};

fn ac2_dataset() -> Option<(Arc<DatDataset>, ResourceStore)> {
    let root = std::env::var_os("AC2_DAT_DIR")?;
    let dataset = DatDataset::open(DatOpenOptions::new(root)).expect("AC2 DATs mount");
    let store = ResourceStore::new(Arc::clone(&dataset));
    assert!(ac2_db::install_ac2_property_types(&store).expect("MasterProperty decodes"));
    Some((dataset, store))
}

fn decode_all<T: ResourcePackedDecode + Send + Sync + 'static>(
    dataset: &DatDataset,
    store: &ResourceStore,
    prefix: u32,
) -> usize {
    let ids = dataset
        .ids_in_range(DatFileKind::Portal, prefix, prefix | 0x00FF_FFFF)
        .collect::<Vec<_>>();
    for did in &ids {
        store
            .load_cached_packed::<T>(TypedResourceRef::new(
                DatFileKind::Portal,
                DataId::new(*did),
                0,
            ))
            .unwrap_or_else(|error| panic!("{} {did:08X}: {error}", T::TYPE_NAME))
            .expect("listed record exists");
    }
    ids.len()
}

#[test]
fn every_legions_material_record_decodes_to_its_exact_length() {
    let Some((dataset, store)) = ac2_dataset() else {
        return;
    };
    assert_eq!(
        decode_all::<Ac2RenderMaterial>(&dataset, &store, 0x2B00_0000),
        130
    );
    assert_eq!(
        decode_all::<Ac2MaterialModifier>(&dataset, &store, 0x3000_0000),
        4035
    );
    assert_eq!(
        decode_all::<Ac2MaterialInstance>(&dataset, &store, 0x3100_0000),
        2401
    );
}

#[test]
fn material_instances_resolve_to_loadable_textures() {
    let Some((dataset, store)) = ac2_dataset() else {
        return;
    };
    let instances = dataset
        .ids_in_range(DatFileKind::Portal, 0x3100_0000, 0x31FF_FFFF)
        .collect::<Vec<_>>();
    let (mut textured, mut loaded) = (0, 0);
    for did in instances.iter().step_by(16) {
        let material = load_ac2_material(&store, DataId::new(*did))
            .expect("instance resolves")
            .expect("instance template exists");
        assert!(!material.layers.is_empty());
        if let Some(chain) = resolve_ac2_material_texture_chain(&store, DataId::new(*did)).unwrap()
        {
            textured += 1;
            loaded += usize::from(load_ac2_texture_asset(&store, chain).unwrap().is_some());
        }
    }
    assert!(
        textured > 100,
        "most sampled instances are textured ({textured})"
    );
    assert_eq!(loaded, textured, "every resolved texture chain loads");
}

#[test]
fn every_legions_texture_chain_loads() {
    let Some((dataset, store)) = ac2_dataset() else {
        return;
    };
    let chains = dataset
        .ids_in_range(DatFileKind::Portal, 0x4000_0000, 0x40FF_FFFF)
        .collect::<Vec<_>>();
    for did in &chains {
        load_ac2_texture_asset(&store, DataId::new(*did))
            .unwrap_or_else(|error| panic!("texture {did:08X}: {error}"))
            .unwrap_or_else(|| panic!("texture {did:08X} has no level"));
    }
    assert_eq!(chains.len(), 4395);
}

#[test]
fn every_legions_mesh_setup_and_visual_desc_decodes() {
    let Some((dataset, store)) = ac2_dataset() else {
        return;
    };
    assert_eq!(
        decode_all::<crate::Ac2Mesh>(&dataset, &store, 0x0600_0000),
        4501
    );
    assert_eq!(
        decode_all::<ac2_anim::Ac2Setup>(&dataset, &store, 0x0400_0000),
        2465
    );
    let descs = dataset
        .ids_in_range(DatFileKind::Portal, 0x1F00_0000, 0x1FFF_FFFF)
        .collect::<Vec<_>>();
    let failures =
        descs
            .iter()
            .filter_map(|did| {
                store
                    .load_cached_packed::<crate::ac2_visual_desc::Ac2VisualDesc>(
                        TypedResourceRef::new(DatFileKind::Portal, DataId::new(*did), 0),
                    )
                    .err()
                    .map(|error| format!("{did:08X}: {error}"))
            })
            .collect::<Vec<_>>();
    assert_eq!(descs.len(), 6504);
    assert!(
        failures.is_empty(),
        "{} VisualDescs failed: {:?}",
        failures.len(),
        &failures[..failures.len().min(5)]
    );
    // Five EntityDescs name properties absent from `MasterProperty`; all others decode exactly.
    let entities = dataset
        .ids_in_range(DatFileKind::Portal, 0x4700_0000, 0x47FF_FFFF)
        .collect::<Vec<_>>();
    let entity_failures = entities
        .iter()
        .filter(|did| {
            store
                .load_cached_packed::<crate::Ac2EntityDesc>(TypedResourceRef::new(
                    DatFileKind::Portal,
                    DataId::new(**did),
                    0,
                ))
                .is_err()
        })
        .count();
    assert_eq!(entities.len(), 15029);
    assert!(entity_failures <= 5, "{entity_failures} EntityDescs failed");
}

#[test]
fn visual_descs_compose_textured_static_models() {
    let Some((dataset, store)) = ac2_dataset() else {
        return;
    };
    let descs = dataset
        .ids_in_range(DatFileKind::Portal, 0x1F00_0000, 0x1FFF_FFFF)
        .collect::<Vec<_>>();
    let (mut resolved, mut drawn) = (0, 0);
    for did in descs.iter().step_by(20) {
        let did = DataId::new(*did);
        if !matches!(crate::resolve_ac2_visual_setup(&store, did), Ok(Some(_))) {
            continue;
        }
        resolved += 1;
        if let Some(model) = crate::decode_ac2_static_model(&store, did).unwrap() {
            drawn += 1;
            assert!(model.bounds.min.x <= model.bounds.max.x);
            for batch in model.surface_batches.iter() {
                assert!(batch
                    .indices
                    .iter()
                    .all(|i| (*i as usize) < batch.vertices.len()));
                assert!(!batch.texture.levels.is_empty());
            }
        }
    }
    assert!(
        resolved > 100,
        "{resolved} sampled VisualDescs resolve to a Setup"
    );
    assert!(
        drawn * 10 >= resolved * 8,
        "{drawn} of {resolved} compose a textured model"
    );
}

#[test]
fn human_stance_animation_poses_the_skinned_body() {
    let Some((_dataset, store)) = ac2_dataset() else {
        return;
    };
    let human = DataId::new(0x1F00_0023);
    let mesh = store
        .load_cached_packed::<crate::Ac2Mesh>(TypedResourceRef::new(
            DatFileKind::Portal,
            DataId::new(0x0600_003A),
            0,
        ))
        .unwrap()
        .unwrap();
    let skin = mesh.skin.as_ref().expect("torso is skinned");
    assert!(
        skin.iter().flatten().all(|(bone, _)| *bone < 41),
        "influences index Setup bones"
    );
    let bind = crate::decode_ac2_static_model(&store, human)
        .unwrap()
        .unwrap();
    let animation =
        ac2_anim::load_ac2_motion_animation(&store, DataId::new(0x0300_0014), 0x8000_0012)
            .unwrap()
            .expect("stance clip");
    let posed = crate::decode_ac2_posed_model(
        &store,
        human,
        Some(crate::Ac2Pose {
            animation: &animation,
            time: 0.0,
        }),
    )
    .unwrap()
    .unwrap();
    let width = |model: &crate::DecodedStaticModel| model.bounds.max.x - model.bounds.min.x;
    assert!(posed
        .surface_batches
        .iter()
        .all(|batch| batch.vertices.iter().all(|v| v.position.is_finite())));
    eprintln!(
        "bind span {:.2} m, posed span {:.2} m",
        width(&bind),
        width(&posed)
    );
    assert!(
        width(&posed) < width(&bind) * 0.9,
        "arms leave the bind T-pose"
    );
}
