//! `ObjectRenderer::set_geometry`'s persistent particle buffers: a smaller geometry refills the
//! buffers a larger one made, and the draw range follows the live indices. Runs when `AC2_DAT_DIR`
//! is set (for a real model's part and batch) and a GPU adapter exists.

use ac2_core::DataId;
use ac2_db::ResourceStore;
use ac2_render::object::{load_object_model, ObjectVertex, RenderSettings};
use ac2_render::Image;

use super::ObjectRenderer;
use crate::{Gpu, TextureKey, HDR_FORMAT};

const HUMAN_MALE_VISUAL: u32 = 0x1F00_0023;

fn vertex(x: f32) -> ObjectVertex {
    ObjectVertex {
        position: [x, 0.0, 0.0],
        normal: [0.0, 0.0, 1.0],
        color: [1.0; 4],
        uvs: [[0.0; 2]; 4],
        tangents: [[0.0; 3]; 2],
    }
}

#[test]
fn particle_buffers_grow_then_refill_in_place() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let Ok(gpu) = Gpu::headless() else {
        eprintln!("no GPU adapter; skipping");
        return;
    };
    let store = ResourceStore::new(dataset);
    let Some(model) = load_object_model(
        &store,
        DataId::new(HUMAN_MALE_VISUAL),
        &RenderSettings::default(),
    )
    .unwrap() else {
        return;
    };
    let mut renderer = ObjectRenderer::new(&gpu, HDR_FORMAT);
    let mut no_images = |_key: TextureKey| -> Option<Image> { None };
    let mut uploaded = renderer.upload(&gpu, &model, &mut no_images);
    assert!(
        !uploaded.parts[0].batches.is_empty(),
        "the part has a batch"
    );

    let big_vertices: Vec<ObjectVertex> = (0..300).map(|i| vertex(i as f32)).collect();
    let big_indices: Vec<u32> = (0..300).collect();
    renderer.set_geometry(&gpu, &mut uploaded, &big_vertices, &big_indices);
    let (vertex_size, index_size) = (
        uploaded.parts[0].vertices.size(),
        uploaded.parts[0].indices.size(),
    );
    assert!(vertex_size >= 300 * std::mem::size_of::<super::GpuVertex>() as u64);
    assert_eq!(uploaded.parts[0].batches[0].range, 0..300);

    let small_vertices: Vec<ObjectVertex> = (0..10).map(|i| vertex(i as f32)).collect();
    let small_indices: Vec<u32> = (0..12).collect();
    renderer.set_geometry(&gpu, &mut uploaded, &small_vertices, &small_indices);
    let part = &uploaded.parts[0];
    assert_eq!(
        part.batches[0].range,
        0..12,
        "the draw follows the second call"
    );
    assert_eq!(
        part.vertices.size(),
        vertex_size,
        "no reallocation when it fits"
    );
    assert_eq!(
        part.indices.size(),
        index_size,
        "no reallocation when it fits"
    );

    // Empty geometry draws nothing and writes nothing.
    renderer.set_geometry(&gpu, &mut uploaded, &[], &[]);
    assert_eq!(uploaded.parts[0].batches[0].range, 0..0);
}
