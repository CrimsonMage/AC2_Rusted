//! `UIElement_ListBox`'s scroll bars and keyboard stepping.
//!
//! A list names its scroll bars by `vertical_scrollbar` / `horizontal_scrollbar`. A scroll bar's
//! new position sets the list's view to `position * (paper - height)` (`ListBox::HandleScrollBarMessage`),
//! and the list's view sets the bar back to `view / (paper - height)`, zero when the paper is no
//! taller than the list (`ListBox::GetPercentageV`, `UpdateControls`). `height` is the list's own
//! inclusive box height (`y1 - y0 + 1`), read from the machine code of both functions (admin
//! 7148 0x005F2D90 and 0x005F2C30, Legions 0x005B38C0 and 0x005B3760): the same range the text and
//! tree bars use (`text_scroll.rs`). While the scroll bar drives the view the bar is not written
//! back (the `0x20000` guard). The line and page steps are in `list_goal.rs`. A wheel turn
//! over a list goes to its scroll bar (`ListBox::HandleMouseWheel`).
//!
//! Keyboard stepping (`ListBox::StartActive`, `StartUpActive` ... `StartRightActive`) moves the
//! selection by a fixed count of items: one, or a row or column of them, by the list's layout and
//! wraps past either end.
//!
//! Not modelled yet: paging by holding the track, smooth
//! scrolling (`goal_x_view`, `StartAnimation`), rollover and drag selection, and no list is
//! activated yet, so the arrow keys never reach a list (they are camera input outside a text
//! element).

use crate::element::UiTree;
use crate::layout::AttributeValue;
use crate::token::tokenize;

/// `UIElement_ListBox::StartUpActive` ... `StartRightActive`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListStep {
    Up,
    Down,
    Left,
    Right,
}

/// The UI action of each arrow key (`UIACTION_LEFT` ... `UIACTION_DOWN`), as the text element
/// reads them.
pub fn list_step_for_action(action: u32) -> Option<ListStep> {
    match action {
        0x4000_0013 => Some(ListStep::Left),
        0x4000_0014 => Some(ListStep::Right),
        0x4000_0015 => Some(ListStep::Up),
        0x4000_0016 => Some(ListStep::Down),
        _ => None,
    }
}

/// The list's box extent along the scroll axis: `UiBox::height` or `width`, inclusive.
fn box_extent(rect: crate::element::UiBox, vertical: bool) -> i32 {
    if vertical {
        rect.height()
    } else {
        rect.width()
    }
}

/// The view's scrollable range: the paper less the list's own box extent, zero when the paper
/// fits (`ListBox::GetPercentageV`: `m_nPaperHeight - (y1 - y0 + 1)`).
fn scroll_range(paper: i32, extent: i32) -> i32 {
    (paper - extent).max(0)
}

impl UiTree {
    /// The scroll bar a list names in `vertical_scrollbar` (or `horizontal_scrollbar`), resolved
    /// as `GetScrollbar` does (see `named_scrollbar`).
    pub(crate) fn list_scrollbar(&self, list: usize, vertical: bool) -> Option<usize> {
        let name = if vertical {
            "vertical_scrollbar"
        } else {
            "horizontal_scrollbar"
        };
        self.named_scrollbar(list, name)
    }

    /// The lists that name `scrollbar` as their vertical or horizontal scroll bar, in any root,
    /// with whether it is their vertical one.
    pub(crate) fn lists_bound_to(&self, scrollbar: usize) -> Vec<(usize, bool)> {
        let id = self.el(scrollbar).id();
        let mut lists = Vec::new();
        for (index, slot) in self.slots.iter().enumerate() {
            let Some(el) = slot.as_ref() else {
                continue;
            };
            if !crate::list_box_select::is_list_kind(el.desc.kind) {
                continue;
            }
            for (name, vertical) in [
                ("vertical_scrollbar", true),
                ("horizontal_scrollbar", false),
            ] {
                let names_it = matches!(
                    el.attributes.get(&tokenize(name)),
                    Some(AttributeValue::Id(v)) if *v == id
                );
                if names_it && self.list_scrollbar(index, vertical) == Some(scrollbar) {
                    lists.push((index, vertical));
                }
            }
        }
        lists
    }

    /// `ListBox::HandleScrollBarMessage` for `new_position`: the lists the scroll bar drives take
    /// its position as their view.
    pub(crate) fn lists_follow_scrollbar(&mut self, scrollbar: usize) {
        let position = f64::from(self.scrollbar_position(scrollbar));
        for (list, vertical) in self.lists_bound_to(scrollbar) {
            let extent = box_extent(self.el(list).rect, vertical);
            let Some(state) = self.el_mut(list).list.as_mut() else {
                continue;
            };
            let paper = if vertical {
                state.paper.1
            } else {
                state.paper.0
            };
            let view = (position * f64::from(scroll_range(paper, extent))) as i32;
            state.from_scrollbar = true;
            if vertical {
                self.list_set_view(list, None, Some(view));
            } else {
                self.list_set_view(list, Some(view), None);
            }
            if let Some(state) = self.el_mut(list).list.as_mut() {
                state.from_scrollbar = false;
            }
        }
    }

