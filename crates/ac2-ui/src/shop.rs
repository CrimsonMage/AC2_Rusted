//! `gmShopUI`: the vendor's shop panel (`Shop` of the gameplay layout), a child framework made
//! when a store opens (`CreateManagedElements`) and destroyed after it has been hidden for two
//! minutes (`ListenToMessage`'s global loop). `RecvNotice_ViewStore` opens it with the store's
//! name, description, portrait and sort text, and its sales ([`shop_rows`](super::shop_rows));
//! `UpdateStore` rebuilds the list, `UpdateSale` a sale's stock, `LeaveStore` closes it.
//!
//! A press on a sale selects it (`Select`, which asks the player to select the item), and the
//! purchase button buys the selected one (`PurchaseItem`). A purchase asks how many when the
//! player may buy more than one (the stack dialog, [`ShopStackCall::Init`]), then asks the
//! player to buy it (`FinishDragFromShop` -> [`ShopRequest::Purchase`]). The server's
//! `ConfirmPurchase` raises the confirmation, whose answer is [`ShopRequest::PurchaseConfirmed`].
//!
//! The notices arrive as the `recv_*` methods, the presses as [`ShopUi::message`], the dialogs'
//! answers as [`ShopUi::dialog_outcomes`] and the stack dialog's as [`ShopUi::recv_stack_ui_done`].
//! What the panel asks of the player is queued for [`ShopUi::take_requests`], the stack dialogs it
//! wants for [`ShopUi::take_stack_calls`], and the purchase messages for the chat through
//! [`ShopUi::take_messages`].
//!
//! Not modelled: the sort text's paging beyond the request (the sales come back as the server
//! sends them), the help page (`KeyStone::OpenHelp`), the examination panel's own state (the
//! request is [`ShopRequest::OpenExamination`]).

use std::sync::Arc;

use ac2_net::protocol::StringInfo;
use ac2_weenie::Shop;

use crate::dialog::{DialogFactory, DialogOutcome, DIALOG_CONFIRMATION};
use crate::element::UiTree;
use crate::layout::LayoutDesc;
use crate::mouse::UiMessage;
use crate::skill_tree::SkillVar;
use crate::token::{tokenize, BUTTON_GHOSTED, MSG_BUTTON_RELEASE_ON, MSG_MOUSE_PRESS, STATELESS};
use crate::ui_string::{UiString, UiStrings};

pub(crate) fn id(name: &str) -> u32 {
    tokenize(name)
}

/// `INVALID_IID`: no item.
pub(crate) const INVALID: u64 = 0;
/// `m_myContext`: the shop's drag context (its sales' drag source). Retail takes it from
/// `ContainerDisplay::RequestContext`, a counter of its own; the panel keeps one for its life, so a
/// constant does, and the stack dialogs' counter is left alone.
const SHOP_CONTEXT: u32 = 0x0C00_0001;
/// The selected sale's state (`Token_Selected`).
pub(crate) const SELECTED: &str = "selected";
/// `ListenToMessage`'s global loop: a hidden shop waits this long before it is destroyed.
const DORMANT_LIMIT: f64 = 120.0;
/// A page request waits this long before the buttons are live again (`m_ttNextUpdate`).
const PAGE_WAIT: f64 = 1.0;
/// A typed sort text waits this long before its page is asked for (`m_ttTimeToUpdateSortText`).
const SORT_WAIT: f64 = 3.0;
/// The text messages the sort text is registered for (`ListenToMessage`'s three `TextMsg_*`, with
/// `deactivated` the one that does not wait).
const SORT_TEXT_MESSAGES: [u32; 4] = [
    tokenize("activated"),
    tokenize("key_input"),
    tokenize("character_input"),
    tokenize("deactivated"),
];
/// The sale's press (`data` 0x40000004: select) and its examine (0x40000005).
const SALE_SELECT: i32 = 0x4000_0004;
const SALE_EXAMINE: i32 = 0x4000_0005;

