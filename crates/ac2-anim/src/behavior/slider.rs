//! `SliderTracker`: named animation sliders moving at a constant rate toward a desired value
//! (`DoSlider`), read by the animators and by colour and opacity (`SliderInfo` per slider).

use super::hash::RetailHash;
use super::Ac2AnimEvent;
use super::F_EPSILON;

/// Sliders `SliderTracker::Init` starts at 1: and the colour and opacity sliders.
pub const AC2_SLIDER_INIT: [u32; 6] = [
    0x4000_000a,
    0x4000_0015,
    0x4000_0016,
    0x4000_0017,
    0x4000_0018,
    0x4000_001a,
];

#[derive(Debug, Clone, Copy, PartialEq)]
struct SliderInfo {
    value: f32,
    vel: f32,
    desired: f32,
    context: u32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Ac2SliderTracker {
    sliders: RetailHash<SliderInfo>,
    active: bool,
    events: Vec<Ac2AnimEvent>,
}

impl Ac2SliderTracker {
    /// `SliderTracker::SliderTracker` then `Init`.
    pub fn new() -> Self {
        let mut tracker = Self::default();
        for slider in AC2_SLIDER_INIT {
            tracker.do_slider(0, slider, 1.0, 0.0);
        }
        tracker
    }

    /// `SliderTracker::DoSlider`: move `slider` to `pos` over `time` seconds (at once under
    /// 0.0002 s). A target within 0.0002 of the current one only answers `context`.
    pub fn do_slider(&mut self, context: u32, slider: u32, pos: f32, time: f32) -> bool {
        let desired = self.sliders.get(slider).map_or(0.0, |info| info.desired);
        if (f64::from(pos) - f64::from(desired)).abs() < f64::from(F_EPSILON) {
            if context != 0 {
                self.events
                    .push(Ac2AnimEvent::SliderDone { context, error: 0 });
            }
            return true;
        }
        if time != 0.0 {
            self.active = true;
        }
        self.sliders.add(
            slider,
            SliderInfo {
                value: 0.0,
                vel: 0.0,
                desired: 0.0,
                context,
            },
        );
        if context != 0 {
            self.events
                .push(Ac2AnimEvent::SliderDone { context, error: 0 });
        }
        let Some(info) = self.sliders.get_mut(slider) else {
            return true;
        };
        info.desired = pos;
        if !(f64::from(time) >= f64::from(F_EPSILON)) {
            info.value = pos;
            info.vel = 0.0;
        } else {
            info.vel = ((f64::from(pos) - f64::from(info.value)) / f64::from(time)) as f32;
        }
        true
    }

    /// `SliderTracker::Update` (slider interpolators are not modelled): each moving slider
    /// advances by `vel * quantum`, stopping on its desired value.
    pub fn update(&mut self, quantum: f32) {
        if !self.active {
            return;
        }
        self.active = false;
        for slider in self.sliders.keys() {
            let Some(info) = self.sliders.get_mut(slider) else {
                continue;
            };
            if !(f64::from(info.vel).abs() >= f64::from(F_EPSILON)) {
                continue;
            }
            self.active = true;
            let next = f64::from(quantum) * f64::from(info.vel) + f64::from(info.value);
            let vel = f64::from(info.vel);
            let desired = f64::from(info.desired);
            if (vel < 0.0 && next <= desired) || (vel > 0.0 && next >= desired) {
                info.value = info.desired;
                info.vel = 0.0;
            } else {
                info.value = next as f32;
            }
        }
    }

    /// `SliderTracker::GetSliderCurrentValue` (0 for an unknown slider).
    pub fn value(&self, slider: u32) -> f32 {
        self.sliders.get(slider).map_or(0.0, |info| info.value)
    }

    /// `SliderTracker::GetSliderDesiredValue`.
    pub fn desired(&self, slider: u32) -> f32 {
        self.sliders.get(slider).map_or(0.0, |info| info.desired)
    }

    /// `SliderTracker::GetSliderVelocity`.
    pub fn velocity(&self, slider: u32) -> f32 {
        self.sliders.get(slider).map_or(0.0, |info| info.vel)
    }

    /// `SliderTracker::GetNetworkSliders`: the tracked sliders `template` marks weenie
    /// controlled, in retail iteration order.
    pub fn network_sliders(&self, template: &crate::Ac2CharTemplate) -> Vec<u32> {
        self.sliders
            .iter()
            .map(|(slider, _)| slider)
            .filter(|&slider| template.is_network_slider(slider))
            .collect()
    }

    /// Every slider and its current value, in retail iteration order.
    pub fn values(&self) -> Vec<(u32, f32)> {
        self.sliders
            .iter()
            .map(|(slider, info)| (slider, info.value))
            .collect()
    }

    pub fn take_events(&mut self) -> Vec<Ac2AnimEvent> {
        std::mem::take(&mut self.events)
    }
}
