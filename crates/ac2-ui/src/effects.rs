//! `gmEffectsUI`: the status bar's effect strip (`EffectListBox`). Each of the player's effects
//! (`gmCWInterface::InqEffectList`) is an `EffectTemplate` from the status bar layout with the
//! effect's name as its tooltip and its icon in `IconField`, kept in order of expiry
//! (`EffectTimeout`: effects without a duration last). A timed effect is `active` with its
//! `EffectTimeMeter` running from what is left down to empty over the time left, turning `mid`
//! and `low` near its end; a pulsed effect is `pulsed` and remembers its expiry. A missed pulse
//! alerts the effect's icon and starts its meter running down (`RecvNotice_PulseMissed`).
//! Pressing an effect selects it and asks to examine it.

use std::sync::Arc;

use crate::element::UiTree;
use crate::layout::{AttributeValue, LayoutDesc};
use crate::mouse::UiMessage;
use crate::token::{tokenize, MSG_MOUSE_PRESS};

fn id(name: &str) -> u32 {
    tokenize(name)
}

/// One effect as `InqEffectList` gives it (`EffectUINode`).
#[derive(Debug, Clone, PartialEq)]
pub struct EffectEntry {
    pub eid: u32,
    pub name: String,
    pub icon: u32,
    /// `m_timeCast`, `m_duration` (-1 when unset).
    pub time_cast: f64,
    pub duration: f64,
    pub pulsed: bool,
}

/// What a press asks: `SetSelection(INVALID_IID)`, an examination request for the effect
/// (type `0x40000001`), and the examination panel opened.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExamineEffect(pub u32);

/// `gmEffectsUI`'s state.
#[derive(Debug, Clone)]
pub struct EffectsUi {
    layout: Arc<LayoutDesc>,
    list: Option<usize>,
    selected_effect: u32,
    selected_element: Option<usize>,
}

fn id_attr(tree: &UiTree, element: usize, name: &str) -> Option<u32> {
    match tree.get(element)?.attribute(id(name)) {
        Some(AttributeValue::Id(v)) => Some(*v),
        _ => None,
    }
}

fn float_attr(tree: &UiTree, element: usize, name: &str) -> Option<f32> {
    match tree.get(element)?.attribute(id(name)) {
        Some(AttributeValue::Float(v)) => Some(*v),
        _ => None,
    }
}

impl EffectsUi {
    /// `gmEffectsUI::gmEffectsUI`: the list (an `IUIManager` lookup of `EffectListBox`).
    pub fn create(tree: &UiTree, layout: &Arc<LayoutDesc>) -> Self {
        let list = tree
            .roots()
            .iter()
            .find_map(|&r| tree.child_recursive(r, id("EffectListBox")));
        Self {
            layout: Arc::clone(layout),
            list,
            selected_effect: 0,
            selected_element: None,
        }
    }

    /// The strip's effects, in order.
    pub fn shown(&self, tree: &UiTree) -> Vec<u32> {
        self.list.map_or(Vec::new(), |l| {
            tree.list_items(l)
                .iter()
                .filter_map(|&e| id_attr(tree, e, "EffectID"))
                .collect()
        })
    }

    /// The element showing effect `eid`.
    pub fn element(&self, tree: &UiTree, eid: u32) -> Option<usize> {
        let list = self.list?;
        tree.list_items(list)
            .iter()
            .copied()
            .find(|&e| id_attr(tree, e, "EffectID") == Some(eid))
    }

    /// The icon field's state for its `Selected` and `Alert` attributes.
    fn icon_state(tree: &UiTree, field: usize) -> u32 {
        let selected = tree.bool_attribute(field, id("Selected"));
        let alert = tree.bool_attribute(field, id("Alert"));
        id(match (selected, alert) {
            (true, true) => "selected_alert",
            (true, false) => "selected",
            (false, true) => "unselected_alert",
            (false, false) => "unselected",
        })
    }

