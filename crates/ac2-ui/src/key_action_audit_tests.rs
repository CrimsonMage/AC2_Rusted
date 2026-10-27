//! The key actions the DAT maps bind, each traced to its Legions 7160 handler and to ours.
//!
//! The maps are `KEYMAP_MAIN`, `KEYMAP_SHIFT` and `KEYMAP_CONTROL` (`gmClient::InitKeymap`) and
//! the preproc map (`CInputManager_WIN32::Init`, enum `0x40000001`). Status: `handled` (ours
//! already did it), `wired` (this audit added it), `gap` (retail does it, ours does not yet) and
//! `skip` (the Legions client has no handler for the key: an admin-only feature, so nothing is
//! done, as retail does nothing).
//!
//! Retail handlers are cited by class and function. Global key presses reach the listeners as
//! `0xffff0003` messages carrying the action (`GLOBAL_KEY_PRESS`), and releases as `0xffff0004`.

use ac2_db::ResourceStore;

use super::maps::{load_action_map, load_key_map, KEYMAP_CONTROL, KEYMAP_MAIN, KEYMAP_SHIFT};

const KEYMAP_PREPROC: u32 = 0x4000_0001;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Status {
    Handled,
    Wired,
    Gap,
    Skip,
}

/// `(action, binding name, retail handler, our handler, status)`.
const AUDIT: &[(u32, &str, &str, &str, Status)] = &[
    // Pointer, mouse buttons, modifiers and text keys (preproc).
    (
        0x4000_0001,
        "PointerX",
        "CInputManager_WIN32::IsUIAction (pointer axis)",
        "input.rs pointer axes; mouse.rs",
        Status::Handled,
    ),
    (
        0x4000_0002,
        "PointerY",
        "CInputManager_WIN32::IsUIAction (pointer axis)",
        "input.rs pointer axes; mouse.rs",
        Status::Handled,
    ),
    (
        0x4000_0003,
        "Scroll",
        "CInputManager_WIN32::IsUIAction (wheel)",
        "input.rs; controls.rs ui_messages (world wheel zoom)",
        Status::Handled,
    ),
    (
        0x4000_0004,
        "SelectLeft",
        "UIElementManager::KeyPressEvent (mouse button)",
        "mouse.rs",
        Status::Handled,
    ),
    (
        0x4000_0005,
        "SelectRight",
        "UIElementManager::KeyPressEvent (mouse button)",
        "mouse.rs",
        Status::Handled,
    ),
    (
        0x4000_000D,
        "LEFT_CTRL_KEY",
        "ActionInterpreter::CtrlKeyDown (modifier 2)",
        "interpreter.rs modifier_bitfield; input.rs ctrl_down",
        Status::Handled,
    ),
    (
        0x4000_000E,
        "LEFT_SHIFT_KEY",
        "ActionInterpreter::ShiftKeyDown (modifier 1)",
        "interpreter.rs modifier_bitfield; input.rs shift_down",
        Status::Handled,
    ),
    (
        0x4000_0010,
        "LEFT_ALT_KEY",
        "GetModifierBitfield (modifier 4)",
        "interpreter.rs modifier_bitfield",
        Status::Handled,
    ),
    (
        0x4000_0011,
        "RIGHT_ALT_KEY",
        "no action value (class 0)",
        "none needed",
        Status::Handled,
    ),
    (
        0x4000_0012,
        "RIGHT_CTRL_KEY",
        "no action value (class 0)",
        "none needed",
        Status::Handled,
    ),
    (
        0x4000_0013,
        "LEFT_KEY",
        "gmCameraUI::ListenToMessage (turn left)",
        "camera_ui.rs (resends the turn action)",
        Status::Handled,
    ),
    (
        0x4000_0014,
        "RIGHT_KEY",
        "gmCameraUI::ListenToMessage (turn right)",
        "camera_ui.rs",
        Status::Handled,
    ),
    (
        0x4000_0015,
        "UP_KEY",
        "gmCameraUI::ListenToMessage (forward); ChatWindow::KeyPress (history)",
        "camera_ui.rs; chat.rs",
        Status::Handled,
    ),
    (
        0x4000_0016,
        "DOWN_KEY",
        "gmCameraUI::ListenToMessage (backward); ChatWindow::KeyPress (history)",
        "camera_ui.rs; chat.rs",
        Status::Handled,
    ),
    (
        0x4000_0018,
        "USE_ACTIVATION_KEY",
        "gmChatUI::DispatchKeypressToWindow",
        "chat.rs ENTER",
        Status::Handled,
    ),
    (
        0x4000_0019,
        "PAGEUP_KEY",
        "ChatWindow::KeyPress (page)",
        "chat.rs PAGE_UP",
        Status::Handled,
    ),
    (
        0x4000_001A,
        "PAGEDOWN_KEY",
        "ChatWindow::KeyPress (page)",
        "chat.rs PAGE_DOWN",
        Status::Handled,
    ),
    (
        0x4000_001B,
        "HOME_KEY",
        "ChatWindow::KeyPress; UIElement_Text",
        "chat.rs HOME; text_edit.rs",
        Status::Handled,
    ),
    (
        0x4000_001C,
        "END_KEY",
        "ChatWindow::KeyPress; UIElement_Text",
        "chat.rs END; text_edit.rs",
        Status::Handled,
    ),
    (
        0x4000_001D,
        "DELETE_KEY",
        "UIElement_Text::CharacterHandler (no action value)",
        "text_edit.rs",
        Status::Handled,
    ),
    (
        0x4000_001E,
        "ESC_KEY",
        "gmStoryUI::ListenToMessage; gmSelectionUI::ListenToMessage",
        "hud_story.rs; target_cycle.rs; chat.rs ESCAPE",
        Status::Handled,
    ),
    (
        0x4000_001F,
        "BS_KEY",
        "UIElement_Text::CharacterHandler (no action value)",
        "text_edit.rs",
        Status::Handled,
    ),
    (
        0x4000_002D,
        "CameraInstantZoomIn",
        "gmCameraUI (UI 0x400000B8 wheel in) -> CameraManager::ZoomIn",
        "camera_ui.rs UI_WHEEL_IN -> CAMERA_ZOOM_IN",
        Status::Handled,
    ),
    (
        0x4000_00B2,
        "CameraInstantZoomOut",
        "gmCameraUI (UI 0x400000B9 wheel out) -> CameraManager::ZoomOut",
        "camera_ui.rs UI_WHEEL_OUT -> CAMERA_ZOOM_OUT",
        Status::Handled,
    ),
    // Movement (PlayerMovementModule::UseTime over the held actions).
    (
        0x4000_0020,
        "Forward_KEY",
        "PlayerMovementModule::UseTime (forward)",
        "ac2-weenie movement_module.rs ACTION_FORWARD",
        Status::Handled,
    ),
    (
        0x4000_0021,
        "Backward_KEY",
        "PlayerMovementModule::UseTime (forward, extent -1)",
        "movement_module.rs ACTION_FORWARD",
        Status::Handled,
    ),
    (
        0x4000_0022,
        "Strafe_Left_KEY",
        "PlayerMovementModule::UseTime (strafe negative)",
        "movement_module.rs ACTION_STRAFE_NEGATIVE",
        Status::Handled,
    ),
    (
        0x4000_0023,
        "Strafe_Right_KEY",
        "PlayerMovementModule::UseTime (strafe positive)",
        "movement_module.rs ACTION_STRAFE_POSITIVE",
        Status::Handled,
    ),
    (
        0x4000_0024,
        "Turn_Left_KEY",
        "PlayerMovementModule::UseTime (turn positive)",
        "movement_module.rs ACTION_TURN_POSITIVE",
        Status::Handled,
    ),
    (
        0x4000_0025,
        "Turn_Right_KEY",
        "PlayerMovementModule::UseTime (turn negative)",
        "movement_module.rs ACTION_TURN_NEGATIVE",
        Status::Handled,
    ),
    (
        0x4000_0026,
        "Jump_KEY",
        "PlayerMovementModule::UseTime (jump, one-shot)",
        "movement_module.rs ACTION_JUMP",
        Status::Handled,
    ),
    (
        0x4000_0027,
        "RunLock_KEY",
        "ActionInterpreter (run lock toggle)",
        "interpreter.rs TURN_OFF run lock",
        Status::Handled,
    ),
    // Camera (CameraManager::StartAction and StopAction, Legions 0x00546FF0 and 0x00547250).
    (
        0x4000_0028,
        "CameraSwingOutLeft",
        "CameraManager::StartAction -> FakeTransition",
        "camera.rs fake_transition (camera's own turn)",
        Status::Wired,
    ),
    (
        0x4000_0029,
        "CameraSwingOutRight",
        "CameraManager::StartAction -> SwingOutLeft (Legions)",
        "camera.rs SWING_OUT_RIGHT -> swing_out_left",
        Status::Wired,
    ),
    (
        0x4000_002A,
        "CameraShoulderView",
        "CameraManager::ShoulderView",
        "camera.rs SHOULDER_VIEW",
        Status::Handled,
    ),
    (
        0x4000_002E,
        "CameraRotateLeft",
        "CameraManager::StartAction (heading)",
        "camera.rs ROTATE_LEFT",
        Status::Handled,
    ),
    (
        0x4000_002F,
        "CameraRotateRight",
        "CameraManager::StartAction (heading)",
        "camera.rs ROTATE_RIGHT",
        Status::Handled,
    ),
    (
        0x4000_0030,
        "CameraReset",
        "CameraManager::SetDefaults",
        "camera.rs RESET",
        Status::Handled,
    ),
    (
        0x4000_00AE,
        "CameraLookAtGround",
        "CameraManager::LookAtGround",
        "camera.rs LOOK_AT_GROUND",
        Status::Handled,
    ),
    (
        0x4000_00B1,
        "CameraLookAtSelf",
        "CameraManager::LookAtSelf",
        "camera.rs LOOK_AT_SELF",
        Status::Wired,
    ),
    (
        0x4000_00B3,
        "CameraMapView",
        "CameraManager::SetMapView",
        "camera.rs MAP_VIEW (SceneTool degrade switch not modelled)",
        Status::Wired,
    ),
    (
        0x4000_00B4,
        "CameraZoomIn",
        "CameraManager::StartAction (scale 0.75^(extent*speed))",
        "camera.rs ZOOM_SCALE_IN",
        Status::Wired,
    ),
    (
        0x4000_00B5,
        "CameraZoomOut",
        "CameraManager::StartAction (scale 1.5^(extent*speed))",
        "camera.rs ZOOM_SCALE_OUT",
        Status::Wired,
    ),
    (
        0x4000_00B6,
        "CameraTiltUp",
        "CameraManager::StartAction (pitch)",
        "camera.rs PITCH_UP",
        Status::Handled,
    ),
    (
        0x4000_00B7,
        "CameraTiltDown",
        "CameraManager::StartAction (pitch)",
        "camera.rs PITCH_DOWN",
        Status::Handled,
    ),
    (
        0x4000_00A2,
        "CameraFirstPerson",
        "gmCameraUI::ListenToMessage -> SetFirstPersonState",
        "camera_ui.rs UI_FIRST_PERSON",
        Status::Handled,
    ),
    (
        0x4000_00BA,
        "CameraToggleMouseLook",
        "gmCameraUI::ListenToMessage -> SetMouseLookState",
        "camera_ui.rs UI_MOUSE_LOOK_TOGGLE",
        Status::Handled,
    ),
    // Gameplay HUD (gmGamePlayUI::ListenToMessage, gmToolbarUI, gmShortcutUI and the helpers).
    (
        0x4100_0001,
        "EXAMINATION_PANEL_KEY",
        "gmToolbarUI::ListenToMessage (child toggle)",
        "toolbar.rs ToolbarChild::key",
        Status::Handled,
    ),
    (
        0x4100_0002,
        "INVENTORY_PANEL_KEY",
        "gmToolbarUI::ListenToMessage (child toggle)",
        "toolbar.rs ToolbarChild::key",
        Status::Handled,
    ),
    (
        0x4100_0003,
        "NAVIGATION_PANEL_KEY",
        "gmToolbarUI::ListenToMessage (child toggle)",
        "toolbar.rs ToolbarChild::key",
        Status::Handled,
    ),
    (
        0x4100_0004,
        "OPTION_PANEL_KEY",
        "gmToolbarUI::ListenToMessage (child toggle)",
        "toolbar.rs ToolbarChild::key",
        Status::Handled,
    ),
    (
        0x4100_0005,
        "QUEST_PANEL_KEY",
        "gmToolbarUI::ListenToMessage (child toggle)",
        "toolbar.rs ToolbarChild::key",
        Status::Handled,
    ),
    (
        0x4100_0006,
        "SKILL_PANEL_KEY",
        "gmToolbarUI::ListenToMessage (child toggle)",
        "toolbar.rs ToolbarChild::key",
        Status::Handled,
    ),
    (
        0x4100_0007,
        "SOCIAL_PANEL_KEY",
        "gmToolbarUI::ListenToMessage (child toggle)",
        "toolbar.rs ToolbarChild::key",
        Status::Handled,
    ),
    (
        0x4100_0008,
        "TOOLBAR_PANEL_KEY",
        "gmGamePlayUI::ListenToMessage -> ChildKeyPress",
        "gameplay_keys.rs; hud_view.rs gameplay_keys",
        Status::Handled,
    ),
    (
        0x4100_0009,
        "STATUSBAR_PANEL_KEY",
        "gmGamePlayUI::ListenToMessage -> ChildKeyPress",
        "gameplay_keys.rs; hud_view.rs gameplay_keys",
        Status::Handled,
    ),
    (
        0x4100_000B,
        "RADAR_KEY",
        "gmGamePlayUI::ListenToMessage -> ChildKeyPress",
        "gameplay_keys.rs; hud_view.rs gameplay_keys",
        Status::Handled,
    ),
    (
        0x4100_000C,
        "ALL_UI_KEY",
        "gmGamePlayUI::ListenToMessage -> Show toggle",
        "hud_view.rs gameplay_show_key",
        Status::Handled,
    ),
    (
        0x4100_000D,
        "USE_KEY",
        "gmGamePlayUI::ListenToMessage -> gmCWInterface::UseSelection",
        "hud_view.rs take_use_key -> view.rs Command::UseSelection",
        Status::Wired,
    ),
    (
        0x4100_000E,
        "LOGOUT_KEY",
        "gmGamePlayUI::ListenToMessage (_doEndSession)",
        "hud_game_dialogs.rs",
        Status::Handled,
    ),
    (
        0x4100_000F,
        "SHOW_NAMES_KEY",
        "gm3DTextUI::ListenToMessage (name plates)",
        "name_plates.rs",
        Status::Handled,
    ),
    (
        0x4100_0010,
        "START_COMMAND_KEY",
        "ChatWindow::HandleCommandOrAliasKey",
        "chat.rs COMMAND_KEY",
        Status::Handled,
    ),
    (
        0x4100_0011,
        "START_ALIAS_KEY",
        "ChatWindow::HandleCommandOrAliasKey",
        "chat.rs ALIAS_KEY",
        Status::Handled,
    ),
    (
        0x4100_0013,
        "AUTOATTACK_KEY",
        "gmShortcutUI::ListenToMessage (RequestAttackStateChange)",
        "shortcut.rs",
        Status::Handled,
    ),
    (
        0x4100_0014,
        "VISUALIZE_SOUND_KEY",
        "no handler in Legions 7160 (admin gmUIFlow only)",
        "none (admin-only)",
        Status::Skip,
    ),
    (
        0x4100_0015,
        "BENCHMARKING_KEY",
        "no handler in Legions 7160 (admin only)",
        "none (admin-only)",
        Status::Skip,
    ),
    (
        0x4100_0016,
        "DEBUG_HUD_KEY",
        "no handler in Legions 7160 (admin SceneTool::ShowDebugHUD only)",
        "none (admin-only)",
        Status::Skip,
    ),
    (
        0x4100_0017,
        "SCREENSHOT_KEY",
        "gmUIFlow::ListenToMessage (release) -> Device::SaveScreenshot",
        "none yet: needs a surface read-back in the window path",
        Status::Gap,
    ),
    (
        0x4100_0018,
        "SELECTION_SELF_KEY",
        "gmSelectionUI (select self)",
        "selection_notices.rs RESELECT_KEY; target_cycle.rs",
        Status::Handled,
    ),
    (
        0x4100_0019,
        "SELECTION_PREVIOUS_CREATURE_KEY",
        "gmSelectionUI::Select (TargetTypeVsKey)",
        "target_cycle.rs",
        Status::Handled,
    ),
    (
        0x4100_001A,
        "SELECTION_NEXT_CREATURE_KEY",
        "gmSelectionUI::Select",
        "target_cycle.rs",
        Status::Handled,
    ),
    (
        0x4100_001B,
        "SELECTION_NEAREST_CREATURE_KEY",
        "gmSelectionUI::Select",
        "target_cycle.rs",
        Status::Handled,
    ),
    (
        0x4100_001C,
        "SELECTION_PREVIOUS_ITEM_KEY",
        "gmSelectionUI::Select",
        "target_cycle.rs",
        Status::Handled,
    ),
    (
        0x4100_001D,
        "SELECTION_NEXT_ITEM_KEY",
        "gmSelectionUI::Select",
        "target_cycle.rs",
        Status::Handled,
    ),
    (
        0x4100_001E,
        "SELECTION_NEAREST_ITEM_KEY",
        "gmSelectionUI::Select",
        "target_cycle.rs",
        Status::Handled,
    ),
    (
        0x4100_001F,
        "SELECTION_PREVIOUS_PC_KEY",
        "gmSelectionUI::Select",
        "target_cycle.rs",
        Status::Handled,
    ),
    (
        0x4100_0020,
        "SELECTION_NEXT_PC_KEY",
        "gmSelectionUI::Select",
        "target_cycle.rs",
        Status::Handled,
    ),
    (
        0x4100_0021,
        "SELECTION_NEAREST_PC_KEY",
        "gmSelectionUI::Select",
        "target_cycle.rs",
        Status::Handled,
    ),
    (
        0x4100_0022,
        "SELECTION_PREVIOUS_FOE_KEY",
        "gmSelectionUI::Select (foes)",
        "target_cycle.rs",
        Status::Handled,
    ),
    (
        0x4100_0023,
        "SELECTION_NEXT_FOE_KEY",
        "gmSelectionUI::Select (foes)",
        "target_cycle.rs",
        Status::Handled,
    ),
    (
        0x4100_0024,
        "SELECTION_NEAREST_FOE_KEY",
        "gmSelectionUI::Select (foes; auto-target)",
        "target_cycle.rs; target_select.rs",
        Status::Handled,
    ),
    (
        0x4100_0025,
        "SELECTION_PREVIOUS_FELLOW_KEY",
        "gmSelectionUI::Select (fellows)",
        "target_cycle.rs",
        Status::Handled,
    ),
    (
        0x4100_0026,
        "SELECTION_NEXT_FELLOW_KEY",
        "gmSelectionUI::Select (fellows)",
        "target_cycle.rs",
        Status::Handled,
    ),
    (
        0x4100_0027,
        "SELECTION_NEAREST_FELLOW_KEY",
        "gmSelectionUI::Select (fellows)",
        "target_cycle.rs",
        Status::Handled,
    ),
    (
        0x4100_0028,
        "SELECTION_LAST_KEY",
        "gmSelectionUI::ListenToMessage (last selection)",
        "selection_notices.rs LAST_SELECTION_KEY",
        Status::Handled,
    ),
    (
        0x4100_002A,
        "SELECTION_LAST_ATTACKER_KEY",
        "gmSelectionUI::ListenToMessage (last attacker)",
        "selection_notices.rs LAST_ATTACKER_KEY",
        Status::Handled,
    ),
    (
        0x4100_002B,
        "SHORTCUT_1_KEY",
        "gmShortcutUI::ListenToMessage (use slot)",
        "shortcut.rs key_press",
        Status::Handled,
    ),
    (
        0x4100_002C,
        "SHORTCUT_2_KEY",
        "gmShortcutUI::ListenToMessage (use slot)",
        "shortcut.rs key_press (range)",
        Status::Handled,
    ),
    (
        0x4100_002D,
        "SHORTCUT_3_KEY",
        "gmShortcutUI::ListenToMessage (use slot)",
        "shortcut.rs key_press (range)",
        Status::Handled,
    ),
    (
        0x4100_002E,
        "SHORTCUT_4_KEY",
        "gmShortcutUI::ListenToMessage (use slot)",
        "shortcut.rs key_press (range)",
        Status::Handled,
    ),
    (
        0x4100_002F,
        "SHORTCUT_5_KEY",
        "gmShortcutUI::ListenToMessage (use slot)",
        "shortcut.rs key_press (range)",
        Status::Handled,
    ),
    (
        0x4100_0030,
        "SHORTCUT_6_KEY",
        "gmShortcutUI::ListenToMessage (use slot)",
        "shortcut.rs key_press (range)",
        Status::Handled,
    ),
    (
        0x4100_0031,
        "SHORTCUT_7_KEY",
        "gmShortcutUI::ListenToMessage (use slot)",
        "shortcut.rs key_press (range)",
        Status::Handled,
    ),
    (
        0x4100_0032,
        "SHORTCUT_8_KEY",
        "gmShortcutUI::ListenToMessage (use slot)",
        "shortcut.rs key_press (range)",
        Status::Handled,
    ),
    (
        0x4100_0033,
        "SHORTCUT_9_KEY",
        "gmShortcutUI::ListenToMessage (use slot)",
        "shortcut.rs key_press (range)",
        Status::Handled,
    ),
    (
        0x4100_0034,
        "SHORTCUT_0_KEY",
        "gmShortcutUI::ListenToMessage (use slot)",
        "shortcut.rs key_press (range)",
        Status::Handled,
    ),
    (
        0x4100_0035,
        "SHORTCUT_SET_1_KEY",
        "gmShortcutUI::ListenToMessage (set)",
        "shortcut.rs show_set (range)",
        Status::Handled,
    ),
    (
        0x4100_003E,
        "SHORTCUT_SET_0_KEY",
        "gmShortcutUI::ListenToMessage (set)",
        "shortcut.rs show_set (range)",
        Status::Handled,
    ),
    (
        0x4100_003F,
        "SHORTCUT_SET_NEXT_KEY",
        "gmShortcutUI::ListenToMessage (set)",
        "shortcut.rs key_press",
        Status::Handled,
    ),
    (
        0x4100_0040,
        "SHORTCUT_SET_PREV_KEY",
        "gmShortcutUI::ListenToMessage (set)",
        "shortcut.rs key_press",
        Status::Handled,
    ),
    (
        0x4100_0042,
        "CHATWINDOW_2_KEY",
        "gmChatUI::ListenToMessage (window toggle)",
        "chat.rs WINDOW_KEYS",
        Status::Handled,
    ),
    (
        0x4100_0043,
        "CHATWINDOW_3_KEY",
        "gmChatUI::ListenToMessage (window toggle)",
        "chat.rs WINDOW_KEYS",
        Status::Handled,
    ),
    (
        0x4100_0044,
        "CHATWINDOW_4_KEY",
        "gmChatUI::ListenToMessage (window toggle)",
        "chat.rs WINDOW_KEYS",
        Status::Handled,
    ),
    (
        0x4100_0045,
        "PRINT_ACTIVE_SOUND_KEY",
        "no handler in Legions 7160 (admin only)",
        "none (admin-only)",
        Status::Skip,
    ),
    (
        0x4100_0046,
        "GROOVE_LEVEL_UP_KEY",
        "no handler in Legions 7160 (admin SoundManager::TweakGrooveLevel only)",
        "none (admin-only)",
        Status::Skip,
    ),
    (
        0x4100_0047,
        "GROOVE_LEVEL_DOWN_KEY",
        "no handler in Legions 7160 (admin only)",
        "none (admin-only)",
        Status::Skip,
    ),
    (
        0x4100_0049,
        "REPLY_KEY",
        "ChatWindow::HandleReplyKey",
        "chat.rs REPLY_KEY",
        Status::Handled,
    ),
    (
        0x4100_0053,
        "SELECTION_FELLOW_1_KEY",
        "FellowshipHelper::ListenToMessage (SelectFellowByIdx)",
        "fellowship.rs",
        Status::Handled,
    ),
    (
        0x4100_005B,
        "SELECTION_FELLOW_9_KEY",
        "FellowshipHelper::ListenToMessage (SelectFellowByIdx)",
        "fellowship.rs (range)",
        Status::Handled,
    ),
    (
        0x4100_005C,
        "SELECTION_PREVIOUS_PET_KEY",
        "gmSelectionUI::Select (pets)",
        "target_cycle.rs",
        Status::Handled,
    ),
    (
        0x4100_005D,
        "SELECTION_NEXT_PET_KEY",
        "gmSelectionUI::Select (pets)",
        "target_cycle.rs",
        Status::Handled,
    ),
    (
        0x4100_005F,
        "SHORTCUT_B_1_KEY",
        "gmShortcutUI::ListenToMessage (use slot, second bar)",
        "shortcut.rs key_press (range)",
        Status::Handled,
    ),
    (
        0x4100_0068,
        "SHORTCUT_B_0_KEY",
        "gmShortcutUI::ListenToMessage (use slot, second bar)",
        "shortcut.rs key_press (range)",
        Status::Handled,
    ),
    (
        0x4100_006A,
        "SELECTION_NEXT_EXAM_TAB_KEY",
        "no handler in Legions 7160 (admin gmExaminationUI only)",
        "none (admin-only)",
        Status::Skip,
    ),
    (
        0x4100_006B,
        "SELECTION_PREV_EXAM_TAB_KEY",
        "no handler in Legions 7160 (admin gmExaminationUI only)",
        "none (admin-only)",
        Status::Skip,
    ),
    (
        0x4100_0036,
        "SHORTCUT_SET_2_KEY",
        "gmShortcutUI::ListenToMessage (set)",
        "shortcut.rs show_set (range)",
        Status::Handled,
    ),
    (
        0x4100_0037,
        "SHORTCUT_SET_3_KEY",
        "gmShortcutUI::ListenToMessage (set)",
        "shortcut.rs show_set (range)",
        Status::Handled,
    ),
    (
        0x4100_0038,
        "SHORTCUT_SET_4_KEY",
        "gmShortcutUI::ListenToMessage (set)",
        "shortcut.rs show_set (range)",
        Status::Handled,
    ),
    (
        0x4100_0039,
        "SHORTCUT_SET_5_KEY",
        "gmShortcutUI::ListenToMessage (set)",
        "shortcut.rs show_set (range)",
        Status::Handled,
    ),
    (
        0x4100_003a,
        "SHORTCUT_SET_6_KEY",
        "gmShortcutUI::ListenToMessage (set)",
        "shortcut.rs show_set (range)",
        Status::Handled,
    ),
    (
        0x4100_003b,
        "SHORTCUT_SET_7_KEY",
        "gmShortcutUI::ListenToMessage (set)",
        "shortcut.rs show_set (range)",
        Status::Handled,
    ),
    (
        0x4100_003c,
        "SHORTCUT_SET_8_KEY",
        "gmShortcutUI::ListenToMessage (set)",
        "shortcut.rs show_set (range)",
        Status::Handled,
    ),
    (
        0x4100_003d,
        "SHORTCUT_SET_9_KEY",
        "gmShortcutUI::ListenToMessage (set)",
        "shortcut.rs show_set (range)",
        Status::Handled,
    ),
    (
        0x4100_0054,
        "SELECTION_FELLOW_2_KEY",
        "FellowshipHelper::ListenToMessage (SelectFellowByIdx)",
        "fellowship.rs (range)",
        Status::Handled,
    ),
    (
        0x4100_0055,
        "SELECTION_FELLOW_3_KEY",
        "FellowshipHelper::ListenToMessage (SelectFellowByIdx)",
        "fellowship.rs (range)",
        Status::Handled,
    ),
    (
        0x4100_0056,
        "SELECTION_FELLOW_4_KEY",
        "FellowshipHelper::ListenToMessage (SelectFellowByIdx)",
        "fellowship.rs (range)",
        Status::Handled,
    ),
    (
        0x4100_0057,
        "SELECTION_FELLOW_5_KEY",
        "FellowshipHelper::ListenToMessage (SelectFellowByIdx)",
        "fellowship.rs (range)",
        Status::Handled,
    ),
    (
        0x4100_0058,
        "SELECTION_FELLOW_6_KEY",
        "FellowshipHelper::ListenToMessage (SelectFellowByIdx)",
        "fellowship.rs (range)",
        Status::Handled,
    ),
    (
        0x4100_0059,
        "SELECTION_FELLOW_7_KEY",
        "FellowshipHelper::ListenToMessage (SelectFellowByIdx)",
        "fellowship.rs (range)",
        Status::Handled,
    ),
    (
        0x4100_005a,
        "SELECTION_FELLOW_8_KEY",
        "FellowshipHelper::ListenToMessage (SelectFellowByIdx)",
        "fellowship.rs (range)",
        Status::Handled,
    ),
    (
        0x4100_0060,
        "SHORTCUT_B_2_KEY",
        "gmShortcutUI::ListenToMessage (use slot, second bar)",
        "shortcut.rs key_press (range)",
        Status::Handled,
    ),
    (
        0x4100_0061,
        "SHORTCUT_B_3_KEY",
        "gmShortcutUI::ListenToMessage (use slot, second bar)",
        "shortcut.rs key_press (range)",
        Status::Handled,
    ),
    (
        0x4100_0062,
        "SHORTCUT_B_4_KEY",
        "gmShortcutUI::ListenToMessage (use slot, second bar)",
        "shortcut.rs key_press (range)",
        Status::Handled,
    ),
    (
        0x4100_0063,
        "SHORTCUT_B_5_KEY",
        "gmShortcutUI::ListenToMessage (use slot, second bar)",
        "shortcut.rs key_press (range)",
        Status::Handled,
    ),
    (
        0x4100_0064,
        "SHORTCUT_B_6_KEY",
        "gmShortcutUI::ListenToMessage (use slot, second bar)",
        "shortcut.rs key_press (range)",
        Status::Handled,
    ),
    (
        0x4100_0065,
        "SHORTCUT_B_7_KEY",
        "gmShortcutUI::ListenToMessage (use slot, second bar)",
        "shortcut.rs key_press (range)",
        Status::Handled,
    ),
    (
        0x4100_0066,
        "SHORTCUT_B_8_KEY",
        "gmShortcutUI::ListenToMessage (use slot, second bar)",
        "shortcut.rs key_press (range)",
        Status::Handled,
    ),
    (
        0x4100_0067,
        "SHORTCUT_B_9_KEY",
        "gmShortcutUI::ListenToMessage (use slot, second bar)",
        "shortcut.rs key_press (range)",
        Status::Handled,
    ),
    (
        0x4000_00BB,
        "KEYSTONE_PANEL_KEY",
        "gmToolbarUI::ListenToMessage -> ShowKeyStone (KeyStone help)",
        "none yet: KeyStone help is not ported",
        Status::Gap,
    ),
];

