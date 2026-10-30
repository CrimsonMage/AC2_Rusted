use super::*;

/// Temporary ids, each call's distinct from every other's (`AllocateTemporaryID`).
fn ids() -> impl FnMut() -> Option<u64> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(crate::temporary_ids::TEMPORARY_BASE);
    || Some(NEXT.fetch_add(1, Ordering::Relaxed))
}

fn vdesc_node(lifetime: f32) -> Ac2FxObject {
    Ac2FxObject {
        vdesc: 0x1F00_0001,
        lifetime,
        ..Ac2FxObject::default()
    }
}

/// `DoCreateObjectFX` hands the object to the FX; a stop fades it out and destroys it three
/// seconds on, and the FX's record goes.
#[test]
fn a_stopped_fx_fades_and_destroys_the_objects_it_created() {
    let mut fx = FxObjects::default();
    fx.start(7, 9, &[vdesc_node(0.0)], 10.0, 1.0, &|_| None, &mut ids());
    let pending = fx.take_pending();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].emitter, 7);
    let id = pending[0].id;
    fx.stop(7, 9, &[], 11.0);
    assert!(fx.record(7, 9).is_none());
    assert!(fx.step(11.25).is_empty(), "the fade runs first");
    assert!((fx.looks().tints[&id][3] - 0.5).abs() < 1e-6, "half faded");
    assert!(fx.step(13.9).is_empty());
    assert_eq!(
        fx.step(14.0),
        vec![id],
        "destroyed three seconds after the fade began"
    );
}

/// A startup FX is not stopped by `StopFX` (`m_bStartupFX` without the force flag).
/// `DoCreateObjectFX`'s alpha moves: an instant clear, then a fade over the ramp (0.25 s for
/// a zero ramp).
#[test]
fn a_created_object_fades_in_over_its_ramp_time() {
    assert_eq!(create_alpha_moves(0.0), [(0.0, 0.0), (1.0, 0.25)]);
    assert_eq!(create_alpha_moves(0.5), [(0.0, 0.0), (1.0, 0.5)]);
    let mut fx = FxObjects::default();
    let node = Ac2FxObject {
        ramp_time: 2.0,
        ..vdesc_node(0.0)
    };
    fx.start(7, 9, &[node], 0.0, 1.0, &|_| None, &mut ids());
    let pending = fx.take_pending();
    assert_eq!(pending[0].ramp_time, 2.0);
}

#[test]
fn a_startup_fx_keeps_running_through_stop() {
    let mut fx = FxObjects::default();
    fx.start_startup(7, 9, &[vdesc_node(0.0)], &mut ids());
    fx.stop(7, 9, &[], 1.0);
    assert!(fx.record(7, 9).is_some_and(|record| record.startup));
    assert!(fx.step(10.0).is_empty());
}

/// `m_hide_target`: the created object is hidden until the FX stops, then it shows (its
/// fade keeps it out of the frame only once it is destroyed).
#[test]
fn a_hidden_target_shows_again_when_the_fx_stops() {
    let mut fx = FxObjects::default();
    let node = Ac2FxObject {
        hide_target: true,
        ..vdesc_node(0.0)
    };
    fx.start(7, 9, &[node], 0.0, 1.0, &|_| None, &mut ids());
    let id = fx.take_pending()[0].id;
    assert!(fx.looks().hidden.contains(&id));
    fx.stop(7, 9, &[], 1.0);
    assert!(!fx.looks().hidden.contains(&id));
}

/// `DoColorFX` ramps the emitter's colour, and `StopFX` restores a node that asks for it.
#[test]
fn a_colour_ramps_and_restores_on_stop() {
    let mut fx = FxObjects::default();
    let node = Ac2FxObject {
        color: Some([1.0, 0.0, 0.0, 1.0]),
        ramp_time: 1.0,
        restore: true,
        ..Ac2FxObject::default()
    };
    fx.start(7, 9, &[node], 0.0, 1.0, &|_| None, &mut ids());
    assert_eq!(fx.colour(7, 0.5), Some([1.0, 0.5, 0.5, 1.0]));
    fx.stop(7, 9, &[node], 2.0);
    assert_eq!(fx.colour(7, 2.05), Some([1.0, 0.5, 0.5, 1.0]));
    assert_eq!(fx.colour(7, 2.2), None, "back to white");
}

