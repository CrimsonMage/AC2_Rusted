//! Headless snapshot of the human male visual in its bind pose and posed by the stance clip,
//! drawn through the GPU blend (`BlendLitVertices` in the vertex program). Each silhouette's
//! horizontal extent must match the CPU reference blend projected through the same camera. Runs
//! when `AC2_DAT_DIR` is set and a GPU adapter exists; writes PNGs to `target/ac2-snapshots/`.

use std::path::PathBuf;

use ac2_anim::Ac2Setup;
use ac2_core::DataId;
use ac2_dat::DatFileKind;
use ac2_db::{ResourceStore, TypedResourceRef};
use ac2_gpu::{
    DrawGroup, FrameTarget, Gpu, ObjectInstance, ObjectRenderer, TextureKey, COLOR_FORMAT,
};
use ac2_math::{Mat4, Vec3};
use ac2_render::object::{
    blend_vertex, load_object_model, load_object_model_with, ObjectEnvironment, ObjectModel,
    ObjectPose, RenderSettings,
};
use ac2_render::{load_material_image, load_texture_image, Camera, SceneLighting};

const SIZE: (u32, u32) = (400, 500);
const HUMAN_MALE_VISUAL: u32 = 0x1F00_0023;
/// The human `AnimMap` and its stance clip (as `ac2-assets`' posed-model test uses).
const HUMAN_ANIM_MAP: u32 = 0x0300_0014;
const STANCE: u32 = 0x8000_0012;
const BACKGROUND: [f32; 3] = [1.0, 0.0, 1.0];
/// An appearance table whose key 1 ramps the skin material (`m_materialType` 0x81000001) through
/// modifiers by scalar.
const SKIN_TABLE: u32 = 0x2000_0000;
const SKIN_KEY: u32 = 1;

fn write_png(name: &str, rgba: &[u8]) {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/ac2-snapshots");
    std::fs::create_dir_all(&dir).expect("snapshot directory");
    let file = std::fs::File::create(dir.join(name)).expect("png file");
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), SIZE.0, SIZE.1);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .unwrap()
        .write_image_data(rgba)
        .unwrap();
}

/// Leftmost and rightmost pixel columns that are not the background.
fn silhouette_columns(rgba: &[u8]) -> Option<(u32, u32)> {
    let mut columns = None::<(u32, u32)>;
    for (index, px) in rgba.chunks_exact(4).enumerate() {
        let background = px[0] > 250 && px[1] < 5 && px[2] > 250;
        if !background {
            let x = index as u32 % SIZE.0;
            columns = Some(columns.map_or((x, x), |(l, r)| (l.min(x), r.max(x))));
        }
    }
    columns
}

/// The CPU reference: every drawn vertex blended and placed as the GPU does, projected to pixel
/// columns.
fn expected_columns(model: &ObjectModel, pose: &ObjectPose, view_proj: Mat4) -> (f32, f32) {
    let (mut left, mut right) = (f32::MAX, f32::MIN);
    for (index, part) in model.parts.iter().enumerate() {
        let skin = pose.skins[index].filter(|_| part.skin.is_some());
        for batch in &part.batches {
            for &vertex in batch.indices.iter() {
                let v = &part.vertices[vertex as usize];
                let local = match (skin, &part.skin) {
                    (Some(first), Some(influences)) => {
                        blend_vertex(
                            v.position,
                            v.normal,
                            &influences[vertex as usize],
                            &pose.palette[first as usize..],
                        )
                        .0
                    }
                    _ => v.position,
                };
                let clip = view_proj
                    * pose.parts[index]
                        .transform_point3(Vec3::from(local))
                        .extend(1.0);
                let x = (clip.x / clip.w * 0.5 + 0.5) * SIZE.0 as f32;
                left = left.min(x);
                right = right.max(x);
            }
        }
    }
    (left, right)
}

