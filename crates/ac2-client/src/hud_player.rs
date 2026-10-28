//! The HUD frameworks that ask the player, run once a frame with the client: the skills panel
//! (`gmSkillTreeUI`) and the shortcut bar (`gmShortcutUI`) take the frame's notices
//! (`RecvNotice_*`) and the tree's messages, ask the player's script and the session through
//! [`ClientPlayer`], and their requests become the player's calls and gameplay commands. The chat
//! takes the player's chat options when they are reloaded.
//!
//! [`HudView::frameworks`] hands the frame to `ac2_ui::Hud::frame`, which runs the passes in the
//! order the frameworks get them (the reload of the options, the map, the first frame in the
//! world, the pick, the notices, the messages, the help pages and the ticks). The client supplies
//! its [`ac2_ui::HudHost`] and the option and shortcut sinks ([`crate::hud_player_message`]), and
//! maps the requests it returns to gameplay commands ([`crate::hud_request`]).

use std::cell::RefCell;

use ac2_net::protocol::ExaminationRequest;
use ac2_script::host::ScriptNotice;
use ac2_weenie::SessionEvent;

use crate::client::Client;
use crate::frame::ClientFrame;
use crate::gameplay::Command;
use crate::hud_friends::friend_target;
use crate::hud_player_message::{ClientOptions, ClientShortcuts};
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

impl HudView {
    /// One frame of the frameworks that ask the player; returns the gameplay commands they ask
    /// for. The sequencing is `ac2_ui::Hud::frame`'s; this builds its inputs over the client and
    /// maps the requests it returns. They wait for the HUD, the string tables and the player in
    /// the world.
    pub(crate) fn frameworks(
        &mut self,
        client: &mut Client,
        frame: &ClientFrame,
        events: &[SessionEvent],
        script_notices: &[ScriptNotice],
        doll_pick: Option<Option<u32>>,
    ) -> Vec<Command> {
        let strings = self.panels.strings.clone();
        let player = strings.as_ref().map(|strings| ClientPlayer {
            client: RefCell::new(client),
            frame,
            strings: &strings.tables,
            ui_table: strings.ui_table,
            enums: &strings.enums,
        });
        let looks = || name_looks(frame);
        let friend = friend_target(frame.selection.as_ref());
        let inputs = ac2_ui::FrameInputs {
            server_time: frame.server_time,
            player: frame.player.as_ref().map(|p| p.id),
            player_cell: frame.player.as_ref().map(|p| p.cell),
            in_world: frame.in_world,
            selection: frame.selection.as_ref().map(|s| s.id),
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
        let requests = {
            let HudView {
                hud, dirty, panels, ..
            } = self;
            let tree = hud.as_mut().map(|(tree, _)| tree);
            match player.as_ref() {
                Some(player) => {
                    let mut options = ClientOptions { player };
                    let mut shortcuts = ClientShortcuts { player };
                    let hosts = ac2_ui::FrameHosts {
                        player,
                        options: &mut options,
                        shortcuts: &mut shortcuts,
                    };
                    panels.frame(tree, dirty, Some(hosts), &inputs)
                }
                None => panels.frame::<ClientPlayer<'_>>(tree, dirty, None, &inputs),
            }
        };
        let mut out = PassOut::default();
        hud_request_commands(requests, &mut out);
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
