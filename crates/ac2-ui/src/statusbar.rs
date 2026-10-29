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

/// Whether the status bar is shown (its root is visible; `gmStatusbarUI::Shown`).
pub fn statusbar_shown(tree: &UiTree) -> bool {
    statusbar_field(tree)
        .and_then(|f| tree.get(f))
        .is_some_and(|e| e.visible)
}

/// `gmStatusbarUI::Show`: `UIFramework::Show` records the flag, and the root element shows or
/// hides with it. Returns the top preclude it sends (`CM_UI::SendNotice_PrecludeFromTop`): the
/// field's `precludeheight` while shown, else 0 (Legions 0x422fc4, admin 0x4231f4: `Shown()`
/// masks the height; the admin decompile shows a literal 0), which the reticle and name plates
/// keep clear (`gm3DTextUI::RecvNotice_PrecludeFromTop`).
pub fn show_statusbar(tree: &mut UiTree, shown: bool) -> i32 {
    let Some(field) = statusbar_field(tree) else {
        return 0;
    };
    tree.set_visible(field, shown);
    match tree
        .get(field)
        .and_then(|e| e.attribute(tokenize("precludeheight")))
    {
        Some(crate::layout::AttributeValue::Int(height)) if shown => *height,
        _ => 0,
    }
}
