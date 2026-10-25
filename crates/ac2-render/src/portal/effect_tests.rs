//! `PortalSpace::Effect`'s phase machine against the decompile's constants: 1 s per phase,
//! the fade alpha, the field-of-view scales and the shutdown fader's channels.

use ac2_core::Random;

use super::{Phase, PortalSpace};

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-5
}

#[test]
fn a_wanted_effect_fades_in_for_a_second_then_runs_active() {
    let mut random = Random::default();
    let mut effect = PortalSpace::new();
    assert!(!effect.in_portal_space());
    effect.enter(false, 10.0, &mut random);
    assert!(effect.in_portal_space(), "wanted is in portal space");
    // Start (not instant) only sets the want; the phase starts at the next Think.
    assert_eq!(effect.phase(), Phase::Inactive);
    effect.think(10.0, &mut random);
    assert_eq!(effect.phase(), Phase::StartupPhase0);
    let draw = *effect.draw();
    assert!(draw.world && draw.background && draw.tunnels && draw.fade_only_world);
    assert!(close(draw.world_fov_scale, 1.0));
    assert!(close(draw.tunnel_visibility, 0.0));
    assert!(draw.fader);
    assert_eq!(draw.fader_color, [0.0, 0.0, 0.0, 0.0]);
    // Half a second in: the field of view narrows by 0.8 * 0.25 and the fades reach 0.5.
    effect.think(10.5, &mut random);
    let draw = *effect.draw();
    assert!(close(draw.world_fov_scale, 1.0 - 0.25 * 0.8));
    assert!(close(draw.background_opacity, 0.5));
    assert!(close(draw.fader_color[3], 0.5));
    // Past one second: active, the world no longer drawn, the tunnels at full visibility.
    effect.think(11.1, &mut random);
    assert_eq!(effect.phase(), Phase::Active);
    let draw = *effect.draw();
    assert!(!draw.world, "the world is not drawn in the tunnel");
    assert!(draw.tunnels && draw.background && !draw.fader);
    assert!(close(draw.tunnel_visibility, 1.0));
}

#[test]
fn an_instant_start_is_active_at_once() {
    let mut random = Random::default();
    let mut effect = PortalSpace::new();
    effect.enter(true, 5.0, &mut random);
    assert_eq!(effect.phase(), Phase::Active);
    assert!(effect.wanted());
}

#[test]
fn exiting_shuts_down_with_a_grey_fade_then_the_world_returns() {
    let mut random = Random::default();
    let mut effect = PortalSpace::new();
    effect.enter(true, 0.0, &mut random);
    effect.think(0.5, &mut random);
    effect.exit(false, 0.5);
    assert!(!effect.wanted());
    effect.think(0.5, &mut random);
    assert_eq!(effect.phase(), Phase::ShutdownPhase0);
    // Shutdown phase 0: the camera widens by 1.4 t^2 + 1; the fader is (0, 1, 1, t).
    effect.think(1.0, &mut random);
    let draw = *effect.draw();
    assert!(close(draw.camera_fov_scale, 0.25 * 1.4 + 1.0));
    assert!(draw.fader);
    assert!(close(draw.fader_color[0], 0.0));
    assert!(close(draw.fader_color[1], 1.0));
    assert!(close(draw.fader_color[2], 1.0));
    assert!(close(draw.fader_color[3], 0.5));
    // Phase 1 of the shutdown starts at the transition: the world returns with a field of view
    // of 1.4 (1 - t)^2 + 1, t the time into the phase.
    effect.think(1.6, &mut random);
    assert_eq!(effect.phase(), Phase::ShutdownPhase1);
    effect.think(1.9, &mut random);
    let draw = *effect.draw();
    assert!(draw.world);
    assert!(close(draw.world_fov_scale, 1.4 * 0.7 * 0.7 + 1.0));
    assert!(close(draw.fader_color[3], 0.7));
    // Past both phases: inactive, nothing drawn but the world.
    effect.think(2.7, &mut random);
    assert_eq!(effect.phase(), Phase::Inactive);
    assert!(!effect.in_portal_space());
    assert!(!effect.draw().tunnels);
}

#[test]
fn a_teleport_that_ends_during_the_fade_in_still_runs_its_shutdown() {
    let mut random = Random::default();
    let mut effect = PortalSpace::new();
    effect.enter(false, 0.0, &mut random);
    effect.think(0.0, &mut random);
    effect.exit(false, 0.2);
    // The fade-in finishes into Active; the wanted flag is gone, so Active shuts down next.
    effect.think(1.2, &mut random);
    assert_eq!(effect.phase(), Phase::Active);
    effect.think(1.3, &mut random);
    assert_eq!(effect.phase(), Phase::ShutdownPhase0);
}

#[test]
fn the_random_factor_is_rolled_when_the_effect_starts() {
    let mut random = Random::default();
    let mut effect = PortalSpace::new();
    effect.enter(true, 0.0, &mut random);
    let factor = effect.random_factor();
    assert!((0.0..10000.0).contains(&factor), "{factor}");
}
