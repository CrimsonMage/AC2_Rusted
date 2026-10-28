//! `UIElement_ListBox`'s drag selection and drag rollover.
//!
//! A left press that selects an item on a `click_select` list that also has `drag_select`
//! opens a drag selection while the left button is down (`HandleSelection`, `StartDragSelect`):
//! until any button comes up, each pointer move selects the item under the pointer, with the
//! `new_selection` broadcast (`Global_MouseMove`, `EndDragSelect`).
//!
//! While something is being dragged, the pointer coming over the top of a `container` list with
//! `drag_rollover` starts a rollover (`MouseOverTop`, `StartRollover`): each pointer move puts the
//! item under the pointer in the list's `drag_rollover_accept` state, or `drag_rollover_reject`
//! when the list refuses the dragged item (its drop check, or `disallow_drag_in`), and puts the
//! item it left back in the state it had (`Rollover`). The rollover ends when the drag does or a
//! button comes up (`EndRollover`).
//!
//! Admin 7148 addresses: `StartDragSelect` 0x005F2A50, `EndDragSelect` 0x005F2AC0,
//! `StartRollover` 0x005F2B10, `EndRollover` 0x005F2B60, `MouseOverTop` 0x005F3190, `Rollover`
//! 0x005F3E70 (the drop check is the element's callback at +0x15C), `HandleSelection`
//! 0x005F3FD0, `Global_MouseMove` 0x005F44C0, `ListenToMessage` 0x005F4AB0 / 0x005F4B00.

use crate::element::UiTree;
use crate::token::tokenize;

const CONTAINER: u32 = tokenize("container");
const DISALLOW_DRAG_IN: u32 = tokenize("disallow_drag_in");
const DRAG_ROLLOVER_ACCEPT: u32 = tokenize("drag_rollover_accept");
const DRAG_ROLLOVER_REJECT: u32 = tokenize("drag_rollover_reject");

impl UiTree {
    /// `HandleSelection`'s tail: a `drag_select` list opens a drag selection while the left
    /// button is down (`StartDragSelect`).
    pub(crate) fn list_start_drag_select(&mut self, list: usize) {
        if !self.list_flag(list, "drag_select") || !self.drag.left_down {
            return;
        }
        let state = self.list(list);
        if state.drag_selecting {
            return;
        }
        state.drag_selecting = true;
        self.track_list(list);
    }

    /// `MouseOverTop(true)`: a drag over a `container` list with `drag_rollover` starts a
    /// rollover.
    pub(crate) fn list_mouse_over_top(&mut self, list: usize, over_top: bool) {
        if !over_top || !crate::list_box_select::is_list_kind(self.el(list).desc.kind) {
            return;
        }
        if self.el(list).list.as_ref().is_some_and(|s| s.rollover) || self.drag.element.is_none() {
            return;
        }
        if !self.bool_attribute(list, CONTAINER) || !self.list_flag(list, "drag_rollover") {
            return;
        }
        self.list(list).rollover = true;
        self.track_list(list);
    }

    fn track_list(&mut self, list: usize) {
        if !self.list_trackers.contains(&list) {
            self.list_trackers.push(list);
        }
    }

    /// `Global_MouseMove` for every list with an open drag selection or rollover.
    pub(crate) fn lists_global_mouse_move(&mut self) {
        for list in self.list_trackers.clone() {
            let Some(state) = self.get(list).and_then(|e| e.list.as_ref()) else {
                continue;
            };
            let (selecting, rollover, selected) =
                (state.drag_selecting, state.rollover, state.selected);
            if selecting {
                let under = self.list_item_under_mouse(list);
                if under.is_some() && under != selected {
                    self.list_set_selected_item(list, under, true);
                }
            }
            if rollover {
                self.list_rollover(list);
            }
        }
    }

    /// The global mouse up: every drag selection and rollover ends (`EndDragSelect`,
    /// `EndRollover`).
    pub(crate) fn lists_global_mouse_up(&mut self) {
        for list in std::mem::take(&mut self.list_trackers) {
            if self.get(list).is_none() {
                continue;
            }
            self.list(list).drag_selecting = false;
            self.list_end_rollover(list);
        }
    }

    /// `UIElement_ListBox::Rollover`.
    fn list_rollover(&mut self, list: usize) {
        let Some(item) = self.drag.element else {
            self.list_end_rollover(list);
            return;
        };
        let accepted = !self.bool_attribute(list, DISALLOW_DRAG_IN)
            && self
                .el(list)
                .drag_drop
                .is_none_or(|check| self.run_drag_check(check, item, list));
        let state = if accepted {
            DRAG_ROLLOVER_ACCEPT
        } else {
            DRAG_ROLLOVER_REJECT
        };
        let under = self.list_item_under_mouse(list);
        let last = self.list(list).drag_last_over;
        if last.map(|(element, _)| element) == under {
            return;
        }
        if let Some((element, old)) = last {
            if self.get(element).is_some() {
                self.set_state(element, old);
            }
            self.list(list).drag_last_over = None;
        }
        if let Some(element) = under {
            let current = self.el(element).state();
            if current != state {
                self.list(list).drag_last_over = Some((element, current));
                self.set_state(element, state);
            }
        }
    }

    /// `UIElement_ListBox::EndRollover`: the item rolled over goes back to its state.
    fn list_end_rollover(&mut self, list: usize) {
        let state = self.list(list);
        if !state.rollover {
            return;
        }
        state.rollover = false;
        if let Some((element, old)) = state.drag_last_over.take() {
            if self.get(element).is_some() {
                self.set_state(element, old);
            }
        }
    }
}
