//! The modern loot window's leader (AGENTS.md: the HUD's two styles; the design's variant D): a
//! dashed gold line from the window's foot to the top of the container it lists, ending in a dot,
//! while the window shows and the container is on the screen. Retail drew none: its window sat
//! over the container. The line is its own root, its dash tiled along it and the root turned about
//! its centre; neither part takes the pointer.

use std::sync::Arc;

use crate::element::UiTree;
use crate::modern_art as art;
use crate::token::{tokenize, MOUSE_INVISIBLE};

/// The line's thickness and the dot's size.
const THICK: i32 = 2;
const DOT: i32 = 7;

/// The leader's two roots.
#[derive(Debug, Clone)]
pub(crate) struct Leader {
    line: usize,
    dot: usize,
}

/// A root copying `field`'s description without its states, drawing `image`, hidden.
fn make_root(tree: &mut UiTree, field: usize, id: &str, image: u32) -> Option<usize> {
    let layout = Arc::clone(&tree.el(field).layout);
    let mut desc = (*tree.el(field).reference).clone();
    desc.id = tokenize(id);
    desc.states.clear();
    desc.edges = [0; 4];
    let root = tree.create_element(layout, Arc::new(desc), true)?;
    tree.el_mut(root)
        .attributes
        .remove(&crate::token::ALPHA_VALUE);
    tree.set_z_level(root, 3);
    tree.add_root(root);
    tree.set_overlay_root(root);
    tree.set_image(root, image, crate::compose::DRAW_OVER);
    tree.set_bool(root, MOUSE_INVISIBLE, true);
    tree.set_visible(root, false);
    Some(root)
}

impl Leader {
    /// The line and the dot, made beside the loot window's `field`; hidden.
    pub(crate) fn make(tree: &mut UiTree, field: usize) -> Option<Self> {
        let line = make_root(tree, field, "ModernLootLeader", art::LOOT_LEADER_DASH)?;
        let dot = make_root(tree, field, "ModernLootLeaderDot", art::LOOT_LEADER_DOT)?;
        tree.resize_to(dot, DOT, DOT);
        Some(Self { line, dot })
    }

    /// The leader from `from` (the window's foot) to `to` (the container's top on the screen),
    /// or hidden without one. Whether the surfaces must be drawn again (a size or a shown state
    /// changed; a move or a turn only moves them).
    pub(crate) fn update(
        &self,
        tree: &mut UiTree,
        from: (i32, i32),
        to: Option<(i32, i32)>,
    ) -> bool {
        let Some(to) = to else {
            return self.hide(tree);
        };
        let (dx, dy) = (f64::from(to.0 - from.0), f64::from(to.1 - from.1));
        let length = dx.hypot(dy).round() as i32;
        if length < 4 {
            return self.hide(tree);
        }
        let mut redraw = false;
        // Laid flat, centred on the middle of the two points, then turned onto them (the
        // rotation turns anticlockwise on the screen, whose y runs down).
        let (mx, my) = ((from.0 + to.0) / 2, (from.1 + to.1) / 2);
        let degrees = (-dy).atan2(dx).to_degrees().round() as i32;
        if tree.el(self.line).rect.width() != length {
            tree.resize_to(self.line, length, THICK);
            redraw = true;
        }
        tree.set_object_position(self.line, mx - length / 2, my - THICK / 2);
        tree.set_object_rotation(self.line, degrees);
        tree.set_object_position(self.dot, to.0 - DOT / 2, to.1 - DOT / 2);
        for root in [self.line, self.dot] {
            if !tree.el(root).visible {
                tree.set_visible(root, true);
                redraw = true;
            }
        }
        redraw
    }

    /// The leader hidden; whether it showed.
    pub(crate) fn hide(&self, tree: &mut UiTree) -> bool {
        let mut redraw = false;
        for root in [self.line, self.dot] {
            if tree.el(root).visible {
                tree.set_visible(root, false);
                redraw = true;
            }
        }
        redraw
    }
}
