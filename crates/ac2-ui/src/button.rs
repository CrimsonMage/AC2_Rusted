//! `UIElement_Button`: a button's state follows its attributes and the mouse
//! (`UIElement_Button::UpdateState`): `ghosted`; else (a radio button already pressed stays
//! pressed) pressed while the left button is held on it, rolled over while the pointer is over it
//! and `rollover` is set, else normal, each in its `highlighted` form when that is set. A press
//! broadcasts `mouse_press`; a release broadcasts `mouse_release_on` (over the button) or
//! `mouse_release_off`, and toggles `highlighted` on a `boolean_button` released over it.
//! A ghosted button broadcasts only with `broadcast_when_ghosted`.
//!
//! A left press on a `hot_button` also broadcasts `hot_click` (and steps the scroll bars that
//! name the button).
//!
//! Not modelled yet: hot buttons' repeating clicks while held, activation by keyboard.

use crate::element::UiTree;
use crate::mouse::{UI_WHEEL_IN, UI_WHEEL_OUT};
use crate::token::tokenize;
use crate::token::{
    BUTTON, BUTTON_BOOLEAN, BUTTON_BROADCAST_WHEN_GHOSTED, BUTTON_GHOSTED, BUTTON_HIGHLIGHTED,
    BUTTON_RADIO, BUTTON_ROLLOVER, MOUSE_EVENT_PASSTHROUGH, MSG_BUTTON_RELEASE_OFF,
    MSG_BUTTON_RELEASE_ON, MSG_HIGHLIGHT_STATE_CHANGE, MSG_MOUSE_PRESS, STATE_GHOSTED,
    STATE_HIGHLIGHT, STATE_HIGHLIGHT_PRESSED, STATE_HIGHLIGHT_ROLLOVER, STATE_NORMAL,
    STATE_NORMAL_PRESSED, STATE_NORMAL_ROLLOVER,
};

const BUTTON_HOT: u32 = tokenize("hot_button");
/// `ButtonMsg_hot_click`.
const MSG_HOT_CLICK: u32 = tokenize("hot_click");

/// The left button actions a button presses on (`0x40000004`, `0x40000007`).
fn is_left(action: u32) -> bool {
    matches!(action, 0x4000_0004 | 0x4000_0007)
}

impl UiTree {
    pub(crate) fn is_button(&self, index: usize) -> bool {
        self.el(index).desc.kind == BUTTON
    }

    /// `UIElement_Button::UpdateState`.
    pub(crate) fn button_update_state(&mut self, index: usize) {
        if !self.is_button(index) {
            return;
        }
        let state = if self.bool_attribute(index, BUTTON_GHOSTED) {
            STATE_GHOSTED
        } else {
            let highlighted = self.bool_attribute(index, BUTTON_HIGHLIGHTED);
            let rollover = self.bool_attribute(index, BUTTON_ROLLOVER);
            let radio = self.bool_attribute(index, BUTTON_RADIO);
            let el = self.el(index);
            if radio && el.state == STATE_NORMAL_PRESSED {
                return;
            }
            match (el.mouse_over, el.button_pressed, rollover, highlighted) {
                (true, true, _, false) => STATE_NORMAL_PRESSED,
                (true, true, _, true) => STATE_HIGHLIGHT_PRESSED,
                (true, false, true, false) => STATE_NORMAL_ROLLOVER,
                (true, false, true, true) => STATE_HIGHLIGHT_ROLLOVER,
                (_, _, _, true) => STATE_HIGHLIGHT,
                _ => STATE_NORMAL,
            }
        };
        if self.el(index).state != state {
            self.set_state(index, state);
        }
    }

    /// `UIElement_Button::SetState`'s first half: a highlight state sets `highlighted`.
    pub(crate) fn button_set_state(&mut self, index: usize, state: u32) {
        if self.is_button(index)
            && matches!(
                state,
                STATE_HIGHLIGHT | STATE_HIGHLIGHT_ROLLOVER | STATE_HIGHLIGHT_PRESSED
            )
            && !self.bool_attribute(index, BUTTON_HIGHLIGHTED)
        {
            self.set_bool(index, BUTTON_HIGHLIGHTED, true);
        }
    }

    /// `UIElement_Button::SetAttribute_Bool`, after a change.
    pub(crate) fn button_attribute(&mut self, index: usize, id: u32, value: bool) {
        if !self.is_button(index) {
            return;
        }
        if matches!(id, BUTTON_GHOSTED | BUTTON_HIGHLIGHTED | BUTTON_ROLLOVER) {
            self.button_update_state(index);
        }
        if id == BUTTON_HIGHLIGHTED {
            self.broadcast(index, MSG_HIGHLIGHT_STATE_CHANGE, i32::from(value));
        }
    }

    fn button_broadcasts(&self, index: usize) -> bool {
        !self.bool_attribute(index, BUTTON_GHOSTED)
            || self.bool_attribute(index, BUTTON_BROADCAST_WHEN_GHOSTED)
    }

    /// `UIElement_Button::MouseDown`, after `UIElement::MouseDown`. A button takes a press (it
    /// claims it, `processed`) but not a wheel turn: the turn goes on to the scroll bar or the
    /// scrollable parent the button sits in (`UIRegion::NotifyMouseDown`'s children-first walk
    /// stops at the first claim), so a wheel over a thumb or an arrow of a bar still reaches it.
    pub(crate) fn button_mouse_down(&mut self, index: usize, action: u32, processed: &mut bool) {
        if !self.is_button(index) {
            return;
        }
        let wheel = matches!(action, UI_WHEEL_IN | UI_WHEEL_OUT);
        if !wheel && !self.bool_attribute(index, MOUSE_EVENT_PASSTHROUGH) {
            *processed = true;
        }
        if is_left(action) {
            self.el_mut(index).button_pressed = true;
            self.button_update_state(index);
        }
        if self.button_broadcasts(index) {
            self.broadcast(index, MSG_MOUSE_PRESS, action as i32);
            // A hot button clicks once on the press (the repeats while held are not modelled).
            if is_left(action) && self.bool_attribute(index, BUTTON_HOT) {
                self.broadcast(index, MSG_HOT_CLICK, action as i32);
                self.scrollbar_hot_click(index);
            }
        }
    }

    /// `UIElement_Button::MouseUp`, after `UIElement::MouseUp`.
    pub(crate) fn button_mouse_up(&mut self, index: usize, action: u32, processed: &mut bool) {
        if !self.is_button(index) {
            return;
        }
        if !self.bool_attribute(index, MOUSE_EVENT_PASSTHROUGH) {
            *processed = true;
        }
        let ghosted = self.bool_attribute(index, BUTTON_GHOSTED);
        if is_left(action) {
            self.el_mut(index).button_pressed = false;
            if self.el(index).mouse_over && !ghosted && self.bool_attribute(index, BUTTON_BOOLEAN) {
                let highlighted = self.bool_attribute(index, BUTTON_HIGHLIGHTED);
                self.set_bool(index, BUTTON_HIGHLIGHTED, !highlighted);
            }
            self.button_update_state(index);
        }
        if self.button_broadcasts(index) {
            let message = if self.el(index).mouse_over {
                MSG_BUTTON_RELEASE_ON
            } else {
                MSG_BUTTON_RELEASE_OFF
            };
            self.broadcast(index, message, action as i32);
        }
    }
}
