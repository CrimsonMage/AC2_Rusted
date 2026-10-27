#![forbid(unsafe_code)]

//! Asheron's Call 2 input and UI: the key maps and action map from the DATs, the input manager
//! that turns key and mouse events into actions (`CInputManager_WIN32`), and the action
//! interpreter that turns actions into the movement, behaviour and camera input the client core
//! consumes (`ActionInterpreter`); and the UI: layouts (`LayoutDesc`), the element tree built
//! from them (`UIElementManager`, `UIElement`) and its software compositor (`UIRegion::DrawHere`).

mod allegiance;
mod backpack;
mod button;
mod camera_ui;
mod character;
mod character_options;
mod friends;
pub use friends::{FriendsRequest, FriendsUi, FRIEND_ROWS};
mod friends_panel;
pub use friends_panel::{status_request, FriendsPanel, STATUS_TABLE};
#[cfg(test)]
mod character_tests;
mod chat;
mod chat_options;
mod chat_options_setup;
mod chat_options_table;
#[cfg(test)]
mod chat_options_tests;
mod chat_window;
mod compose;
mod controls;
mod craft;
mod craft_pages;
mod damage_text;
mod death;
mod dialog;
mod drag;
mod effects;
mod element;
mod examination;
mod external_container;
mod fellow_info;
mod fellow_rows;
mod fellowship;
mod game_dialogs;
mod hud;
mod indicators;
mod ingredient;
mod ingredient_slots;
mod input;
mod interpreter;
mod item_display;
mod journal_quests;
pub use journal_quests::{JournalQuestsUi, JournalSource, QuestTab, CODE_LIST as QUEST_CODE_LIST};
mod book;
#[cfg(test)]
mod book_tests;
mod items_3d;
#[cfg(test)]
mod items_3d_tests;
mod journal_time;
mod layout;
mod layout_media;
mod list_box;
mod list_box_select;
mod maps;
mod media;
mod meter;
mod money_bag;
mod mouse;
mod name_plates;
mod navigation;
mod navigation_detail;
#[cfg(test)]
mod navigation_detail_tests;
mod navigation_map;
mod net_status;
pub use book::{BookRequest, BookUi};
mod overview_map;
mod panel;
mod paper_doll;
mod pk_status;
mod place;
mod placement;
pub mod pregame;
mod quests_panel;
pub use quests_panel::QuestsUi;
mod catalog;
#[cfg(test)]
mod catalog_tests;
mod gameplay_keys;
mod gameplay_story;
#[cfg(test)]
mod gameplay_story_tests;
mod radar;
mod reticle;
mod scene_viewport;
mod scrollbar;
mod secure_trade;
mod selection_notices;
#[cfg(test)]
mod selection_notices_tests;
mod selection_ui;
mod shop;
mod shop_rows;
#[cfg(test)]
mod shop_tests;
mod shortcut;
mod shortcut_states;
mod skill_tree;
mod stack;
mod statusbar;
pub use gameplay_keys::{
    child_for_key, GameplayChild, GAMEPLAY_SHOW_KEY, RADAR_CHILD_KEY, STATUSBAR_CHILD_KEY,
    TOOLBAR_CHILD_KEY,
};
pub use gameplay_story::{GameplayStory, PortalSpaceCall, StoryEvent, StoryScene};
mod story;
#[cfg(test)]
mod story_layout_tests;
pub use story::{StoryField, StoryFrame, StoryPlayback, StoryShow, StoryStep, SKIP_STORY_MESSAGE};
#[cfg(test)]
mod story_tests;
mod story_view;
#[cfg(test)]
mod story_view_tests;
pub use story_view::StoryView;
mod target_cycle;
#[cfg(test)]
mod tests;
mod text;
mod text_attr;
mod text_data;
mod text_edit;
mod token;
mod toolbar;
mod tooltip;
mod transition;
mod tree_view;
mod ui_string;
mod vitae;
mod vitals;

