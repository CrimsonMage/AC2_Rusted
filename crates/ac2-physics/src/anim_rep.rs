//! The animation half of `PHYSICSOBJREP`: its `ModeTracker` (over the body's behaviour table),
//! its `SliderTracker`, and the `AnimationObj` answers (`GetComboCVel` from the animator each
//! anim key plays) the trackers ask for.

use std::collections::HashMap;
use std::sync::Arc;

use ac2_anim::{
    Ac2AnimEvent, Ac2AnimationSource, Ac2BehaviorTable, Ac2CharTemplate, Ac2ComboInfo,
    Ac2ModeTracker, Ac2MotionAnimator, Ac2SliderTracker,
};

use crate::fpu::{ld, st};
use crate::motion_interp::{ELEMENT_GROUND, MODE_DEFAULT};
use crate::motion_interp_behaviors::MotionRep;

/// What a body animates with: its behaviour table (`VisualDesc` behaviour table DID, DB
/// 0x40000040), the `CharTemplate` (0x2F000000) and the animator each of the table's anim keys
/// plays through the body's `AnimMap`.
#[derive(Debug, Clone)]
pub struct BodyAnimation {
    pub table: Arc<Ac2BehaviorTable>,
    pub template: Arc<Ac2CharTemplate>,
    pub animators: HashMap<u32, Arc<Ac2MotionAnimator>>,
}

#[derive(Debug, Clone)]
pub(crate) struct AnimRep {
    pub(crate) modes: Ac2ModeTracker,
    pub(crate) sliders: Ac2SliderTracker,
    animators: Arc<HashMap<u32, Arc<Ac2MotionAnimator>>>,
    player: bool,
    events: Vec<Ac2AnimEvent>,
}

/// `PHYSICSOBJREP` as the trackers' `AnimationObj`.
pub(crate) struct RepSource<'a> {
    animators: &'a HashMap<u32, Arc<Ac2MotionAnimator>>,
    sliders: &'a [(u32, f32)],
    player: bool,
}

impl Ac2AnimationSource for RepSource<'_> {
    /// `CPartArray::GetComboCVel`: the inverse of the animator's period at the current slider
    /// values (0 without an animator or with a zero period).
    fn combo_cvel(&self, combo: Ac2ComboInfo) -> f32 {
        let Some(animator) = self.animators.get(&combo.anim_key) else {
            return 0.0;
        };
        let period = animator.blend(self.sliders).period();
        if period == 0.0 {
            0.0
        } else {
            st(1.0 / ld(period))
        }
    }

    /// `CPartArray::GetHoldLocations`: the `AnimInfo` hold loop is not decoded.
    fn hold_loop(&self, _combo: Ac2ComboInfo) -> Option<(f32, f32)> {
        None
    }

    fn is_the_player(&self) -> bool {
        self.player
    }
}

impl AnimRep {
    /// `SetupModeTracker`/`SetupSliderTracker` without a behaviour table: nothing animates.
    pub(crate) fn unanimated() -> Self {
        let template = Arc::new(Ac2CharTemplate {
            behaviors: HashMap::new(),
            default_element: ELEMENT_GROUND,
            default_mode: MODE_DEFAULT,
        });
        Self {
            modes: Ac2ModeTracker::new(template),
            sliders: Ac2SliderTracker::new(),
            animators: Arc::new(HashMap::new()),
            player: true,
            events: Vec::new(),
        }
    }

    /// `PHYSICSOBJREP::SetupModeTracker` (`ModeTracker::Init`) and `SetupSliderTracker`.
    pub(crate) fn new(animation: BodyAnimation, player: bool) -> Self {
        let mut rep = Self {
            modes: Ac2ModeTracker::new(animation.template),
            sliders: Ac2SliderTracker::new(),
            animators: Arc::new(animation.animators),
            player,
            events: Vec::new(),
        };
        let source = rep.source_parts();
        let table = Some(animation.table);
        rep.modes.init(table, &source.as_source());
        rep
    }

    fn source_parts(&self) -> SourceParts {
        SourceParts {
            animators: Arc::clone(&self.animators),
            sliders: self.sliders.values(),
            player: self.player,
        }
    }

    pub(crate) fn is_player(&self) -> bool {
        self.player
    }

    pub(crate) fn can_animate(&self) -> bool {
        self.modes.can_animate()
    }

