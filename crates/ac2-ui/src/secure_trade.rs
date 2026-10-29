//! `gmSecureTradeUI`: the secure trade panel (`SecureTradeField` of the gameplay layout). It is a
//! child framework over the gameplay layout, hidden until a trade opens: `BeOpened` shows it, or
//! `BeRegistered` when the player asked (`acceptpending`). The partner's request shows a
//! confirmation (`ID_SecureTrade_AcceptTradeText`) instead, and its answer opens the trade
//! (`DoAcceptTrade`). Each side has a list (`TradeListBoxSelf` / `TradeListBoxOther`) of its
//! items and a `TradeButtonInner` that reads `accepted` / `notaccepted`; `Accept` and `Decline`
//! are the buttons (ghosted once pressed). A trade that ends (`BeClosed`) shows the partner's
//! `EndedText` and closes after the field's `delay`, from the global loop.
//!
//! The notices arrive as the `RecvNotice_*` methods, the presses and drops as
//! [`SecureTradeUi::message`], the confirmation's answer as [`SecureTradeUi::dialog_outcomes`],
//! the stack dialog's answer as [`SecureTradeUi::recv_stack_ui_done`]. What the panel asks of the
//! player is queued for [`SecureTradeUi::take_requests`]; a split asks the stack dialog through
//! [`SecureTradeUi::take_splits`].
//!
//! The retail notice handlers are called through the `gmNoticeHandler` base at `+0x60` of the
//! panel, which is why Ghidra's `this[-1]` appears; the fields here are the primary object's:
//! `_partnerID` (0x68), `_playerID` (0x70), `_partnerName` (0x78), `_acceptTradeContext` (0xa4),
//! `_selfList` (0xa8), `_otherList` (0xb8), `_dropContext` (0xc8), `_itemToTrade` (0xd0),
//! `_tradeEnded` (0xd8), `_whenToClose` (0xe0, a double the x87 adds `delay` to at 53 bits).

use std::sync::Arc;

use crate::dialog::{DialogFactory, DialogOutcome, DIALOG_CONFIRMATION};
use crate::drag::MSG_STOP_DRAG_ITEM;
use crate::element::UiTree;
use crate::item_display::{dragged_item, is_item_copy, InventorySource, ItemDisplay};
use crate::layout::{AttributeValue, LayoutDesc};
use crate::mouse::UiMessage;
use crate::skill_tree::SkillVar;
use crate::token::{tokenize, BUTTON_GHOSTED, MSG_BUTTON_RELEASE_ON, STATELESS};
use crate::ui_string::{UiString, UiStrings};

fn id(name: &str) -> u32 {
    tokenize(name)
}

/// `INVALID_IID`: no item, no player.
const INVALID: u64 = 0;

/// What the panel asks of the player: the `gmCWInterface` calls the trade makes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecureTradeRequest {
    /// `gmCWInterface::OpenTradeNegotiations(player, partner)`: the partner's request is accepted.
    Open { partner: u64 },
    /// `gmCWInterface::OfferItemForTrade(player, item, quantity)`.
    Offer { item: u64, quantity: u32 },
    /// `gmCWInterface::RevokeItemFromTrade(player, item)`.
    Revoke { item: u64 },
    /// `gmCWInterface::AcceptTrade(player)`.
    Accept,
    /// `gmCWInterface::DeclineTrade(player)`.
    Decline,
    /// `gmCWInterface::CloseTradeNegotiations(player)`.
    Close,
}

/// `CM_UI::SendNotice_InitStackUI` for a stack dropped on the player's list: `amount` the default
/// and the most. The answer comes back to [`SecureTradeUi::recv_stack_ui_done`] with `context`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SecureTradeSplit {
    pub context: u32,
    pub item: u64,
    pub amount: u32,
}

/// Which side of the trade a player or item is on (`Self` / `Other`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    Own,
    Partner,
}

impl Side {
    fn token(self) -> u32 {
        match self {
            Side::Own => id("Self"),
            Side::Partner => id("Other"),
        }
    }

