//! The floating arrow (a modern addition): a pointer at the top of the screen that turns towards a
//! target, the Tracker's picked match or a coordinate clicked in chat, with the target's
//! coordinates and distance under it.
//!
//! Both parts are the targeting reticle's own overlays made a second time from the gameplay
//! layout: `OffScreenIndicator` (its `reticle_arrow` art, pointing right when unturned, turned by
//! the surface object's rotation as the reticle turns it) and `SelectionName` for the label. They
//! are overlays over the 3D view, not windows.

use std::sync::Arc;

use crate::element::UiTree;
use crate::layout::LayoutDesc;
use crate::token::tokenize;

/// The arrow's centre below the screen's top, in display pixels.
const TOP: i32 = 84;
/// The label's gap under the arrow.
const LABEL_GAP: i32 = 2;

/// What the arrow shows this frame: the target's bearing relative to the view (degrees, clockwise
/// from straight ahead) and its label.
#[derive(Debug, Clone, PartialEq)]
pub struct GuideArrowView {
    pub relative_degrees: f32,
    pub label: String,
}

#[derive(Debug, Clone)]
pub struct GuideArrow {
    arrow: usize,
    label: Option<usize>,
    /// What was last shown (rotation, label), so an unchanged frame changes nothing.
    shown: Option<(i32, String)>,
}

impl GuideArrow {
    /// The arrow and its label, hidden. `None` when the layout has no `OffScreenIndicator`.
    pub fn create(tree: &mut UiTree, layout: &Arc<LayoutDesc>) -> Option<Self> {
        // Copies under ids of their own, so a lookup of the reticle's roots never finds these.
        let mut root = |name: &str, id: &str| {
            let mut desc = (**layout.element(tokenize(name))?).clone();
            desc.id = tokenize(id);
            let index = tree.create_root_from(Arc::clone(layout), Arc::new(desc))?;
            tree.set_overlay_root(index);
            tree.show_object(index, false);
            Some(index)
        };
        let arrow = root("OffScreenIndicator", "GuideArrowPointer")?;
        let label = root("SelectionName", "GuideArrowCaption");
        Some(Self {
            arrow,
            label,
            shown: None,
        })
    }

    /// The arrow's root (for tests and the frame's layering).
    pub fn arrow(&self) -> usize {
        self.arrow
    }

    /// Shows the arrow turned to `view` at the top middle of the display, or hides it.
    pub fn update(&mut self, tree: &mut UiTree, view: Option<&GuideArrowView>) {
        let Some(view) = view else {
            if self.shown.take().is_some() {
                tree.show_object(self.arrow, false);
                if let Some(label) = self.label {
                    tree.show_object(label, false);
                }
            }
            return;
        };
        // The art points right unturned and the rotation turns it anticlockwise (the reticle's
        // indicator turns 90 to point up): straight ahead is 90, to the right is 0.
        let rotation = (90.0 - view.relative_degrees).rem_euclid(360.0).round() as i32 % 360;
        if self.shown.as_ref() == Some(&(rotation, view.label.clone())) {
            return;
        }
        let (display_w, _) = tree.display;
        let size = |tree: &UiTree, index: usize| {
            tree.get(index)
                .map_or((0, 0), |e| (e.rect().width(), e.rect().height()))
        };
        let (w, h) = size(tree, self.arrow);
        tree.show_object(self.arrow, true);
        tree.set_object_rotation(self.arrow, rotation);
        tree.set_object_position(self.arrow, display_w / 2 - w / 2, TOP - h / 2);
        if let Some(label) = self.label {
            tree.set_text(label, &view.label);
            // The name element sizes to its text (as the reticle's name does).
            let width = tree
                .text_size_with_margins(label, &view.label)
                .map_or(w, |(tw, _)| tw);
            let height = size(tree, label).1;
            tree.resize_to(label, width, height);
            tree.show_object(label, true);
            tree.set_object_position(label, display_w / 2 - width / 2, TOP + h / 2 + LABEL_GAP);
        }
        self.shown = Some((rotation, view.label.clone()));
    }
}
