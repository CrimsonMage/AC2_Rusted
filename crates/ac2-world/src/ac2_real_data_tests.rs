//! Real-data checks against an Asheron's Call 2 installation. They run only when
//! `AC2_DAT_DIR` points at an AC2 DAT directory, so ordinary test runs stay hermetic.

use std::sync::Arc;

use ac2_assets::resolve_ac2_material_texture_chain;
use ac2_dat::{DatDataset, DatOpenOptions};
use ac2_db::ResourceStore;

use crate::ac2_land_defs::load_ac2_land_defs;
use crate::ac2_terrain_desc::load_ac2_terrain_desc;
use crate::surface_desc::load_ac2_surface_desc;

fn ac2_store() -> Option<ResourceStore> {
    let root = std::env::var_os("AC2_DAT_DIR")?;
    let dataset = DatDataset::open(DatOpenOptions::new(root)).expect("AC2 DATs mount");
    Some(ResourceStore::new(Arc::clone(&dataset)))
}

#[test]
fn ac2_terrain_descriptors_resolve_to_a_texture_chain() {
    let Some(store) = ac2_store() else {
        return;
    };
    let land = load_ac2_land_defs(&store)
        .expect("land defs decode")
        .expect("land defs exist");
    assert_eq!(land.height_table().len(), 256);
    assert_eq!(land.logical_cell_size(), 20.0);
    assert_eq!(land.block_cells(), 8);

    let terrain = load_ac2_terrain_desc(&store)
        .expect("terrain desc decodes")
        .expect("terrain desc exists");
    let surfaces = load_ac2_surface_desc(&store)
        .expect("CSurfaceDesc decodes to the last byte")
        .expect("CSurfaceDesc exists");
    assert_eq!(surfaces.cloud_texture.raw() & 0xFF00_0000, 0x4000_0000);

    // Terrain 19 dominates the sampled AC2 landblocks.
    let surface = terrain
        .surface_type(19)
        .expect("terrain 19 has a surface type");
    let flat = crate::surface_info::vertex_info(
        surfaces
            .surface(surface)
            .expect("surface type is described"),
        0.0,
        2032,
        2032,
    )
    .expect("flat ground has a material group");
    assert!(flat.corner_alpha.is_some() && flat.side_alpha.is_some());
    let material = flat.base_material.expect("base material");
    let chain = resolve_ac2_material_texture_chain(&store, material)
        .expect("material decodes")
        .expect("material references a texture chain");
    assert_eq!(chain.raw() & 0xFF00_0000, 0x4000_0000);
}

#[test]
fn ac2_landblock_loads_with_metric_heights_and_surface_textures() {
    let Some(store) = ac2_store() else {
        return;
    };
    let source = crate::Ac2WorldSource::open(Arc::new(store))
        .expect("AC2 vocabulary decodes")
        .expect("AC2 dataset recognised");
    let block = source
        .load_landblock(ac2_core::CellId::new(0x7F7F_0001))
        .expect("AC2 landblock decodes")
        .expect("AC2 landblock exists");
    let heights = block.height_grid();
    assert_eq!(
        block.vertex_side(),
        17,
        "blocks keep their native 17 x 17 grid"
    );
    assert_eq!(heights.len(), 289);
    assert!(heights.iter().all(|h| (0.0..600.0).contains(h)));
    assert!(
        heights.iter().any(|h| *h > 100.0),
        "0x7F7F is upland terrain"
    );
    let mut roads = 0;
    for x in 0..17 {
        for y in 0..17 {
            let [nx, ny, nz] = block.normal(x, y).expect("vertex in range");
            assert!(((nx * nx + ny * ny + nz * nz) - 1.0).abs() < 1e-4 && nz > 0.0);
            let info = block.surface_info(x, y).expect("vertex in range");
            let base = info
                .base_material
                .expect("every vertex picks a base material");
            assert_eq!(base.raw() & 0xFF00_0000, 0x2B00_0000);
            assert_eq!(info.corner_alpha.unwrap().raw() & 0xFF00_0000, 0x4000_0000);
            roads += usize::from(block.road_info(x, y).unwrap().base_material.is_some());
        }
    }
    assert!(roads < 289, "{roads} road vertices");
}

#[test]
fn ac2_region_publishes_sky_lighting_and_fog() {
    let Some(store) = ac2_store() else {
        return;
    };
    let source = crate::Ac2WorldSource::open(Arc::new(store))
        .expect("AC2 vocabulary decodes")
        .expect("AC2 dataset recognised");
    let region = source
        .region()
        .expect("AC2 region, sky, days and game time decode");
    assert_eq!(region.region_number(), 1);
    // `GameTime` 0x35000000: a 7347 s day; sample every hour-ish across one day.
    let mut pitches = Vec::new();
    for step in 0..24 {
        let sample = region
            .sample_sky_environment(f64::from(step) * 7347.0 / 24.0)
            .expect("every time of day has a lighting key");
        assert!(sample.directional_brightness() > 0.0);
        pitches.push(sample.directional_pitch_degrees());
    }
    assert!(pitches.iter().any(|p| *p < 60.0) && pitches.iter().any(|p| *p > 100.0));
    assert!(region.sky_day_context_at(3600.0).is_some());
    // `CDayDesc::GetSky`: some sky object is up at every hour, and every graphic is a Setup or
    // visual description.
    for step in 0..24 {
        let objects = region
            .sky_objects(f64::from(step) * 7347.0 / 24.0)
            .expect("the day has sky objects");
        if std::env::var_os("AC2_DEBUG_SKY").is_some() {
            eprintln!("{step:2}: {objects:?}");
        }
        assert!(objects.iter().any(|object| object.gfx.is_some()));
        for gfx in objects.iter().filter_map(|object| object.gfx) {
            assert!(matches!(gfx.raw() >> 24, 0x04 | 0x1F), "{gfx:?}");
        }
    }
}

#[test]
fn ac2_town_block_places_entities_with_visuals() {
    let Some(store) = ac2_store() else {
        return;
    };
    let source = crate::Ac2WorldSource::open(Arc::new(store))
        .expect("AC2 vocabulary decodes")
        .expect("AC2 dataset recognised");
    // 0xCB4D carries the most `CLandBlockInfo` entities in the Legions cell DAT (297).
    let markers = source
        .entity_markers(ac2_core::CellId::new(0xCB4D_0001))
        .expect("CLandBlockInfo decodes");
    assert!(
        markers.len() > 250,
        "{} entities resolve to a VisualDesc",
        markers.len()
    );
    let block_origin = [f32::from(0xCB_u8) * 160.0, f32::from(0x4D_u8) * 160.0];
    for marker in markers.iter() {
        assert_eq!(marker.visual().raw() & 0xFF00_0000, 0x1F00_0000);
        let origin = marker.world_frame().origin;
        assert!((block_origin[0] - 1.0..block_origin[0] + 161.0).contains(&origin.x));
        assert!((block_origin[1] - 1.0..block_origin[1] + 161.0).contains(&origin.y));
    }
}
