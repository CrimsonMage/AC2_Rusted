//! The HUD frameworks' tick pass: the panels' ticks and global loops after the messages, the
//! dialogs' outcomes, and the calls the panels queued. The requests come back as [`HudRequest`]s
//! in the order the panels raised them; the client maps them to gameplay commands.

use ac2_weenie::SessionEvent;

use crate::{
    allegiance_calls, assistance_outcomes, consignment_requests, death_notices, fellowship_calls,
    shop_stack_calls, social_dialog_outcomes, AllegianceUi, AssistancePanel, CatalogUi,
    CharacterUi, ChatOptionsUi, ChatUi, ConsignmentUi, DeathPanel, DialogFactory, DialogRequest,
    DisplayNotice, ExaminationUi, FellowshipUi, GameplayDialogs, HudRequest, HudSource, MoneyBagUi,
    NewSaleUi, OptionsHost, OptionsPages, PetUi, PopupQuestion, SecureTradeUi, ShopRequest, ShopUi,
    ShortcutCalls, ShortcutRequest, ShortcutUi, SkillTreeUi, StackUi, ToolbarUi, UiTree,
};

/// The text type of the store's purchase messages (`TEXT_TYPE_ERROR`).
const SHOP_MESSAGE_TEXT_TYPE: u32 = 0x1;

/// The panels the tick pass feeds, borrowed from the client's HUD for one frame, and the frame's
/// values. A panel that is not made yet is `None` and gets nothing.
pub struct TickPanels<'a> {
    pub tree: &'a mut UiTree,
    /// The server's time at this frame.
    pub server_time: f64,
    /// The selected object's id, for the fellowship page.
    pub selection: Option<u64>,
    /// The game dialogs' backlogs: the script questions and the message boxes raised before the
    /// HUD's dialogs existed.
    pub popup_backlog: &'a mut Vec<PopupQuestion>,
    pub display_backlog: &'a mut Vec<DisplayNotice>,
    /// `gmClientUI`'s page.
    pub options: &'a mut OptionsPages,
    /// The client side of `gmClientUI` (its preferences and live applies).
    pub options_host: &'a mut dyn OptionsHost,
    /// The client's shortcut calls (`gmShortcutUI`'s use and set).
    pub shortcut_calls: &'a mut dyn ShortcutCalls,
    pub game_dialogs: &'a mut GameplayDialogs,
    pub death: &'a mut DeathPanel,
    pub dialogs: &'a mut Option<DialogFactory>,
    pub toolbar: &'a mut Option<ToolbarUi>,
    pub secure_trade: &'a mut Option<SecureTradeUi>,
    pub stack: &'a mut Option<StackUi>,
    pub money: &'a mut Option<MoneyBagUi>,
    pub pets: &'a mut Option<PetUi>,
    pub catalog: &'a mut Option<CatalogUi>,
    pub consignment: &'a mut Option<ConsignmentUi>,
    pub new_sale: &'a mut Option<NewSaleUi>,
    pub shop: &'a mut Option<ShopUi>,
    pub chat: &'a mut Option<ChatUi>,
    pub allegiance: &'a mut Option<AllegianceUi>,
    pub fellowship: &'a mut Option<FellowshipUi>,
    pub assistance: &'a mut Option<AssistancePanel>,
    pub character: &'a mut Option<CharacterUi>,
    pub chat_options: &'a mut Option<ChatOptionsUi>,
    pub examination: &'a mut Option<ExaminationUi>,
    pub skills: &'a mut Option<SkillTreeUi>,
    pub shortcuts: &'a mut Option<ShortcutUi>,
}

/// One frame's tick pass; returns the requests the panels raised, in order.
pub fn tick_pass<P: HudSource>(
    panels: TickPanels<'_>,
    player: &P,
    events: &[SessionEvent],
) -> Vec<HudRequest> {
    let TickPanels {
        tree,
        server_time,
        selection,
        popup_backlog,
        display_backlog,
        options,
        options_host,
        shortcut_calls,
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
        skills,
        shortcuts,
    } = panels;
    let mut out = Vec::new();
    // `gmToolbarUI::ManageChildren`: the buttons' tooltips follow the children's shown states.
    if let Some(bar) = toolbar.as_ref() {
        bar.refresh_tooltips(tree, player);
    }
    // `gmSecureTradeUI`'s global loop: an ended trade closes; a stack its list dropped asks.
    if let Some(panel) = secure_trade.as_mut() {
        panel.tick(tree, server_time);
        for split in panel.take_splits() {
            crate::secure_trade_split(split, tree, stack.as_mut(), player);
        }
    }
    death_notices(death, tree, dialogs, player, events);
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
            out.push(HudRequest::LogOff);
        }
    }
    if let Some(dialogs) = dialogs.as_mut() {
        let outcomes = dialogs.take_outcomes();
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
    // Legions passes the panel's filter word (+0xB8; its filter tabs are not modelled).
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
        for request in bar.take_requests() {
            match request {
                ShortcutRequest::Use(index) => shortcut_calls.use_shortcut(index),
                ShortcutRequest::Set(index, shortcut) => {
                    shortcut_calls.set_shortcut(index, &shortcut)
                }
                ShortcutRequest::Examine { kind, data, item } => {
                    out.push(HudRequest::Examine { kind, data, item });
                }
                ShortcutRequest::Attack(_) | ShortcutRequest::WhichSet(_) => {}
            }
        }
    }
    out
}
