//! The UI's gameplay commands and the selection the selection panel shows.
//!
//! Retail's UI calls the player weenie's IFuns (`gmCWInterface::SetSelection`,
//! `SetExaminationRequest`, `StartExamination`, `StopExamination`, `DoEmote`) and its chat
//! commands (`CWInterface::ProcessText`); their notices come back as
//! [`SessionEvent::Notice`] in the frame's events (`CM_*::SendNotice_*`, which retail's UI
//! classes listen to).

use ac2_net::protocol::{ExaminationRequest, SelectionInfo};
use ac2_weenie::{SessionEvent, NO_SELECTION};

use crate::client::Client;

/// A gameplay command from the UI.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// `gmCWInterface::SetSelection`: select an object, or clear the selection.
    Select(Option<u64>),
    /// `gmCWInterface::SetExaminationRequest`.
    Examine(ExaminationRequest),
    /// `gmCWInterface::StartExamination` (the examination panel opened, with its word).
    StartExamination(u32),
    /// `gmCWInterface::StopExamination` (the panel closed).
    StopExamination,
    /// The say command: text to everyone nearby.
    Say(String),
    /// The tell command: text to the named character.
    Tell { recipient: String, text: String },
    /// `gmCWInterface::DoEmote`, with the emote the emote table named.
    Emote(u32),
    /// `gmCWInterface::RequestGUN_SendToRoom` (Turbine chat): text to a chat room, with the
    /// script's chat flags.
    SendToRoom { room: u32, text: String, flags: u32 },
}

/// The selected object, as `gmSelectionUI` shows it.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectionFrame {
    pub id: u64,
    /// The weenie's name, when its description carries one.
    pub name: Option<String>,
    /// The server's latest `SelectionInfo` for it (health and vigor for a creature).
    pub info: Option<SelectionInfo>,
}

impl Client {
    /// Runs the frame's commands; their notices join the frame's events.
    pub(crate) fn commands(&mut self, commands: &[Command]) -> Vec<SessionEvent> {
        let mut events = Vec::new();
        for command in commands {
            events.extend(match command {
                Command::Select(id) => self.session.select(id.unwrap_or(NO_SELECTION)),
                Command::Examine(request) => self.session.examine(*request),
                Command::StartExamination(context) => self.session.start_examination(*context),
                Command::StopExamination => self.session.stop_examination(),
                Command::Say(text) => {
                    self.session.say(text);
                    Vec::new()
                }
                Command::Tell { recipient, text } => {
                    self.session.tell(recipient, text);
                    Vec::new()
                }
                Command::Emote(emote) => {
                    self.session.emote(*emote);
                    Vec::new()
                }
                Command::SendToRoom { room, text, flags } => {
                    self.session.send_to_room(*room, text, *flags);
                    Vec::new()
                }
            });
        }
        events
    }

    /// The selection for [`crate::ClientFrame::selection`].
    pub(crate) fn selection_frame(&self) -> Option<SelectionFrame> {
        let script = self.session.player_script();
        let id = script.selection()?;
        Some(SelectionFrame {
            id,
            name: self.session.objects().get(id).and_then(|o| o.name.clone()),
            info: script
                .selection_info()
                .filter(|(info_id, _)| *info_id == id)
                .map(|(_, info)| info),
        })
    }
}
