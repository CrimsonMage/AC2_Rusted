//! The HUD frameworks' tick pass: the panels' ticks and the dialogs' outcomes after the messages,
//! and the requests the panels queued become gameplay commands.

use ac2_ui::{ShortcutRequest, SkillRequest};
use ac2_weenie::SessionEvent;

use crate::frame::ClientFrame;
use crate::gameplay::Command;
use crate::hud_character::character_commands;
use crate::hud_chat_options::chat_options_commands;
use crate::hud_death::{death_answers, death_notices};
use crate::hud_game_dialogs::{game_dialog_answers, game_dialog_logout, game_dialog_notices};
use crate::hud_player::{examine, PassOut};
use crate::hud_view::HudView;
use crate::ui_sources::{shortcut_to_info, ClientPlayer};

/// The text type of the store's purchase messages (`TEXT_TYPE_ERROR`).
const SHOP_MESSAGE_TEXT_TYPE: u32 = 0x1;

impl HudView {
    /// The panels' ticks and global loops after the messages, the dialogs' outcomes and the calls the
    /// panels queued (`gmSecureTradeUI`, `gmPetUI`, `gmShopUI`, `gmSkillTreeUI`, `gmShortcutUI`).
    pub(crate) fn tick_pass(
        &mut self,
        player: &ClientPlayer<'_>,
        frame: &ClientFrame,
        events: &[SessionEvent],
    ) -> PassOut {
        let HudView {
            hud,
            allegiance,
            assistance,
            catalog,
            character,
            chat,
            chat_options,
            consignment,
            death,
            dialogs,
            examination,
            fellowship,
            game_dialogs,
            popup_backlog,
            money,
            new_sale,
            options,
            pets,
            secure_trade,
            shop,
            shortcuts,
            skills,
            stack,
            toolbar,
            ..
        } = self;
        let Some((tree, _)) = hud else {
            return PassOut::default();
        };
        let mut commands = Vec::new();
        let mut open_examination = false;
        // `gmToolbarUI::ManageChildren`: the buttons' tooltips follow the children's shown states.
        if let Some(bar) = toolbar.as_ref() {
            bar.refresh_tooltips(tree, player);
        }
        // `gmSecureTradeUI`'s global loop: an ended trade closes; a stack its list dropped asks.
        if let Some(panel) = secure_trade.as_mut() {
            panel.tick(tree, frame.server_time);
            for split in panel.take_splits() {
                crate::ui_inventory::secure_trade_split(split, tree, stack.as_mut(), player);
            }
        }
        death_notices(death, tree, dialogs, player, events);
        game_dialog_notices(game_dialogs, tree, dialogs, player, events, popup_backlog);
        game_dialog_logout(game_dialogs, death, tree, dialogs, player, &mut commands);
        if let Some(dialogs) = dialogs.as_mut() {
            let outcomes = dialogs.take_outcomes();
            death_answers(death, &outcomes, &mut commands);
            game_dialog_answers(
                game_dialogs,
                death,
                tree,
                dialogs,
                player,
                &outcomes,
                &mut commands,
            );
            crate::ui_social::dialog_outcomes(
                tree,
                fellowship.as_mut(),
                allegiance.as_mut(),
                &outcomes,
            );
            if let Some(page) = assistance.as_mut() {
                crate::hud_assistance::assistance_outcomes(page, &outcomes);
            }
            // The train, untrain and cancel confirmations (`gmSkillTreeUI::RecvNotice_CloseConfirmationDialog`).
            if let Some(skills) = skills.as_mut() {
                skills.dialog_outcomes(&outcomes);
            }
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
            // The sale's confirmation (`gmShopNewSaleUI::RecvNotice_CloseConfirmationDialog`).
            if let Some(panel) = new_sale.as_mut() {
                panel.dialog_outcomes(tree, dialogs, &outcomes, frame.server_time);
                commands.extend(crate::hud_shop::new_sale_commands(panel.take_requests()));
            }
            // The restock's confirmation (`gmShopConsignmentUI::RecvNotice_CloseConfirmationDialog`).
            if let Some(panel) = consignment.as_mut() {
                panel.dialog_outcomes(&outcomes);
                open_examination |= crate::hud_shop::consignment_requests(
                    panel.take_requests(),
                    new_sale.as_mut(),
                    tree,
                    dialogs,
                    player,
                    frame.server_time,
                    &mut commands,
                );
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
            // The client page's questions and its requests (`gmClientUI::Apply`, Recall, Default).
            options.outcomes(tree, dialogs, &outcomes);
            options.run(&mut player.client.borrow_mut(), tree, dialogs);
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
        // `gmPetUI::ListenToMessage` on the refresh message: each pass asks for the rows' wanted modes.
        if let Some(panel) = pets.as_mut() {
            let actions = panel.tick(tree, frame.server_time);
            crate::hud_pets::pet_commands(actions, &mut commands);
        }
        // `gmShopUI`'s global loop, its stack dialog's calls, its calls on the player and the
        // purchase messages for the chat (`CM_Character::SendNotice_DisplayMessage`).
        if let Some(panel) = catalog.as_mut() {
            panel.tick(tree, frame.server_time);
            commands.extend(crate::hud_shop::catalog_commands(panel.take_requests()));
        }
        if let Some(panel) = consignment.as_mut() {
            panel.tick(tree, frame.server_time);
        }
        if let Some(panel) = new_sale.as_mut() {
            panel.tick(tree, frame.server_time);
            commands.extend(crate::hud_shop::new_sale_commands(panel.take_requests()));
            crate::hud_shop::shop_stack_calls(panel.take_stack_calls(), tree, stack.as_mut());
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
        if let (Some(skills), Some(dialogs)) = (skills.as_mut(), dialogs.as_mut()) {
            skills.follow_show_state(tree, dialogs);
        }
        if let Some(skills) = skills.as_mut() {
            for request in skills.take_requests() {
                match request {
                    SkillRequest::Examine { kind, data } => {
                        examine(kind, data, 0, &mut commands);
                        open_examination = true;
                    }
                    SkillRequest::Train(skill) => {
                        commands.push(Command::Skill(ac2_weenie::SkillRequest::Train(skill)));
                    }
                    SkillRequest::Raise(skill) => {
                        commands.push(Command::Skill(ac2_weenie::SkillRequest::Raise(skill)));
                    }
                    SkillRequest::Untrain(skill) => {
                        commands.push(Command::Skill(ac2_weenie::SkillRequest::Untrain(skill)));
                    }
                    SkillRequest::CancelUntrain(_) => {
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
        PassOut {
            commands,
            open_examination,
            tells,
        }
    }
}