/// What the panel asks of the player: the `gmCWInterface` calls the shop makes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShopRequest {
    /// `gmCWInterface::RequestSales(index)`: the page the sort text names.
    Sales(u32),
    /// `gmCWInterface::RequestNextSales`.
    NextSales,
    /// `gmCWInterface::RequestPrevSales`.
    PrevSales,
    /// `gmCWInterface::RequestLeaveStore`.
    LeaveStore,
    /// `gmCWInterface::SetSelection(item)`: a sale was selected.
    Select(u64),
    /// `gmCWInterface::PurchaseItem(item, quantity, slot)`.
    Purchase { item: u64, quantity: u32, slot: u32 },
    /// `gmCWInterface::PurchaseConfirmed(answer)`.
    PurchaseConfirmed(bool),
    /// `CM_UI::SendNotice_OpenExaminationPanel`: a sale was examined.
    OpenExamination,
}

/// The stack dialog calls the shop makes (`CM_UI::SendNotice_InitStackUI` / `CloseStackUI`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShopStackCall {
    /// `InitStackUI(context, text, 0, max, INVALID, INVALID)`: how many of the item to buy.
    Init {
        context: u32,
        text: String,
        max: u32,
    },
    /// `CloseStackUI(context)`.
    Close(u32),
}

/// What the panel reads from the client: the strings it renders and the server's strings.
pub trait ShopSource: UiStrings {
    /// A string the server sent (a `StringInfo` on the wire), rendered.
    fn render_wire(&self, info: &StringInfo) -> String;
}

/// `gmShopUI`'s state.
#[derive(Debug, Clone)]
pub struct ShopUi {
    pub(crate) layout: Arc<LayoutDesc>,
    /// `m_pShopField`: the panel's root, while the managed elements exist.
    pub(crate) field: Option<usize>,
    /// `IsShown`.
    pub(crate) shown: bool,
    /// `m_myContext`: the sales' drag source context.
    pub(crate) context: u32,
    /// `m_stackContext` (0: none) and `m_confirmationContext` (0: none).
    pub(crate) stack_context: u32,
    pub(crate) confirm_context: u32,
    /// `m_selectedElement`: the selected sale's element.
    pub(crate) selected: Option<usize>,
    /// `m_elements`: each sale's item and element.
    pub(crate) rows: Vec<(u64, usize)>,
    /// `m_itemToPurchase` and `m_targetSlot`.
    pub(crate) item_to_purchase: u64,
    pub(crate) target_slot: u32,
    /// `m_bSortTextEdited`, `m_ttTimeToUpdateSortText`, `m_ttNextUpdate` (0: none) and
    /// `m_timeDormant`, on the global loop's clock.
    pub(crate) sort_edited: bool,
    pub(crate) sort_time: f64,
    pub(crate) next_update: f64,
    pub(crate) dormant_since: f64,
    requests: Vec<ShopRequest>,
    stack_calls: Vec<ShopStackCall>,
    messages: Vec<String>,
}

impl ShopUi {
    /// `gmShopUI::gmShopUI`: hidden, with no managed elements; the gameplay layout is kept for them.
    pub fn create(gameplay: &Arc<LayoutDesc>) -> Self {
        Self {
            layout: Arc::clone(gameplay),
            field: None,
            shown: false,
            context: SHOP_CONTEXT,
            stack_context: 0,
            confirm_context: 0,
            selected: None,
            rows: Vec::new(),
            item_to_purchase: INVALID,
            target_slot: 0,
            sort_edited: false,
            sort_time: 0.0,
            next_update: 0.0,
            dormant_since: 0.0,
            requests: Vec::new(),
            stack_calls: Vec::new(),
            messages: Vec::new(),
        }
    }

    /// The panel's root, while its managed elements exist.
    pub fn field(&self) -> Option<usize> {
        self.field
    }