    /// `gmEffectsUI::SelectElement`.
    fn select_element(&mut self, tree: &mut UiTree, element: Option<usize>) {
        if self.selected_element == element {
            return;
        }
        for (old, on) in [(self.selected_element, false), (element, true)] {
            let field = old.and_then(|e| tree.child_recursive(e, id("IconField")));
            if let Some(field) = field {
                tree.set_bool(field, id("Selected"), on);
                let state = Self::icon_state(tree, field);
                tree.set_state(field, state);
            }
        }
        self.selected_element = element;
    }

    /// `gmEffectsUI::Init` at time `now`: effects added in order of expiry, gone ones removed,
    /// the selected one marked.
    pub fn init(&mut self, tree: &mut UiTree, effects: &[EffectEntry], now: f64) {
        self.selected_element = None;
        let Some(list) = self.list else {
            return;
        };
        tree.list_multi_op_start(list);
        for effect in effects {
            let timed = effect.duration != -1.0 && effect.time_cast != -1.0;
            let duration_left = if timed {
                (effect.time_cast + effect.duration) as f32
            } else {
                f32::MAX
            };
            // The first shown effect that outlasts this one, unless it is shown already.
            let mut before = None;
            let mut found = false;
            for &item in tree.list_items(list) {
                if id_attr(tree, item, "EffectID") == Some(effect.eid) {
                    found = true;
                    break;
                }
                if before.is_none() {
                    let timeout =
                        float_attr(tree, item, "EffectTimeout").unwrap_or(f32::MIN_POSITIVE);
                    if duration_left < timeout {
                        before = Some(item);
                    }
                }
            }
            if !found {
                self.add_effect(tree, list, effect, duration_left, before, now);
            }
        }
        // Gone effects leave, from the end; the selected one is marked.
        let items = tree.list_items(list).to_vec();
        for &item in items.iter().rev() {
            let eid = id_attr(tree, item, "EffectID");
            if !eid.is_some_and(|eid| effects.iter().any(|e| e.eid == eid)) {
                if let Some(at) = tree.list_what_num(list, item) {
                    if let Some(removed) = tree.list_remove_item(list, at) {
                        tree.delete_element(removed);
                    }
                }
            } else if eid == Some(self.selected_effect) && self.selected_effect != 0 {
                self.selected_element = Some(item);
                if let Some(field) = tree.child_recursive(item, id("IconField")) {
                    tree.set_bool(field, id("Selected"), true);
                    tree.set_state(field, id("selected"));
                }
            }
        }
        tree.list_multi_op_complete(list);
    }

    fn add_effect(
        &self,
        tree: &mut UiTree,
        list: usize,
        effect: &EffectEntry,
        duration_left: f32,
        before: Option<usize>,
        now: f64,
    ) {
        let Some(desc) = self.layout.element(id("EffectTemplate")).cloned() else {
            return;
        };
        let Some(element) = tree.create_element(Arc::clone(&self.layout), desc, false) else {
            return;
        };
        tree.set_tooltip(element, Some(effect.name.clone()));
        let at = before
            .and_then(|b| tree.list_what_num(list, b))
            .unwrap_or(tree.list_items(list).len());
        tree.list_insert_item(list, element, at);
        if let Some(field) = tree.child_recursive(element, id("IconField")) {
            // `AddImageToState(icon, 2, stateless)`.
            tree.set_image(field, effect.icon, 2);
        }
        if effect.duration != -1.0 && effect.time_cast != -1.0 {
            let expiry = effect.time_cast + effect.duration;
            if now < expiry {
                if effect.pulsed {
                    tree.set_state(element, id("pulsed"));
                    tree.set_float(element, id("ExpirationTime"), expiry as f32);
                } else {
                    tree.set_state(element, id("active"));
                    Self::start_meter(tree, element, effect, expiry, now);
                }
            }
        }
        tree.el_mut(element)
            .attributes
            .insert(id("EffectID"), AttributeValue::Id(effect.eid));
        tree.set_float(element, id("EffectTimeout"), duration_left);
    }