/// An object whose alpha slider is at 0 draws clear; white animation leaves the FX colour.
#[test]
fn the_animation_scale_multiplies_the_fx_colour() {
    assert_eq!(
        drawn_tint([1.0, 0.5, 0.25, 1.0], [1.0; 4]),
        [1.0, 0.5, 0.25, 1.0]
    );
    assert_eq!(
        drawn_tint([1.0; 4], [0.5, 1.0, 1.0, 0.0]),
        [0.5, 1.0, 1.0, 0.0]
    );
    assert_eq!(drawn_tint([1.0; 4], [1.0, 1.0, 1.0, 0.25])[3], 0.25);
}

/// `FXManager::GetCurrentFX` lists the running FX that are not startup FX, and
/// `FXManager::DestroyObject` fades only temporary ids.
#[test]
fn current_fx_skips_startup_fx_and_destroy_object_wants_a_temporary_id() {
    let mut fx = FxObjects::default();
    let node = vdesc_node(10.0);
    fx.start(1, 5, &[node], 0.0, 1.0, &|_| None, &mut ids());
    fx.start(1, 3, &[node], 0.0, 1.0, &|_| None, &mut ids());
    fx.start_startup(1, 4, &[node], &mut ids());
    let listed: Vec<u32> = fx.current_fx(1).into_iter().map(|(id, _)| id).collect();
    assert_eq!(listed, vec![3, 5]);
    assert!(fx.current_fx(2).is_empty());
    assert!(!fx.destroy_object(0x8000_0000_0000_0001, false, 1.0));
    assert!(fx.fades.is_empty());
    let object = fx.record(1, 5).expect("the FX runs").objects[0];
    assert!(fx.destroy_object(object, true, 1.0));
    assert!(fx.fades.contains_key(&object));
}

/// `FXManager::DoTargettedFX`'s duration: the last node decides. An object node lasts its
/// lifetime (15 s by default), a looping system or a target FX never ends, a removing
/// appearance key lasts 0.1 s, an FX script its period, and anything else ends at once.
#[test]
fn nodes_play_by_their_gates() {
    let gated = |player_only, day_time, night_time| Ac2FxObject {
        gate: ac2_assets::Ac2FxNodeGate {
            player_only,
            day_time,
            night_time,
            has_channels: false,
        },
        ..Ac2FxObject::default()
    };
    let nodes = [gated(true, true, true), gated(false, true, false)];
    assert_eq!(gated_nodes(&nodes, false, false).len(), 1);
    assert_eq!(gated_nodes(&nodes, true, false).len(), 2);
    assert_eq!(gated_nodes(&nodes, true, true).len(), 1);
}

#[test]
fn an_fx_ends_by_its_last_nodes_duration() {
    let end_of = |node: Ac2FxObject, script: Option<f32>| {
        let mut fx = FxObjects::default();
        fx.start(1, 2, &[node], 10.0, 1.0, &|_| script, &mut ids());
        fx.record(1, 2).map(|record| record.end)
    };
    // An object for its lifetime; a zero lifetime falls to `CalcDuration`'s end at once.
    assert_eq!(end_of(vdesc_node(3.0), None), Some(Some(13.0)));
    assert_eq!(end_of(vdesc_node(0.0), None), Some(Some(10.0)));
    assert_eq!(end_of(vdesc_node(-1.0), None), Some(None));
    // A looping system, a target FX or a kept appearance never end; a removing key does.
    let looping = Ac2FxObject {
        particle_looping: true,
        color: Some([1.0; 4]),
        ..Ac2FxObject::default()
    };
    assert_eq!(end_of(looping, None), Some(None));
    let appearance = |scalar| Ac2FxObject {
        appearance: 0x2500_0010,
        appearance_key: (3, scalar),
        ..Ac2FxObject::default()
    };
    assert_eq!(end_of(appearance(0.5), None), Some(None));
    assert_eq!(
        end_of(appearance(-1.0), None),
        Some(Some(10.0 + f64::from(0.1f32)))
    );
    // An FX script's period.
    let script = Ac2FxObject {
        fx_script: 0x5700_0001,
        color: Some([1.0; 4]),
        ..Ac2FxObject::default()
    };
    assert_eq!(end_of(script, Some(2.5)), Some(Some(12.5)));
}