    /// `IsShown`.
    pub fn is_shown(&self) -> bool {
        self.shown
    }

    /// The selected sale's item.
    pub fn selected_item(&self) -> Option<u64> {
        let row = self.selected?;
        self.rows
            .iter()
            .find(|(_, r)| *r == row)
            .map(|(item, _)| *item)
    }

    /// The sales' items, in the list's order.
    pub fn sale_items(&self) -> Vec<u64> {
        self.rows.iter().map(|(item, _)| *item).collect()
    }

    /// The sale element of `item`, while the list shows it.
    pub fn sale_element(&self, item: u64) -> Option<usize> {
        self.rows.iter().find(|(i, _)| *i == item).map(|(_, r)| *r)
    }

    /// The purchase confirmation's dialog context (0: none).
    pub fn confirm_context(&self) -> u32 {
        self.confirm_context
    }

    /// The requests the panel made since the last call, for the client to send.
    pub fn take_requests(&mut self) -> Vec<ShopRequest> {
        std::mem::take(&mut self.requests)
    }

    /// The stack dialog calls since the last call.
    pub fn take_stack_calls(&mut self) -> Vec<ShopStackCall> {
        std::mem::take(&mut self.stack_calls)
    }

    /// The purchase messages since the last call (`CM_Character::SendNotice_DisplayMessage`).
    pub fn take_messages(&mut self) -> Vec<String> {
        std::mem::take(&mut self.messages)
    }

    /// The element `name` under the panel.
    pub(crate) fn child(&self, tree: &UiTree, name: &str) -> Option<usize> {
        tree.child_recursive(self.field?, id(name))
    }

    /// `gmShopUI::CreateManagedElements`: the `Shop` root and its managed children, once.
    fn create_managed(&mut self, tree: &mut UiTree) -> bool {
        if self.field.is_some() {
            return true;
        }
        self.field = tree.create_root(Arc::clone(&self.layout), id("Shop"));
        if let Some(field) = self.field {
            if let Some(next) = tree.child_recursive(field, id("ShopNextButton")) {
                tree.set_bool(next, BUTTON_GHOSTED, true);
            }
            if let Some(prev) = tree.child_recursive(field, id("ShopPrevButton")) {
                tree.set_bool(prev, BUTTON_GHOSTED, true);
            }
        }
        self.field.is_some()
    }

    /// `gmShopUI::DestroyManagedElements`: the root and the sales go.
    fn destroy_managed(&mut self, tree: &mut UiTree) {
        if let Some(field) = self.field.take() {
            tree.remove_root(field);
        }
        self.selected = None;
        self.rows.clear();
    }

    /// `gmShopUI::Show`: shown makes the managed elements and shows them; hidden closes the stack
    /// and the confirmation and hides the root, which waits to be destroyed.
    pub(crate) fn show(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        shown: bool,
        now: f64,
    ) {
        self.shown = shown;
        if shown {
            if self.create_managed(tree) {
                if let Some(field) = self.field {
                    tree.set_visible(field, true);
                }
            }
            return;
        }
        if self.stack_context != 0 {
            self.stack_calls
                .push(ShopStackCall::Close(self.stack_context));
            self.stack_context = 0;
        }
        if self.confirm_context != 0 {
            dialogs.close_dialog(tree, self.confirm_context);
            self.confirm_context = 0;
        }
        if let Some(field) = self.field {
            tree.set_visible(field, false);
        }
        self.dormant_since = now;
    }