    fn list_box(self) -> u32 {
        match self {
            Side::Own => id("TradeListBoxSelf"),
            Side::Partner => id("TradeListBoxOther"),
        }
    }
}

/// `gmSecureTradeUI`'s state.
#[derive(Debug, Clone)]
pub struct SecureTradeUi {
    layout: Arc<LayoutDesc>,
    /// `_secureTradeField`: the panel's root while it is shown.
    field: Option<usize>,
    /// `IsShown`.
    shown: bool,
    /// `_playerID`, `_partnerID` and `_partnerName`.
    player: u64,
    partner: u64,
    partner_name: String,
    /// `_acceptTradeContext`: the confirmation of a partner's request (0: none).
    accept_context: u32,
    /// `_selfList` and `_otherList`, in the order they were added.
    own_items: Vec<ItemDisplay>,
    partner_items: Vec<ItemDisplay>,
    /// `_dropContext`: the stack dialog a list drop waits on.
    drop_context: u32,
    /// `_itemToTrade`: the item the drop that waits on its stack offers.
    item_to_trade: u64,
    /// `_tradeEnded` and `_whenToClose` (the global loop's clock).
    trade_ended: bool,
    when_to_close: f64,
    requests: Vec<SecureTradeRequest>,
    splits: Vec<SecureTradeSplit>,
}

impl SecureTradeUi {
    /// `gmSecureTradeUI::gmSecureTradeUI`: hidden, no field. The player is set by
    /// [`Self::set_player`] once the player is in the world (the constructor reads it).
    pub fn create(gameplay: &Arc<LayoutDesc>) -> Self {
        Self {
            layout: Arc::clone(gameplay),
            field: None,
            shown: false,
            player: INVALID,
            partner: INVALID,
            partner_name: String::new(),
            accept_context: 0,
            own_items: Vec::new(),
            partner_items: Vec::new(),
            drop_context: 0,
            item_to_trade: INVALID,
            trade_ended: false,
            when_to_close: 0.0,
            requests: Vec::new(),
            splits: Vec::new(),
        }
    }

    /// The player's id (`CObjectMaint::GetPlayerWeenieObject()`'s `m_iid`).
    pub fn set_player(&mut self, player: u64) {
        self.player = player;
    }

    /// The panel's root, while it is shown.
    pub fn field(&self) -> Option<usize> {
        self.field
    }

    /// `IsShown`.
    pub fn is_shown(&self) -> bool {
        self.shown
    }

    /// `_partnerID`.
    pub fn partner(&self) -> u64 {
        self.partner
    }

    /// `_partnerName`.
    pub fn partner_name(&self) -> &str {
        &self.partner_name
    }

    /// `_tradeEnded`: the trade is over and waits for its close.
    pub fn trade_ended(&self) -> bool {
        self.trade_ended
    }

    /// The items in the player's list, in order.
    pub fn own_items(&self) -> Vec<u64> {
        self.own_items.iter().map(|d| d.item).collect()
    }

    /// The items in the partner's list, in order.
    pub fn partner_items(&self) -> Vec<u64> {
        self.partner_items.iter().map(|d| d.item).collect()
    }

    /// `ItemDisplay::RecvNotice_SetTargetingPointer` of both lists' items.
    pub fn set_targeting_pointer(&self, tree: &mut UiTree, on: bool) {
        for display in self.own_items.iter().chain(&self.partner_items) {
            display.set_targeting_pointer(tree, on);
        }
    }

    /// The requests the panel made since the last call, for the client to send.
    pub fn take_requests(&mut self) -> Vec<SecureTradeRequest> {
        std::mem::take(&mut self.requests)
    }

    /// The stack dialogs the panel asked for since the last call.
    pub fn take_splits(&mut self) -> Vec<SecureTradeSplit> {
        std::mem::take(&mut self.splits)
    }

    fn side_of(&self, src: u64) -> Side {
        if src == self.player {
            Side::Own
        } else {
            Side::Partner
        }
    }

