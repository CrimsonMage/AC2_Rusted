//! `UIElement_ListBox`'s goal steps and column queries: the line and page steps the scroll bar's
//! messages and buttons call (`VerticalLineIncrement` ... `HorizontalPageDecrement`), the column
//! of an item (`GetCol`, `GetLeftCol`, `ScrollToCol`), the scroll share (`GetPercentageH`,
//! `GetPercentageV`), selection by element id and the multi-operation wrappers.
//!
//! Every step computes a goal from the current view (from the goal while the list moves) and
//! sets it as the `goal_*_view` attribute (`list_animate.rs`).

use crate::element::UiTree;

impl UiTree {
    /// The list's own box extent along an axis, `y1 - y0 + 1` or `x1 - x0 + 1`.
    fn list_extent(&self, list: usize, vertical: bool) -> i32 {
        let rect = self.el(list).rect;
        if vertical {
            rect.height()
        } else {
            rect.width()
        }
    }

    /// `(paper, item, view)` along an axis, `None` for an element with no list state.
    fn list_axis(&self, list: usize, vertical: bool) -> Option<(i32, i32, i32)> {
        let state = self.el(list).list.as_ref()?;
        Some(if vertical {
            (state.paper.1, state.item.1, state.view.1)
        } else {
            (state.paper.0, state.item.0, state.view.0)
        })
    }

    /// `UIElement_ListBox::VerticalLineIncrement` (`steps` 1) and `VerticalLineDecrement` (-1),
    /// or the horizontal pair when `vertical` is false: the goal moves one item, within `0` and
    /// `paper - extent`.
    pub(crate) fn list_line_step(&mut self, list: usize, vertical: bool, steps: i32) {
        if self.is_msize(list) {
            self.list_msize_step(list, vertical, steps >= 0, false);
            return;
        }
        let Some((paper, item, view)) = self.list_axis(list, vertical) else {
            return;
        };
        let view = self.list_step_base(list, vertical, view);
        let goal = if steps >= 0 {
            // `min(view + item, paper - (y1 - y0) - 1)`.
            (view + item).min(paper - self.list_extent(list, vertical))
        } else {
            // `max(view - item, 0)`.
            (view - item).max(0)
        };
        self.list_set_goal(list, vertical, goal);
    }

    /// `UIElement_ListBox::VerticalPageIncrement` (`forward`) and `VerticalPageDecrement`, or the
    /// horizontal pair: the goal moves a page less one item (`extent - item`), within `0` and
    /// `paper - extent`.
    pub(crate) fn list_page_step(&mut self, list: usize, vertical: bool, forward: bool) {
        if self.is_msize(list) {
            self.list_msize_step(list, vertical, forward, true);
            return;
        }
        let Some((paper, item, view)) = self.list_axis(list, vertical) else {
            return;
        };
        let view = self.list_step_base(list, vertical, view);
        let extent = self.list_extent(list, vertical);
        let goal = if forward {
            (view + extent - item).min(paper - extent)
        } else {
            (view - (extent - item)).max(0)
        };
        self.list_set_goal(list, vertical, goal);
    }

    /// `UIElement_ListBox::ListenToMessage(hot_click)` (admin 0x005F31F0): a hot button a list
    /// names as its `up_button` / `down_button` steps it a line up / down
    /// (`VerticalLineDecrement` / `VerticalLineIncrement`), its `left_button` / `right_button` a
    /// column (`HorizontalLineDecrement` / `HorizontalLineIncrement`). A `unique_msg` list hears
    /// only buttons in its own root.
    pub(crate) fn list_hot_click(&mut self, button: usize) {
        let button_id = self.el(button).id();
        let lists: Vec<usize> = self
            .slots
            .iter()
            .enumerate()
            .filter_map(|(i, el)| {
                crate::list_box_select::is_list_kind(el.as_ref()?.desc.kind).then_some(i)
            })
            .collect();
        for list in lists {
            for (name, vertical, steps) in [
                ("up_button", true, -1),
                ("down_button", true, 1),
                ("left_button", false, -1),
                ("right_button", false, 1),
            ] {
                let named = matches!(
                    self.el(list).attributes.get(&crate::token::tokenize(name)),
                    Some(crate::layout::AttributeValue::Id(id)) if *id == button_id
                );
                if !named {
                    continue;
                }
                if self.list_flag(list, "unique_msg") && self.root_of(button) != self.root_of(list)
                {
                    continue;
                }
                self.list_line_step(list, vertical, steps);
            }
        }
    }

