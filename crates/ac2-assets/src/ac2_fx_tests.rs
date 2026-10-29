use std::sync::Arc;

use ac2_db::{ResourceStore, TypedResourceRef};

use super::*;

/// Every `DBFXTable` in portal.dat decodes, and the nodes that name a camera effect
/// (`m_cameraFX_id`) name `CameraFX` records (DIDs `0x55xxxxxx`).
#[test]
fn every_fx_table_decodes_and_its_camera_effects_are_camera_fx_records() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ResourceStore::new(Arc::clone(&dataset));
    assert!(ac2_db::install_ac2_property_types(&store).expect("MasterProperty decodes"));
    let ids: Vec<u32> = dataset
        .ids_in_range(ac2_dat::DatFileKind::Portal, 0x1E00_0000, 0x1EFF_FFFE)
        .collect();
    assert!(!ids.is_empty());
    let mut camera = Vec::new();
    for did in ids {
        let table = store
            .load_cached_packed::<Ac2FxTable>(TypedResourceRef::new(
                ac2_dat::DatFileKind::Portal,
                DataId::new(did),
                0,
            ))
            .unwrap_or_else(|e| panic!("{did:08X}: {e:?}"))
            .unwrap_or_else(|| panic!("{did:08X} missing"));
        for (fx, nodes) in table.entries.iter() {
            for node in nodes.iter().filter(|n| n.object.camera_fx != 0) {
                camera.push((did, *fx, node.object.camera_fx));
            }
        }
    }
    eprintln!("{} camera FX nodes: {camera:08X?}", camera.len());
    assert!(!camera.is_empty(), "some FX play a camera effect");
    for (table, fx, id) in camera {
        assert_eq!(id >> 24, 0x55, "{table:08X} fx {fx:08X}");
    }
}
