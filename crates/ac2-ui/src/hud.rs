//! The gameplay HUD: the root elements `gmGamePlayUI::SetupChildren`'s child frameworks create
//! (`UIChildFramework::CreateAndAddRootElement(layout enum, element)`), and the images they show.
//!
//! `gmRadarUI` makes `RadarField` and `HeadingField` from the gameplay layout (`0x41000009`),
//! `gmSelectionUI` makes `ItemSelectionField` from it, `gmStatusbarUI` makes `StatusbarField`
//! from the status bar layout (`0x4100000F`) and `gmToolbarUI` makes `ToolbarField`,
//! `LeftSidePanel` and `RightSidePanel` from the toolbar layout (`0x41000003`), in that order.
//! `gmStackUI` and `gmExternalContainerUI` make their hidden `StackableField` and
//! `ExternalContainerField` from the gameplay layout; `gmSecureTradeUI` takes the gameplay layout
//! too and makes its `SecureTradeField` when a trade opens. The other children
//! (shop, book, pet, ...) open dialogs on demand and are left out.
//! Of the toolbar's own children (`gmToolbarUI::SetupChildren`) the chat (`gmChatUI`) is made,
//! and the panel children only show and hide their fields ([`ToolbarUi`]); their contents are
//! not made yet.

use std::collections::HashMap;
use std::sync::Arc;

use ac2_core::DataId;
use ac2_db::{ResourceResult, ResourceStore};

