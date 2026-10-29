//! `UIElement_Tree`'s vertical scroll bar.
//!
//! A tree names its bar in `vertical_scrollbar`. `UIElement_Tree::UpdateControls` sets the bar's
//! position to `GetPercentageV` (the view's `y_view` over its scrollable range) and disables it
//! when the paper fits. A bar's new position sets `y_view` (`ListenToMessage`); a line step moves
//! the view by the tree's `node_height` (`VerticalLineIncrement` / `VerticalLineDecrement`,
//! `goal_y_view`, here set at once). A wheel turn over the tree goes to its bar
//! (`UIElement_Tree::MouseDown`). The scrollable range is the paper less the element's height,
//! as for texts (`text_scroll.rs`): `UIElement_Tree::GetPercentageV` (admin 7148 0x005F5E50,
//! Legions 0x005B6980) divides by `m_paperHeight - (y1 - y0 + 1)`.
//!
//! Not modelled yet: the horizontal bar, and the smooth move to `goal_y_view`. The page steps
//! (`VerticalPageIncrement` ...) are `tree_goal_step`, reached by the bar's page messages.

use crate::element::UiTree;
use crate::layout::AttributeValue;
use crate::mouse::{UI_WHEEL_IN, UI_WHEEL_OUT};
use crate::token::tokenize;

const TREE: u32 = tokenize("tree");
const VERTICAL_SCROLLBAR: &str = "vertical_scrollbar";

impl UiTree {
    /// The vertical scroll bar a tree names (`GetScrollbar`, see `named_scrollbar`).
    pub(crate) fn tree_scrollbar(&self, tree: usize) -> Option<usize> {
        self.named_scrollbar(tree, VERTICAL_SCROLLBAR)
    }

