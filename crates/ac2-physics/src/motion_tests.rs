//! The motion events on a human body from the real DATs (`AC2_DAT_DIR`; skipped without): what
//! each `CPhysicsObj::RecvEvt_*` does to the body once the client has unpacked it.

use std::sync::Arc;

use ac2_anim::{Ac2AnimEvent, Ac2BehaviorParams};
use ac2_assets::{resolve_ac2_visual_motion_interp, resolve_ac2_visual_parts, Ac2SetupPhysics};
use ac2_core::DataId;
use ac2_dat::DatFileKind;
use ac2_db::{ResourceStore, TypedResourceRef};
use ac2_world::load_ac2_motion_interp_desc;

use crate::body::PhysicsBody;
use crate::frame::Frame;
use crate::loader::load_body_shape;
use crate::loader_anim::load_body_animation;
use crate::motion::{apply_motion, MotionEvent};
use crate::position::Position;
use crate::vector::{Quaternion, Vector};

/// The human visual (the double's player), and the motion table a visual without its own names.
const HUMAN: u32 = 0x1F00_0023;
const HUMAN_MOTION_INTERP_DESC: u32 = 0x3200_0000;
/// `MODE_DEFAULT` (peace) and a ground mode its behaviour table can switch to.
const PEACE: u32 = 0x4000_0001;
const OTHER_MODE: u32 = 0x4100_0009;
/// The `dead` mode and the `death` behaviour.
const DEAD: u32 = 0x4100_0005;
const DEATH: u32 = 0x4100_001F;
/// Behaviours of the peace mode on the ground.
const WAVE: u32 = 0x4000_000A;
const OTHER_BEHAVIOR: u32 = 0x4000_000B;
/// A slider the human's `CharTemplate` leaves to the client, and a network one.
const CLIENT_SLIDER: u32 = 0x4000_0001;
const NETWORK_SLIDER: u32 = 0x4000_000A;

fn at(cell: u32, origin: [f32; 3]) -> Position {
    Position::new(
        cell,
        Frame::new(
            Vector::new(origin[0], origin[1], origin[2]),
            Quaternion::new(1.0, 0.0, 0.0, 0.0),
        ),
    )
}

/// A human body as the client makes it (`ac2-smartbox`'s `load_body_data`): its collision
/// shape, motion table, radius and animation. The player's or another object's.
fn human(player: bool) -> Option<PhysicsBody> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = Arc::new(ResourceStore::new(dataset));
    let visual = DataId::new(HUMAN);
    let shape = load_body_shape(&store, visual)
        .expect("body setup decodes")
        .expect("the visual has a Setup");
    let radius = match resolve_ac2_visual_parts(&store, visual)
        .expect("visual parts decode")
        .first()
    {
        Some((setup, _)) => store
            .load_cached_packed::<Ac2SetupPhysics>(TypedResourceRef::new(
                DatFileKind::Portal,
                *setup,
                0,
            ))
            .expect("setup physics decodes")
            .map_or(0.0, |physics| physics.radius),
        None => 0.0,
    };
    let midesc = resolve_ac2_visual_motion_interp(&store, visual)
        .expect("motion interp decodes")
        .unwrap_or(DataId::new(HUMAN_MOTION_INTERP_DESC));
    let motion = load_ac2_motion_interp_desc(&store, midesc)
        .expect("motion interp desc decodes")
        .map(|desc| Arc::new((*desc).clone()));
    let animation = load_body_animation(&store, visual).expect("animation decodes");
    let mut body = PhysicsBody::new(at(0x0299_0109, [59.0, 240.0, -44.5]), shape, motion);
    body.set_player(player);
    body.set_radius(radius);
    body.set_animation(animation.expect("the human animates"));
    Some(body)
}

fn animate(body: &mut PhysicsBody, seconds: f32) {
    let mut random = ac2_core::Random::default();
    for step in 0..(seconds * 20.0) as u32 {
        body.update_animation(0.05, 1.0 + f64::from(step) * 0.05, &mut random);
    }
}

/// The behaviour a `DoBehavior` starts: `behavior`, entering `mode` first when given, and the
/// copy's context 1 (`RecvEvt_DoBehavior`).
fn behaviour(behavior: u32, mode: Option<u32>) -> Ac2BehaviorParams {
    let mut params = Ac2BehaviorParams::new(behavior);
    if let Some(mode) = mode {
        params.mode = mode;
    }
    params.context = 1;
    params
}

#[test]
fn modes_behaviours_and_layers_reach_the_body() {
    let Some(mut body) = human(false) else {
        return;
    };
    assert_eq!(body.mode(), PEACE);
    // `RecvEvt_SetMode`: at once.
    apply_motion(&mut body, &MotionEvent::SetMode(OTHER_MODE));
    assert_eq!(body.mode(), OTHER_MODE);
    // `RecvEvt_DoMode` -> `DoMode(0, mode)`: back to peace through the table's transitions.
    apply_motion(&mut body, &MotionEvent::DoMode(PEACE));
    animate(&mut body, 5.0);
    assert_eq!(body.mode(), PEACE);
    // `RecvEvt_DoBehavior` (bit 1: the behaviour), then `RecvEvt_StopBehavior`.
    apply_motion(&mut body, &MotionEvent::DoBehavior(behaviour(WAVE, None)));
    assert!(body.is_doing_behavior(WAVE));
    apply_motion(&mut body, &MotionEvent::StopBehavior(WAVE));
    animate(&mut body, 1.0);
    assert!(!body.is_doing_behavior(WAVE));
    // Bit 2 names the mode the behaviour enters first (`BehaviorParams::mModeID`).
    apply_motion(
        &mut body,
        &MotionEvent::DoBehavior(behaviour(OTHER_BEHAVIOR, Some(PEACE))),
    );
    assert!(body.is_doing_behavior(OTHER_BEHAVIOR));
    apply_motion(&mut body, &MotionEvent::StopAllWeenieBehaviors);
    apply_motion(&mut body, &MotionEvent::StopMoveToCancelsBehaviors);
    apply_motion(&mut body, &MotionEvent::StopLayer(0));
    animate(&mut body, 1.0);
    assert_eq!(body.mode(), PEACE);
}

