#![forbid(unsafe_code)]

//! Asheron's Call 2 input and UI: the key maps and action map from the DATs, the input manager
//! that turns key and mouse events into actions (`CInputManager_WIN32`), and the action
//! interpreter that turns actions into the movement, behaviour and camera input the client core
//! consumes (`ActionInterpreter`); and the UI: layouts (`LayoutDesc`), the element tree built
//! from them (`UIElementManager`, `UIElement`) and its software compositor (`UIRegion::DrawHere`).

mod activation;
#[cfg(test)]
mod activation_tests;
mod addition_passes;
mod allegiance;
mod assistance;
mod backpack;
mod button;
mod camera_ui;
mod character;
mod character_options;
mod friends;
mod option_calls;
mod option_expander;
mod option_panel;
pub use option_panel::{OptionAction, OptionPanel};
mod social_notices;
mod social_page;
pub use friends::{FriendsRequest, FriendsUi, FRIEND_ROWS};
mod friends_panel;
pub use friends_panel::{friend_event, status_request, FriendEvent, FriendsPanel, STATUS_TABLE};
#[cfg(test)]
mod character_tests;
mod chat;
mod chat_options;
mod chat_options_setup;
mod chat_options_table;
#[cfg(test)]
mod chat_options_tests;
mod chat_spew;
mod chat_window;
mod client_options_page;
mod client_page;
mod compose;
mod compose_filter;
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
mod examination_effects;
mod examination_tabs;
pub use examination_effects::{expand_examination_effects, splice_effects, ExaminationSource};
#[cfg(test)]
mod examination_tests;
mod external_container;
mod fellow_info;
pub mod fellow_markers;
mod fellow_rows;
mod fellowship;
mod field_rollover;
mod frame_passes;
mod glyph_find;
#[cfg(test)]
mod glyph_find_tests;
mod hud_effects;
pub use hud_effects::HudEffects;
mod game_dialogs;
mod hud;
pub use frame_passes::{FrameHosts, FrameInputs, HudHost, Reloads};
mod hud_notices;
pub use hud_notices::notice_pass;
mod hud_message;
pub use hud_message::{consignment_requests, message_pass, MessageInputs, ShortcutCalls};
mod options_pages;
pub use options_pages::{live_effects, OptionsEffect, OptionsHost, OptionsPages, DISPLAY_KEYS};
mod hud_request;
pub use hud_request::HudRequest;
mod hud_tick;
pub use hud_tick::{tick_pass, TickInputs};
mod hud_source;
mod hud_state;
pub use hud_source::HudSource;
pub use hud_state::Hud;
mod indicators;
mod ingredient;
mod ingredient_slots;
mod input;
#[cfg(test)]
mod input_tests;
mod interpreter;
mod inventory_notices;
mod item_display;
mod journal_quests;
pub mod keymap_file;
#[cfg(test)]
mod keymap_file_tests;
pub use journal_quests::{JournalQuestsUi, JournalSource, QuestTab, CODE_LIST as QUEST_CODE_LIST};
#[cfg(test)]
mod assistance_tests;
mod book;
#[cfg(test)]
mod book_tests;
mod items_3d;
#[cfg(test)]
mod items_3d_tests;
mod journal_time;
mod layout;
mod menu;
mod options_window;
mod pref_labels;
pub use options_window::{ApplyOutcome, Close, OptionsPrefs, OptionsWindow, PageOutcome};
pub use pref_labels::pref_label;
#[cfg(test)]
mod menu_tests;
#[cfg(test)]
mod social_page_tests;
pub use menu::MSG_MENU_NEW_SELECTION;
#[cfg(test)]
mod key_action_audit_tests;
mod layout_media;
mod list_animate;
mod list_box;
mod list_box_select;
mod list_drag;
mod list_goal;
mod list_msize;
mod list_scroll;
#[cfg(test)]
mod list_scroll_tests;
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
pub use book::{book_pass, BookRequest, BookUi};
pub use help::{help_button_topic, load_help_pages, HelpPages, HELP_BUTTONS, KEYSTONE_PANEL_KEY};
pub use help_window::{help_pass, HelpWindow};
mod overview_map;
mod panel;
mod paper_doll;
pub mod paper_doll_scene;
mod pk_status;
mod place;
mod placement;
pub mod pregame;
mod quests_panel;
pub use quests_panel::{QuestSource, QuestsUi};
pub use story_quests::{StoryQuestAction, StoryQuestsUi, STORY_CODE_LIST};
mod catalog;
#[cfg(test)]
mod catalog_tests;
mod consignment;
#[cfg(test)]
mod consignment_tests;
mod gameplay_keys;
mod gameplay_story;
#[cfg(test)]
mod gameplay_story_tests;
mod newsale;
#[cfg(test)]
mod newsale_tests;
mod radar;
mod reticle;
mod world_overlays;
pub use world_overlays::{radar_view, OverlayBox};
mod scene_viewport;
mod scrollbar;
mod secure_trade;
mod selection_notices;
#[cfg(test)]
mod selection_notices_tests;
mod selection_ui;
#[cfg(test)]
mod selection_ui_tests;
mod shop;
mod shop_notices;
mod shop_rows;
#[cfg(test)]
mod shop_tests;
mod shortcut;
mod shortcut_notices;
mod shortcut_states;
mod skill_tree;
mod stack;
mod status_notices;
#[cfg(test)]
mod status_notices_tests;
pub use status_notices::{pk_notice, StatusBar, StatusSource};
mod statusbar;
pub use gameplay_keys::{
    child_for_key, GameplayChild, GAMEPLAY_SHOW_KEY, RADAR_CHILD_KEY, STATUSBAR_CHILD_KEY,
    TOOLBAR_CHILD_KEY, USE_KEY,
};
pub use gameplay_story::{
    story_frame, story_scenes, GameplayStory, PortalSpaceCall, StoryEvent, StoryScene,
};
mod story;
#[cfg(test)]
mod story_layout_tests;
mod story_quests;
#[cfg(test)]
mod story_quests_tests;
pub use story::{StoryField, StoryFrame, StoryPlayback, StoryShow, StoryStep, SKIP_STORY_MESSAGE};
#[cfg(test)]
mod story_tests;
mod story_view;
#[cfg(test)]
mod story_view_tests;
pub use story_view::{subtitle_text, StoryView};
mod book_rows;
pub mod chat_text;
pub mod guide_arrow;
mod guide_page;
pub mod guide_window;
pub mod help;
#[cfg(test)]
mod help_buttons_tests;
#[cfg(test)]
mod help_tests;
pub mod help_view;
#[cfg(test)]
mod help_view_tests;
pub mod help_window;
#[cfg(test)]
mod help_window_tests;
pub mod ksml;
#[cfg(test)]
mod ksml_tests;
mod ksml_xml;
mod resize;
#[cfg(test)]
mod resize_tests;
#[cfg(test)]
mod scroll_coverage_tests;
#[cfg(test)]
mod scroll_wheel_tests;
mod target_cycle;
mod target_select;
pub mod tracker_ui;
mod window_parts;
pub mod world_map;
pub mod world_map_layers;
pub mod world_map_sidecar;
pub mod world_map_tiles;
pub use target_select::{
    frame_selection_calls, select_key_calls, SelectionCall, SelectionObject, SelectionSource,
};
mod icon_desc;
#[cfg(test)]
mod tests;
mod text;
mod text_attr;
mod text_clip;
mod text_clipboard;
mod text_data;
mod text_edit;
pub use text_edit::TextPress;
pub mod ac1_buttons;
pub mod hud_style;
pub mod modern_art;
mod modern_inventory;
#[cfg(test)]
mod modern_inventory_tests;
mod modern_strip;
mod modern_toolbar;
#[cfg(test)]
mod modern_toolbar_tests;
mod text_scroll;
mod token;
mod toolbar;
mod tooltip;
pub use hud_style::HudStyle;
pub mod tooltip_prefs;
mod transition;
mod tree_scroll;
mod tree_view;
mod wheel;
#[cfg(test)]
mod wheel_tests;
mod window_locations;
mod window_wheel;
pub use wheel::{wheel_event_delta, WheelAccumulator, PIXELS_PER_NOTCH, WHEEL_DELTA};
pub use window_locations::{
    clip_to_pixel, pixel_to_clip, saved_windows, ui_save_locations, SavedWindow, WindowLocation,
};
mod window_positions;
pub use window_positions::{
    parse as parse_window_positions, radar_coords_hidden_file, read_radar_coords_hidden,
    read_window_positions, to_text as window_positions_text, window_positions_file,
    write_radar_coords_hidden, write_window_positions, WindowPosition,
};
mod radar_state;
pub use radar_state::{
    add_map_note, groove_of, MapMarker, MarkerRequest, RadarBlip, RadarFrame, RadarState,
    GROOVE_DELAY,
};
mod ui_images;
mod ui_text;
pub use ui_images::UiImageLoader;
pub use ui_text::{
    game_date_time, map_coords_string, shortcut_from_info, shortcut_to_info, UiStringTables,
    UiText, GAME_TIME_TABLE, MAP_COORDS_TABLE,
};
mod ui_string;
mod vitae;
mod vitals;
#[cfg(test)]
mod window_locations_tests;
#[cfg(test)]
mod window_positions_tests;

