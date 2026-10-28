//! Decal geometry and fade on synthetic ground: the clip keeps what lies inside the box, the
//! texture coordinates run across it, and the fade follows `DecalEffect::Think`.

use ac2_core::DataId;
use ac2_math::Vec3;

use super::{construct_decal, opacity_at, DecalEffects, DecalSpec, GroundVertex, DECAL_LIFT};

fn flat(x0: f32, y0: f32, x1: f32, y1: f32) -> [[GroundVertex; 3]; 2] {
    let v = |x, y| GroundVertex {
        position: [x, y, 0.0],
        normal: [0.0, 0.0, 1.0],
    };
    [
        [v(x0, y0), v(x1, y0), v(x0, y1)],
        [v(x1, y0), v(x1, y1), v(x0, y1)],
    ]
}

fn area(tris: &[[super::DecalCorner; 3]]) -> f64 {
    tris.iter()
        .map(|t| {
            let p = t.map(|c| (f64::from(c.position[0]), f64::from(c.position[1])));
            ((p[1].0 - p[0].0) * (p[2].1 - p[0].1) - (p[2].0 - p[0].0) * (p[1].1 - p[0].1)).abs()
                * 0.5
        })
        .sum()
}

#[test]
fn ground_larger_than_the_box_is_clipped_to_the_box() {
    // A 4 m square of ground under a 2 m box at the origin: the box's footprint, 2 x 2, remains.
    let ground = flat(-2.0, -2.0, 2.0, 2.0);
    let tris = construct_decal(
        [0.0; 3],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 0.0],
        [2.0; 3],
        &ground,
    );
    assert!(!tris.is_empty());
    assert!((area(&tris) - 4.0).abs() < 1e-4, "area {}", area(&tris));
    for tri in &tris {
        for c in tri {
            assert!(c.position[0].abs() <= 1.0 + 1e-5 && c.position[1].abs() <= 1.0 + 1e-5);
            assert!(
                (c.position[2] - DECAL_LIFT).abs() < 1e-7,
                "lifted off the ground"
            );
        }
    }
}

#[test]
fn ground_outside_the_box_gives_no_triangles() {
    let ground = flat(10.0, 10.0, 12.0, 12.0);
    let tris = construct_decal(
        [0.0; 3],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 0.0],
        [2.0; 3],
        &ground,
    );
    assert!(tris.is_empty());
}

#[test]
fn uv_runs_zero_to_one_across_the_box_along_the_tangent() {
    // Ground exactly the box: the corner at +x maps to u = 1, the origin to (0.5, 0.5).
    let ground = flat(-1.0, -1.0, 1.0, 1.0);
    let tris = construct_decal(
        [0.0; 3],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 0.0],
        [2.0; 3],
        &ground,
    );
    let uvs: Vec<[f32; 2]> = tris.iter().flatten().map(|c| c.uv).collect();
    let max_u = uvs.iter().map(|uv| uv[0]).fold(f32::MIN, f32::max);
    let min_u = uvs.iter().map(|uv| uv[0]).fold(f32::MAX, f32::min);
    assert!(
        (max_u - 1.0).abs() < 1e-5 && (min_u - 0.0).abs() < 1e-5,
        "u {min_u}..{max_u}"
    );
    assert!(uvs.iter().all(|uv| (0.0..=1.0).contains(&uv[1])));
}

#[test]
fn a_rotated_tangent_turns_the_texture_axes() {
    // Tangent along +y: a ground point at +y maps to u = 1, not v.
    let ground = flat(-1.0, -1.0, 1.0, 1.0);
    let tris = construct_decal(
        [0.0; 3],
        [0.0, 0.0, 1.0],
        [0.0, 1.0, 0.0],
        [2.0; 3],
        &ground,
    );
    let top = tris
        .iter()
        .flatten()
        .find(|c| c.position[1] > 0.99)
        .expect("a corner at +y");
    assert!((top.uv[0] - 1.0).abs() < 1e-5, "u at +y {}", top.uv[0]);
}

#[test]
fn opacity_fades_in_then_out_and_dies_at_the_lifespan() {
    // Lifespan 10 s: fade in over 0.25 s, fade out over the last 1 s of the 2 s fade window.
    assert_eq!(opacity_at(0.0, 10.0, false), Some(0.0));
    assert_eq!(opacity_at(0.125, 10.0, false), Some(1.0));
    assert_eq!(opacity_at(9.5, 10.0, false), Some(0.5));
    assert_eq!(opacity_at(10.0, 10.0, false), None);
    // A draw-once decal is never faded in.
    assert_eq!(opacity_at(0.0, 10.0, true), Some(1.0));
}

fn spec(material: u32, once: bool) -> DecalSpec {
    DecalSpec {
        material: DataId::new(material),
        lifespan: 10.0,
        size: 2.0,
        direction: Vec3::new(1.0, 0.0, 0.0),
        color: [1.0, 0.5, 0.25, 0.8],
        draw_once: once,
        origin: Vec3::new(0.0, 0.0, 0.0),
    }
}

#[test]
fn decals_of_one_material_draw_as_one_geometry_and_expire() {
    let ground = flat(-2.0, -2.0, 2.0, 2.0);
    let mut effects = DecalEffects::default();
    effects.create(spec(0x3100_0001, false), 0.0, &ground);
    effects.create(spec(0x3100_0001, false), 0.0, &ground);
    effects.create(spec(0x3100_0002, false), 0.0, &ground);
    assert_eq!(effects.count(), 3);

    effects.think(9.5);
    let geometry = effects.geometry();
    assert_eq!(geometry.len(), 2, "one geometry per material");
    let first = &geometry[0];
    assert_eq!(first.0, DataId::new(0x3100_0001));
    let one = construct_decal(
        [0.0; 3],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 0.0],
        [2.0; 3],
        &ground,
    )
    .len();
    assert_eq!(first.1.indices.len(), 2 * one * 3, "two decals' triangles");
    // The fade at 9.5 s is half opacity: alpha truncates 0.8 * 0.5 * 255 to 102.
    let alpha = first.1.vertices[0].color[3];
    assert!((alpha - 102.0 / 255.0).abs() < 1e-6, "alpha {alpha}");

    effects.think(10.0);
    assert_eq!(effects.count(), 0, "expired decals leave");
}

#[test]
fn a_draw_once_decal_is_spent_by_its_draw() {
    let ground = flat(-2.0, -2.0, 2.0, 2.0);
    let mut effects = DecalEffects::default();
    effects.create(spec(0x3100_0003, true), 0.0, &ground);
    effects.think(0.0);
    assert_eq!(effects.geometry().len(), 1);
    assert_eq!(effects.count(), 0, "drawn once, gone");
}

#[test]
fn a_decal_with_no_ground_is_alive_and_draws_nothing() {
    let mut effects = DecalEffects::default();
    effects.create(spec(0x3100_0004, false), 0.0, &[]);
    effects.think(1.0);
    assert_eq!(effects.count(), 1);
    assert!(effects.geometry().is_empty());
}
