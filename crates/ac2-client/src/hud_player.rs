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
    /// The player confirmed the quit question: close the client after the commands go out.
    pub(crate) quit: bool,
    pub(crate) tells: Vec<String>,
    /// The help topics the panels asked to open (`KeyStone::OpenHelp` tokens), in order.
    pub(crate) help: Vec<u32>,
    /// The story's show state for the gameplay panels (`gmStoryUI` begin `false`, hide `true`).
    pub(crate) story_gameplay: Vec<bool>,
    /// `gmGamePlayUI::Show` calls of `RecvNotice_UpdateDeathState` (dead `false`, alive `true`).
    pub(crate) death_gameplay: Vec<bool>,
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
        picks: (Option<Option<u32>>, Option<Option<u64>>),
    ) -> Vec<Command> {
        let strings = self.panels.strings.clone();
        let player = strings.as_ref().map(|strings| ClientPlayer {
            client: RefCell::new(client),
            frame,
            strings: &strings.tables,
            ui_table: strings.ui_table,
            enums: &strings.enums,
        });
        let looks = || name_looks(frame, player.as_ref());
        let friend = friend_target(frame.selection.as_ref());
        let inputs = ac2_ui::FrameInputs {
            server_time: frame.server_time,
            player: frame.player.as_ref().map(|p| p.id),
            player_cell: frame.player.as_ref().map(|p| p.cell),
            in_world: frame.in_world,
            player_moving: frame
                .player
                .as_ref()
                .is_some_and(|p| ac2_physics::is_moving_velocity(p.velocity)),
            selection: frame.selection.as_ref().map(|s| s.id),
            player_vitals: &frame.player_vitals,
            money: frame.inventory.money.max(0) as u32,
            markers: frame.markers.as_slice(),
            date_time: frame.date_time,
            name_looks: &looks,
            friend_target: friend,
            events,
            script_notices,
            doll_pick: picks.0,
            world_pick: picks.1,
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
        // The Tracker's requests run against the Tracker here; the rest become commands.
        let (tracker, requests): (Vec<_>, Vec<_>) = requests.into_iter().partition(|r| {
            matches!(
                r,
                ac2_ui::HudRequest::Tracker(_)
                    | ac2_ui::HudRequest::PointAt { .. }
                    | ac2_ui::HudRequest::TextPress(_)
            )
        });
        self.tracker_frame(frame, tracker, &mut out.commands);
        hud_request_commands(requests, &mut out);
        self.panels.quit_requested |= out.quit;
        // The examination panel, the help topics and the tells (`FellowshipHelper`).
        if let Some((tree, _)) = self.hud.as_mut() {
            let effects = ac2_ui::HudEffects {
                open_examination: out.open_examination,
                help: std::mem::take(&mut out.help),
                tells: std::mem::take(&mut out.tells),
            };
            self.panels.apply_effects(tree, &mut self.dirty, effects);
        }
        // The gameplay panels leave while a story plays and come back as they were after it.
        for shown in std::mem::take(&mut out.story_gameplay) {
            self.set_story_gameplay_shown(shown);
        }
        // Dying hides them and coming back shows them, unless a story holds the UI
        // (`gmGamePlayUI::Show`).
        for shown in std::mem::take(&mut out.death_gameplay) {
            if self.story_hid_gameplay.is_none() {
                self.set_gameplay_shown(shown);
            }
        }
        out.commands
    }
}

impl HudView {
    /// [`crate::view::ClientView::offline_frameworks`]: the additions' passes
    /// (`ac2_ui::Hud::offline_frame`), then the Tracker.
    pub(crate) fn offline_frameworks(&mut self, frame: &ClientFrame) {
        let tree = self.hud.as_mut().map(|(tree, _)| tree);
        let requests = self.panels.offline_frame(tree, &mut self.dirty);
        self.tracker_frame(frame, requests, &mut Vec::new());
    }

    /// Runs `toggle` on the HUD's panels and tree, once the HUD is built.
    pub(crate) fn toggle_addition(&mut self, toggle: fn(&mut ac2_ui::Hud, &mut ac2_ui::UiTree)) {
        if let Some((tree, _)) = self.hud.as_mut() {
            toggle(&mut self.panels, tree);
            self.dirty = true;
        }
    }
}

/// The radar shape `gm3DTextUI::AddName` gives no name plate.
const NO_NAME_SHAPE: u32 = 0x4000_000B;

/// `gm3DTextUI::AddName`'s inputs: every object with a radar look, its look's colour and its
/// name at its quantity (`GetName(GetQuantity())`: a stack's count and plural, rendered from the
/// player's tables).
pub(crate) fn name_looks(
    frame: &ClientFrame,
    player: Option<&ClientPlayer<'_>>,
) -> Vec<ac2_ui::NameLook> {
    frame
        .radar
        .blips
        .iter()
        // `AddName` names no object whose radar shape is `0x4000000B`.
        .filter(|b| b.shape != NO_NAME_SHAPE)
        .filter_map(|b| {
            let object = frame.objects.iter().find(|o| o.id == b.id)?;
            Some(ac2_ui::NameLook {
                id: b.id,
                name: match (&object.stack_name, player) {
                    (Some(stack), Some(player)) => player.render_info(stack).into(),
                    _ => object.name.clone()?,
                },
                color: b.color,
            })
        })
        .collect()
}
