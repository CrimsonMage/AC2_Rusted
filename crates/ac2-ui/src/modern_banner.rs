//! The modern style's targeting banner (AGENTS.md: the HUD's two styles; the design's variant B):
//! while the targeting pointer is on (`RecvNotice_SetTargetingPointer`, an item waiting for its
//! target), a gold banner at the display's top centre says to pick a target. Retail showed only
//! the pointer's cursor. The banner takes no pointer; the targeting is retail's.

use std::sync::Arc;

use crate::element::UiTree;
use crate::layout::LayoutDesc;
use crate::modern_art as art;
use crate::token::{tokenize, MOUSE_INVISIBLE};
use crate::window_parts::{set_button_label, text_button};

/// The banner's text, its height, its side padding and its distance from the top.
const TEXT: &str = "Select a target: click an item or object (Esc cancels)";
const HEIGHT: i32 = 30;
const PAD: i32 = 20;
const TOP: i32 = 6;

/// The banner: its own root and its text.
#[derive(Debug, Clone)]
pub struct TargetBanner {
    root: usize,
    width: i32,
}

impl TargetBanner {
    /// The banner made as its own root from `window`'s layout (copying `template`, a plain field)
    /// with a text from the gameplay layout's text button; hidden.
    pub fn make(
        tree: &mut UiTree,
        window: usize,
        template: usize,
        gameplay: &Arc<LayoutDesc>,
    ) -> Option<Self> {
        let layout = Arc::clone(&tree.el(window).layout);
        let mut desc = (*tree.el(template).reference).clone();
        desc.id = tokenize("ModernTargetBanner");
        desc.states.clear();
        desc.edges = [0; 4];
        let root = tree.create_element(layout, Arc::new(desc), true)?;
        tree.el_mut(root)
            .attributes
            .remove(&crate::token::ALPHA_VALUE);
        tree.set_z_level(root, 1);
        tree.add_root(root);
        tree.set_overlay_root(root);
        tree.set_image(root, art::TARGET_BANNER, crate::compose::DRAW_OVER);
        tree.set_int(root, crate::compose::NINE_SLICE, art::TARGET_BANNER_BORDER);
        tree.set_bool(root, MOUSE_INVISIBLE, true);
        let label = text_button(
            tree,
            gameplay,
            root,
            "ModernTargetBannerText",
            "",
            (0, 0, 300),
        )?;
        // Kept where it is put: the copied button's edges would re-place it as the display grows.
        crate::modern_inventory::keep(tree, label);
        tree.clear_image(label);
        tree.set_bool(label, MOUSE_INVISIBLE, true);
        let text = tree
            .children(label)
            .iter()
            .copied()
            .find(|&c| tree.el(c).text.is_some());
        let width = text
            .and_then(|t| tree.text_line_width(t, TEXT))
            .unwrap_or(300)
            + 2 * PAD;
        tree.resize_to(root, width, HEIGHT);
        tree.move_to(label, PAD, (HEIGHT - 20) / 2 + 1);
        tree.resize_to(label, width - 2 * PAD, 20);
        set_button_label(tree, label, TEXT);
        if let Some(t) = text {
            crate::modern_inventory::keep(tree, t);
            tree.set_bool(t, MOUSE_INVISIBLE, true);
            // The design's dark text on the gold.
            if let Some(state) = tree.el_mut(t).text.as_mut() {
                state.color = [
                    0x16 as f32 / 255.0,
                    0x10 as f32 / 255.0,
                    0x0A as f32 / 255.0,
                    1.0,
                ];
                for glyph in &mut state.glyphs {
                    glyph.color = state.color;
                }
            }
        }
        tree.set_visible(root, false);
        Some(Self { root, width })
    }

    /// The banner shown at the top's centre while the targeting pointer is on (every frame).
    pub fn tick(&self, tree: &mut UiTree) {
        let on = tree.targeting;
        if on {
            let x = (tree.display.0 - self.width) / 2;
            let now = tree.screen_box(self.root);
            if (now.x0, now.y0) != (x, TOP) {
                tree.set_object_position(self.root, x, TOP);
                tree.move_to(self.root, x, TOP);
            }
        }
        if tree.el(self.root).visible != on {
            tree.set_visible(self.root, on);
        }
    }
}
