//! The chat command line on the session: the line the chat window entered
//! (`ChatWindow::ProcessCommand` -> `CWInterface::ProcessText`), and the speech, tells and
//! server calls that feed the chat state.

use ac2_net::protocol::{GameplayOptionsProfile, ScriptCall, ScriptMessage};

use super::gameplay::player_qualities;
use super::{OnlineSession, SessionEvent};
use crate::shell::{room_refused, ChatCommands, ChatHistory, ChatState, ShellAction, Speaker};

impl OnlineSession {
    /// The registered chat commands, resolved from the DATs on the loader side.
    pub fn set_chat_commands(&mut self, commands: ChatCommands) {
        self.chat.set_commands(commands);
    }

    /// One step through a chat history for the chat window's keys
    /// (`ChatWindow::DisplayCmdHistory` / `DisplayRetellHistory` / `DisplayReplyHistory`):
    /// `prev` walks to older entries; `None` past either end.
    pub fn chat_history(&mut self, history: ChatHistory, prev: bool) -> Option<String> {
        self.chat.history(history, prev)
    }

    /// The chat state (squelch list, rooms).
    pub fn chat(&self) -> &ChatState {
        &self.chat
    }

    /// `CWInterface::ProcessText`: run a line the chat window entered (a `/` command, or
    /// speech). Messages go to the server byte-exact; lines for the chat windows and
    /// commands the port does not run come back.
    pub fn process_text(&mut self, line: &str) -> Vec<SessionEvent> {
        let q = player_qualities(&self.world);
        let weenie = q.weenie.as_ref();
        let name = weenie
            .and_then(|w| w.name.as_ref())
            .and_then(|n| n.literal.clone())
            .unwrap_or_default();
        let monarch = weenie
            .and_then(|w| w.monarch)
            .or_else(|| q.iids.get(&0x1C4).copied())
            .unwrap_or(0);
        let speaker = Speaker {
            name: &name,
            monarch,
        };
        let actions = self
            .chat
            .process_text(line, speaker, self.script.string_tables());
        let mut out = Vec::new();
        for action in actions {
            match action {
                ShellAction::Send(request) => self.send(request.to_event()),
                ShellAction::Notice(notice) => out.push(SessionEvent::Notice(notice)),
                // `CPlayer` 0x103 / 0xE4 / 0x14C: a refused request shows the chat-server line.
                ShellAction::SendToRoom { room, text, flags } => {
                    if self.send_to_room(room, &text, flags).is_none() {
                        let refused = room_refused(self.script.string_tables());
                        out.push(SessionEvent::Notice(refused));
                    }
                }
                ShellAction::Command {
                    handler,
                    text,
                    args,
                } => out.push(SessionEvent::ShellCommand {
                    handler,
                    text,
                    args,
                }),
            }
        }
        out
    }

    /// `CWInterface::DisplayHelpText(cmd)`: the help the shell has for `cmd` (everything for an
    /// empty one) as lines for the chat windows.
    pub fn display_help(&self, cmd: &str) -> Vec<SessionEvent> {
        self.chat
            .display_help(cmd, self.script.string_tables())
            .into_iter()
            .filter_map(|action| match action {
                ShellAction::Notice(notice) => Some(SessionEvent::Notice(notice)),
                _ => None,
            })
            .collect()
    }

    /// `CCommunicationSystem` 0x6 / 0x1: speech or a tell from a squelched name is dropped
    /// (`CPlayer` 0x11A); a tell heard sets the /reply name (`CPlayer` 0xD1). Only literal
    /// names are compared (a table name would need the string tables' text).
    pub(super) fn chat_drops(&mut self, message: &ScriptMessage) -> bool {
        let (name, tell) = match message {
            ScriptMessage::HearSpeech { speaker_name, .. } => (speaker_name, false),
            ScriptMessage::HearTell { teller_name, .. } => (teller_name, true),
            _ => return false,
        };
        let literal = name.literal.as_deref().unwrap_or_default();
        if self.chat.has_squelched(literal) {
            return true;
        }
        if tell {
            self.chat.heard_tell(name);
        }
        false
    }

    /// The server calls that set the chat rooms; the call still reaches the caller raw.
    pub(super) fn receive_chat_call(
        &mut self,
        function: u32,
        arguments: &[u8],
    ) -> Vec<SessionEvent> {
        let Some(Ok(call)) = ScriptCall::decode(function, arguments) else {
            return Vec::new();
        };
        self.chat
            .server_call(&call, self.script.string_tables())
            .into_iter()
            .map(SessionEvent::Notice)
            .collect()
    }

    /// The options profile's aliases and chat filters (`CPlayer` 0x65).
    pub(super) fn chat_options(&mut self, profile: &GameplayOptionsProfile) {
        self.chat.login_options(profile);
    }
}
