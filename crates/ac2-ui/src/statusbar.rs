//! `gmStatusbarUI`'s own functions over its `StatusbarField` root. Its children (`gmVitalsUI`,
//! `gmEffectsUI`, `gmIndicatorsUI`) are made by [`crate::hud`] in `SetupChildren`'s order and held
//! as the HUD's fields, so there is no child list to walk: they go with the HUD's tree.

use crate::element::UiTree;
use crate::token::tokenize;

/// The status bar's root (`StatusbarField`), if the layout has one.
pub fn statusbar_field(tree: &UiTree) -> Option<usize> {
    tree.roots().iter().copied().find(|&r| {
        tree.get(r)
            .is_some_and(|e| e.id() == tokenize("StatusbarField"))
    })
}

/// `gmStatusbarUI::Show`: `UIFramework::Show` records the flag, and the root element shows or
/// hides with it. The top preclude is sent as 0 whatever the flag
/// (`CM_UI::SendNotice_PrecludeFromTop(0)`), so the reticle keeps no space from the top.
pub fn show_statusbar(tree: &mut UiTree, shown: bool) {
    if let Some(field) = statusbar_field(tree) {
        tree.set_visible(field, shown);
    }
}
