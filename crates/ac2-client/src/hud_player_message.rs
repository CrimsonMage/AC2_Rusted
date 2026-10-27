//! The HUD frameworks' message pass: the frame's tree messages reach the panels in the order the
//! frameworks listen to them, and the panels' requests become gameplay commands.

use ac2_ui::UiMessage;

use crate::frame::ClientFrame;
use crate::gameplay::Command;
use crate::hud_character::character_commands;
use crate::hud_chat_options::chat_options_commands;
use crate::hud_game_dialogs::game_dialog_message;
use crate::hud_player::{examine, name_looks, PassOut};
use crate::hud_view::HudView;
use crate::ui_sources::ClientPlayer;

impl HudView {
    /// The quests' global loop, then the frame's messages to the panels in the frameworks' order
    /// (`ListenToMessage`).
    pub(crate) fn message_pass(
        &mut self,
        player: &ClientPlayer<'_>,
        frame: &ClientFrame,
        messages: &[UiMessage],
    ) -> PassOut {
        let HudView {
            hud,
            allegiance,
            assistance,
            backpack,
            catalog,
            character,
            chat_options,
            consignment,
            dialogs,
            effects,
            external_container,
            fellowship,
            friends,
            game_dialogs,
            items_3d,
            mini_fellow,
            money,
            names,
            options,
            new_sale,
            now,
            paper_doll,
            pets,
            quests,
            radar,
            secure_trade,
            shift_held,
            shop,
            shortcuts,
            skills,
            social,
            stack,
            toolbar,
            toolbar_layout,
            vitae,
            vitals,
            ..
        } = self;
        let Some((tree, _)) = hud else {
            return PassOut::default();
        };
        let mut commands = Vec::new();
        let mut open_examination = false;
        // `gmQuestsUI::Show` when the toolbar's Quest child changed, then
        // `gmJournalQuestsUI::ListenToMessage` on the global loop: the pages laid out, and the
        // time displays counted down.
        if let Some(panel) = quests.as_mut() {
            if let Some(bar) = toolbar.as_ref() {
                panel.sync(tree, bar.shown(tree, ac2_ui::ToolbarChild::Quest));
            }
            panel.journal_mut().global_loop(tree, player, *now);
            panel.story_global_loop(tree, player, || {
                player.client.borrow_mut().inq_act_info_list()
            });
        }
        // `gmClientUI`'s widgets, once the HUD and the preferences are there.
        options.make(&player.client.borrow(), tree, toolbar_layout.as_ref());
        // `TimedDialog`'s countdown: a question run out answers no.
        if let Some(dialogs) = dialogs.as_mut() {
            dialogs.tick(tree, *now);
        }
        for message in messages {
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
            if let Some(dialogs) = dialogs.as_mut() {
                options.message(&mut player.client.borrow_mut(), tree, dialogs, message);
            }
            if let (Some(skills), Some(dialogs)) = (skills.as_mut(), dialogs.as_mut()) {
                skills.message(tree, dialogs, player, message);
            }
            if let Some(bar) = shortcuts.as_mut() {
                bar.message(tree, player, message);
            }
            if let Some(panel) = fellowship.as_mut() {
                panel.message(tree, message);
            }
            if let Some(panel) = mini_fellow.as_mut() {
                panel.message(tree, message);
            }
            if let Some(panel) = pets.as_mut() {
                let selection = frame.selection.as_ref().map(|s| s.id);
                let actions = panel.message(tree, message, selection);
                crate::hud_pets::pet_commands(actions, &mut commands);
            }
            if let Some(panel) = friends.as_mut() {
                let target = crate::hud_friends::friend_target(frame.selection.as_ref());
                crate::hud_friends::friends_message(panel, tree, message, target, &mut commands);
            }
            if let Some(panel) = social.as_mut() {
                // The help page is not modelled: the panel's `OpenHelp` is dropped.
                let _ = panel.message(tree, message);
            }
            if let (Some(page), Some(dialogs)) = (assistance.as_mut(), dialogs.as_mut()) {
                crate::hud_assistance::assistance_message(page, tree, dialogs, player, message);
            }
            if let Some(panel) = quests.as_mut() {
                panel.journal_mut().message(tree, message);
                if let Some(action) = panel.story_message(tree, message) {
                    crate::ui_quests::play_story_scene(panel.story(), action, player);
                }
            }
            if let Some(bar) = vitals.as_mut() {
                bar.message(tree, message, player);
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
                .and_then(|r| r.message(tree, player, message))
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
                .and_then(|p| p.message(tree, player, message))
            {
                open_examination |= crate::ui_inventory::item_request(
                    request,
                    tree,
                    stack.as_mut(),
                    player,
                    &mut commands,
                );
            }
            // `gmMoneyBagUI::ListenToMessage`: a drop on the bag.
            if let (Some(bag), Some(dialogs)) = (money.as_mut(), dialogs.as_mut()) {
                if let Some(request) =
                    bag.message(tree, player, player, dialogs, message, *shift_held)
                {
                    open_examination |= crate::ui_inventory::item_request(
                        request,
                        tree,
                        stack.as_mut(),
                        player,
                        &mut commands,
                    );
                }
            }
            if let Some(panel) = external_container.as_mut() {
                let (request, close) = panel.message(tree, player, message);
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
                        player,
                        &mut commands,
                    );
                }
            }
            // `gmStackUI::ListenToMessage`; its `StackUIDone` finishes the waiting drop.
            let done = stack
                .as_mut()
                .and_then(|d| d.message(tree, message, player));
            if let Some(done) = done {
                let (context, amount) = (done.context, done.amount);
                let request = backpack
                    .as_mut()
                    .and_then(|p| p.stack_ui_done(player, context, amount))
                    .or_else(|| {
                        external_container
                            .as_mut()
                            .and_then(|p| p.stack_ui_done(player, context, amount))
                    });
                let request = request.or_else(|| items_3d.as_mut()?.stack_ui_done(context, amount));
                let request = request.or_else(|| {
                    let (bag, dialogs) = (money.as_mut()?, dialogs.as_mut()?);
                    bag.stack_ui_done(tree, player, player, dialogs, context, amount, *shift_held)
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
                if let Some(panel) = new_sale.as_mut() {
                    panel.recv_stack_ui_done(tree, player, context, amount);
                }
                if let (Some(panel), Some(dialogs)) = (consignment.as_mut(), dialogs.as_mut()) {
                    panel.recv_stack_ui_done(tree, dialogs, player, context, amount);
                    open_examination |= crate::hud_shop::consignment_requests(
                        panel.take_requests(),
                        new_sale.as_mut(),
                        tree,
                        dialogs,
                        player,
                        frame.server_time,
                        &mut commands,
                    );
                    crate::hud_shop::shop_stack_calls(
                        panel.take_stack_calls(),
                        tree,
                        stack.as_mut(),
                    );
                }
            }
            if let Some(request) = paper_doll
                .as_mut()
                .and_then(|d| d.message(tree, player, message))
            {
                open_examination |= crate::ui_inventory::item_commands(request, &mut commands);
            }
            if let (Some(page), Some(dialogs)) = (allegiance.as_mut(), dialogs.as_mut()) {
                page.message(tree, dialogs, player, message);
            }
            if let Some(panel) = secure_trade.as_mut() {
                panel.message(tree, player, message);
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
                panel.message(tree, player, dialogs, message, frame.server_time, &mut most);
            }
            game_dialog_message(game_dialogs, message);
            // `gmShopCatalogUI::ListenToMessage`: the catalog's buttons and rows.
            if let Some(panel) = catalog.as_mut() {
                panel.message(tree, message, frame.server_time);
                commands.extend(crate::hud_shop::catalog_commands(panel.take_requests()));
            }
            // `gmShopConsignmentUI::ListenToMessage`: the consignment's buttons, rows and drop.
            if let (Some(panel), Some(dialogs)) = (consignment.as_mut(), dialogs.as_mut()) {
                panel.message(tree, dialogs, player, message, frame.server_time);
                open_examination |= crate::hud_shop::consignment_requests(
                    panel.take_requests(),
                    new_sale.as_mut(),
                    tree,
                    dialogs,
                    player,
                    frame.server_time,
                    &mut commands,
                );
                crate::hud_shop::shop_stack_calls(panel.take_stack_calls(), tree, stack.as_mut());
            }
            // `gmShopNewSaleUI::ListenToMessage`: the sale's buttons and the drop on its slot.
            if let (Some(panel), Some(dialogs)) = (new_sale.as_mut(), dialogs.as_mut()) {
                let mut charge = || {
                    player
                        .client
                        .borrow_mut()
                        .consignment_charge()
                        .unwrap_or(0.0)
                };
                panel.message(
                    tree,
                    dialogs,
                    player,
                    message,
                    frame.server_time,
                    &mut charge,
                );
                commands.extend(crate::hud_shop::new_sale_commands(panel.take_requests()));
                crate::hud_shop::shop_stack_calls(panel.take_stack_calls(), tree, stack.as_mut());
            }
            if let Some(dialogs) = dialogs.as_mut() {
                dialogs.message(tree, message);
            }
        }
        PassOut {
            commands,
            open_examination,
            ..PassOut::default()
        }
    }
}
