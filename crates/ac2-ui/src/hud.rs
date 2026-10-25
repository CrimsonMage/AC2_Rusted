//! The gameplay HUD: the root elements `gmGamePlayUI::SetupChildren`'s child frameworks create
//! (`UIChildFramework::CreateAndAddRootElement(layout enum, element)`), and the images they show.
//!
//! `gmRadarUI` makes `RadarField` and `HeadingField` from the gameplay layout (`0x41000009`),
//! `gmSelectionUI` makes `ItemSelectionField` from it, `gmStatusbarUI` makes `StatusbarField`
//! from the status bar layout (`0x4100000F`) and `gmToolbarUI` makes `ToolbarField`,
//! `LeftSidePanel` and `RightSidePanel` from the toolbar layout (`0x41000003`), in that order.
//! The other children (stack, trade, shop, book, pet, ...) open dialogs on demand and are left out,
//! as are the toolbar's own children (`gmToolbarUI::SetupChildren`: chat, inventory, ...).

use std::collections::HashMap;
use std::sync::Arc;

use ac2_core::DataId;
use ac2_db::{ResourceResult, ResourceStore};

use crate::compose::UiPixels;
use crate::element::UiTree;
use crate::layout::{load_layout, LayoutDesc};
use crate::token::tokenize;

/// `UILAYOUT_GROUP_ENUM` (`UIMainFramework::CreateAndAddRootElement`'s
/// `FileArray::GetDIDFromEnum(layoutEnum, 0x40000003)`).
pub const UILAYOUT_GROUP: u32 = 0x4000_0003;
pub const LAYOUT_TOOLBAR: u32 = 0x4100_0003;
pub const LAYOUT_GAMEPLAY: u32 = 0x4100_0009;
pub const LAYOUT_STATUSBAR: u32 = 0x4100_000F;

/// The HUD roots in creation order: (layout enum, element name).
pub const GAMEPLAY_ROOTS: [(u32, &str); 7] = [
    (LAYOUT_GAMEPLAY, "RadarField"),
    (LAYOUT_GAMEPLAY, "HeadingField"),
    (LAYOUT_GAMEPLAY, "ItemSelectionField"),
    (LAYOUT_STATUSBAR, "StatusbarField"),
    (LAYOUT_TOOLBAR, "ToolbarField"),
    (LAYOUT_TOOLBAR, "LeftSidePanel"),
    (LAYOUT_TOOLBAR, "RightSidePanel"),
];

/// The layout a layout enum names.
pub fn load_layout_by_enum(
    store: &ResourceStore,
    layout_enum: u32,
) -> ResourceResult<Option<Arc<LayoutDesc>>> {
    let Some(did) = ac2_assets::ac2_enum_did(store, UILAYOUT_GROUP, layout_enum)? else {
        return Ok(None);
    };
    Ok(load_layout(store, did)?.map(|layout| Arc::new((*layout).clone())))
}

/// The HUD on a `width` x `height` display.
pub fn build_gameplay_hud(
    store: &ResourceStore,
    width: i32,
    height: i32,
) -> ResourceResult<UiTree> {
    let mut tree = UiTree::new(width, height);
    let mut layouts = HashMap::new();
    for (layout_enum, name) in GAMEPLAY_ROOTS {
        if !layouts.contains_key(&layout_enum) {
            layouts.insert(layout_enum, load_layout_by_enum(store, layout_enum)?);
        }
        let Some(layout) = &layouts[&layout_enum] else {
            continue;
        };
        let root = tree.create_root(Arc::clone(layout), tokenize(name));
        if let (Some(root), "ToolbarField") = (root, name) {
            // `gmToolbarUI::gmToolbarUI` hides the dismount and consignment buttons until a
            // mount or a sale needs them.
            for button in ["DismountButton", "ConsignmentsButton"] {
                if let Some(button) = tree.child_recursive(root, tokenize(button)) {
                    tree.set_visible(button, false);
                }
            }
        }
    }
    Ok(tree)
}

/// Every image the tree shows (`Graphic::Graphic`); a missing image draws nothing.
pub fn load_ui_images(
    store: &ResourceStore,
    tree: &UiTree,
) -> ResourceResult<HashMap<u32, UiPixels>> {
    let mut images = HashMap::new();
    for file in tree.image_files() {
        if let Some(surface) = ac2_assets::load_ac2_ui_surface(store, DataId::new(file))? {
            images.insert(
                file,
                UiPixels {
                    width: surface.width,
                    height: surface.height,
                    rgba: surface.rgba,
                },
            );
        }
    }
    Ok(images)
}
