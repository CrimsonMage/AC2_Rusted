//! Line of sight on synthetic worlds (no game data): terrain, landblock edges, a pillar and a
//! one-room building with a door to the outside.

use std::sync::Arc;

use super::objects::{trace_ray_cylsphere, trace_ray_sphere};
use super::{LosParameters, Ray, RayIntersection};
use crate::bsp::{BspTree, Node, Polygon};
use crate::envcell::{CellBsp, CellPortal, CellStructure, EnvCell};
use crate::frame::Frame;
use crate::landdefs::{lcoord_to_gid, SIDE_VERTEX_COUNT};
use crate::objects::{ObjectShape, StaticObject};
use crate::position::Position;
use crate::shapes::cylsphere::CylSphere;
use crate::terrain::Terrain;
use crate::vector::{Plane, Quaternion, Sphere, Vector};

const BLOCK_X: i32 = 0x7F * 8;
const BLOCK_Y: i32 = 0x7F * 8;
const ROOM: u32 = 0x7F7F_0100;

fn v(x: f32, y: f32, z: f32) -> Vector {
    Vector::new(x, y, z)
}

/// The land cell of block 7F7F under block point (x, y).
fn cell_at(x: f32, y: f32) -> u32 {
    lcoord_to_gid(BLOCK_X + (x / 20.0) as i32, BLOCK_Y + (y / 20.0) as i32)
}

fn block(terrain: &mut Terrain, block_cell_id: u32, height: impl Fn(usize, usize) -> f32) {
    let heights: Vec<f32> = (0..SIDE_VERTEX_COUNT)
        .flat_map(|x| (0..SIDE_VERTEX_COUNT).map(move |y| (x, y)))
        .map(|(x, y)| height(x, y))
        .collect();
    terrain.insert_heights(block_cell_id, &heights, |_, _| false);
}

fn flat() -> Terrain {
    let mut terrain = Terrain::new();
    block(&mut terrain, 0x7F7F_FFFF, |_, _| 0.0);
    terrain
}

fn check(terrain: &Terrain, from: Vector, offset: Vector) -> LosParameters {
    let mut params = LosParameters::new();
    params.set_from_position_and_offset(
        &Position::new(
            cell_at(from.x, from.y),
            Frame::new(from, Quaternion::IDENTITY),
        ),
        offset,
    );
    assert!(terrain.los_check(&mut params));
    params
}

fn near(a: Vector, b: Vector) -> bool {
    (a.x - b.x).abs() < 1e-3 && (a.y - b.y).abs() < 1e-3 && (a.z - b.z).abs() < 1e-3
}

#[test]
fn rays_from_offsets() {
    let ray = Ray::from_offset(v(1.0, 2.0, 3.0), v(0.0001, -0.0001, 0.0));
    assert_eq!((ray.length, ray.dir), (0.0, v(0.0, 1.0, 0.0)));
    let ray = Ray::from_offset(v(0.0, 0.0, 0.0), v(3.0, 0.0, 4.0));
    assert_eq!(ray.length, 5.0);
    assert!(near(ray.dir, v(0.6, 0.0, 0.8)));
    assert!(near(ray.end_point(), v(3.0, 0.0, 4.0)));
}

#[test]
fn a_ray_down_stops_on_the_ground() {
    let terrain = flat();
    let params = check(&terrain, v(30.0, 30.0, 10.0), v(0.0, 0.0, -20.0));
    assert_eq!(params.trace_result(), 1);
    assert_eq!(params.final_cell_id(), cell_at(30.0, 30.0));
    assert!(
        near(params.end_point(), v(30.0, 30.0, 0.0)),
        "{:?}",
        params.end_point()
    );
    assert_eq!(params.impact_normal(), v(0.0, 0.0, 1.0));
}

#[test]
fn a_clear_ray_ends_where_it_was_aimed() {
    let terrain = flat();
    let params = check(&terrain, v(10.0, 10.0, 5.0), v(40.0, 60.0, 0.0));
    assert_eq!(params.trace_result(), 0);
    assert_eq!(params.final_cell_id(), cell_at(50.0, 70.0));
    assert!(near(params.end_point(), v(50.0, 70.0, 5.0)));
    assert_eq!(params.impact_normal(), Vector::ZERO);
}

#[test]
fn a_hillside_blocks_the_ray() {
    // The ground rises 50 m between x 80 and 90.
    let mut terrain = Terrain::new();
    block(
        &mut terrain,
        0x7F7F_FFFF,
        |x, _| if x >= 9 { 50.0 } else { 0.0 },
    );
    let params = check(&terrain, v(10.0, 50.0, 5.0), v(140.0, 0.0, 0.0));
    assert_eq!(params.trace_result(), 1);
    assert!(
        near(params.end_point(), v(81.0, 50.0, 5.0)),
        "{:?}",
        params.end_point()
    );
    let n = params.impact_normal();
    assert!((n.x + 0.980_580_7).abs() < 1e-6 && n.z > 0.19, "{n:?}");
}

