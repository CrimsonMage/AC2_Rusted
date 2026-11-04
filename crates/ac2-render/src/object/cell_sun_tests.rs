//! Real-data check of the interior sun pass (`CEnvCell::UpdateDynamicLights` with `bSunOnly`): a
//! dungeon cell's sun-lit vertices for one shadow-generation direction take the sun's colour over
//! their static diffuse where they face it; every other vertex keeps the static diffuse.

use std::sync::Arc;

use ac2_assets::Ac2Mesh;
use ac2_core::CellId;
use ac2_dat::DatFileKind;
use ac2_db::{ResourceStore, TypedResourceRef};
use ac2_world::Ac2WorldSource;

use super::cell::CellLighting;
use super::cell_sun::{CellSun, SunPass};

#[test]
fn a_dungeon_cells_sun_lit_vertices_take_the_sun() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(Arc::clone(&dataset)));
    let world = Ac2WorldSource::open(Arc::clone(&store)).unwrap().unwrap();
    let blocks: Vec<u32> = dataset
        .ids_in_range(DatFileKind::Cell, 0, u32::MAX)
        .filter(|did| did & 0xFFFF == 0xFFFC)
        .collect();
    for did in blocks {
        let block = CellId::new(did | 0xFFFF);
        let Some(info) = world.light_info(block).ok().flatten() else {
            continue;
        };
        let Some(interior) = world.landblock_interior(block).ok().flatten() else {
            continue;
        };
        for id in interior.cell_ids(block) {
            let Some(direction) = (0..9).find(|d| info.sun_lit_indices(id, *d).is_some()) else {
                continue;
            };
            let cell = world.env_cell(id).unwrap().expect("listed cell");
            let mesh = store
                .load_cached_packed::<Ac2Mesh>(TypedResourceRef::new(
                    DatFileKind::Portal,
                    cell.cell_mesh,
                    0,
                ))
                .unwrap()
                .expect("cell mesh");
            let lighting = CellLighting {
                light_info: Some(&info),
                interior_ambient: interior.ambient.unwrap_or([0.0; 3]),
                time: 0.0,
            };
            let sun = CellSun::new(&mesh, &cell, lighting).expect("a sun-lit cell keeps its pass");
            let listed = sun.sun_lit(direction).to_vec();
            // The sun shining straight onto the first listed vertex.
            let normal = mesh.vertices[usize::from(listed[0])].normal;
            let pass = SunPass {
                l0: Some(direction),
                l1: None,
                alpha: 1.0,
                direction: -cell.frame.orientation.mul_vec3(normal),
                color: [0.6, 0.5, 0.4],
                intensity: 1.0,
            };
            let colors = sun.bake(&pass).expect("a direction is chosen");
            let base = sun.static_colors();
            let mut brighter = 0;
            for (index, (lit, base)) in colors.iter().zip(base).enumerate() {
                if listed.contains(&(index as u16)) {
                    brighter += usize::from(lit != base);
                } else {
                    assert_eq!(lit, base, "{id:?} vertex {index} is not sun-lit");
                }
            }
            assert!(brighter > 0, "{id:?}");
            // No direction chosen: the vertices stay as they are.
            assert!(sun.bake(&SunPass { l0: None, ..pass }).is_none());
            return;
        }
    }
    panic!("no dungeon cell has sun-lit vertices");
}