/// The death presentation the server's `DoBehavior` starts (`gmbehaviors.emp` `death`, entering
/// `gmmodes.emp` `dead` first): the body rests in the `dead` mode once the behaviour has played,
/// and a `SetMode` of the default mode (the respawn) stands it up again.
#[test]
fn a_death_behavior_lays_the_body_in_the_dead_mode() {
    let Some(mut body) = human(false) else {
        return;
    };
    assert_eq!(body.mode(), PEACE);
    apply_motion(
        &mut body,
        &MotionEvent::DoBehavior(behaviour(DEATH, Some(DEAD))),
    );
    assert!(body.is_doing_behavior(DEATH));
    animate(&mut body, 6.0);
    assert_eq!(body.mode(), DEAD);
    apply_motion(&mut body, &MotionEvent::SetMode(PEACE));
    assert_eq!(body.mode(), PEACE);
}

#[test]
fn the_player_keeps_its_client_sliders_and_others_take_every_slider() {
    let (Some(mut remote), Some(mut player)) = (human(false), human(true)) else {
        return;
    };
    let slide = |slider| MotionEvent::DoSlider {
        slider,
        value: 0.75,
        time: 0.0,
    };
    for body in [&mut remote, &mut player] {
        apply_motion(body, &slide(CLIENT_SLIDER));
        apply_motion(body, &slide(NETWORK_SLIDER));
    }
    assert_eq!(remote.slider_desired_value(CLIENT_SLIDER), 0.75);
    assert_eq!(remote.slider_desired_value(NETWORK_SLIDER), 0.75);
    assert_ne!(player.slider_desired_value(CLIENT_SLIDER), 0.75);
    assert_eq!(player.slider_desired_value(NETWORK_SLIDER), 0.75);
}

/// `ECM_Physics::RecvEvt_SetVelocityScale`, `SetAccelerationScale`, `SetJumpScale` and
/// `SetHoverHeight` reach the body's tuning; a negative jump scale is refused.
#[test]
fn the_server_tunes_the_body_scales_and_hover_height() {
    let Some(mut body) = human(false) else {
        return;
    };
    apply_motion(&mut body, &MotionEvent::SetVelocityScale(0.5));
    apply_motion(&mut body, &MotionEvent::SetAccelerationScale(0.25));
    apply_motion(&mut body, &MotionEvent::SetJumpScale(2.0));
    apply_motion(&mut body, &MotionEvent::SetHoverHeight(1.5));
    let tuning = body.tuning();
    assert_eq!(tuning.velocity_scale, 0.5);
    assert_eq!(tuning.acceleration_scale, 0.25);
    assert_eq!(tuning.jump_scale, 2.0);
    assert_eq!(tuning.hover_height, 1.5);
    apply_motion(&mut body, &MotionEvent::SetJumpScale(-1.0));
    assert_eq!(body.tuning().jump_scale, 2.0);
}

/// `ECM_Physics::RecvEvt_DoFX` and `StopFX` start and stop the effect's sounds: the body's
/// animation events carry them to the client's FX manager.
#[test]
fn a_server_fx_reaches_the_body_as_fx_events() {
    let Some(mut body) = human(false) else {
        return;
    };
    apply_motion(
        &mut body,
        &MotionEvent::DoFx {
            fx: 0x4000_0010,
            scalar: 1.0,
        },
    );
    apply_motion(&mut body, &MotionEvent::StopFx(0x4000_0010));
    let events = body.take_animation_events();
    assert!(events.iter().any(|e| matches!(
        e,
        Ac2AnimEvent::StartFx { fx: 0x4000_0010, intensity } if *intensity == 1.0
    )));
    assert!(events
        .iter()
        .any(|e| matches!(e, Ac2AnimEvent::StopFx { fx: 0x4000_0010 })));
}

/// A behaviour that locks actions (`BehaviorParams::lock_actions`) emits the player's
/// `LockActions` when it starts; the client's movement module follows that event
/// (`client_audio::animation_sounds`), and `UnlockActions` ends it.
#[test]
fn a_locking_behaviour_on_the_player_emits_lock_events() {
    let Some(mut player) = human(true) else {
        return;
    };
    let mut locking = behaviour(WAVE, None);
    locking.lock_actions = true;
    apply_motion(&mut player, &MotionEvent::DoBehavior(locking));
    animate(&mut player, 1.0);
    let events = player.take_animation_events();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Ac2AnimEvent::LockActions)),
        "{events:?}"
    );
}

#[test]
fn a_change_of_element_marks_the_body_for_its_held_children() {
    const WATER: u32 = 0x4000_0003;
    let Some(mut body) = human(false) else {
        return;
    };
    // `m_bElementChanged` starts set; `PrepareObjectForDrawing` clears it.
    assert!(body.element_changed());
    body.clear_element_changed();
    // `CPhysicsObj::DoElement` marks only an element other than the current one.
    let current = body.current_element();
    body.with_object(|object| object.do_element(current));
    assert!(!body.element_changed());
    body.with_object(|object| object.do_element(WATER));
    assert!(body.element_changed());
    assert_eq!(body.desired_element(), WATER);
}
