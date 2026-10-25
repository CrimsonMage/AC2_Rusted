//! Drag and drop (`UIElementManager::CheckDragandDrop` / `StartDragandDrop` /
//! `StopDragandDrop`, `UIElement::DragItem` / `CatchDroppedItem` / `DragAndDropComplete`).
//!
//! A press records when and where (`MouseDownEvent`). While the left button is held, once it has
//! been held 0.3 seconds or the pointer has moved four pixels on an axis, the element under the
//! pointer starts a drag: one not `dragable` says `not_dragable` and passes the drag to its
//! parent; otherwise its parent (the drag's owner) makes a copy of it (`DragItem`: refused when
//! `disallow_drag_out`; the copy takes the element's state, attributes and images) and says
//! `start_drag_item`. The copy becomes its own root at the pointer, kept out of the pointer's
//! hits, and follows the pointer. On the release the element under the pointer catches it
//! (`CatchDroppedItem`: a `container` that does not `disallow_drag_in` says `stop_drag_item`
//! with the copy, dropped; any other passes it to its parent), and an owner whose copy was not
//! caught says `stop_drag_item`, not dropped. The copy then goes.
//!
//! The frameworks read a drop from the `stop_drag_item` message: its `item` is the copy, which
//! stays readable until the next global loop.
//!
//! Not modelled yet: the frameworks' drag callbacks (`m_dragOutCallback`, `m_dragDropCallback`;
//! the frameworks answer the messages instead) and drag selection in list boxes.

use std::sync::Arc;

use crate::element::UiTree;
use crate::mouse::UiMessage;
use crate::token::tokenize;

pub(crate) const MSG_START_DRAG_ITEM: u32 = tokenize("start_drag_item");
pub(crate) const MSG_STOP_DRAG_ITEM: u32 = tokenize("stop_drag_item");
const MSG_NOT_DRAGABLE: u32 = tokenize("not_dragable");
const DRAGABLE: u32 = tokenize("dragable");
const CONTAINER: u32 = tokenize("container");
const DISALLOW_DRAG_IN: u32 = tokenize("disallow_drag_in");
const DISALLOW_DRAG_OUT: u32 = tokenize("disallow_drag_out");

/// The left mouse button.
pub(crate) const LEFT: u32 = 0x4000_0004;

/// `UIElementManager`'s drag state.
#[derive(Debug, Clone, Default)]
pub(crate) struct DragState {
    /// `m_dragElement`, `m_dragOwner`, `m_dragXOffset` / `m_dragYOffset`.
    pub element: Option<usize>,
    pub owner: Option<usize>,
    pub offset: (i32, i32),
    /// `m_buttonDownTime` (`None` for -1) and `m_dragX` / `m_dragY`.
    pub button_down: Option<f64>,
    pub start: (i32, i32),
    pub left_down: bool,
    /// Copies dropped this frame, deleted on the next global loop.
    pub dropped: Vec<usize>,
}

impl UiTree {
    /// `MouseDownEvent`'s part: when and where the press was.
    pub(crate) fn drag_mouse_down(&mut self, action: u32) {
        self.drag.button_down = Some(self.time);
        self.drag.start = self.pointer;
        if action == LEFT {
            self.drag.left_down = true;
        }
    }

    /// `MouseMoveEvent`'s part: the copy follows the pointer, then `CheckDragandDrop`.
    pub(crate) fn drag_mouse_move(&mut self) {
        if let Some(copy) = self.drag.element {
            let (x, y) = self.pointer;
            let (dx, dy) = self.drag.offset;
            self.set_object_position(copy, x + dx, y + dy);
        }
        self.check_drag_and_drop();
    }

    /// `MouseUpEvent`'s part, after the pointer pass: the drop.
    pub(crate) fn drag_mouse_up(&mut self, action: u32) {
        if action == LEFT {
            self.drag.left_down = false;
        }
        self.stop_drag_and_drop();
    }

    /// `UIElementManager::CheckDragandDrop`.
    pub(crate) fn check_drag_and_drop(&mut self) {
        if self.drag.element.is_some() {
            return;
        }
        // The region from the last pointer pass may have gone since.
        let Some(top) = self.mouse_top.filter(|&t| self.get(t).is_some()) else {
            return;
        };
        let at = self.screen_box(top);
        let (sx, sy) = self.drag.start;
        self.start_drag_and_drop(top, at.x0 - sx, at.y0 - sy);
    }

    /// `UIElementManager::StartDragandDrop`.
    fn start_drag_and_drop(&mut self, element: usize, offset_x: i32, offset_y: i32) -> bool {
        let Some(down) = self.drag.button_down else {
            return false;
        };
        let (sx, sy) = self.drag.start;
        if !self.drag.left_down || sx < 0 || sy < 0 || sx > self.display.0 || sy > self.display.1 {
            return false;
        }
        let (dx, dy) = (self.pointer.0 - sx, self.pointer.1 - sy);
        if !(0.3 <= self.time - down || 15 < dx * dx || 15 < dy * dy) {
            return false;
        }
        if !self.bool_attribute(element, DRAGABLE) {
            self.broadcast(element, MSG_NOT_DRAGABLE, 0);
            return match self.el(element).parent {
                Some(parent) => self.start_drag_and_drop(parent, offset_x, offset_y),
                None => false,
            };
        }
        let owner = self.el(element).parent;
        let copy = match owner {
            Some(owner) => self.drag_item(owner, element),
            None => None,
        };
        let Some(copy) = copy.filter(|&c| c != element) else {
            return false;
        };
        self.clear_drag_and_drop();
        let rect = self.el(copy).rect;
        let (width, height) = (rect.width(), rect.height());
        let offset = (offset_x.clamp(0, width), offset_y.clamp(0, height));
        let (x, y) = (self.pointer.0 + offset.0, self.pointer.1 + offset.1);
        self.set_parent(copy, None);
        self.move_to(copy, 0, 0);
        let el = self.el_mut(copy);
        el.object = Some((x, y));
        el.z = 0xFFFF;
        self.add_root(copy);
        self.set_visible(copy, true);
        self.drag.element = Some(copy);
        self.drag.owner = owner;
        self.drag.offset = offset;
        self.moved = true;
        true
    }

