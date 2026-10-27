//! `gmShopNewSaleUI`: the new-sale dialog of the consignment (`NewSaleField` of the gameplay
//! layout), a child framework made when the consignment's new button opens it (the
//! `CM_Store::SendNotice_InitNewSale` the consignment raises, with an empty item) or when an item
//! is dropped on the consignment's list. The player drops one item on the sale slot (`NewSale`,
//! `HandleDragDrop`: a stack of more than one asks how many), types the price (`NewSalePrice`) and
//! presses `AddSaleButton` (`AddSale`): a price below one, no item or no quantity is refused with a
//! message; otherwise a confirmation asks for the consignment fee (`GetConsignmentCharge`). The
//! answer's yes sends `gmCWInterface::RequestSell(item, quantity, price)` and hides the dialog.
//! `ShopCloseButton` hides it and leaves the store (`RequestLeaveStore`).
//!
//! The entry points are [`NewSaleUi::recv_init_new_sale`] (the notice), [`NewSaleUi::message`] (the
//! presses and the drop), [`NewSaleUi::recv_stack_ui_done`] and
//! [`NewSaleUi::recv_close_confirmation_dialog`] (the dialogs' answers) and [`NewSaleUi::tick`] (the
//! global loop). What the dialog asks of the player is queued for [`NewSaleUi::take_requests`] and
//! the stack dialogs it wants for [`NewSaleUi::take_stack_calls`].
//!
//! Not modelled: the sale slot's tooltip on mouse over (`ID_Shop_NewSale_Tooltip`) and the help
//! page (`KeyStone::OpenHelp`); the item's name is the inventory's name at its own quantity.

use std::sync::Arc;

use crate::dialog::{DialogFactory, DialogOutcome, DIALOG_CONFIRMATION, DIALOG_MESSAGE};
use crate::drag::MSG_STOP_DRAG_ITEM;
use crate::element::UiTree;
use crate::item_display::{dragged_item, is_item_copy, InventorySource};
use crate::layout::LayoutDesc;
use crate::mouse::UiMessage;
use crate::shop::{id, ShopStackCall, INVALID};
use crate::skill_tree::SkillVar;
use crate::token::MSG_BUTTON_RELEASE_ON;
use crate::ui_string::{UiString, UiStrings};

/// `ListenToMessage`'s global loop: a hidden new-sale dialog waits this long before it is destroyed.
const DORMANT_LIMIT: f64 = 120.0;

/// What the new-sale dialog asks of the player: the `gmCWInterface` calls it makes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NewSaleRequest {
    /// `gmCWInterface::RequestSell(item, quantity, price)`: the consignment's sale, confirmed.
    Sell {
        item: u64,
        quantity: u32,
        price: u32,
    },
    /// `gmCWInterface::RequestLeaveStore`: the close button.
    LeaveStore,
}

/// What the new-sale dialog reads from the client: the inventory (the item's icon, name and
/// stack) and the strings it renders.
pub trait NewSaleSource: InventorySource + UiStrings {}

impl<T: InventorySource + UiStrings + ?Sized> NewSaleSource for T {}

/// `gmShopNewSaleUI`'s state.
#[derive(Debug, Clone)]
pub struct NewSaleUi {
    pub(crate) layout: Arc<LayoutDesc>,
    /// The dialog's root, while its managed elements exist (`m_pShopNewSaleField`).
    pub(crate) field: Option<usize>,
    /// `IsShown`.
    pub(crate) shown: bool,
    /// `m_chosenItem` (`INVALID` for none) and `m_chosenQuantity`.
    pub(crate) item: u64,
    pub(crate) quantity: u32,
    /// `m_chosenPrice`, set when the sale is confirmed for asking.
    pub(crate) price: u32,
    /// `m_stackContext` and `m_confirmationContext` (0: none).
    pub(crate) stack_context: u32,
    pub(crate) confirm_context: u32,
    /// `m_timeDormant`, on the global loop's clock.
    pub(crate) dormant_since: f64,
    requests: Vec<NewSaleRequest>,
    stack_calls: Vec<ShopStackCall>,
}

