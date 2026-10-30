//! An FX node's object runs its behaviour and deletes itself when it ends
//! (`FXManager::DoCreateObjectFX`'s `m_behavior` with `mDestroyOnCompletion`), on the real DATs
//! (`AC2_DAT_DIR`, skipped without): the object `0x1F000797` of FX `0x4100049A` (table
//! `0x1E000056`, behaviour `0x41000086`).

use std::sync::Arc;

use ac2_anim::{Ac2AnimEvent, Ac2BehaviorParams};
use ac2_core::Random;
use ac2_db::ResourceStore;
use ac2_physics::{Frame, PhysicsBody, Position, Quaternion, Vector};

use crate::world_data::{DatWorld, WorldData};

/// The visual the FX node creates and the behaviour it runs on it.
const OBJECT_VISUAL: u32 = 0x1F00_0797;
const BEHAVIOR: u32 = 0x4100_0086;

fn body(world: &DatWorld) -> Option<PhysicsBody> {
    let data = world.body(Some(OBJECT_VISUAL)).ok()?;
    let place = Position::new(
        0x0299_0109,
        Frame::new(
            Vector::new(59.0, 240.0, -44.5),
            Quaternion::new(1.0, 0.0, 0.0, 0.0),
        ),
    );
    let mut body = PhysicsBody::new(place, data.body.shape.clone(), data.body.motion.clone());
    body.set_radius(data.body.radius);
    body.set_animation(data.body.animation.clone()?);
    Some(body)
}

#[test]
fn a_created_objects_behaviour_deletes_the_object_when_it_ends() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let world = DatWorld::new(store).expect("world data opens");
    let mut body = body(&world).expect("the FX's object animates");
    // `DoCreateObjectFX`: the node's behaviour with destroy-on-completion and context 1.
    let mut params = Ac2BehaviorParams::new(BEHAVIOR);
    params.destroy_on_completion = true;
    params.context = 1;
    assert!(body.do_behavior(&params), "the behaviour starts");
    let mut random = Random::default();
    let mut deleted_at = None;
    let mut done_at = None;
    for step in 0..400 {
        let now = f64::from(step) * 0.05;
        body.update_animation(0.05, now, &mut random);
        for event in body.take_animation_events() {
            match event {
                Ac2AnimEvent::BehaviorDone { context: 1, .. } => {
                    done_at.get_or_insert(step);
                }
                Ac2AnimEvent::DeleteSelf { fade, delay } => {
                    assert_eq!((fade, delay), (0.0, 3.0));
                    assert!(deleted_at.replace(step).is_none(), "deleted once");
                }
                _ => {}
            }
        }
    }
    let deleted = deleted_at.expect("the object deletes itself");
    assert!(done_at.is_some_and(|done| done >= deleted));
}

#[test]
fn a_behaviour_without_the_parameter_never_deletes_its_object() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let world = DatWorld::new(Arc::new(ResourceStore::new(dataset))).expect("world data opens");
    let mut body = body(&world).expect("the FX's object animates");
    let mut params = Ac2BehaviorParams::new(BEHAVIOR);
    params.context = 1;
    assert!(body.do_behavior(&params));
    let mut random = Random::default();
    for step in 0..400 {
        body.update_animation(0.05, f64::from(step) * 0.05, &mut random);
        assert!(!body
            .take_animation_events()
            .iter()
            .any(|e| matches!(e, Ac2AnimEvent::DeleteSelf { .. })));
    }
}
