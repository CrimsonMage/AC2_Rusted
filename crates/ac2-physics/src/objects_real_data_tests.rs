//! Real-data checks of static-object collision; they run only when `AC2_DAT_DIR` points at the
//! DATs. Expectations are invariants (nothing recorded from the data).

use std::sync::Arc;

use ac2_core::{CellId, DataId};
use ac2_db::ResourceStore;
use ac2_world::{load_ac2_motion_interp_desc, Ac2WorldSource};

use crate::body::{MovementInput, PhysicsBody, PHYSICS_QUANTUM};
use crate::cells::CellSource;
use crate::fpu::{ld, st};
use crate::frame::Frame;
use crate::loader::{load_block_objects, load_body_shape, load_terrain_block};
use crate::objects::{StaticObject, HAS_PHYSICS_BSP_PS};
use crate::physics_obj_place::{SetPositionError, SLIDE_SPF};
use crate::position::Position;
use crate::terrain::Terrain;
use crate::vector::{Quaternion, Sphere, Vector};

const HUMAN_MOTION_INTERP_DESC: u32 = 0x3200_0000;
const HUMAN_MALE_VISUAL: u32 = 0x1F00_0023;
/// A landblock whose `CLandBlockInfo` places physics-BSP entities.
const BLOCK: u32 = 0x3A79_0000;

struct World {
    store: Arc<ResourceStore>,
    terrain: Terrain,
    objects: Vec<u32>,
}

fn world() -> Option<World> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = Arc::new(ResourceStore::new(Arc::clone(&dataset)));
    let source = Ac2WorldSource::open(Arc::clone(&store))
        .expect("world source opens")
        .expect("AC2 landscape");
    let mut terrain = Terrain::new();
    let (bx, by) = (BLOCK >> 24, (BLOCK >> 16) & 0xFF);
    for dx in -1..=1_i32 {
        for dy in -1..=1_i32 {
            let block = ((bx as i32 + dx) as u32) << 24 | ((by as i32 + dy) as u32) << 16;
            load_terrain_block(&source, &mut terrain, CellId::new(block | 0xFFFF))
                .expect("landblock decodes");
        }
    }
    let added = load_block_objects(&source, &mut terrain, CellId::new(BLOCK | 0xFFFF))
        .expect("block objects load");
    let objects = (0..added as u32).collect();
    Some(World {
        store,
        terrain,
        objects,
    })
}

/// The body's spheres in the object's mesh space (`SPHEREPATH::cache_localspace_sphere`).
fn local_spheres(body: &PhysicsBody, spheres: &[Sphere], object: &StaticObject) -> Vec<Sphere> {
    let position = body.position();
    let inverse = Vector::new(
        st(1.0 / ld(object.scale.x)),
        st(1.0 / ld(object.scale.y)),
        st(1.0 / ld(object.scale.z)),
    );
    spheres
        .iter()
        .map(|sphere| {
            let c = object.position.local_to_local(&position, sphere.center);
            Sphere::new(
                Vector::new(c.x * inverse.x, c.y * inverse.y, c.z * inverse.z),
                sphere.radius * inverse.z,
            )
        })
        .collect()
}

#[test]
fn landblock_objects_load_with_their_shapes_and_shadow_cells() {
    let Some(world) = world() else {
        return;
    };
    assert!(!world.objects.is_empty());
    let mut bsp = 0;
    for &id in &world.objects {
        let object = world.terrain.object(id).expect("object is resident");
        assert!(object.containment_radius > 0.0, "{id}");
        if object.state & HAS_PHYSICS_BSP_PS != 0 {
            bsp += 1;
        }
        let cell = world
            .terrain
            .land_cell(object.position.cell_id)
            .expect("the object's own cell is resident");
        assert!(cell.objects.contains(&id), "{id} shadows its own cell");
    }
    assert!(bsp > 0, "block {BLOCK:08x} places physics-BSP objects");
}

