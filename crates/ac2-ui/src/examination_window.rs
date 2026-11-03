//! The examination panel as a window of its own (AGENTS.md: the examination window). Retail's
//! `ExaminationField` is one of the three fields of `LeftSidePanel` (toolbar layout
//! `0x41000003`, beside `NavigationField` and `OptionField`), and `gmToolbarUI::ManageChildren`
//! hid the other two when one showed, so examining something closed the options or the map and
//! opening either closed the examination. Here the toolbar layout's `LeftSidePanel` keeps the
//! other two, and `ExaminationField` moves into [`EXAMINATION_PANEL`], a copy of the panel's
//! shell (its edges, z level and attributes) made a root of its own beside it, so it shows,
//! moves and keeps its place apart from them ([`crate::ToolbarChild`] leaves it out of the
//! side's exclusion). The field, its elements and `gmExaminationUI`'s behaviour are retail's.

use std::sync::Arc;

use crate::layout::{ElementDesc, LayoutDesc};
use crate::token::tokenize;

/// The window's root element (the client's own name).
pub const EXAMINATION_PANEL: &str = "ExaminationPanel";

/// The toolbar layout with `ExaminationField` out of `LeftSidePanel` and in its own root,
/// placed just right of the panel. A layout without them is returned unchanged.
pub(crate) fn restyle(layout: &LayoutDesc) -> LayoutDesc {
    let mut layout = layout.clone();
    let field = tokenize("ExaminationField");
    let Some(index) = layout
        .elements
        .iter()
        .position(|e| e.id == tokenize("LeftSidePanel"))
    else {
        return layout;
    };
    let mut side: ElementDesc = (*layout.elements[index]).clone();
    let mut window = side.clone();
    let mut moved = None;
    for state in &mut side.states {
        if let Some(at) = state.children.iter().position(|c| c.id == field) {
            moved = Some(state.id);
            state.children.remove(at);
        }
    }
    let Some(moved) = moved else {
        return layout;
    };
    window.id = tokenize(EXAMINATION_PANEL);
    for state in &mut window.states {
        state.children.retain(|c| c.id == field);
        if state.id != moved {
            state.children.clear();
        }
    }
    let width = window.rect.x1 - window.rect.x0 + 1;
    window.rect.x0 += width;
    window.rect.x1 += width;
    layout.elements[index] = Arc::new(side);
    layout.elements.insert(index + 1, Arc::new(window));
    layout
}