pub use allegiance::{AllegianceRequest, AllegianceSource, AllegianceUi};
pub use backpack::BackpackUi;
pub use camera_ui::{CameraUi, CameraUiEffect};
pub use catalog::{CatalogRequest, CatalogUi};
pub use character::{CharacterRequest, CharacterUi};
pub use character_options::{
    OptionSetting, OptionsSettings, Setter, PROFILE_LATENCY_INDICATOR_GRAPH,
};
pub use chat::{
    ChatHistory, ChatRequest, ChatSettings, ChatStrings, ChatUi, Modifiers, CHAT_STRING_TABLE,
    REPLY_STRING_TABLE, STRING_TABLE_GROUP,
};
pub use chat_options::{ChatOptionRequest, ChatOptionSetting, ChatOptionsUi};
pub use chat_window::{ChatMessageInfo, CHAT_TEXT_TYPES};
pub use compose::{UiPixels, UiSurface};
pub use controls::{ControlFrame, Controls};
pub use craft::{CraftCommand, CraftSource, CraftUi, STATE_CATEGORY, STATE_RECIPE, STATE_SKILL};
pub use damage_text::{DamagePlace, DamageTexts};
pub use death::{resurrection_prompt, you_died_string, DeathPanel, OfferAnswer};
pub use dialog::{
    DialogFactory, DialogOutcome, DIALOG_CONFIRMATION, DIALOG_MESSAGE, DIALOG_RESURRECT,
    DIALOG_TIMED, DIALOG_WAIT, LAYOUT_DIALOGS,
};
pub use drag::DragCheck;
pub use effects::{EffectEntry, EffectsUi, ExamineEffect};
pub use element::{UiBox, UiElement, UiImage, UiPlacement, UiTree};
pub use examination::{ExaminationCall, ExaminationItem, ExaminationUi};
pub use external_container::{ExternalContainerRequest, ExternalContainerUi};
pub use fellow_info::{vital_text, Fellow, FellowStrings};
pub use fellow_rows::FellowNotice;
pub use fellowship::{FellowshipData, FellowshipRequest, FellowshipSource, FellowshipUi};
pub use game_dialogs::{GameplayDialogs, ERROR_TEXT_TYPE, LOGOUT_ACTION};
pub use hud::{
    build_gameplay_hud, load_layout_by_enum, load_ui_images, GameplayHud, GAMEPLAY_ROOTS,
    LAYOUT_GAMEPLAY, LAYOUT_STATUSBAR, LAYOUT_TOOLBAR, UILAYOUT_GROUP,
};
pub use indicators::IndicatorsUi;
pub use ingredient::{IngredientCommand, IngredientSource, IngredientUi};
pub use input::{
    is_ui_action, meta_mode_from_key, InputEvent, InputManager, KeyPress, RawAction,
    ACTION_POINTER_X, ACTION_POINTER_Y, DEVICE_KEYBOARD, DEVICE_MOUSE,
};
pub use interpreter::{
    ActionInfo, ActionInterpreter, Dispatch, CLASS_BEHAVIOR, CLASS_CAMERA, CLASS_MODIFIER,
    CLASS_MOVEMENT, CLASS_UI, TOGGLE_HOLD, TOGGLE_ONE_SHOT, TOGGLE_TOGGLE,
};
pub use item_display::{ContainerDisplay, InventorySource, ItemDisplay, ItemRequest, ItemSlot};
pub use items_3d::Items3dUi;
pub use layout::{
    load_layout, AttributeDesc, AttributeValue, ElementDesc, LayoutDesc, MediaDesc, Rect,
    StateDesc, TransitionData, TransitionDesc, LAYOUT_FIRST, LAYOUT_LAST,
};
pub use list_box_select::{CLICK_SELECT, MSG_NEW_SELECTION};
pub use maps::{
    load_action_map, load_key_map, ActionMap, ActionMapping, ActionValue, KeyMap, KEYMAP_CONTROL,
    KEYMAP_GROUP, KEYMAP_MAIN, KEYMAP_SHIFT,
};
pub use meter::{VitalDisplay, VitalText};
pub use mini_fellow::MiniFellowUi;
pub use money_bag::MoneyBagUi;
pub use mouse::{
    is_mouse_button, UiMessage, GLOBAL_KEY_PRESS, GLOBAL_KEY_RELEASE, GLOBAL_MOUSE_DOWN,
    GLOBAL_MOUSE_MOVE, GLOBAL_MOUSE_UP, UI_POINTER_X, UI_POINTER_Y, UI_WHEEL_IN, UI_WHEEL_OUT,
};
pub use name_plates::{NameLook, NamePlace, NamePlates};
pub use navigation::{MapNote, NavigationUi, NavigationView};
pub use navigation_detail::{
    mul_div, tile_position, window_origin, DETAIL_WINDOW, MAP_FIELD_PRESS, MAP_PARENT_SWITCH,
    PAN_STEP, PAN_STEP_SHIFT,
};
pub use navigation_map::MapFrame;
pub use net_status::{LinkSample, NetStatusUi};
pub use overview_map::OverviewMapUi;
pub use paper_doll::{mesh_type_to_inv_loc, PaperDollUi};
pub use pets::{next_mode, previous_mode, PetAction, PetUi, PET_MODES};
pub use pk_status::PkStatusUi;
pub use radar::{RadarDot, RadarRequest, RadarUi, RadarView};
pub use reticle::{Reticle, ReticleOutcome, ReticleTarget};
pub use scene_viewport::{scene_viewports, SceneViewport};
pub use scrollbar::MSG_NEW_POSITION;
pub use secure_trade::{SecureTradeRequest, SecureTradeSplit, SecureTradeUi};
pub use selection_notices::{
    being_deleted, direct_key, last_attacker_notice, selectability_notice, set_targeting_pointer,
    SelectionAction, SelectionSnapshot,
};
pub use selection_ui::{SelectionClick, SelectionOutcome, SelectionUi};
pub use shop::{ShopRequest, ShopSource, ShopStackCall, ShopUi};
pub use shortcut::{
    Shortcut, ShortcutOptions, ShortcutRequest, ShortcutSource, ShortcutUi, SHORTCUT_ALIAS,
    SHORTCUT_EMPTY, SHORTCUT_ITEM, SHORTCUT_RECIPE, SHORTCUT_SKILL,
};
pub use skill_tree::{
    SkillNode, SkillPanel, SkillRequest, SkillSource, SkillStatus, SkillTreeUi, SkillVar,
    EXAMINE_PANEL, EXAMINE_SKILL, NODE_SKILL, ROOT_MAGIC_PANEL, ROOT_MELEE_PANEL,
    ROOT_MISSILE_PANEL, ROOT_NONCOMBAT_PANEL,
};
pub use stack::{StackUi, StackUiDone};
pub use statusbar::{show_statusbar, statusbar_shown};
pub use target_cycle::{is_select_key, select_key_of, select_target, Candidate};
pub use text::UiTextData;
pub use text_data::{StoreTextData, DEFAULT_FONT, FONT_GROUP};
pub use token::{
    tokenize, MSG_BUTTON_RELEASE_ON, MSG_MOUSE_PRESS, MSG_MOUSE_PRESS_TOP, REFERENCE, REFERENT,
    STATELESS,
};
pub use toolbar::{ToolbarChild, ToolbarUi};
pub use ui_string::{UiString, UiStrings, UI_TABLE};
pub use vitae::{VitaeStatusUi, EXAMINE_VITAE};
pub use vitals::{
    VitalsUi, VITAL_EXPERIENCE, VITAL_FOCUS, VITAL_HEALTH, VITAL_PK_DAMAGE, VITAL_PK_VIGOR_LOSS,
    VITAL_VIGOR,
};

