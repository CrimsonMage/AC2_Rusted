//! The sun shadow cascades tens of kilometres from the world's origin: a body shadowing another
//! draws the same shadow there as near the origin. With the casters and receivers placed from
//! absolute coordinates in f32, a point rounds to millimetres in the cascade, so a body's shadow
//! edges stepped as it moved (`Cascade::clip_rel`). Runs when `AC2_DAT_DIR` is set and a GPU
//! adapter exists.

use ac2_core::DataId;
use ac2_db::ResourceStore;
use ac2_math::{Mat4, Vec3};
use ac2_render::object::{load_object_model, ObjectEnvironment, RenderSettings};
use ac2_render::prefs::ObjectShadows;
use ac2_render::{load_material_image, load_texture_image, Camera, SceneLighting};

use crate::sun_shadow::{fit_cascades, SunShadowTier, SunShadows};
use crate::{DrawGroup, FrameTarget, Gpu, ObjectInstance, ObjectRenderer, TextureKey, HDR_FORMAT};

const HUMAN_MALE_VISUAL: u32 = 0x1F00_0023;
const SIZE: (u32, u32) = (400, 500);
const BACKGROUND: [f32; 3] = [1.0, 0.0, 1.0];

#[test]
fn a_body_shadows_another_alike_far_from_the_worlds_origin() {
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
    let mut load = |key: TextureKey| match key {
        TextureKey::Material(did) => load_material_image(&store, did).ok().flatten(),
        TextureKey::Mask(did) | TextureKey::Texture(did) => {
            load_texture_image(&store, did).ok().flatten()
        }
    };
    let mut objects = ObjectRenderer::new(&gpu, HDR_FORMAT);
    let uploaded = objects.upload(&gpu, &model, &mut load);
    let mut shadows = SunShadows::new(&gpu);
    shadows.set_level(&gpu, ObjectShadows::High);
    objects.set_sun_shadow(&gpu, shadows.binding());
    let tier = SunShadowTier::of(ObjectShadows::High).unwrap();
    let target = FrameTarget::new(&gpu.device, SIZE.0, SIZE.1);
    let aspect = SIZE.0 as f32 / SIZE.1 as f32;
    // A low sun from +x; the second body stands between it and the first.
    let lighting = SceneLighting {
        light_dir: Vec3::new(-1.0, 0.0, -0.6).normalize() * 0.8,
        dir_brightness: 0.8,
        ambient: [0.3; 3],
        shadow_strength: [0.7; 3],
        ..SceneLighting::default()
    };
    let camera_at = |offset: Vec3| Camera {
        eye: Vec3::new(0.3, -4.5, 1.2) + offset,
        target: Vec3::new(0.3, 0.0, 0.9) + offset,
        fov_y: 0.7,
        near: 0.1,
        far: 100.0,
    };
    // The cascades snap their windows to whole texels of the world, so the far scene lies on the
    // same texel grid only where the offset is whole texels along the light's axes: with the sun
    // in the x-z plane those are -y and a tilted x-z axis, so the offset runs along y alone, a
    // multiple of every cascade's texel (2 r / size: twice the radii's least common multiple).
    let radii = fit_cascades(&camera_at(Vec3::ZERO), aspect, lighting.light_dir, &tier)
        .unwrap()
        .iter()
        .map(|c| c.radius as u64)
        .collect::<Vec<_>>();
    let gcd = |mut a: u64, mut b: u64| {
        while b != 0 {
            (a, b) = (b, a % b);
        }
        a
    };
    let step = 2 * radii.iter().fold(1, |l, &r| l / gcd(l, r) * r);
    assert!(step < 1 << 20, "radii {radii:?}");
    let distance = step * 16_000_u64.div_ceil(step);
    let mut render = |offset: Vec3, shadowed: bool| {
        let camera = camera_at(offset);
        let instances = [
            ObjectInstance::new(&uploaded, Mat4::from_translation(offset)),
            ObjectInstance::new(
                &uploaded,
                Mat4::from_translation(offset + Vec3::new(0.9, 0.0, 0.0)),
            ),
        ];
        let cascades = fit_cascades(&camera, aspect, lighting.light_dir, &tier)
            .filter(|_| shadowed)
            .unwrap_or_default();
        let mut encoder = gpu.device.create_command_encoder(&Default::default());
        objects.draw_sun_casters(
            &gpu,
            &mut encoder,
            shadows.layer_views(),
            (camera.eye, &cascades),
            &instances,
        );
        gpu.queue.submit([encoder.finish()]);
        shadows.write(&gpu, camera.eye, &cascades, lighting.shadow_strength);
        objects.prepare(
            &gpu,
            &camera,
            &ObjectEnvironment::outside(&lighting),
            aspect,
            0.0,
            &[DrawGroup {
                instances: &instances,
                ordered: false,
                environment: None,
                sky: false,
            }],
        );
        target.render_scene(&gpu, BACKGROUND, |pass| {
            objects.draw(pass, 0, &instances);
        });
        target.read_rgba(&gpu.device, &gpu.queue)
    };
    let unshadowed = render(Vec3::ZERO, false);
    let near = render(Vec3::ZERO, true);
    // A whole number of metres keeps the bodies exact in f32.
    let far = render(Vec3::new(0.0, distance as f32, 0.0), true);
    let differ = |a: &[u8], b: &[u8]| {
        a.chunks_exact(4)
            .zip(b.chunks_exact(4))
            .filter(|(a, b)| a[..3].iter().zip(&b[..3]).any(|(x, y)| x.abs_diff(*y) > 6))
            .count()
    };
    let covered = near
        .chunks_exact(4)
        .filter(|px| {
            px[..3]
                .iter()
                .zip(&near[..3])
                .any(|(a, b)| a.abs_diff(*b) > 2)
        })
        .count();
    let shadowed = differ(&unshadowed, &near);
    let moved = differ(&near, &far);
    eprintln!(
        "{shadowed} pixels shadowed, {moved} of {covered} differ {distance} m from the origin"
    );
    assert!(covered > 5000, "the bodies cover pixels: {covered}");
    assert!(
        shadowed > 500,
        "the second body shadows the first: {shadowed}"
    );
    assert!(
        moved * 500 < covered,
        "{moved} of {covered} pixels changed {distance} m out"
    );
}
