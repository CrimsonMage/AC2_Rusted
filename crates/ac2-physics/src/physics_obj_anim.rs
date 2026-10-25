//! `CPhysicsObj`'s animation entry points over `PHYSICSOBJREP`'s mode and slider trackers:
//! `DoMode`, `SetMode`, `DoBehavior`, `ReleaseBehavior`, the `Stop*` family, `DoSlider`,
//! `RecvEvt_DoSlider` and the animation half of `set_description`. The weenie's mode hook
//! (`CWeenieObject` on a mode change), child propagation and the story manager are not
//! modelled.

use std::collections::{BTreeMap, BTreeSet};

use ac2_anim::Ac2BehaviorParams;

use crate::fpu::{ld, st, F_EPSILON};
use crate::objects::STATIC_PS;

use super::PhysicsObj;

/// `PhysicsState` `ANIMATION_FROZEN_PS`.
pub(crate) const ANIMATION_FROZEN_PS: u32 = 0x4_0000;
/// `TransientState` `ANIMATION_ACTIVE_TS`.
const ANIMATION_ACTIVE_TS: u32 = 0x100;

/// One `PhysicsDesc` slider (`SliderData`): where the sender's slider is heading and how fast.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DescSlider {
    pub slider: u32,
    /// `SliderData::m_value`: the desired value.
    pub value: f32,
    /// `SliderData::m_velocity`.
    pub velocity: f32,
}

impl PhysicsObj {
    /// `CPhysicsObj::is_animation_frozen`.
    pub(crate) fn is_animation_frozen(&self) -> bool {
        self.state & ANIMATION_FROZEN_PS != 0
    }

    /// `CPhysicsObj::set_animation_active(1)`: a static body never becomes animation-active.
    fn set_animation_active(&mut self) {
        if self.state & STATIC_PS == 0 {
            self.transient_state |= ANIMATION_ACTIVE_TS;
        }
    }

    /// `CPhysicsObj::DoMode`: unless animation is frozen, `ModeTracker::DoMode` (the mode
    /// callback reports reaching it to `context`). Always true, as retail returns.
    pub(crate) fn do_mode(&mut self, context: u32, mode: u32) -> bool {
        if self.is_animation_frozen() {
            return true;
        }
        self.set_animation_active();
        self.anim
            .with_modes(|modes, source| modes.do_mode(context, mode, source));
        self.sync_mode();
        true
    }

    /// `CPhysicsObj::SetMode`: `ModeTracker::SetMode`, switching at once.
    pub(crate) fn set_mode(&mut self, mode: u32) -> bool {
        self.set_animation_active();
        let set = self
            .anim
            .with_modes(|modes, source| modes.set_mode(mode, source));
        self.sync_mode();
        set
    }

    /// `CPhysicsObj::DoBehavior(BehaviorParams&)`: unless animation is frozen, the params' mode
    /// first (`PHYSICSOBJREP::DoMode(0, mode)`), then `ModeTracker::DoBehavior`. Returns whether
    /// the tracker started it (retail returns 1 regardless).
    pub(crate) fn do_behavior(&mut self, params: &Ac2BehaviorParams) -> bool {
        if self.is_animation_frozen() {
            return false;
        }
        self.set_animation_active();
        if params.mode != 0 {
            self.anim
                .with_modes(|modes, source| modes.do_mode(0, params.mode, source));
        }
        let started = self
            .anim
            .with_modes(|modes, source| modes.do_behavior(params, source));
        self.sync_mode();
        started
    }

    /// `CPhysicsObj::ReleaseBehavior`: the params' mode first (`DoMode(0, mode)`), then
    /// `ModeTracker::ReleaseBehavior`.
    pub(crate) fn release_behavior(&mut self, behavior: u32, params: &Ac2BehaviorParams) -> bool {
        self.set_animation_active();
        if params.mode != 0 {
            self.do_mode(0, params.mode);
        }
        self.anim
            .with_modes(|modes, source| modes.release_behavior(behavior, params, source));
        true
    }

