//! Real-data parity with the Python reference generator. Runs only when `AC2_DAT_DIR`
//! points at the Legions DAT directory.

use std::sync::Arc;

use ac2_dat::{DatDataset, DatOpenOptions};
use ac2_core::CellId;
use ac2_math::Vec3;
use ac2_db::ResourceStore;

use super::real_data_expected::{
    Expected, FRILLS_CB4D, FRILLS_F18F, FRILL_SUMS_CB4D, FRILL_SUMS_F18F, OBJECTS_CB4D,
    OBJECTS_F18F,
};
use super::{ac2_scene_object_visual, ac2_scene_placements, Ac2ScenePlacement};

fn ac2_store() -> Option<ResourceStore> {
    let root = std::env::var_os("AC2_DAT_DIR")?;
    let dataset = DatDataset::open(DatOpenOptions::new(root)).expect("AC2 DATs mount");
    Some(ResourceStore::new(Arc::clone(&dataset)))
}

fn heading(placement: &Ac2ScenePlacement) -> f64 {
    let forward = placement.rotation * Vec3::Y;
    f64::from(forward.x)
        .atan2(f64::from(forward.y))
        .to_degrees()
        .rem_euclid(360.0)
}

fn assert_matches(label: &str, actual: &[&Ac2ScenePlacement], expected: &[Expected]) {
    for (index, (placement, (did, origin, expected_heading, scale))) in
        actual.iter().zip(expected).enumerate()
    {
        assert_eq!(placement.object.raw(), *did, "{label}[{index}] object");
        for axis in 0..3 {
            let got = f64::from(placement.origin.to_array()[axis]);
            assert!(
                (got - origin[axis]).abs() < 0.01,
                "{label}[{index}] origin {:?} vs {origin:?}",
                placement.origin
            );
            let got_scale = f64::from(placement.scale.to_array()[axis]);
            assert!(
                (got_scale - scale[axis]).abs() < 0.001,
                "{label}[{index}] scale {:?} vs {scale:?}",
                placement.scale
            );
        }
        let delta = (heading(placement) - expected_heading + 180.0).rem_euclid(360.0) - 180.0;
        assert!(
            delta.abs() < 0.1,
            "{label}[{index}] heading {} vs {expected_heading}",
            heading(placement)
        );
    }
}

fn check_block(
    store: &ResourceStore,
    block: u32,
    objects: &[Expected],
    frills: &[Expected],
    sums: [f64; 4],
    frill_count: usize,
) {
    let placements =
        ac2_scene_placements(store, CellId::new(block << 16 | 1)).expect("scenery generates");
    let statics = placements.iter().filter(|p| !p.frill).collect::<Vec<_>>();
    let grass = placements.iter().filter(|p| p.frill).collect::<Vec<_>>();
    assert_eq!(statics.len(), objects.len(), "{block:04X} static objects");
    assert_eq!(grass.len(), frill_count, "{block:04X} frills");
    assert_matches(&format!("{block:04X} objects"), &statics, objects);
    assert_matches(&format!("{block:04X} frills"), &grass, frills);
    let mut actual = [0.0_f64; 4];
    for frill in &grass {
        for (sum, value) in actual.iter_mut().zip(frill.origin.to_array()) {
            *sum += f64::from(value);
        }
        actual[3] += f64::from(frill.scale.x);
    }
    for (got, want) in actual.iter().zip(sums) {
        assert!(
            (got - want).abs() < 0.05,
            "{block:04X} frill sums {actual:?} vs {sums:?}"
        );
    }
}

#[test]
fn ac2_scene_placements_match_the_reference_generator() {
    let Some(store) = ac2_store() else {
        return;
    };
    check_block(
        &store,
        0xCB4D,
        OBJECTS_CB4D,
        FRILLS_CB4D,
        FRILL_SUMS_CB4D,
        347,
    );
    check_block(
        &store,
        0xF18F,
        OBJECTS_F18F,
        FRILLS_F18F,
        FRILL_SUMS_F18F,
        750,
    );
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
