//! The near landscape against real AC2 data (`AC2_DAT_DIR`).

use std::collections::BTreeSet;
use std::sync::Arc;

use ac2_core::{CellId, DataId};
use ac2_dat::{DatDataset, DatOpenOptions};
use ac2_db::ResourceStore;
use ac2_world::Ac2WorldSource;

use super::build_terrain_mesh;
use crate::{load_material_image, load_texture_image};

fn world() -> Option<Arc<Ac2WorldSource>> {
    let root = std::env::var_os("AC2_DAT_DIR")?;
    let dataset = DatDataset::open(DatOpenOptions::new(root)).expect("AC2 DATs mount");
    Ac2WorldSource::open(Arc::new(ResourceStore::new(dataset))).expect("AC2 vocabulary decodes")
}

#[test]
fn landblocks_build_blended_meshes_whose_layers_all_load() {
    let Some(world) = world() else {
        return;
    };
    let store = world.resource_store();
    let mut materials = BTreeSet::<DataId>::new();
    let mut masks = BTreeSet::<DataId>::new();
    for block in [0x7F7F_0001_u32, 0xCB4D_0001] {
        let id = CellId::new(block);
        let landblock = world.load_landblock(id).unwrap().unwrap();
        let mesh = build_terrain_mesh(id, &landblock, world.block_size());
        let triangles: usize = mesh.batches.iter().map(|b| b.indices.len() / 3).sum();
        assert!(
            triangles > 400 && triangles <= 512,
            "{block:08x}: {triangles} triangles"
        );
        assert!(
            mesh.batches.iter().any(|b| b.layers.0.len() > 1),
            "some polygons blend"
        );
        for batch in &mesh.batches {
            for layer in &batch.layers.0 {
                materials.insert(layer.material);
                masks.extend(layer.alpha.map(|(did, _)| did));
            }
        }
    }
    for material in &materials {
        let image = load_material_image(store, *material).unwrap();
        assert!(
            image.is_some(),
            "material {:08x} has a base texture",
            material.raw()
        );
    }
    for mask in &masks {
        let image = load_texture_image(store, *mask)
            .unwrap()
            .expect("alpha mask decodes");
        let base = &image.levels[0].rgba;
        let channel = |c: usize| {
            let values: BTreeSet<u8> = base.iter().skip(c).step_by(4).copied().collect();
            (
                values.first().copied(),
                values.last().copied(),
                values.len(),
            )
        };
        eprintln!(
            "mask {:08x} {}x{} r{:?} a{:?}",
            mask.raw(),
            image.width(),
            image.height(),
            channel(0),
            channel(3)
        );
    }
}