impl NewSaleUi {
    /// `gmShopNewSaleUI::gmShopNewSaleUI`: hidden, with no managed elements; the gameplay layout
    /// is kept for them.
    pub fn create(gameplay: &Arc<LayoutDesc>) -> Self {
        Self {
            layout: Arc::clone(gameplay),
            field: None,
            shown: false,
            item: INVALID,
            quantity: 0,
            price: 0,
            stack_context: 0,
            confirm_context: 0,
            dormant_since: 0.0,
            requests: Vec::new(),
            stack_calls: Vec::new(),
        }
    }

    /// The dialog's root, while its managed elements exist.
    pub fn field(&self) -> Option<usize> {
        self.field
    }

    /// `IsShown`.
    pub fn is_shown(&self) -> bool {
        self.shown
    }

    /// The item for sale (`INVALID` for none).
    pub fn chosen_item(&self) -> u64 {
        self.item
    }

    /// The quantity for sale.
    pub fn chosen_quantity(&self) -> u32 {
        self.quantity
    }

    /// The requests the dialog made since the last call, for the client to send.
    pub fn take_requests(&mut self) -> Vec<NewSaleRequest> {
        std::mem::take(&mut self.requests)
    }

    /// The stack dialog calls the dialog made since the last call.
    pub fn take_stack_calls(&mut self) -> Vec<ShopStackCall> {
        std::mem::take(&mut self.stack_calls)
    }

    /// The element `name` under the dialog.
    fn child(&self, tree: &UiTree, name: &str) -> Option<usize> {
        tree.child_recursive(self.field?, id(name))
    }

    /// `gmShopNewSaleUI::CreateManagedElements`: the `NewSaleField` root, once.
    fn create_managed(&mut self, tree: &mut UiTree) -> bool {
        if self.field.is_none() {
            self.field = tree.create_root(Arc::clone(&self.layout), id("NewSaleField"));
        }
        self.field.is_some()
    }

    /// `gmShopNewSaleUI::DestroyManagedElements`: the root goes; the choice is cleared with it.
    fn destroy_managed(&mut self, tree: &mut UiTree) {
        if let Some(field) = self.field.take() {
            tree.remove_root(field);
        }
    }

