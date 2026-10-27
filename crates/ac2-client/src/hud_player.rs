//! The HUD frameworks that ask the player, run once a frame with the client: the skills panel
//! (`gmSkillTreeUI`) and the shortcut bar (`gmShortcutUI`) take the frame's notices
//! (`RecvNotice_*`) and the tree's messages, ask the player's script and the session through
//! [`ClientPlayer`], and their requests become the player's calls and gameplay commands. The chat
//! takes the player's chat options when they are reloaded.
//!
//! [`HudView::frameworks`] runs the passes in the order the frameworks get them: the reload of
//! the options ([`HudView::reload_pass`]), the map ([`HudView::map_pass`]), the first frame in the
//! world ([`HudView::init_pass`]), the pick and the selection ([`HudView::pick_pass`]), the notices
//! ([`crate::hud_player_notice`]), the messages ([`crate::hud_player_message`]) and the ticks
//! ([`crate::hud_player_tick`]).

use std::cell::RefCell;

use ac2_net::protocol::ExaminationRequest;
use ac2_script::host::ScriptNotice;
use ac2_ui::PROFILE_LATENCY_INDICATOR_GRAPH;
use ac2_weenie::{GameplayNotice, SessionEvent};

use crate::client::Client;
use crate::frame::ClientFrame;
use crate::gameplay::Command;
use crate::hud_character::{character_commands, options_settings};
use crate::hud_chat_options::chat_settings;
use crate::hud_status::StatusBar;
use crate::hud_view::HudView;
use crate::ui_sources::ClientPlayer;

/// The player's race (int 290, `UpdatePaperDollPosition`).
const RACE: u32 = 0x122;

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
}

impl PassOut {
    fn append(&mut self, pass: PassOut) {
        self.commands.extend(pass.commands);
        self.open_examination |= pass.open_examination;
        self.tells.extend(pass.tells);
    }
}

