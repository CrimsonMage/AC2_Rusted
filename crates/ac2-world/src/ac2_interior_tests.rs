//! Real-data checks of the AC2 interior decoders over every record of a Legions installation.
//! They run only when `AC2_DAT_DIR` points at an AC2 DAT directory.

use std::collections::HashMap;
use std::sync::Arc;

use ac2_core::{CellId, DataId};
use ac2_dat::{DatDataset, DatFileKind};
use ac2_db::ResourceStore;
use ac2_math::Vec3;

use crate::{Ac2LightKind, Ac2WorldSource};

fn ac2_world() -> Option<(Arc<DatDataset>, Arc<Ac2WorldSource>)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = Arc::new(ResourceStore::new(Arc::clone(&dataset)));
    let world = Ac2WorldSource::open(store)
        .expect("AC2 vocabulary decodes")
        .expect("AC2 dataset recognised");
    Some((dataset, world))
}

fn cell_ids(dataset: &DatDataset, low: impl Fn(u32) -> bool) -> Vec<u32> {
    dataset
        .ids_in_range(DatFileKind::Cell, 0, u32::MAX)
        .filter(|did| low(did & 0xFFFF))
        .collect()
}

#[test]
fn every_ac2_cell_mesh_structure_decodes_exactly() {
    let Some((dataset, world)) = ac2_world() else {
        return;
    };
    let meshes = dataset
        .ids_in_range(DatFileKind::Portal, 0x0100_0000, 0x01FF_FFFF)
        .collect::<Vec<_>>();
    assert_eq!(meshes.len(), 329);
    let mut portals = 0;
    for did in &meshes {
        let structure = world
            .cell_struct(DataId::new(*did))
            .unwrap_or_else(|error| panic!("cell mesh {did:08X}: {error}"))
            .expect("listed record exists");
        assert!(structure.cell_bsp.is_some());
        portals += structure.portal_polygons.len();
        // The render decoder reads the same record's vertices and fragments.
        let mesh = world
            .resource_store()
            .load_cached_packed::<ac2_assets::Ac2Mesh>(ac2_db::TypedResourceRef::new(
                DatFileKind::Portal,
                DataId::new(*did),
                0,
            ))
            .expect("cell mesh render data decodes")
            .expect("listed record exists");
        assert_eq!(mesh.vertices.len(), structure.positions.len());
    }
    assert_eq!(portals, 934);
}

#[test]
fn every_ac2_env_cell_decodes_and_resolves_its_structure() {
    let Some((dataset, world)) = ac2_world() else {
        return;
    };
    let ids = cell_ids(&dataset, |low| (0x0100..0xFFFC).contains(&low));
    assert_eq!(ids.len(), 57_984);
    let (mut outside, mut inside_own, mut with_key) = (0, 0, 0);
    for did in &ids {
        let cell = world
            .env_cell(CellId::new(*did))
            .unwrap_or_else(|error| panic!("env cell {did:08X}: {error}"))
            .expect("listed record exists");
        let structure = world
            .cell_struct(cell.cell_mesh)
            .expect("cell mesh decodes")
            .expect("cell mesh exists");
        for portal in cell.portals.iter() {
            assert!(
                (portal.polygon as usize) < structure.portal_polygons.len(),
                "{did:08X} portal"
            );
            outside += usize::from(portal.other_cell.is_none());
        }
        with_key += usize::from(cell.appearance_key.is_some());
        // The centre of the mesh bounds lies inside nearly every cell's own CELL_BSP.
        let centre = (structure.bounds_min + structure.bounds_max) * 0.5;
        let world_centre = world
            .env_cell_world_frame(&cell)
            .local_to_global_point(centre);
        inside_own += usize::from(world.env_cell_contains(&cell, world_centre).unwrap());
    }
    assert_eq!(outside, 167);
    assert_eq!(with_key, 36_654 - 1_464);
    assert!(
        inside_own * 10 > ids.len() * 9,
        "{inside_own} of {} cells contain their centre",
        ids.len()
    );
}

