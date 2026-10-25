#![forbid(unsafe_code)]

//! Asheron's Call 2 input and UI: the key maps and action map from the DATs, the input manager
//! that turns key and mouse events into actions (`CInputManager_WIN32`), and the action
//! interpreter that turns actions into the movement, behaviour and camera input the client core
//! consumes (`ActionInterpreter`); and the UI: layouts (`LayoutDesc`), the element tree built
//! from them (`UIElementManager`, `UIElement`) and its software compositor (`UIRegion::DrawHere`).

mod button;
mod camera_ui;
mod compose;
mod controls;
mod element;
mod hud;
mod input;
mod interpreter;
mod layout;
mod layout_media;
mod maps;
mod mouse;
mod place;
#[cfg(test)]
mod tests;
mod text;
mod text_attr;
mod text_data;
mod token;

pub use camera_ui::{CameraUi, CameraUiEffect};
pub use compose::{UiPixels, UiSurface};
pub use controls::{ControlFrame, Controls};
pub use element::{UiBox, UiElement, UiImage, UiTree};
pub use hud::{
    build_gameplay_hud, load_layout_by_enum, load_ui_images, GAMEPLAY_ROOTS, LAYOUT_GAMEPLAY,
    LAYOUT_STATUSBAR, LAYOUT_TOOLBAR, UILAYOUT_GROUP,
};
pub use input::{
    meta_mode_from_key, InputEvent, InputManager, RawAction, ACTION_POINTER_X, ACTION_POINTER_Y,
    DEVICE_KEYBOARD, DEVICE_MOUSE,
};
pub use interpreter::{
    ActionInfo, ActionInterpreter, Dispatch, CLASS_BEHAVIOR, CLASS_CAMERA, CLASS_MODIFIER,
    CLASS_MOVEMENT, CLASS_UI, TOGGLE_HOLD, TOGGLE_ONE_SHOT, TOGGLE_TOGGLE,
};
pub use layout::{
    load_layout, AttributeDesc, AttributeValue, ElementDesc, LayoutDesc, MediaDesc, Rect,
    StateDesc, TransitionData, TransitionDesc, LAYOUT_FIRST, LAYOUT_LAST,
};
pub use maps::{
    load_action_map, load_key_map, ActionMap, ActionMapping, ActionValue, KeyMap, KEYMAP_CONTROL,
    KEYMAP_GROUP, KEYMAP_MAIN, KEYMAP_SHIFT,
};
pub use mouse::{
    is_mouse_button, UiMessage, GLOBAL_KEY_PRESS, GLOBAL_KEY_RELEASE, GLOBAL_MOUSE_DOWN,
    GLOBAL_MOUSE_MOVE, GLOBAL_MOUSE_UP, UI_POINTER_X, UI_POINTER_Y, UI_WHEEL_IN, UI_WHEEL_OUT,
};
pub use text::UiTextData;
pub use text_data::{StoreTextData, DEFAULT_FONT, FONT_GROUP};
pub use token::{tokenize, REFERENCE, REFERENT, STATELESS};

#[cfg(test)]
mod element_tests;
#[cfg(test)]
mod mouse_tests;
#[cfg(test)]
mod real_data_tests;
#[cfg(test)]
mod text_tests;
