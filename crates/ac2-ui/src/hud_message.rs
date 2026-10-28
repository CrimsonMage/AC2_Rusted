//! The HUD frameworks' message pass (`ListenToMessage`): the quests' global loop, the options'
//! and dialogs' countdowns, then each of the frame's messages to the panels in the order the
//! frameworks listen to them. The panels are the HUD's ([`Hud`]); the requests the panels raise
//! come back as [`HudRequest`]s in that order, and the client maps them to gameplay commands.

use crate::{
    assistance_message, select_key_calls, select_key_of, shop_stack_calls, ConsignmentRequest,
    DialogFactory, ExamineEffect, ExternalContainerRequest, Hud, HudRequest, HudSource, NameLook,
    NamePlates, NewSaleSource, NewSaleUi, OptionsHost, Shortcut, SocialAction, StackUi,
    ToolbarChild, UiMessage, UiTree, EXAMINE_VITAE, GLOBAL_KEY_PRESS, LOGOUT_ACTION,
};
use crate::{item_dialog, ItemRequest};

/// The client's shortcut calls (`gmShortcutUI`'s requests on the player's script and session).
pub trait ShortcutCalls {
    /// Uses the shortcut in slot `index`.
    fn use_shortcut(&mut self, index: usize);
    /// Sets slot `index` to `shortcut`.
    fn set_shortcut(&mut self, index: usize, shortcut: &Shortcut);
}

/// The frame's values the message pass takes beside the HUD and its tree: the server's time, the
/// selection, the friends page's target, the client's answers to the panels' questions and the
/// client side of `gmClientUI`.
pub struct MessageInputs<'a> {
    /// The server's time at this frame.
    pub server_time: f64,
    /// The selected object's id, for the pet panel.
    pub selection: Option<u64>,
    /// The friends page's target (another player's name, when it is selected).
    pub friend_target: Option<&'a str>,
    /// The name plates' looks, taken only when the names key toggles them.
    pub name_looks: &'a dyn Fn() -> Vec<NameLook>,
    /// The shop's purchase limit for an item (`max_purchase_quantity`, 0 when unknown).
    pub max_purchase: &'a mut dyn FnMut(u64) -> u32,
    /// The consignment charge the script returns (`consignment_charge`, 0 when unknown).
    pub consignment_charge: &'a mut dyn FnMut() -> f32,
    /// The client side of `gmClientUI` (its preferences and live applies).
    pub options_host: &'a mut dyn OptionsHost,
}

