//! The mouse over the UI: `UIElementManager::StartInput` / `StopInput` turn UI actions into
//! pointer moves (`0x40000001` x, `0x40000002` y, absolute), button presses (`0x40000004`..
//! `0x40000009`) and wheel turns (`0x400000B8` in, `0x400000B9` out, presses only).
//!
//! `UIElementManager::UpdateMouse` walks the roots topmost first (lowest z), each region's children
//! before itself (`UIRegion::NotifyMouseMove`): an element is under the pointer when its parent is,
//! it is shown and the pointer is in its box, and it counts only while `mouse_invisible` is unset.
//! The first element found is the top region. Elements broadcast `mouse_over` / `mouse_over_top`
//! as those change and `mouse_move(_top)` while the pointer is over them.
//! `MouseDownEvent` offers a press to every element under the pointer (`NotifyMouseDown`): each
//! broadcasts `mouse_press_top` (when top) and `mouse_press`; a child of a root (and any button)
//! ends the press unless `mouse_event_passthrough`. A press held on an element the pointer has
//! left is released off it. `MouseUpEvent` releases the button on every element that holds it.
//! Each event also goes out as a global message (`0xFFFF0000` down, `0xFFFF0001` up,
//! `0xFFFF0002` move).
//!
//! Drag and drop is `drag.rs`, moving and resizing windows `window_locations.rs` and `resize.rs`,
//! activation `activation.rs`.

use crate::element::UiTree;
use crate::layout::AttributeValue;
use crate::token::{
    MOUSE_EVENT_PASSTHROUGH, MSG_MOUSE_MOVE, MSG_MOUSE_MOVE_TOP, MSG_MOUSE_OVER,
    MSG_MOUSE_OVER_TOP, MSG_MOUSE_PRESS, MSG_MOUSE_PRESS_TOP, MSG_MOUSE_RELEASE,
    MSG_MOUSE_RELEASE_TOP,
};
use crate::window_locations::UI_LEFT_BUTTON;

/// `UIMESSAGE_GLOBAL_*` (`UIElementManager::BroadcastGlobalMessage`).
pub const GLOBAL_MOUSE_DOWN: u32 = 0xFFFF_0000;
pub const GLOBAL_MOUSE_UP: u32 = 0xFFFF_0001;
pub const GLOBAL_MOUSE_MOVE: u32 = 0xFFFF_0002;
pub const GLOBAL_KEY_PRESS: u32 = 0xFFFF_0003;
pub const GLOBAL_KEY_RELEASE: u32 = 0xFFFF_0004;

/// UI actions (`UIElementManager::StartInput`).
pub const UI_POINTER_X: u32 = 0x4000_0001;
pub const UI_POINTER_Y: u32 = 0x4000_0002;
pub use crate::camera_ui::{UI_WHEEL_IN, UI_WHEEL_OUT};

/// A message an element (or the manager, for globals) broadcast
/// (`UIElementManager::BroadcastMessage` / `BroadcastGlobalMessage`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UiMessage {
    /// The element id (`referenceDesc->elementID`); 0 for globals.
    pub element_id: u32,
    pub element: Option<usize>,
    pub message: u32,
    pub data: i32,
    /// The dragged copy a `start_drag_item` / `stop_drag_item` carries.
    pub item: Option<usize>,
}

/// Whether `action` is one of the buttons `StartInput` sends to `MouseDownEvent`.
pub fn is_mouse_button(action: u32) -> bool {
    (0x4000_0004..=0x4000_0009).contains(&action)
}

impl UiTree {
    /// `UIElement::BroadcastMessage`.
    pub(crate) fn broadcast(&mut self, index: usize, message: u32, data: i32) {
        let element_id = self.el(index).id();
        self.messages.push(UiMessage {
            element_id,
            element: Some(index),
            message,
            data,
            item: None,
        });
    }

    pub(crate) fn broadcast_global(&mut self, message: u32, data: u32) {
        self.messages.push(UiMessage {
            element_id: 0,
            element: None,
            message,
            data: data as i32,
            item: None,
        });
    }

    /// The messages broadcast since the last call.
    pub fn take_messages(&mut self) -> Vec<UiMessage> {
        std::mem::take(&mut self.messages)
    }

    /// The pointer (`ICIDM` mouse position).
    pub fn pointer(&self) -> (i32, i32) {
        self.pointer
    }

    /// The element the pointer is over at the top (`m_mouseTopRegion`).
    pub fn mouse_top(&self) -> Option<usize> {
        self.mouse_top
    }

    pub(crate) fn bool_attribute(&self, index: usize, id: u32) -> bool {
        matches!(
            self.el(index).attributes.get(&id),
            Some(AttributeValue::Bool(true))
        )
    }

