//! Element placement: `UIElement::UpdateSizeAndPosition`, `ResizeTo`, `MoveTo` and `SetZLevel`.
//!
//! An element's authored box (`GetOriginalPosition`: the current state's box, else the stateless
//! state's, else the element's own) was laid out against its parent's authored box (a root's
//! against the layout's display size). Each edge follows the parent's change in size by its
//! anchor: left/top `2` move with the far side, `3` centre; right/bottom `1` move with the far
//! side, `3` centre; any other value keeps the authored coordinate. Once an element has a size,
//! an edge anchored `0` keeps where it is.

use std::sync::Arc;

use crate::element::{state_of, UiBox, UiTree};
use crate::layout::{AttributeValue, ElementDesc};
use crate::token::{MAX_HEIGHT, MAX_WIDTH, MIN_HEIGHT, MIN_WIDTH, STATELESS};

impl UiTree {
    /// `UIElement::GetPositionFromState`.
    fn position_from_state(desc: &ElementDesc, state: u32) -> Option<(UiBox, i32)> {
        let state = state_of(desc, state)?;
        (state.rect.x0 != -1).then(|| {
            let r = state.rect;
            (
                UiBox::new(r.x0.into(), r.y0.into(), r.x1.into(), r.y1.into()),
                state.z_level.into(),
            )
        })
    }

    /// `UIElement::GetPositionFromDesc` for an element in state `state`.
    fn position_from_desc(desc: &ElementDesc, state: u32) -> (UiBox, i32) {
        Self::position_from_state(desc, state)
            .or_else(|| Self::position_from_state(desc, STATELESS))
            .unwrap_or_else(|| {
                let r = desc.rect;
                (
                    UiBox::new(r.x0.into(), r.y0.into(), r.x1.into(), r.y1.into()),
                    desc.z_level.into(),
                )
            })
    }

    /// `UIElement::GetParentOriginalPosition`: the parent's authored box from the description
    /// that lists the child (its reference's current or stateless state, else its own).
    fn parent_original_position(&self, parent: usize, child_id: u32) -> UiBox {
        let el = self.el(parent);
        let lists = |desc: &ElementDesc, state: u32| {
            state_of(desc, state).is_some_and(|s| s.children.iter().any(|c| c.id == child_id))
        };
        let from_reference = lists(&el.reference, el.state) || lists(&el.reference, STATELESS);
        let desc: &Arc<ElementDesc> = if from_reference {
            &el.reference
        } else {
            &el.desc
        };
        Self::position_from_desc(desc, el.state).0
    }

    /// The top-left of an element's place in its layout (`UIElement`'s authored rectangle, before
    /// a saved window position or a move): the offset a docked part keeps from its window.
    pub(crate) fn layout_origin(&self, index: usize) -> (i32, i32) {
        let el = self.el(index);
        let (original, _) = Self::position_from_desc(&el.reference, el.state);
        (original.x0, original.y0)
    }

    /// The element's box on the screen.
    pub fn screen_box(&self, index: usize) -> UiBox {
        let el = self.el(index);
        let (x, y) = match (el.object, el.parent) {
            (Some(origin), _) => origin,
            (None, Some(parent)) => {
                let parent = self.screen_box(parent);
                (parent.x0, parent.y0)
            }
            (None, None) => (0, 0),
        };
        el.rect.offset(x, y)
    }

    pub(crate) fn int_attribute(&self, index: usize, id: u32) -> Option<i32> {
        match self.el(index).attributes.get(&id) {
            Some(AttributeValue::Int(value)) => Some(*value),
            _ => None,
        }
    }

    /// `UIElement::UpdateSizeAndPosition`.
    pub(crate) fn update_size_and_position(&mut self, index: usize) {
        let el = self.el(index);
        let (original_parent, current_parent) = match el.parent {
            None => (
                UiBox::new(
                    0,
                    0,
                    i32::from(el.layout.width) - 1,
                    i32::from(el.layout.height) - 1,
                ),
                UiBox::new(0, 0, self.display.0 - 1, self.display.1 - 1),
            ),
            Some(parent) => (
                self.parent_original_position(parent, el.id()),
                self.el(parent).rect,
            ),
        };
        let (original, z) = Self::position_from_desc(&el.reference, el.state);
        let mut current = original;
        let grow_x =
            (current_parent.x1 - current_parent.x0) - (original_parent.x1 - original_parent.x0);
        let parent_height = current_parent.y1 - current_parent.y0;
        let grow_y = parent_height - (original_parent.y1 - original_parent.y0);
        let width = original.x1 - original.x0;
        let height = original.y1 - original.y0;
        let parent_width = current_parent.x1 - current_parent.x0;
        let edges = el.reference.edges;
        match edges[0] {
            2 => current.x0 = original.x0 + grow_x,
            3 => current.x0 = (parent_width + 1) / 2 - (width + 1) / 2,
            _ => {}
        }
        match edges[2] {
            1 => {
                current.x1 = grow_x + original.x1;
                if current.x1 < 1 {
                    current.x1 = original.x1;
                }
            }
            3 => current.x1 = (width + 1) / 2 - 1 + (parent_width + 1) / 2,
            _ => {}
        }
        match edges[1] {
            2 => current.y0 = original.y0 + grow_y,
            3 => current.y0 = (parent_height + 1) / 2 - (height + 1) / 2,
            _ => {}
        }
        match edges[3] {
            1 => {
                current.y1 = grow_y + original.y1;
                if current.y1 < 1 {
                    current.y1 = original.y1;
                }
            }
            3 => current.y1 = (height + 1) / 2 - 1 + (parent_height + 1) / 2,
            _ => {}
        }
        let sized = el.rect.x1 - el.rect.x0 != -1 || el.rect.y1 - el.rect.y0 != -1;
        if sized {
            let now = if el.object.is_some() {
                self.screen_box(index)
            } else {
                el.rect
            };
            for (edge, value, kept) in [
                (edges[0], &mut current.x0, now.x0),
                (edges[1], &mut current.y0, now.y0),
                (edges[2], &mut current.x1, now.x1),
                (edges[3], &mut current.y1, now.y1),
            ] {
                if edge == 0 {
                    *value = kept;
                }
            }
        }
        self.resize_to(index, current.width(), current.height());
        self.move_to(index, current.x0, current.y0);
        self.set_z_level(index, z);
    }

