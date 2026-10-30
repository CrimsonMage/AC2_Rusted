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
//!
//! The bar's grip resizes `ShortcutField`; on the next global loop (`_bDirty`) the bar snaps to
//! whole rows (`resize_loop`).

use crate::element::UiTree;
use crate::layout::AttributeValue;
use crate::shortcut::{ShortcutRequest, ShortcutSource, ShortcutUi, SHORTCUT_SKILL};
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
    /// meter (`gmShortcutUI::CheckSkill`'s timer), until `now` plus `left`. A slot without a
    /// meter gets no entry. A total of zero counts as `1e-4` seconds (`0x38d1b717`).
    pub fn reset_timer(
        &mut self,
        tree: &mut UiTree,
        index: usize,
        skill: u32,
        total: f64,
        left: f64,
    ) {
        let meter = self
            .list
            .and_then(|list| tree.list_items(list).get(index).copied())
            .and_then(|slot| tree.child_recursive(slot, tokenize("TimeLeftMeter")));
        let Some(meter) = meter else {
            return;
        };
        let total = if total as f32 == 0.0 {
            f64::from(1.0e-4_f32)
        } else {
            total
        };
        let end = self.now + left;
        if let Some(at) = self.time_left_position(skill, index) {
            self.timers[at].total = total;
            self.timers[at].end = end;
            return;
        }
        self.timers.push(TimeLeft {
            index,
            skill,
            meter: Some(meter),
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

    /// `gmShortcutUI::ListenToMessage`'s global loop with `_bDirty` set (the field moved or was
    /// resized: retail's `MoveTo` / `ResizeTo` broadcast `location_changed` for a `save_loc`
    /// element; here the field's box differs from the one the bar last gave it). The height
    /// snaps to whole rows, `rows = (int)((height - border_size) / row_height + 0.5)`, kept
    /// within `min_height` and `max_height` with the bottom edge where it was; the list takes
    /// the rows (`max_rows`), the slots are padded (`UpdatePadding`) and the rows are sent on
    /// (`SendNotice_SetShortcutHeight`, [`ShortcutRequest::Height`]).
    pub fn resize_loop(&mut self, tree: &mut UiTree, source: &dyn ShortcutSource) {
        let (Some(field), Some(list)) = (self.field, self.list) else {
            return;
        };
        let now = tree.screen_box(field);
        match self.own_box {
            None => {
                self.own_box = Some(now);
                return;
            }
            Some(own) if own == now => return,
            Some(_) => {}
        }
        self.own_box = Some(now);
        let int = |name: &str| match tree.get(field)?.attribute(tokenize(name)) {
            Some(AttributeValue::Int(v)) => Some(*v),
            _ => None,
        };
        let (Some(border), Some(row_height), Some(min), Some(max)) = (
            int("border_size"),
            int("row_height"),
            int("min_height"),
            int("max_height"),
        ) else {
            return;
        };
        if row_height == 0 {
            return;
        }
        let rect = tree.el(field).rect;
        let height = rect.height();
        // `_ftol2` of the x87 sum: truncation toward zero.
        let rows = (f64::from(height - border) / f64::from(row_height) + 0.5) as i32;
        let mut excess = height - rows * row_height - border;
        let mut snapped = height;
        if excess != 0 {
            snapped -= excess;
            if snapped < min {
                excess += min - snapped;
                snapped = min;
            }
            if max < snapped {
                excess += snapped - max;
                snapped = max;
            }
            if excess != 0 {
                let (x, y) = tree.el(field).object.unwrap_or((0, 0));
                tree.resize_to(field, rect.width(), snapped);
                tree.move_to(field, x, y + excess);
            }
        }
        tree.el_mut(list)
            .attributes
            .insert(tokenize("max_rows"), AttributeValue::Int(rows));
        let width = tree.el(list).rect.width();
        tree.resize_to(list, width, row_height * rows);
        self.update_padding(tree, source);
        self.rows = rows;
        self.requests
            .push(ShortcutRequest::Height(rows.max(0) as u32));
        self.own_box = Some(tree.screen_box(field));
    }
}
