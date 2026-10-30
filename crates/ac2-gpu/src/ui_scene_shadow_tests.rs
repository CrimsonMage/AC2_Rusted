//! A UI viewport's scene casts its objects' shadows (`UIScene::Render`'s volumetric shadows):
//! two bodies in a lit scene seen through an orthographic camera 10 km back, the one between the
//! sun and the other darkening it, and nothing darkened below `DynamicObjectShadows` 2. Runs when
//! `AC2_DAT_DIR` is set and a GPU adapter exists.

use ac2_core::DataId;
use ac2_db::ResourceStore;
use ac2_math::{Mat4, Vec3};
use ac2_render::object::{load_object_model, RenderSettings};
use ac2_render::prefs::ObjectShadows;
use ac2_render::scene::{Loaded, LoadedObject, SceneFrame, SceneObject, SceneUpdate};
use ac2_render::{load_material_image, load_texture_image, Camera, SceneLighting, TextureKey};

use crate::{FrameTarget, Gpu, SceneRenderer};

const HUMAN_MALE_VISUAL: u32 = 0x1F00_0023;
const SIZE: (u32, u32) = (400, 500);
const BACKGROUND: [f32; 3] = [1.0, 0.0, 1.0];

#[test]
fn a_viewport_scene_casts_its_objects_shadows() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let Ok(gpu) = Gpu::headless() else {
        eprintln!("no GPU adapter; skipping");
        return;
    };
    let store = ResourceStore::new(dataset);
    let model = load_object_model(
        &store,
        DataId::new(HUMAN_MALE_VISUAL),
        &RenderSettings::default(),
    )
    .unwrap()
    .expect("human draws");
    let images = model
        .texture_keys()
        .into_iter()
        .map(|key| {
            let image = match key {
                TextureKey::Material(did) => load_material_image(&store, did).ok().flatten(),
                TextureKey::Mask(did) | TextureKey::Texture(did) => {
                    load_texture_image(&store, did).ok().flatten()
                }
            };
            (key, image)
        })
        .collect();
    let mut renderer = SceneRenderer::new(&gpu);
    renderer.set_viewport_scene();
    renderer.apply(
        &gpu,
        SceneUpdate {
            loaded: vec![Loaded::Object(LoadedObject {
                key: 1,
                model: Some(model),
                pick: None,
                scale: Vec3::ONE,
                images,
            })],
            evicted: Vec::new(),
        },
    );
    let target = FrameTarget::new(&gpu.device, SIZE.0, SIZE.1);
    let aspect = SIZE.0 as f32 / SIZE.1 as f32;
    // The default sun lights it from +x, low; the second body stands between it and the first.
    let lighting = SceneLighting {
        light_dir: Vec3::new(-1.0, 0.0, -0.6).normalize() * 0.5,
        dir_brightness: 0.5,
        dir_color: [1.0; 3],
        ambient: [0.3; 3],
        shadow_strength: [0.75; 3],
        fog_range: (1.0e6, 2.0e6),
        ..SceneLighting::default()
    };
    // An orthographic camera's view is taken from 10 km back, as `scene_camera` does.
    let camera = Camera {
        eye: Vec3::new(0.3, -10_000.0, 1.0),
        target: Vec3::new(0.3, 0.0, 1.0),
        fov_y: 2.0 * (1.4_f32 / 2.0 / 10_000.0).atan(),
        near: 9_990.0,
        far: 10_020.0,
    };
    let mut render = |level: ObjectShadows| {
        renderer.set_object_shadows(level);
        let objects = [Vec3::ZERO, Vec3::new(0.9, 0.0, 0.0)]
            .map(|at| SceneObject {
                key: 1,
                world: Mat4::from_translation(at),
                pose: None,
                tint: [1.0; 4],
                point_lights: Vec::new(),
                detail: Default::default(),
            })
            .to_vec();
        let frame = SceneFrame {
            lighting,
            ..SceneFrame::default()
        };
        renderer.prepare(&gpu, &camera, aspect, 0.0, frame, objects);
        target.render_scene(&gpu, BACKGROUND, |pass| renderer.draw(pass));
        target.read_rgba(&gpu.device, &gpu.queue)
    };
    let off = render(ObjectShadows::Off);
    let low = render(ObjectShadows::Low);
    let medium = render(ObjectShadows::Medium);
    let differ = |a: &[u8], b: &[u8]| {
        a.chunks_exact(4)
            .zip(b.chunks_exact(4))
            .filter(|(a, b)| a[..3].iter().zip(&b[..3]).any(|(x, y)| x.abs_diff(*y) > 6))
            .count()
    };
    let covered = off
        .chunks_exact(4)
        .filter(|px| {
            px[..3]
                .iter()
                .zip(&off[..3])
                .any(|(a, b)| a.abs_diff(*b) > 2)
        })
        .count();
    let shadowed = differ(&off, &medium);
    eprintln!("{shadowed} pixels shadowed of {covered} covered");
    assert!(covered > 5000, "the bodies cover pixels: {covered}");
    assert_eq!(
        differ(&off, &low),
        0,
        "retail drew no volumes below level 2"
    );
    assert!(
        shadowed > 500,
        "the body in front shadows the other: {shadowed}"
    );
}
