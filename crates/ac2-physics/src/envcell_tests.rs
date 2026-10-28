//! Interiors on synthetic data (no game data): two 4 x 4 x 3 m rooms joined by a doorway, the
//! first with a door to the outside; the body walks between them and in from outdoors.

use std::sync::Arc;

use crate::body::{MovementInput, PhysicsBody, PHYSICS_QUANTUM};
use crate::bsp::{BspTree, Node, Polygon};
use crate::envcell::{CellBsp, CellBspNode, CellPortal, CellStructure, EnvCell};
use crate::frame::Frame;
use crate::landdefs::{lcoord_to_gid, SIDE_VERTEX_COUNT};
use crate::motion_interp::{ELEMENT_AIR, ELEMENT_GROUND};
use crate::physics_obj::BodyShape;
use crate::physics_obj_place::SetPositionError;
use crate::position::Position;
use crate::terrain::Terrain;
use crate::test_support::{desc, walking_values};
use crate::vector::{Plane, Quaternion, Sphere, Vector};

const BLOCK: u32 = 0x7F7F_0000;
const ROOM_A: u32 = BLOCK | 0x0100;
const ROOM_B: u32 = BLOCK | 0x0101;
/// Both rooms' frames sit at this block point.
const BASE: (f32, f32) = (80.0, 80.0);

fn v(x: f32, y: f32, z: f32) -> Vector {
    Vector::new(x, y, z)
}

/// A quad wound so its plane faces `facing`.
fn quad(points: [Vector; 4], facing: Vector) -> Polygon {
    let polygon = Polygon::new(points.to_vec());
    let n = polygon.plane.n;
    if n.x * facing.x + n.y * facing.y + n.z * facing.z > 0.0 {
        return polygon;
    }
    let mut reversed = points.to_vec();
    reversed.reverse();
    Polygon::new(reversed)
}

/// The wall at `y` (facing `facing_y`) with a doorway from x 1.5 to 2.5, 2.5 m high.
fn door_wall(y: f32, facing_y: f32) -> Vec<Polygon> {
    let f = v(0.0, facing_y, 0.0);
    vec![
        quad(
            [
                v(0.0, y, 0.0),
                v(1.5, y, 0.0),
                v(1.5, y, 3.0),
                v(0.0, y, 3.0),
            ],
            f,
        ),
        quad(
            [
                v(2.5, y, 0.0),
                v(4.0, y, 0.0),
                v(4.0, y, 3.0),
                v(2.5, y, 3.0),
            ],
            f,
        ),
        quad(
            [
                v(1.5, y, 2.5),
                v(2.5, y, 2.5),
                v(2.5, y, 3.0),
                v(1.5, y, 3.0),
            ],
            f,
        ),
    ]
}

fn solid_wall(y: f32, facing_y: f32) -> Polygon {
    quad(
        [
            v(0.0, y, 0.0),
            v(4.0, y, 0.0),
            v(4.0, y, 3.0),
            v(0.0, y, 3.0),
        ],
        v(0.0, facing_y, 0.0),
    )
}

fn doorway(y: f32) -> Polygon {
    Polygon::new(vec![
        v(1.5, y, 0.0),
        v(2.5, y, 0.0),
        v(2.5, y, 2.5),
        v(1.5, y, 2.5),
    ])
}

/// A room spanning y0..y1 (cell space) whose south and north walls are given.
fn room(y0: f32, y1: f32, mut walls: Vec<Polygon>) -> Arc<CellStructure> {
    walls.extend([
        quad(
            [
                v(0.0, y0, 0.0),
                v(4.0, y0, 0.0),
                v(4.0, y1, 0.0),
                v(0.0, y1, 0.0),
            ],
            v(0.0, 0.0, 1.0),
        ),
        quad(
            [
                v(0.0, y0, 3.0),
                v(4.0, y0, 3.0),
                v(4.0, y1, 3.0),
                v(0.0, y1, 3.0),
            ],
            v(0.0, 0.0, -1.0),
        ),
        quad(
            [
                v(0.0, y0, 0.0),
                v(0.0, y1, 0.0),
                v(0.0, y1, 3.0),
                v(0.0, y0, 3.0),
            ],
            v(1.0, 0.0, 0.0),
        ),
        quad(
            [
                v(4.0, y0, 0.0),
                v(4.0, y1, 0.0),
                v(4.0, y1, 3.0),
                v(4.0, y0, 3.0),
            ],
            v(-1.0, 0.0, 0.0),
        ),
    ]);
    let leaf = Node::Leaf {
        sphere: Sphere::new(v(2.0, (y0 + y1) * 0.5, 1.5), 4.0),
        solid: false,
        polygons: (0..walls.len()).collect(),
    };
    // Inside: x in 0..4, y in y0..y1, z in 0..3.
    let planes = [
        Plane::new(v(1.0, 0.0, 0.0), 0.0),
        Plane::new(v(-1.0, 0.0, 0.0), 4.0),
        Plane::new(v(0.0, 1.0, 0.0), -y0),
        Plane::new(v(0.0, -1.0, 0.0), y1),
        Plane::new(v(0.0, 0.0, 1.0), 0.0),
        Plane::new(v(0.0, 0.0, -1.0), 3.0),
    ];
    let nodes = planes
        .iter()
        .enumerate()
        .map(|(i, &plane)| CellBspNode {
            plane,
            pos: (i + 1 < planes.len()).then_some(i + 1),
            neg: None,
        })
        .collect();
    Arc::new(CellStructure {
        physics: Some(Arc::new(BspTree::new(vec![leaf], walls))),
        cell_bsp: CellBsp { nodes },
        portal_polygons: Vec::new(),
    })
}

