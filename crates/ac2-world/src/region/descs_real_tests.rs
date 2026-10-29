//! The region's fog, encounter and property descriptors, and its info lists, on the Legions DAT
//! (`AC2_DAT_DIR`).

use ac2_core::DataId;
use ac2_dat::DatFileKind;
use ac2_db::{ResourcePackedDecode, ResourceStore, TypedResourceRef};

use super::{ac2_region_info, EncounterDesc, FogDesc, InfoList, PropertyDesc, RegionInfo};

fn store() -> Option<ResourceStore> {
    Some(ResourceStore::new(ac2_dat::test_support::env_dataset()?))
}

fn load<T: ResourcePackedDecode + Send + Sync + 'static>(
    store: &ResourceStore,
    did: DataId,
) -> Option<ac2_db::CachedResource<T>> {
    store
        .load_cached_packed::<T>(TypedResourceRef::new(DatFileKind::Portal, did, 0))
        .unwrap_or_else(|error| panic!("{did:?} decodes: {error:?}"))
}

fn region(store: &ResourceStore) -> RegionInfo {
    ac2_region_info(store).unwrap().expect("region record")
}

#[test]
fn region_info_lists_carry_every_kind_of_record() {
    let Some(store) = store() else { return };
    let region = region(&store);
    for list in InfoList::ALL {
        eprintln!("{list:?}: {:?}", region.list(list));
    }
    assert!(region.current(InfoList::Sky).is_some());
    assert!(region.current(InfoList::Water).is_some());
    assert!(!region.sub_data_ids().is_empty());
}

#[test]
fn fog_desc_decodes_from_the_region_fog_list() {
    let Some(store) = store() else { return };
    let region = region(&store);
    for lookup in region.list(InfoList::Fog) {
        let fog = load::<FogDesc>(&store, lookup.data_id).expect("fog desc exists");
        eprintln!("{fog:?}");
        assert!(fog.default_fog.is_some() || !fog.fog_types.is_empty());
    }
}

#[test]
fn encounter_desc_decodes_from_the_region_encounter_list() {
    let Some(store) = store() else { return };
    let region = region(&store);
    for lookup in region.list(InfoList::Encounter) {
        let table = load::<EncounterDesc>(&store, lookup.data_id).expect("encounter exists");
        // The shipped table decodes to the last byte; it carries no rows.
        assert!(table.rows.is_empty());
        assert_eq!(table.sub_data_ids(), vec![table.block_encounter_map]);
    }
}

#[test]
fn property_desc_decodes_from_the_region_property_list() {
    let Some(store) = store() else { return };
    ac2_db::install_ac2_property_types(&store).expect("MasterProperty installs");
    let region = region(&store);
    for lookup in region.list(InfoList::Property) {
        let desc = load::<PropertyDesc>(&store, lookup.data_id).expect("property desc exists");
        assert!(!desc.block_props.is_empty());
        let maps = |did: DataId| crate::load_ac2_block_map(&store, did).unwrap();
        let mut evaluated = 0;
        for prop in &desc.block_props {
            assert!(!prop.map_values.is_empty(), "{:08X}", prop.pname);
            let map = maps(prop.block_map).expect("block map exists");
            // A cell well inside the world, a quarter into its landblock.
            let found = desc.block_property_at(prop, &map, 0xA9B2_0001, [40.0, 40.0]);
            if let Some(found) = found {
                assert_eq!(found.name, prop.pname);
                evaluated += 1;
            }
        }
        assert!(evaluated > 0);
        let all = desc.block_properties_at(&maps, 0xA9B2_0001, [40.0, 40.0]);
        assert_eq!(all.len(), evaluated);
        // Every block of the world is enterable or not by the passability enum; the sample is a
        // mix when the property is present.
        let mut passable = 0;
        for x in (0..255).step_by(5) {
            for y in (0..255).step_by(5) {
                passable += usize::from(desc.passability(&maps, x, y));
            }
        }
        eprintln!("passable sample blocks: {passable} of 2601");
        assert!(passable > 0);
    }
}

#[test]
fn water_desc_lookups_answer_per_type_on_the_dataset() {
    let Some(store) = store() else { return };
    let region = region(&store);
    let did = region.current(InfoList::Water).expect("region water desc");
    let water = crate::Ac2WaterSource::load(&store, did)
        .unwrap()
        .expect("water desc and maps exist");
    let wet: Vec<u32> = (0..64).filter(|index| water.has_water(*index)).collect();
    assert!(!wet.is_empty(), "some water type holds water");
    for index in wet {
        assert!(water.tile_width(index) > 0.0);
        assert_ne!(water.above_material(index).raw(), 0);
        let lights = water
            .water_lighting(index)
            .expect("a water type has lighting");
        assert!(!lights.is_empty());
        assert!(water.vertex_color(index).is_some());
    }
}

