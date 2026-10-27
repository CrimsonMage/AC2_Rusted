//! `UIElement_ListBox`'s selection: one selected item (`m_pSelectedItem`). Selecting another
//! item turns the old one to the list's `item_normal_state` and the new one to its
//! `item_selected_state` when `selected_item_state_change` is set, broadcasts `new_selection`
//! with the new index (-1 for none) when asked, and lays the list out again
//! (`SetSelectedItem`). Selecting by index also scrolls the item into view (`SetSelectedIndex`,
//! `ScrollToShow`). A left press on a `click_select` list selects the item under the pointer
//! (`MouseDown`, `HandleSelection`) and is not passed on unless `mouse_event_passthrough`.
//!
//! Not modelled yet: drag selection (`drag_select`), keyboard stepping (`StartActive`) and the
//! mouse wheel's scroll bar.

use crate::element::UiTree;
use crate::layout::AttributeValue;
use crate::mouse::{UI_WHEEL_IN, UI_WHEEL_OUT};
use crate::token::{tokenize, MOUSE_EVENT_PASSTHROUGH};

pub(crate) const LIST_BOX: u32 = tokenize("listbox");
/// `ListBoxMsg_new_selection`.
pub const MSG_NEW_SELECTION: u32 = tokenize("new_selection");
const ITEM_NORMAL_STATE: u32 = tokenize("item_normal_state");
const ITEM_SELECTED_STATE: u32 = tokenize("item_selected_state");
const SELECTED_ITEM_STATE_CHANGE: u32 = tokenize("selected_item_state_change");
/// `ListBox_click_select`.
pub const CLICK_SELECT: u32 = tokenize("click_select");
/// The left button actions (`0x40000004`, `0x40000007` for a double click).
const LEFT: [u32; 2] = [0x4000_0004, 0x4000_0007];

impl UiTree {
    /// `UIElement_ListBox::GetSelectedIndex`.
    pub fn list_selected_index(&self, list: usize) -> Option<usize> {
        let selected = self.el(list).list.as_ref()?.selected?;
        self.list_what_num(list, selected)
    }

    /// `m_pSelectedItem`.
    pub fn list_selected_item(&self, list: usize) -> Option<usize> {
        self.el(list).list.as_ref()?.selected
    }

    fn id_attribute(&self, index: usize, id: u32) -> Option<u32> {
        match self.el(index).attributes.get(&id) {
            Some(AttributeValue::Id(value)) => Some(*value),
            _ => None,
        }
    }

    /// `UIElement_ListBox::SetSelectedItem`: `item` (not one of the items: none) selected.
    pub fn list_set_selected_item(&mut self, list: usize, item: Option<usize>, broadcast: bool) {
        let old = self.list_selected_item(list);
        if old == item {
            return;
        }
        let index = item.and_then(|item| self.list_what_num(list, item));
        let new = index.and(item);
        self.list(list).selected = new;
        if self.bool_attribute(list, SELECTED_ITEM_STATE_CHANGE) {
            if let (Some(old), Some(state)) = (old, self.id_attribute(list, ITEM_NORMAL_STATE)) {
                if self.get(old).is_some() {
                    self.set_state(old, state);
                }
            }
            if let (Some(new), Some(state)) = (new, self.id_attribute(list, ITEM_SELECTED_STATE)) {
                self.set_state(new, state);
            }
        }
        if broadcast {
            let data = index.map_or(-1, |i| i as i32);
            self.broadcast(list, MSG_NEW_SELECTION, data);
            self.menu_list_selection(list);
        }
        self.list_update_layout(list);
    }

    /// `UIElement_ListBox::SetSelectedIndex`: the item at `index` selected and scrolled into
    /// view; past the end, none.
    pub fn list_set_selected_index(&mut self, list: usize, index: Option<usize>, broadcast: bool) {
        let item = index.and_then(|i| self.list_items(list).get(i).copied());
        match (item, index) {
            (Some(item), Some(index)) => {
                self.list_set_selected_item(list, Some(item), broadcast);
                self.list_scroll_to_show(list, index);
            }
            _ => self.list_set_selected_item(list, None, broadcast),
        }
    }

    /// `UIElement_ListBox::ScrollToShow(long, int)`, immediately: the view moves the least
    /// whole number of columns and rows that brings item `index` into the list's box.
    pub fn list_scroll_to_show(&mut self, list: usize, index: usize) {
        let Some(state) = self.el(list).list.as_ref() else {
            return;
        };
        if index >= state.items.len() || state.rows < 1 || state.cols < 1 {
            return;
        }
        let horizontal = self.list_flag(list, "horizontal");
        let i = index as i32;
        let (col, row) = if horizontal {
            (i % state.cols, i / state.cols)
        } else {
            (i / state.rows, i % state.rows)
        };
        let (item, view) = (state.item, state.view);
        let rect = self.el(list).rect;
        if item.0 * col < view.0 {
            self.list_set_view(list, Some(item.0 * col), None);
        } else if item.0 > 0 && view.0 / item.0 + rect.width() / item.0 <= col {
            self.list_set_view(list, Some(item.0 * (col - rect.width() / item.0 + 1)), None);
        }
        if item.1 * row < view.1 {
            self.list_scroll_to_row(list, row);
        } else if item.1 > 0 && view.1 / item.1 + rect.height() / item.1 <= row {
            self.list_scroll_to_row(list, row - rect.height() / item.1 + 1);
        }
    }

    /// `UIElement_ListBox::MouseDown` after `UIElement::MouseDown`: a left press selects the
    /// item under the pointer (`HandleSelection`) and is taken unless the list passes it on.
    pub(crate) fn list_mouse_down(&mut self, index: usize, action: u32, processed: &mut bool) {
        if self.el(index).desc.kind != LIST_BOX {
            return;
        }
        let wheel = matches!(action, UI_WHEEL_IN | UI_WHEEL_OUT);
        if !wheel && !LEFT.contains(&action) {
            return;
        }
        if wheel {
            self.list_wheel(index, action);
        } else if self.bool_attribute(index, CLICK_SELECT) {
            if let Some(item) = self.list_item_under_mouse(index) {
                self.list_set_selected_item(index, Some(item), true);
            }
        }
        if !self.bool_attribute(index, MOUSE_EVENT_PASSTHROUGH) {
            *processed = true;
        }
    }
}
