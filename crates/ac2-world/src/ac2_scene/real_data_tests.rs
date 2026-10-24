//! Real-data checks of the scenery generator. Run only when `AC2_DAT_DIR` points at an AC2 DAT
//! directory; they assert invariants, not stored outputs, so no game data lives in the repo.

use std::sync::Arc;

use ac2_core::CellId;
use ac2_dat::{DatDataset, DatOpenOptions};
use ac2_db::ResourceStore;

use super::{ac2_scene_object_visual, ac2_scene_placements};

fn ac2_store() -> Option<ResourceStore> {
    let root = std::env::var_os("AC2_DAT_DIR")?;
    let dataset = DatDataset::open(DatOpenOptions::new(root)).expect("AC2 DATs mount");
    Some(ResourceStore::new(Arc::clone(&dataset)))
}

#[test]
fn scenery_is_deterministic_inside_its_block_and_on_valid_scales() {
    let Some(store) = ac2_store() else {
        return;
    };
    for block in [0xCB4D_u32, 0xF18F, 0x7F7F] {
        let id = CellId::new(block << 16 | 1);
        let first = ac2_scene_placements(&store, id).expect("scenery generates");
        let again = ac2_scene_placements(&store, id).expect("scenery generates");
        assert_eq!(first, again, "{block:04X}: generation is deterministic");
        assert!(first.iter().any(|p| p.frill), "{block:04X}: grass frills");
        assert!(first.iter().any(|p| !p.frill), "{block:04X}: scenery objects");
        for p in &first {
            let [x, y, z] = p.origin.to_array();
            assert!((-0.01..160.01).contains(&x) && (-0.01..160.01).contains(&y), "{p:?}");
            assert!(z.is_finite() && p.scale.min_element() > 0.0, "{p:?}");
            assert!((p.rotation.length() - 1.0).abs() < 1e-3, "{p:?}");
        }
    }
}

#[test]
fn ac2_scene_objects_resolve_to_visuals() {
    let Some(store) = ac2_store() else {
        return;
    };
    let placements =
        ac2_scene_placements(&store, CellId::new(0xCB4D_0001)).expect("scenery generates");
    let mut resolved = 0;
    for placement in &placements {
        if let Some((visual, _scale)) =
            ac2_scene_object_visual(&store, placement.object).expect("object decodes")
        {
            assert!(
                matches!(visual.raw() >> 24, 0x1F | 0x04),
                "{:#010x}",
                visual.raw()
            );
            resolved += 1;
        }
    }
    assert!(
        resolved * 10 >= placements.len() * 9,
        "{resolved} of {} resolve",
        placements.len()
    );
}
