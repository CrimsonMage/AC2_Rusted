use ac2_math::{Mat4, Vec3, Vec4};
use ac2_render::Camera;

use super::{halton, is_cut, jitter_ndc, jitter_pixels, jittered, TaaFrame, JITTER_PHASES};

const ASPECT: f32 = 16.0 / 9.0;
const SIZE: (u32, u32) = (1280, 720);

fn camera(eye: [f32; 3], look: [f32; 3], fov_y: f32) -> Camera {
    let eye = Vec3::from_array(eye);
    Camera {
        eye,
        target: eye + Vec3::from_array(look),
        fov_y,
        near: 0.1,
        far: 10_000.0,
    }
}

#[test]
fn the_jitter_is_the_halton_two_three_sequence_centred_on_the_pixel() {
    let base2: Vec<f32> = (1..=8).map(|i| halton(i, 2)).collect();
    assert_eq!(base2, [0.5, 0.25, 0.75, 0.125, 0.625, 0.375, 0.875, 0.0625]);
    let base3: Vec<f32> = (1..=8).map(|i| halton(i, 3)).collect();
    let third = 1.0 / 3.0;
    for (got, want) in base3.iter().zip([
        third,
        2.0 * third,
        1.0 / 9.0,
        4.0 / 9.0,
        7.0 / 9.0,
        2.0 / 9.0,
        5.0 / 9.0,
        8.0 / 9.0,
    ]) {
        assert!((got - want).abs() < 1e-6, "{got} {want}");
    }
    assert_eq!(jitter_pixels(0), [0.0, third - 0.5]);
    for frame in 0..64 {
        let [x, y] = jitter_pixels(frame);
        assert!((-0.5..0.5).contains(&x) && (-0.5..0.5).contains(&y));
        assert_eq!(jitter_pixels(frame), jitter_pixels(frame + JITTER_PHASES));
    }
    // Eight distinct phases.
    let mut phases: Vec<[u32; 2]> = (0..8)
        .map(|f| jitter_pixels(f).map(|v| (v * 1e4) as i32 as u32))
        .collect();
    phases.sort_unstable();
    phases.dedup();
    assert_eq!(phases.len(), 8);
}

#[test]
fn the_jitter_helper_moves_a_point_by_exactly_the_requested_pixels() {
    let camera = camera([100.0, 200.0, 5.0], [0.3, 1.0, 0.1], 1.0);
    let view_projection = crate::sky::camera_at_origin(&camera).view_projection(ASPECT);
    let point = Vec4::new(2.0, 30.0, 4.0, 1.0);
    let screen = |m: Mat4| {
        let clip = m * point;
        [
            (clip.x / clip.w * 0.5 + 0.5) * SIZE.0 as f32,
            (0.5 - clip.y / clip.w * 0.5) * SIZE.1 as f32,
        ]
    };
    let before = screen(view_projection);
    for pixels in [[0.25, -0.25], [-0.5, 0.375], [0.4375, 0.0625]] {
        let after = screen(jittered(view_projection, jitter_ndc(pixels, SIZE)));
        assert!(
            (after[0] - before[0] - pixels[0]).abs() < 2e-3,
            "{pixels:?}"
        );
        assert!(
            (after[1] - before[1] - pixels[1]).abs() < 2e-3,
            "{pixels:?}"
        );
    }
    // The depth is untouched, and no jitter returns the matrix itself.
    let moved = jittered(view_projection, jitter_ndc([0.3, 0.3], SIZE)) * point;
    let rest = view_projection * point;
    assert_eq!((moved.z, moved.w), (rest.z, rest.w));
    assert_eq!(jittered(view_projection, [0.0; 2]), view_projection);
}

/// `taa.wgsl`'s `motion` for a geometry pixel: where the pixel at `ndc` and `depth` of the
/// current frame was on the previous frame's screen (NDC).
fn reproject(current: &TaaFrame, previous: &TaaFrame, ndc: [f32; 2], depth: f32) -> [f32; 2] {
    let h = current.view_projection.inverse() * Vec4::new(ndc[0], ndc[1], depth, 1.0);
    let delta = Vec3::from_array(previous.eye_delta(current));
    let clip = previous.view_projection * (h.truncate() / h.w - delta).extend(1.0);
    [clip.x / clip.w, clip.y / clip.w]
}

