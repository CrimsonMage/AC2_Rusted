//! The world frame's input pass, shared by the window program and the tests that drive it the
//! way a player's mouse and keyboard do: the controls' UI actions into the HUD, the text the
//! HUD's active field takes, the chat's requests, the camera UI's messages, the selection's
//! key messages, and the controls' frame as the client's [`FrameInput`]; then, after the
//! frame, the 3D selection's pick (`gmSelectionUI::Finish3DSelection`).

use ac2_ui::{Controls, SelectionUi};

use crate::view::ClientView;
use crate::{frame_input, pick_targets, Client, ClientFrame, Command, FrameInput};

/// What the input pass found beside the frame's input.
pub struct WorldInputFrame {
    /// The controls' frame and the commands queued since the last frame.
    pub input: FrameInput,
    /// `gmUIFlow::ListenToMessage`: the screenshot key was released this frame.
    pub screenshot: bool,
}

/// The input half of a world frame: everything between the window's events and
/// `Client::use_time`. `stamp` and `now` are the input clock (milliseconds, seconds).
pub fn begin_frame(
    controls: &mut Controls,
    selection: &mut SelectionUi,
    commands: &mut Vec<Command>,
    view: &mut ClientView,
    client: &mut Client,
    (stamp, now): (u32, f64),
) -> WorldInputFrame {
    // The UI tree takes the UI actions first; its messages reach the camera UI.
    let ui_input = controls.take_ui_input();
    let modifiers = controls.modifiers();
    let mut messages = view.ui_input(&ui_input, modifiers);
    // `gmUIFlow::ListenToMessage`: the screenshot key's release saves the frame.
    let screenshot = messages.iter().any(|message| {
        message.message == ac2_ui::GLOBAL_KEY_RELEASE
            && message.element.is_none()
            && message.data as u32 == crate::keys::SCREENSHOT_KEY
    });
    let (actions, characters, _) = controls.take_text_input();
    messages.extend(view.text_input(&actions, &characters, modifiers));
    for request in view.take_chat_requests() {
        match request {
            ac2_ui::ChatRequest::Line(line) => commands.push(Command::ChatLine(line)),
            // `ChatWindow::DisplayCmdHistory` / `DisplayRetellHistory` / `DisplayReplyHistory`.
            ac2_ui::ChatRequest::History { history, prev } => {
                let kind = match history {
                    ac2_ui::ChatHistory::Command => ac2_weenie::ChatHistory::Command,
                    ac2_ui::ChatHistory::Retell => ac2_weenie::ChatHistory::Retell,
                    ac2_ui::ChatHistory::Reply => ac2_weenie::ChatHistory::Reply,
                };
                let Some(item) = client.session_mut().chat_history(kind, prev) else {
                    continue;
                };
                if history == ac2_ui::ChatHistory::Command {
                    view.set_chat_entry(&item);
                } else {
                    view.assisted_tell(&item);
                }
            }
        }
    }
    controls.set_text_mode(view.text_mode(), stamp, now);
    controls.set_ime_allowed(view.ime_allowed());
    controls.set_copy_text(view.text_copy_focus());
    controls.set_camera_keys_blocked(view.camera_keys_blocked());
    controls.ui_messages(&messages, now);
    let notices = view.take_mouse_look_notices();
    let notices = notices.into_iter().chain(client.take_mouse_look_notices());
    controls.update_can_mouse_look(notices);
    for message in &messages {
        selection.message(message);
    }
    let mut input = frame_input(controls.frame());
    input.commands.append(commands);
    WorldInputFrame { input, screenshot }
}

/// The selection render after the frame: the object under the pointer, selected (or the
/// selection cleared) by `gmSelectionUI::Finish3DSelection`. Returns whether the examination
/// panel should open.
pub fn finish_selection(
    selection: &mut SelectionUi,
    controls: &mut Controls,
    commands: &mut Vec<Command>,
    view: &ClientView,
    client: &Client,
    frame: &ClientFrame,
    screen: (u32, u32),
) -> bool {
    if !selection.wants_pick() {
        return false;
    }
    let potential = view.pick_list(frame, screen, view.pointer());
    let (targets, player_type, player_id) = pick_targets(client, frame, &potential);
    let Some(outcome) = selection.finish(&targets, player_type, player_id) else {
        return false;
    };
    // Right-hold look: a right press that picked nothing turns the camera while held.
    if outcome.right_click && outcome.select.is_none() {
        controls.start_right_look();
    }
    commands.push(Command::Select(outcome.select));
    commands.push(Command::LookAtSelection(outcome.select));
    if outcome.use_selection {
        commands.push(Command::UseSelection);
    }
    outcome.open_examination
}
