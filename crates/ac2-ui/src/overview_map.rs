//! `gmOverviewMapUI`: the overview map (`OverviewMap` in the gameplay layout) with its label and
//! client area, and its close and help buttons.
//!
//! `gmOverviewMapUI::Show` shows the field (retail creates its elements on the first show and
//! destroys them on hide; here they are made with the HUD and only hidden). `ShowMap(mode)` shows
//! it and sets the label's and the client area's state to the map's (`SetState`). The close
//! button hides it (`ListenToMessage`); the help button opens the help page, which is not ported.
//!
//! The label is the overview's first text element: its name is a token this layout does not
//! expose under a known name.
//!
//! `gmNavigationUI::ListenToMessage` shows the overview for the panel's zoom buttons
//! (`NavigationZoomToOverview*Button`): each calls `ShowMap` with its map's state. The Legions
//! constructor binds four of them (Osteth, Omishan, Arramora, LinvakMassif); the fifth map state
//! Legions carries (`KnorrMap`) has no button this module can name, so it is not wired.

use std::sync::Arc;

use crate::element::UiTree;
use crate::layout::LayoutDesc;
use crate::token::tokenize;

/// The overview's zoom buttons (`gmNavigationUI`'s `m_pOverviewOstethButton` and the others) and
/// the map state each passes to `gmOverviewMapUI::ShowMap`.
const MAP_BUTTONS: [(u32, u32); 4] = [
    (
        tokenize("NavigationZoomToOverviewOstethButton"),
        tokenize("OstethMap"),
    ),
    (
        tokenize("NavigationZoomToOverviewOmishanButton"),
        tokenize("OmishanMap"),
    ),
    (
        tokenize("NavigationZoomToOverviewArramoraButton"),
        tokenize("ArramoraMap"),
    ),
    (
        tokenize("NavigationZoomToOverviewLinvakMassifButton"),
        tokenize("LinvakMassifMap"),
    ),
];

/// The map state a zoom button shows (`gmNavigationUI::ListenToMessage`), `None` for an element
/// that is not one of them.
pub fn map_for_button(element_id: u32) -> Option<u32> {
    MAP_BUTTONS
        .iter()
        .find(|(button, _)| *button == element_id)
        .map(|&(_, map)| map)
}

/// `gmOverviewMapUI`'s state over its gameplay elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OverviewMapUi {
    /// `m_pOverviewField`.
    field: usize,
    /// `m_pLabel`.
    label: Option<usize>,
    /// `m_pClientArea`.
    client_area: Option<usize>,
    /// The close button (`OverviewMapCloseButton`).
    close: Option<usize>,
}

impl OverviewMapUi {
    /// `gmOverviewMapUI::CreateManagedElements`: the `OverviewMap` root from the gameplay layout,
    /// with its label and client area found, hidden until a map is shown.
    pub fn create(tree: &mut UiTree, gameplay: &Arc<LayoutDesc>) -> Option<Self> {
        let field = tree.create_root(Arc::clone(gameplay), tokenize("OverviewMap"))?;
        tree.set_visible(field, false);
        let client_area = tree.child_recursive(field, tokenize("OverviewMapClientArea"));
        let label = first_text(tree, field);
        Some(Self {
            field,
            label,
            client_area,
            close: tree.child_recursive(field, tokenize("OverviewMapCloseButton")),
        })
    }

    /// The overview field's root.
    pub fn field(&self) -> usize {
        self.field
    }

    /// The label element.
    pub fn label(&self) -> Option<usize> {
        self.label
    }

    /// The client area element.
    pub fn client_area(&self) -> Option<usize> {
        self.client_area
    }

    /// `gmOverviewMapUI::Show`.
    pub fn show(&self, tree: &mut UiTree, shown: bool) {
        if tree.get(self.field).is_some_and(|e| e.visible != shown) {
            tree.set_visible(self.field, shown);
        }
    }

    /// Whether the overview is shown.
    pub fn is_shown(&self, tree: &UiTree) -> bool {
        tree.get(self.field).is_some_and(|e| e.visible)
    }

    /// `gmOverviewMapUI::ShowMap(mode)`: shown, with the label and the client area in the map's
    /// state `mode`.
    pub fn show_map(&self, tree: &mut UiTree, mode: u32) {
        self.show(tree, true);
        for element in [self.label, self.client_area].into_iter().flatten() {
            tree.set_state(element, mode);
        }
    }

    /// `gmOverviewMapUI::ListenToMessage`: the close button hides the overview. The navigation
    /// panel's zoom buttons show their map (`gmNavigationUI::ListenToMessage` calls `ShowMap`
    /// with the map's state). Returns whether `element_id` was one of these buttons.
    pub fn message(&self, tree: &mut UiTree, element_id: u32) -> bool {
        if let Some(map) = map_for_button(element_id) {
            self.show_map(tree, map);
            return true;
        }
        let is_close = element_id == tokenize("OverviewMapCloseButton") && self.close.is_some();
        if is_close {
            self.show(tree, false);
        }
        is_close
    }
}

/// The first text element under `root`, in the tree's order: the overview's label.
fn first_text(tree: &UiTree, root: usize) -> Option<usize> {
    for &child in tree.children(root) {
        if tree
            .get(child)
            .is_some_and(|e| e.kind() == crate::token::TEXT)
        {
            return Some(child);
        }
        if let Some(found) = first_text(tree, child) {
            return Some(found);
        }
    }
    None
}