    /// `UIElement::DragItem` of `owner`: a copy of `item` matched to it, or none when the owner
    /// does not let it out.
    fn drag_item(&mut self, owner: usize, item: usize) -> Option<usize> {
        if self.bool_attribute(owner, DISALLOW_DRAG_OUT) {
            return None;
        }
        let copy = self.copy_element(item)?;
        self.broadcast(owner, MSG_START_DRAG_ITEM, 0);
        Some(copy)
    }

    /// `UIElement::CopyElem`, then `SetState`, `MatchAttributes` and `MatchGraphics`.
    fn copy_element(&mut self, item: usize) -> Option<usize> {
        let el = self.el(item);
        let (layout, desc, state) = (Arc::clone(&el.layout), Arc::clone(&el.desc), el.state);
        let copy = self.create_element(layout, desc, true)?;
        self.set_state(copy, state);
        let attributes = self.el(item).attributes.clone();
        self.el_mut(copy).attributes.extend(attributes);
        self.match_graphics(copy, item);
        Some(copy)
    }

    /// `UIElement::MatchGraphics`: the image, then each child's from the child of the same
    /// description.
    fn match_graphics(&mut self, copy: usize, item: usize) {
        let (image, alpha, value) = {
            let el = self.el(item);
            (el.image, el.alpha_image, el.alpha_value)
        };
        if image.is_some() {
            let el = self.el_mut(copy);
            el.image = image;
            el.alpha_image = alpha;
            el.alpha_value = value;
        }
        for child in self.el(copy).children.clone() {
            let desc = self.el(child).desc.id;
            let source = self
                .el(item)
                .children
                .iter()
                .copied()
                .find(|&c| self.el(c).desc.id == desc);
            if let Some(source) = source {
                self.match_graphics(child, source);
            }
        }
    }

    /// `UIElementManager::StopDragandDrop`.
    pub(crate) fn stop_drag_and_drop(&mut self) {
        let Some(copy) = self.drag.element else {
            return;
        };
        let dropped = match self.mouse_top.filter(|&t| self.get(t).is_some()) {
            Some(target) => self.catch_dropped_item(target, copy),
            None => false,
        };
        if let Some(owner) = self.drag.owner {
            // `UIElement::DragAndDropComplete`.
            if !dropped {
                self.broadcast_item(owner, MSG_STOP_DRAG_ITEM, 0, copy);
            }
        }
        self.clear_drag_and_drop();
    }

    /// `UIElement::CatchDroppedItem` (and `UIElement_Field`'s).
    fn catch_dropped_item(&mut self, element: usize, item: usize) -> bool {
        if element == item {
            return false;
        }
        if self.bool_attribute(element, CONTAINER)
            && !self.bool_attribute(element, DISALLOW_DRAG_IN)
        {
            self.broadcast_item(element, MSG_STOP_DRAG_ITEM, 1, item);
            return true;
        }
        match self.el(element).parent {
            Some(parent) => self.catch_dropped_item(parent, item),
            None => false,
        }
    }

    /// `UIElementManager::ClearDragandDrop`: the copy leaves the screen (it is deleted on the
    /// next global loop, after the frameworks read the drop).
    fn clear_drag_and_drop(&mut self) {
        self.drag.owner = None;
        if let Some(copy) = self.drag.element.take() {
            self.roots.retain(|&r| r != copy);
            self.drag.dropped.push(copy);
            self.changed = true;
            self.moved = true;
        }
    }

    /// The copies dropped since the last global loop, deleted.
    pub(crate) fn drag_use_time(&mut self) {
        for copy in std::mem::take(&mut self.drag.dropped) {
            if self.get(copy).is_some() {
                self.delete(copy);
            }
        }
        if self.drag.left_down {
            self.check_drag_and_drop();
        }
    }

    /// `IUIManager`'s `StartDragandDrop` of `element` from code (`gmPaperDollUI::
    /// HandlePaperDollDrag`): a drag of it starts when the held button would start one.
    pub fn start_drag(&mut self, element: usize) -> bool {
        self.drag.element.is_none() && self.start_drag_and_drop(element, 0, 0)
    }

    /// The element being dragged (the copy at the pointer), if any.
    pub fn dragged(&self) -> Option<usize> {
        self.drag.element
    }

    fn broadcast_item(&mut self, index: usize, message: u32, data: i32, item: usize) {
        let element_id = self.el(index).id();
        self.messages.push(UiMessage {
            element_id,
            element: Some(index),
            message,
            data,
            item: Some(item),
        });
    }
}
