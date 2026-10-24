//! The animation half of `PHYSICSOBJREP`: its `ModeTracker` (over the body's behaviour table),
//! its `SliderTracker`, the `DBAnimator` each anim key plays, and the `AnimationObj` answers
//! (`GetComboCVel`, `GetHoldLocations`, `GetSliderCurrentValue`) the trackers and animators ask
//! for.

use std::collections::HashMap;
use std::sync::Arc;

use ac2_anim::{
    Ac2AnimEvent, Ac2AnimationSource, Ac2AnimatorContext, Ac2BehaviorTable, Ac2BoneTransform,
    Ac2CharTemplate, Ac2ComboInfo, Ac2DbAnimator, Ac2ModeTracker, Ac2Setup, Ac2SliderTracker,
    Ac2TrackerState,
};

use crate::fpu::{ld, st};
use crate::motion_interp::{ELEMENT_GROUND, MODE_DEFAULT};
use crate::motion_interp_behaviors::MotionRep;

/// What a body animates with: its behaviour table (`VisualDesc` behaviour table DID, DB
/// 0x40000040), the `CharTemplate` (0x2F000000), the animator each of the table's anim keys
/// plays through the body's `AnimMap`, and the Setup whose bones the animators pose.
#[derive(Debug, Clone)]
pub struct BodyAnimation {
    pub table: Arc<Ac2BehaviorTable>,
    pub template: Arc<Ac2CharTemplate>,
    pub animators: HashMap<u32, Arc<Ac2DbAnimator>>,
    pub setup: Option<Arc<Ac2Setup>>,
    /// `PHYSICSOBJREP::m_scale` (`GetVdescScale`).
    pub vdesc_scale: [f32; 3],
}

#[derive(Debug, Clone)]
pub(crate) struct AnimRep {
    pub(crate) modes: Ac2ModeTracker,
    pub(crate) sliders: Ac2SliderTracker,
    pub(crate) animators: Arc<HashMap<u32, Arc<Ac2DbAnimator>>>,
    pub(crate) table: Option<Arc<Ac2BehaviorTable>>,
    pub(crate) setup: Option<Arc<Ac2Setup>>,
    pub(crate) vdesc_scale: [f32; 3],
    /// The bone `TransformBase`s of the last `DoAnimation`.
    pub(crate) bones: Vec<Ac2BoneTransform>,
    player: bool,
    events: Vec<Ac2AnimEvent>,
}

/// `PHYSICSOBJREP` as the trackers' `AnimationObj`: animator periods at the current slider
/// values.
struct RepSource<'a> {
    animators: &'a HashMap<u32, Arc<Ac2DbAnimator>>,
    sliders: &'a Ac2SliderTracker,
    player: bool,
}

/// The read-only `AnimationObj` view animator periods need (sliders only).
struct SliderView<'a>(&'a Ac2SliderTracker);

impl Ac2AnimatorContext for SliderView<'_> {
    fn slider(&self, slider: u32) -> f32 {
        self.0.value(slider)
    }
    fn do_slider(&mut self, _slider: u32, _pos: f32, _time: f32) {}
    fn anim_velocity(&self) -> [f32; 3] {
        [0.0; 3]
    }
    fn effects_scale(&self) -> [f32; 3] {
        [1.0; 3]
    }
    fn vdesc_scale(&self) -> [f32; 3] {
        [1.0; 3]
    }
    fn omega(&self) -> [f32; 3] {
        [0.0; 3]
    }
    fn terrain_type(&self) -> u32 {
        0
    }
}

impl Ac2AnimationSource for RepSource<'_> {
    /// `PHYSICSOBJREP::GetComboCVel` -> `CPartArray::GetComboCVel`: the inverse of the
    /// animator's period (0 without an animator or with a zero period).
    fn combo_cvel(&self, tracker: Ac2TrackerState, combo: Ac2ComboInfo) -> f32 {
        let Some(animator) = self.animators.get(&combo.anim_key) else {
            return 0.0;
        };
        let period = animator.period(&SliderView(self.sliders), tracker);
        if period == 0.0 {
            0.0
        } else {
            st(1.0 / ld(period))
        }
    }

    /// `PHYSICSOBJREP::GetHoldLocations` (a loop of 0 to 0 is none).
    fn hold_loop(&self, tracker: Ac2TrackerState, combo: Ac2ComboInfo) -> Option<(f32, f32)> {
        self.animators
            .get(&combo.anim_key)?
            .hold_loop(&SliderView(self.sliders), tracker)
            .filter(|&(start, end)| start != 0.0 || end != 0.0)
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
            table: None,
            setup: None,
            vdesc_scale: [1.0; 3],
            bones: Vec::new(),
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
            table: Some(Arc::clone(&animation.table)),
            setup: animation.setup,
            vdesc_scale: animation.vdesc_scale,
            bones: Vec::new(),
            player,
            events: Vec::new(),
        };
        let table = Some(animation.table);
        rep.with_modes(|modes, source| modes.init(table, source));
        rep
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
        let source = RepSource {
            animators: &self.animators,
            sliders: &self.sliders,
            player: self.player,
        };
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

    pub(crate) fn push_events(&mut self, events: impl IntoIterator<Item = Ac2AnimEvent>) {
        self.events.extend(events);
    }

    pub(crate) fn take_events(&mut self) -> Vec<Ac2AnimEvent> {
        std::mem::take(&mut self.events)
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
