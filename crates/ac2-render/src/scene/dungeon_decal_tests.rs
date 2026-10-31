//! `DecalEffect::ConstructDungeonDecal` on a real dungeon block (`AC2_DAT_DIR`).

use std::sync::Arc;

use ac2_core::DataId;
use ac2_db::ResourceStore;
use ac2_math::Vec3;
use ac2_world::Ac2WorldSource;

use crate::cell_geometry::nearby_geometry;
use crate::decal::DecalSpec;
use crate::scene::WorldScene;

const SIZE: f32 = 8.0;

/// A decal fired inside the cells of dungeon block `0x02BB` clips against the cell's mesh (and the
/// neighbours' behind portals its box reaches): its triangles lie in the planes of source
/// triangles, inside the decal's box.
#[test]
fn a_decal_in_a_dungeon_cell_lands_on_the_cells_surfaces() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let world = Ac2WorldSource::open(Arc::new(ResourceStore::new(dataset)))
        .unwrap()
        .unwrap();
    let mut scene = WorldScene::new(Arc::clone(&world), 1);
    scene.set_center((0x02, 0xBB));
    let _ = scene.wait();
    let cull = scene.blocks[&(0x02, 0xBB)]
        .cull
        .clone()
        .expect("the dungeon block keeps its cells");
    let mut ids: Vec<u32> = cull.cells.keys().copied().collect();
    ids.sort_unstable();
    let mut checked = 0;
    for id in ids.into_iter().take(12) {
        let cell = &cull.cells[&id];
        // Low in the cell, where the floor is within the box.
        let mut origin = (cell.bbox.min + cell.bbox.max) * 0.5;
        origin.z = cell.bbox.min.z + 0.5;
        let spec = DecalSpec {
            material: DataId::new(0x3100_0001),
            lifespan: 30.0,
            size: SIZE,
            direction: Vec3::Y,
            color: [1.0; 4],
            draw_once: false,
            origin,
            cell: id,
        };
        let source = nearby_geometry(&cull.cells, id, origin, SIZE).expect("the cell has a mesh");
        let ground = scene
            .dungeon_decal_ground(&spec)
            .expect("the visible cell is resident");
        assert_eq!(ground.len() * 3, source.origins.len());
        let decal = crate::decal::DecalEffect::new(spec, 0.0, &ground);
        if source.origins.is_empty() {
            assert!(decal.alive() && decal.triangles().is_empty());
            continue;
        }
        let half = f64::from(SIZE) * 0.5 + 0.1;
        for tri in decal.triangles() {
            for corner in tri {
                let p = Vec3::from_array(corner.position) - Vec3::new(0.0, 0.0, 0.015);
                assert!(
                    (f64::from(p.x - origin.x)).abs() <= half
                        && (f64::from(p.y - origin.y)).abs() <= half
                        && (f64::from(p.z - origin.z)).abs() <= half,
                    "{id:08X}: {p} outside the box"
                );
                // On the plane of a source triangle.
                let on_surface = source.origins.chunks_exact(3).any(|t| {
                    let n = (t[1] - t[0]).cross(t[2] - t[0]).normalize_or_zero();
                    n != Vec3::ZERO && n.dot(p - t[0]).abs() < 2e-3
                });
                assert!(on_surface, "{id:08X}: {p} is on no cell surface");
            }
            checked += 1;
        }
    }
    assert!(checked > 0, "a decal in the dungeon made triangles");
}

/// A dungeon decal whose cell is not resident makes none (`ConstructDungeonDecal` fails).
#[test]
fn a_decal_in_an_unknown_cell_makes_none() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let world = Ac2WorldSource::open(Arc::new(ResourceStore::new(dataset)))
        .unwrap()
        .unwrap();
    let mut scene = WorldScene::new(world, 1);
    scene.start_object_decal(DecalSpec {
        material: DataId::new(0x3100_0001),
        lifespan: 30.0,
        size: SIZE,
        direction: Vec3::Y,
        color: [1.0; 4],
        draw_once: false,
        origin: Vec3::ZERO,
        cell: 0x02BB_0100,
    });
    assert_eq!(scene.decals.count(), 0);
}