fn rooms() -> Terrain {
    let heights = vec![0.0; SIDE_VERTEX_COUNT * SIDE_VERTEX_COUNT];
    let mut terrain = Terrain::new();
    terrain.insert_heights(BLOCK | 0xFFFF, &heights, |_, _| false);
    let position = Position::new(
        ROOM_A,
        Frame::new(v(BASE.0, BASE.1, 0.0), Quaternion::IDENTITY),
    );
    let mut a_walls = door_wall(0.0, 1.0);
    a_walls.extend(door_wall(4.0, -1.0));
    terrain.insert_env_cell(EnvCell {
        cell_id: ROOM_A,
        position,
        structure: room(0.0, 4.0, a_walls),
        portals: vec![
            CellPortal {
                other_cell: None,
                other_portal: None,
                polygon: doorway(0.0),
            },
            CellPortal {
                other_cell: Some(ROOM_B),
                other_portal: Some(0),
                polygon: doorway(4.0),
            },
        ],
        seen_outside: true,
        stab_list: vec![ROOM_B],
        objects: Vec::new(),
        physics_material: None,
    });
    let mut b_walls = door_wall(4.0, 1.0);
    b_walls.push(solid_wall(8.0, -1.0));
    terrain.insert_env_cell(EnvCell {
        cell_id: ROOM_B,
        position: Position::new(ROOM_B, position.frame),
        structure: room(4.0, 8.0, b_walls),
        portals: vec![CellPortal {
            other_cell: Some(ROOM_A),
            other_portal: Some(1),
            polygon: doorway(4.0),
        }],
        seen_outside: false,
        stab_list: vec![ROOM_A],
        objects: Vec::new(),
        physics_material: None,
    });
    terrain.connect_block(BLOCK, &[ROOM_A]);
    terrain
}

fn walker(cell_id: u32, x: f32, y: f32) -> PhysicsBody {
    let mut air = walking_values();
    air.forward_acceleration = 0.0;
    let shape = BodyShape {
        spheres: vec![
            Sphere::new(v(0.0, 0.0, 0.4), 0.4),
            Sphere::new(v(0.0, 0.0, 1.2), 0.4),
        ],
        height: 1.8,
        step_up_height: 0.5,
        step_down_height: 0.5,
        scale: 1.0,
        allow_free_heading: false,
    };
    let desc = desc(&[(ELEMENT_GROUND, walking_values()), (ELEMENT_AIR, air)]);
    let start = Position::new(cell_id, Frame::new(v(x, y, 0.0), Quaternion::IDENTITY));
    PhysicsBody::new(start, shape, Some(desc))
}

/// Settles, then walks along `heading` (compass degrees) for `seconds`; returns every position.
fn walk(body: &mut PhysicsBody, world: &Terrain, heading: f32, seconds: f32) -> Vec<Position> {
    for _ in 0..5 {
        body.step_quantum(world);
    }
    body.set_input(MovementInput {
        motion: [0.0, 1.0, 0.0],
        heading: Some(heading),
        turn: 0.0,
    });
    for _ in 0..10 {
        body.step_quantum(world);
    }
    let quanta = (seconds / PHYSICS_QUANTUM).round() as usize;
    (0..quanta)
        .map(|_| {
            body.step_quantum(world);
            body.position()
        })
        .collect()
}