    fn items_mut(&mut self, side: Side) -> &mut Vec<ItemDisplay> {
        match side {
            Side::Own => &mut self.own_items,
            Side::Partner => &mut self.partner_items,
        }
    }

    /// `gmSecureTradeUI::Show`: shown makes the field (`CreateElement(0x41000009,
    /// SecureTradeField)`) if there is none; hidden destroys it with the items in its lists.
    fn show(&mut self, tree: &mut UiTree, shown: bool) {
        self.shown = shown;
        if !shown {
            if let Some(field) = self.field.take() {
                tree.remove_root(field);
            }
            self.own_items.clear();
            self.partner_items.clear();
            return;
        }
        if self.field.is_none() {
            self.field = tree.create_root(Arc::clone(&self.layout), id("SecureTradeField"));
        }
        if let Some(field) = self.field {
            tree.set_visible(field, true);
        }
    }

    /// The panel's element under `name` (its child of the side's element is `side`).
    fn child_of_side(&self, tree: &UiTree, side: Side, name: &str) -> Option<usize> {
        let field = self.field?;
        let side = tree.child_recursive(field, side.token())?;
        tree.child_recursive(side, id(name))
    }

    /// `gmSecureTradeUI::UpdateButton`: the side's `TradeButtonInner` shows `accepted`.
    fn update_button(&self, tree: &mut UiTree, src: u64, accepted: bool) {
        let side = self.side_of(src);
        let Some(button) = self.child_of_side(tree, side, "TradeButtonInner") else {
            return;
        };
        tree.set_state(button, STATELESS);
        let state = if accepted {
            id("accepted")
        } else {
            id("notaccepted")
        };
        tree.set_state(button, state);
    }

    /// `gmSecureTradeUI::MakeAcceptTradeDialog`: the partner's request asks for a confirmation,
    /// unless one is up.
    fn make_accept_dialog(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        strings: &dyn UiStrings,
    ) {
        if self.accept_context != 0 {
            return;
        }
        let text = UiString::ui(
            id("ID_SecureTrade_AcceptTradeText"),
            vec![(id("value"), SkillVar::Text(self.partner_name.clone()))],
        );
        let text = strings.render_string(&text);
        self.accept_context = dialogs.make_dialog(tree, &text, DIALOG_CONFIRMATION, false);
    }

    /// `gmSecureTradeUI::RecvNotice_Trade_BeRegistered`: `master` opened the trade with `slave`
    /// (`partner_name` is the partner's name). The player's own request shows the panel
    /// (`acceptpending`); the partner's asks through the confirmation.
    pub fn recv_trade_be_registered(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        strings: &dyn UiStrings,
        master: u64,
        slave: u64,
        partner_name: &str,
    ) {
        self.partner = if master == self.player { slave } else { master };
        self.partner_name = partner_name.to_owned();
        if self.shown && self.trade_ended {
            self.trade_ended = false;
            self.show(tree, false);
        }
        if master == self.player {
            self.show(tree, true);
            if let Some(field) = self.field {
                tree.set_state(field, id("acceptpending"));
            }
        } else {
            self.make_accept_dialog(tree, dialogs, strings);
        }
    }

    /// `gmSecureTradeUI::RecvNotice_Trade_BeOpened`: the panel shows the trade, with the partner's
    /// name on its label. (The lists' drop callbacks are the message checks in [`Self::message`].)
    pub fn recv_trade_be_opened(&mut self, tree: &mut UiTree) {
        self.show(tree, true);
        let Some(field) = self.field else {
            return;
        };
        tree.set_state(field, id("trade"));
        if let Some(label) = tree.child_recursive(field, id("PartnerLabel")) {
            tree.set_text(label, &self.partner_name);
        }
    }

    /// `gmSecureTradeUI::ResetInternal`: the side's items gone, its list flushed and its button
    /// `notaccepted`.
    fn reset_side(&mut self, tree: &mut UiTree, side: Side) {
        self.items_mut(side).clear();
        let Some(field) = self.field else {
            return;
        };
        let Some(side_element) = tree.child_recursive(field, side.token()) else {
            return;
        };
        let Some(list) = tree.child_recursive(side_element, side.list_box()) else {
            return;
        };
        tree.list_flush(list);
        if let Some(button) = tree.child_recursive(side_element, id("TradeButtonInner")) {
            tree.set_state(button, id("notaccepted"));
        }
    }

