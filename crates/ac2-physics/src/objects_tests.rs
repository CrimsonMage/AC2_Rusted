//! Walking into static objects on synthetic terrain (no game data): physics-BSP boxes,
//! cylinders and spheres stop, slide, lift and hold the body.

use std::sync::Arc;

use crate::body::{MovementInput, PhysicsBody, PHYSICS_QUANTUM};
use crate::bsp::{BspTree, Node, Polygon};
use crate::frame::Frame;
use crate::landdefs::{lcoord_to_gid, SIDE_VERTEX_COUNT};
use crate::motion_interp::{ELEMENT_AIR, ELEMENT_GROUND};
use crate::objects::{ObjectShape, StaticObject};
use crate::physics_obj::BodyShape;
use crate::physics_obj_place::{SetPositionError, FORCE_SPF, SLIDE_SPF};
use crate::position::Position;
use crate::shapes::cylsphere::CylSphere;
use crate::terrain::Terrain;
use crate::test_support::{desc, walking_values};
use crate::vector::{Quaternion, Sphere, Vector};

const BLOCK: u32 = 0x7F7F_FFFF;

fn cell_at(x: f32, y: f32) -> u32 {
    lcoord_to_gid(0x7F * 8 + (x / 20.0) as i32, 0x7F * 8 + (y / 20.0) as i32)
}

fn flat() -> Terrain {
    let heights = vec![0.0; SIDE_VERTEX_COUNT * SIDE_VERTEX_COUNT];
    let mut terrain = Terrain::new();
    terrain.insert_heights(BLOCK, &heights, |_, _| false);
    terrain
}

fn position(x: f32, y: f32, z: f32) -> Position {
    Position::new(
        cell_at(x, y),
        Frame::new(Vector::new(x, y, z), Quaternion::IDENTITY),
    )
}

/// A closed box `min..max` of outward-facing quads in one leaf.
fn box_tree(min: Vector, max: Vector) -> BspTree {
    let (a, b) = (min, max);
    let v = Vector::new;
    let faces = vec![
        vec![
            v(a.x, a.y, b.z),
            v(b.x, a.y, b.z),
            v(b.x, b.y, b.z),
            v(a.x, b.y, b.z),
        ],
        vec![
            v(a.x, a.y, a.z),
            v(a.x, b.y, a.z),
            v(b.x, b.y, a.z),
            v(b.x, a.y, a.z),
        ],
        vec![
            v(b.x, a.y, a.z),
            v(b.x, b.y, a.z),
            v(b.x, b.y, b.z),
            v(b.x, a.y, b.z),
        ],
        vec![
            v(a.x, a.y, a.z),
            v(a.x, a.y, b.z),
            v(a.x, b.y, b.z),
            v(a.x, b.y, a.z),
        ],
        vec![
            v(a.x, b.y, a.z),
            v(a.x, b.y, b.z),
            v(b.x, b.y, b.z),
            v(b.x, b.y, a.z),
        ],
        vec![
            v(a.x, a.y, a.z),
            v(b.x, a.y, a.z),
            v(b.x, a.y, b.z),
            v(a.x, a.y, b.z),
        ],
    ];
    let polygons: Vec<Polygon> = faces.into_iter().map(Polygon::new).collect();
    let center = v((a.x + b.x) * 0.5, (a.y + b.y) * 0.5, (a.z + b.z) * 0.5);
    let radius = ((b.x - a.x).powi(2) + (b.y - a.y).powi(2) + (b.z - a.z).powi(2)).sqrt();
    let leaf = Node::Leaf {
        sphere: Sphere::new(center, radius),
        solid: false,
        polygons: (0..polygons.len()).collect(),
    };
    BspTree::new(vec![leaf], polygons)
}

fn world_with(object: StaticObject) -> Terrain {
    let mut terrain = flat();
    terrain.insert_object(object);
    terrain
}

fn bsp_box(at: Position, min: Vector, max: Vector) -> StaticObject {
    let shape = ObjectShape::new(vec![Arc::new(box_tree(min, max))], Vec::new(), Vec::new());
    StaticObject::new(at, Vector::new(1.0, 1.0, 1.0), Arc::new(shape))
}