    /// `Init`'s meter for a timed effect (from objdump: the state thresholds take `1/duration`
    /// at 300 and 60 times, capped at a half and a quarter, the first on the unrounded
    /// reciprocal, the second on the stored float).
    fn start_meter(tree: &mut UiTree, element: usize, effect: &EffectEntry, expiry: f64, now: f64) {
        let Some(meter) = tree.child_recursive(element, id("EffectTimeMeter")) else {
            return;
        };
        let elapsed = ((now - effect.time_cast) / effect.duration) as f32;
        let left = 1.0 - elapsed;
        tree.meter_set_state_at_percentage(meter, id("full"), 0.0);
        let inv = 1.0 / effect.duration;
        let inv_left = 1.0 / left;
        let mid_cut = (inv * 300.0).min(0.5);
        let mid = (((1.0 - mid_cut - f64::from(elapsed)) * f64::from(inv_left)) as f32).max(0.0);
        tree.meter_set_state_at_percentage(meter, id("mid"), mid);
        let low_cut = (f64::from(inv as f32) * 60.0).min(0.25);
        let low = (((1.0 - low_cut - f64::from(elapsed)) * f64::from(inv_left)) as f32).max(0.0);
        tree.meter_set_state_at_percentage(meter, id("low"), low);
        tree.set_float(meter, id("smooth_movement_duration"), (expiry - now) as f32);
        tree.set_float(meter, id("position"), left);
        tree.set_float(meter, id("goal_position"), 0.0);
    }

    /// `gmEffectsUI::RecvNotice_PulseMissed`: the effect's icon alerted, the effect `active`,
    /// its meter `low` and running down from full to its expiry.
    pub fn pulse_missed(&mut self, tree: &mut UiTree, eid: u32, now: f64) {
        let Some(element) = self.element(tree, eid) else {
            return;
        };
        let Some(field) = tree.child_recursive(element, id("IconField")) else {
            return;
        };
        tree.set_bool(field, id("Alert"), true);
        let state = Self::icon_state(tree, field);
        tree.set_state(field, state);
        tree.set_state(element, id("active"));
        let Some(meter) = tree.child_recursive(element, id("EffectTimeMeter")) else {
            return;
        };
        if let Some(fill) = tree.child(meter, id("left_fill")) {
            tree.set_state(fill, id("low"));
        }
        if let Some(expiry) = float_attr(tree, element, "ExpirationTime") {
            tree.set_float(
                meter,
                id("smooth_movement_duration"),
                (f64::from(expiry) - now) as f32,
            );
            tree.set_float(meter, id("position"), 1.0);
            tree.set_float(meter, id("goal_position"), 0.0);
        }
    }

    /// `RecvNotice_ExaminationRequestChanged`: another kind of request deselects; a new effect
    /// is selected and the strip built again.
    pub fn examination_request_changed(
        &mut self,
        tree: &mut UiTree,
        kind: u32,
        data: u32,
        effects: &[EffectEntry],
        now: f64,
    ) {
        if kind != 0x4000_0001 {
            self.select_element(tree, None);
            self.selected_effect = 0;
        } else if data != self.selected_effect {
            self.selected_effect = data;
            self.init(tree, effects, now);
        }
    }

    /// `gmEffectsUI::ListenToMessage`: a left or right press on an effect.
    pub fn message(&mut self, tree: &mut UiTree, message: &UiMessage) -> Option<ExamineEffect> {
        if message.message != MSG_MOUSE_PRESS || message.element_id != id("EffectTemplate") {
            return None;
        }
        if !matches!(message.data as u32, 0x4000_0004 | 0x4000_0005) {
            return None;
        }
        let element = message.element?;
        let eid = id_attr(tree, element, "EffectID").unwrap_or(0);
        self.select_element(tree, Some(element));
        self.selected_effect = eid;
        Some(ExamineEffect(eid))
    }
}