/// `FXManager::DoSliderFX`: a slider node asks its emitter's slider (and with `m_prop_color`
/// its children's) to move to the intensity over the ramp time, once per new FX.
#[test]
fn a_slider_node_asks_for_its_slider_move() {
    let mut fx = FxObjects::default();
    let node = Ac2FxObject {
        slider: ac2_assets::Ac2FxSlider {
            id: 0x4000_001A,
            intensity: 0.0,
            propagate: true,
        },
        ramp_time: 0.5,
        ..Ac2FxObject::default()
    };
    fx.start(7, 0x8100_028E, &[node], 0.0, 1.0, &|_| None, &mut ids());
    // A running FX only takes the scalar: no second move.
    fx.start(7, 0x8100_028E, &[node], 1.0, 1.0, &|_| None, &mut ids());
    assert_eq!(
        fx.take_sliders(),
        vec![SliderMove {
            emitter: 7,
            slider: 0x4000_001A,
            scalar: 0.0,
            ramp_time: 0.5,
            propagate: true,
        }]
    );
    assert!(fx.take_sliders().is_empty());
}

/// `FXManager::DoAppearanceFX`: an appearance node applies its key to the emitter's effects
/// description at the key's scalar times the FX's, a running FX re-scaled to nothing removes
/// it, and `StopFX` removes it.
#[test]
fn an_appearance_node_applies_its_key_until_the_fx_stops() {
    let mut fx = FxObjects::default();
    let node = Ac2FxObject {
        appearance: 0x2500_0010,
        appearance_key: (0x1000_0003, 0.5),
        ..Ac2FxObject::default()
    };
    let applied = |fx: &FxObjects| {
        fx.appearance_effects(7).and_then(|e| {
            e.applied_scalar(ac2_physics::GLOBAL_PART_GROUP, 0x2500_0010, 0x1000_0003)
        })
    };
    fx.start(7, 9, &[node], 0.0, 2.0, &|_| None, &mut ids());
    assert_eq!(applied(&fx), Some(1.0), "0.5 times the FX's 2.0");
    // The running FX taking a zero scalar removes it (`DoTargettedFX`'s update branch).
    fx.start(7, 9, &[node], 0.0, 0.0, &|_| None, &mut ids());
    assert_eq!(applied(&fx), None);
    fx.start(7, 9, &[node], 0.0, 1.0, &|_| None, &mut ids());
    assert_eq!(applied(&fx), Some(0.5));
    fx.stop(7, 9, &[node], 1.0);
    assert_eq!(applied(&fx), None, "StopFX removes it");
    assert!(fx.appearance_effects(7).is_none());
}

/// `FXManager::UseTime`: a ramping node starts its appearance removed (the new FX's current
/// scalar is 0), and each step moves the current scalar `quantum / m_ramp_time` toward the
/// desired one and applies the appearance again at it; a lower desired scalar ramps it down.
#[test]
fn a_ramping_appearance_follows_the_fx_scalar() {
    let mut fx = FxObjects::default();
    let node = Ac2FxObject {
        appearance: 0x2500_0010,
        appearance_key: (0x1000_0003, 1.0),
        ramp_time: 2.0,
        ..Ac2FxObject::default()
    };
    let applied = |fx: &FxObjects| {
        fx.appearance_effects(7).and_then(|e| {
            e.applied_scalar(ac2_physics::GLOBAL_PART_GROUP, 0x2500_0010, 0x1000_0003)
        })
    };
    fx.start(7, 9, &[node], 0.0, 1.0, &|_| None, &mut ids());
    assert_eq!(applied(&fx), None, "a ramping node starts at 0");
    fx.use_time(0.5);
    assert_eq!(applied(&fx), Some(0.25));
    fx.use_time(1.0);
    assert_eq!(applied(&fx), Some(0.75));
    fx.use_time(1.0);
    assert_eq!(applied(&fx), Some(1.0), "held at the desired scalar");
    // A lower scalar on the running FX ramps down; the ramp keeps the current one.
    fx.start(7, 9, &[node], 0.0, 0.0, &|_| None, &mut ids());
    assert_eq!(applied(&fx), Some(1.0));
    fx.use_time(1.0);
    assert_eq!(applied(&fx), Some(0.5));
    fx.use_time(1.0);
    assert_eq!(applied(&fx), None, "ramped to nothing: removed");
}