/// One frame's messages through the panels, in the frameworks' order; returns the requests the
/// panels raised, in order. The panels are the HUD's ([`Hud`]); `now`, the shift key and the
/// options page's layout are the HUD's too.
pub fn message_pass<P: HudSource>(
    hud: &mut Hud,
    tree: &mut UiTree,
    inputs: MessageInputs<'_>,
    player: &P,
    messages: &[UiMessage],
) -> Vec<HudRequest> {
    let MessageInputs {
        server_time,
        selection,
        friend_target,
        name_looks,
        max_purchase,
        consignment_charge,
        options_host,
    } = inputs;
    let now = hud.now();
    let mut out = Vec::new();
    // `gmQuestsUI::Show` when the toolbar's Quest child changed, then
    // `gmJournalQuestsUI::ListenToMessage` on the global loop: the pages laid out, and the time
    // displays counted down.
    if let Some(panel) = hud.quests.as_mut() {
        if let Some(bar) = hud.toolbar.as_ref() {
            panel.sync(tree, bar.shown(tree, ToolbarChild::Quest));
        }
        panel.journal_mut().global_loop(tree, player, now);
        panel.story_global_loop(tree, player, || player.act_list());
    }
    // `gmClientUI`'s widgets, once the HUD and the preferences are there.
    hud.options
        .make(options_host, tree, hud.toolbar_layout.as_ref());
    // `TimedDialog`'s countdown: a question run out answers no.
    if let Some(dialogs) = hud.dialogs.as_mut() {
        dialogs.tick(tree, now);
    }
    for message in messages {
        // `gmToolbarUI::ListenToMessage`: the dismount button calls `gmCWInterface::Dismount`.
        if hud
            .toolbar
            .as_ref()
            .is_some_and(|bar| bar.is_dismount_press(message))
        {
            out.push(HudRequest::Dismount);
        }
        // `gmToolbarUI::ListenToMessage`: the consignments button's press asks for the sale
        // reminders (`gmCWInterface::DisplaySaleReminders`).
        if hud
            .toolbar
            .as_ref()
            .is_some_and(|bar| bar.is_sale_reminder_press(message))
        {
            out.push(HudRequest::DisplaySaleReminders);
        }
        // `gmSelectionUI::ListenToMessage`: a target key selects (the cycling keys).
        if let Some(key) = select_key_of(message) {
            let typing = tree.text_mode();
            out.extend(
                select_key_calls(player, key, typing)
                    .into_iter()
                    .map(HudRequest::Selection),
            );
        }
        if let (Some(page), Some(dialogs)) = (hud.character.as_mut(), hud.dialogs.as_mut()) {
            page.message(tree, dialogs, message);
            out.extend(page.take_requests().into_iter().map(HudRequest::Character));
        }
        if let (Some(page), Some(dialogs)) = (hud.chat_options.as_mut(), hud.dialogs.as_mut()) {
            page.message(tree, dialogs, message);
            out.extend(page.take_requests().into_iter().map(HudRequest::ChatOption));
        }
        if let Some(dialogs) = hud.dialogs.as_mut() {
            hud.options.message(options_host, tree, dialogs, message);
        }
        if let (Some(skills), Some(dialogs)) = (hud.skills.as_mut(), hud.dialogs.as_mut()) {
            skills.message(tree, dialogs, player, message);
        }
        if let Some(bar) = hud.shortcuts.as_mut() {
            bar.message(tree, player, message);
        }
        // `gmOverviewMapUI::ListenToMessage` (the close button) and the zoom buttons of
        // `gmNavigationUI::ListenToMessage` (`ShowMap` with the button's map).
        if let Some(overview) = hud.overview.as_ref() {
            overview.message(tree, message.element_id);
        }
        // `gmOptionUI::ListenToMessage`: leave, quit, assistance and the command tab.
        if let (Some(panel), Some(dialogs)) = (hud.option_panel.as_mut(), hud.dialogs.as_mut()) {
            panel.message(tree, dialogs, player, message);
        }
        // `gmExaminationUI::ListenToMessage`: the filter tabs and the tab keys.
        if let Some(panel) = hud.examination.as_mut() {
            panel.message(tree, message, hud.ctrl_held);
            out.extend(panel.take_calls().into_iter().map(HudRequest::Examination));
        }
        if let Some(panel) = hud.fellowship.as_mut() {
            panel.message(tree, message);
        }
        if let Some(panel) = hud.mini_fellow.as_mut() {
            panel.message(tree, message);
        }
        if let Some(panel) = hud.pets.as_mut() {
            out.extend(
                panel
                    .message(tree, message, selection)
                    .into_iter()
                    .map(HudRequest::Pet),
            );
        }
        if let Some(panel) = hud.friends.as_mut() {
            out.extend(
                panel
                    .message(tree, message, friend_target)
                    .into_iter()
                    .map(HudRequest::Friends),
            );
        }
        // `gmSocialUI::ListenToMessage` (`SocialHelpButton`): `KeyStone::OpenHelp`.
        if let Some(panel) = hud.social.as_mut() {
            if let Some(SocialAction::OpenHelp(token)) = panel.message(tree, message) {
                out.push(HudRequest::OpenHelp(token));
            }
        }
        if let (Some(page), Some(dialogs)) = (hud.assistance.as_mut(), hud.dialogs.as_mut()) {
            assistance_message(page, tree, dialogs, player, message);
        }
        if let Some(panel) = hud.quests.as_mut() {
            panel.journal_mut().message(tree, message);
            if let Some(action) = panel.story_message(tree, message) {
                panel.play_story_action(action, player);
            }
        }
        if let Some(bar) = hud.vitals.as_mut() {
            bar.message(tree, message, player);
        }
        // `gmVitaeStatusUI::ListenToMessage`: examine the vitae.
        if hud.vitae.as_mut().is_some_and(|v| v.message(tree, message)) {
            out.push(HudRequest::Examine {
                kind: EXAMINE_VITAE,
                data: 0,
                item: 0,
            });
        }
        // `gmRadarUI::ListenToMessage`: select the blip under the pointer, or show or hide the
        // coordinates.
        if let Some(request) = hud
            .radar
            .as_mut()
            .and_then(|r| r.message(tree, player, message))
        {
            out.push(HudRequest::Radar(request));
        }
        // `gm3DTextUI::ListenToMessage`: a plate selects its object; the names key toggles the
        // names (`ToggleNames`).
        if let Some(plates) = hud.names_mut() {
            if let Some(id) = plates.message(tree, message) {
                out.push(HudRequest::Select(id));
            }
            if NamePlates::toggle_requested(message) {
                let show = !plates.showing();
                plates.set_showing(tree, show, &name_looks());
                out.push(HudRequest::SetShowObjectNames(show));
            }
        }
        // `gmEffectsUI::ListenToMessage`: examine the pressed effect.
        if let Some(ExamineEffect(eid)) =
            hud.effects.as_mut().and_then(|s| s.message(tree, message))
        {
            out.push(HudRequest::Examine {
                kind: 0x4000_0001,
                data: eid,
                item: 0,
            });
        }
        if let Some(request) = hud
            .backpack
            .as_mut()
            .and_then(|p| p.message(tree, player, message))
        {
            push_item(request, tree, &mut hud.stack, player, &mut out);
        }
        // `gmMoneyBagUI::ListenToMessage`: a drop on the bag.
        if let (Some(bag), Some(dialogs)) = (hud.money.as_mut(), hud.dialogs.as_mut()) {
            if let Some(request) =
                bag.message(tree, player, player, dialogs, message, hud.shift_held)
            {
                push_item(request, tree, &mut hud.stack, player, &mut out);
            }
        }
        if let Some(panel) = hud.external_container.as_mut() {
            let (request, close) = panel.message(tree, player, message);
            if let Some(dialog) = hud.stack.as_mut() {
                for context in close {
                    dialog.close_stack_ui(tree, context);
                }
            }
            if let Some(request) = request {
                push_container(request, tree, &mut hud.stack, player, &mut out);
            }
        }
        // `gmStackUI::ListenToMessage`; its `StackUIDone` finishes the waiting drop.
        let done = hud
            .stack
            .as_mut()
            .and_then(|d| d.message(tree, message, player));
        if let Some(done) = done {
            let (context, amount) = (done.context, done.amount);
            let request = hud
                .backpack
                .as_mut()
                .and_then(|p| p.stack_ui_done(player, context, amount))
                .or_else(|| {
                    hud.external_container
                        .as_mut()
                        .and_then(|p| p.stack_ui_done(player, context, amount))
                });
            let request = request.or_else(|| hud.items_3d.as_mut()?.stack_ui_done(context, amount));
            let request = request.or_else(|| {
                let (bag, dialogs) = (hud.money.as_mut()?, hud.dialogs.as_mut()?);
                bag.stack_ui_done(
                    tree,
                    player,
                    player,
                    dialogs,
                    context,
                    amount,
                    hud.shift_held,
                )
            });
            if let Some(request) = request {
                out.push(HudRequest::Item(request));
            }
            if let Some(panel) = hud.secure_trade.as_mut() {
                panel.recv_stack_ui_done(context, amount);
            }
            if let Some(panel) = hud.shop.as_mut() {
                panel.recv_stack_ui_done(context, amount);
            }
            if let Some(panel) = hud.new_sale.as_mut() {
                panel.recv_stack_ui_done(tree, player, context, amount);
            }
            if let (Some(panel), Some(dialogs)) = (hud.consignment.as_mut(), hud.dialogs.as_mut()) {
                panel.recv_stack_ui_done(tree, dialogs, player, context, amount);
                consignment_requests(
                    panel.take_requests(),
                    hud.new_sale.as_mut(),
                    tree,
                    dialogs,
                    player,
                    server_time,
                    &mut out,
                );
                shop_stack_calls(panel.take_stack_calls(), tree, hud.stack.as_mut());
            }
        }
        if let Some(request) = hud
            .paper_doll
            .as_mut()
            .and_then(|d| d.message(tree, player, message))
        {
            out.push(HudRequest::Item(request));
        }
        if let (Some(page), Some(dialogs)) = (hud.allegiance.as_mut(), hud.dialogs.as_mut()) {
            page.message(tree, dialogs, player, message);
        }
        if let Some(panel) = hud.secure_trade.as_mut() {
            panel.message(tree, player, message);
        }
        // `gmShopUI::ListenToMessage`; a purchase asks the player how many it may buy.
        if let (Some(panel), Some(dialogs)) = (hud.shop.as_mut(), hud.dialogs.as_mut()) {
            panel.message(tree, player, dialogs, message, server_time, max_purchase);
        }
        // `gmGamePlayUI::ListenToMessage`: the logout key (`LOGOUT_KEY`, global key press
        // `0x4100000e`) asks for the end of the session on the next loop.
        if message.element.is_none()
            && message.message == GLOBAL_KEY_PRESS
            && message.data as u32 == LOGOUT_ACTION
        {
            hud.game_dialogs.logout_key();
        }
        // `gmShopCatalogUI::ListenToMessage`: the catalog's buttons and rows.
        if let Some(panel) = hud.catalog.as_mut() {
            panel.message(tree, message, server_time);
            out.extend(panel.take_requests().into_iter().map(HudRequest::Catalog));
        }
        // `gmShopConsignmentUI::ListenToMessage`: the consignment's buttons, rows and drop.
        if let (Some(panel), Some(dialogs)) = (hud.consignment.as_mut(), hud.dialogs.as_mut()) {
            panel.message(tree, dialogs, player, message, server_time);
            consignment_requests(
                panel.take_requests(),
                hud.new_sale.as_mut(),
                tree,
                dialogs,
                player,
                server_time,
                &mut out,
            );
            shop_stack_calls(panel.take_stack_calls(), tree, hud.stack.as_mut());
        }
        // `gmShopNewSaleUI::ListenToMessage`: the sale's buttons and the drop on its slot.
        if let (Some(panel), Some(dialogs)) = (hud.new_sale.as_mut(), hud.dialogs.as_mut()) {
            panel.message(
                tree,
                dialogs,
                player,
                message,
                server_time,
                consignment_charge,
            );
            out.extend(panel.take_requests().into_iter().map(HudRequest::NewSale));
            shop_stack_calls(panel.take_stack_calls(), tree, hud.stack.as_mut());
        }
        if let Some(dialogs) = hud.dialogs.as_mut() {
            dialogs.message(tree, message);
        }
    }
    out
}