    /// `UIElement_ListBox::GetLeftCol`: the column at the view's left edge.
    pub fn list_left_col(&self, list: usize) -> i32 {
        match self.list_axis(list, false) {
            Some((_, item, view)) if item >= 1 => view / item,
            _ => 0,
        }
    }

    /// `UIElement_ListBox::ScrollToCol` (immediately): the view's left edge at column `col`.
    pub fn list_scroll_to_col(&mut self, list: usize, col: i32) {
        if self.is_msize(list) {
            self.list_msize_scroll_to(list, false, col);
            return;
        }
        let item = self.list_axis(list, false).map_or(0, |a| a.1);
        self.list_set_view(list, Some(item * col), None);
    }

    /// `UIElement_ListBox::GetCol`: the column of item `index`; `-1` for none or an empty list,
    /// `0` before the list has a layout.
    pub fn list_col(&self, list: usize, index: Option<usize>) -> i32 {
        let Some(state) = self.el(list).list.as_ref() else {
            return 0;
        };
        if state.rows < 1 || state.cols < 1 {
            return 0;
        }
        match index {
            Some(index) if !state.items.is_empty() => {
                let index = index as i32;
                if self.list_flag(list, "horizontal") {
                    index % state.cols
                } else {
                    index / state.rows
                }
            }
            _ => -1,
        }
    }

    /// `UIElement_ListBox::GetPercentageH` (`vertical` false) and `GetPercentageV`: the view's
    /// share of the scrollable paper, `0` when the paper is no longer than the list.
    pub fn list_percentage(&self, list: usize, vertical: bool) -> f32 {
        let Some((paper, _, view)) = self.list_axis(list, vertical) else {
            return 0.0;
        };
        let extent = self.list_extent(list, vertical);
        if paper <= extent {
            return 0.0;
        }
        (view as f32 / (paper - extent) as f32).clamp(0.0, 1.0)
    }

    /// `UIElement_ListBox::SetSelectedItemByID`: the first item with element id `id` selected
    /// (and shown); no such item selects nothing.
    pub fn list_set_selected_item_by_id(&mut self, list: usize, id: u32, broadcast: bool) {
        let found = self
            .list_items(list)
            .iter()
            .position(|&item| self.el(item).id() == id);
        match found {
            Some(index) => {
                let item = self.list_items(list)[index];
                self.list_set_selected_item(list, Some(item), broadcast);
                self.list_scroll_to_show(list, index);
            }
            None => self.list_set_selected_item(list, None, broadcast),
        }
    }

    /// `UIElement_ListBox::MultiOpAddItem`: a multi-operation opens, then the item is added.
    pub fn list_multi_op_add_item(&mut self, list: usize, item: usize) {
        self.list_multi_op_start(list);
        self.list_add_item(list, item);
    }

    /// `UIElement_ListBox::MultiOpInsertItem`: a multi-operation opens, then the item goes in at
    /// `at`.
    pub fn list_multi_op_insert_item(&mut self, list: usize, item: usize, at: usize) {
        self.list_multi_op_start(list);
        self.list_insert_item(list, item, at);
    }

    /// `UIElement_ListBox::MultiOpRemoveItem(index)`: a multi-operation opens, then the item at
    /// `index` is taken out.
    pub fn list_multi_op_remove_item(&mut self, list: usize, index: usize) -> Option<usize> {
        self.list_multi_op_start(list);
        self.list_remove_item(list, index)
    }

    /// `UIElement_ListBox::MultiOpRemoveItem(item)`: as above for an item; one the list does not
    /// hold removes nothing.
    pub fn list_multi_op_remove(&mut self, list: usize, item: usize) -> Option<usize> {
        self.list_multi_op_start(list);
        let index = self.list_what_num(list, item)?;
        self.list_remove_item(list, index)
    }

    /// `UIElement_ListBox::UpdateAndDraw` (and the tail of `ResizeTo`): the layout again, the
    /// view kept in bounds, the controls updated, and the list drawn.
    pub fn list_update_and_draw(&mut self, list: usize) {
        self.list_relayout(list);
        self.changed = true;
    }
}