/// `FXManager::CalcDuration`: a node whose sound loops never ends by itself, like a looping
/// particle system; one that does not ends at once.
#[test]
fn a_looping_sound_keeps_the_fx_running() {
    let end_of = |sound_looping| {
        let mut fx = FxObjects::default();
        let node = Ac2FxObject {
            color: Some([1.0; 4]),
            sound: 0x2A00_0001,
            sound_looping,
            ..Ac2FxObject::default()
        };
        fx.start(1, 2, &[node], 10.0, 1.0, &|_| None, &mut ids());
        fx.record(1, 2).map(|record| record.end)
    };
    assert_eq!(end_of(true), Some(None));
    assert_eq!(end_of(false), Some(Some(10.0)));
}

fn targeted(id: u64) -> FxTarget {
    FxTarget {
        id,
        ..FxTarget::default()
    }
}

fn start_on(fx: &mut FxObjects, target: &FxTarget, node: Ac2FxObject) {
    fx.start_targeted(1, 2, &[node], 0.0, 0.75, target, &|_| None, &mut ids());
}

/// `FXManager::GetCurrentFX` keeps each FX's scalar and target (`FXScalarAndTarget`), and a
/// running FX started again takes the new target with the new desired scalar.
#[test]
fn current_fx_keeps_the_scalar_and_target() {
    let mut fx = FxObjects::default();
    let node = Ac2FxObject {
        color: Some([1.0; 4]),
        lifetime: -1.0,
        vdesc: 0,
        appearance: 5,
        appearance_key: (1, 1.0),
        ..Ac2FxObject::default()
    };
    start_on(&mut fx, &targeted(0x40), node);
    assert_eq!(
        fx.current_fx(1),
        vec![(
            2,
            FxScalarAndTarget {
                scalar: 0.75,
                target: 0x40
            }
        )]
    );
    fx.start_targeted(
        1,
        2,
        &[node],
        1.0,
        0.25,
        &targeted(0x41),
        &|_| None,
        &mut ids(),
    );
    let (_, listed) = fx.current_fx(1)[0];
    assert_eq!(listed.target, 0x41);
    assert_eq!(fx.record(1, 2).map(|r| r.desired), Some(0.25));
}

/// `DoTargetAndCloningFXs`: the target hides and shows again, takes the node's target FX
/// with the FX's current scalar, and is destroyed only with a temporary id.
#[test]
fn a_node_acts_on_the_fxs_target() {
    let mut fx = FxObjects::default();
    let node = Ac2FxObject {
        hide_target: true,
        target_fx_id: 0x77,
        destroy_target: true,
        ..Ac2FxObject::default()
    };
    start_on(&mut fx, &targeted(0x40), node);
    assert!(fx.looks().hidden.contains(&0x40));
    assert_eq!(
        fx.take_target_calls(),
        vec![TargetCall::DoFx {
            object: 0x40,
            fx: 0x77,
            scalar: 0.75
        }]
    );
    assert!(fx.fades.is_empty(), "a server object is never destroyed");
    fx.stop(1, 2, &[], 1.0);
    assert!(!fx.looks().hidden.contains(&0x40));
    assert_eq!(
        fx.take_target_calls(),
        vec![TargetCall::StopFx {
            object: 0x40,
            fx: 0x77
        }],
        "the target's FX stops with the FX"
    );
    // A temporary object is faded and destroyed.
    let temp = crate::temporary_ids::TEMPORARY_BASE + 5;
    let mut fx = FxObjects::default();
    start_on(&mut fx, &targeted(temp), node);
    assert!(fx.fades.contains_key(&temp));
}