    /// The trees that name `scrollbar` as their vertical scroll bar, in any root.
    pub(crate) fn trees_bound_to(&self, scrollbar: usize) -> Vec<usize> {
        let id = self.el(scrollbar).id();
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| {
                let el = slot.as_ref()?;
                let names_it = el.desc.kind == TREE
                    && matches!(
                        el.attributes.get(&tokenize(VERTICAL_SCROLLBAR)),
                        Some(AttributeValue::Id(v)) if *v == id
                    );
                (names_it && self.tree_scrollbar(index) == Some(scrollbar)).then_some(index)
            })
            .collect()
    }

    /// The paper's height the view can scroll over: `m_paperHeight - (y1 - y0 + 1)`, the paper
    /// less the element's inclusive height (zero when the paper fits).
    fn tree_scroll_range(&self, tree: usize) -> i32 {
        let paper = self.el(tree).tree.as_ref().map_or(0, |s| s.paper.1);
        (paper - self.el(tree).rect.height()).max(0)
    }

    /// `UIElement_Tree::ListenToMessage` for the vertical bar's `new_position`.
    pub(crate) fn trees_follow_scrollbar(&mut self, scrollbar: usize) {
        let position = f64::from(self.scrollbar_position(scrollbar));
        for tree in self.trees_bound_to(scrollbar) {
            let y = (position * f64::from(self.tree_scroll_range(tree))) as i32;
            self.tree_set_view_y(tree, y);
        }
    }

    /// Sets `y_view` within the scrollable range, lays the items out and moves the bar to match.
    fn tree_set_view_y(&mut self, tree: usize, y: i32) {
        let range = self.tree_scroll_range(tree);
        self.tree_state(tree).view.1 = y.clamp(0, range);
        self.tree_update_layout(tree);
        self.tree_update_scrollbar(tree);
    }

    /// `UIElement_Tree::UpdateControls` for the vertical bar.
    pub(crate) fn tree_update_scrollbar(&mut self, tree: usize) {
        let Some(bar) = self.tree_scrollbar(tree) else {
            return;
        };
        let range = self.tree_scroll_range(tree);
        let view = self.el(tree).tree.as_ref().map_or(0, |s| s.view.1);
        let share = if range <= 0 {
            0.0
        } else {
            (f64::from(view) / f64::from(range)).clamp(0.0, 1.0)
        };
        self.scrollbar_set_position(bar, share as f32);
        self.el_mut(bar)
            .attributes
            .insert(tokenize("disabled"), AttributeValue::Bool(range <= 0));
    }

    /// The node size along an axis: `node_height` (vertical) or `node_width`.
    fn tree_node_size(&self, tree: usize, vertical: bool) -> i32 {
        let name = if vertical {
            "node_height"
        } else {
            "node_width"
        };
        match self.el(tree).attributes.get(&tokenize(name)) {
            Some(AttributeValue::Int(v)) => *v,
            _ => 0,
        }
    }

    /// `UIElement_Tree::VerticalLineIncrement` (`steps` 1) and `VerticalLineDecrement` (-1): one
    /// node row.
    pub(crate) fn tree_line_step(&mut self, tree: usize, steps: i32) {
        self.tree_goal_step(tree, true, steps >= 0, false);
    }

    /// The goal steps of a tree (`VerticalLineIncrement` ... `HorizontalPageDecrement`): the view
    /// moves a node (`page` false) or a page less one node (the element's extent less the node
    /// size) back or forward, within `0` and `paper - extent`. Only the vertical view is kept: the
    /// horizontal bar is not modelled, so a horizontal step moves `x_view` alone.
    pub(crate) fn tree_goal_step(
        &mut self,
        tree: usize,
        vertical: bool,
        forward: bool,
        page: bool,
    ) {
        let node = self.tree_node_size(tree, vertical);
        if node <= 0 && !page {
            return;
        }
        let rect = self.el(tree).rect;
        let extent = if vertical {
            rect.height()
        } else {
            rect.width()
        };
        let Some(state) = self.el(tree).tree.as_ref() else {
            return;
        };
        let (paper, view) = if vertical {
            (state.paper.1, state.view.1)
        } else {
            (state.paper.0, state.view.0)
        };
        let delta = if page { extent - node } else { node };
        let goal = if forward {
            (view + delta).min(paper - extent)
        } else {
            (view - delta).max(0)
        };
        if vertical {
            self.tree_set_view_y(tree, goal);
        } else {
            let range = (paper - extent).max(0);
            self.tree_state(tree).view.0 = goal.clamp(0, range);
            self.tree_update_layout(tree);
        }
    }

    /// `UIElement_Tree::GetPercentageV` (`vertical`) and `GetPercentageH`: the view's share of the
    /// scrollable paper, `0` when the paper is no longer than the element.
    pub fn tree_percentage(&self, tree: usize, vertical: bool) -> f32 {
        let Some(state) = self.el(tree).tree.as_ref() else {
            return 0.0;
        };
        let rect = self.el(tree).rect;
        let (paper, view, extent) = if vertical {
            (state.paper.1, state.view.1, rect.height())
        } else {
            (state.paper.0, state.view.0, rect.width())
        };
        if paper <= extent {
            return 0.0;
        }
        (view as f32 / (paper - extent) as f32).clamp(0.0, 1.0)
    }

    /// A wheel turn over tree `tree`: its vertical bar takes it (`UIElement_Tree::MouseDown`).
    pub(crate) fn tree_wheel(&mut self, tree: usize, action: u32) {
        let Some(bar) = self.tree_scrollbar(tree) else {
            return;
        };
        self.wheel_scrolled = true;
        self.tree_update_scrollbar(tree);
        self.scrollbar_mouse_down(bar, action);
    }

    /// `UIElement_Tree::MouseDown`: a wheel turn over the tree.
    pub(crate) fn tree_mouse_down(&mut self, index: usize, action: u32) {
        if self.el(index).desc.kind == TREE && matches!(action, UI_WHEEL_IN | UI_WHEEL_OUT) {
            self.tree_wheel(index, action);
        }
    }
}