    /// `gmSecureTradeUI::RecvNotice_Trade_BeClosed`: the trade ends. The confirmation closes, a
    /// trade that is open resets both lists. Our close hides the panel; the partner's shows its
    /// `EndedText` and closes `delay` seconds after `now` (the global loop, [`Self::tick`]).
    pub fn recv_trade_be_closed(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        strings: &dyn UiStrings,
        source: u64,
        now: f64,
    ) {
        if !self.shown {
            return;
        }
        if self.accept_context != 0 {
            dialogs.close_dialog(tree, self.accept_context);
            self.accept_context = 0;
        }
        let trade_open = self
            .field
            .and_then(|f| tree.get(f))
            .is_some_and(|e| e.state() == id("trade"));
        if trade_open {
            self.reset_side(tree, Side::Own);
            self.reset_side(tree, Side::Partner);
        }
        if source == self.player {
            self.show(tree, false);
            return;
        }
        self.when_to_close = now;
        if let Some(field) = self.field {
            tree.set_state(field, id("partnerclosed"));
            if let Some(ended) = tree.child_recursive(field, id("EndedText")) {
                let text = UiString::ui(
                    id("ID_SecureTrade_Ended"),
                    vec![(id("name"), SkillVar::Text(self.partner_name.clone()))],
                );
                tree.set_text(ended, &strings.render_string(&text));
            }
            // `GetAttribute_Float(field, token_delay)`; a missing delay closes at once.
            let delay = match tree.get(field).and_then(|e| e.attribute(id("delay"))) {
                Some(AttributeValue::Float(seconds)) => *seconds,
                _ => 0.0,
            };
            self.when_to_close += f64::from(delay);
        }
        self.trade_ended = true;
    }

    /// `gmSecureTradeUI::RecvNotice_Trade_BeReset`: an open trade's lists are reset.
    pub fn recv_trade_be_reset(&mut self, tree: &mut UiTree) {
        if !self.shown {
            return;
        }
        let trade_open = self
            .field
            .and_then(|f| tree.get(f))
            .is_some_and(|e| e.state() == id("trade"));
        if trade_open {
            self.reset_side(tree, Side::Own);
            self.reset_side(tree, Side::Partner);
        }
    }

    /// `gmSecureTradeUI::RecvNotice_Trade_BeAcceptanceCleared`: neither side accepts.
    pub fn recv_trade_be_acceptance_cleared(&mut self, tree: &mut UiTree) {
        if !self.shown {
            return;
        }
        self.update_button(tree, self.player, false);
        self.update_button(tree, self.partner, false);
    }

    /// `gmSecureTradeUI::RecvNotice_Trade_BeAccepted`: the partner's accept shows on its side;
    /// `INVALID` shows the player's own button enabled again.
    pub fn recv_trade_be_accepted(&mut self, tree: &mut UiTree, src: u64) {
        self.answered(tree, src, "AcceptTradeButton", true);
    }

    /// `gmSecureTradeUI::RecvNotice_Trade_BeDeclined`: as [`Self::recv_trade_be_accepted`] for the
    /// decline. A decline does not close the panel.
    pub fn recv_trade_be_declined(&mut self, tree: &mut UiTree, src: u64) {
        self.answered(tree, src, "DeclineTradeButton", false);
    }

    fn answered(&self, tree: &mut UiTree, src: u64, button: &str, accepted: bool) {
        if !self.shown {
            return;
        }
        if src != INVALID {
            self.update_button(tree, src, accepted);
            return;
        }
        if let Some(button) = self.child_of_side(tree, Side::Own, button) {
            tree.set_bool(button, BUTTON_GHOSTED, false);
        }
    }