#[test]
fn human_draws_posed_through_the_gpu_blend() {
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
    let setup_did = model.skeleton.as_ref().unwrap().groups[0].setup;
    let setup = store
        .load_cached_packed::<Ac2Setup>(TypedResourceRef::new(DatFileKind::Portal, setup_did, 0))
        .unwrap()
        .unwrap();
    let animation =
        ac2_anim::load_ac2_motion_animation(&store, DataId::new(HUMAN_ANIM_MAP), STANCE)
            .unwrap()
            .expect("stance clip");
    let stance = animation.sample(0.0, &setup.rest_locals());

    let mut load = |key: TextureKey| match key {
        TextureKey::Material(did) => load_material_image(&store, did).ok().flatten(),
        TextureKey::Mask(did) | TextureKey::Texture(did) => {
            load_texture_image(&store, did).ok().flatten()
        }
    };
    let mut objects = ObjectRenderer::new(&gpu, COLOR_FORMAT);
    let uploaded = objects.upload(&gpu, &model, &mut load);
    let target = FrameTarget::new(&gpu.device, SIZE.0, SIZE.1);
    let aspect = SIZE.0 as f32 / SIZE.1 as f32;
    let camera = Camera {
        eye: Vec3::new(0.0, -4.5, 1.0),
        target: Vec3::new(0.0, 0.0, 0.9),
        fov_y: 0.6,
        near: 0.1,
        far: 100.0,
    };
    let lighting = SceneLighting::default();
    let mut widths = Vec::new();
    for (name, root) in [
        ("character_human_bind.png", None),
        ("character_human_stance.png", Some(&stance[..])),
    ] {
        let pose = model.pose(root).unwrap();
        let instances = [ObjectInstance {
            pose: Some(&pose),
            ..ObjectInstance::new(&uploaded, Mat4::IDENTITY)
        }];
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
            }],
        );
        target.render(&gpu.device, &gpu.queue, BACKGROUND, |pass| {
            objects.draw(pass, 0, &instances);
        });
        let rgba = target.read_rgba(&gpu.device, &gpu.queue);
        write_png(name, &rgba);
        let (left, right) = silhouette_columns(&rgba).expect("the body covers pixels");
        let (want_left, want_right) =
            expected_columns(&model, &pose, camera.view_projection(aspect));
        eprintln!("{name}: columns {left}..{right}, CPU blend {want_left:.1}..{want_right:.1}");
        assert!(
            (left as f32 - want_left).abs() < 3.0 && (right as f32 - want_right).abs() < 3.0,
            "{name}: GPU silhouette {left}..{right} vs CPU {want_left:.1}..{want_right:.1}"
        );
        widths.push(right - left);
    }
    assert!(
        (widths[1] as f32) < widths[0] as f32 * 0.8,
        "the stance lowers the T-pose arms: {widths:?}"
    );
}

/// Mean colour of the non-background pixels.
fn body_colour(rgba: &[u8]) -> [f32; 3] {
    let (mut sum, mut n) = ([0.0_f32; 3], 0.0_f32);
    for px in rgba.chunks_exact(4) {
        if !(px[0] > 250 && px[1] < 5 && px[2] > 250) {
            for c in 0..3 {
                sum[c] += f32::from(px[c]);
            }
            n += 1.0;
        }
    }
    sum.map(|c| c / n.max(1.0))
}

#[test]
fn human_skin_takes_the_appearance_tables_modifiers() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let Ok(gpu) = Gpu::headless() else {
        eprintln!("no GPU adapter; skipping");
        return;
    };
    let store = ResourceStore::new(dataset);
    let settings = RenderSettings::default();
    // The object's local description: the human visual as parent, the skin key applied
    // globally.
    let local = ac2_assets::Ac2VisualDesc {
        parent: Some(DataId::new(HUMAN_MALE_VISUAL)),
        motion_interp: None,
        behavior_table: None,
        modes: None,
        scale: None,
        child_scale: None,
        particle_scale: None,
        part_groups: std::sync::Arc::from([]),
        part_group_appearance: std::sync::Arc::from([]),
        global_appearance: std::sync::Arc::from([(
            DataId::new(SKIN_TABLE),
            std::sync::Arc::from([(SKIN_KEY, 0.5)]),
        )]),
        fx: ac2_assets::Ac2VisualFx::default(),
    };
    let bare = load_object_model(&store, DataId::new(HUMAN_MALE_VISUAL), &settings)
        .unwrap()
        .unwrap();
    let skinned = load_object_model_with(
        &store,
        DataId::new(HUMAN_MALE_VISUAL),
        Some(&local),
        &settings,
    )
    .unwrap()
    .unwrap();
    let mut load = |key: TextureKey| match key {
        TextureKey::Material(did) => load_material_image(&store, did).ok().flatten(),
        TextureKey::Mask(did) | TextureKey::Texture(did) => {
            load_texture_image(&store, did).ok().flatten()
        }
    };
    let mut objects = ObjectRenderer::new(&gpu, COLOR_FORMAT);
    let target = FrameTarget::new(&gpu.device, SIZE.0, SIZE.1);
    let aspect = SIZE.0 as f32 / SIZE.1 as f32;
    let camera = Camera {
        eye: Vec3::new(0.0, -4.5, 1.0),
        target: Vec3::new(0.0, 0.0, 0.9),
        fov_y: 0.6,
        near: 0.1,
        far: 100.0,
    };
    let mut colours = Vec::new();
    for (name, model) in [
        ("character_human_bare.png", &bare),
        ("character_human_skin.png", &skinned),
    ] {
        let uploaded = objects.upload(&gpu, model, &mut load);
        let instances = [ObjectInstance::new(&uploaded, Mat4::IDENTITY)];
        objects.prepare(
            &gpu,
            &camera,
            &ObjectEnvironment::outside(&SceneLighting::default()),
            aspect,
            0.0,
            &[DrawGroup {
                instances: &instances,
                ordered: false,
                environment: None,
            }],
        );
        target.render(&gpu.device, &gpu.queue, BACKGROUND, |pass| {
            objects.draw(pass, 0, &instances);
        });
        let rgba = target.read_rgba(&gpu.device, &gpu.queue);
        write_png(name, &rgba);
        let colour = body_colour(&rgba);
        eprintln!("{name}: body colour {colour:.1?}");
        colours.push(colour);
    }
    let differs = (0..3).any(|c| (colours[0][c] - colours[1][c]).abs() > 8.0);
    assert!(differs, "the skin modifiers change the body: {colours:?}");
}
