//! Real-data animation checks (only with `AC2_DAT_DIR`): the human male body loads its
//! behaviour table, animators and Setup, idles, walks and jumps through the mode tracker, and
//! its posed bones stay finite. Every expectation is an invariant of the data.

use std::sync::Arc;

use ac2_core::{CellId, DataId};
use ac2_db::ResourceStore;
use ac2_world::{load_ac2_motion_interp_desc, Ac2WorldSource};

use crate::body::{PhysicsBody, PHYSICS_QUANTUM};
use crate::frame::Frame;
use crate::landdefs::lcoord_to_gid;
use crate::loader::{load_body_shape, load_terrain_block};
use crate::loader_anim::load_body_animation;
use crate::motion_interp::{ELEMENT_AIR, ELEMENT_GROUND};
use crate::position::Position;
use crate::terrain::Terrain;
use crate::vector::{Quaternion, Vector};

const HUMAN_MOTION_INTERP_DESC: u32 = 0x3200_0000;
const HUMAN_MALE_VISUAL: u32 = 0x1F00_0023;
const IDLE: u32 = 0x4000_0002;
const MOVE: u32 = 0x4000_0001;
const STANDING_JUMP: u32 = 0x4000_000a;

fn scene() -> Option<(Terrain, PhysicsBody)> {
    scene_for(HUMAN_MALE_VISUAL)
}

/// A body of `visual` (with the human's motion interp description) standing on the test block.
pub(crate) fn scene_for(visual: u32) -> Option<(Terrain, PhysicsBody)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = Arc::new(ResourceStore::new(Arc::clone(&dataset)));
    let world = Ac2WorldSource::open(Arc::clone(&store))
        .expect("world source opens")
        .expect("AC2 landscape");
    let mut terrain = Terrain::new();
    load_terrain_block(&world, &mut terrain, CellId::new(0x7F7F_FFFF)).expect("block decodes");
    let desc = load_ac2_motion_interp_desc(&store, DataId::new(HUMAN_MOTION_INTERP_DESC))
        .expect("motion interp desc decodes")
        .expect("human motion interp desc");
    let shape = load_body_shape(&store, DataId::new(visual))
        .expect("setup decodes")
        .expect("human setup");
    let animation = load_body_animation(&store, DataId::new(visual))
        .expect("animation decodes")
        .expect("animation");
    assert!(!animation.animators.is_empty());
    assert!(animation.setup.is_some());
    let cell = lcoord_to_gid(0x7F * 8 + 4, 0x7F * 8 + 4);
    let ground = terrain
        .height_above_terrain(&Position::new(
            cell,
            Frame::new(Vector::new(80.0, 80.0, 0.0), Quaternion::IDENTITY),
        ))
        .map_or(0.0, |height| -height);
    let start = Position::new(
        cell,
        Frame::new(Vector::new(80.0, 80.0, ground + 0.5), Quaternion::IDENTITY),
    );
    let mut body = PhysicsBody::new(start, shape, Some(Arc::new((*desc).clone())));
    body.set_player(true);
    body.set_animation(animation);
    Some((terrain, body))
}

pub(crate) fn run(terrain: &Terrain, body: &mut PhysicsBody, seconds: f32) {
    for _ in 0..(seconds / PHYSICS_QUANTUM).round() as usize {
        body.step_quantum(terrain);
        crate::test_support::animate(body, PHYSICS_QUANTUM);
    }
}

fn assert_finite(body: &PhysicsBody) {
    let parts = body.posed_parts();
    assert!(!parts.is_empty());
    for part in parts {
        assert!(
            part.offset.is_finite() && part.rotation.is_finite() && part.scale.is_finite(),
            "{part:?}"
        );
    }
}

#[test]
fn a_human_idles_walks_and_jumps_through_its_behaviour_table() {
    let Some((terrain, mut body)) = scene() else {
        return;
    };
    run(&terrain, &mut body, 2.0);
    assert_eq!(body.current_element(), ELEMENT_GROUND);
    assert!(body.is_doing_behavior(IDLE), "the table's idle plays");
    assert_finite(&body);
    let idle = body.posed_parts();
    body.do_motion([0.0, 1.0, 0.0]);
    run(&terrain, &mut body, 2.0);
    assert!(body.is_doing_behavior(MOVE));
    assert_finite(&body);
    assert_ne!(body.posed_parts(), idle, "walking moves the bones");
    body.do_motion([0.0, 0.0, 0.0]);
    run(&terrain, &mut body, 2.0);
    body.take_animation_events();
    assert!(body.jump());
    // Standing, the jump is the jump behaviour; its animation's jump impulse jumps.
    assert!(body.is_doing_behavior(STANDING_JUMP));
    let mut waited = 0;
    while body.movement_state().times_jumped == 0 && waited < 30 {
        run(&terrain, &mut body, PHYSICS_QUANTUM);
        waited += 1;
    }
    assert_eq!(body.movement_state().times_jumped, 1, "the impulse jumped");
    assert_eq!(body.movement_state().element, ELEMENT_AIR);
    let mut reached_air = false;
    for _ in 0..30 {
        run(&terrain, &mut body, PHYSICS_QUANTUM);
        reached_air |= body.current_element() == ELEMENT_AIR;
        assert_finite(&body);
    }
    assert!(reached_air, "the animation reached the air element");
    assert_eq!(body.current_element(), ELEMENT_GROUND, "landed again");
}
