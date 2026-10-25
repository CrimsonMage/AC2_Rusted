//! The HUD frameworks that ask the player, run once a frame with the client: the skills panel
//! (`gmSkillTreeUI`) and the shortcut bar (`gmShortcutUI`) take the frame's notices
//! (`RecvNotice_*`) and the tree's messages, ask the player's script and the session through
//! [`ClientPlayer`], and their requests become the player's calls and gameplay commands. The chat
//! takes the player's chat options when they are reloaded.

use std::cell::RefCell;

use ac2_net::protocol::ExaminationRequest;
use ac2_script::host::ScriptNotice;
use ac2_ui::{ChatSettings, ShortcutRequest, SkillRequest};
use ac2_weenie::{GameplayNotice, SessionEvent};

use crate::client::Client;
use crate::frame::ClientFrame;
use crate::gameplay::Command;
use crate::hud_view::HudView;
use crate::ui_sources::{shortcut_to_info, ClientPlayer};

/// The player's race (int 290, `UpdatePaperDollPosition`).
const RACE: u32 = 0x122;

/// The player's chat options as the chat windows take them.
fn chat_settings(client: &mut Client) -> Option<ChatSettings> {
    let settings = client.chat_settings().ok()?;
    Some(ChatSettings {
        filters: settings.filters,
        font_sizes: settings.font_sizes,
        font_colors: settings.font_colors,
        popups: settings
            .popups
            .into_iter()
            .map(|(kind, popup)| (kind, popup != 0))
            .collect(),
    })
}

