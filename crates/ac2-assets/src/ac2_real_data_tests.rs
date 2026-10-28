//! Real-data checks against an Asheron's Call 2 installation. They run only when
//! `AC2_DAT_DIR` points at an AC2 DAT directory, so ordinary test runs stay hermetic.

use std::sync::Arc;

use ac2_core::DataId;
use ac2_dat::{DatDataset, DatFileKind};
use ac2_db::{ResourcePackedDecode, ResourceStore, TypedResourceRef};

use crate::ac2_material::{Ac2MaterialInstance, Ac2MaterialModifier, Ac2RenderMaterial};
use crate::{load_ac2_material, load_ac2_texture_asset, resolve_ac2_material_texture_chain};

fn ac2_dataset() -> Option<(Arc<DatDataset>, ResourceStore)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
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

#[test]
fn every_appearance_table_decodes_all_its_atom_kinds() {
    let Some((dataset, store)) = ac2_dataset() else {
        return;
    };
    let tables = dataset
        .ids_in_range(DatFileKind::Portal, 0x2000_0000, 0x20FF_FFFF)
        .collect::<Vec<_>>();
    assert!(!tables.is_empty());
    let mut kinds = [0_usize; 6];
    for did in &tables {
        let table =
            store
                .load_cached_packed::<crate::ac2_appearance::Ac2AppearanceTable>(
                    TypedResourceRef::new(DatFileKind::Portal, DataId::new(*did), 0),
                )
                .unwrap_or_else(|error| panic!("AppearanceTable {did:08X}: {error}"))
                .expect("listed record exists");
        for (_, atoms) in table.atoms.values().flatten() {
            kinds[0] += atoms.materials.len();
            kinds[1] += atoms.opacities.len();
            kinds[2] += atoms.meshes.len();
            kinds[3] += atoms.part_groups.len();
            kinds[4] += atoms.fx.len();
            kinds[5] += atoms.icons.len();
        }
    }
    eprintln!(
        "{} tables: {} material, {} opacity, {} mesh, {} part-group, {} FX, {} icon atoms",
        tables.len(),
        kinds[0],
        kinds[1],
        kinds[2],
        kinds[3],
        kinds[4],
        kinds[5]
    );
    assert!(
        kinds[0] > 0 && kinds[2] > 0,
        "characters swap meshes and materials"
    );
}

/// `AppearanceTableAdaptor::HasAppearanceKey`, `InqTsysAppearanceKeyModCount` and
/// `InqAppearanceKeyModList` over every shipped table.
#[test]
fn appearance_key_queries_follow_the_table_data() {
    use crate::ac2_appearance::Ac2AppearanceTable;
    let Some((dataset, store)) = ac2_dataset() else {
        return;
    };
    let mut lists = 0;
    for did in dataset.ids_in_range(DatFileKind::Portal, 0x2000_0000, 0x20FF_FFFF) {
        let did = DataId::new(did);
        let table = store
            .load_cached_packed::<Ac2AppearanceTable>(TypedResourceRef::new(
                DatFileKind::Portal,
                did,
                0,
            ))
            .unwrap()
            .unwrap();
        for (key, collections) in &table.atoms {
            assert_eq!(
                crate::ac2_has_appearance_key(&store, did, *key).unwrap(),
                Some(true)
            );
            let list = crate::ac2_appearance_key_mod_list(&store, did, *key)
                .unwrap()
                .unwrap();
            assert!(
                list.windows(2).all(|pair| pair[0] <= pair[1]),
                "{did:?} {key:#x}"
            );
            // The table's own collections are all in the list (base tables may add more).
            assert!(list.len() >= collections.len());
            lists += 1;
        }
        for (key, count) in &table.threshold_counts {
            assert_eq!(
                crate::ac2_appearance_key_mod_count(&store, did, *key).unwrap(),
                Some(Some(*count))
            );
        }
        assert_eq!(
            crate::ac2_appearance_key_mod_count(&store, did, 0xFFFF_FFFE).unwrap(),
            Some(None)
        );
    }
    assert!(lists > 0);
    // A table that is not there answers the adaptor's miss.
    let missing = DataId::new(0x20FF_FF00);
    assert_eq!(
        crate::ac2_has_appearance_key(&store, missing, 1).unwrap(),
        None
    );
    assert_eq!(
        crate::ac2_appearance_key_mod_list(&store, missing, 1).unwrap(),
        None
    );
}

/// `VisualDesc::GetTotalIconDesc`: the DAT visuals' own icon layers are kept (the pack bit
/// `0x80`), merged along the parent chain in layer-id order, and the icon atoms of the
/// visual's global appearance keys are applied on top.
#[test]
fn total_icon_desc_merges_the_chain_and_applies_icon_atoms() {
    let Some((dataset, store)) = ac2_dataset() else {
        return;
    };
    let (mut with_layers, mut with_atoms) = (0, 0);
    for did in dataset.ids_in_range(DatFileKind::Portal, 0x1F00_0000, 0x1FFF_FFFF) {
        let visual = DataId::new(did);
        let Some(icon) = crate::ac2_total_icon_desc(&store, visual, None).unwrap() else {
            continue;
        };
        with_layers += 1;
        assert!(
            icon.layers
                .windows(2)
                .all(|p| p[0].layer_id <= p[1].layer_id),
            "{did:08X}: layers stay in layer-id order"
        );
        let desc = store
            .load_cached_packed::<crate::Ac2VisualDesc>(TypedResourceRef::new(
                DatFileKind::Portal,
                visual,
                0,
            ))
            .unwrap()
            .unwrap();
        // Every layer the visual itself streams is in the merged description.
        for own in &desc.icon.layers {
            assert!(
                icon.layers.iter().any(|l| l.layer_id == own.layer_id),
                "{did:08X}"
            );
        }
        let mut atoms = crate::Ac2Atoms::default();
        for (table, keys) in desc.global_appearance.iter() {
            for (key, scalar) in keys.iter() {
                if *scalar != -1.0 {
                    crate::ac2_appearance_atoms(&store, *table, *key, *scalar, false, &mut atoms)
                        .unwrap();
                }
            }
        }
        for atom in &atoms.icons {
            with_atoms += 1;
            let layer = icon
                .layers
                .iter()
                .find(|l| l.layer_id == atom.layer_id)
                .unwrap_or_else(|| panic!("{did:08X}: icon atom layer {}", atom.layer_id));
            if atom.image != 0 {
                assert_eq!(layer.image, atom.image, "{did:08X}");
            }
            if atom.shift_color != 0 {
                assert_eq!(layer.shift_color, atom.shift_color, "{did:08X}");
            }
        }
    }
    eprintln!("{with_layers} visuals with icon layers, {with_atoms} icon atoms applied");
    assert!(with_layers > 0);
}