#[test]
fn every_ac2_landblock_interior_and_light_info_decodes() {
    let Some((dataset, world)) = ac2_world() else {
        return;
    };
    let infos = cell_ids(&dataset, |low| low == 0xFFFE);
    let mut cells = 0;
    for did in &infos {
        let interior = world
            .landblock_interior(CellId::new(*did))
            .unwrap_or_else(|error| panic!("landblock info {did:08X}: {error}"))
            .expect("listed record exists");
        cells += interior.cell_count as usize;
        for stab in interior.outside_stab_cells.iter() {
            assert_eq!(stab.raw() & 0xFFFF_0000, did & 0xFFFF_0000);
        }
    }
    // Two landblocks carry env cell records without a LandBlockInfo listing them.
    assert_eq!(cells, 57_013);
    let lights = cell_ids(&dataset, |low| low == 0xFFFC);
    assert_eq!(lights.len(), 1_042);
    let mut vertex_counts = HashMap::<DataId, usize>::new();
    let mut point_lights = 0;
    for did in &lights {
        let info = world
            .light_info(CellId::new(*did))
            .unwrap_or_else(|error| panic!("light info {did:08X}: {error}"))
            .expect("listed record exists");
        for light in info.lights.iter() {
            point_lights += usize::from(light.kind == Ac2LightKind::Point);
            for (cell, indices) in light.lit.iter() {
                let Some(env) = world.env_cell(*cell).unwrap() else {
                    continue;
                };
                let vertices = *vertex_counts.entry(env.cell_mesh).or_insert_with(|| {
                    world
                        .cell_struct(env.cell_mesh)
                        .unwrap()
                        .map_or(0, |s| s.positions.len())
                });
                assert!(
                    indices.iter().all(|index| usize::from(*index) < vertices),
                    "{cell:?} lit index"
                );
            }
        }
    }
    assert_eq!(point_lights, 39_859);
}

#[test]
fn light_info_cell_lookup_lists_the_lights_that_light_the_cell() {
    let Some((dataset, world)) = ac2_world() else {
        return;
    };
    let mut checked = 0;
    let mut sunlit = 0;
    for did in cell_ids(&dataset, |low| low == 0xFFFC) {
        let info = world
            .light_info(CellId::new(did))
            .unwrap()
            .expect("listed record exists");
        for (key, light) in info.lights.iter().enumerate() {
            for (cell, indices) in light.lit.iter() {
                // `GetLitIndices` answers the light's own list; `None` for an empty one.
                let lit = info.lit_indices(*cell, key as u32);
                assert_eq!(lit.is_some(), !indices.is_empty());
                if let Some(lit) = lit {
                    assert_eq!(lit, &indices[..]);
                    // A light that lights a cell is one of the cell's lookup lights.
                    let listed = (0..info.num_lights(*cell))
                        .filter_map(|n| info.light(*cell, n))
                        .any(|(listed, _)| listed as usize == key);
                    assert!(listed, "{cell:?} lookup lacks light {key}");
                    checked += 1;
                }
            }
        }
        assert!(info.light(CellId::new(did), usize::MAX).is_none());
        for tod in 0..9 {
            for light in info.lights.iter().flat_map(|l| l.lit.iter()) {
                sunlit += usize::from(info.sun_lit_indices(light.0, tod).is_some());
            }
        }
        assert!(info.sun_lit_indices(CellId::new(did), 9).is_none());
    }
    assert!(checked > 1_000, "{checked}");
    eprintln!("sun-lit cell lists: {sunlit}");
}

#[test]
fn ac2_dungeon_cell_is_found_from_a_point_inside_it() {
    let Some((_, world)) = ac2_world() else {
        return;
    };
    let cell = world
        .env_cell(CellId::new(0x02BB_0100))
        .unwrap()
        .expect("dungeon cell exists");
    let structure = world
        .cell_struct(cell.cell_mesh)
        .unwrap()
        .expect("cell mesh exists");
    let centre = (structure.bounds_min + structure.bounds_max) * 0.5;
    let point = world
        .env_cell_world_frame(&cell)
        .local_to_global_point(centre);
    let found = world
        .find_env_cell(
            point,
            cell.visible_cells.iter().copied().chain([cell.cell_id]),
        )
        .unwrap()
        .expect("a stab-list cell contains the point");
    assert_eq!(found.cell_id.raw() & 0xFFFF_0000, 0x02BB_0000);
    let collision = world
        .env_cell_collision(cell.cell_id)
        .unwrap()
        .expect("collision source");
    assert!(!collision.world_triangles.is_empty());
    assert!(collision.physics_bsp().is_some());
    assert!(collision
        .world_triangles
        .iter()
        .flatten()
        .all(|p| p.is_finite() && *p != Vec3::ZERO));
}