#[test]
fn a_body_inside_a_room_stands_on_its_floor() {
    let world = rooms();
    let mut body = walker(ROOM_A, BASE.0 + 2.0, BASE.1 + 2.0);
    for _ in 0..20 {
        body.step_quantum(&world);
    }
    let state = body.movement_state();
    assert_eq!(state.cell_id, ROOM_A);
    assert!(state.contact && state.on_walkable, "{state:?}");
    assert!(state.origin[2].abs() < 0.01, "{state:?}");
}

#[test]
fn room_walls_stop_the_body() {
    let world = rooms();
    let mut body = walker(ROOM_A, BASE.0 + 2.0, BASE.1 + 1.0);
    // East: the wall at cell x = 4.
    let path = walk(&mut body, &world, 90.0, 2.0);
    assert!(path.iter().all(|p| p.cell_id == ROOM_A), "{path:?}");
    let x = path
        .iter()
        .map(|p| p.frame.origin.x)
        .fold(f32::MIN, f32::max);
    assert!(x <= BASE.0 + 4.0 - 0.39 && x > BASE.0 + 3.0, "{x}");
}

#[test]
fn walking_through_a_doorway_enters_the_next_room() {
    let world = rooms();
    let mut body = walker(ROOM_A, BASE.0 + 2.0, BASE.1 + 1.0);
    let path = walk(&mut body, &world, 0.0, 2.0);
    let last = path.last().expect("steps");
    assert_eq!(last.cell_id, ROOM_B, "{path:?}");
    assert!(last.frame.origin.y > BASE.1 + 5.0, "{last:?}");
    // It reaches the far wall of room B and no further.
    let y = path
        .iter()
        .map(|p| p.frame.origin.y)
        .fold(f32::MIN, f32::max);
    assert!(y <= BASE.1 + 8.0 - 0.39, "{y}");
}

#[test]
fn walking_in_from_outside_enters_the_building() {
    let world = rooms();
    let outside = lcoord_to_gid(0x7F * 8 + 4, 0x7F * 8 + 3);
    let mut body = walker(outside, BASE.0 + 2.0, BASE.1 - 2.0);
    let path = walk(&mut body, &world, 0.0, 1.5);
    let first_inside = path
        .iter()
        .find(|p| p.cell_id & 0xFFFF >= 0x100)
        .expect("the body went indoors");
    assert_eq!(first_inside.cell_id, ROOM_A, "{path:?}");
    // Outdoors it stays in land cells until its centre crosses the doorway.
    assert!(path
        .iter()
        .filter(|p| p.frame.origin.y < BASE.1 - 0.05)
        .all(|p| p.cell_id & 0xFFFF < 0x100));
    assert!(path
        .iter()
        .any(|p| p.cell_id == ROOM_A && p.frame.origin.y > BASE.1 + 1.0));
}

#[test]
fn placing_in_a_room_puts_the_body_on_its_floor() {
    let world = rooms();
    let mut body = walker(ROOM_A, BASE.0 + 2.0, BASE.1 + 2.0);
    let target = Position::new(
        ROOM_B,
        Frame::new(v(BASE.0 + 2.0, BASE.1 + 6.0, 0.2), Quaternion::IDENTITY),
    );
    assert_eq!(body.place(target, 0, &world), SetPositionError::Ok);
    let state = body.movement_state();
    assert_eq!(state.cell_id, ROOM_B);
    assert!(state.contact && state.origin[2].abs() < 0.01, "{state:?}");
    // A point of room A named with room B's id resolves through B's stab list.
    let misnamed = Position::new(
        ROOM_B,
        Frame::new(v(BASE.0 + 2.0, BASE.1 + 2.0, 0.0), Quaternion::IDENTITY),
    );
    assert_eq!(body.place(misnamed, 0, &world), SetPositionError::Ok);
    assert_eq!(body.position().cell_id, ROOM_A);
}

#[test]
fn an_interior_cell_names_its_physics_material() {
    let mut terrain = Terrain::new();
    terrain.insert_env_cell(EnvCell {
        cell_id: ROOM_A,
        position: Position::new(ROOM_A, Frame::new(v(80.0, 80.0, 0.0), Quaternion::IDENTITY)),
        structure: room(0.0, 4.0, Vec::new()),
        portals: Vec::new(),
        seen_outside: false,
        stab_list: Vec::new(),
        objects: Vec::new(),
        physics_material: Some(77),
    });
    assert_eq!(terrain.env_material(ROOM_A), Some(77));
    assert_eq!(terrain.env_material(ROOM_B), None);
}