/// `SetSelection(INVALID_IID)` (or the item), `SetExaminationRequest`.
fn examine(kind: u32, data: u32, item: u64, commands: &mut Vec<Command>) {
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
        let mut commands = Vec::new();
        let mut open_examination = false;
        let Some(strings) = self.strings.clone() else {
            return commands;
        };
        if !frame.in_world {
            return commands;
        }
        let HudView {
            hud,
            skills,
            shortcuts,
            examination,
            chat,
            player_loaded,
            dirty,
            fellowship,
            allegiance,
            vitals,
            effects,
            dialogs,
            stack,
            external_container,
            secure_trade,
            money,
            shift_held,
            backpack,
            paper_doll,
            player_race,
            toolbar_layout,
            item_selection,
            radar_cell,
            radar_coordinates,
            radar,
            names,
            ..
        } = self;
        let Some((tree, _)) = hud else {
            return commands;
        };
        let reload_chat = !*player_loaded
            || script_notices.iter().any(|n| {
                matches!(
                    n.name,
                    "CM_Communication::SendNotice_ReloadChatSettings"
                        | "CM_Character::SendNotice_ReloadOptions"
                )
            });
        if reload_chat {
            if let (Some(chat), Some(settings)) = (chat.as_mut(), chat_settings(client)) {
                chat.reload_chat_settings(tree, &settings);
            }
        }
        // `gmRadarUI::ReloadOptions` (at login, `Init`, and on `RecvNotice_ReloadOptions`).
        let reload_radar = !*player_loaded
            || script_notices
                .iter()
                .any(|n| n.name == "CM_Character::SendNotice_ReloadOptions");
        let show_radar_coords = reload_radar.then(|| client.show_radar_coords());
        // `gm3DTextUI::RecvNotice_ReloadOptions`: the names as the options say.
        let show_names = reload_radar.then(|| client.show_object_names());
        *player_race = frame
            .player
            .as_ref()
            .and_then(|p| client.session().objects().get(p.id))
            .and_then(|o| o.qualities.inq_int(RACE));
        let player = ClientPlayer {
            client: RefCell::new(client),
            frame,
            strings: &strings.tables,
            ui_table: strings.ui_table,
            enums: &strings.enums,
        };
        if let Some(plates) = names.as_mut() {
            let looks = name_looks(frame);
            match show_names {
                Some(show) => plates.set_showing(tree, show, &looks),
                None => plates.sync(tree, &looks),
            }
        }
        if let (Some(show), Some(radar)) = (show_radar_coords, radar.as_mut()) {
            radar.reload_options(tree, &player, show);
        }
        // `gmRadarUI::Update`'s `Position::ToRealMapCoords_Info`, again when the cell changes.
        let cell = frame.player.as_ref().map(|p| p.cell);
        if cell.is_some() && cell != *radar_cell {
            *radar_cell = cell;
            let string = crate::ui_sources::map_coords_string(cell.unwrap_or(0));
            *radar_coordinates = Some(ac2_ui::UiStrings::render_string(&player, &string));
        }
        let mut reload_shortcuts = !*player_loaded
            || script_notices
                .iter()
                .any(|n| n.name == "CM_Character::SendNotice_ReloadOptions");
        let mut check_shortcuts = false;
        if !*player_loaded {
            *player_loaded = true;
            // `gmSecureTradeUI::gmSecureTradeUI` reads the player.
            if let (Some(panel), Some(id)) = (secure_trade.as_mut(), frame.player.as_ref()) {
                panel.set_player(id.id);
            }
            if let Some(skills) = skills.as_mut() {
                skills.update_skill_panel(tree, &player);
            }
            if let Some(page) = allegiance.as_mut() {
                page.set_player(tree, ac2_ui::FellowshipSource::player(&player));
            }
            if let Some(bar) = vitals.as_mut() {
                let text = |cur, max| ac2_ui::vital_text(&player, cur, max);
                bar.apply_snapshot(tree, &frame.player_vitals, &text);
            }
            // `gmEffectsUI::gmEffectsUI`'s `Init`.
            if let Some(strip) = effects.as_mut() {
                let list = crate::ui_inventory::effect_entries(&player);
                strip.init(tree, &list, frame.server_time);
            }
            // `gmInventoryUI::SetupChildren`'s backpack over the player.
            if let (Some(layout), Some(id)) =
                (toolbar_layout.as_ref(), frame.player.as_ref().map(|p| p.id))
            {
                *item_selection = ac2_ui::InventorySource::selection(&player);
                *backpack = Some(ac2_ui::BackpackUi::create(tree, layout, &player, id));
                // `gmMoneyBagUI::gmMoneyBagUI` and its amount (`RecvNotice_UpdateMoney`).
                let mut bag = ac2_ui::MoneyBagUi::create(tree);
                bag.set_money(tree, &player, frame.inventory.money.max(0) as u32);
                *money = Some(bag);
                let equipped = &frame.inventory.equipped;
                let doll = ac2_ui::PaperDollUi::create(tree, layout, &player, id, equipped);
                doll.follow_panel(tree);
                *paper_doll = Some(doll);
            }
        }
        // `gmPaperDollUI::FinishPaperDollClick` with the scene's answer.
        if let (Some(doll), Some(mesh_type)) = (paper_doll.as_mut(), doll_pick) {
            if let Some(request) = doll.finish_pick(tree, mesh_type) {
                open_examination |= crate::ui_inventory::item_commands(request, &mut commands);
            }
        }
        for event in events {
            if let Some(dialog) = stack.as_mut() {
                crate::ui_inventory::stack_event(dialog, tree, event);
            }
            if let Some(panel) = external_container.as_mut() {
                crate::ui_inventory::external_container_event(
                    panel,
                    tree,
                    &player,
                    stack.as_mut(),
                    *item_selection,
                    event,
                );
            }
            let SessionEvent::Notice(notice) = event else {
                continue;
            };
            if let (Some(panel), Some(dialogs)) = (secure_trade.as_mut(), dialogs.as_mut()) {
                crate::ui_inventory::secure_trade_notice(
                    panel,
                    tree,
                    dialogs,
                    &player,
                    frame.server_time,
                    notice,
                );
            }
            if let (Some(panel), Some(dialogs)) = (fellowship.as_mut(), dialogs.as_mut()) {
                crate::ui_social::fellowship_notice(panel, tree, dialogs, &player, notice);
            }
            if let (Some(page), Some(dialogs)) = (allegiance.as_mut(), dialogs.as_mut()) {
                crate::ui_social::allegiance_notice(page, tree, dialogs, &player, notice);
            }
            if let Some(doll) = paper_doll.as_mut() {
                let old = *item_selection;
                crate::ui_inventory::paper_doll_notice(doll, tree, &player, old, notice);
            }
            if let Some(pack) = backpack.as_mut() {
                crate::ui_inventory::backpack_notice(pack, tree, &player, item_selection, notice);
            }
            if let Some(strip) = effects.as_mut() {
                let now = frame.server_time;
                match notice {
                    GameplayNotice::UpdateEffect => {
                        let list = crate::ui_inventory::effect_entries(&player);
                        strip.init(tree, &list, now);
                    }
                    GameplayNotice::PulseMissed(eid) => strip.pulse_missed(tree, *eid, now),
                    GameplayNotice::ExaminationRequestChanged(request) => {
                        let list = crate::ui_inventory::effect_entries(&player);
                        strip.examination_request_changed(
                            tree,
                            request.kind,
                            request.data_enum,
                            &list,
                            now,
                        );
                    }
                    _ => {}
                }
            }
            if let Some(bar) = vitals.as_mut() {
                let text = |cur, max| ac2_ui::vital_text(&player, cur, max);
                match *notice {
                    GameplayNotice::UpdateVital {
                        vital,
                        current,
                        max,
                    } => {
                        bar.update_vital(tree, vital, current, max, &text);
                    }
                    GameplayNotice::UpdateHeroStatus { hero, current, max } => {
                        bar.update_hero_status(tree, hero, current, max, &text);
                    }
                    _ => {}
                }
            }
            match notice {
                GameplayNotice::UpdateExaminationProfile(profile) => {
                    if let Some(panel) = examination.as_mut() {
                        let items = player.examination_items(profile);
                        panel.update_profile(tree, &items);
                    }
                }
                GameplayNotice::ExaminationRequestChanged(request) => {
                    if let Some(panel) = examination.as_mut() {
                        panel.request_changed(tree, request.kind);
                    }
                    if let Some(skills) = skills.as_mut() {
                        skills.examination_request_changed(
                            tree,
                            &player,
                            request.kind,
                            request.data_enum,
                        );
                    }
                }
                GameplayNotice::UpdateSkillPanel { .. } => {
                    if let Some(skills) = skills.as_mut() {
                        skills.update_skill_panel(tree, &player);
                    }
                }
                GameplayNotice::UpdateSkill(skill) => {
                    if let Some(skills) = skills.as_mut() {
                        skills.update_skill(tree, &player, *skill);
                    }
                    check_shortcuts = true;
                }
                GameplayNotice::ReloadShortcuts | GameplayNotice::ReloadOptions => {
                    reload_shortcuts = true;
                }
                GameplayNotice::UpdateInventory { .. } | GameplayNotice::UpdateContainer { .. } => {
                    check_shortcuts = true;
                }
                GameplayNotice::UpdateMoney(amount) => {
                    if let Some(bag) = money.as_mut() {
                        bag.set_money(tree, &player, *amount);
                    }
                }
                _ => {}
            }
        }
        if let Some(bar) = shortcuts.as_mut() {
            if reload_shortcuts {
                bar.reload(tree, &player);
            } else if check_shortcuts {
                bar.check_all(tree, &player);
            }
        }
        for message in &messages {
            if let Some(skills) = skills.as_mut() {
                skills.message(tree, &player, message);
            }
            if let Some(bar) = shortcuts.as_mut() {
                bar.message(tree, &player, message);
            }
            if let Some(panel) = fellowship.as_mut() {
                panel.message(tree, message);
            }
            if let Some(bar) = vitals.as_mut() {
                bar.message(tree, message);
            }
            // `gmRadarUI::ListenToMessage`: select the blip under the pointer, or show or hide
            // the coordinates.
            if let Some(request) = radar
                .as_mut()
                .and_then(|r| r.message(tree, &player, message))
            {
                commands.push(match request {
                    ac2_ui::RadarRequest::Select(id) => Command::Select(Some(id)),
                    ac2_ui::RadarRequest::ShowCoords(show) => Command::SetShowRadarCoords(show),
                });
            }
            // `gm3DTextUI::ListenToMessage`: a plate selects its object; the names key toggles
            // the names (`ToggleNames`).
            if let Some(plates) = names.as_mut() {
                if let Some(id) = plates.message(tree, message) {
                    commands.push(Command::Select(Some(id)));
                }
                if ac2_ui::NamePlates::toggle_requested(message) {
                    let show = !plates.showing();
                    plates.set_showing(tree, show, &name_looks(frame));
                    commands.push(Command::SetShowObjectNames(show));
                }
            }
            // `gmEffectsUI::ListenToMessage`: examine the pressed effect.
            if let Some(ac2_ui::ExamineEffect(eid)) =
                effects.as_mut().and_then(|s| s.message(tree, message))
            {
                examine(0x4000_0001, eid, 0, &mut commands);
                open_examination = true;
            }
            if let Some(request) = backpack
                .as_mut()
                .and_then(|p| p.message(tree, &player, message))
            {
                open_examination |= crate::ui_inventory::item_request(
                    request,
                    tree,
                    stack.as_mut(),
                    &player,
                    &mut commands,
                );
            }
            // `gmMoneyBagUI::ListenToMessage`: a drop on the bag.
            if let (Some(bag), Some(dialogs)) = (money.as_mut(), dialogs.as_mut()) {
                if let Some(request) =
                    bag.message(tree, &player, &player, dialogs, message, *shift_held)
                {
                    open_examination |= crate::ui_inventory::item_request(
                        request,
                        tree,
                        stack.as_mut(),
                        &player,
                        &mut commands,
                    );
                }
            }
            if let Some(panel) = external_container.as_mut() {
                let (request, close) = panel.message(tree, &player, message);
                if let Some(dialog) = stack.as_mut() {
                    for context in close {
                        dialog.close_stack_ui(tree, context);
                    }
                }
                if let Some(request) = request {
                    open_examination |= crate::ui_inventory::external_container_request(
                        request,
                        tree,
                        stack.as_mut(),
                        &player,
                        &mut commands,
                    );
                }
            }
            // `gmStackUI::ListenToMessage`; its `StackUIDone` finishes the waiting drop.
            let done = stack
                .as_mut()
                .and_then(|d| d.message(tree, message, &player));
            if let Some(done) = done {
                let (context, amount) = (done.context, done.amount);
                let request = backpack
                    .as_mut()
                    .and_then(|p| p.stack_ui_done(&player, context, amount))
                    .or_else(|| {
                        external_container
                            .as_mut()
                            .and_then(|p| p.stack_ui_done(&player, context, amount))
                    });
                let request = request.or_else(|| {
                    let (bag, dialogs) = (money.as_mut()?, dialogs.as_mut()?);
                    bag.stack_ui_done(
                        tree,
                        &player,
                        &player,
                        dialogs,
                        context,
                        amount,
                        *shift_held,
                    )
                });
                if let Some(request) = request {
                    crate::ui_inventory::item_commands(request, &mut commands);
                }
                if let Some(panel) = secure_trade.as_mut() {
                    panel.recv_stack_ui_done(context, amount);
                }
            }
            if let Some(request) = paper_doll
                .as_mut()
                .and_then(|d| d.message(tree, &player, message))
            {
                open_examination |= crate::ui_inventory::item_commands(request, &mut commands);
            }
            if let (Some(page), Some(dialogs)) = (allegiance.as_mut(), dialogs.as_mut()) {
                page.message(tree, dialogs, &player, message);
            }
            if let Some(panel) = secure_trade.as_mut() {
                panel.message(tree, &player, message);
            }
            if let Some(dialogs) = dialogs.as_mut() {
                dialogs.message(tree, message);
            }
        }
        // `gmSecureTradeUI`'s global loop: an ended trade closes; a stack its list dropped asks.
        if let Some(panel) = secure_trade.as_mut() {
            panel.tick(tree, frame.server_time);
            for split in panel.take_splits() {
                crate::ui_inventory::secure_trade_split(split, tree, stack.as_mut(), &player);
            }
        }
        if let Some(dialogs) = dialogs.as_mut() {
            let outcomes = dialogs.take_outcomes();
            crate::ui_social::dialog_outcomes(
                tree,
                fellowship.as_mut(),
                allegiance.as_mut(),
                &outcomes,
            );
            // The melt's confirmation (`CloseConfirmationDialog`).
            if let Some(request) = money.as_mut().and_then(|b| b.take_outcome(&outcomes)) {
                crate::ui_inventory::item_commands(request, &mut commands);
            }
            // The partner's request's confirmation (`CloseConfirmationDialog`).
            if let Some(panel) = secure_trade.as_mut() {
                panel.dialog_outcomes(tree, &outcomes);
            }
        }
        // `gmSecureTradeUI`'s calls on the player (`gmCWInterface::*Trade*`).
        if let Some(panel) = secure_trade.as_mut() {
            for request in panel.take_requests() {
                use ac2_ui::SecureTradeRequest as R;
                commands.push(match request {
                    R::Open { partner } => Command::OpenTrade(partner),
                    R::Offer { item, quantity } => Command::OfferItemForTrade {
                        item,
                        amount: quantity,
                    },
                    R::Revoke { item } => Command::RevokeItemFromTrade(item),
                    R::Accept => Command::AcceptTrade,
                    R::Decline => Command::DeclineTrade,
                    R::Close => Command::CloseTrade,
                });
            }
        }
        if let Some(page) = allegiance.as_mut() {
            page.follow_show_state(tree);
            crate::ui_social::allegiance_commands(page.take_requests(), &mut commands);
        }
        let mut tells = Vec::new();
        if let Some(panel) = fellowship.as_mut() {
            let selection = frame.selection.as_ref().map(|s| s.id);
            tells = crate::ui_social::fellowship_commands(
                panel.take_requests(),
                selection,
                &mut commands,
            );
        }
        // `gmExaminationUI::Show`: the panel follows its field; opening holds the request open.
        // Legions passes the panel's filter word (+0xB8; its filter tabs are not modelled).
        if let Some(panel) = examination.as_mut() {
            panel.follow_field(tree);
            for call in panel.take_calls() {
                commands.push(match call {
                    ac2_ui::ExaminationCall::Start => Command::StartExamination(0),
                    ac2_ui::ExaminationCall::Stop => Command::StopExamination,
                });
            }
        }
        if let Some(skills) = skills.as_mut() {
            for request in skills.take_requests() {
                match request {
                    SkillRequest::Examine { kind, data } => {
                        examine(kind, data, 0, &mut commands);
                        open_examination = true;
                    }
                    // The confirmation dialogs are not modelled: confirmed at once.
                    SkillRequest::Train(skill) | SkillRequest::ConfirmTrain(skill) => {
                        commands.push(Command::Skill(ac2_weenie::SkillRequest::Train(skill)));
                    }
                    SkillRequest::Raise(skill) => {
                        commands.push(Command::Skill(ac2_weenie::SkillRequest::Raise(skill)));
                    }
                    SkillRequest::ConfirmUntrain(skill) => {
                        commands.push(Command::Skill(ac2_weenie::SkillRequest::Untrain(skill)));
                    }
                    SkillRequest::ConfirmCancelUntrain(_) => {
                        commands.push(Command::Skill(ac2_weenie::SkillRequest::CancelUntrain));
                    }
                }
            }
        }
        if let Some(bar) = shortcuts.as_mut() {
            for request in bar.take_requests() {
                let mut client = player.client.borrow_mut();
                let result = match request {
                    ShortcutRequest::Use(index) => client.use_shortcut(index as u32).map(drop),
                    ShortcutRequest::Set(index, shortcut) => client
                        .set_shortcut(index as u32, shortcut_to_info(&shortcut))
                        .map(drop),
                    ShortcutRequest::Examine { kind, data, item } => {
                        examine(kind, data, item, &mut commands);
                        open_examination = true;
                        Ok(())
                    }
                    ShortcutRequest::Attack(_) | ShortcutRequest::WhichSet(_) => Ok(()),
                };
                if let Err(error) = result {
                    eprintln!("ac2: shortcut: {error}");
                }
            }
        }
        if tree.take_changed() {
            *dirty = true;
        }
        if open_examination {
            self.open_examination();
        }
        // `FellowshipHelper::ListenToMessage`: a member's name into the chat line.
        for name in tells {
            self.assisted_tell(&name);
        }
        commands
    }
}

/// `gm3DTextUI::AddName`'s inputs: every object with a radar look, its look's colour and its
/// name.
fn name_looks(frame: &ClientFrame) -> Vec<ac2_ui::NameLook> {
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
