use ac2_math::{Mat4, Quat, Vec3, Vec4};
use ac2_render::Camera;

use super::*;
use crate::taa::{jitter_ndc, jittered};

const ASPECT: f32 = 16.0 / 9.0;
const SIZE: (u32, u32) = (1280, 720);

fn camera(eye: [f32; 3], look: [f32; 3]) -> Camera {
    let eye = Vec3::from_array(eye);
    Camera {
        eye,
        target: eye + Vec3::from_array(look),
        fov_y: 1.0,
        near: 0.1,
        far: 10_000.0,
    }
}

fn view_projection(camera: &Camera) -> Mat4 {
    crate::sky::camera_at_origin(camera).view_projection(ASPECT)
}

/// `object.wgsl`'s vertex and fragment stages for a rigid vertex of a part: the current clip
/// position of the frame (jittered, as drawn) and the previous one, then `screen_velocity`.
/// `world` and `before` are the part's mesh-to-world matrices now and on the previous frame.
#[allow(clippy::too_many_arguments)]
fn shader_velocity(
    vertex: Vec3,
    (world, before): (Mat4, Mat4),
    (now, then): (&Camera, &Camera),
    jitter: [f32; 2],
) -> [f32; 2] {
    // `prepare`: the part's origin off the eye, in f32 for the current frame (as laid out).
    let mut world_rel = world;
    world_rel.w_axis -= now.eye.extend(0.0);
    let rel = world_rel.w_axis;
    let jittered_vp = jittered(view_projection(now), jitter);
    let rel_position = world.transform_vector3(vertex) + rel.truncate();
    let clip = jittered_vp * rel_position.extend(1.0);
    let cur = [
        clip.x - jitter[0] * clip.w,
        clip.y - jitter[1] * clip.w,
        clip.w,
    ];
    let previous = part_previous(&before, None, 0, Mat4::IDENTITY, then.eye);
    let before_position = before.transform_vector3(vertex) + previous.rel.truncate();
    let prev = view_projection(then) * before_position.extend(1.0);
    let d = [
        prev.x / prev.w - cur[0] / cur[2],
        prev.y / prev.w - cur[1] / cur[2],
    ];
    [d[0] * 0.5, -d[1] * 0.5]
}

/// `taa.wgsl`'s `motion` for a pixel at `ndc` and `depth` of the current frame: previous minus
/// current screen position, uv units.
fn camera_velocity(now: &Camera, then: &Camera, ndc: [f32; 2], depth: f32) -> [f32; 2] {
    let h = view_projection(now).inverse() * Vec4::new(ndc[0], ndc[1], depth, 1.0);
    let delta = (then.eye.as_dvec3() - now.eye.as_dvec3()).as_vec3();
    let clip = view_projection(then) * (h.truncate() / h.w - delta).extend(1.0);
    [
        (clip.x / clip.w - ndc[0]) * 0.5,
        -(clip.y / clip.w - ndc[1]) * 0.5,
    ]
}

fn pixels(uv: [f32; 2]) -> [f32; 2] {
    [uv[0] * SIZE.0 as f32, uv[1] * SIZE.1 as f32]
}

#[test]
fn a_static_object_moves_on_screen_as_the_camera_reprojection_says() {
    // Thirty kilometres out, a step of the eye and a turn of the view, with the frame's jitter.
    let then = camera([30_123.456, 28_765.432, 120.5], [0.2, 1.0, -0.02]);
    let now = camera([30_123.9, 28_766.1, 120.55], [0.26, 1.0, -0.03]);
    let jitter = jitter_ndc([0.25, -0.375], SIZE);
    let world = Mat4::from_rotation_translation(
        Quat::from_rotation_z(0.7),
        Vec3::new(30_140.0, 28_790.0, 121.0),
    );
    for vertex in [
        Vec3::new(0.3, -0.2, 1.1),
        Vec3::new(-0.8, 0.5, 0.0),
        Vec3::new(2.0, 1.0, 1.9),
    ] {
        let velocity = shader_velocity(vertex, (world, world), (&now, &then), jitter);
        // The pixel the vertex covers, unjittered.
        let mut rel = world;
        rel.w_axis -= now.eye.extend(0.0);
        let clip = view_projection(&now)
            * (world.transform_vector3(vertex) + rel.w_axis.truncate()).extend(1.0);
        let ndc = [clip.x / clip.w, clip.y / clip.w];
        let want = camera_velocity(&now, &then, ndc, clip.z / clip.w);
        let (got, want) = (pixels(velocity), pixels(want));
        assert!(
            (got[0] - want[0]).abs() < 0.01 && (got[1] - want[1]).abs() < 0.01,
            "{vertex:?}: {got:?} against {want:?}"
        );
    }
}

#[test]
fn a_body_moved_a_tenth_of_a_metre_shows_that_motion_on_screen() {
    // The camera stands still, 30 km out; the body steps 0.1 m to the right of the view.
    let eye = camera([30_000.0, 28_000.0, 100.0], [0.0, 1.0, 0.0]);
    let before = Mat4::from_translation(Vec3::new(30_000.0, 28_020.0, 100.0));
    let after = Mat4::from_translation(Vec3::new(30_000.1, 28_020.0, 100.0));
    let velocity = pixels(shader_velocity(
        Vec3::ZERO,
        (after, before),
        (&eye, &eye),
        [0.0; 2],
    ));
    // A point 20 m ahead moving 0.1 m sideways: half the width is 20 * tan(fov / 2) * aspect m.
    let half_width_metres = 20.0 * (0.5_f64).tan() * f64::from(ASPECT);
    // The matrices hold f32 (2 mm apart out here), so the step is what they hold.
    let moved = f64::from(after.w_axis.x) - f64::from(before.w_axis.x);
    assert!((moved - 0.1).abs() < 0.003);
    let step = moved / half_width_metres * f64::from(SIZE.0) / 2.0;
    // It was `step` pixels to the left of where it is: the previous minus the current position.
    assert!(
        (f64::from(velocity[0]) + step).abs() < 0.01,
        "{velocity:?} against {step}"
    );
    assert!(velocity[1].abs() < 0.01, "{velocity:?}");
}

#[test]
fn no_record_a_teleport_or_another_rig_leaves_the_object_without_motion() {
    let at = |x: f32| Mat4::from_translation(Vec3::new(x, 0.0, 0.0));
    let pose = |parts: usize, palette: usize| ObjectPose {
        parts: vec![Mat4::IDENTITY; parts],
        palette: vec![Mat4::IDENTITY; palette],
        ..Default::default()
    };
    let (two, three) = (pose(2, 4), pose(3, 4));
    // Scenery keeps its place; nothing known gives nothing.
    assert!(previous_placement(Previous::Static, at(5.0), Some(&two)).is_some());
    assert!(previous_placement(Previous::Unknown, at(5.0), Some(&two)).is_none());
    let moved = |world, pose| Previous::Moved { world, pose };
    // A step of a metre is motion; a jump past the teleport distance is not.
    let step = previous_placement(moved(at(4.0), Some(&two)), at(5.0), Some(&two));
    assert_eq!(step.map(|(world, _)| world), Some(at(4.0)));
    assert!(previous_placement(moved(at(-100.0), Some(&two)), at(5.0), Some(&two)).is_none());
    // A different rig (another model on the same key) has no usable previous pose.
    assert!(previous_placement(moved(at(4.0), Some(&three)), at(5.0), Some(&two)).is_none());
    assert!(previous_placement(moved(at(4.0), None), at(5.0), Some(&two)).is_none());
    assert!(previous_placement(moved(at(4.0), None), at(5.0), None).is_some());
}
