//! `UIElement_Field`'s drag rollover: while something is dragged, the pointer over the top of a
//! `container` field with `drag_rollover` puts the field in its `drag_rollover_accept` state, or
//! `drag_rollover_reject` when it refuses the dragged item (its drop check, or
//! `disallow_drag_in`); the field takes back the state it had when the pointer leaves or the item
//! is dropped on it. Admin 7148: `MouseOverTop` 0x005FBCB0, `CatchDroppedItem` 0x005FBDB0. The
//! lists' rollover, item by item, is `list_drag.rs`.

use crate::element::UiTree;
use crate::token::tokenize;

const FIELD: u32 = tokenize("field");
const CONTAINER: u32 = tokenize("container");
const DRAG_ROLLOVER: u32 = tokenize("drag_rollover");
const DISALLOW_DRAG_IN: u32 = tokenize("disallow_drag_in");
const DRAG_ROLLOVER_ACCEPT: u32 = tokenize("drag_rollover_accept");
const DRAG_ROLLOVER_REJECT: u32 = tokenize("drag_rollover_reject");

impl UiTree {
    /// `UIElement_Field::MouseOverTop`: over the top while something is dragged, the field's
    /// rollover state (its own state kept the first time, `m_oldState`); off it, that state back.
    pub(crate) fn field_mouse_over_top(&mut self, field: usize, over_top: bool) {
        if self.el(field).desc.kind != FIELD {
            return;
        }
        if !over_top {
            self.field_end_rollover(field);
            return;
        }
        let Some(item) = self.drag.element else {
            return;
        };
        if !self.bool_attribute(field, CONTAINER) || !self.bool_attribute(field, DRAG_ROLLOVER) {
            return;
        }
        let accepted = !self.bool_attribute(field, DISALLOW_DRAG_IN)
            && self
                .el(field)
                .drag_drop
                .is_none_or(|check| self.run_drag_check(check, item, field));
        let state = if accepted {
            DRAG_ROLLOVER_ACCEPT
        } else {
            DRAG_ROLLOVER_REJECT
        };
        let current = self.el(field).state();
        if state != current {
            let old = &mut self.el_mut(field).field_rollover;
            if old.is_none() {
                *old = Some(current);
            }
            self.set_state(field, state);
        }
    }

    /// `UIElement_Field::CatchDroppedItem`'s first step, and the pointer leaving: the state the
    /// field had before its rollover, when it had one.
    pub(crate) fn field_end_rollover(&mut self, field: usize) {
        if let Some(old) = self.el_mut(field).field_rollover.take() {
            self.set_state(field, old);
        }
    }
}