fn walker(x: f32, y: f32, z: f32) -> PhysicsBody {
    let mut air = walking_values();
    air.forward_acceleration = 0.0;
    air.backward_acceleration = 0.0;
    air.strafe_acceleration = 0.0;
    let shape = BodyShape {
        spheres: vec![
            Sphere::new(Vector::new(0.0, 0.0, 0.4), 0.4),
            Sphere::new(Vector::new(0.0, 0.0, 1.2), 0.4),
        ],
        height: 1.8,
        step_up_height: 0.5,
        step_down_height: 0.5,
        scale: 1.0,
        allow_free_heading: false,
    };
    let desc = desc(&[(ELEMENT_GROUND, walking_values()), (ELEMENT_AIR, air)]);
    PhysicsBody::new(position(x, y, z), shape, Some(desc))
}

/// Settles the body, then walks it north for `seconds`, returning every origin on the way.
fn walk_north(body: &mut PhysicsBody, world: &Terrain, seconds: f32) -> Vec<[f32; 3]> {
    for _ in 0..5 {
        body.step_quantum(world);
    }
    body.set_input(MovementInput {
        motion: [0.0, 1.0, 0.0],
        heading: Some(0.0),
        turn: 0.0,
    });
    let quanta = (seconds / PHYSICS_QUANTUM).round() as usize;
    (0..quanta)
        .map(|_| {
            body.step_quantum(world);
            body.movement_state().origin
        })
        .collect()
}

#[test]
fn a_bsp_wall_stops_the_body_at_its_radius() {
    // The wall's south face is at y = 84.
    let wall = bsp_box(
        position(80.0, 84.0, 0.0),
        Vector::new(-3.0, 0.0, 0.0),
        Vector::new(3.0, 2.0, 3.0),
    );
    let world = world_with(wall);
    let mut body = walker(80.0, 80.0, 0.0);
    let path = walk_north(&mut body, &world, 3.0);
    let last = *path.last().expect("steps");
    assert!(path.iter().all(|p| p[1] <= 84.0 - 0.39), "{path:?}");
    assert!(last[1] > 83.0, "walked up to the wall: {last:?}");
    assert!(last[2].abs() < 0.01, "still on the ground: {last:?}");
    assert!(body.movement_state().contact);
}

#[test]
fn a_low_bsp_ledge_is_stepped_onto_and_off() {
    // A 0.3 m ledge from y = 84 to 88, lower than the 0.5 m step-up height.
    let ledge = bsp_box(
        position(80.0, 84.0, 0.0),
        Vector::new(-3.0, 0.0, 0.0),
        Vector::new(3.0, 4.0, 0.3),
    );
    let world = world_with(ledge);
    let mut body = walker(80.0, 80.0, 0.0);
    let path = walk_north(&mut body, &world, 3.5);
    let on_top: Vec<_> = path.iter().filter(|p| p[1] > 84.5 && p[1] < 87.5).collect();
    assert!(!on_top.is_empty(), "{path:?}");
    assert!(
        on_top.iter().all(|p| (p[2] - 0.3).abs() < 0.02),
        "{on_top:?}"
    );
    let last = path.last().expect("steps");
    assert!(
        last[1] > 88.5 && last[2].abs() < 0.01,
        "walked off the far side: {last:?}"
    );
}

#[test]
fn a_body_dropped_onto_a_bsp_platform_stands_on_it() {
    let platform = bsp_box(
        position(80.0, 80.0, 0.0),
        Vector::new(-5.0, -5.0, 0.0),
        Vector::new(5.0, 5.0, 1.0),
    );
    let world = world_with(platform);
    let mut body = walker(80.0, 80.0, 3.0);
    for _ in 0..30 {
        body.step_quantum(&world);
    }
    let state = body.movement_state();
    assert!(state.contact && state.on_walkable, "{state:?}");
    assert!((state.origin[2] - 1.0).abs() < 0.01, "{state:?}");
}