    /// `UIElementManager::StartInput`.
    pub fn start_input(&mut self, action: u32, extent: f32) {
        match action {
            UI_POINTER_X => {
                self.pointer.0 = extent as i32;
                self.mouse_move_event();
            }
            UI_POINTER_Y => {
                self.pointer.1 = extent as i32;
                self.mouse_move_event();
            }
            0x4000_0003 => {}
            UI_WHEEL_IN | UI_WHEEL_OUT => self.mouse_down_event(action),
            a if is_mouse_button(a) => self.mouse_down_event(action),
            _ => self.key_press_event(action),
        }
    }

    /// `UIElementManager::StopInput` (its `KeyReleaseEvent` for a key action,
    /// `crate::activation`).
    pub fn stop_input(&mut self, action: u32) {
        match action {
            UI_POINTER_X | UI_POINTER_Y => self.mouse_move_event(),
            0x4000_0003 | UI_WHEEL_IN | UI_WHEEL_OUT => {}
            a if is_mouse_button(a) => self.mouse_up_event(action),
            _ => self.key_release_event(action),
        }
    }

    /// `UIElementManager::MouseMoveEvent`; the manager's next `UseTime` runs `UpdateMouse`.
    fn mouse_move_event(&mut self) {
        self.tooltip.last_mouse_move = self.time;
        self.broadcast_global(GLOBAL_MOUSE_MOVE, 0);
        self.lists_global_mouse_move();
        self.scrollbar_mouse_move();
        // `MouseMoveEvent` checks the drag against the region under the pointer before the move
        // (`UpdateMouse` runs in the next `UseTime`).
        self.drag_mouse_move();
        self.movement_mouse_move();
        self.resize_mouse_move();
        self.text_drag_move();
        self.update_mouse();
    }

    /// `UIElementManager::UpdateMouse`.
    pub fn update_mouse(&mut self) {
        let mut top = None;
        let mut processed = false;
        self.new_tooltip_owner = None;
        for root in self.roots.clone() {
            if processed {
                break;
            }
            // The tooltip and the dragged copy are not under the pointer.
            if Some(root) == self.tooltip.element || Some(root) == self.drag.element {
                continue;
            }
            let (ox, oy) = self.el(root).object.unwrap_or((0, 0));
            let (x, y) = (self.pointer.0 - ox, self.pointer.1 - oy);
            self.notify_mouse_move(root, x, y, true, &mut top, &mut processed);
        }
        self.mouse_top = top;
        // Over nothing with a cursor: the default cursor.
        if top.is_none_or(|t| self.el(t).cursor.0 == 0) {
            self.use_default_pointer_cursor();
        }
        let owner = self.new_tooltip_owner.take();
        self.set_tooltip_owner(owner);
    }

    /// `UIRegion::NotifyMouseMove`, (`x`, `y`) in the parent's coordinates.
    fn notify_mouse_move(
        &mut self,
        index: usize,
        x: i32,
        y: i32,
        over_parent: bool,
        top: &mut Option<usize>,
        processed: &mut bool,
    ) {
        let el = self.el(index);
        let rect = el.rect;
        let mut over = over_parent
            && el.visible
            && (rect.x0..=rect.x1).contains(&x)
            && (rect.y0..=rect.y1).contains(&y);
        for child in el.children.clone() {
            if *processed {
                break;
            }
            self.notify_mouse_move(child, x - rect.x0, y - rect.y0, over, top, processed);
        }
        if self.new_tooltip_owner.is_none() && over && self.is_tooltip_on(index) {
            self.new_tooltip_owner = Some(index);
        }
        over &= self.el(index).mouse_visible;
        let mut is_top = false;
        if top.is_none() && over {
            *top = Some(index);
            is_top = true;
        }
        if over && !*processed {
            // `UIElement::MouseMove`.
            if self.el(index).mouse_over_top {
                self.broadcast(index, MSG_MOUSE_MOVE_TOP, 0);
            }
            self.broadcast(index, MSG_MOUSE_MOVE, 0);
        }
        if over != self.el(index).mouse_over && !*processed {
            // `UIElement::MouseOver`.
            self.el_mut(index).mouse_over = over;
            self.broadcast(index, MSG_MOUSE_OVER, i32::from(over));
        }
        if is_top != self.el(index).mouse_over_top && !*processed {
            // `UIElement::MouseOverTop`.
            self.el_mut(index).mouse_over_top = is_top;
            let (file, x, y) = self.el(index).cursor;
            if is_top && file != 0 {
                self.set_pointer_cursor(file, x, y, false);
            }
            self.broadcast(index, MSG_MOUSE_OVER_TOP, i32::from(is_top));
            self.button_update_state(index);
            self.panel_page_mouse_over_top(index, is_top);
            self.list_mouse_over_top(index, is_top);
            self.field_mouse_over_top(index, is_top);
            self.menu_update_state(index);
        }
    }