    /// `gmShopUI::RecvNotice_ViewStore`: the store opens with its name, description, portrait,
    /// sort text and sales. The notice's shop is the one the list is built from.
    pub fn recv_view_store(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        s: &dyn ShopSource,
        shop: &Shop,
        now: f64,
    ) {
        self.show(tree, dialogs, true, now);
        if self.field.is_none() {
            return;
        }
        if let Some(name) = self.child(tree, "ShopName") {
            tree.set_text(name, &s.render_wire(&shop.name));
        }
        if let Some(desc) = self.child(tree, "ShopDesc") {
            tree.set_text(desc, &s.render_wire(&shop.desc));
        }
        if let Some(portrait) = self.child(tree, "Portrait") {
            tree.add_image_to_state(portrait, shop.portrait, 0, crate::token::STATELESS);
            tree.set_visible(portrait, true);
        }
        if let Some(sort) = self.child(tree, "SortText") {
            let text = s.render_wire(&shop.sorter);
            tree.set_text(sort, &text);
        }
        self.update_sales(tree, s, shop);
    }

    /// `gmShopUI::RecvNotice_UpdateStore`: while shown, the list is rebuilt from the shop.
    pub fn recv_update_store(&mut self, tree: &mut UiTree, s: &dyn ShopSource, shop: &Shop) {
        if self.shown {
            self.update_sales(tree, s, shop);
        }
    }

    /// `gmShopUI::RecvNotice_LeaveStore`: the store closes.
    pub fn recv_leave_store(&mut self, tree: &mut UiTree, dialogs: &mut DialogFactory, now: f64) {
        self.show(tree, dialogs, false, now);
    }

    /// `gmShopUI::RecvNotice_UpdatePosition`: the page's buttons follow the position and size.
    pub fn recv_update_position(&mut self, tree: &mut UiTree, position: i32, size: i32) {
        if self.shown {
            self.update_position(tree, position, size);
        }
    }

    /// `gmShopUI::RecvNotice_SelectionChanged`: a selection elsewhere clears the sale's.
    pub fn recv_selection_changed(&mut self, tree: &mut UiTree, new: u64) {
        if !self.shown {
            return;
        }
        if self.selected.is_some() && self.selected_item() != Some(new) {
            self.select(tree, None);
        }
    }

    /// `gmShopUI::RecvNotice_ConfirmPurchase`: while shown, the confirmation asks whether to buy
    /// `quantity` of `product` from `trade` for `cost`.
    pub fn recv_confirm_purchase(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        s: &dyn ShopSource,
        product: &StringInfo,
        quantity: u32,
        trade: &StringInfo,
        cost: u32,
    ) {
        if !self.shown {
            return;
        }
        let string_id = if quantity == 1 {
            "ID_Store_ConfirmPurchaseSingular"
        } else {
            "ID_Store_ConfirmPurchasePlural"
        };
        let vars = vec![
            (id("Product"), SkillVar::Text(s.render_wire(product))),
            (id("Quantity"), SkillVar::UInt(u64::from(quantity))),
            (id("Trade"), SkillVar::Text(s.render_wire(trade))),
            (id("Cost"), SkillVar::UInt(u64::from(cost))),
        ];
        let text = s.render_string(&UiString::ui(id(string_id), vars));
        self.confirm_context = dialogs.make_dialog(tree, &text, DIALOG_CONFIRMATION, false);
    }

    /// `gmShopUI::RecvNotice_CloseConfirmationDialog`: the answer to the purchase's confirmation.
    pub fn recv_close_confirmation_dialog(&mut self, context: u32, yes: bool) {
        if self.shown && context == self.confirm_context {
            self.confirm_context = 0;
            self.requests.push(ShopRequest::PurchaseConfirmed(yes));
        }
    }

    /// The confirmation's answers among the dialogs' outcomes.
    pub fn dialog_outcomes(&mut self, outcomes: &[DialogOutcome]) {
        for outcome in outcomes {
            if let DialogOutcome::Confirmation { context, yes } = *outcome {
                self.recv_close_confirmation_dialog(context, yes);
            }
        }
    }

    /// `gmShopUI::RecvNotice_StackUIDone`: the stack's amount buys the item.
    pub fn recv_stack_ui_done(&mut self, context: u32, amount: u32) {
        if self.shown && context == self.stack_context {
            self.finish_drag_from_shop(amount);
        }
    }

