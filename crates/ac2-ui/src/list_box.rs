//! `UIElement_ListBox`: items in a grid of equal cells, the first item's size
//! (`UpdateLayout`). There are `max_columns` columns (at least one) and as many rows as the items
//! need; a `horizontal` list fills each row across, otherwise each column down. The items sit
//! at their cells less the view (`x_view`, `y_view`), which stays within the paper and, unless
//! the list moves smoothly, on a cell boundary (`ConstrainViewValue`). Adding, inserting,
//! replacing and removing items lay the list out again unless a multi-operation is open; a
//! change of view broadcasts `new_view` (`ViewChanged`).
//!
//! The scroll bars (`list_scroll.rs`) drive and follow the view; smooth movement is in
//! `list_animate.rs`, drag selection and rollover in `list_drag.rs`.
//!
//! Not modelled yet: `allow_updating` turned off.

use crate::element::UiTree;
use crate::layout::AttributeValue;
use crate::token::tokenize;

const MSG_NEW_VIEW: u32 = tokenize("new_view");

/// A list box's own state (`m_listItems`, the layout and the view record).
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct ListBoxState {
    pub items: Vec<usize>,
    pub cols: i32,
    pub rows: i32,
    pub item: (i32, i32),
    pub paper: (i32, i32),
    /// `m_nXViewRecord`, `m_nYViewRecord`.
    pub view: (i32, i32),
    /// Bit 0x200000: a multi-operation is open.
    pub multiop: bool,
    /// `m_pSelectedItem` (`list_box_select.rs`).
    pub selected: Option<usize>,
    /// The menu whose drop-down this list is (`UIElement_Menu`'s listener on `new_selection`).
    pub menu: Option<usize>,
    /// Bit 0x20000: the view is being set from a scroll bar (`list_scroll.rs`).
    pub from_scrollbar: bool,
    /// A sized list's column widths and row heights (`m_nItemWidths`, `m_nItemHeights`).
    pub col_widths: Vec<i32>,
    pub row_heights: Vec<i32>,
    /// Bit 0x8000 and the smooth movement's ends and times (`list_animate.rs`).
    pub anim: Option<crate::list_animate::ListAnim>,
    /// Bit 0x80000: the global loop set the view, so a view set from elsewhere stops the
    /// animation.
    pub anim_moved: bool,
    /// Bit 0x10000: a drag selection is open (`list_drag.rs`).
    pub drag_selecting: bool,
    /// Bit 0x100000: a dragged item is rolling over the list.
    pub rollover: bool,
    /// `m_dragLastOver` with `m_lastDragOverState`.
    pub drag_last_over: Option<(usize, u32)>,
}

impl UiTree {
    pub(crate) fn list(&mut self, list: usize) -> &mut ListBoxState {
        self.el_mut(list).list.get_or_insert_with(Default::default)
    }

    pub(crate) fn list_flag(&self, list: usize, name: &str) -> bool {
        matches!(
            self.el(list).attributes.get(&tokenize(name)),
            Some(AttributeValue::Bool(true))
        )
    }

    pub(crate) fn list_int(&self, list: usize, name: &str) -> Option<i32> {
        match self.el(list).attributes.get(&tokenize(name)) {
            Some(AttributeValue::Int(v)) => Some(*v),
            _ => None,
        }
    }

    /// The items, first to last.
    pub fn list_items(&self, list: usize) -> &[usize] {
        self.el(list).list.as_ref().map_or(&[], |l| &l.items)
    }

    /// `UIElement_ListBox::WhatNum`: an item's index.
    pub fn list_what_num(&self, list: usize, item: usize) -> Option<usize> {
        self.list_items(list).iter().position(|&i| i == item)
    }

    /// `UIElement_ListBox::MultiOpStart` (the multi-op adds and removes).
    pub fn list_multi_op_start(&mut self, list: usize) {
        self.list(list).multiop = true;
    }

    /// `UIElement_ListBox::MultiOpComplete`.
    pub fn list_multi_op_complete(&mut self, list: usize) {
        self.list(list).multiop = false;
        self.list_relayout(list);
    }

    /// `UpdateLayout`, `KeepViewInBounds` (and `UpdateControls`) after a change, unless a
    /// multi-operation is open.
    fn list_changed(&mut self, list: usize) {
        if !self.list(list).multiop {
            self.list_relayout(list);
        }
    }

