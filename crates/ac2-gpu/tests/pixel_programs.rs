//! Every authored `ps.1.1` shader in the Legions materials parses and translates to WGSL the
//! device accepts. Runs when `AC2_DAT_DIR` is set and a GPU adapter exists.

use std::collections::BTreeSet;
use std::sync::Arc;

use ac2_core::DataId;
use ac2_dat::DatFileKind;
use ac2_db::ResourceStore;
use ac2_gpu::{Gpu, ObjectRenderer};
use ac2_render::object::parse_pixel_shader;

#[test]
fn every_legions_pixel_shader_translates() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let Ok(gpu) = Gpu::headless() else {
        eprintln!("no GPU adapter; skipping");
        return;
    };
    let store = ResourceStore::new(Arc::clone(&dataset));
    ac2_db::install_ac2_property_types(&store).unwrap();
    let mut sources = BTreeSet::new();
    for prefix in [0x2B00_0000_u32, 0x3100_0000] {
        for did in dataset.ids_in_range(DatFileKind::Portal, prefix, prefix | 0x00FF_FFFF) {
            let Some(material) = ac2_assets::load_ac2_material(&store, DataId::new(did)).unwrap()
            else {
                continue;
            };
            for layer in material.layers {
                if !layer.pixel_shader.is_empty() {
                    sources.insert(layer.pixel_shader.to_vec());
                }
            }
        }
    }
    assert!(sources.len() >= 20, "Legions ships about 21 pixel shaders");
    for source in &sources {
        let program = parse_pixel_shader(source)
            .unwrap_or_else(|error| panic!("{error}:\n{}", String::from_utf8_lossy(source)));
        if let Some(error) = ObjectRenderer::validate_program(&gpu, &program) {
            panic!("{error}\n{}", String::from_utf8_lossy(source));
        }
    }
}
