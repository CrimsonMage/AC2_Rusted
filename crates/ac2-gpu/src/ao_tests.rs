use super::tier;
use std::f32::consts::{FRAC_PI_2, PI};

// A CPU mirror of `ao.wgsl`'s integral: `slice_arc`, `tangent_cos` and `multi_bounce`.

fn tangent_cos(n: f32, side: f32) -> f32 {
    (n + side * FRAC_PI_2).cos()
}

fn slice_arc(n: f32, h1: f32, h2: f32) -> f32 {
    let (c, s) = (n.cos(), n.sin());
    0.25 * (-(2.0 * h1 - n).cos() + c + 2.0 * h1 * s)
        + 0.25 * (-(2.0 * h2 - n).cos() + c + 2.0 * h2 * s)
}

fn multi_bounce(visibility: f32, albedo: f32) -> f32 {
    let a = 2.0404 * albedo - 0.3324;
    let b = -4.7951 * albedo + 0.6417;
    let c = 2.7552 * albedo + 0.6903;
    visibility.max(((visibility * a + b) * visibility + c) * visibility)
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize(a: [f32; 3]) -> [f32; 3] {
    let l = dot(a, a).sqrt();
    a.map(|x| x / l)
}

/// The slice at angle `phi` through `view` for `normal`: the projected normal's length and its
/// signed angle from the view vector, as the shader finds them.
fn slice(view: [f32; 3], normal: [f32; 3], phi: f32) -> (f32, f32) {
    let direction = [phi.cos(), phi.sin(), 0.0];
    let axis = normalize(cross(direction, view));
    let d = dot(direction, view);
    let ortho = [0, 1, 2].map(|i| direction[i] - d * view[i]);
    let k = dot(normal, axis);
    let projected = [0, 1, 2].map(|i| normal[i] - axis[i] * k);
    let length = dot(projected, projected).sqrt();
    let sign = if dot(ortho, projected) >= 0.0 {
        1.0
    } else {
        -1.0
    };
    (
        length,
        sign * (dot(projected, view) / length).clamp(-1.0, 1.0).acos(),
    )
}

/// The visibility over `slices` slices when `horizon(phi, side)` gives the horizon cosine each
/// way (none: the tangent plane).
fn visibility(
    view: [f32; 3],
    normal: [f32; 3],
    slices: u32,
    horizon: impl Fn(f32, f32) -> Option<f32>,
) -> f32 {
    let mut total = 0.0;
    for s in 0..slices {
        let phi = (s as f32 + 0.5) / slices as f32 * PI;
        let (length, n) = slice(view, normal, phi);
        let h = |side: f32| {
            let low = tangent_cos(n, side);
            horizon(phi, side)
                .map_or(low, |c| c.max(low))
                .clamp(-1.0, 1.0)
        };
        total += length * slice_arc(n, -h(-1.0).acos(), h(1.0).acos());
    }
    total / slices as f32
}

#[test]
fn a_flat_plane_is_fully_visible_from_any_view() {
    for tilt in [0.0_f32, 0.4, 0.9, 1.3] {
        let view = [0.0, 0.0, 1.0];
        let normal = [tilt.sin(), 0.0, tilt.cos()];
        let v = visibility(view, normal, 64, |_, _| None);
        assert!((v - 1.0).abs() < 0.01, "tilt {tilt}: {v}");
    }
}

#[test]
fn a_right_angle_corner_is_half_visible() {
    // The point sits in the crease of a floor and a vertical wall, seen along the floor's
    // normal: every slice that reaches the wall has the wall's face (straight along the view) as
    // its horizon on that side, so half the cosine-weighted hemisphere is blocked.
    let view = [0.0, 0.0, 1.0];
    let v = visibility(view, view, 256, |phi, side| {
        // The wall fills the y < 0 half: a slice direction with sin(phi) * side < 0 hits it.
        (phi.sin() * side < 0.0).then_some(1.0)
    });
    assert!((v - 0.5).abs() < 0.02, "{v}");
}

#[test]
fn a_fully_enclosed_point_is_dark() {
    let view = [0.0, 0.0, 1.0];
    let v = visibility(view, view, 16, |_, _| Some(1.0));
    assert!(v < 0.01, "{v}");
}

#[test]
fn multi_bounce_keeps_full_visibility_and_is_monotonic() {
    for albedo in [0.2, 0.5, 0.9] {
        assert!(
            (multi_bounce(1.0, albedo) - 1.0).abs() < 1e-3,
            "albedo {albedo}"
        );
        assert_eq!(multi_bounce(0.0, albedo), 0.0);
        let mut last = 0.0;
        for i in 0..=100 {
            let x = i as f32 / 100.0;
            let y = multi_bounce(x, albedo);
            assert!(y >= last - 1e-6 && y >= x - 1e-6, "albedo {albedo} at {x}");
            last = y;
        }
    }
}

#[test]
fn the_quality_tiers_set_the_slices_and_steps() {
    assert_eq!(tier(0), None);
    assert_eq!(tier(1), Some((2, 4)));
    assert_eq!(tier(2), Some((3, 6)));
}