#[test]
fn walking_at_physics_bsp_objects_never_enters_their_polygons() {
    let Some(world) = world() else {
        return;
    };
    let desc = load_ac2_motion_interp_desc(&world.store, DataId::new(HUMAN_MOTION_INTERP_DESC))
        .expect("motion interp desc decodes")
        .expect("human motion interp desc");
    let desc: Arc<_> = Arc::new((*desc).clone());
    let shape = load_body_shape(&world.store, DataId::new(HUMAN_MALE_VISUAL))
        .expect("body setup decodes")
        .expect("human body setup");
    let spheres: Vec<Sphere> = shape
        .spheres
        .iter()
        .map(|s| {
            Sphere::new(
                Vector::new(s.center.x, s.center.y, s.center.z),
                s.radius * 0.5,
            )
        })
        .collect();
    let (mut checked, mut deflected) = (0, 0);
    for &id in &world.objects {
        let object = world.terrain.object(id).expect("object");
        if object.state & HAS_PHYSICS_BSP_PS == 0 || checked >= 4 {
            continue;
        }
        let target = object.position;
        let mut start = target;
        start.frame = Frame::new(
            Vector::new(target.frame.origin.x, target.frame.origin.y - 8.0, 0.0),
            Quaternion::IDENTITY,
        );
        let mut cell = start.cell_id;
        let mut origin = start.frame.origin;
        if !crate::landdefs::adjust_to_outside(&mut cell, &mut origin) {
            continue;
        }
        let probe = Position::new(cell, Frame::new(origin, Quaternion::IDENTITY));
        let Some(depth) = world.terrain.height_above_terrain(&probe) else {
            continue;
        };
        origin.z = origin.z - depth + 0.5;
        let start = Position::new(cell, Frame::new(origin, Quaternion::IDENTITY));
        let mut body = PhysicsBody::new(start, shape.clone(), Some(Arc::clone(&desc)));
        for _ in 0..10 {
            body.step_quantum(&world.terrain);
        }
        body.set_input(MovementInput {
            motion: [0.0, 1.0, 0.0],
            heading: Some(0.0),
            turn: 0.0,
        });
        for step in 0..(5.0 / PHYSICS_QUANTUM) as usize {
            body.step_quantum(&world.terrain);
            for tree in &object.shape.bsps {
                for sphere in local_spheres(&body, &spheres, object) {
                    let inside = tree.polygons.iter().any(|p| p.hits_sphere(&sphere));
                    assert!(
                        !inside,
                        "object {id} step {step}: {sphere:?} at {:?}",
                        body.position()
                    );
                }
            }
        }
        let end = body.position().frame.origin;
        if (end.x - start.frame.origin.x).abs() > 0.1 {
            deflected += 1;
        }
        checked += 1;
    }
    assert!(checked > 0);
    assert!(
        deflected > 0,
        "walking at the objects slid along at least one"
    );

    // Placing the body at each object's origin with sliding leaves it clear of the polygons.
    let mut placed = 0;
    for &id in &world.objects {
        let object = world.terrain.object(id).expect("object");
        if object.state & HAS_PHYSICS_BSP_PS == 0 {
            continue;
        }
        let target = Position::new(
            object.position.cell_id,
            Frame::new(object.position.frame.origin, Quaternion::IDENTITY),
        );
        let mut body = PhysicsBody::new(target, shape.clone(), Some(Arc::clone(&desc)));
        if body.place(target, SLIDE_SPF, &world.terrain) != SetPositionError::Ok {
            continue;
        }
        for tree in &object.shape.bsps {
            for sphere in local_spheres(&body, &spheres, object) {
                let inside = tree.polygons.iter().any(|p| p.hits_sphere(&sphere));
                assert!(!inside, "placed into object {id}: {:?}", body.position());
            }
        }
        placed += 1;
    }
    assert!(placed > 0);
}