#[test]
fn the_trace_stops_at_the_edge_of_the_resident_world() {
    let terrain = flat();
    let params = check(&terrain, v(150.0, 50.0, 5.0), v(30.0, 0.0, 0.0));
    assert_eq!(params.trace_result(), 2);
    assert_eq!(params.next_block(), (0x807F_FFFF, 3));
    // The end point is in the final cell's landblock: the west edge of 807F.
    assert_eq!(params.final_cell_id() & 0xFFFF_0000, 0x807F_0000);
    assert!(
        near(params.end_point(), v(0.0, 50.0, 5.0)),
        "{:?}",
        params.end_point()
    );
}

#[test]
fn the_trace_crosses_into_the_next_landblock() {
    let mut terrain = flat();
    // The neighbour to the east rises 50 m between its x 10 and 20.
    block(
        &mut terrain,
        0x807F_FFFF,
        |x, _| if x >= 2 { 50.0 } else { 0.0 },
    );
    let params = check(&terrain, v(150.0, 50.0, 5.0), v(40.0, 0.0, 0.0));
    assert_eq!(params.trace_result(), 1);
    assert_eq!(params.final_cell_id() & 0xFFFF_0000, 0x807F_0000);
    assert!(
        near(params.end_point(), v(11.0, 50.0, 5.0)),
        "{:?}",
        params.end_point()
    );
}

fn pillar(terrain: &mut Terrain) {
    let shape = Arc::new(ObjectShape::new(
        Vec::new(),
        vec![CylSphere {
            low: Vector::ZERO,
            height: 4.0,
            radius: 1.0,
        }],
        Vec::new(),
    ));
    let at = v(60.0, 50.0, 0.0);
    terrain.insert_object(StaticObject::new(
        Position::new(cell_at(at.x, at.y), Frame::new(at, Quaternion::IDENTITY)),
        v(1.0, 1.0, 1.0),
        shape,
    ));
}

#[test]
fn objects_block_the_physics_ray_only() {
    let mut terrain = flat();
    pillar(&mut terrain);
    let params = check(&terrain, v(50.0, 50.0, 2.0), v(20.0, 0.0, 0.0));
    assert_eq!(params.trace_result(), 1);
    assert!(
        near(params.end_point(), v(59.0, 50.0, 2.0)),
        "{:?}",
        params.end_point()
    );
    assert_eq!(params.impact_normal(), v(-1.0, -0.0, -0.0));

    let mut params = LosParameters::new();
    params.intersect = RayIntersection::EnvironmentOnly;
    params.set_from_position_and_offset(
        &Position::new(
            cell_at(50.0, 50.0),
            Frame::new(v(50.0, 50.0, 2.0), Quaternion::IDENTITY),
        ),
        v(20.0, 0.0, 0.0),
    );
    assert!(terrain.los_check(&mut params));
    assert_eq!(params.trace_result(), 0);
    assert!(near(params.end_point(), v(70.0, 50.0, 2.0)));
}

#[test]
fn shape_traces() {
    let ray = Ray::line_segment(v(-5.0, 0.0, 0.0), v(5.0, 0.0, 0.0));
    let sphere = Sphere::new(Vector::ZERO, 1.0);
    assert_eq!(trace_ray_sphere(&sphere, &ray), Some(0.4));
    let inside = Ray::line_segment(v(0.5, 0.0, 0.0), v(5.0, 0.0, 0.0));
    assert_eq!(trace_ray_sphere(&sphere, &inside), Some(0.0));
    let away = Ray::line_segment(v(2.0, 0.0, 0.0), v(5.0, 0.0, 0.0));
    assert_eq!(trace_ray_sphere(&sphere, &away), None);

    let cylinder = CylSphere {
        low: v(0.0, 0.0, -1.0),
        height: 2.0,
        radius: 1.0,
    };
    assert_eq!(trace_ray_cylsphere(&cylinder, &ray), Some(0.4));
    // A level ray meets the side whatever its height (retail skips the cap test when
    // |dir.z| <= F_EPSILON); a rising one passes over.
    let level = Ray::line_segment(v(-5.0, 0.0, 2.0), v(5.0, 0.0, 2.0));
    assert_eq!(trace_ray_cylsphere(&cylinder, &level), Some(0.4));
    let over = Ray::line_segment(v(-5.0, 0.0, 2.0), v(5.0, 0.0, 3.0));
    assert_eq!(trace_ray_cylsphere(&cylinder, &over), None);
    // Down onto the top cap from above the axis.
    let down = Ray::line_segment(v(0.2, 0.0, 3.0), v(0.2, 0.0, -3.0));
    let t = trace_ray_cylsphere(&cylinder, &down).expect("cap");
    assert!((t - 2.0 / 6.0).abs() < 1e-6, "{t}");
}