/// `m_parent` hangs the target from the holding point, `m_unparent` frees it.
#[test]
fn a_node_parents_and_unparents_the_target() {
    let mut fx = FxObjects::default();
    let node = Ac2FxObject {
        holding_location: 4,
        parent: true,
        unparent: true,
        ..Ac2FxObject::default()
    };
    start_on(&mut fx, &targeted(0x40), node);
    assert_eq!(
        fx.take_target_calls(),
        vec![
            TargetCall::Parent {
                object: 0x40,
                emitter: 1,
                attach: Ac2FxObjectAttach::Emitter
            },
            TargetCall::Unparent { object: 0x40 },
        ]
    );
    // Without a holding location nothing is parented.
    let node = Ac2FxObject {
        parent: true,
        ..Ac2FxObject::default()
    };
    start_on(&mut fx, &targeted(0x41), node);
    assert!(fx.take_target_calls().is_empty());
}

/// A retargeting node acts on a clone of the emitter's description, made with the cloned
/// appearance and kept on the FX; `m_conditional` acts on the target itself. The emitter
/// takes a made object as its FX target.
#[test]
fn a_retargeting_node_clones_the_emitters_description() {
    let mut fx = FxObjects::default();
    let target = FxTarget {
        id: 0x40,
        vdesc_to_clone: 0x1F00_0042,
        apr: 9,
        apr_keys: vec![(3, 0.5)],
    };
    let node = Ac2FxObject {
        retarget: true,
        hide_target: true,
        holding_location: 2,
        parent: true,
        target_fx_id: 0x77,
        ..Ac2FxObject::default()
    };
    start_on(&mut fx, &target, node);
    let pending = fx.take_pending();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].visual, 0x1F00_0042);
    let clone = pending[0].id;
    assert!(fx.record(1, 2).unwrap().objects.contains(&clone));
    assert!(
        fx.appearance_effects(clone).is_some(),
        "the cloned appearance"
    );
    assert_eq!(fx.take_new_targets(), vec![(1, clone)]);
    assert_eq!(
        fx.take_target_calls(),
        vec![TargetCall::DoFx {
            object: clone,
            fx: 0x77,
            scalar: 0.75
        }]
    );
    assert!(fx.looks().hidden.contains(&0x40), "the original hides");
    // Not parented, the clone has no place.
    let mut fx = FxObjects::default();
    let node = Ac2FxObject {
        retarget: true,
        ..Ac2FxObject::default()
    };
    start_on(&mut fx, &target, node);
    assert_eq!(fx.take_pending()[0].attach, Ac2FxObjectAttach::Unplaced);
    // Conditional: no clone, the target itself takes the call.
    let mut fx = FxObjects::default();
    let node = Ac2FxObject {
        retarget: true,
        conditional: true,
        target_fx_id: 0x77,
        ..Ac2FxObject::default()
    };
    start_on(&mut fx, &target, node);
    assert!(fx.take_pending().is_empty());
    assert!(matches!(
        fx.take_target_calls()[0],
        TargetCall::DoFx { object: 0x40, .. }
    ));
    // No description to clone, nothing is made.
    let mut fx = FxObjects::default();
    let node = Ac2FxObject {
        retarget: true,
        ..Ac2FxObject::default()
    };
    start_on(&mut fx, &targeted(0x40), node);
    assert!(fx.take_pending().is_empty());
}

/// `DoCreateObjectFX`: the made object is the FX's target and the emitter's.
#[test]
fn a_made_object_becomes_the_emitters_fx_target() {
    let mut fx = FxObjects::default();
    fx.start(7, 9, &[vdesc_node(0.0)], 0.0, 1.0, &|_| None, &mut ids());
    let id = fx.take_pending()[0].id;
    assert_eq!(fx.take_new_targets(), vec![(7, id)]);
    assert_eq!(fx.current_fx(7)[0].1.target, 0);
}
