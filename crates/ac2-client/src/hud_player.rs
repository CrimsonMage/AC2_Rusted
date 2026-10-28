//! The HUD frameworks that ask the player, run once a frame with the client: the skills panel
//! (`gmSkillTreeUI`) and the shortcut bar (`gmShortcutUI`) take the frame's notices
//! (`RecvNotice_*`) and the tree's messages, ask the player's script and the session through
//! [`ClientPlayer`], and their requests become the player's calls and gameplay commands. The chat
//! takes the player's chat options when they are reloaded.
//!
//! [`HudView::frameworks`] runs the passes in the order the frameworks get them: the reload of
//! the options, the map, the first frame in the world and the pick (`ac2_ui::Hud`'s passes over
//! [`ac2_ui::HudHost`]), the notices ([`crate::hud_player_notice`]), the messages
//! ([`crate::hud_player_message`]) and the ticks ([`crate::hud_player_tick`]).

use std::cell::RefCell;

use ac2_net::protocol::ExaminationRequest;
use ac2_script::host::ScriptNotice;
use ac2_weenie::SessionEvent;

use crate::client::Client;
use crate::frame::ClientFrame;
use crate::gameplay::Command;
use crate::hud_friends::friend_target;
use crate::hud_request::hud_request_commands;
use crate::hud_view::HudView;
use crate::ui_sources::ClientPlayer;

/// `SetSelection(INVALID_IID)` (or the item), `SetExaminationRequest`.
pub(crate) fn examine(kind: u32, data: u32, item: u64, commands: &mut Vec<Command>) {
    let mut request = ExaminationRequest {
        kind,
        ..ExaminationRequest::default()
    };
    match kind {
        0x4000_0002 => request.iid = item,
        0x4000_0007 => request.data_id = data,
        _ => request.data_enum = data,
    }
    commands.push(Command::Select((item != 0).then_some(item)));
    commands.push(Command::Examine(request));
}

/// What the frame's passes ask for: the gameplay commands, whether an examination opens, and the
/// names the assisted tell sends.
#[derive(Default)]
pub(crate) struct PassOut {
    pub(crate) commands: Vec<Command>,
    pub(crate) open_examination: bool,
    pub(crate) tells: Vec<String>,
    /// The help topics the panels asked to open (`KeyStone::OpenHelp` tokens), in order.
    pub(crate) help: Vec<u32>,
}

impl PassOut {
    fn append(&mut self, pass: PassOut) {
        self.commands.extend(pass.commands);
        self.open_examination |= pass.open_examination;
        self.tells.extend(pass.tells);
        self.help.extend(pass.help);
    }
}

impl HudView {
    /// One frame of the frameworks that ask the player; returns the gameplay commands they ask
    /// for. They wait for the HUD, the string tables and the player in the world.
    pub(crate) fn frameworks(
        &mut self,
        client: &mut Client,
        frame: &ClientFrame,
        events: &[SessionEvent],
        script_notices: &[ScriptNotice],
        doll_pick: Option<Option<u32>>,
    ) -> Vec<Command> {
        let messages = std::mem::take(&mut self.panels.player_messages);
        // `gmCWInterface::PopupYesNoDialog` raises its question at once, whether or not the HUD
        // is up yet: the dialogs are the game's own, always there in retail. A question that
        // comes before the HUD's dialogs waits in the backlog and is asked when they are.
        self.panels
            .popup_backlog
            .extend(ac2_ui::popup_questions(events));
        // `gmGamePlayUI::RecvNotice_DisplayMessage` is registered with the HUD's children
        // (`gmGamePlayUI::SetupChildren`), so a box raised in the world waits for the HUD; before
        // the world there is no handler and the notice is gone, as in retail.
        if frame.in_world {
            self.panels
                .display_backlog
                .extend(ac2_ui::display_notices(events));
        }
        let mut out = PassOut::default();
        let Some(strings) = self.panels.strings.clone() else {
            return out.commands;
        };
        if !frame.in_world || self.hud.is_none() {
            return out.commands;
        }
        let player = ClientPlayer {
            client: RefCell::new(client),
            frame,
            strings: &strings.tables,
            ui_table: strings.ui_table,
            enums: &strings.enums,
        };
        let looks = || name_looks(frame);
        let friend = friend_target(frame.selection.as_ref());
        let inputs = ac2_ui::FrameInputs {
            server_time: frame.server_time,
            player: frame.player.as_ref().map(|p| p.id),
            player_cell: frame.player.as_ref().map(|p| p.cell),
            player_vitals: &frame.player_vitals,
            money: frame.inventory.money.max(0) as u32,
            markers: frame.markers.as_slice(),
            date_time: frame.date_time,
            name_looks: &looks,
            friend_target: friend,
            events,
            script_notices,
            doll_pick,
        };
        // The reload, map, first-frame and pick passes, in the frameworks' order.
        let (reload, reloads, pick) = {
            let HudView {
                hud, dirty, panels, ..
            } = self;
            let Some((tree, _)) = hud else {
                return out.commands;
            };
            let (reload, reloads) = panels.reload_pass(tree, dirty, &player, &inputs);
            panels.map_pass(tree, &player, &inputs, reloads);
            panels.init_pass(tree, &player, &inputs);
            let pick = panels.pick_pass(tree, &player, &inputs);
            (reload, reloads, pick)
        };
        hud_request_commands(reload, &mut out);
        hud_request_commands(pick, &mut out);
        out.append(self.notice_pass(&player, frame, events, reloads.radar));
        out.append(self.message_pass(&player, frame, &messages));
        self.help_pass(&messages);
        out.append(self.tick_pass(&player, frame, events));
        let HudView { hud, dirty, .. } = self;
        if let Some((tree, _)) = hud {
            if tree.take_changed() {
                *dirty = true;
            }
        }
        if out.open_examination {
            self.open_examination();
        }
        for token in std::mem::take(&mut out.help) {
            self.open_help(token);
        }
        // `FellowshipHelper::ListenToMessage`: a member's name into the chat line.
        for name in out.tells {
            self.assisted_tell(&name);
        }
        out.commands
    }
}

/// `gm3DTextUI::AddName`'s inputs: every object with a radar look, its look's colour and its
/// name.
pub(crate) fn name_looks(frame: &ClientFrame) -> Vec<ac2_ui::NameLook> {
    frame
        .radar
        .blips
        .iter()
        .filter_map(|b| {
            let object = frame.objects.iter().find(|o| o.id == b.id)?;
            Some(ac2_ui::NameLook {
                id: b.id,
                name: object.name.clone()?,
                color: b.color,
            })
        })
        .collect()
}
