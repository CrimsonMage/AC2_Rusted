//! A decal's triangles uploaded as the particle geometry of an object model: the decals of one
//! material fill the draw range (`DecalEffectGroup::Render`), and an empty set draws nothing.
//! Runs when `AC2_DAT_DIR` is set (for a real model's part) and a GPU adapter exists.

use ac2_core::DataId;
use ac2_db::ResourceStore;
use ac2_math::Vec3;
use ac2_render::decal::{DecalEffects, DecalSpec, GroundVertex};
use ac2_render::object::{load_object_model, RenderSettings};
use ac2_render::Image;

use super::ObjectRenderer;
use crate::{Gpu, TextureKey, HDR_FORMAT};

const HUMAN_MALE_VISUAL: u32 = 0x1F00_0023;
const MATERIAL: u32 = 0x3100_0001;

fn ground() -> [[GroundVertex; 3]; 2] {
    let v = |x: f32, y: f32| GroundVertex {
        position: [x, y, 0.0],
        normal: [0.0, 0.0, 1.0],
    };
    [
        [v(-2.0, -2.0), v(2.0, -2.0), v(-2.0, 2.0)],
        [v(2.0, -2.0), v(2.0, 2.0), v(-2.0, 2.0)],
    ]
}

#[test]
fn decal_triangles_fill_the_particle_draw_range() {
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

    let spec = DecalSpec {
        material: DataId::new(MATERIAL),
        lifespan: 10.0,
        size: 2.0,
        direction: Vec3::new(1.0, 0.0, 0.0),
        color: [1.0, 1.0, 1.0, 1.0],
        draw_once: false,
        origin: Vec3::ZERO,
    };
    let mut decals = DecalEffects::default();
    decals.create(spec, 0.0, &ground());
    decals.think(0.5);
    let geometry = decals.geometry();
    let (_, decal) = geometry.first().expect("the decal draws one material");
    assert!(!decal.indices.is_empty(), "the decal has triangles");

    renderer.set_geometry(&gpu, &mut uploaded, &decal.vertices, &decal.indices);
    assert_eq!(
        uploaded.parts[0].batches[0].range,
        0..decal.indices.len() as u32,
        "the decal's triangles are the draw range"
    );

    renderer.set_geometry(&gpu, &mut uploaded, &[], &[]);
    assert_eq!(uploaded.parts[0].batches[0].range, 0..0);
}