#[cfg(test)]
mod allegiance_tests;
#[cfg(test)]
mod backpack_tests;
#[cfg(test)]
mod chat_tests;
#[cfg(test)]
mod damage_text_tests;
#[cfg(test)]
mod dialog_tests;
#[cfg(test)]
mod drag_tests;
#[cfg(test)]
mod element_tests;
#[cfg(test)]
mod external_container_tests;
#[cfg(test)]
mod fellowship_tests;
#[cfg(test)]
mod indicators_tests;
#[cfg(test)]
mod journal_quests_tests;
#[cfg(test)]
mod list_box_select_tests;
#[cfg(test)]
mod list_box_tests;
#[cfg(test)]
mod media_tests;
#[cfg(test)]
mod meter_tests;
mod mini_fellow;
#[cfg(test)]
mod mini_fellow_tests;
#[cfg(test)]
mod money_bag_tests;
#[cfg(test)]
#[cfg(test)]
mod mouse_tests;
#[cfg(test)]
mod name_plates_tests;
#[cfg(test)]
mod navigation_tests;
#[cfg(test)]
mod paper_doll_tests;
mod pets;
#[cfg(test)]
mod pets_tests;
#[cfg(test)]
mod quests_panel_tests;
#[cfg(test)]
mod radar_tests;
#[cfg(test)]
mod real_data_tests;
#[cfg(test)]
mod scrollbar_tests;
#[cfg(test)]
mod secure_trade_tests;
#[cfg(test)]
mod shortcut_states_tests;
#[cfg(test)]
mod shortcut_tests;
#[cfg(test)]
mod skill_tree_tests;
#[cfg(test)]
mod stack_tests;
#[cfg(test)]
mod statusbar_tests;
#[cfg(test)]
mod target_cycle_tests;
#[cfg(test)]
mod text_tests;
#[cfg(test)]
mod toolbar_tests;
#[cfg(test)]
mod tooltip_tests;
#[cfg(test)]
mod tree_tests;
#[cfg(test)]
mod vitals_tests;
