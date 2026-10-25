//! `UIElement_Meter`: a bar whose fill children (`left_fill`, `top_fill`, `right_fill`,
//! `bottom_fill`) show `position` (0 to 1). With `move_fill` the fills keep their size and slide
//! inside the meter's box, which clips them; otherwise they are cut to the position
//! (`UpdateLayout`). Setting `goal_position` moves `position` there at once, or with
//! `smooth_movement` over `smooth_movement_duration` seconds of the global loop
//! (`StartAnimation`, `Global_Loop`); a meter with states at percentages switches its fills'
//! states as the animation passes them (`SetStateAtPercentage`, `CalculateNextStateChange`).
//!
//! [`VitalDisplay`] is the gameplay helper that drives a meter and its text from a current and
//! maximum value.

use crate::element::UiTree;
use crate::token::{tokenize, STATELESS};

pub(crate) const METER: u32 = tokenize("meter");
const POSITION: u32 = tokenize("position");
const GOAL_POSITION: u32 = tokenize("goal_position");
const MOVE_FILL: u32 = tokenize("move_fill");
const SMOOTH_MOVEMENT: u32 = tokenize("smooth_movement");
const SMOOTH_MOVEMENT_DURATION: u32 = tokenize("smooth_movement_duration");
const LEFT_FILL: u32 = tokenize("left_fill");
const TOP_FILL: u32 = tokenize("top_fill");
const RIGHT_FILL: u32 = tokenize("right_fill");
const BOTTOM_FILL: u32 = tokenize("bottom_fill");

/// `UIElement_Meter`'s own members.
#[derive(Debug, Clone, Default)]
pub(crate) struct MeterState {
    animating: bool,
    anim_start_time: f64,
    anim_end_time: f64,
    anim_start_pos: f32,
    anim_end_pos: f32,
    /// `m_percentageToStateMap`: (state, percentage), in the order they were set.
    states: Vec<(u32, f32)>,
    next_state_change: f32,
    next_state: u32,
}

/// `_ftol2`: truncation toward zero.
fn ftol(value: f64) -> i32 {
    value as i32
}

impl UiTree {
    /// `UIElement_Meter::UIElement_Meter`: before the description's attributes, a 0.25 second
    /// smooth movement and position and goal 0.
    pub(crate) fn meter_create(&mut self, index: usize) {
        self.el_mut(index).meter = Some(Box::new(MeterState {
            next_state: STATELESS,
            ..MeterState::default()
        }));
        self.set_float(index, SMOOTH_MOVEMENT_DURATION, 0.25);
        self.set_float(index, POSITION, 0.0);
        self.set_float(index, GOAL_POSITION, 0.0);
    }

    fn is_meter(&self, index: usize) -> bool {
        self.el(index).meter.is_some()
    }

    /// `UIElement::GetChild`: the first direct child with id `id`.
    pub(crate) fn child(&self, index: usize, id: u32) -> Option<usize> {
        self.el(index)
            .children
            .iter()
            .copied()
            .find(|&c| self.el(c).id() == id)
    }

    /// `UIRegion::GetObjectX0` / `GetObjectY0`: the box's corner in its surface.
    fn object_origin(&self, index: usize) -> (i32, i32) {
        let el = self.el(index);
        let (x, y) = el.parent.map_or((0, 0), |p| self.object_origin(p));
        (el.rect.x0 + x, el.rect.y0 + y)
    }

    /// `UIElement::SetAttribute_Float`, with `UIElement_Meter::SetAttribute_Float`'s cases:
    /// whether the value was new.
    pub fn set_float(&mut self, index: usize, id: u32, value: f32) -> bool {
        let previous = self
            .el_mut(index)
            .attributes
            .insert(id, crate::layout::AttributeValue::Float(value));
        let changed = previous != Some(crate::layout::AttributeValue::Float(value));
        if changed {
            self.changed = true;
        }
        if !self.is_meter(index) {
            return changed;
        }
        if changed && id == POSITION {
            self.meter_update_layout(index);
        }
        if id == GOAL_POSITION {
            // Retail reads the bool into the value's own slot: without `smooth_movement` a
            // non-zero goal counts as set.
            let smooth = match self.el(index).attributes.get(&SMOOTH_MOVEMENT) {
                Some(crate::layout::AttributeValue::Bool(on)) => *on,
                _ => value != 0.0,
            };
            if smooth {
                self.meter_start_animation(index);
            } else {
                self.set_float(index, POSITION, value);
            }
        }
        changed
    }