#[test]
fn cylinders_and_spheres_are_not_walked_through() {
    let cylinder = ObjectShape::new(
        Vec::new(),
        vec![CylSphere {
            low: Vector::ZERO,
            height: 3.0,
            radius: 0.5,
        }],
        Vec::new(),
    );
    let ball = ObjectShape::new(
        Vec::new(),
        Vec::new(),
        vec![Sphere::new(Vector::new(0.0, 0.0, 0.8), 0.8)],
    );
    // The ball's centre is 0.4 m from both body sphere centres vertically.
    let ball_reach = (1.2_f32 * 1.2 - 0.4 * 0.4).sqrt();
    for (shape, reach) in [(cylinder, 0.9_f32), (ball, ball_reach)] {
        let at = position(80.2, 84.0, 0.0);
        let world = world_with(StaticObject::new(
            at,
            Vector::new(1.0, 1.0, 1.0),
            Arc::new(shape),
        ));
        let mut body = walker(80.0, 80.0, 0.0);
        let path = walk_north(&mut body, &world, 3.0);
        for p in &path {
            let (dx, dy) = (p[0] - 80.2, p[1] - 84.0);
            assert!(
                (dx * dx + dy * dy).sqrt() >= reach - 0.02,
                "{p:?} in {path:?}"
            );
        }
        assert!(path.last().expect("steps")[1] > 82.5, "{path:?}");
    }
}

#[test]
fn a_scaled_object_collides_at_its_scaled_size() {
    // A 1 m box scaled by 3 has its south face at y = 84 - 1.5.
    let shape = ObjectShape::new(
        vec![Arc::new(box_tree(
            Vector::new(-0.5, -0.5, 0.0),
            Vector::new(0.5, 0.5, 1.0),
        ))],
        Vec::new(),
        Vec::new(),
    );
    let object = StaticObject::new(
        position(80.0, 84.0, 0.0),
        Vector::new(3.0, 3.0, 3.0),
        Arc::new(shape),
    );
    let world = world_with(object);
    let mut body = walker(80.0, 79.0, 0.0);
    let path = walk_north(&mut body, &world, 3.0);
    let last = path.last().expect("steps");
    assert!(last[1] <= 82.5 - 0.39 && last[1] > 81.5, "{path:?}");
}

#[test]
fn placing_into_a_box_wall_slides_out_or_fails() {
    let block = bsp_box(
        position(80.0, 84.0, 0.0),
        Vector::new(-1.0, -1.0, 0.0),
        Vector::new(1.0, 1.0, 2.0),
    );
    let world = world_with(block);
    let mut body = walker(70.0, 70.0, 0.0);
    let before = body.position();
    // The body's spheres would cut the box's north face (y = 85).
    let error = body.place(position(80.0, 85.2, 0.0), 0, &world);
    assert_ne!(error, SetPositionError::Ok);
    assert_eq!(
        body.position(),
        before,
        "a failed placement leaves the body"
    );
    let error = body.place(position(80.0, 85.2, 0.0), SLIDE_SPF, &world);
    assert_eq!(error, SetPositionError::Ok);
    let p = body.position().frame.origin;
    let (dx, dy) = (p.x - 80.0, p.y - 84.0);
    assert!(
        dx.abs() >= 1.39 || dy.abs() >= 1.39 || p.z >= 2.0,
        "placed clear of the box: {p:?}"
    );
}

#[test]
fn placing_near_the_ground_steps_down_onto_it() {
    let world = flat();
    let mut body = walker(70.0, 70.0, 0.0);
    assert_eq!(
        body.place(position(75.0, 75.0, 0.3), 0, &world),
        SetPositionError::Ok
    );
    let state = body.movement_state();
    assert!(state.contact && state.origin[2].abs() < 0.01, "{state:?}");
    assert_eq!(
        body.place(position(75.0, 75.0, 5.0), 0, &world),
        SetPositionError::Ok
    );
    let state = body.movement_state();
    assert!(
        !state.contact && (state.origin[2] - 5.0).abs() < 1e-6,
        "{state:?}"
    );
    assert_eq!(
        body.place(position(75.0, 75.0, 5.0), FORCE_SPF, &world),
        SetPositionError::Ok
    );
}