/// The three keys the input manager loads, with the preproc map.
fn bound_actions(store: &ResourceStore) -> Vec<u32> {
    let mut actions = Vec::new();
    for enum_value in [KEYMAP_PREPROC, KEYMAP_MAIN, KEYMAP_SHIFT, KEYMAP_CONTROL] {
        let map = load_key_map(store, enum_value)
            .unwrap()
            .unwrap_or_else(|| panic!("key map {enum_value:#x}"));
        actions.extend(map.mappings.iter().map(|m| m.action));
    }
    actions.sort_unstable();
    actions.dedup();
    actions
}

#[test]
fn every_bound_action_has_an_audit_row() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ResourceStore::new(dataset);
    assert!(load_action_map(&store).unwrap().is_some());
    let bound = bound_actions(&store);
    for action in &bound {
        assert!(
            AUDIT.iter().any(|row| row.0 == *action),
            "no audit row for {action:#010x}"
        );
    }
    for row in AUDIT {
        assert!(
            bound.contains(&row.0),
            "audit row {} ({:#010x}) is not bound by any map",
            row.1,
            row.0
        );
    }
}

#[test]
fn every_gap_and_skip_row_says_why() {
    for (action, name, retail, ours, status) in AUDIT {
        assert!(
            !retail.is_empty() && !ours.is_empty(),
            "{name} {action:#010x}"
        );
        if matches!(status, Status::Gap | Status::Skip) {
            assert!(!ours.is_empty(), "{name}");
        }
    }
    let skips = AUDIT.iter().filter(|r| r.4 == Status::Skip).count();
    let gaps = AUDIT.iter().filter(|r| r.4 == Status::Gap).count();
    assert_eq!(gaps, 2, "screenshot and KeyStone");
    assert_eq!(
        skips, 8,
        "admin-only keys: sound, benchmark, debug HUD, groove, exam tabs"
    );
}
