//! The gameplay HUD: the root elements `gmGamePlayUI::SetupChildren`'s child frameworks create
//! (`UIChildFramework::CreateAndAddRootElement(layout enum, element)`), and the images they show.
//!
//! `gmRadarUI` makes `RadarField` and `HeadingField` from the gameplay layout (`0x41000009`),
//! `gmSelectionUI` makes `ItemSelectionField` from it, `gmStatusbarUI` makes `StatusbarField`
//! from the status bar layout (`0x4100000F`) and `gmToolbarUI` makes `ToolbarField`,
//! `LeftSidePanel` and `RightSidePanel` from the toolbar layout (`0x41000003`), in that order.
//! The other children (stack, trade, shop, book, pet, ...) open dialogs on demand and are left out.
//! Of the toolbar's own children (`gmToolbarUI::SetupChildren`) the chat (`gmChatUI`) is made,
//! and the panel children only show and hide their fields ([`ToolbarUi`]); their contents are
//! not made yet.

use std::collections::HashMap;
use std::sync::Arc;

use ac2_core::DataId;
use ac2_db::{ResourceResult, ResourceStore};

use crate::chat::{ChatStrings, ChatUi, CHAT_STRING_TABLE, REPLY_STRING_TABLE, STRING_TABLE_GROUP};
use crate::compose::UiPixels;
use crate::element::UiTree;
use crate::layout::{load_layout, LayoutDesc};
use crate::reticle::Reticle;
use crate::shortcut::ShortcutUi;
use crate::skill_tree::SkillTreeUi;
use crate::text_data::StoreTextData;
use crate::token::tokenize;
use crate::toolbar::ToolbarUi;

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

/// The gameplay HUD's tree and the frameworks that drive it.
#[derive(Debug, Clone)]
pub struct GameplayHud {
    pub tree: UiTree,
    /// `gm3DTextUI`'s targeting reticle.
    pub reticle: Option<Reticle>,
    /// `gmShortcutUI`.
    pub shortcuts: Option<ShortcutUi>,
    /// `gmChatUI`.
    pub chat: Option<ChatUi>,
    /// `gmToolbarUI`'s panel children.
    pub toolbar: ToolbarUi,
    /// `gmSkillUI`'s `gmSkillTreeUI`.
    pub skills: Option<SkillTreeUi>,
}

/// The HUD on a `width` x `height` display; its text is read through `store`, then frozen.
/// `gm3DTextUI` comes first in `gmGamePlayUI::SetupChildren`, so its reticle is made first.
pub fn build_gameplay_hud(
    store: &Arc<ResourceStore>,
    width: i32,
    height: i32,
) -> ResourceResult<GameplayHud> {
    let mut tree = UiTree::new(width, height);
    let text_data = Arc::new(StoreTextData::new(Arc::clone(store)));
    tree.set_text_data(text_data.clone());
    let mut layouts = HashMap::new();
    let gameplay = load_layout_by_enum(store, LAYOUT_GAMEPLAY)?;
    let reticle = gameplay
        .as_ref()
        .and_then(|layout| Reticle::create(&mut tree, layout));
    layouts.insert(LAYOUT_GAMEPLAY, gameplay);
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
    // `gmToolbarUI::SetupChildren`: the shortcut bar first, then the chat.
    let toolbar_field = tree.roots().iter().copied().find(|&r| {
        tree.get(r)
            .is_some_and(|e| e.id() == tokenize("ToolbarField"))
    });
    let shortcuts = match (&layouts[&LAYOUT_TOOLBAR], toolbar_field) {
        (Some(toolbar), Some(field)) => Some(ShortcutUi::create(&mut tree, toolbar, field)),
        _ => None,
    };
    let chat = match &layouts[&LAYOUT_TOOLBAR] {
        Some(toolbar) => Some(ChatUi::create(&mut tree, toolbar, chat_strings(store)?)),
        None => None,
    };
    // `gmSkillUI::gmSkillUI`: the skill tree over `SkillField`.
    let skill_field = tree
        .roots()
        .iter()
        .find_map(|&r| tree.child_recursive(r, tokenize("SkillField")));
    let skills = match (&layouts[&LAYOUT_TOOLBAR], skill_field) {
        (Some(toolbar), Some(field)) => Some(SkillTreeUi::create(&tree, toolbar, field)),
        _ => None,
    };
    let toolbar = ToolbarUi::create(&mut tree);
    text_data.freeze();
    Ok(GameplayHud {
        tree,
        reticle,
        shortcuts,
        chat,
        toolbar,
        skills,
    })
}

/// The chat's strings (`StringInfo::SetTableEnum` of the chat and reply tables).
fn chat_strings(store: &ResourceStore) -> ResourceResult<ChatStrings> {
    let table = |table_enum: u32| -> ResourceResult<Option<ac2_db::StringTable>> {
        let Some(did) = ac2_assets::ac2_enum_did(store, STRING_TABLE_GROUP, table_enum)? else {
            return Ok(None);
        };
        Ok(ac2_db::load_string_table(store, did)?.map(|t| (*t).clone()))
    };
    let chat = table(CHAT_STRING_TABLE)?;
    let reply = table(REPLY_STRING_TABLE)?;
    let entry = |t: &Option<ac2_db::StringTable>, name: &str| {
        t.as_ref().and_then(|t| t.get(tokenize(name))).cloned()
    };
    let text = |t: &Option<ac2_db::StringTable>, name: &str| {
        entry(t, name).map(|s| s.text().encode_utf16().collect())
    };
    Ok(ChatStrings {
        command_prefix: text(&chat, "ID_Chat_CommandPrefix"),
        alias_prefix: text(&chat, "ID_Chat_AliasPrefix"),
        unknown_command: text(&chat, "ID_Chat_UnknownCommand"),
        usage_message: text(&chat, "ID_Chat_UsageMessage"),
        chat_entry: text(&chat, "ID_Chat_ChatEntry"),
        reply: text(&reply, "ID_Reply"),
        assisted_tell: entry(&chat, "ID_Chat_AssistedTell"),
    })
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