    /// `UIElement_ListBox::UpdateControls` (admin 0x005F38D0), unless the scroll bar is driving
    /// the view: each bound bar takes the list's share of the paper as its widget size (at most
    /// one) and the view's share of the scroll as its position, with its layout held until both
    /// are set (`allow_updating`), and is disabled when the list shows the whole paper. The
    /// buttons the list names are ghosted at the ends (`HandleButtonGhosting`): up and left at a
    /// view of zero, down and right once the view reaches the end.
    pub(crate) fn list_update_controls(&mut self, list: usize) {
        if self
            .el(list)
            .attributes
            .get(&tokenize("allow_control_updating"))
            == Some(&AttributeValue::Bool(false))
        {
            return;
        }
        let Some(state) = self.el(list).list.as_ref() else {
            return;
        };
        let (paper, view, from_scrollbar) = (state.paper, state.view, state.from_scrollbar);
        let (mut total, mut size) = (paper.1, box_extent(self.el(list).rect, true));
        for (vertical, bar) in [
            (true, self.list_scrollbar(list, true)),
            (false, self.list_scrollbar(list, false)),
        ] {
            let Some(bar) = bar else {
                continue;
            };
            let extent = box_extent(self.el(list).rect, vertical);
            let (paper_extent, at) = if vertical {
                (paper.1, view.1)
            } else {
                (paper.0, view.0)
            };
            (total, size) = (paper_extent.max(extent), extent);
            if from_scrollbar {
                continue;
            }
            let range = scroll_range(paper_extent, extent);
            let share = if range <= 0 {
                0.0
            } else {
                (f64::from(at) / f64::from(range)).clamp(0.0, 1.0)
            };
            let widget_size = (extent as f32 / total as f32).min(1.0);
            let size_id = if vertical {
                "y_widget_size"
            } else {
                "x_widget_size"
            };
            let attrs = &mut self.el_mut(bar).attributes;
            attrs.insert(tokenize("allow_updating"), AttributeValue::Bool(false));
            attrs.insert(tokenize("disabled"), AttributeValue::Bool(false));
            attrs.insert(tokenize(size_id), AttributeValue::Float(widget_size));
            self.scrollbar_set_position(bar, share as f32);
            self.el_mut(bar)
                .attributes
                .insert(tokenize("allow_updating"), AttributeValue::Bool(true));
            self.scrollbar_update_layout(bar);
            if paper_extent <= extent {
                self.el_mut(bar)
                    .attributes
                    .insert(tokenize("disabled"), AttributeValue::Bool(true));
                self.scrollbar_update_layout(bar);
            }
        }
        // `HandleButtonGhosting`, by the last bar's totals as retail's single bar.
        for (name, ghosted) in [
            ("up_button", view.1 == 0),
            ("down_button", total <= view.1 + size),
            ("left_button", view.0 == 0),
            ("right_button", total <= view.0 + size),
        ] {
            if let Some(button) = self.named_element(list, name, None, "unique_msg") {
                self.set_bool(button, tokenize("ghosted"), ghosted);
            }
        }
    }

    /// `UIElement_ListBox::StartActive`: the item `delta` places from the selected one is
    /// selected and shown, wrapping past either end; an empty list selects nothing.
    pub fn list_step_selection(&mut self, list: usize, delta: i32) {
        let items = self.list_items(list).to_vec();
        let count = items.len() as i32;
        if count == 0 {
            self.list_set_selected_item(list, None, true);
            return;
        }
        let current = self
            .list_selected_item(list)
            .and_then(|s| items.iter().position(|&i| i == s))
            .map_or(-1, |i| i as i32);
        let mut next = current + delta;
        if next < 0 {
            next = count - 1;
        }
        if next >= count {
            next = 0;
        }
        self.list_set_selected_item(list, Some(items[next as usize]), true);
        self.list_scroll_to_show(list, next as usize);
    }

    /// `UIElement_ListBox::StartUpActive` and its three partners: the selection steps by the
    /// layout (a row or a column for a `horizontal` list, otherwise one item across the columns).
    pub fn list_step_active(&mut self, list: usize, step: ListStep) {
        let horizontal = self.list_flag(list, "horizontal");
        let Some(state) = self.el(list).list.as_ref() else {
            return;
        };
        let (cols, rows) = (state.cols, state.rows);
        let delta = match step {
            ListStep::Up if horizontal => -cols,
            ListStep::Up => -1,
            ListStep::Down if horizontal => cols,
            ListStep::Down => 1,
            ListStep::Left if horizontal => -1,
            ListStep::Left => -rows,
            ListStep::Right if horizontal => 1,
            ListStep::Right => rows,
        };
        self.list_step_selection(list, delta);
    }

    /// A wheel turn over list `list`: its scroll bar takes it (`ListBox::HandleMouseWheel`).
    pub(crate) fn list_wheel(&mut self, list: usize, action: u32) {
        if let Some(bar) = self
            .list_scrollbar(list, true)
            .or_else(|| self.list_scrollbar(list, false))
        {
            self.wheel_scrolled = true;
            self.scrollbar_mouse_wheel(bar, action);
        }
    }

    /// An arrow key while list `list` is the active element: it steps the selection
    /// (`list_step_active`); `false` for any other action.
    pub fn list_key(&mut self, list: usize, action: u32, extent: f32) -> bool {
        let Some(step) = list_step_for_action(action) else {
            return false;
        };
        if extent != 0.0 {
            self.list_step_active(list, step);
        }
        true
    }
}
