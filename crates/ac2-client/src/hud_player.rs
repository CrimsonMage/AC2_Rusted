//! The HUD frameworks that ask the player, run once a frame with the client: the skills panel
//! (`gmSkillTreeUI`) and the shortcut bar (`gmShortcutUI`) take the frame's notices
//! (`RecvNotice_*`) and the tree's messages, ask the player's script and the session through
//! [`ClientPlayer`], and their requests become the player's calls and gameplay commands. The chat
//! takes the player's chat options when they are reloaded.

use std::cell::RefCell;

use ac2_net::protocol::ExaminationRequest;
use ac2_script::host::ScriptNotice;
use ac2_ui::{ChatSettings, ShortcutRequest, SkillRequest, PROFILE_LATENCY_INDICATOR_GRAPH};
use ac2_weenie::{GameplayNotice, SessionEvent};

use crate::client::Client;
use crate::frame::ClientFrame;
use crate::gameplay::Command;
use crate::hud_character::character_commands;
use crate::hud_chat_options::{chat_options_commands, ui_chat_settings};
use crate::hud_death::{death_answers, death_notices};
use crate::hud_game_dialogs::{
    game_dialog_answers, game_dialog_logout, game_dialog_message, game_dialog_notices,
};
use crate::hud_status::StatusBar;
use crate::hud_view::HudView;
use crate::ui_sources::{shortcut_to_info, ClientPlayer};

/// The player's race (int 290, `UpdatePaperDollPosition`).
const RACE: u32 = 0x122;

/// The player's chat options as the chat windows and the chat options page take them.
/// The text type of the store's purchase messages (`TEXT_TYPE_ERROR`).
const SHOP_MESSAGE_TEXT_TYPE: u32 = 0x1;

fn chat_settings(client: &mut Client) -> Option<ChatSettings> {
    Some(ui_chat_settings(&client.chat_settings().ok()?))
}

