//! Real-data checks of the scenery generator. Run only when `AC2_DAT_DIR` points at an AC2 DAT
//! directory; they assert invariants, not stored outputs, so no game data lives in the repo.

use std::sync::Arc;

use ac2_core::CellId;
use ac2_db::ResourceStore;

use super::{ac2_block_async_load_dids, ac2_scene_object_visual, ac2_scene_placements};
use ac2_db::install_ac2_property_types;

fn ac2_store() -> Option<ResourceStore> {
    let dataset = ac2_dat::test_support::env_dataset()?;
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
        assert!(
            first.iter().any(|p| !p.frill),
            "{block:04X}: scenery objects"
        );
        for p in &first {
            let [x, y, z] = p.origin.to_array();
            assert!(
                (-0.01..160.01).contains(&x) && (-0.01..160.01).contains(&y),
                "{p:?}"
            );
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

/// Retail tests only a group's location against the clearance list (and not groups that skip the
/// legal-position check, nor objects a vertex scene places directly), so placements may still sit
/// inside a clearance; this checks the shapes themselves.
#[test]
fn entity_clearance_shapes_decode_across_the_cell_dat() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ResourceStore::new(Arc::clone(&dataset));
    install_ac2_property_types(&store).expect("MasterProperty decodes");
    let (mut blocks, mut boxes, mut cylinders) = (0, 0, 0);
    let infos = dataset
        .iter_ids(ac2_dat::DatFileKind::Cell)
        .filter(|id| id & 0xFFFF == 0xFFFE);
    for info in infos {
        let clearances = super::block::load_clearances(&store, info).expect("clearances decode");
        let mut shaped = false;
        for c in &clearances {
            for (min, max) in &c.boxes {
                // Some authored boxes are inverted; `IntersectsBox` measures them as given.
                assert!(min.iter().chain(max).all(|v| v.is_finite()), "{info:08X}");
                boxes += 1;
                shaped = true;
            }
            for cyl in &c.cylinders {
                assert!(
                    cyl.radius.is_finite() && cyl.height.is_finite(),
                    "{info:08X}: {cyl:?}"
                );
                cylinders += 1;
                shaped = true;
            }
        }
        blocks += usize::from(shaped);
    }
    eprintln!("{blocks} blocks; {boxes} clearance boxes, {cylinders} cylinders");
    assert!(blocks > 0 && boxes + cylinders > 0);
}

#[test]
fn landblocks_carry_ambient_sound_points_of_the_region_sound_desc() {
    let Some(store) = ac2_store() else {
        return;
    };
    assert_eq!(
        crate::ac2_region_sound_desc(&store).unwrap(),
        Some(ac2_core::DataId::new(0x1000_0000)),
        "the region's sound desc"
    );
    for block in [0xCB4D_u32, 0xF18F, 0x7F7F] {
        let points = super::ac2_ambient_sound_points(&store, CellId::new(block << 16 | 1))
            .expect("points decode");
        assert!(!points.is_empty(), "{block:04X}: the land sounds");
        for p in &points {
            assert!(p.sound_type != 0);
            // `block_length / side_vertex_count` spacing: the last vertex short of the edge.
            assert!((0.0..160.0).contains(&p.origin[0]) && (0.0..160.0).contains(&p.origin[1]));
            assert!(p.origin[2].is_finite());
        }
    }
}

#[test]
fn a_blocks_async_load_dids_are_its_vertices_scene_files() {
    let Some(store) = ac2_store() else {
        return;
    };
    let id = CellId::new(0xCB4D_0001);
    let dids = ac2_block_async_load_dids(&store, id).expect("scene picks");
    // One per vertex of the 17 x 17 lattice.
    assert_eq!(dids.len(), 17 * 17);
    assert!(
        dids.iter().any(|did| did.raw() != 0),
        "some vertex has a scene"
    );
    // The generator places scenery from exactly these scene files, so a block with scenery has some.
    assert!(!ac2_scene_placements(&store, id).unwrap().is_empty());
}

/// The retail grass reference (12.1N 49.2W, block 0x408E) stands in a meadow whose vertex scenes
/// nest the grid grass scene (`0x020000B6`: freq-1 clumps 2 to 2.5 m apart); the town block
/// 0xCB4D only has scattered tufts (`0x020000F8`, about two per vertex). Both are what
/// `ProcScene`'s frill groups author, so the meadow must carry far denser 10 m patches.
#[test]
fn the_reference_meadow_carries_dense_grid_grass_and_the_town_scattered_tufts() {
    let Some(store) = ac2_store() else {
        return;
    };
    let densest = |block: u32| {
        let frills: Vec<_> = ac2_scene_placements(&store, CellId::new(block << 16 | 1))
            .expect("scenery generates")
            .into_iter()
            .filter(|p| p.frill)
            .map(|p| p.origin)
            .collect();
        (0..16)
            .flat_map(|ix| (0..16).map(move |iy| (ix as f32 * 10.0, iy as f32 * 10.0)))
            .map(|(x0, y0)| {
                frills
                    .iter()
                    .filter(|o| (x0..x0 + 10.0).contains(&o.x) && (y0..y0 + 10.0).contains(&o.y))
                    .count()
            })
            .max()
            .unwrap_or(0)
    };
    let (meadow, town) = (densest(0x408E), densest(0xCB4D));
    eprintln!("densest 10 m frill patch: meadow {meadow}, town {town}");
    assert!(meadow >= 20, "densest meadow patch {meadow}");
    assert!(
        town < meadow / 2,
        "densest town patch {town} vs meadow {meadow}"
    );
}
