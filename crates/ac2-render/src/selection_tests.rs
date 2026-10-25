//! Picking boxes in front of a camera.

use ac2_math::{Mat4, Vec3};

use crate::selection::{pick, pick_part, PickCandidate, PickMesh};
use crate::Camera;

/// A unit cube centred on the origin, as twelve triangles.
fn cube() -> PickMesh {
    let p = |x: f32, y: f32, z: f32| Vec3::new(x - 0.5, y - 0.5, z - 0.5);
    let faces = [
        [p(0., 0., 0.), p(1., 0., 0.), p(1., 1., 0.), p(0., 1., 0.)],
        [p(0., 0., 1.), p(1., 0., 1.), p(1., 1., 1.), p(0., 1., 1.)],
        [p(0., 0., 0.), p(1., 0., 0.), p(1., 0., 1.), p(0., 0., 1.)],
        [p(0., 1., 0.), p(1., 1., 0.), p(1., 1., 1.), p(0., 1., 1.)],
        [p(0., 0., 0.), p(0., 1., 0.), p(0., 1., 1.), p(0., 0., 1.)],
        [p(1., 0., 0.), p(1., 1., 0.), p(1., 1., 1.), p(1., 0., 1.)],
    ];
    let triangles = faces
        .iter()
        .flat_map(|f| [[f[0], f[1], f[2]], [f[0], f[2], f[3]]])
        .collect();
    PickMesh {
        min: Vec3::splat(-0.5),
        max: Vec3::splat(0.5),
        parts: vec![(Mat4::IDENTITY, triangles)],
        part_types: vec![0x8100_001C],
    }
}

fn camera() -> Camera {
    Camera {
        eye: Vec3::new(0.0, -10.0, 0.0),
        target: Vec3::ZERO,
        fov_y: 60_f32.to_radians(),
        near: 0.1,
        far: 100.0,
    }
}

fn at(id: u64, mesh: &PickMesh, x: f32, y: f32, player: bool) -> PickCandidate<'_> {
    PickCandidate {
        id,
        world: Mat4::from_translation(Vec3::new(x, y, 0.0)),
        pose: None,
        mesh,
        third_person_player: player,
    }
}

#[test]
fn the_nearest_hit_under_the_cursor_is_selected() {
    let mesh = cube();
    let camera = camera();
    let vp = camera.view_projection(1.0);
    let screen = (200, 200);
    let centre = (100, 100);
    // A near and a far cube on the view axis: the near one wins.
    let candidates = [at(1, &mesh, 0.0, 5.0, false), at(2, &mesh, 0.0, 0.0, false)];
    assert_eq!(pick(vp, camera.eye, screen, centre, &candidates), Some(2));
    // The hit part's mesh type (`ObjectSelection::meshType`).
    assert_eq!(
        pick_part(vp, camera.eye, screen, centre, &candidates),
        Some((2, 0x8100_001C))
    );
    // Off to the side nothing is under the cursor.
    assert_eq!(pick(vp, camera.eye, screen, (5, 5), &candidates), None);
}

#[test]
fn the_third_person_player_loses_to_anything_else() {
    let mesh = cube();
    let camera = camera();
    let vp = camera.view_projection(1.0);
    // The player's cube is nearer, but it is selected behind everything.
    let candidates = [at(7, &mesh, 0.0, -3.0, true), at(8, &mesh, 0.0, 0.0, false)];
    assert_eq!(
        pick(vp, camera.eye, (200, 200), (100, 100), &candidates),
        Some(8)
    );
    let alone = [at(7, &mesh, 0.0, -3.0, true)];
    assert_eq!(
        pick(vp, camera.eye, (200, 200), (100, 100), &alone),
        Some(7)
    );
}