pub use allegiance::{AllegianceRequest, AllegianceSource, AllegianceUi};
pub use assistance::{
    assistance_message, assistance_outcomes, AssistancePanel, AssistanceRequest, AssistanceSource,
    CALL_TYPES,
};
pub use backpack::BackpackUi;
pub use camera_ui::{CameraUi, CameraUiEffect};
pub use catalog::{CatalogRequest, CatalogUi};
pub use character::{CharacterRequest, CharacterUi};
pub use character_options::{
    OptionSetting, OptionsSettings, Setter, PROFILE_HIDE_PET_UI, PROFILE_LATENCY_INDICATOR_GRAPH,
};
pub use chat::{
    ChatHistory, ChatRequest, ChatSettings, ChatStrings, ChatUi, Modifiers, CHAT_STRING_TABLE,
    REPLY_STRING_TABLE, STRING_TABLE_GROUP,
};
pub use chat_options::{ChatOptionRequest, ChatOptionSetting, ChatOptionsUi};
pub use chat_text::ChatText;
pub use chat_window::{chat_window_id, ChatMessageInfo, CHAT_TEXT_TYPES, TEXT_TYPE_STANDARD};
pub use client_options_page::{ClientOptionsUi, ClientRequest};
pub use client_page::{
    ButtonStates, ClientPage, ClientPageText, PageAction, RENDERING_SECTION, SECTION_ORDER,
};
pub use compose::{UiPixels, UiSurface};
pub use consignment::{ConsignmentRequest, ConsignmentSource, ConsignmentUi};
pub use controls::{ControlFrame, Controls};
pub use craft::{CraftCommand, CraftSource, CraftUi, STATE_CATEGORY, STATE_RECIPE, STATE_SKILL};
pub use damage_text::{DamagePlace, DamageTexts};
pub use death::{death_notices, resurrection_prompt, you_died_string, DeathPanel, OfferAnswer};
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
pub use game_dialogs::{
    display_notices, popup_questions, DialogRequest, DialogText, DisplayNotice, GameplayDialogs,
    PopupQuestion, ERROR_TEXT_TYPE, LOGOUT_ACTION,
};
pub use hud::{
    build_gameplay_hud, build_styled_hud, load_images, load_layout_by_enum, load_ui_images,
    GameplayHud, GAMEPLAY_ROOTS, LAYOUT_GAMEPLAY, LAYOUT_STATUSBAR, LAYOUT_TOOLBAR, UILAYOUT_GROUP,
};
pub use icon_desc::IconLayer;
pub use indicators::IndicatorsUi;
pub use ingredient::{IngredientCommand, IngredientSource, IngredientUi};
pub use input::{
    clipboard_control, is_ui_action, meta_mode_from_key, typed_text, InputEvent, InputManager,
    KeyPress, RawAction, ACTION_POINTER_X, ACTION_POINTER_Y, DEVICE_KEYBOARD, DEVICE_MOUSE,
};
pub use interpreter::{
    ActionInfo, ActionInterpreter, Dispatch, CLASS_BEHAVIOR, CLASS_CAMERA, CLASS_MODIFIER,
    CLASS_MOVEMENT, CLASS_UI, TOGGLE_HOLD, TOGGLE_ONE_SHOT, TOGGLE_TOGGLE,
};
pub use inventory_notices::{
    backpack_notice, external_container_event, item_dialog, paper_doll_notice, secure_trade_notice,
    secure_trade_split, stack_event, InventoryNoticeSource,
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
pub use newsale::{NewSaleRequest, NewSaleSource, NewSaleUi};
pub use option_calls::{chat_option_call, option_call};
pub use overview_map::OverviewMapUi;
pub use paper_doll::{mesh_type_to_inv_loc, PaperDollUi};
pub use pets::{next_mode, pet_notice, previous_mode, PetAction, PetSource, PetUi, PET_MODES};
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
pub use selection_ui::{PotentialTarget, SelectionClick, SelectionOutcome, SelectionUi};
pub use shop::{ShopRequest, ShopSource, ShopStackCall, ShopUi};
pub use shop_notices::{catalog_notice, consignment_notice, shop_notice, shop_stack_calls};
pub use shortcut::{
    Shortcut, ShortcutOptions, ShortcutRequest, ShortcutSource, ShortcutUi, SHORTCUT_ALIAS,
    SHORTCUT_EMPTY, SHORTCUT_ITEM, SHORTCUT_RECIPE, SHORTCUT_SKILL,
};
pub use shortcut_notices::{apply_shortcut_checks, shortcut_checks_of, ShortcutCheck};
pub use skill_tree::{
    SkillNode, SkillPanel, SkillRequest, SkillSource, SkillStatus, SkillTreeUi, SkillVar,
    EXAMINE_PANEL, EXAMINE_SKILL, NODE_SKILL, ROOT_MAGIC_PANEL, ROOT_MELEE_PANEL,
    ROOT_MISSILE_PANEL, ROOT_NONCOMBAT_PANEL,
};
pub use social_notices::{
    allegiance_calls, allegiance_notice, fellowship_calls, fellowship_notice,
    social_dialog_outcomes, SocialCall, SocialSource,
};
pub use social_page::{SocialAction, SocialPanel, SOCIAL_HELP_TOPIC};
pub use stack::{StackUi, StackUiDone};
pub use statusbar::{show_statusbar, statusbar_shown};
pub use target_cycle::{is_select_key, select_key_of, select_target, Candidate};
pub use text::UiTextData;
pub use text_clipboard::{MemoryClipboard, TextClipboard};
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
mod element_names_tests;
#[cfg(test)]
mod element_tests;
#[cfg(test)]
mod external_container_tests;
#[cfg(test)]
mod fellowship_tests;
#[cfg(test)]
mod friends_panel_tests;
#[cfg(test)]
mod indicators_tests;
#[cfg(test)]
mod journal_quests_tests;
#[cfg(test)]
mod list_box_select_tests;
#[cfg(test)]
mod list_box_tests;
#[cfg(test)]
mod list_drag_tests;
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
mod reticle_color_tests;
#[cfg(test)]
mod scrollbar_tests;
#[cfg(test)]
mod secure_trade_tests;
#[cfg(test)]
mod shortcut_checks_tests;
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
mod target_select_tests;
#[cfg(test)]
mod text_bar_tests;
#[cfg(test)]
mod text_clip_tests;
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
