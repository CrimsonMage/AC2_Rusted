//! The HUD frameworks' tick pass: the panels' ticks and global loops after the messages, the
//! dialogs' outcomes, and the calls the panels queued. The panels are the HUD's ([`Hud`]); the
//! requests come back as [`HudRequest`]s in the order the panels raised them, and the client maps
//! them to gameplay commands.

use ac2_weenie::SessionEvent;

use crate::{
    allegiance_calls, assistance_outcomes, consignment_requests, death_notices, fellowship_calls,
    shop_stack_calls, social_dialog_outcomes, CatalogRequest, CatalogUi, ConsignmentUi,
    DialogRequest, Hud, HudRequest, HudSource, NewSaleUi, OptionsHost, ShopRequest, ShopUi,
    ShortcutCalls, ShortcutRequest, UiTree,
};

/// The text type of the store's purchase messages (`TEXT_TYPE_ERROR`).
const SHOP_MESSAGE_TEXT_TYPE: u32 = 0x1;

/// The frame's values the tick pass takes beside the HUD and its tree: the server's time, the
/// selection, the client side of `gmClientUI` and the client's shortcut calls.
pub struct TickInputs<'a> {
    /// The server's time at this frame.
    pub server_time: f64,
    /// The selected object's id, for the fellowship page.
    pub selection: Option<u64>,
    /// The client side of `gmClientUI` (its preferences and live applies).
    pub options_host: &'a mut dyn OptionsHost,
    /// The client's shortcut calls (`gmShortcutUI`'s use and set).
    pub shortcut_calls: &'a mut dyn ShortcutCalls,
}