    /// `UIElement::ResizeTo`: clamped by `max_height`, `max_width`, `min_height` and
    /// `min_width`, then every child placed again.
    pub fn resize_to(&mut self, index: usize, width: i32, height: i32) {
        let (mut width, mut height) = (width, height);
        if let Some(max) = self.int_attribute(index, MAX_HEIGHT) {
            height = height.min(max);
        }
        if let Some(max) = self.int_attribute(index, MAX_WIDTH) {
            width = width.min(max);
        }
        if let Some(min) = self.int_attribute(index, MIN_HEIGHT) {
            height = height.max(min);
        }
        if let Some(min) = self.int_attribute(index, MIN_WIDTH) {
            width = width.max(min);
        }
        // `UIRegion::ResizeTo`.
        let el = self.el_mut(index);
        el.rect.x1 = el.rect.x0 - 1 + width;
        el.rect.y1 = el.rect.y0 - 1 + height;
        // `UIElement_Text::ResizeTo`.
        if let Some(text) = el.text.as_mut() {
            text.determine_margins(width, height);
            text.dirty = true;
        }
        for child in self.el(index).children.clone() {
            self.update_size_and_position(child);
        }
        // `UIElement_Scrollbar::ResizeTo`.
        self.scrollbar_update_layout(index);
        // `UIElement_Meter::ResizeTo`.
        self.meter_update_layout(index);
        // `UIElement_ListBox::ResizeTo`: laid out again, the view kept in bounds, the controls
        // updated.
        if crate::list_box_select::is_list_kind(self.el(index).desc.kind)
            && self.el(index).list.is_some()
        {
            self.list_relayout(index);
        }
    }

    /// `UIElement::MoveTo`: a root keeps its box at the origin of its surface object, which is
    /// kept on the display; any other element moves its box.
    pub fn move_to(&mut self, index: usize, x: i32, y: i32) {
        let display = self.display;
        let el = self.el_mut(index);
        let mut object_moved = false;
        let (x, y) = match el.object {
            Some(old) => {
                let x = x.max(0).min(display.0 - 1 + (el.rect.x0 - el.rect.x1));
                let y = y.max(0).min(display.1 - 1 + (el.rect.y0 - el.rect.y1));
                object_moved = old != (x, y);
                el.object = Some((x, y));
                (0, 0)
            }
            None => (x, y),
        };
        // `UIRegion::MoveTo`.
        let rect = el.rect;
        el.rect = rect.offset(x - rect.x0, y - rect.y0);
        let rect_moved = el.rect != rect;
        // A root's surface object is placed again (the window follows a drag frame by frame); a
        // child's new place is drawn by composing again.
        if object_moved {
            self.moved = true;
        } else if rect_moved {
            self.changed = true;
        }
        // `UIElement_Meter::MoveTo`.
        self.meter_update_layout(index);
    }

    /// `UIElement::SetZLevel`: a new z re-sorts the element among its siblings (or the roots).
    pub fn set_z_level(&mut self, index: usize, z: i32) {
        if self.el(index).z == z {
            return;
        }
        self.el_mut(index).z = z;
        if self.roots.contains(&index) {
            self.add_root(index);
        } else if let Some(parent) = self.el(index).parent {
            self.set_parent(index, Some(parent));
        }
    }

    /// The render device changed size: every root placed again (`UIElementManager::RefreshEvent`'s
    /// `UpdateSizeAndPosition` per root, without its `0xffff0007` / `0xffff0008` broadcasts).
    pub fn set_display(&mut self, width: i32, height: i32) {
        self.display = (width, height);
        for root in self.roots.clone() {
            self.update_size_and_position(root);
        }
    }

    /// `UIElement::GetRootElement`.
    pub fn root_of(&self, mut index: usize) -> usize {
        while let Some(parent) = self.el(index).parent {
            index = parent;
        }
        index
    }
}