    /// `gmSecureTradeUI::AddItem`: an item joins the side's list, unless it is listed already.
    /// Its icon is dragable only on the player's side, and it is not a shortcut (so it cannot be
    /// dropped back on a list). It is named at the offered count (`SetQuantity(num)`: Legions
    /// 0x42130a passes the fifth argument; the decompile shows the item's high dword).
    fn add_item(
        &mut self,
        tree: &mut UiTree,
        s: &dyn InventorySource,
        src: u64,
        item: u64,
        num: u32,
    ) {
        let side = self.side_of(src);
        let Some(field) = self.field else {
            return;
        };
        let Some(side_element) = tree.child_recursive(field, side.token()) else {
            return;
        };
        let Some(list) = tree.child_recursive(side_element, side.list_box()) else {
            return;
        };
        if self.items_mut(side).iter().any(|d| d.item == item) {
            return;
        }
        let Some(desc) = self.layout.element(id("TradeItemTemplate")).cloned() else {
            return;
        };
        let Some(background) = tree.create_element(Arc::clone(&self.layout), desc, false) else {
            return;
        };
        tree.list_add_item(list, background);
        let Some(mut display) =
            ItemDisplay::init(tree, s, background, id("ItemIcon"), item, INVALID, false)
        else {
            return;
        };
        display.set_draggable(tree, s, side == Side::Own);
        display.set_quantity(tree, s, num);
        display.set_shortcut_type(tree, 0);
        self.items_mut(side).push(display);
    }

    /// `gmSecureTradeUI::RecvNotice_Trade_BeOffered`: while shown, the offered item joins its
    /// side's list.
    pub fn recv_trade_be_offered(
        &mut self,
        tree: &mut UiTree,
        s: &dyn InventorySource,
        src: u64,
        item: u64,
        amount: u32,
    ) {
        if self.shown {
            self.add_item(tree, s, src, item, amount);
        }
    }

    /// `gmSecureTradeUI::RemoveItem`: the item leaves its side's list and its icon goes with it.
    fn remove_item(&mut self, tree: &mut UiTree, src: u64, item: u64) {
        let side = self.side_of(src);
        let Some(field) = self.field else {
            return;
        };
        let Some(side_element) = tree.child_recursive(field, side.token()) else {
            return;
        };
        let Some(list) = tree.child_recursive(side_element, side.list_box()) else {
            return;
        };
        let items = self.items_mut(side);
        let Some(at) = items.iter().position(|d| d.item == item) else {
            return;
        };
        let display = items.remove(at);
        tree.list_delete_item(list, display.background);
    }

    /// `gmSecureTradeUI::RecvNotice_Trade_BeRevoked`.
    pub fn recv_trade_be_revoked(&mut self, tree: &mut UiTree, src: u64, item: u64) {
        self.remove_item(tree, src, item);
    }

    /// `gmSecureTradeUI::RecvNotice_CloseConfirmationDialog`: the answer to the partner's request
    /// (`yes` opens the trade).
    pub fn recv_close_confirmation_dialog(&mut self, tree: &mut UiTree, context: u32, yes: bool) {
        if self.accept_context == context {
            self.do_accept_trade(tree, yes);
        }
    }

    /// The confirmation's answers among the dialogs' outcomes.
    pub fn dialog_outcomes(&mut self, tree: &mut UiTree, outcomes: &[DialogOutcome]) {
        for outcome in outcomes {
            if let DialogOutcome::Confirmation { context, yes } = *outcome {
                self.recv_close_confirmation_dialog(tree, context, yes);
            }
        }
    }

    /// `gmSecureTradeUI::DoAcceptTrade`: `yes` opens the trade; otherwise the trade is closed
    /// (or, when it has ended, hidden).
    fn do_accept_trade(&mut self, tree: &mut UiTree, yes: bool) {
        if yes {
            self.requests.push(SecureTradeRequest::Open {
                partner: self.partner,
            });
            return;
        }
        if self.trade_ended {
            self.show(tree, false);
            self.trade_ended = false;
        } else {
            self.requests.push(SecureTradeRequest::Close);
        }
    }