    /// `gmShopUI::RecvNotice_HandleDragFrom`: an item dragged from the shop (this panel's context)
    /// into the pack's `slot` is bought there, when it is one of the shop's sales.
    pub fn recv_handle_drag_from(
        &mut self,
        tree: &mut UiTree,
        s: &dyn ShopSource,
        item: u64,
        slot: u32,
        context: u32,
        max: &mut dyn FnMut(u64) -> u32,
    ) {
        if self.shown && context == self.context && self.sale_element(item).is_some() {
            self.purchase(tree, s, item, slot, max);
        }
    }

    /// `gmShopUI::PurchaseItem`: `item` is bought into `slot`. Refused when the sale is not on
    /// the list or is unavailable; otherwise it is bought at once (one), or the stack dialog asks
    /// how many (more, up to `max`, the most the player may buy).
    pub(crate) fn purchase(
        &mut self,
        tree: &mut UiTree,
        s: &dyn ShopSource,
        item: u64,
        slot: u32,
        max: &mut dyn FnMut(u64) -> u32,
    ) {
        self.item_to_purchase = item;
        self.target_slot = slot;
        let most = max(item);
        if most == 0 {
            self.say(s, "ID_Store_CantAfford");
            return;
        }
        if most != 1 {
            self.stack_context = tree.request_stack_context();
            let text = s.render_string(&UiString::ui(id("ID_Store_ShopStack"), Vec::new()));
            self.stack_calls.push(ShopStackCall::Init {
                context: self.stack_context,
                text,
                max: most,
            });
            return;
        }
        self.finish_drag_from_shop(1);
    }

    /// `gmShopUI::FinishDragFromShop`: the purchase of `quantity` of the item, into its slot.
    fn finish_drag_from_shop(&mut self, quantity: u32) {
        if self.item_to_purchase != INVALID {
            self.requests.push(ShopRequest::Purchase {
                item: self.item_to_purchase,
                quantity,
                slot: self.target_slot,
            });
        }
    }

    /// The purchase's refusal, as a message to the chat (`CM_Character::SendNotice_DisplayMessage`).
    fn say(&mut self, s: &dyn ShopSource, string: &str) {
        self.messages
            .push(s.render_string(&UiString::ui(id(string), Vec::new())));
    }

    /// `gmShopUI::Select`: the sale's element becomes the selection (its state changes), the
    /// purchase button is ghosted without one, and the item is selected for the player.
    pub(crate) fn select(&mut self, tree: &mut UiTree, row: Option<usize>) {
        if row != self.selected {
            if let Some(old) = self.selected {
                tree.set_state(old, STATELESS);
            }
            if let Some(new) = row {
                tree.set_state(new, id(SELECTED));
            }
            self.selected = row;
        }
        if let Some(purchase) = self.child(tree, "PurchaseButton") {
            tree.set_bool(purchase, BUTTON_GHOSTED, self.selected.is_none());
        }
        if let Some(item) = self.selected_item() {
            self.requests.push(ShopRequest::Select(item));
        }
    }

    /// `gmShopUI::ListenToMessage` for the global loop (`UIMESSAGE_GLOBAL_LOOP`, at `now`): a
    /// hidden shop is destroyed after its time, a typed sort text asks for its page, and the page
    /// buttons are live again after their wait.
    pub fn tick(&mut self, tree: &mut UiTree, now: f64) {
        if !self.shown {
            if self.field.is_some() && DORMANT_LIMIT < now - self.dormant_since {
                self.destroy_managed(tree);
            }
            return;
        }
        if self.next_update == 0.0 {
            if self.sort_edited && self.sort_time < now {
                self.next_update = now + PAGE_WAIT;
                self.sort_edited = false;
                let text = self
                    .child(tree, "SortText")
                    .and_then(|sort| tree.get(sort)?.text())
                    .unwrap_or_default();
                self.requests.push(ShopRequest::Sales(parse_page(&text)));
            }
        } else if self.next_update < now {
            self.next_update = 0.0;
            for name in ["ShopNextButton", "ShopPrevButton"] {
                if let Some(button) = self.child(tree, name) {
                    if !tree.bool_attribute(button, id("unavailable")) {
                        tree.set_bool(button, BUTTON_GHOSTED, false);
                    }
                }
            }
        }
    }