#[test]
fn sky_tick_sizes_run_from_the_minimum_to_the_maximum_by_quality() {
    let Some(store) = store() else { return };
    let region = super::load_ac2_region(&store)
        .unwrap()
        .expect("region and sky exist");
    let low = region.sky_tick_size(-1.0).unwrap();
    let middle = region.sky_tick_size(0.5).unwrap();
    let high = region.sky_tick_size(1.0).unwrap();
    assert!(low <= middle && middle <= high, "{low} {middle} {high}");
    assert!(region.sky_light_tick_size(0.5).unwrap() > 0.0);
    // The first day of the sky is the one the region loaded first.
    assert!(region.sky_day_by_index(0).is_some());
    assert!(region.sky_day_by_index(10_000).is_none());
}

#[test]
fn the_dataset_cell_info_layout_packs_the_six_fields_in_order() {
    use crate::cell_info::CellInfoField::*;
    let Some(store) = store() else { return };
    let land = crate::ac2_land_defs::load_ac2_land_defs(&store)
        .unwrap()
        .expect("land defs exist");
    let layout = land.cell_info_layout();
    let mut offset = 0;
    for field in [Terrain, Scene, Encounter, Color, Road, Replace] {
        assert_eq!(layout.offset(field), offset);
        offset += layout.bit_length(field);
    }
    assert!(offset <= 32);
    assert!(layout.bit_length(Terrain) > 0 && layout.bit_length(Road) > 0);
}

#[test]
fn a_blocks_water_index_is_its_type_map_value() {
    let Some(store) = store() else { return };
    let region = region(&store);
    let did = region.current(InfoList::Water).expect("region water desc");
    let water = crate::Ac2WaterSource::load(&store, did)
        .unwrap()
        .expect("water desc and maps exist");
    let table = vec![1000.0_f32; 256];
    let mut wet = 0;
    for x in (0..255u32).step_by(5) {
        for y in (0..255u32).step_by(5) {
            let block = ac2_core::CellId::new((x << 24) | (y << 16));
            let index = water.block_water_index(block);
            // A block that holds water (forced by a high surface and a low floor) reports the
            // same index through `CLandBlockMesh::GetWaterIndex`.
            if let Some(block_water) = water.block_water(block, &table, -1.0) {
                assert_eq!(block_water.water_index(), index);
                assert!(water.has_water(index));
                wet += 1;
            }
        }
    }
    assert!(wet > 0, "some blocks hold water");
}

#[test]
fn sky_time_of_day_keys_decode_every_field_to_sane_values() {
    let Some(store) = store() else { return };
    let region = super::load_ac2_region(&store)
        .unwrap()
        .expect("region and sky exist");
    let sky = region.sky.as_ref().expect("sky");
    let mut keys = 0;
    for day in sky.day_groups.iter() {
        assert!(!day.lightings.is_empty());
        for key in day.lightings.iter() {
            keys += 1;
            // `SkyTimeOfDay::StreamPack` field order: time, fog, direct and ambient light, the
            // replacements, then the rain (all read to the record's end by the day decoder).
            assert!((0.0..=1.0).contains(&key.begin), "begin {}", key.begin);
            assert!(key.world_fog.min_distance <= key.world_fog.max_distance);
            assert!(
                (0.0..=1.0).contains(&key.rain.amount),
                "rain {:?}",
                key.rain
            );
            assert!(key.lighting.dir_brightness.is_finite());
            assert!((0.0..=360.0).contains(&key.lighting.dir_heading));
            assert!(key.lighting.ambient_brightness >= 0.0);
        }
    }
    assert!(keys > 0);
}

#[test]
fn the_region_fog_names_fog_areas_by_landblock_corner() {
    let Some(store) = store() else { return };
    let fog = super::RegionFog::load(&store)
        .unwrap()
        .expect("the region has a fog map");
    // `GetFogCorners`: most of the world has no fog area; some blocks have one at a corner.
    let fogged: Vec<(u32, u32)> = (0..255)
        .flat_map(|x| (0..255).map(move |y| (x, y)))
        .filter(|(x, y)| fog.corners(*x, *y).is_some())
        .collect();
    eprintln!(
        "fogged blocks: {} e.g. {:02X?}",
        fogged.len(),
        &fogged[..fogged.len().min(12)]
    );
    assert!(!fogged.is_empty());
    assert!(fogged.len() < 255 * 255);
}