    /// `CPhysicsObj::StopBehavior`.
    pub(crate) fn stop_behavior(&mut self, behavior: u32) {
        self.set_animation_active();
        self.anim
            .with_modes(|modes, source| modes.stop_behavior(behavior, source));
    }

    /// `CPhysicsObj::StopLayer`.
    pub(crate) fn stop_layer(&mut self, layer: u32) {
        self.set_animation_active();
        self.anim
            .with_modes(|modes, source| modes.stop_layer(layer, source));
    }

    /// `CPhysicsObj::StopAllWeenieBehaviors`.
    pub(crate) fn stop_all_weenie_behaviors(&mut self) -> bool {
        self.anim
            .with_modes(|modes, source| modes.stop_all_weenie_behaviors(source))
    }

    /// `CPhysicsObj::StopMoveToCancelsBehaviors`.
    pub(crate) fn stop_move_to_cancels_behaviors(&mut self) {
        self.anim
            .with_modes(|modes, source| modes.stop_move_to_cancels_behaviors(source));
    }

    /// `CPhysicsObj::DoSlider` -> `PHYSICSOBJREP::DoSlider`.
    pub(crate) fn do_slider(&mut self, context: u32, slider: u32, pos: f32, time: f32) -> bool {
        self.set_animation_active();
        self.anim.do_slider(context, slider, pos, time);
        true
    }

    /// `CPhysicsObj::RecvEvt_DoSlider`: the player ignores the server on a slider its
    /// `CharTemplate` leaves to the client (`CharTemplate::IsUserControlled`).
    pub(crate) fn recv_do_slider(&mut self, slider: u32, value: f32, time: f32) -> bool {
        if self.anim.is_player() && self.anim.modes.template().is_user_controlled(slider) {
            return false;
        }
        self.do_slider(0, slider, value, time)
    }

    /// The animation half of `CPhysicsObj::set_description`: `DoMode(0, mode)`; then the
    /// running network behaviours (`GetNetworkBehaviorParams`) the description does not list
    /// stop, and the listed ones not running start (`DoBehavior`); then every network slider and
    /// every described slider, in ascending id order, moves to the described value (0 when not
    /// described) unless it is already heading there at that velocity. A slider with a velocity
    /// takes `|desired - value| / velocity` seconds to get there.
    pub(crate) fn set_description_animation(
        &mut self,
        mode: u32,
        behaviors: &[Ac2BehaviorParams],
        sliders: &[DescSlider],
    ) {
        self.do_mode(0, mode);
        let network = self.anim.modes.network_behavior_params();
        for running in &network {
            if !behaviors.iter().any(|b| b.behavior == running.behavior) {
                self.stop_behavior(running.behavior);
            }
        }
        for described in behaviors {
            if !network.iter().any(|b| b.behavior == described.behavior) {
                self.do_behavior(described);
            }
        }
        let mut described: BTreeMap<u32, (f32, f32)> = BTreeMap::new();
        for slider in sliders {
            described
                .entry(slider.slider)
                .or_insert((slider.value, slider.velocity));
        }
        let mut ids: BTreeSet<u32> = self
            .anim
            .sliders
            .network_sliders(self.anim.modes.template())
            .into_iter()
            .collect();
        ids.extend(described.keys().copied());
        for id in ids {
            let desired = self.anim.sliders.desired(id);
            let velocity = self.anim.sliders.velocity(id);
            let (value, rate) = described.get(&id).copied().unwrap_or((0.0, 0.0));
            if desired == value && velocity == rate {
                continue;
            }
            let time = if ld(F_EPSILON) <= ld(rate).abs() {
                st(((ld(desired) - ld(value)) / ld(rate)).abs())
            } else {
                0.0
            };
            self.do_slider(0, id, value, time);
        }
    }
}
