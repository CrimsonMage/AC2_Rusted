//! `gmShortcutUI`'s special attacks and skill reset timers on the bar.
//!
//! A special attack is requested (`RecvNotice_RequestSpecialAttack`: its slots go pending and
//! stop dragging), starts (`StartSpecialAttack`: active), and ends (`EndSpecialAttack`: back to
//! stateless or toggled, dragging again, and forgotten). `ToggleSpecialAttack` sets a
//! maneuver's slots toggled or stateless. A slot is one shortcut of the maneuver's skill type
//! (`0x40000001`) and maneuver enum.
//!
//! A skill on reset keeps a `TimeLeftInfo` (`index`, `skill`, the slot's `TimeLeftMeter`, the
//! total and the end time): each frame (`UpdateTimeLeft`) the meter fills by the time spent
//! (`1 - (end - now) / total`, clamped), and the entry goes when the time is up or its slot no
//! longer holds that skill.

use crate::element::UiTree;
use crate::shortcut::{ShortcutUi, SHORTCUT_SKILL};
use crate::token::tokenize;

/// `TimeLeftInfo`: a skill's reset on one slot.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct TimeLeft {
    /// The shortcut's index (its slot).
    pub(crate) index: usize,
    /// `skill`: the skill's enum.
    pub(crate) skill: u32,
    /// `meter`: the slot's `TimeLeftMeter` element, when it has one.
    pub(crate) meter: Option<usize>,
    /// `timeTotal` and `timeEnd` (`Timer::cur_time`).
    pub(crate) total: f64,
    pub(crate) end: f64,
}

/// `gmShortcutUI::RecvNotice_ToggleSpecialAttack` and the request, start and end's slot states.
fn set_slot_state(tree: &mut UiTree, slot: usize, state: &str, dragable: Option<bool>) {
    tree.set_state(slot, tokenize(state));
    if let Some(dragable) = dragable {
        tree.set_bool(slot, tokenize("dragable"), dragable);
    }
}

impl ShortcutUi {
    /// The bar's item elements for the shortcuts of `maneuver` (`gmShortcutUI`'s walk over
    /// `_shortcuts` and `UIElement_ListBox::GetItem`).
    fn maneuver_slots(&self, tree: &UiTree, maneuver: u32) -> Vec<usize> {
        let Some(list) = self.list else {
            return Vec::new();
        };
        let items = tree.list_items(list);
        self.shortcuts
            .iter()
            .enumerate()
            .filter(|(_, s)| s.kind == SHORTCUT_SKILL && s.skill == maneuver)
            .filter_map(|(index, _)| items.get(index).copied())
            .collect()
    }

    /// `gmShortcutUI::RecvNotice_ToggleSpecialAttack`: the maneuver's slots toggled (or
    /// stateless), with their `toggled` flag.
    pub fn toggle_special_attack(&mut self, tree: &mut UiTree, maneuver: u32, toggled: bool) {
        for slot in self.maneuver_slots(tree, maneuver) {
            let state = if toggled { "toggled" } else { "stateless" };
            set_slot_state(tree, slot, state, None);
            tree.set_bool(slot, tokenize("toggled"), toggled);
        }
    }

    /// `gmShortcutUI::RecvNotice_RequestSpecialAttack`: the attack is kept as the maneuver's
    /// (once) and the maneuver's slots go pending, not dragable.
    pub fn request_special_attack(&mut self, tree: &mut UiTree, attack: u32, maneuver: u32) {
        if self.specials.iter().any(|&(a, _)| a == attack) {
            return;
        }
        self.specials.push((attack, maneuver));
        for slot in self.maneuver_slots(tree, maneuver) {
            set_slot_state(tree, slot, "pending", Some(false));
        }
    }

    /// `gmShortcutUI::RecvNotice_StartSpecialAttack`: the attack's maneuver's slots go active.
    pub fn start_special_attack(&mut self, tree: &mut UiTree, attack: u32) {
        let Some(&(_, maneuver)) = self.specials.iter().find(|(a, _)| *a == attack) else {
            return;
        };
        for slot in self.maneuver_slots(tree, maneuver) {
            set_slot_state(tree, slot, "active", None);
        }
    }

    /// `gmShortcutUI::RecvNotice_EndSpecialAttack`: the maneuver's slots return to toggled
    /// (when their flag says so) or stateless, dragable again; the attack is forgotten.
    pub fn end_special_attack(&mut self, tree: &mut UiTree, attack: u32) {
        let Some(at) = self.specials.iter().position(|(a, _)| *a == attack) else {
            return;
        };
        let (_, maneuver) = self.specials.remove(at);
        for slot in self.maneuver_slots(tree, maneuver) {
            let toggled = tree.get(slot).is_some_and(|e| {
                matches!(
                    e.attribute(tokenize("toggled")),
                    Some(crate::layout::AttributeValue::Bool(true))
                )
            });
            let state = if toggled { "toggled" } else { "stateless" };
            set_slot_state(tree, slot, state, Some(true));
        }
    }

    /// `gmShortcutUI::CheckTimeLeftArray`: the entry for `skill` on slot `index`.
    pub(crate) fn time_left_position(&self, skill: u32, index: usize) -> Option<usize> {
        self.timers
            .iter()
            .position(|t| t.skill == skill && t.index == index)
    }

    /// The skill on slot `index` went on reset: its entry is updated, or made with the slot's
    /// meter (`gmShortcutUI::CheckSkill`'s timer), until `now` plus `left`.
    pub fn reset_timer(
        &mut self,
        tree: &mut UiTree,
        index: usize,
        skill: u32,
        total: f64,
        left: f64,
    ) {
        if left <= 0.0 {
            return;
        }
        let end = self.now + left;
        if let Some(at) = self.time_left_position(skill, index) {
            self.timers[at].total = total;
            self.timers[at].end = end;
            return;
        }
        let meter = self
            .list
            .and_then(|list| tree.list_items(list).get(index).copied())
            .and_then(|slot| tree.child_recursive(slot, tokenize("TimeLeftMeter")));
        self.timers.push(TimeLeft {
            index,
            skill,
            meter,
            total,
            end,
        });
    }

    /// `gmShortcutUI::UpdateTimeLeft` at `now`: each entry's meter fills with the time spent, and
    /// an entry goes when its time is up (its meter full) or its slot no longer holds its skill.
    pub fn update_time_left(&mut self, tree: &mut UiTree, now: f64) {
        self.now = now;
        let mut at = 0;
        while at < self.timers.len() {
            let entry = self.timers[at];
            let holds_skill = self
                .shortcuts
                .get(entry.index)
                .is_some_and(|s| s.kind == SHORTCUT_SKILL && s.skill == entry.skill);
            if !holds_skill {
                self.timers.remove(at);
                continue;
            }
            // `(timeEnd - now) / timeTotal`, clamped to 0..=1 (a NaN clamps to 1).
            let ratio = (entry.end - now) / entry.total;
            let remaining = if ratio <= 1.0 { ratio.max(0.0) } else { 1.0 };
            let filled = 1.0 - remaining;
            if let Some(meter) = entry.meter {
                tree.set_float(meter, tokenize("position"), filled as f32);
                if filled == 1.0 {
                    self.timers.remove(at);
                    continue;
                }
            }
            at += 1;
        }
    }
}
