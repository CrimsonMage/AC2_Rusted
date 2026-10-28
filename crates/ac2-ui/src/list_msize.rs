//! `UIElement_ListBoxMSize`: a list box whose columns and rows have their own sizes. A column is
//! as wide as its widest item and a row as tall as its tallest (`CalculateItemWidthsHeights`);
//! the items sit at the running sum of those sizes less the view (`UpdateLayout`), and the paper
//! is the sum of the columns by the sum of the rows. The view is not snapped to cells
//! (`ConstrainViewValue`: within `0` and `paper - extent`), the line and page steps move by the
//! size of the cell at the view's edge, and a point finds its cell by the running sums
//! (`InqItemIndex`, `GetItemIndexAtPoint`).
//!
//! Not modelled: `ScrollToShow` (it reads the base list's item size, which the sized list never
//! sets) and the smooth animation of the goal (see `list_goal.rs`).

use crate::element::UiTree;
use crate::list_box_select::LIST_BOX_MSIZE;

/// `UIElement_ListBoxMSize::InqItemIndex`: the index of the cell whose running end first
/// reaches `loc`; `None` when `loc` is past the last cell (or there are none).
pub(crate) fn item_index(loc: i32, sizes: &[i32]) -> Option<usize> {
    let mut end = 0;
    for (index, size) in sizes.iter().enumerate() {
        end += size;
        if end >= loc {
            return Some(index);
        }
    }
    None
}

/// `UIElement_ListBoxMSize::InqItemSize`: the sum of the first `index` sizes, which is where cell
/// `index` starts.
pub(crate) fn item_offset(index: usize, sizes: &[i32]) -> i32 {
    sizes.iter().take(index).sum()
}

impl UiTree {
    pub(crate) fn is_msize(&self, list: usize) -> bool {
        self.el(list).desc.kind == LIST_BOX_MSIZE
    }

    /// `UIElement_ListBoxMSize::CalculateItemWidthsHeights` and `UpdateLayout`: the columns and
    /// rows are measured, then each item moves to its cell less the view.
    pub(crate) fn list_msize_layout(&mut self, list: usize) {
        let horizontal = self.list_flag(list, "horizontal");
        let max_columns = self.list_int(list, "max_columns").unwrap_or(0).max(1);
        let items = self.list_items(list).to_vec();
        let state = self.list(list);
        state.cols = 1;
        state.rows = 1;
        state.paper = (0, 0);
        state.col_widths.clear();
        state.row_heights.clear();
        if items.is_empty() {
            self.changed = true;
            return;
        }
        let cols = max_columns;
        let rows = ((items.len() as f64 / f64::from(cols)).ceil() as i32).max(1);
        let mut widths = vec![0; cols as usize];
        let mut heights = vec![0; rows as usize];
        let (mut col, mut row) = (0_i32, 0_i32);
        for &item in &items {
            let rect = self.el(item).rect;
            widths[col as usize] = widths[col as usize].max(rect.width());
            heights[row as usize] = heights[row as usize].max(rect.height());
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
            // Past the last cell the cursor wraps without a cell to measure.
            col = col.min(cols - 1);
            row = row.min(rows - 1);
        }
        let state = self.list(list);
        state.cols = cols;
        state.rows = rows;
        state.paper = (widths.iter().sum(), heights.iter().sum());
        let view = state.view;
        state.col_widths = widths.clone();
        state.row_heights = heights.clone();
        let (mut x, mut y) = (-view.0, -view.1);
        let (mut col, mut row) = (0_usize, 0_usize);
        for &item in &items {
            if self.el(item).parent != Some(list) {
                self.set_parent(item, Some(list));
            }
            self.move_to(item, x, y);
            if horizontal {
                if col as i32 == cols - 1 {
                    x = -view.0;
                    col = 0;
                    y += heights[row];
                    row = (row + 1).min(heights.len() - 1);
                } else {
                    x += widths[col];
                    col += 1;
                }
            } else if row as i32 == rows - 1 {
                y = -view.1;
                x += widths[col];
                col = (col + 1).min(widths.len() - 1);
                row = 0;
            } else {
                y += heights[row];
                row += 1;
            }
        }
        self.changed = true;
    }