    /// `UIElement_Meter::UpdateLayout`.
    pub(crate) fn meter_update_layout(&mut self, index: usize) {
        if !self.is_meter(index) {
            return;
        }
        let position = f64::from(self.float_attribute(index, POSITION).unwrap_or(0.0));
        let move_fill = self.bool_attribute(index, MOVE_FILL);
        let el = self.el(index);
        let rect = el.rect;
        // The meter's own corner plus its parent's place in the surface.
        let (left, top) = match el.parent {
            Some(parent) => {
                let (x, y) = self.object_origin(parent);
                (rect.x0 + x, rect.y0 + y)
            }
            None => (rect.x0, rect.y0),
        };
        let width = rect.x1 - rect.x0 + 1;
        let height = rect.y1 - rect.y0 + 1;
        let (w, h) = (f64::from(width), f64::from(height));
        if let Some(fill) = self.child(index, LEFT_FILL) {
            if move_fill {
                self.move_to(fill, ftol(-((1.0 - position) * w)), 0);
            } else {
                self.resize_to(fill, ftol(w * position), height);
                self.move_to(fill, left, top);
            }
        }
        if let Some(fill) = self.child(index, TOP_FILL) {
            if move_fill {
                self.move_to(fill, 0, ftol(-((1.0 - position) * h)));
            } else {
                self.resize_to(fill, width, ftol(h * position));
                self.move_to(fill, left, top);
            }
        }
        let right = ftol(w * position);
        if let Some(fill) = self.child(index, RIGHT_FILL) {
            if move_fill {
                self.move_to(fill, right, 0);
            } else {
                self.resize_to(fill, width - right, height);
                self.move_to(fill, right + left, top);
            }
        }
        let bottom = ftol(h * position);
        if let Some(fill) = self.child(index, BOTTOM_FILL) {
            if move_fill {
                self.move_to(fill, 0, bottom);
            } else {
                self.resize_to(fill, width, height - bottom);
                self.move_to(fill, left, top + bottom);
            }
        }
        self.changed = true;
    }

    /// `UIElement_Meter::SetStateAtPercentage`.
    pub fn meter_set_state_at_percentage(&mut self, index: usize, state: u32, percentage: f32) {
        if let Some(meter) = self.el_mut(index).meter.as_mut() {
            match meter.states.iter_mut().find(|(s, _)| *s == state) {
                Some(entry) => entry.1 = percentage,
                None => meter.states.push((state, percentage)),
            }
        }
    }

    /// `UIElement_Meter::CalculateNextStateChange`: the nearest state at or above `percentage`.
    fn meter_next_state_change(meter: &mut MeterState, percentage: f32) {
        let (mut best, mut change, mut state) = (1.0f32, 1.0f32, STATELESS);
        for &(s, at) in &meter.states {
            let diff = at - percentage;
            if 0.0 <= diff && diff < best {
                (best, change, state) = (diff, at, s);
            }
        }
        meter.next_state_change = change;
        meter.next_state = state;
    }

    /// `UIElement_Meter::StartAnimation`: from the position to the goal over the duration; a
    /// meter already moving starts half a duration back from a mirrored position.
    fn meter_start_animation(&mut self, index: usize) {
        let duration = self
            .float_attribute(index, SMOOTH_MOVEMENT_DURATION)
            .unwrap_or(0.0);
        if duration == 0.0 {
            return;
        }
        let position = self.float_attribute(index, POSITION).unwrap_or(0.0);
        let goal = self.float_attribute(index, GOAL_POSITION).unwrap_or(0.0);
        let now = self.time;
        let Some(meter) = self.el_mut(index).meter.as_mut() else {
            return;
        };
        meter.anim_end_pos = goal;
        if meter.animating {
            meter.anim_start_time = now - f64::from(duration) * 0.5;
            meter.anim_start_pos =
                (f64::from(position) - f64::from(goal) + f64::from(position)) as f32;
        } else {
            meter.anim_start_time = now;
            meter.anim_start_pos = position;
            meter.animating = true;
        }
        if !meter.states.is_empty() {
            Self::meter_next_state_change(meter, 0.0);
        }
        meter.anim_end_time = f64::from(duration) + meter.anim_start_time;
    }