    pub(crate) fn list_relayout(&mut self, list: usize) {
        self.list_update_layout(list);
        self.list_keep_view_in_bounds(list);
        self.list_update_controls(list);
    }

    /// `UIElement_ListBox::AddItem`.
    pub fn list_add_item(&mut self, list: usize, item: usize) {
        let at = self.list_items(list).len();
        self.list_insert_item(list, item, at);
    }

    /// `UIElement_ListBox::InsertItem` (`InsertOrderedByIndex`: at the end when past it).
    pub fn list_insert_item(&mut self, list: usize, item: usize, at: usize) {
        let state = self.list(list);
        let at = at.min(state.items.len());
        state.items.insert(at, item);
        self.list_changed(list);
    }

    /// `UIElement_ListBox::SetItem`: the item at `index` replaced (deleted) by `item`.
    pub fn list_set_item(&mut self, list: usize, item: usize, index: usize) {
        if let Some(&old) = self.list_items(list).get(index) {
            self.list_delete_item(list, old);
        }
        self.list_insert_item(list, item, index);
    }

    /// `UIElement_ListBox::RemoveItem`: the item taken out (no longer a child), not deleted.
    pub fn list_remove_item(&mut self, list: usize, index: usize) -> Option<usize> {
        let state = self.list(list);
        if index >= state.items.len() {
            return None;
        }
        let item = state.items.remove(index);
        if state.selected == Some(item) {
            state.selected = None;
        }
        self.set_parent(item, None);
        self.list_changed(list);
        Some(item)
    }

    /// `UIElement_ListBox::DeleteItem`.
    pub fn list_delete_item(&mut self, list: usize, item: usize) -> bool {
        let Some(index) = self.list_what_num(list, item) else {
            return false;
        };
        match self.list_remove_item(list, index) {
            Some(item) => {
                self.delete(item);
                true
            }
            None => false,
        }
    }

    /// `UIElement_ListBox::Flush`: every item deleted, the view home.
    pub fn list_flush(&mut self, list: usize) {
        let state = self.list(list);
        state.selected = None;
        let items = std::mem::take(&mut state.items);
        for item in items {
            self.set_parent(item, None);
            self.delete(item);
        }
        self.list_set_view(list, Some(0), Some(0));
        self.list_relayout(list);
    }

    /// `UIElement_ListBox::Home`.
    pub fn list_home(&mut self, list: usize) {
        self.list_set_view(list, Some(0), Some(0));
    }

    /// `UIElement_ListBox::End`.
    pub fn list_end(&mut self, list: usize) {
        let paper = self.list(list).paper;
        self.list_set_view(list, Some(paper.0), Some(paper.1));
    }

    /// `UIElement_ListBox::ScrollToRow` (immediately; smooth scrolling is not modelled).
    pub fn list_scroll_to_row(&mut self, list: usize, row: i32) {
        if self.is_msize(list) {
            self.list_msize_scroll_to(list, true, row);
            return;
        }
        let height = self.list(list).item.1;
        self.list_set_view(list, None, Some(height * row));
    }

    /// `UIElement_ListBox::GetTopRow`.
    pub fn list_top_row(&self, list: usize) -> i32 {
        let Some(state) = self.el(list).list.as_ref() else {
            return 0;
        };
        if state.item.1 < 1 {
            0
        } else {
            state.view.1 / state.item.1
        }
    }

    /// `UIElement_ListBox::ConstrainViewValue`.
    pub(crate) fn list_constrain(&self, list: usize, x: bool, value: i32) -> i32 {
        if self.is_msize(list) {
            return self.list_msize_constrain(list, x, value);
        }
        let Some(state) = self.el(list).list.as_ref() else {
            return 0;
        };
        let (size, paper) = if x {
            (state.item.0, state.paper.0)
        } else {
            (state.item.1, state.paper.1)
        };
        if size == 0 || value < 0 {
            return 0;
        }
        let value = value.min(paper);
        if self.list_flag(list, "smooth_movement") {
            value
        } else {
            (size / 2 + value) / size * size
        }
    }