    /// Runs `call` on the mode tracker with this rep as its `AnimationObj`.
    pub(crate) fn with_modes<R>(
        &mut self,
        call: impl FnOnce(&mut Ac2ModeTracker, &dyn Ac2AnimationSource) -> R,
    ) -> R {
        let parts = self.source_parts();
        let source = parts.as_source();
        let result = call(&mut self.modes, &source);
        self.events.extend(self.modes.take_events());
        result
    }

    /// `PHYSICSOBJREP::DoElement`.
    pub(crate) fn do_element(&mut self, element: u32) -> bool {
        self.with_modes(|modes, source| modes.do_element(element, source))
    }

    /// `PHYSICSOBJREP::UpdateSliderTracker` (the wind slider is not driven).
    pub(crate) fn update_sliders(&mut self, quantum: f32) {
        self.sliders.update(quantum);
        self.events.extend(self.sliders.take_events());
    }

    /// `PHYSICSOBJREP::UpdateAnimation` (`ModeTracker::Update`).
    pub(crate) fn update_animation(&mut self, quantum: f32) {
        self.with_modes(|modes, source| modes.update(quantum, source));
    }

    pub(crate) fn take_events(&mut self) -> Vec<Ac2AnimEvent> {
        std::mem::take(&mut self.events)
    }
}

/// The owned inputs of a [`RepSource`] (so the mode tracker can be borrowed mutably).
struct SourceParts {
    animators: Arc<HashMap<u32, Arc<Ac2MotionAnimator>>>,
    sliders: Vec<(u32, f32)>,
    player: bool,
}

impl SourceParts {
    fn as_source(&self) -> RepSource<'_> {
        RepSource {
            animators: &self.animators,
            sliders: &self.sliders,
            player: self.player,
        }
    }
}

impl MotionRep for AnimRep {
    fn do_autonomous_behavior(&mut self, behavior: u32) {
        self.with_modes(|modes, source| modes.do_autonomous_behavior(behavior, source));
    }

    fn is_doing_behavior(&self, behavior: u32) -> bool {
        self.modes.is_doing_behavior(behavior)
    }

    fn stop_behavior(&mut self, behavior: u32) {
        self.with_modes(|modes, source| modes.stop_behavior(behavior, source));
    }

    fn do_mode(&mut self, context: u32, mode: u32) {
        self.with_modes(|modes, source| modes.do_mode(context, mode, source));
    }

    fn desired_mode(&self) -> u32 {
        self.modes.desired_mode()
    }

    /// `PHYSICSOBJREP::DoFX`: queued for the FX manager.
    fn do_fx(&mut self, fx: u32, intensity: f32) {
        self.events.push(Ac2AnimEvent::StartFx { fx, intensity });
    }
}

impl crate::physics_obj::PhysicsObj {
    /// `CPhysicsObj::UpdateAnimation` for an object updated every animation tick (the player;
    /// the distance-degraded step, look-at, FX scripts and triggers are not modelled): the
    /// slider tracker, the half-step of local acceleration added to the animation velocity, then
    /// `ModeTracker::Update` when the body has a behaviour table.
    pub(crate) fn update_animation(&mut self, quantum: f32) {
        self.anim.update_sliders(quantum);
        let a = self.position.frame.global_to_local_vec(self.acceleration);
        let q = ld(quantum);
        let half = ld(0.5_f32);
        let cz = ld(st(ld(a.z) * half));
        let ax1 = ld(st(ld(a.x) * half * q));
        let by = ld(a.y) * half * q;
        let ax = st(ax1 * q);
        let ay = st(by * q);
        let az = cz * q * q;
        let v = self.anim_velocity;
        self.anim_velocity = crate::vector::Vector::new(
            st(ld(ax) + ld(v.x)),
            st(ld(ay) + ld(v.y)),
            st(az + ld(v.z)),
        );
        if self.anim.can_animate() {
            self.anim.update_animation(quantum);
            self.sync_mode();
        }
    }

    /// Installs the body's behaviour table (`PHYSICSOBJREP::SetupModeTracker`), keeping its
    /// current element as the desired one.
    pub(crate) fn set_animation(&mut self, animation: BodyAnimation, player: bool) {
        self.anim = AnimRep::new(animation, player);
        self.sync_mode();
    }

    pub(crate) fn take_animation_events(&mut self) -> Vec<Ac2AnimEvent> {
        self.anim.take_events()
    }
}