/// The options profile as the options page reads it (`gmCWInterface::Get*`).
fn options_settings(client: &mut Client) -> Option<ac2_ui::OptionsSettings> {
    let p = client.gameplay_options().ok()?;
    Some(ac2_ui::OptionsSettings {
        bits: p.bit_field,
        radar_mask: p.radar_mask,
        shortcut_height: p.shortcut_height,
        damage_text_range_other: p.damage_text_range_other,
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
            mini_fellow,
            quests,
            allegiance,
            vitals,
            effects,
            vitae,
            net,
            dialogs,
            death,
            game_dialogs,
            stack,
            external_container,
            secure_trade,
            shop,
            catalog,
            character,
            chat_options,
            money,
            shift_held,
            backpack,
            items_3d,
            paper_doll,
            player_race,
            toolbar_layout,
            item_selection,
            radar_cell,
            radar_coordinates,
            navigation_time,
            radar,
            names,
            damage,
            now,
            toolbar,
            ..
        } = self;
        let Some((tree, _)) = hud else {
            return commands;
        };
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
        // `gmRadarUI::ReloadOptions` (at login, `Init`, and on `RecvNotice_ReloadOptions`).
        let reload_radar = !*player_loaded
            || script_notices
                .iter()
                .any(|n| n.name == "CM_Character::SendNotice_ReloadOptions");
        let show_radar_coords = reload_radar.then(|| client.show_radar_coords());
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
            character_commands(page, &mut commands);
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
        // `gmNavigationUI::UpdateMacroMap`'s date and time: rendered again when the calendar moves.
        if let Some(date_time) = frame.date_time {
            if navigation_time.as_ref().map(|(known, _, _)| *known) != Some(date_time) {
                let (date, time) = crate::ui_sources::game_date_time(date_time);
                *navigation_time = Some((
                    date_time,
                    ac2_ui::UiStrings::render_string(&player, &date),
                    ac2_ui::UiStrings::render_string(&player, &time),
                ));
            }
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
            // `gmVitalsUI`, `gmEffectsUI` (`Init`) and `gmVitaeStatusUI` read the player.
            StatusBar {
                vitals: &mut *vitals,
                effects: &mut *effects,
                vitae: &mut *vitae,
                net: &mut *net,
            }
            .player_loaded(tree, &player, &frame.player_vitals, frame.server_time);
            // `gmInventoryUI::SetupChildren`'s backpack over the player.
            if let (Some(layout), Some(id)) =
                (toolbar_layout.as_ref(), frame.player.as_ref().map(|p| p.id))
            {
                *item_selection = ac2_ui::InventorySource::selection(&player);
                *backpack = Some(ac2_ui::BackpackUi::create(tree, layout, &player, id));
                *items_3d = Some(ac2_ui::Items3dUi::create(tree, id));
                // `gmMoneyBagUI::gmMoneyBagUI` and its amount (`RecvNotice_UpdateMoney`).
                let mut bag = ac2_ui::MoneyBagUi::create(tree);
                bag.set_money(tree, &player, frame.inventory.money.max(0) as u32);
                *money = Some(bag);
                let equipped = player.equipped();
                let doll = ac2_ui::PaperDollUi::create(tree, layout, &player, id, &equipped);
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
            // `gmNetStatusUI::OnHeartbeat`: the link averages the session closed this heartbeat.
            if let SessionEvent::LinkHeartbeat(links) = event {
                StatusBar {
                    vitals: &mut *vitals,
                    effects: &mut *effects,
                    vitae: &mut *vitae,
                    net: &mut *net,
                }
                .link_heartbeat(tree, &player, links);
                // The bandwidth graph moved: the status bar is composed again.
                *dirty = true;
                continue;
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
            if let (Some(panel), Some(dialogs)) = (shop.as_mut(), dialogs.as_mut()) {
                crate::hud_shop::shop_notice(
                    panel,
                    tree,
                    dialogs,
                    &player,
                    frame.server_time,
                    notice,
                );
            }
            if let Some(panel) = catalog.as_mut() {
                crate::hud_shop::catalog_notice(panel, tree, &player, frame.server_time, notice);
            }
            if let Some(dialogs) = dialogs.as_mut() {
                crate::ui_social::fellowship_notice(
                    fellowship.as_mut(),
                    mini_fellow.as_mut(),
                    tree,
                    dialogs,
                    &player,
                    notice,
                );
            }
            if let Some(panel) = quests.as_mut() {
                crate::ui_quests::quest_notice(panel, tree, &player, notice);
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
            StatusBar {
                vitals: &mut *vitals,
                effects: &mut *effects,
                vitae: &mut *vitae,
                net: &mut *net,
            }
            .notice(tree, &player, frame.server_time, notice);
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
                // `gmShortcutUI::RecvNotice_UpdateResetTime`: CheckAllSkills.
                GameplayNotice::UpdateResetTime => check_shortcuts = true,
                // `gmShortcutUI::RecvNotice_ToggleSpecialAttack`, `RequestSpecialAttack`,
                // `StartSpecialAttack` and `EndSpecialAttack`.
                GameplayNotice::ToggleSpecialAttack { maneuver, toggled } => {
                    if let Some(bar) = shortcuts.as_mut() {
                        bar.toggle_special_attack(tree, *maneuver, *toggled);
                    }
                }
                GameplayNotice::RequestSpecialAttack { maneuver, attack } => {
                    if let Some(bar) = shortcuts.as_mut() {
                        bar.request_special_attack(tree, *attack, *maneuver);
                    }
                }
                GameplayNotice::StartSpecialAttack(attack) => {
                    if let Some(bar) = shortcuts.as_mut() {
                        bar.start_special_attack(tree, *attack);
                    }
                }
                GameplayNotice::EndSpecialAttack(attack) => {
                    if let Some(bar) = shortcuts.as_mut() {
                        bar.end_special_attack(tree, *attack);
                    }
                }
                GameplayNotice::UpdateMoney(amount) => {
                    if let Some(bag) = money.as_mut() {
                        bag.set_money(tree, &player, *amount);
                    }
                }
                // `gmToolbarUI::RecvNotice_ShowSaleReminder` and `RecvNotice_UpdateMountState`.
                GameplayNotice::ShowSaleReminder(show) => {
                    if let Some(bar) = toolbar.as_ref() {
                        bar.sale_reminder(tree, *show);
                    }
                }
                GameplayNotice::UpdateMountState(on_mount) => {
                    if let Some(bar) = toolbar.as_ref() {
                        bar.mount_state(tree, *on_mount);
                    }
                }
                // `gmSelectionUI::RecvNotice_Select` (`target_select::frame_notices`).
                GameplayNotice::Select(_) => {}
                // `gm3DTextUI::RecvNotice_Do3DDamage`.
                GameplayNotice::Do3DDamage { object, text } => {
                    if let Some(damage) = damage.as_mut() {
                        damage.add(tree, *object, text, *now);
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
        // `gmQuestsUI::Show` when the toolbar's Quest child changed, then
        // `gmJournalQuestsUI::ListenToMessage` on the global loop: the pages laid out, and the
        // time displays counted down.
        if let Some(panel) = quests.as_mut() {
            if let Some(bar) = toolbar.as_ref() {
                panel.sync(tree, bar.shown(tree, ac2_ui::ToolbarChild::Quest));
            }
            panel.journal_mut().global_loop(tree, &player, *now);
        }
        for message in &messages {
            // `gmSelectionUI::ListenToMessage`: a target key selects (the cycling keys).
            // `gmToolbarUI::ListenToMessage`: the dismount button calls `gmCWInterface::Dismount`.
            if toolbar
                .as_ref()
                .is_some_and(|bar| bar.is_dismount_press(message))
            {
                commands.push(Command::Dismount);
            }
            if let Some(key) = ac2_ui::select_key_of(message) {
                let mut client = player.client.borrow_mut();
                let typing = tree.text_mode();
                commands.extend(crate::target_select::select_commands(
                    &mut client,
                    frame,
                    key,
                    typing,
                ));
            }
            if let (Some(page), Some(dialogs)) = (character.as_mut(), dialogs.as_mut()) {
                page.message(tree, dialogs, message);
                character_commands(page, &mut commands);
            }
            if let (Some(page), Some(dialogs)) = (chat_options.as_mut(), dialogs.as_mut()) {
                page.message(tree, dialogs, message);
                chat_options_commands(page, &mut commands);
            }
            if let Some(skills) = skills.as_mut() {
                skills.message(tree, &player, message);
            }
            if let Some(bar) = shortcuts.as_mut() {
                bar.message(tree, &player, message);
            }
            if let Some(panel) = fellowship.as_mut() {
                panel.message(tree, message);
            }
            if let Some(panel) = mini_fellow.as_mut() {
                panel.message(tree, message);
            }
            if let Some(panel) = quests.as_mut() {
                panel.journal_mut().message(tree, message);
            }
            if let Some(bar) = vitals.as_mut() {
                bar.message(tree, message, &player);
            }
            // `gmVitaeStatusUI::ListenToMessage`: examine the vitae.
            if vitae.as_mut().is_some_and(|v| v.message(tree, message)) {
                examine(ac2_ui::EXAMINE_VITAE, 0, 0, &mut commands);
                open_examination = true;
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
                let request = request.or_else(|| items_3d.as_mut()?.stack_ui_done(context, amount));
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
                if let Some(panel) = shop.as_mut() {
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
            // `gmShopUI::ListenToMessage`; a purchase asks the player how many it may buy.
            if let (Some(panel), Some(dialogs)) = (shop.as_mut(), dialogs.as_mut()) {
                let mut most = |item| {
                    player
                        .client
                        .borrow_mut()
                        .max_purchase_quantity(item)
                        .unwrap_or(0)
                };
                panel.message(
                    tree,
                    &player,
                    dialogs,
                    message,
                    frame.server_time,
                    &mut most,
                );
            }
            game_dialog_message(game_dialogs, message);
            // `gmShopCatalogUI::ListenToMessage`: the catalog's buttons and rows.
            if let Some(panel) = catalog.as_mut() {
                panel.message(tree, message, frame.server_time);
                commands.extend(crate::hud_shop::catalog_commands(panel.take_requests()));
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
        death_notices(death, tree, dialogs, &player, events);
        game_dialog_notices(game_dialogs, tree, dialogs, &player, events);
        game_dialog_logout(game_dialogs, death, tree, dialogs, &player, &mut commands);
        if let Some(dialogs) = dialogs.as_mut() {
            let outcomes = dialogs.take_outcomes();
            death_answers(death, &outcomes, &mut commands);
            game_dialog_answers(
                game_dialogs,
                death,
                tree,
                dialogs,
                &player,
                &outcomes,
                &mut commands,
            );
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
            // The purchase's confirmation (`gmShopUI::RecvNotice_CloseConfirmationDialog`).
            if let Some(panel) = shop.as_mut() {
                panel.dialog_outcomes(&outcomes);
            }
            // The recall's and default's confirmations (`gmCharacterUI::RecvNotice_CloseConfirmationDialog`).
            if let Some(page) = character.as_mut() {
                page.dialog_outcomes(tree, dialogs, &outcomes);
                character_commands(page, &mut commands);
            }
            // The chat options' confirmations (`gmChatOptionsUI::RecvNotice_CloseConfirmationDialog`).
            if let Some(page) = chat_options.as_mut() {
                page.dialog_outcomes(tree, dialogs, &outcomes);
                chat_options_commands(page, &mut commands);
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
        // `gmShopUI`'s global loop, its stack dialog's calls, its calls on the player and the
        // purchase messages for the chat (`CM_Character::SendNotice_DisplayMessage`).
        if let Some(panel) = catalog.as_mut() {
            panel.tick(tree, frame.server_time);
            commands.extend(crate::hud_shop::catalog_commands(panel.take_requests()));
        }
        if let Some(panel) = shop.as_mut() {
            panel.tick(tree, frame.server_time);
            let requests = panel.take_requests();
            open_examination |= requests.contains(&ac2_ui::ShopRequest::OpenExamination);
            commands.extend(crate::hud_shop::shop_commands(requests));
            crate::hud_shop::shop_stack_calls(panel.take_stack_calls(), tree, stack.as_mut());
            if let Some(chat) = chat.as_mut() {
                for text in panel.take_messages() {
                    chat.display(SHOP_MESSAGE_TEXT_TYPE, &text);
                }
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