    /// `SetAttribute_Int(x_view / y_view)`: constrained, then `ViewChanged`.
    pub fn list_set_view(&mut self, list: usize, x: Option<i32>, y: Option<i32>) {
        // A view set while the global loop is moving the list stops the movement.
        if self
            .el(list)
            .list
            .as_ref()
            .is_some_and(|s| s.anim.is_some() && s.anim_moved)
        {
            self.list_stop_animation(list);
        }
        let x = x.map(|v| self.list_constrain(list, true, v));
        let y = y.map(|v| self.list_constrain(list, false, v));
        let attrs = &mut self.el_mut(list).attributes;
        if let Some(x) = x {
            attrs.insert(tokenize("x_view"), AttributeValue::Int(x));
        }
        if let Some(y) = y {
            attrs.insert(tokenize("y_view"), AttributeValue::Int(y));
        }
        self.list_view_changed(list);
    }

    /// `UIElement_ListBox::ViewChanged`.
    fn list_view_changed(&mut self, list: usize) {
        let view = (
            self.list_int(list, "x_view").unwrap_or(0),
            self.list_int(list, "y_view").unwrap_or(0),
        );
        let state = self.list(list);
        if state.view != view {
            state.view = view;
            self.list_relayout(list);
            self.broadcast(list, MSG_NEW_VIEW, 0);
        }
    }

    /// `UIElement_ListBox::KeepViewInBounds`.
    fn list_keep_view_in_bounds(&mut self, list: usize) {
        let x = self.list_int(list, "x_view").unwrap_or(0);
        let y = self.list_int(list, "y_view").unwrap_or(0);
        let (nx, ny) = (
            self.list_constrain(list, true, x),
            self.list_constrain(list, false, y),
        );
        if (nx, ny) != (x, y) {
            self.list_set_view(list, Some(nx), Some(ny));
        }
    }

    /// `UIElement_ListBox::UpdateLayout`.
    pub(crate) fn list_update_layout(&mut self, list: usize) {
        if self.el(list).attributes.get(&tokenize("allow_updating"))
            == Some(&AttributeValue::Bool(false))
        {
            return;
        }
        if self.is_msize(list) {
            self.list_msize_layout(list);
            return;
        }
        let horizontal = self.list_flag(list, "horizontal");
        let max_columns = self.list_int(list, "max_columns").unwrap_or(1).max(1);
        let items = self.list_items(list).to_vec();
        let state = self.list(list);
        state.cols = 1;
        state.rows = 1;
        state.paper = (0, 0);
        if let Some(&first) = items.first() {
            let rect = self.el(first).rect;
            let item = (rect.width(), rect.height());
            let cols = max_columns;
            let rows = ((items.len() as f64 / f64::from(cols)).ceil() as i32).max(1);
            let state = self.list(list);
            state.item = item;
            state.cols = cols;
            state.rows = rows;
            let view = state.view;
            let (mut col, mut row) = (0, 0);
            for &element in &items {
                if self.el(element).parent != Some(list) {
                    self.set_parent(element, Some(list));
                }
                self.move_to(element, item.0 * col - view.0, item.1 * row - view.1);
                if horizontal {
                    if col == cols - 1 {
                        col = 0;
                        row += 1;
                    } else {
                        col += 1;
                    }
                } else if row == rows - 1 {
                    row = 0;
                    col += 1;
                } else {
                    row += 1;
                }
            }
        }
        let state = self.list(list);
        state.paper = (state.item.0 * state.cols, state.item.1 * state.rows);
        self.changed = true;
    }

    /// `UIElement_ListBox::GetItemIndexAtPoint` for a point in screen pixels.
    pub fn list_index_at(&self, list: usize, x: i32, y: i32) -> Option<usize> {
        if self.is_msize(list) {
            return self.list_msize_index_at(list, x, y);
        }
        let state = self.el(list).list.as_ref()?;
        let b = self.screen_box(list);
        if x < b.x0 || x > b.x1 || y < b.y0 || y > b.y1 || state.item.0 < 1 || state.item.1 < 1 {
            return None;
        }
        let (px, py) = (state.view.0 + x - b.x0, state.view.1 + y - b.y0);
        if px >= state.cols * state.item.0 || py >= state.rows * state.item.1 {
            return None;
        }
        let (col, row) = (px / state.item.0, py / state.item.1);
        let index = if self.list_flag(list, "horizontal") {
            state.cols * row + col
        } else {
            state.rows * col + row
        };
        (index >= 0 && (index as usize) < state.items.len()).then_some(index as usize)
    }

    /// `UIElement_ListBox::GetItemUnderMouse`.
    pub fn list_item_under_mouse(&self, list: usize) -> Option<usize> {
        let (x, y) = self.pointer;
        let index = self.list_index_at(list, x, y)?;
        self.list_items(list).get(index).copied()
    }
}
