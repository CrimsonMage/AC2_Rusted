//! The velocity the object pass writes, read back from the scene target: a posed human (skinned
//! parts) seen by a still camera, standing, moved a tenth of a metre since the previous frame,
//! deformed by a tenth of a metre since it, or with no record. Runs when `AC2_DAT_DIR` is set and a
//! GPU adapter exists.

use ac2_core::DataId;
use ac2_db::ResourceStore;
use ac2_math::{Mat4, Vec3};
use ac2_render::object::{load_object_model, ObjectEnvironment, ObjectPose, RenderSettings};
use ac2_render::{Camera, Image, SceneLighting};

use super::{DrawGroup, ObjectInstance, ObjectRenderer, Previous};
use crate::{Gpu, HdrTarget, TextureKey, ToneMap, COLOR_FORMAT, HDR_FORMAT, NO_VELOCITY};

const HUMAN_MALE_VISUAL: u32 = 0x1F00_0023;
const SIZE: (u32, u32) = (320, 400);

/// An IEEE half float as f32.
fn half(bits: u16) -> f32 {
    let (sign, exponent, fraction) = (
        bits >> 15,
        i32::from((bits >> 10) & 31),
        f32::from(bits & 1023),
    );
    let magnitude = match exponent {
        0 => fraction * 2.0_f32.powi(-24),
        31 => f32::INFINITY,
        e => (1.0 + fraction / 1024.0) * 2.0_f32.powi(e - 15),
    };
    if sign == 1 {
        -magnitude
    } else {
        magnitude
    }
}

/// The velocity target's texels (x, y), row by row.
fn read_velocity(gpu: &Gpu, hdr: &HdrTarget) -> Vec<[f32; 2]> {
    let row = SIZE.0 * 4;
    let padded =
        row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("velocity read-back"),
        size: u64::from(padded) * u64::from(SIZE.1),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = gpu.device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        hdr.velocity.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(SIZE.1),
            },
        },
        wgpu::Extent3d {
            width: SIZE.0,
            height: SIZE.1,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit([encoder.finish()]);
    buffer.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    let mapped = buffer.slice(..).get_mapped_range().expect("mapped");
    let mut texels = Vec::new();
    for y in 0..SIZE.1 as usize {
        for x in 0..SIZE.0 as usize {
            let at = y * padded as usize + x * 4;
            let component =
                |i: usize| half(u16::from_le_bytes([mapped[at + i], mapped[at + i + 1]]));
            texels.push([component(0), component(2)]);
        }
    }
    texels
}

#[test]
fn objects_write_their_screen_step_from_the_previous_placement() {
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
    let mut renderer = ObjectRenderer::new(&gpu, HDR_FORMAT);
    let mut no_images = |_key: TextureKey| -> Option<Image> { None };
    let uploaded = renderer.upload(&gpu, &model, &mut no_images);
    let pose = model.pose(None).unwrap();
    // The same pose with every deform matrix moved a tenth of a metre to the right.
    let mut shifted = pose.clone();
    for matrix in &mut shifted.palette {
        *matrix = Mat4::from_translation(Vec3::new(0.1, 0.0, 0.0)) * *matrix;
    }
    let (eye, aspect) = (Vec3::new(0.0, -4.5, 1.0), SIZE.0 as f32 / SIZE.1 as f32);
    let camera = Camera {
        eye,
        target: Vec3::new(0.0, 0.0, 0.9),
        fov_y: 0.6,
        near: 0.1,
        far: 100.0,
    };
    let lighting = SceneLighting::default();
    let tonemap = ToneMap::new(&gpu, COLOR_FORMAT);
    let hdr = HdrTarget::new(&gpu, &tonemap, SIZE.0, SIZE.1);
    hdr.set_keep_velocity(true);
    renderer.set_motion(true);
    let world = Mat4::IDENTITY;
    // The pixels a frame covers with an object's velocity, and their mean step.
    let mut frame = |previous: Previous<'_>, pose: &ObjectPose| {
        let instances = [ObjectInstance {
            pose: Some(pose),
            previous,
            ..ObjectInstance::new(&uploaded, world)
        }];
        let groups = [DrawGroup {
            instances: &instances,
            ordered: false,
            environment: None,
            sky: false,
        }];
        renderer.prepare(
            &gpu,
            &camera,
            &ObjectEnvironment::outside(&lighting),
            aspect,
            0.0,
            &groups,
        );
        let mut encoder = gpu.device.create_command_encoder(&Default::default());
        hdr.render(&gpu, &mut encoder, [0.0, 0.0, 0.0, 1.0], None, |pass| {
            renderer.draw(pass, 0, &instances);
        });
        gpu.queue.submit([encoder.finish()]);
        let texels = read_velocity(&gpu, &hdr);
        let covered: Vec<[f32; 2]> = texels.into_iter().filter(|v| v[0] < 100.0).collect();
        let mean = |axis: usize| {
            covered.iter().map(|v| v[axis]).sum::<f32>() / covered.len().max(1) as f32
        };
        (covered.len(), [mean(0), mean(1)], covered)
    };
    // The first frame has no previous camera: no velocity anywhere.
    let (none, ..) = frame(Previous::Static, &pose);
    assert_eq!(none, 0, "no previous camera, no motion vectors");
    // A standing body: its step is the camera's, none.
    let (standing, _, texels) = frame(Previous::Static, &pose);
    assert!(standing > 2000, "the body covers pixels: {standing}");
    assert!(texels
        .iter()
        .all(|v| v[0].abs() < 2e-4 && v[1].abs() < 2e-4));
    // Moved a tenth of a metre right since the previous frame (it stood at -0.1): the previous
    // position is to the left, a step of 0.1 m over the view's width at the body's 4.5 m.
    let width = 2.0 * 4.5 * (0.3_f32).tan() * aspect;
    let want = -0.1 / width;
    let before = Previous::Moved {
        world: Mat4::from_translation(Vec3::new(-0.1, 0.0, 0.0)),
        pose: Some(&pose),
    };
    let (moved, mean, _) = frame(before, &pose);
    assert!(moved > 2000);
    assert!(
        (mean[0] - want).abs() < want.abs() * 0.15,
        "{mean:?} against {want}"
    );
    assert!(mean[1].abs() < want.abs() * 0.05, "{mean:?}");
    // Deformed a tenth of a metre right since the previous frame by its bones alone: the
    // previous skinned position is 0.1 m to the right, the opposite step.
    let before = Previous::Moved {
        world,
        pose: Some(&shifted),
    };
    let (deformed, mean, _) = frame(before, &pose);
    assert!(deformed > 2000);
    assert!(
        (mean[0] + want).abs() < want.abs() * 0.15,
        "{mean:?} against {}",
        -want
    );
    // No record: the sentinel, so the resolve takes the camera's motion.
    let (unknown, ..) = frame(Previous::Unknown, &pose);
    assert_eq!(unknown, 0, "an object without a record leaves the sentinel");
    assert_eq!(NO_VELOCITY, 1000.0);
}