    /// `gmShopNewSaleUI::Show`: shown makes the managed elements and shows them. Hidden closes the
    /// stack dialog and the confirmation, and waits to be destroyed.
    pub fn show(&mut self, tree: &mut UiTree, dialogs: &mut DialogFactory, shown: bool, now: f64) {
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

    /// `CM_Store::SendNotice_InitNewSale`'s receiver (`gmShopNewSaleUI::RecvNotice_InitNewSale`):
    /// the dialog opens with the item and its quantity (none, for the empty dialog).
    pub fn recv_init_new_sale(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        s: &dyn NewSaleSource,
        item: u64,
        quantity: u32,
        now: f64,
    ) {
        self.item = item;
        self.quantity = quantity;
        self.show(tree, dialogs, true, now);
        self.update(tree, s);
    }

    /// `gmShopNewSaleUI::Update`: the item's icon and name, and the price cleared. With no item the
    /// icon and name are cleared.
    pub fn update(&mut self, tree: &mut UiTree, s: &dyn NewSaleSource) {
        if self.field.is_none() {
            return;
        }
        let slot = if self.item == INVALID {
            None
        } else {
            s.item(self.item)
        };
        if let Some(icon) = self.child(tree, "NewSaleIcon") {
            match &slot {
                Some(slot) => {
                    let image = slot.icon_layers.first().copied().unwrap_or(slot.icon);
                    tree.set_image(icon, image, 2);
                }
                None => tree.clear_image(icon),
            }
        }
        if let Some(desc) = self.child(tree, "NewSaleDesc") {
            let name = slot.as_ref().map_or(String::new(), |s| s.name.clone());
            tree.set_text(desc, &name);
        }
        if let Some(price) = self.child(tree, "NewSalePrice") {
            tree.set_text(price, "");
        }
    }

    /// `gmShopNewSaleUI::RecvNotice_StackUIDone`: the stack's amount is the quantity for sale.
    pub fn recv_stack_ui_done(
        &mut self,
        tree: &mut UiTree,
        s: &dyn NewSaleSource,
        context: u32,
        amount: u32,
    ) {
        if self.shown && context == self.stack_context && amount != 0 {
            self.quantity = amount;
            self.update(tree, s);
        }
    }

    /// `gmShopNewSaleUI::HandleDragDrop`: the dropped item becomes the sale. One of it is chosen at
    /// once; more ask how many first (the stack dialog, `ID_Store_ConsignmentSellStack`).
    pub fn handle_drag_drop(&mut self, tree: &mut UiTree, s: &dyn NewSaleSource, copy: usize) {
        let Some(dragged) = dragged_item(tree, copy) else {
            return;
        };
        let Some(slot) = s.item(dragged.item) else {
            return;
        };
        self.item = dragged.item;
        self.stack_context = tree.request_stack_context();
        if slot.quantity == 1 {
            self.recv_stack_ui_done(tree, s, self.stack_context, 1);
            return;
        }
        let text = s.render_string(&UiString::ui(
            id("ID_Store_ConsignmentSellStack"),
            Vec::new(),
        ));
        self.stack_calls.push(ShopStackCall::Init {
            context: self.stack_context,
            text,
            max: slot.quantity,
        });
    }

    /// `gmShopNewSaleUI::AddSale`: the price in the field is checked, then the sale is asked for
    /// with its consignment fee (`charge`, `gmCWInterface::GetConsignmentCharge`). A refusal is a
    /// message dialog; the question is shown only for a known item.
    pub fn add_sale(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        s: &dyn NewSaleSource,
        charge: &mut dyn FnMut() -> f32,
    ) {
        if self.item == INVALID {
            self.message_dialog(tree, dialogs, s, "ID_Shop_NewSale_Err_NoItem");
            return;
        }
        if self.quantity == 0 {
            self.message_dialog(tree, dialogs, s, "ID_Shop_NewSale_Err_NoQuantity");
            return;
        }
        let typed = self
            .child(tree, "NewSalePrice")
            .and_then(|price| tree.get(price)?.text())
            .unwrap_or_default();
        let price = parse_price(&typed);
        if price < 1 {
            self.message_dialog(tree, dialogs, s, "ID_Shop_NewSale_Err_NoPrice");
            return;
        }
        self.price = u32::try_from(price).unwrap_or(u32::MAX);
        let Some(slot) = s.item(self.item) else {
            return;
        };
        let fee = consignment_fee(self.price, self.quantity, charge());
        let string = if self.quantity == 1 {
            "ID_Store_ConsignmentConfirmSellSingular"
        } else {
            "ID_Store_ConsignmentConfirmSellPlural"
        };
        // `gmShopNewSaleUI::AddSale`'s confirmation variables: `product` (the item's name), `cost`
        // (the consignment fee) and `price` (the price the player typed), by their tokens.
        let vars = vec![
            (id("product"), SkillVar::Text(slot.name)),
            (id("cost"), SkillVar::UInt(u64::from(fee))),
            (id("price"), SkillVar::UInt(u64::from(self.price))),
        ];
        let text = s.render_string(&UiString::ui(id(string), vars));
        self.confirm_context = dialogs.make_dialog(tree, &text, DIALOG_CONFIRMATION, false);
    }

    /// A refusal of the sale, as a message dialog.
    fn message_dialog(
        &self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        s: &dyn NewSaleSource,
        string: &str,
    ) {
        let text = s.render_string(&UiString::ui(id(string), Vec::new()));
        dialogs.make_dialog(tree, &text, DIALOG_MESSAGE, false);
    }

    /// `gmShopNewSaleUI::RecvNotice_CloseConfirmationDialog`: a yes to the sale's confirmation asks
    /// for the sale and hides the dialog (`Show(0)`).
    pub fn recv_close_confirmation_dialog(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        context: u32,
        yes: bool,
        now: f64,
    ) {
        if !self.shown || !yes || context != self.confirm_context {
            return;
        }
        self.confirm_context = 0;
        if self.item != INVALID && self.quantity != 0 && self.price != 0 {
            self.requests.push(NewSaleRequest::Sell {
                item: self.item,
                quantity: self.quantity,
                price: self.price,
            });
            self.show(tree, dialogs, false, now);
        }
    }

    /// The confirmation's answer among the dialogs' outcomes.
    pub fn dialog_outcomes(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        outcomes: &[DialogOutcome],
        now: f64,
    ) {
        for outcome in outcomes {
            if let DialogOutcome::Confirmation { context, yes } = *outcome {
                self.recv_close_confirmation_dialog(tree, dialogs, context, yes, now);
            }
        }
    }

    /// `gmShopNewSaleUI::ListenToMessage`: `ShopCloseButton` hides the dialog and leaves the store,
    /// `AddSaleButton` adds the sale, and an item dropped on `NewSale` becomes the sale.
    pub fn message(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        s: &dyn NewSaleSource,
        message: &UiMessage,
        now: f64,
        charge: &mut dyn FnMut() -> f32,
    ) {
        if !self.shown {
            return;
        }
        let element = message.element_id;
        if message.message == MSG_BUTTON_RELEASE_ON {
            if element == id("ShopCloseButton") {
                self.show(tree, dialogs, false, now);
                self.requests.push(NewSaleRequest::LeaveStore);
            } else if element == id("AddSaleButton") {
                self.add_sale(tree, dialogs, s, charge);
            }
            return;
        }
        if element == id("NewSale") && message.message == MSG_STOP_DRAG_ITEM && message.data != 0 {
            if let Some(copy) = message.item {
                if is_item_copy(tree, copy) {
                    self.handle_drag_drop(tree, s, copy);
                }
            }
        }
    }

    /// `gmShopNewSaleUI::ListenToMessage` for the global loop (`now`): a hidden dialog is destroyed
    /// after its time.
    pub fn tick(&mut self, tree: &mut UiTree, now: f64) {
        if !self.shown && self.field.is_some() && DORMANT_LIMIT < now - self.dormant_since {
            self.destroy_managed(tree);
        }
    }
}

/// The price as `wcstol` with base 0 reads it: a `0x` hex, a `0` octal, else decimal; a text that
/// reads no number is zero.
pub fn parse_price(text: &str) -> i64 {
    let text = text.trim();
    let (negative, digits) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text.strip_prefix('+').unwrap_or(text)),
    };
    let (radix, body) = if let Some(hex) = digits
        .strip_prefix("0x")
        .or_else(|| digits.strip_prefix("0X"))
    {
        (16, hex)
    } else if digits.len() > 1 && digits.starts_with('0') {
        (8, &digits[1..])
    } else {
        (10, digits)
    };
    let end = body
        .find(|c: char| !c.is_digit(radix))
        .unwrap_or(body.len());
    let value = i64::from_str_radix(&body[..end], radix).unwrap_or(0);
    if negative {
        -value
    } else {
        value
    }
}

/// The consignment fee of a sale: the price times the quantity times the charge, rounded up, and
/// at least one (`gmShopNewSaleUI::AddSale`'s `ceil` and `_ftol2`, evaluated in double precision).
pub fn consignment_fee(price: u32, quantity: u32, charge: f32) -> u32 {
    let fee = (f64::from(price) * f64::from(quantity) * f64::from(charge)).ceil();
    if fee.is_nan() || fee < 1.0 {
        return 1;
    }
    if fee >= f64::from(u32::MAX) {
        return u32::MAX;
    }
    fee as u32
}