/// One frame's tick pass; returns the requests the panels raised, in order. The panels are the
/// HUD's ([`Hud`]).
pub fn tick_pass<P: HudSource>(
    hud: &mut Hud,
    tree: &mut UiTree,
    inputs: TickInputs<'_>,
    player: &P,
    events: &[SessionEvent],
) -> Vec<HudRequest> {
    let TickInputs {
        server_time,
        selection,
        options_host,
        shortcut_calls,
    } = inputs;
    let Hud {
        popup_backlog,
        display_backlog,
        options,
        game_dialogs,
        death,
        dialogs,
        toolbar,
        secure_trade,
        stack,
        money,
        pets,
        catalog,
        consignment,
        new_sale,
        shop,
        chat,
        allegiance,
        fellowship,
        assistance,
        character,
        chat_options,
        examination,
        option_panel,
        skills,
        shortcuts,
        ..
    } = hud;
    let mut out = Vec::new();
    // `gmToolbarUI::ManageChildren`: the buttons' tooltips follow the children's shown states.
    if let Some(bar) = toolbar.as_ref() {
        bar.refresh_tooltips(tree, player);
        // The modern cluster's parts beside the shortcut bar follow it when it moved or resized.
        bar.follow_shortcut_bar(tree);
    }
    // `gmSecureTradeUI`'s global loop: an ended trade closes; a stack its list dropped asks.
    if let Some(panel) = secure_trade.as_mut() {
        panel.tick(tree, server_time);
        for split in panel.take_splits() {
            crate::secure_trade_split(split, tree, stack.as_mut(), player);
        }
    }
    // `gmMoneyBagUI_DDCB`'s inventory part, for the copy being dragged (read when it is dropped).
    if money.is_some() {
        let accepts = tree
            .dragged()
            .is_some_and(|copy| crate::MoneyBagUi::drag_accepts(tree, player, copy));
        tree.set_money_bag_accepts(accepts);
    }
    let mut shown = Vec::new();
    death_notices(
        death,
        tree,
        dialogs,
        player,
        events,
        game_dialogs.wait_context() != 0,
        &mut shown,
    );
    out.extend(shown.into_iter().map(HudRequest::GameplayShown));
    if let Some(dialogs) = dialogs.as_mut() {
        game_dialogs.frame_notices(
            tree,
            dialogs,
            player,
            events,
            popup_backlog,
            display_backlog,
        );
    }
    if let Some(dialogs) = dialogs.as_mut() {
        if game_dialogs.end_session_loop(tree, dialogs, player, death) {
            leave_open_stores(shop, consignment, new_sale, catalog, &mut out);
            out.push(HudRequest::LogOff);
        }
    }
    // `gmOptionUI::Show(0)`: closing the panel with changes asks first.
    if let (Some(panel), Some(dialogs)) = (option_panel.as_mut(), dialogs.as_mut()) {
        if panel.closing(tree) {
            let changed = character.as_ref().is_some_and(|p| p.changed())
                || options.changed(options_host)
                || chat_options.as_ref().is_some_and(|p| p.changed());
            panel.close(tree, dialogs, player, changed);
        }
    }
    if let Some(dialogs) = dialogs.as_mut() {
        let outcomes = dialogs.take_outcomes();
        // `gmOptionUI::RecvNotice_CloseConfirmationDialog`.
        if let Some(panel) = option_panel.as_mut() {
            for action in panel.dialog_outcomes(tree, &outcomes) {
                match action {
                    crate::OptionAction::Confirm(yes) => {
                        if let Some(page) = character.as_mut() {
                            if yes {
                                page.apply(tree);
                            } else {
                                page.recall(tree);
                            }
                        }
                        if let Some(page) = chat_options.as_mut() {
                            if yes {
                                page.apply(tree);
                            } else {
                                page.recall(tree);
                            }
                        }
                        options.confirm_close(yes);
                    }
                    crate::OptionAction::Logoff => {
                        leave_open_stores(shop, consignment, new_sale, catalog, &mut out);
                        out.push(HudRequest::Quit);
                    }
                    crate::OptionAction::EndCharacterSession => game_dialogs.logout_key(),
                }
            }
        }
        if death.take_resurrect(&outcomes) {
            out.push(HudRequest::Dialog(DialogRequest::RequestResurrect));
        }
        out.extend(
            game_dialogs
                .frame_answers(death, tree, dialogs, player, &outcomes)
                .into_iter()
                .map(HudRequest::Dialog),
        );
        social_dialog_outcomes(tree, fellowship.as_mut(), allegiance.as_mut(), &outcomes);
        if let Some(page) = assistance.as_mut() {
            assistance_outcomes(page, &outcomes);
        }
        // The train, untrain and cancel confirmations (`gmSkillTreeUI::RecvNotice_CloseConfirmationDialog`).
        if let Some(skills) = skills.as_mut() {
            skills.dialog_outcomes(&outcomes);
        }
        // The melt's confirmation (`CloseConfirmationDialog`).
        if let Some(request) = money.as_mut().and_then(|b| b.take_outcome(&outcomes)) {
            out.push(HudRequest::Item(request));
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
            panel.dialog_outcomes(tree, dialogs, &outcomes, server_time);
            out.extend(panel.take_requests().into_iter().map(HudRequest::NewSale));
        }
        // The restock's confirmation (`gmShopConsignmentUI::RecvNotice_CloseConfirmationDialog`).
        if let Some(panel) = consignment.as_mut() {
            panel.dialog_outcomes(&outcomes);
            consignment_requests(
                panel.take_requests(),
                new_sale.as_mut(),
                tree,
                dialogs,
                player,
                server_time,
                &mut out,
            );
        }
        // The recall's and default's confirmations (`gmCharacterUI::RecvNotice_CloseConfirmationDialog`).
        if let Some(page) = character.as_mut() {
            page.dialog_outcomes(tree, dialogs, &outcomes);
            out.extend(page.take_requests().into_iter().map(HudRequest::Character));
        }
        // The chat options' confirmations (`gmChatOptionsUI::RecvNotice_CloseConfirmationDialog`).
        if let Some(page) = chat_options.as_mut() {
            page.dialog_outcomes(tree, dialogs, &outcomes);
            out.extend(page.take_requests().into_iter().map(HudRequest::ChatOption));
        }
        // The client page's questions and its requests (`gmClientUI::Apply`, Recall, Default).
        options.outcomes(tree, dialogs, &outcomes);
        options.run(options_host, tree, dialogs);
    }
    // `gmSecureTradeUI`'s calls on the player (`gmCWInterface::*Trade*`).
    if let Some(panel) = secure_trade.as_mut() {
        out.extend(
            panel
                .take_requests()
                .into_iter()
                .map(HudRequest::SecureTrade),
        );
    }
    // `gmPetUI::ListenToMessage` on the refresh message: each pass asks for the rows' wanted modes.
    if let Some(panel) = pets.as_mut() {
        out.extend(
            panel
                .tick(tree, server_time)
                .into_iter()
                .map(HudRequest::Pet),
        );
    }
    // `gmShopUI`'s global loop, its stack dialog's calls, its calls on the player and the purchase
    // messages for the chat (`CM_Character::SendNotice_DisplayMessage`).
    if let Some(panel) = catalog.as_mut() {
        panel.tick(tree, server_time);
        out.extend(panel.take_requests().into_iter().map(HudRequest::Catalog));
    }
    if let Some(panel) = consignment.as_mut() {
        panel.tick(tree, server_time);
    }
    if let Some(panel) = new_sale.as_mut() {
        panel.tick(tree, server_time);
        out.extend(panel.take_requests().into_iter().map(HudRequest::NewSale));
        shop_stack_calls(panel.take_stack_calls(), tree, stack.as_mut());
    }
    if let Some(panel) = shop.as_mut() {
        panel.tick(tree, server_time);
        for request in panel.take_requests() {
            out.push(match request {
                ShopRequest::OpenExamination => HudRequest::OpenExamination,
                other => HudRequest::Shop(other),
            });
        }
        shop_stack_calls(panel.take_stack_calls(), tree, stack.as_mut());
        if let Some(chat) = chat.as_mut() {
            for text in panel.take_messages() {
                chat.display(SHOP_MESSAGE_TEXT_TYPE, &text);
            }
        }
    }
    if let Some(page) = allegiance.as_mut() {
        page.follow_show_state(tree);
        out.extend(
            allegiance_calls(page.take_requests())
                .into_iter()
                .map(HudRequest::Social),
        );
    }
    if let Some(panel) = fellowship.as_mut() {
        out.extend(
            fellowship_calls(panel.take_requests(), selection)
                .into_iter()
                .map(HudRequest::Social),
        );
    }
    // `gmExaminationUI::Show`: the panel follows its field; opening holds the request open.
    // Legions passes the panel's filter word (+0xB8; `examination_tabs`).
    if let Some(panel) = examination.as_mut() {
        panel.follow_field(tree);
        // The countdown rows' `TimeDisplay::ListenToMessage` on the global loop.
        panel.global_loop(tree, player, server_time);
        out.extend(panel.take_calls().into_iter().map(HudRequest::Examination));
    }
    if let (Some(skills), Some(dialogs)) = (skills.as_mut(), dialogs.as_mut()) {
        skills.follow_show_state(tree, dialogs);
    }
    if let Some(skills) = skills.as_mut() {
        out.extend(skills.take_requests().into_iter().map(HudRequest::Skill));
    }
    if let Some(bar) = shortcuts.as_mut() {
        bar.resize_loop(tree, player);
        for request in bar.take_requests() {
            match request {
                ShortcutRequest::Use(index) => shortcut_calls.use_shortcut(index),
                ShortcutRequest::Set(index, shortcut) => {
                    shortcut_calls.set_shortcut(index, &shortcut)
                }
                ShortcutRequest::Examine { kind, data, item } => {
                    out.push(HudRequest::Examine { kind, data, item });
                }
                // `gmCharacterUI::RecvNotice_SetShortcutHeight`.
                ShortcutRequest::Height(rows) => {
                    if let Some(page) = character.as_mut() {
                        page.recv_shortcut_height(tree, rows);
                        out.extend(page.take_requests().into_iter().map(HudRequest::Character));
                    }
                }
                ShortcutRequest::Attack(_) | ShortcutRequest::WhichSet(_) => {}
            }
        }
    }
    out
}

/// The store panels' destructors as the session ends (`gmShopUI::~gmShopUI` and
/// `gmShopConsignmentUI::~gmShopConsignmentUI`: `RequestLeaveStore`; `gmShopCatalogUI::~gmShopCatalogUI`:
/// `RequestLeaveCatalog`), ahead of the log off or quit. Retail called them whenever the UI was torn
/// down with the player present; here they go out once each, only while such a panel is open.
pub(crate) fn leave_open_stores(
    shop: &Option<ShopUi>,
    consignment: &Option<ConsignmentUi>,
    new_sale: &Option<NewSaleUi>,
    catalog: &Option<CatalogUi>,
    out: &mut Vec<HudRequest>,
) {
    if shop.as_ref().is_some_and(ShopUi::is_shown)
        || consignment.as_ref().is_some_and(ConsignmentUi::is_shown)
        || new_sale.as_ref().is_some_and(NewSaleUi::is_shown)
    {
        out.push(HudRequest::Shop(ShopRequest::LeaveStore));
    }
    if catalog.as_ref().is_some_and(CatalogUi::is_shown) {
        out.push(HudRequest::Catalog(CatalogRequest::LeaveCatalog));
    }
}
