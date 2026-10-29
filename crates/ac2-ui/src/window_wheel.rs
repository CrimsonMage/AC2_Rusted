//! The wheel over any part of a window (a modern exception, AGENTS.md): retail scrolled only the
//! element under the pointer (a text, list or tree with a bar, or the bar), so a turn over a
//! window's frame, title, tabs or empty space did nothing. Here such a turn scrolls the window's
//! own scrollable content: of the scroll owners shown in the window under the pointer, the one
//! the pointer is level with, else the largest. The world under no window still zooms the camera,
//! and windows whose framework takes the wheel itself (the guide, the help window, the Tracker,
//! the world map) are left to it.

use crate::element::UiTree;
use crate::list_box_select::is_list_kind;
use crate::token::{tokenize, TEXT};

/// The world's element (`gmCameraUI`'s zoom target): a turn over it is the camera's.
const WORLD: u32 = tokenize("ItemSelectionField");
const TREE: u32 = tokenize("tree");

/// What a scroll owner is, for its own wheel path.
#[derive(Clone, Copy)]
enum Owner {
    Text,
    List,
    Tree,
}

impl UiTree {
    /// Marks `root` as a window whose framework takes the wheel itself, so a turn over it that
    /// scrolled no element is left to that framework.
    pub fn set_custom_wheel(&mut self, root: usize, custom: bool) {
        self.custom_wheel_roots.retain(|&r| r != root);
        if custom {
            self.custom_wheel_roots.push(root);
        }
    }

    /// A wheel turn that scrolled no element: the window under the pointer scrolls its content.
    /// From the top element up, the first ancestor holding a shown scroll owner is the window
    /// (the panel the pointer is over, not another panel in the same root); of its owners, the
    /// one the pointer is level with, else the largest, takes the turn by its own wheel path.
    pub(crate) fn window_wheel(&mut self, action: u32) {
        let Some(top) = self.mouse_top() else {
            return;
        };
        let root = self.root_of(top);
        if self.el(top).id() == WORLD
            || self.el(root).id() == WORLD
            || self.custom_wheel_roots.contains(&root)
        {
            return;
        }
        let mut at = Some(top);
        while let Some(window) = at {
            if let Some((index, owner)) = self.wheel_owner_in(window) {
                match owner {
                    Owner::Text => self.text_wheel(index, action),
                    Owner::List => self.list_wheel(index, action),
                    Owner::Tree => self.tree_wheel(index, action),
                }
                return;
            }
            at = self.el(window).parent;
        }
    }

    /// The scroll owner of `window`'s shown subtree that a turn over it scrolls: one the pointer is
    /// level with first, then the largest.
    fn wheel_owner_in(&self, window: usize) -> Option<(usize, Owner)> {
        let (_, pointer_y) = self.pointer;
        let mut best: Option<((bool, i64), usize, Owner)> = None;
        let mut stack = vec![window];
        while let Some(index) = stack.pop() {
            if !self.el(index).visible() {
                continue;
            }
            stack.extend(self.el(index).children.iter().copied());
            let kind = self.el(index).desc.kind;
            let owner = if kind == TEXT && self.text_scrollbar(index).is_some() {
                Owner::Text
            } else if is_list_kind(kind)
                && (self.list_scrollbar(index, true).is_some()
                    || self.list_scrollbar(index, false).is_some())
            {
                Owner::List
            } else if kind == TREE && self.tree_scrollbar(index).is_some() {
                Owner::Tree
            } else {
                continue;
            };
            let area = self.screen_box(index);
            let size = i64::from(area.width()) * i64::from(area.height());
            if size <= 0 {
                continue;
            }
            let rank = ((area.y0..=area.y1).contains(&pointer_y), size);
            if best.is_none_or(|(r, _, _)| rank > r) {
                best = Some((rank, index, owner));
            }
        }
        best.map(|(_, index, owner)| (index, owner))
    }
}