    /// `gmSecureTradeUI::CloseTrade`: close the trade, or hide the ended one.
    fn close_trade(&mut self, tree: &mut UiTree) {
        if self.trade_ended {
            self.show(tree, false);
            self.trade_ended = false;
        } else {
            self.requests.push(SecureTradeRequest::Close);
        }
    }

    /// `gmSecureTradeUI::ListenToMessage` for the global loop (`0xffff0005`): an ended trade
    /// closes once its time has passed.
    pub fn tick(&mut self, tree: &mut UiTree, now: f64) {
        if self.shown && self.trade_ended && self.when_to_close < now {
            self.close_trade(tree);
        }
    }

    /// `gmSecureTradeUI::RecvNotice_StackUIDone`: the answer to a list drop's stack asks the
    /// drop's item for `amount`.
    pub fn recv_stack_ui_done(&mut self, context: u32, amount: u32) {
        if context == self.drop_context {
            self.requests.push(SecureTradeRequest::Offer {
                item: self.item_to_trade,
                quantity: amount,
            });
            self.item_to_trade = INVALID;
        }
    }

    /// `gmSecureTradeUI::FinishListBoxDrop`: the item is offered in `quantity`.
    fn finish_list_box_drop(&mut self, quantity: u32) {
        self.requests.push(SecureTradeRequest::Offer {
            item: self.item_to_trade,
            quantity,
        });
        self.item_to_trade = INVALID;
    }

    /// `gmSecureTradeUI::HandleListBoxDrop`: the player's unequipped item is offered. A stack of
    /// more than one asks how many first: its default and most are the stack's quantity, or its
    /// maximum when that is less.
    fn handle_list_box_drop(&mut self, tree: &mut UiTree, s: &dyn InventorySource, copy: usize) {
        let Some(dragged) = dragged_item(tree, copy) else {
            return;
        };
        if dragged.from != self.player || dragged.equipped {
            return;
        }
        let item = dragged.item;
        self.item_to_trade = item;
        let Some(slot) = s.item(item) else {
            return;
        };
        if slot.stackable && 1 < slot.quantity {
            self.drop_context = tree.request_stack_context();
            let amount = if slot.max_stack < slot.quantity {
                slot.max_stack
            } else {
                slot.quantity
            };
            self.splits.push(SecureTradeSplit {
                context: self.drop_context,
                item,
                amount,
            });
            return;
        }
        self.finish_list_box_drop(slot.quantity);
    }

    /// `gmSecureTradeUI::ListenToMessage`: the buttons (close, cancel, accept, decline) and, while
    /// shown, the lists' drops (`stop_drag_item` with an item copy) and an item dragged out of
    /// the list (`TradeItemTemplate`, `data` 0).
    pub fn message(&mut self, tree: &mut UiTree, s: &dyn InventorySource, message: &UiMessage) {
        if message.message == MSG_BUTTON_RELEASE_ON {
            match message.element_id {
                e if e == id("SecureTradeCloseButton") || e == id("SecureTradeCancelButton") => {
                    self.close_trade(tree);
                    return;
                }
                e if e == id("AcceptTradeButton") => {
                    self.requests.push(SecureTradeRequest::Accept);
                }
                e if e == id("DeclineTradeButton") => {
                    self.requests.push(SecureTradeRequest::Decline);
                }
                _ => return,
            }
            if let Some(button) = message.element {
                tree.set_bool(button, BUTTON_GHOSTED, true);
            }
            return;
        }
        if !self.shown || message.message != MSG_STOP_DRAG_ITEM {
            return;
        }
        let Some(copy) = message.item else {
            return;
        };
        let element = message.element_id;
        if element == id("TradeListBoxSelf") || element == id("TradeListBoxOther") {
            if message.data != 0 && is_item_copy(tree, copy) {
                self.handle_list_box_drop(tree, s, copy);
            }
        } else if element == id("TradeItemTemplate") && message.data == 0 {
            if let Some(dragged) = dragged_item(tree, copy) {
                self.requests
                    .push(SecureTradeRequest::Revoke { item: dragged.item });
            }
        }
    }
}