/// The NDC of the absolute point `at` seen from `camera`, in f64 from the f32 eye, target and
/// field of view: the angular projection, independent of the matrices.
fn exact_ndc(camera: &Camera, at: [f64; 3]) -> [f64; 2] {
    let v = |a: Vec3| a.to_array().map(f64::from);
    let (eye, target) = (v(camera.eye), v(camera.target));
    let norm = |a: [f64; 3]| {
        let l = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
        a.map(|c| c / l)
    };
    let cross = |a: [f64; 3], b: [f64; 3]| {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    };
    let dot = |a: [f64; 3], b: [f64; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let forward = norm([target[0] - eye[0], target[1] - eye[1], target[2] - eye[2]]);
    let right = norm(cross(forward, [0.0, 0.0, 1.0]));
    let up = cross(right, forward);
    let rel = [at[0] - eye[0], at[1] - eye[1], at[2] - eye[2]];
    let tan = (f64::from(camera.fov_y) * 0.5).tan();
    let depth = dot(rel, forward);
    [
        dot(rel, right) / (depth * tan * f64::from(ASPECT)),
        dot(rel, up) / (depth * tan),
    ]
}

#[test]
fn reprojection_at_thirty_kilometres_lands_within_a_hundredth_of_a_pixel() {
    // A step of the eye and a turn of the view, far from the world's origin.
    let before = camera([30_123.456, 28_765.432, 120.5], [0.2, 1.0, -0.02], 1.0);
    let after = camera([30_123.9, 28_766.1, 120.55], [0.26, 1.0, -0.03], 1.0);
    let (previous, current) = (
        TaaFrame::new(&before, ASPECT),
        TaaFrame::new(&after, ASPECT),
    );
    for point in [
        [30_140.0, 28_790.0, 121.0],
        [30_135.5, 28_800.25, 118.0],
        [30_100.0, 28_790.0, 125.0],
        [30_153.0, 28_815.0, 119.0],
    ] {
        let eye = after.eye.to_array().map(f64::from);
        let relative = Vec3::new(
            (point[0] - eye[0]) as f32,
            (point[1] - eye[1]) as f32,
            (point[2] - eye[2]) as f32,
        );
        let clip = current.view_projection * relative.extend(1.0);
        let ndc = [clip.x / clip.w, clip.y / clip.w];
        let found = reproject(&current, &previous, ndc, clip.z / clip.w);
        let want = exact_ndc(&before, point);
        let pixels = [
            (f64::from(found[0]) - want[0]) * f64::from(SIZE.0) / 2.0,
            (f64::from(found[1]) - want[1]) * f64::from(SIZE.1) / 2.0,
        ];
        assert!(
            pixels[0].abs() < 0.01 && pixels[1].abs() < 0.01,
            "{point:?} off by {pixels:?} px"
        );
        // The current frame's own place agrees with the exact projection too.
        let own = exact_ndc(&after, point);
        assert!((f64::from(ndc[0]) - own[0]).abs() * f64::from(SIZE.0) / 2.0 < 0.01);
    }
}

#[test]
fn a_camera_cut_is_an_eye_step_over_five_metres_or_a_field_of_view_change() {
    let at = |x: f32, fov: f32| {
        TaaFrame::new(
            &camera([30_000.0 + x, 28_000.0, 100.0], [0.0, 1.0, 0.0], fov),
            ASPECT,
        )
    };
    let base = at(0.0, 1.0);
    assert!(!is_cut(&base, &at(0.0, 1.0)));
    assert!(!is_cut(&base, &at(4.9, 1.0)), "a fast run is not a cut");
    assert!(is_cut(&base, &at(5.5, 1.0)), "a teleport is");
    assert!(is_cut(&at(5.5, 1.0), &base), "in either direction");
    assert!(is_cut(&base, &at(0.0, 1.1)), "a zoom is");
}