    /// `gmShopUI::ListenToMessage`: the close, sort, page and purchase buttons, and a press on a
    /// sale. `now` is the frame's clock; `max` is `GetMaxPurchaseQuantity` for an item.
    pub fn message(
        &mut self,
        tree: &mut UiTree,
        s: &dyn ShopSource,
        dialogs: &mut DialogFactory,
        message: &UiMessage,
        now: f64,
        max: &mut dyn FnMut(u64) -> u32,
    ) {
        if !self.shown {
            return;
        }
        let element = message.element_id;
        if element == id("ShopCloseButton") {
            if message.message == MSG_BUTTON_RELEASE_ON {
                self.requests.push(ShopRequest::LeaveStore);
                self.show(tree, dialogs, false, now);
            }
        } else if element == id("SortText") && SORT_TEXT_MESSAGES.contains(&message.message) {
            self.sort_edited = true;
            self.sort_time = now;
            if message.message != tokenize("deactivated") {
                self.sort_time += SORT_WAIT;
            }
        } else if element == id("ShopNextButton") && message.message == MSG_BUTTON_RELEASE_ON {
            self.requests.push(ShopRequest::NextSales);
            self.page_wait(tree, now);
        } else if element == id("ShopPrevButton") && message.message == MSG_BUTTON_RELEASE_ON {
            self.requests.push(ShopRequest::PrevSales);
            self.page_wait(tree, now);
        } else if element == id("PurchaseButton") && message.message == MSG_BUTTON_RELEASE_ON {
            if let Some(item) = self.selected_item() {
                self.purchase(tree, s, item, 0, max);
            }
        } else if element == id("Sale") && message.message == MSG_MOUSE_PRESS {
            self.sale_pressed(tree, message);
        }
    }

    /// `ListenToMessage`'s page request tail: both buttons ghosted until the wait is over.
    fn page_wait(&mut self, tree: &mut UiTree, now: f64) {
        self.next_update = now + PAGE_WAIT;
        if let (Some(next), Some(prev)) = (
            self.child(tree, "ShopNextButton"),
            self.child(tree, "ShopPrevButton"),
        ) {
            tree.set_bool(next, BUTTON_GHOSTED, true);
            tree.set_bool(prev, BUTTON_GHOSTED, true);
        }
    }

    /// A press on a sale: the sale is selected; the examine press opens the examination panel.
    fn sale_pressed(&mut self, tree: &mut UiTree, message: &UiMessage) {
        let Some(row) = message.element else {
            return;
        };
        if message.data != SALE_SELECT && message.data != SALE_EXAMINE {
            return;
        }
        if self.selected != Some(row) {
            self.select(tree, Some(row));
        }
        if message.data == SALE_EXAMINE {
            self.requests.push(ShopRequest::OpenExamination);
        }
    }
}

/// `PStringBase::to_int32`'s page: the sort text's number, clamped at zero.
fn parse_page(text: &str) -> u32 {
    let trimmed = text.trim_start();
    let (negative, digits) = match trimmed.as_bytes().first() {
        Some(b'-') => (true, &trimmed[1..]),
        Some(b'+') => (false, &trimmed[1..]),
        _ => (false, trimmed),
    };
    let value: i64 = digits
        .bytes()
        .take_while(u8::is_ascii_digit)
        .fold(0i64, |v, d| {
            (v * 10 + i64::from(d - b'0')).min(i64::from(i32::MAX))
        });
    if negative || value < 0 {
        0
    } else {
        value as u32
    }
}
