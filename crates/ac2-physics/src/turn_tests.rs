//! The player's turning improvements over retail (`AGENTS.md`), on a human body from the real
//! DATs (`AC2_DAT_DIR`; skipped without): it turns in the air with the ground's turn values,
//! and its drawn heading leads to where the next quantum turns it.

use std::sync::Arc;

use ac2_core::DataId;
use ac2_db::ResourceStore;
use ac2_world::load_ac2_motion_interp_desc;

use crate::body::PhysicsBody;
use crate::frame::Frame;
use crate::loader::load_body_shape;
use crate::position::Position;
use crate::terrain::Terrain;
use crate::vector::{Quaternion, Vector};

const HUMAN: u32 = 0x1F00_0023;
const HUMAN_MOTION_INTERP_DESC: u32 = 0x3200_0000;
const ELEMENT_AIR: u32 = 0x4000_0002;

/// A human body high above an empty world: it falls, in the air, the whole test.
fn falling_human(player: bool) -> Option<PhysicsBody> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = Arc::new(ResourceStore::new(dataset));
    let shape = load_body_shape(&store, DataId::new(HUMAN))
        .expect("body setup decodes")
        .expect("the visual has a Setup");
    let motion = load_ac2_motion_interp_desc(&store, DataId::new(HUMAN_MOTION_INTERP_DESC))
        .expect("motion interp desc decodes")
        .map(|desc| Arc::new((*desc).clone()));
    let at = Position::new(
        0x7F7F_001C,
        Frame::new(
            Vector::new(70.0, 70.0, 5000.0),
            Quaternion::new(1.0, 0.0, 0.0, 0.0),
        ),
    );
    let mut body = PhysicsBody::new(at, shape, motion);
    body.set_player(player);
    Some(body)
}

fn heading(position: &Position) -> f64 {
    f64::from(position.frame.heading().degrees())
}

/// The DAT's air element has no turn acceleration: retail's body cannot turn mid-jump. The
/// player turns there with the ground's values; another body still does not.
#[test]
fn the_player_turns_in_the_air_and_another_body_does_not() {
    let (Some(mut player), Some(mut other)) = (falling_human(true), falling_human(false)) else {
        return;
    };
    let terrain = Terrain::new();
    for body in [&mut player, &mut other] {
        body.step_quantum(&terrain);
        assert_eq!(body.desired_element(), ELEMENT_AIR);
        body.set_turn_rate(1.0);
        let before = heading(&body.position());
        for _ in 0..5 {
            body.step_quantum(&terrain);
        }
        let turned = (heading(&body.position()) - before).abs();
        if body.is_player() {
            assert!(turned > 10.0, "the player turned {turned} degrees");
        } else {
            assert!(turned < 1e-3, "another body turned {turned} degrees");
        }
    }
}

/// The drawn heading a whole quantum on is where the next quantum puts it, from a standing
/// start (the press) and while turning.
#[test]
fn the_led_heading_lands_where_the_next_quantum_turns() {
    let Some(mut body) = falling_human(true) else {
        return;
    };
    let terrain = Terrain::new();
    body.step_quantum(&terrain);
    body.set_turn_rate(1.0);
    for step in 0..4 {
        let lead = heading(&body.led_drawing_position(1.0));
        let start = heading(&body.led_drawing_position(0.0));
        assert!((start - heading(&body.position())).abs() < 1e-3);
        body.step_quantum(&terrain);
        let landed = heading(&body.position());
        let mut miss = (lead - landed).abs();
        if miss > 180.0 {
            miss = 360.0 - miss;
        }
        assert!(miss < 0.05, "step {step}: drawn {lead}, landed {landed}");
        assert!(
            (lead - start).abs() > 1e-3,
            "step {step}: the press shows at once"
        );
    }
}