/// A slot's request: the stack dialogs it opens (`ac2_ui::item_dialog`), then the rest as
/// [`HudRequest::Item`].
fn push_item<S: crate::InventoryNoticeSource>(
    request: ItemRequest,
    tree: &mut UiTree,
    stack: &mut Option<StackUi>,
    source: &S,
    out: &mut Vec<HudRequest>,
) {
    if let Some(request) = item_dialog(request, tree, stack.as_mut(), source) {
        out.push(HudRequest::Item(request));
    }
}

/// An external container's request: its item requests open their dialogs, the rest are kept for
/// the client.
fn push_container<S: crate::InventoryNoticeSource>(
    request: ExternalContainerRequest,
    tree: &mut UiTree,
    stack: &mut Option<StackUi>,
    source: &S,
    out: &mut Vec<HudRequest>,
) {
    match request {
        ExternalContainerRequest::Item(request) => push_item(request, tree, stack, source, out),
        other => out.push(HudRequest::ExternalContainer(other)),
    }
}

/// The consignment's requests: the new-sale dialog opens for `NewSale` (`InitNewSale`), the
/// examination is asked for ([`HudRequest::OpenExamination`]), and the rest are kept for the
/// client ([`HudRequest::Consignment`]).
pub fn consignment_requests(
    requests: Vec<ConsignmentRequest>,
    mut new_sale: Option<&mut NewSaleUi>,
    tree: &mut UiTree,
    dialogs: &mut DialogFactory,
    source: &dyn NewSaleSource,
    now: f64,
    out: &mut Vec<HudRequest>,
) {
    for request in requests {
        match request {
            ConsignmentRequest::NewSale { item, quantity } => {
                if let Some(panel) = new_sale.as_deref_mut() {
                    panel.recv_init_new_sale(tree, dialogs, source, item, quantity, now);
                }
            }
            ConsignmentRequest::OpenExamination => out.push(HudRequest::OpenExamination),
            other => out.push(HudRequest::Consignment(other)),
        }
    }
}