    /// `UIElementManager::MouseDownEvent`.
    fn mouse_down_event(&mut self, action: u32) {
        self.broadcast_global(GLOBAL_MOUSE_DOWN, action);
        self.menu_global_mouse(GLOBAL_MOUSE_DOWN);
        self.drag_mouse_down(action);
        // A press away from the active element deactivates it.
        if let Some(active) = self.active {
            if !self.el(active).mouse_over {
                self.deactivate(active);
            }
        }
        // So does a press away from the read-only text that held the copy focus.
        if let Some(focus) = self.copy_focus {
            if self.get(focus).is_none_or(|e| !e.mouse_over) {
                self.copy_focus = None;
            }
        }
        // Alt + press moves the window under the pointer; the press does not reach the controls
        // under it (`alt_move_mouse_down`).
        if self.alt_move_mouse_down(action) {
            self.update_mouse();
            return;
        }
        let mut processed = false;
        let wheel = matches!(action, UI_WHEEL_IN | UI_WHEEL_OUT);
        self.wheel_scrolled = false;
        for root in self.roots.clone() {
            self.notify_mouse_down(root, action, &mut processed);
        }
        // A turn that scrolled no element scrolls the window under the pointer (modern
        // exception, `window_wheel.rs`).
        if wheel && !self.wheel_scrolled {
            self.window_wheel(action);
        }
        self.update_mouse();
    }

    /// `UIRegion::NotifyMouseDown`.
    fn notify_mouse_down(&mut self, index: usize, action: u32, processed: &mut bool) {
        for child in self.el(index).children.clone() {
            if *processed {
                break;
            }
            self.notify_mouse_down(child, action, processed);
        }
        let wheel = matches!(action, UI_WHEEL_IN | UI_WHEEL_OUT);
        if !wheel && self.el(index).mouse_down.contains(&action) {
            if self.el(index).mouse_over {
                return;
            }
            if !*processed {
                self.element_mouse_up(index, action, processed);
            }
            self.el_mut(index).mouse_down.retain(|&a| a != action);
            return;
        }
        if self.el(index).mouse_over {
            if !*processed {
                self.element_mouse_down(index, action, processed);
            }
            let down = &mut self.el_mut(index).mouse_down;
            if !down.contains(&action) {
                down.push(action);
            }
        }
    }

    /// `UIElementManager::MouseUpEvent`.
    fn mouse_up_event(&mut self, action: u32) {
        self.broadcast_global(GLOBAL_MOUSE_UP, action);
        self.lists_global_mouse_up();
        self.menu_global_mouse(GLOBAL_MOUSE_UP);
        let mut processed = false;
        for root in self.roots.clone() {
            if processed {
                break;
            }
            self.notify_mouse_up(root, action, &mut processed);
        }
        self.update_mouse();
        self.drag_mouse_up(action);
        if action == UI_LEFT_BUTTON {
            // The release ends a move, by a drag bar or by Alt + drag, and a resize by its grip.
            self.end_movement();
            self.end_resizing();
        }
    }

    /// `UIRegion::NotifyMouseUp`.
    fn notify_mouse_up(&mut self, index: usize, action: u32, processed: &mut bool) {
        for child in self.el(index).children.clone() {
            if *processed {
                break;
            }
            self.notify_mouse_up(child, action, processed);
        }
        if self.el(index).mouse_down.contains(&action) {
            if !*processed {
                self.element_mouse_up(index, action, processed);
            }
            self.el_mut(index).mouse_down.retain(|&a| a != action);
        }
    }

    /// `UIElement::MouseDown` (and `UIElement_Button::MouseDown`).
    fn element_mouse_down(&mut self, index: usize, action: u32, processed: &mut bool) {
        if self.el(index).mouse_over_top {
            self.broadcast(index, MSG_MOUSE_PRESS_TOP, action as i32);
        }
        self.broadcast(index, MSG_MOUSE_PRESS, action as i32);
        let parent_is_root = self
            .el(index)
            .parent
            .is_some_and(|parent| self.el(parent).parent.is_none());
        if parent_is_root && !self.bool_attribute(index, MOUSE_EVENT_PASSTHROUGH) {
            *processed = true;
        }
        self.dragbar_mouse_down(index, action);
        self.resizebar_mouse_down(index, action);
        self.button_mouse_down(index, action, processed);
        self.menu_mouse_down(index, action, processed);
        self.text_mouse_down(index, action);
        self.tree_mouse_down(index, action);
        self.panel_page_mouse_down(index, action);
        self.list_mouse_down(index, action, processed);
        self.scrollbar_mouse_down(index, action);
    }

    /// `UIElement::MouseUp` (and `UIElement_Button::MouseUp`).
    fn element_mouse_up(&mut self, index: usize, action: u32, processed: &mut bool) {
        if self.el(index).mouse_over_top {
            self.broadcast(index, MSG_MOUSE_RELEASE_TOP, action as i32);
        }
        self.broadcast(index, MSG_MOUSE_RELEASE, action as i32);
        self.dragbar_mouse_up(index, action);
        self.button_mouse_up(index, action, processed);
        self.scrollbar_mouse_up(index);
        self.text_mouse_up(index, action);
    }
}