    /// `UIElement_ListBoxMSize::ConstrainViewValue`: no snapping, within `0` and
    /// `paper - extent`; the whole paper showing keeps the view at `0`.
    pub(crate) fn list_msize_constrain(&self, list: usize, x: bool, value: i32) -> i32 {
        let Some(state) = self.el(list).list.as_ref() else {
            return 0;
        };
        let rect = self.el(list).rect;
        let (paper, extent) = if x {
            (state.paper.0, rect.width())
        } else {
            (state.paper.1, rect.height())
        };
        if value < 0 || paper <= extent {
            0
        } else if paper < value + extent {
            paper - extent
        } else {
            value
        }
    }

    /// The sizes of the columns (`vertical` false) or rows of a sized list.
    fn msize_sizes(&self, list: usize, vertical: bool) -> Vec<i32> {
        self.el(list).list.as_ref().map_or_else(Vec::new, |state| {
            if vertical {
                state.row_heights.clone()
            } else {
                state.col_widths.clone()
            }
        })
    }

    /// `UIElement_ListBoxMSize::VerticalLineIncrement` ... `HorizontalPageDecrement`: the goal
    /// moves by the size of the cell at the view's edge (`delta`), kept within `0` and
    /// `paper - extent` (`UIElement_ListBox::DetermineAndSetNewGoal`). `page` moves by the
    /// extent less that cell instead. Nothing moves when the view is past the last cell.
    pub(crate) fn list_msize_step(
        &mut self,
        list: usize,
        vertical: bool,
        forward: bool,
        page: bool,
    ) {
        let sizes = self.msize_sizes(list, vertical);
        let Some(state) = self.el(list).list.as_ref() else {
            return;
        };
        let (paper, view) = if vertical {
            (state.paper.1, state.view.1)
        } else {
            (state.paper.0, state.view.0)
        };
        let Some(index) = item_index(view, &sizes) else {
            return;
        };
        let size = sizes[index];
        let rect = self.el(list).rect;
        let extent = if vertical {
            rect.height()
        } else {
            rect.width()
        };
        let delta = match (forward, page) {
            (true, false) => size,
            (false, false) => -size,
            (true, true) => extent - size,
            (false, true) => size - extent,
        };
        let goal = view + delta;
        let goal = if forward {
            goal.min(paper - extent)
        } else {
            goal.max(0)
        };
        if vertical {
            self.list_set_view(list, None, Some(goal));
        } else {
            self.list_set_view(list, Some(goal), None);
        }
    }

    /// `UIElement_ListBoxMSize::ScrollToRow` (`vertical`) and `ScrollToCol`, immediately: the
    /// view starts at the cell.
    pub(crate) fn list_msize_scroll_to(&mut self, list: usize, vertical: bool, cell: i32) {
        let offset = item_offset(cell.max(0) as usize, &self.msize_sizes(list, vertical));
        if vertical {
            self.list_set_view(list, None, Some(offset));
        } else {
            self.list_set_view(list, Some(offset), None);
        }
    }

    /// `UIElement_ListBoxMSize::GetItemIndexAtPoint` for a point in screen pixels.
    pub(crate) fn list_msize_index_at(&self, list: usize, x: i32, y: i32) -> Option<usize> {
        let state = self.el(list).list.as_ref()?;
        let b = self.screen_box(list);
        if x < b.x0 || x > b.x1 || y < b.y0 || y > b.y1 {
            return None;
        }
        let (px, py) = (state.view.0 + x - b.x0, state.view.1 + y - b.y0);
        if px >= state.paper.0 || py >= state.paper.1 {
            return None;
        }
        let col = item_index(px, &state.col_widths)? as i32;
        let row = item_index(py, &state.row_heights)? as i32;
        let index = if self.list_flag(list, "horizontal") {
            state.cols * row + col
        } else {
            state.rows * col + row
        };
        (index >= 0 && (index as usize) < state.items.len()).then_some(index as usize)
    }
}