    /// `UIMESSAGE_GLOBAL_LOOP` (`0xffff0005`) at time `now` in seconds: every element's
    /// transitions and media (`UIElement::ListenToMessage`), and every moving meter
    /// (`UIElement_Meter::Global_Loop`).
    pub fn global_loop(&mut self, now: f64) {
        self.time = now;
        self.tooltip_use_time(now);
        self.drag_use_time();
        for index in 0..self.slots.len() {
            self.element_global_loop(index);
            let moving = self
                .get(index)
                .and_then(|e| e.meter.as_ref())
                .is_some_and(|m| m.animating);
            if moving {
                self.meter_global_loop(index, now);
            }
        }
    }

    fn meter_global_loop(&mut self, index: usize, now: f64) {
        let Some(meter) = self.el_mut(index).meter.as_mut() else {
            return;
        };
        let ratio = (now - meter.anim_start_time) / (meter.anim_end_time - meter.anim_start_time);
        let mut percentage = ratio as f32;
        if 1.0 <= ratio {
            percentage = 1.0;
            meter.animating = false;
        }
        let mut new_state = None;
        if !meter.states.is_empty() && meter.next_state_change < percentage {
            new_state = Some(meter.next_state);
            Self::meter_next_state_change(meter, percentage);
        }
        let (start, end) = (
            f64::from(meter.anim_start_pos),
            f64::from(meter.anim_end_pos),
        );
        if let Some(state) = new_state {
            for fill in [LEFT_FILL, TOP_FILL, RIGHT_FILL, BOTTOM_FILL] {
                if let Some(fill) = self.child(index, fill) {
                    self.set_state(fill, state);
                }
            }
        }
        let position = (end - start) * f64::from(percentage) + start;
        self.set_float(index, POSITION, position as f32);
    }
}

/// `VitalDisplay`: a meter and its text showing a current and maximum value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VitalDisplay {
    pub meter: Option<usize>,
    pub text: Option<usize>,
    cur: u64,
    max: u64,
    /// The meter fills from the right (it has a `right_fill`): it shows what is missing.
    right_fill: bool,
}

/// `ID_Vital_DisplayText`'s variables: (`current`, `max`).
pub type VitalText<'a> = &'a dyn Fn(u64, u64) -> String;

impl VitalDisplay {
    /// `VitalDisplay::VitalDisplay`.
    pub fn new(tree: &UiTree, meter: Option<usize>, text: Option<usize>) -> Self {
        Self {
            meter,
            text,
            cur: 0,
            max: 0,
            right_fill: meter.is_some_and(|m| tree.child(m, RIGHT_FILL).is_some()),
        }
    }

    /// `VitalDisplay::Update`: a changed value moves the meter to `cur / max` (clamped; what is
    /// missing for a right fill) and sets the text to `ID_Vital_DisplayText`.
    pub fn update(&mut self, tree: &mut UiTree, cur: u64, max: u64, text: VitalText<'_>) {
        if cur == self.cur && max == self.max {
            return;
        }
        self.max = max;
        self.cur = cur.min(max);
        let mut fraction = if max == 0 {
            0.0
        } else {
            (self.cur as f64 / max as f64) as f32
        };
        if self.right_fill {
            fraction = 1.0 - fraction;
        }
        if let Some(meter) = self.meter {
            tree.set_float(meter, GOAL_POSITION, fraction);
        }
        if let Some(element) = self.text {
            tree.set_text(element, &text(self.cur, max));
        }
    }

    /// `VitalDisplay::UpdateCurrentVal`.
    pub fn update_current(&mut self, tree: &mut UiTree, cur: u64, text: VitalText<'_>) {
        self.update(tree, cur, self.max, text);
    }

    /// `VitalDisplay::UpdateMaxVal`.
    pub fn update_max(&mut self, tree: &mut UiTree, max: u64, text: VitalText<'_>) {
        self.update(tree, self.cur, max, text);
    }

    /// `VitalDisplay::ToggleTextState`: shows or hides the text; whether it now shows.
    pub fn toggle_text_state(&self, tree: &mut UiTree) -> bool {
        let Some(text) = self.text else {
            return false;
        };
        let visible = !tree.get(text).is_some_and(|e| e.visible);
        tree.set_visible(text, visible);
        visible
    }

    /// `VitalDisplay::SetTextState`: shows or hides the text; whether it showed before.
    pub fn set_text_state(&self, tree: &mut UiTree, visible: bool) -> bool {
        let Some(text) = self.text else {
            return false;
        };
        let was = tree.get(text).is_some_and(|e| e.visible);
        tree.set_visible(text, visible);
        was
    }

    /// `VitalDisplay::SetMeterState`.
    pub fn set_meter_state(&self, tree: &mut UiTree, state: u32) {
        if let Some(meter) = self.meter {
            tree.set_state(meter, state);
        }
    }
}