use crate::allegiance::AllegianceUi;
use crate::character::CharacterUi;
use crate::chat::{ChatStrings, ChatUi, CHAT_STRING_TABLE, REPLY_STRING_TABLE, STRING_TABLE_GROUP};
use crate::chat_options::ChatOptionsUi;
use crate::compose::UiPixels;
use crate::craft::CraftUi;
use crate::damage_text::DamageTexts;
use crate::dialog::DialogFactory;
use crate::effects::EffectsUi;
use crate::element::UiTree;
use crate::examination::ExaminationUi;
use crate::external_container::ExternalContainerUi;
use crate::fellowship::FellowshipUi;
use crate::indicators::IndicatorsUi;
use crate::ingredient::IngredientUi;
use crate::layout::{load_layout, LayoutDesc};
use crate::mini_fellow::MiniFellowUi;
use crate::name_plates::NamePlates;
use crate::navigation::NavigationUi;
use crate::overview_map::OverviewMapUi;
use crate::quests_panel::QuestsUi;
use crate::radar::RadarUi;
use crate::reticle::Reticle;
use crate::secure_trade::SecureTradeUi;
use crate::shop::ShopUi;
use crate::shortcut::ShortcutUi;
use crate::skill_tree::SkillTreeUi;
use crate::stack::StackUi;
use crate::statusbar::show_statusbar;
use crate::text_data::StoreTextData;
use crate::token::tokenize;
use crate::toolbar::ToolbarUi;
use crate::ui_string::UI_TABLE;
use crate::vitals::VitalsUi;

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
    /// `gm3DTextUI`'s targeting reticle and name plates.
    pub reticle: Option<Reticle>,
    pub names: Option<NamePlates>,
    /// `gm3DTextUI`'s floating damage numbers.
    pub damage: Option<DamageTexts>,
    /// `gmShortcutUI`.
    pub shortcuts: Option<ShortcutUi>,
    /// `gmChatUI`.
    pub chat: Option<ChatUi>,
    /// `gmToolbarUI`'s panel children.
    pub toolbar: ToolbarUi,
    /// `gmSkillUI`'s `gmSkillTreeUI`.
    pub skills: Option<SkillTreeUi>,
    /// `gmCraftUI`, over `CraftPage`.
    pub craft: Option<CraftUi>,
    /// `gmIngredientUI`, over the gameplay layout (its root made when a recipe opens).
    pub ingredient: Option<IngredientUi>,
    /// `gmExaminationUI`.
    pub examination: Option<ExaminationUi>,
    /// `gmSocialUI`'s `gmAllegianceUI` (its player set once in the world) and `gmFellowshipUI`.
    pub allegiance: Option<AllegianceUi>,
    pub fellowship: Option<FellowshipUi>,
    /// `gmMiniFellowUI`, the fellowship mini panel (hidden until its button is pressed).
    pub mini_fellow: Option<MiniFellowUi>,
    /// `gmQuestsUI`, the Quest panel: its journal over `UnderwayPage` and `FinishedPage`.
    pub quests: Option<QuestsUi>,
    /// The toolbar layout, for the frameworks made once the player is in the world
    /// (`gmInventoryUI`'s backpack).
    pub toolbar_layout: Option<Arc<LayoutDesc>>,
    /// `gmStatusbarUI`'s `gmVitalsUI` and `gmEffectsUI`.
    pub vitals: VitalsUi,
    pub effects: Option<EffectsUi>,
    /// `gmStatusbarUI`'s `gmIndicatorsUI` (its `gmVitaeStatusUI` is ported).
    pub indicators: IndicatorsUi,
    /// `DialogFactory` over the gameplay layout.
    pub dialogs: DialogFactory,
    /// `gmGamePlayUI`'s `gmStackUI`, hidden until a split asks for an amount.
    pub stack: Option<StackUi>,
    /// `gmGamePlayUI`'s `gmExternalContainerUI`, hidden until a container opens.
    pub external_container: Option<ExternalContainerUi>,
    /// `gmRadarUI`'s blips and heading ring.
    pub radar: RadarUi,
    /// `gmGamePlayUI`'s `gmSecureTradeUI`, hidden until a trade opens.
    pub secure_trade: Option<SecureTradeUi>,
    /// `gmCharacterUI`, the character options page (`CharacterPage`).
    pub character: Option<CharacterUi>,
    /// `gmChatOptionsUI`, the chat options page (`ChatPage`).
    pub chat_options: Option<ChatOptionsUi>,
    /// `gmNavigationUI`, the navigation panel's macro map.
    pub navigation: Option<NavigationUi>,
    /// `gmOverviewMapUI`, hidden until a map is shown.
    pub overview: Option<OverviewMapUi>,
    /// `gmShopUI`, the vendor's panel (its managed elements are made when a store opens).
    pub shop: Option<ShopUi>,
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
    let names = match (&gameplay, &reticle) {
        (Some(layout), Some(reticle)) => {
            let (frames, attempts) = reticle.name_tuning();
            Some(NamePlates::create(layout, frames, attempts))
        }
        _ => None,
    };
    let damage = match (&gameplay, &reticle) {
        (Some(layout), Some(reticle)) => {
            let (frames, attempts) = reticle.name_tuning();
            Some(DamageTexts::create(
                layout,
                frames,
                attempts,
                reticle.floaty_travel(),
            ))
        }
        _ => None,
    };
    layouts.insert(LAYOUT_GAMEPLAY, gameplay);
    for (layout_enum, name) in GAMEPLAY_ROOTS {
        if let std::collections::hash_map::Entry::Vacant(e) = layouts.entry(layout_enum) {
            e.insert(load_layout_by_enum(store, layout_enum)?);
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
    // `gmGamePlayUI::SetupChildren` makes the mini panel after the toolbar's roots.
    let mini_fellow = layouts[&LAYOUT_GAMEPLAY]
        .as_ref()
        .map(|gameplay| MiniFellowUi::create(&mut tree, gameplay));
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
    // `gmCraftUI::gmCraftUI`: the craft panel over `CraftPage`.
    let craft_page = tree
        .roots()
        .iter()
        .find_map(|&r| tree.child_recursive(r, crate::craft::CRAFT_PAGE));
    let ingredient = layouts[&LAYOUT_GAMEPLAY].as_ref().map(IngredientUi::new);
    let craft = match (&layouts[&LAYOUT_TOOLBAR], craft_page) {
        (Some(toolbar), Some(page)) => Some(CraftUi::create(toolbar, page)),
        _ => None,
    };
    let examination = layouts[&LAYOUT_TOOLBAR]
        .as_ref()
        .map(|toolbar| ExaminationUi::create(&mut tree, toolbar));
    let allegiance = layouts[&LAYOUT_TOOLBAR]
        .as_ref()
        .map(|toolbar| AllegianceUi::create(&tree, toolbar, 0));
    let fellowship = layouts[&LAYOUT_TOOLBAR]
        .as_ref()
        .map(|toolbar| FellowshipUi::create(&mut tree, toolbar));
    let quests = layouts[&LAYOUT_TOOLBAR]
        .as_ref()
        .map(|toolbar| QuestsUi::create(&tree, toolbar));
    let vitals = VitalsUi::create(&mut tree);
    let effects = layouts[&LAYOUT_STATUSBAR]
        .as_ref()
        .map(|statusbar| EffectsUi::create(&tree, statusbar));
    let indicators = IndicatorsUi::create(&mut tree);
    // `gmStatusbarUI::gmStatusbarUI`: `Show(1)` once its children are made.
    show_statusbar(&mut tree, true);
    let dialogs = DialogFactory::new(layouts[&LAYOUT_GAMEPLAY].clone());
    let stack = layouts[&LAYOUT_GAMEPLAY]
        .as_ref()
        .map(|gameplay| StackUi::create(&mut tree, gameplay));
    let external_container = match (&layouts[&LAYOUT_GAMEPLAY], &layouts[&LAYOUT_TOOLBAR]) {
        (Some(gameplay), Some(toolbar)) => {
            Some(ExternalContainerUi::create(&mut tree, gameplay, toolbar))
        }
        _ => None,
    };
    let toolbar = ToolbarUi::create(&mut tree);
    let radar = RadarUi::create(&tree);
    let secure_trade = layouts[&LAYOUT_GAMEPLAY]
        .as_ref()
        .map(SecureTradeUi::create);
    let ui_table = ac2_assets::ac2_enum_did(store, STRING_TABLE_GROUP, UI_TABLE)?;
    let character = match (&layouts[&LAYOUT_TOOLBAR], ui_table) {
        (Some(toolbar), Some(table)) => {
            CharacterUi::create(&mut tree, toolbar, &*text_data, table.raw())
        }
        _ => None,
    };
    let chat_options = match (&layouts[&LAYOUT_TOOLBAR], ui_table) {
        (Some(toolbar), Some(table)) => {
            ChatOptionsUi::create(&mut tree, toolbar, &*text_data, table.raw())
        }
        _ => None,
    };
    let navigation = layouts[&LAYOUT_TOOLBAR]
        .as_ref()
        .and_then(|toolbar| NavigationUi::create(&mut tree, toolbar));
    let overview = layouts[&LAYOUT_GAMEPLAY]
        .as_ref()
        .and_then(|gameplay| OverviewMapUi::create(&mut tree, gameplay));
    let shop = layouts[&LAYOUT_GAMEPLAY].as_ref().map(ShopUi::create);
    for layout in layouts.values().flatten() {
        crate::tooltip::prefetch_tooltip_strings(&*text_data, layout);
    }
    text_data.freeze();
    Ok(GameplayHud {
        tree,
        reticle,
        names,
        damage,
        shortcuts,
        chat,
        toolbar,
        skills,
        craft,
        ingredient,
        examination,
        allegiance,
        fellowship,
        mini_fellow,
        quests,
        vitals,
        effects,
        indicators,
        toolbar_layout: layouts[&LAYOUT_TOOLBAR].clone(),
        dialogs,
        stack,
        external_container,
        radar,
        secure_trade,
        character,
        chat_options,
        navigation,
        overview,
        shop,
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