/// The reload pass's outputs for the passes after it.
#[derive(Clone, Copy, Default)]
struct Reloads {
    radar: bool,
    show_radar_coords: Option<bool>,
    show_names: Option<bool>,
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
        let messages = std::mem::take(&mut self.player_messages);
        // `gmCWInterface::PopupYesNoDialog` raises its question at once, whether or not the HUD
        // is up yet: the dialogs are the game's own, always there in retail. A question that
        // comes before the HUD's dialogs waits in the backlog and is asked when they are.
        self.popup_backlog
            .extend(crate::hud_game_dialogs::popup_questions(events));
        let mut out = PassOut::default();
        let Some(strings) = self.strings.clone() else {
            return out.commands;
        };
        if !frame.in_world || self.hud.is_none() {
            return out.commands;
        }
        let reloads = self.reload_pass(client, frame, events, script_notices, &mut out.commands);
        let player = ClientPlayer {
            client: RefCell::new(client),
            frame,
            strings: &strings.tables,
            ui_table: strings.ui_table,
            enums: &strings.enums,
        };
        self.map_pass(&player, frame, reloads);
        self.init_pass(&player, frame);
        out.append(self.pick_pass(&player, frame, events, doll_pick));
        out.append(self.notice_pass(&player, frame, events, reloads.radar));
        out.append(self.message_pass(&player, frame, &messages));
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
        // `FellowshipHelper::ListenToMessage`: a member's name into the chat line.
        for name in out.tells {
            self.assisted_tell(&name);
        }
        out.commands
    }

    /// The options and saved layout that reload this frame (`gmChatOptionsUI`, `gmGamePlayUI`,
    /// `gmCharacterUI`, `gmNetStatusUI`, `gm3DTextUI`, `gmRadarUI` `RecvNotice_ReloadOptions`).
    fn reload_pass(
        &mut self,
        client: &mut Client,
        frame: &ClientFrame,
        events: &[SessionEvent],
        script_notices: &[ScriptNotice],
        commands: &mut Vec<Command>,
    ) -> Reloads {
        let HudView {
            hud,
            character,
            chat,
            chat_options,
            dirty,
            net,
            player_loaded,
            player_race,
            radar,
            ..
        } = self;
        let Some((tree, _)) = hud else {
            return Reloads::default();
        };
        // `gmGamePlayUI::RecvNotice_SaveLocationsChanged`: a window moved since the last frame.
        if tree.take_locations_changed() {
            commands.push(Command::SaveWindowLocations(tree.inq_locations()));
        }
        // The window exception (`AGENTS.md`): a window moved and its drag ended, so every
        // window's place is kept on this machine for the character.
        if tree.take_positions_changed() {
            if let Some(player) = frame.player.as_ref() {
                client.save_local_window_positions(player.id, &tree.window_positions());
            }
        }
        // `gmChatOptionsUI`'s Apply sent the reload notice; its values refresh on the options'
        // reload (`RecvNotice_ReloadOptions`), as the page's Refresh does.
        let applied = chat_options.as_mut().is_some_and(|page| page.take_reload());
        let reload_chat = applied
            || !*player_loaded
            || script_notices.iter().any(|n| {
                matches!(
                    n.name,
                    "CM_Communication::SendNotice_ReloadChatSettings"
                        | "CM_Character::SendNotice_ReloadOptions"
                )
            });
        if reload_chat {
            if let Some(settings) = chat_settings(client) {
                if let Some(chat) = chat.as_mut() {
                    chat.reload_chat_settings(tree, &settings);
                }
                let reload_page = !*player_loaded
                    || script_notices
                        .iter()
                        .any(|n| n.name == "CM_Character::SendNotice_ReloadOptions");
                if let (true, Some(page)) = (reload_page, chat_options.as_mut()) {
                    page.refresh(tree, &settings);
                }
            }
        }
        // The chat windows' opacity sliders released since the last frame.
        if let Some(chat) = chat.as_mut() {
            for (window, opacity) in chat.take_opacity_requests() {
                commands.push(Command::SetWindowOpacity { window, opacity });
            }
        }
        // `gmRadarUI::ReloadOptions` (at login, `Init`, and on `RecvNotice_ReloadOptions`).
        let reload_radar = !*player_loaded
            || script_notices
                .iter()
                .any(|n| n.name == "CM_Character::SendNotice_ReloadOptions");
        let show_radar_coords = reload_radar.then(|| client.show_radar_coords());
        // `gmGamePlayUI::RecvNotice_ReloadOptions` (and the refresh at login, `ListenToMessage`
        // 0xffff0007): the player's saved window locations apply to the windows.
        if reload_radar {
            if let Ok(saved) = client.saved_window_locations() {
                tree.apply_locations(&saved);
                if let Some(radar) = radar.as_ref() {
                    radar.location_applied(tree);
                }
            }
            // The player's own places, per character, override the server's (`AGENTS.md`).
            if let Some(player) = frame.player.as_ref() {
                match client.local_window_positions(player.id) {
                    Ok(local) => tree.set_window_positions(&local),
                    Err(error) => eprintln!("ac2: window positions: {error}"),
                }
            }
        }
        // `gmCharacterUI::RecvNotice_ReloadOptions`: the page's values as the profile has them.
        let reload_page = reload_radar
            || events
                .iter()
                .any(|e| matches!(e, SessionEvent::Notice(GameplayNotice::ReloadOptions)));
        if let (true, Some(page)) = (reload_page, character.as_mut()) {
            if let Some(settings) = options_settings(client) {
                page.refresh(tree, &settings);
            }
        }
        if let Some(page) = character.as_mut() {
            character_commands(page, commands);
        }
        // `gmNetStatusUI::RecvNotice_ReloadOptions`: the latency graph's option.
        if let (true, Some(net)) = (reload_radar, net.as_mut()) {
            if let Some(settings) = options_settings(client) {
                net.set_graph_enabled(settings.bits & PROFILE_LATENCY_INDICATOR_GRAPH != 0);
                *dirty = true;
            }
        }
        // `gm3DTextUI::RecvNotice_ReloadOptions`: the names as the options say.
        let show_names = reload_radar.then(|| client.show_object_names());
        *player_race = frame
            .player
            .as_ref()
            .and_then(|p| client.session().objects().get(p.id))
            .and_then(|o| o.qualities.inq_int(RACE));
        Reloads {
            radar: reload_radar,
            show_radar_coords,
            show_names,
        }
    }

    /// The name plates, the radar's options and position, and the macro map's date and time
    /// (`gm3DTextUI`, `gmRadarUI::Update`, `gmNavigationUI::UpdateMacroMap`).
    fn map_pass(&mut self, player: &ClientPlayer<'_>, frame: &ClientFrame, reloads: Reloads) {
        let HudView {
            hud,
            names,
            navigation_time,
            navigation_tooltips,
            radar,
            radar_cell,
            radar_coordinates,
            ..
        } = self;
        let Some((tree, _)) = hud else {
            return;
        };
        let Reloads {
            show_names,
            show_radar_coords,
            ..
        } = reloads;
        if let Some(plates) = names.as_mut() {
            let looks = name_looks(frame);
            match show_names {
                Some(show) => plates.set_showing(tree, show, &looks),
                None => plates.sync(tree, &looks),
            }
        }
        if let (Some(show), Some(radar)) = (show_radar_coords, radar.as_mut()) {
            radar.reload_options(tree, player, show);
        }
        // `gmRadarUI::Update`'s `Position::ToRealMapCoords_Info`, again when the cell changes.
        let cell = frame.player.as_ref().map(|p| p.cell);
        if cell.is_some() && cell != *radar_cell {
            *radar_cell = cell;
            let string = crate::ui_sources::map_coords_string(cell.unwrap_or(0));
            *radar_coordinates = Some(ac2_ui::UiStrings::render_string(player, &string));
        }
        // `gmNavigationUI::AddMarker`'s tooltips, rendered with the frame's markers.
        *navigation_tooltips = frame
            .markers
            .iter()
            .map(|m| player.marker_tooltip(&m.message, m.cell))
            .collect();
        // `gmNavigationUI::UpdateMacroMap`'s date and time: rendered again when the calendar moves.
        if let Some(date_time) = frame.date_time {
            if navigation_time.as_ref().map(|(known, _, _)| *known) != Some(date_time) {
                let (date, time) = crate::ui_sources::game_date_time(date_time);
                *navigation_time = Some((
                    date_time,
                    ac2_ui::UiStrings::render_string(player, &date),
                    ac2_ui::UiStrings::render_string(player, &time),
                ));
            }
        }
    }

    /// The first frame in the world (`!player_loaded`): the panels read the player once.
    fn init_pass(&mut self, player: &ClientPlayer<'_>, frame: &ClientFrame) {
        let HudView {
            hud,
            allegiance,
            backpack,
            effects,
            item_selection,
            items_3d,
            money,
            net,
            paper_doll,
            player_loaded,
            secure_trade,
            skills,
            toolbar_layout,
            vitae,
            vitals,
            ..
        } = self;
        let Some((tree, _)) = hud else {
            return;
        };
        if !*player_loaded {
            *player_loaded = true;
            // `gmSecureTradeUI::gmSecureTradeUI` reads the player.
            if let (Some(panel), Some(id)) = (secure_trade.as_mut(), frame.player.as_ref()) {
                panel.set_player(id.id);
            }
            if let Some(skills) = skills.as_mut() {
                skills.update_skill_panel(tree, player);
            }
            if let Some(page) = allegiance.as_mut() {
                page.set_player(tree, ac2_ui::FellowshipSource::player(player));
            }
            // `gmVitalsUI`, `gmEffectsUI` (`Init`) and `gmVitaeStatusUI` read the player.
            StatusBar {
                vitals: &mut *vitals,
                effects: &mut *effects,
                vitae: &mut *vitae,
                net: &mut *net,
            }
            .player_loaded(tree, player, &frame.player_vitals, frame.server_time);
            // `gmInventoryUI::SetupChildren`'s backpack over the player.
            if let (Some(layout), Some(id)) =
                (toolbar_layout.as_ref(), frame.player.as_ref().map(|p| p.id))
            {
                *item_selection = ac2_ui::InventorySource::selection(player);
                *backpack = Some(ac2_ui::BackpackUi::create(tree, layout, player, id));
                *items_3d = Some(ac2_ui::Items3dUi::create(tree, id));
                // `gmMoneyBagUI::gmMoneyBagUI` and its amount (`RecvNotice_UpdateMoney`).
                let mut bag = ac2_ui::MoneyBagUi::create(tree);
                bag.set_money(tree, player, frame.inventory.money.max(0) as u32);
                *money = Some(bag);
                let equipped = player.equipped();
                let doll = ac2_ui::PaperDollUi::create(tree, layout, player, id, &equipped);
                doll.follow_panel(tree);
                *paper_doll = Some(doll);
            }
        }
    }

    /// The paper doll's pick, the target selection's notices and the GUN friend events, ahead of
    /// the notices.
    fn pick_pass(
        &mut self,
        player: &ClientPlayer<'_>,
        frame: &ClientFrame,
        events: &[SessionEvent],
        doll_pick: Option<Option<u32>>,
    ) -> PassOut {
        let HudView {
            hud,
            friends,
            paper_doll,
            ..
        } = self;
        let Some((tree, _)) = hud else {
            return PassOut::default();
        };
        let mut commands = Vec::new();
        let mut open_examination = false;
        // `gmPaperDollUI::FinishPaperDollClick` with the scene's answer.
        if let (Some(doll), Some(mesh_type)) = (paper_doll.as_mut(), doll_pick) {
            if let Some(request) = doll.finish_pick(tree, mesh_type) {
                open_examination |= crate::ui_inventory::item_commands(request, &mut commands);
            }
        }
        {
            // `gmSelectionUI`: the selection keys, its notices and `RecvNotice_BeingDeleted`.
            let mut client = player.client.borrow_mut();
            let typing = tree.text_mode();
            commands.extend(crate::target_select::frame_notices(
                &mut client,
                frame,
                tree,
                events,
                typing,
            ));
        }
        // `gmFriendsUI`: the GUN list and status the client took since the last frame, then the
        // add and remove buttons for the game selection (`UpdateButtonStates`, every frame).
        let friend_events = player.client.borrow_mut().take_friend_events();
        if let Some(panel) = friends.as_mut() {
            for event in friend_events {
                crate::hud_friends::friend_event(panel, tree, event);
            }
            let target = crate::hud_friends::friend_target(frame.selection.as_ref());
            panel.set_target(tree, target);
        }
        PassOut {
            commands,
            open_examination,
            ..PassOut::default()
        }
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
