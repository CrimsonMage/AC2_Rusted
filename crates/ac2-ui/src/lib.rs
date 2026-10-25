#![forbid(unsafe_code)]

//! Asheron's Call 2 input and UI: the key maps and action map from the DATs, the input manager
//! that turns key and mouse events into actions (`CInputManager_WIN32`), and the action
//! interpreter that turns actions into the movement, behaviour and camera input the client core
//! consumes (`ActionInterpreter`); and the UI: layouts (`LayoutDesc`), the element tree built
//! from them (`UIElementManager`, `UIElement`) and its software compositor (`UIRegion::DrawHere`).

mod allegiance;
mod button;
mod camera_ui;
mod chat;
mod chat_window;
mod compose;
mod controls;
mod dialog;
mod element;
mod examination;
mod fellow_info;
mod fellowship;
mod hud;
mod input;
mod interpreter;
mod layout;
mod layout_media;
mod list_box;
mod list_box_select;
mod maps;
mod media;
mod meter;
mod mouse;
mod panel;
mod place;
pub mod pregame;
mod reticle;
mod scrollbar;
mod selection_ui;
mod shortcut;
mod skill_tree;
#[cfg(test)]
mod tests;
mod text;
mod text_attr;
mod text_data;
mod text_edit;
mod token;
mod toolbar;
mod transition;
mod tree_view;
mod ui_string;
mod vitals;

pub use allegiance::{AllegianceRequest, AllegianceSource, AllegianceUi};
pub use camera_ui::{CameraUi, CameraUiEffect};
pub use chat::{
    ChatHistory, ChatRequest, ChatSettings, ChatStrings, ChatUi, Modifiers, CHAT_STRING_TABLE,
    REPLY_STRING_TABLE, STRING_TABLE_GROUP,
};
pub use chat_window::{ChatMessageInfo, CHAT_TEXT_TYPES};
pub use compose::{UiPixels, UiSurface};
pub use controls::{ControlFrame, Controls};
pub use dialog::{
    DialogFactory, DialogOutcome, DIALOG_CONFIRMATION, DIALOG_MESSAGE, DIALOG_RESURRECT,
    DIALOG_TIMED, DIALOG_WAIT, LAYOUT_DIALOGS,
};
pub use element::{UiBox, UiElement, UiImage, UiPlacement, UiTree};
pub use examination::{ExaminationCall, ExaminationItem, ExaminationUi};
pub use fellow_info::{vital_text, Fellow, FellowStrings};
pub use fellowship::{FellowshipData, FellowshipRequest, FellowshipSource, FellowshipUi};
pub use hud::{
    build_gameplay_hud, load_layout_by_enum, load_ui_images, GameplayHud, GAMEPLAY_ROOTS,
    LAYOUT_GAMEPLAY, LAYOUT_STATUSBAR, LAYOUT_TOOLBAR, UILAYOUT_GROUP,
};
pub use input::{
    is_ui_action, meta_mode_from_key, InputEvent, InputManager, RawAction, ACTION_POINTER_X,
    ACTION_POINTER_Y, DEVICE_KEYBOARD, DEVICE_MOUSE,
};
pub use interpreter::{
    ActionInfo, ActionInterpreter, Dispatch, CLASS_BEHAVIOR, CLASS_CAMERA, CLASS_MODIFIER,
    CLASS_MOVEMENT, CLASS_UI, TOGGLE_HOLD, TOGGLE_ONE_SHOT, TOGGLE_TOGGLE,
};
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
pub use mouse::{
    is_mouse_button, UiMessage, GLOBAL_KEY_PRESS, GLOBAL_KEY_RELEASE, GLOBAL_MOUSE_DOWN,
    GLOBAL_MOUSE_MOVE, GLOBAL_MOUSE_UP, UI_POINTER_X, UI_POINTER_Y, UI_WHEEL_IN, UI_WHEEL_OUT,
};
pub use reticle::{Reticle, ReticleOutcome, ReticleTarget};
pub use scrollbar::MSG_NEW_POSITION;
pub use selection_ui::{SelectionClick, SelectionOutcome, SelectionUi};
pub use shortcut::{
    Shortcut, ShortcutOptions, ShortcutRequest, ShortcutSource, ShortcutUi, SHORTCUT_ALIAS,
    SHORTCUT_EMPTY, SHORTCUT_ITEM, SHORTCUT_RECIPE, SHORTCUT_SKILL,
};
pub use skill_tree::{
    SkillNode, SkillPanel, SkillRequest, SkillSource, SkillStatus, SkillTreeUi, SkillVar,
    EXAMINE_PANEL, EXAMINE_SKILL, NODE_SKILL, ROOT_MAGIC_PANEL, ROOT_MELEE_PANEL,
    ROOT_MISSILE_PANEL, ROOT_NONCOMBAT_PANEL,
};
pub use text::UiTextData;
pub use text_data::{StoreTextData, DEFAULT_FONT, FONT_GROUP};
pub use token::{tokenize, REFERENCE, REFERENT, STATELESS};
pub use toolbar::{ToolbarChild, ToolbarUi};
pub use ui_string::{UiString, UiStrings, UI_TABLE};
pub use vitals::{
    VitalsUi, VITAL_EXPERIENCE, VITAL_FOCUS, VITAL_HEALTH, VITAL_PK_DAMAGE, VITAL_PK_VIGOR_LOSS,
    VITAL_VIGOR,
};

#[cfg(test)]
mod allegiance_tests;
#[cfg(test)]
mod chat_tests;
#[cfg(test)]
mod dialog_tests;
#[cfg(test)]
mod element_tests;
#[cfg(test)]
mod fellowship_tests;
#[cfg(test)]
mod list_box_select_tests;
#[cfg(test)]
mod list_box_tests;
#[cfg(test)]
mod media_tests;
#[cfg(test)]
mod meter_tests;
#[cfg(test)]
mod mouse_tests;
#[cfg(test)]
mod real_data_tests;
#[cfg(test)]
mod scrollbar_tests;
#[cfg(test)]
mod shortcut_tests;
#[cfg(test)]
mod skill_tree_tests;
#[cfg(test)]
mod text_tests;
#[cfg(test)]
mod tree_tests;
#[cfg(test)]
mod vitals_tests;