/// A 4 x 4 x 3 m room at block point (80, 80): a solid wall beyond x = 4 and a door (portal to
/// the outside) filling its x = 0 side.
fn room(terrain: &mut Terrain) {
    let sphere = Sphere::new(v(2.0, 2.0, 1.5), 4.0);
    let nodes = vec![
        Node::Internal {
            sphere,
            plane: Plane::new(v(-1.0, 0.0, 0.0), 4.0),
            pos: Some(1),
            neg: Some(4),
        },
        Node::Internal {
            sphere,
            plane: Plane::new(v(1.0, 0.0, 0.0), 0.0),
            pos: Some(2),
            neg: Some(3),
        },
        Node::Leaf {
            sphere,
            solid: false,
            polygons: Vec::new(),
        },
        Node::Leaf {
            sphere,
            solid: false,
            polygons: Vec::new(),
        },
        Node::Leaf {
            sphere,
            solid: true,
            polygons: Vec::new(),
        },
    ];
    let door = Polygon::new(vec![
        v(0.0, 0.0, 0.0),
        v(0.0, 4.0, 0.0),
        v(0.0, 4.0, 3.0),
        v(0.0, 0.0, 3.0),
    ]);
    let structure = Arc::new(CellStructure {
        physics: Some(Arc::new(BspTree::new(nodes, Vec::new()))),
        cell_bsp: CellBsp::default(),
        portal_polygons: vec![door.clone()],
    });
    terrain.insert_env_cell(EnvCell {
        cell_id: ROOM,
        position: Position::new(ROOM, Frame::new(v(80.0, 80.0, 0.0), Quaternion::IDENTITY)),
        structure,
        portals: vec![CellPortal {
            other_cell: None,
            other_portal: None,
            polygon: door,
        }],
        seen_outside: true,
        stab_list: Vec::new(),
        objects: Vec::new(),
    });
    terrain.connect_block(0x7F7F_FFFF, &[ROOM]);
}

#[test]
fn a_wall_inside_stops_the_ray() {
    let mut terrain = flat();
    room(&mut terrain);
    let mut params = LosParameters::new();
    params.set_from_position_and_offset(
        &Position::new(ROOM, Frame::new(v(81.0, 82.0, 1.0), Quaternion::IDENTITY)),
        v(10.0, 0.0, 0.0),
    );
    assert!(terrain.los_check(&mut params));
    assert_eq!(params.trace_result(), 1);
    assert_eq!(params.final_cell_id(), ROOM);
    assert!(
        near(params.end_point(), v(84.0, 82.0, 1.0)),
        "{:?}",
        params.end_point()
    );
    assert_eq!(params.impact_normal(), v(-1.0, 0.0, 0.0));
}

#[test]
fn a_ray_leaves_through_the_door() {
    let mut terrain = flat();
    room(&mut terrain);
    let mut params = LosParameters::new();
    params.set_from_position_and_offset(
        &Position::new(ROOM, Frame::new(v(82.0, 82.0, 1.0), Quaternion::IDENTITY)),
        v(-10.0, 0.0, 0.0),
    );
    assert!(terrain.los_check(&mut params));
    assert_eq!(params.trace_result(), 0);
    assert_eq!(params.final_cell_id(), cell_at(72.0, 82.0));
    assert!(
        near(params.end_point(), v(72.0, 82.0, 1.0)),
        "{:?}",
        params.end_point()
    );
}

#[test]
fn a_ray_from_outside_enters_through_the_door() {
    let mut terrain = flat();
    room(&mut terrain);
    let params = check(&terrain, v(70.0, 82.0, 1.0), v(20.0, 0.0, 0.0));
    assert_eq!(params.trace_result(), 1);
    assert_eq!(params.final_cell_id(), ROOM);
    assert!(
        near(params.end_point(), v(84.0, 82.0, 1.0)),
        "{:?}",
        params.end_point()
    );
}

#[test]
fn begin_and_end_positions_across_landblocks() {
    let mut params = LosParameters::new();
    let begin = Position::new(
        0x7F7F_0039,
        Frame::new(v(150.0, 10.0, 5.0), Quaternion::IDENTITY),
    );
    let end = Position::new(
        0x807F_0001,
        Frame::new(v(10.0, 10.0, 5.0), Quaternion::IDENTITY),
    );
    params.set_from_begin_and_end_position(&begin, &end);
    let mut terrain = flat();
    block(&mut terrain, 0x807F_FFFF, |_, _| 0.0);
    assert!(terrain.los_check(&mut params));
    assert_eq!(params.trace_result(), 0);
    assert_eq!(params.final_cell_id(), 0x807F_0001);
    assert!(
        near(params.end_point(), v(10.0, 10.0, 5.0)),
        "{:?}",
        params.end_point()
    );
}
