//! Real-data checks of an interior cell's animated lights. `CEnvCell::flush_cells` bakes each
//! light of the cell at the waveforms' values of the moment (`LightSource::CalculateAttenuation`),
//! leaving out the lights whose range animates, and `CEnvCell::UpdateDynamicLights` adds those at
//! the client time of the cell's first light tick.

use std::sync::Arc;

use ac2_assets::Ac2Mesh;
use ac2_core::CellId;
use ac2_dat::DatFileKind;
use ac2_db::CachedResource;
use ac2_db::{ResourceStore, TypedResourceRef};
use ac2_world::{Ac2EnvCell, Ac2LightInfo, Ac2WorldSource};

use super::cell::{bake_cell_colors, bake_static_colors, CellLighting};

/// A block's light info and a cell of it that an animated point light reaches, with its mesh.
fn animated_cell(
    animated: impl Fn(&ac2_world::Ac2StaticLight) -> bool,
    world: &Ac2WorldSource,
    store: &ResourceStore,
    dataset: &ac2_dat::DatDataset,
) -> Option<(
    CachedResource<Ac2LightInfo>,
    CachedResource<Ac2EnvCell>,
    CachedResource<Ac2Mesh>,
)> {
    for did in dataset.ids_in_range(DatFileKind::Cell, 0, u32::MAX) {
        if did & 0xFFFF != 0xFFFC {
            continue;
        }
        let Some(info) = world.light_info(CellId::new(did)).ok().flatten() else {
            continue;
        };
        for light in info.lights.iter().filter(|l| animated(l)) {
            for (cell_id, indices) in light.lit.iter() {
                if indices.is_empty() {
                    continue;
                }
                let Some(cell) = world.env_cell(*cell_id).ok().flatten() else {
                    continue;
                };
                let Some(mesh) = store
                    .load_cached_packed::<Ac2Mesh>(TypedResourceRef::new(
                        DatFileKind::Portal,
                        cell.cell_mesh,
                        0,
                    ))
                    .ok()
                    .flatten()
                else {
                    continue;
                };
                if info.num_lights(cell.cell_id) > 0 {
                    return Some((info.clone(), cell, mesh));
                }
            }
        }
    }
    None
}

fn lighting(info: &Ac2LightInfo, time: f64) -> CellLighting<'_> {
    CellLighting {
        light_info: Some(info),
        interior_ambient: [0.0; 3],
        time,
    }
}

/// A light whose intensity waveform animates lights the cell's walls as bright as the waveform is
/// when the cell bakes (`flush_cells` evaluates `CalculateAttenuation` at the time of the bake).
#[test]
fn a_cells_static_bake_follows_its_lights_intensity_waveform_at_the_bake_time() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(Arc::clone(&dataset)));
    let world = Ac2WorldSource::open(Arc::clone(&store)).unwrap().unwrap();
    let (info, cell, mesh) = animated_cell(
        |light| light.intensity_wave.kind != ac2_math::waveform::kind::NONE,
        &world,
        &store,
        &dataset,
    )
    .expect("the data has a cell lit by an animated light");
    let samples: Vec<Vec<u32>> = (0..40)
        .map(|step| bake_cell_colors(&mesh, &cell, lighting(&info, f64::from(step) * 0.173)))
        .collect();
    let differing = samples.iter().filter(|c| **c != samples[0]).count();
    assert!(differing > 0, "the animated light never changed the cell");
    // No light of the shipped data animates its range, so the first tick adds nothing.
    assert!(info.lights.iter().all(|light| !light.is_range_animating()));
}

/// The lights whose range animates stay out of the static bake and join it, at the waveforms'
/// values of the first tick, in `UpdateDynamicLights` (no shipped light does this, so one real
/// light of a real cell gets a sine range).
#[test]
fn a_range_animated_light_joins_the_static_bake_at_the_first_light_tick() {
    use ac2_math::Waveform;
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(Arc::clone(&dataset)));
    let world = Ac2WorldSource::open(Arc::clone(&store)).unwrap().unwrap();
    let (info, cell, mesh) = animated_cell(|_| true, &world, &store, &dataset)
        .expect("the data has a cell with a lit light");
    let key = (0..info.num_lights(cell.cell_id))
        .filter_map(|n| info.light(cell.cell_id, n))
        .find(|(key, light)| {
            light.kind == ac2_world::Ac2LightKind::Point
                && info.lit_indices(cell.cell_id, *key).is_some()
        })
        .map(|(key, _)| key)
        .expect("a point light reaches the cell");
    let mut modified = (*info).clone();
    modified.lights = modified
        .lights
        .iter()
        .enumerate()
        .map(|(index, light)| {
            let mut light = light.clone();
            light.intensity_wave = Waveform::constant(light.intensity);
            if index == key as usize {
                light.range_wave = Waveform {
                    kind: ac2_math::waveform::kind::SINE,
                    base: 6.0,
                    amplitude: 4.0,
                    frequency: 0.5,
                    ..Waveform::default()
                };
            }
            light
        })
        .collect::<Vec<_>>()
        .into();
    let statics = |time| bake_static_colors(&mesh, &cell, lighting(&modified, time));
    // The static bake leaves the animated light out, whatever the clock says.
    assert_eq!(statics(0.0), statics(0.5));
    let at = |time| bake_cell_colors(&mesh, &cell, lighting(&modified, time));
    let (peak, trough) = (at(0.5), at(1.5));
    assert_ne!(
        peak, trough,
        "the light's range moves the surfaces it reaches"
    );
    let base = statics(0.0);
    assert!(peak.iter().zip(&base).any(|(lit, base)| lit != base));
    for colors in [&peak, &trough] {
        for (lit, base) in colors.iter().zip(&base) {
            for shift in [0, 8, 16] {
                assert!(
                    (lit >> shift) & 0xFF >= (base >> shift) & 0xFF,
                    "lights only add"
                );
            }
        }
    }
}
/// `CEnvCell::DrawCellMesh` draws the cell mesh's degrade for a factor of 1 (the cell's
/// `m_degradeLevel`), capped at 0.5 by the medium detail setting and 0 by the low one.
#[test]
fn a_cell_mesh_draws_the_degrade_its_detail_setting_picks() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(Arc::clone(&dataset)));
    let mesh = store
        .load_cached_packed::<Ac2Mesh>(TypedResourceRef::new(
            DatFileKind::Portal,
            ac2_core::DataId::new(0x0100_0022),
            0,
        ))
        .unwrap()
        .expect("a cell mesh with three degrades");
    assert_eq!(mesh.fragments.iter().map(|f| f.degrade).max(), Some(2));
    assert_eq!(super::cell::cell_degrade(&mesh, 2), 0);
    assert_eq!(super::cell::cell_degrade(&mesh, 1), 1);
    assert_eq!(super::cell::cell_degrade(&mesh, 0), 2);
}
