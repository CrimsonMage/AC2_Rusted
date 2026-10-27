//! `gmShopConsignmentUI`: the storekeeper's consignment panel (`Consignment` of the gameplay
//! layout), a child framework made when the server opens the consignment (`ViewConsignment`).
//! It lists the consigned sales (`UpdateSales`: one `Sale` row each, with its item's icon and
//! description, its cost, and its status: expired, sold or active), pages them (`UpdatePosition`
//! ghosts the page buttons at either end), and selects a row (`Select`, which asks the player to
//! select the item). `CollectButton` collects the selected sale (`RequestCollect`), `NewButton`
//! opens the new-sale dialog empty (`CM_Store::SendNotice_InitNewSale`), and an item dropped on the
//! list restocks a sale it stacks with (`HandleListBoxDrop`: the stack dialog, then the restock's
//! confirmation, `RequestRestock`). `ShopCloseButton` leaves the consignment
//! (`RequestLeaveConsignment`); the server's `UpdateConsignment` replaces the sales, and
//! `LeaveConsignment` closes the panel from the server's side.
//!
//! The entry points are [`ConsignmentUi::recv_view_consignment`], [`ConsignmentUi::recv_update_consignment`],
//! [`ConsignmentUi::recv_leave_consignment`] (the notices), [`ConsignmentUi::message`] (the presses and
//! the drop), [`ConsignmentUi::recv_stack_ui_done`] and [`ConsignmentUi::recv_close_confirmation_dialog`]
//! (the dialogs' answers) and [`ConsignmentUi::tick`] (the global loop). The requests go to
//! [`ConsignmentUi::take_requests`].
//!
//! Not modelled: the 3D portrait of the player (`InitPortrait`, `SyncPortrait`, the viewport's
//! camera), the mount state, the help page, and the examination panel's own state (the request is
//! [`ConsignmentRequest::OpenExamination`]). A page is seven rows (`UpdatePosition`'s count).

use std::sync::Arc;

use ac2_weenie::{Shop, ShopSale};

use crate::dialog::{DialogFactory, DialogOutcome, DIALOG_CONFIRMATION};
use crate::drag::MSG_STOP_DRAG_ITEM;
use crate::element::UiTree;
use crate::item_display::{dragged_item, is_item_copy, InventorySource};
use crate::layout::LayoutDesc;
use crate::mouse::UiMessage;
use crate::shop::{id, ShopSource, ShopStackCall, INVALID, SELECTED};
use crate::token::{BUTTON_GHOSTED, MSG_BUTTON_RELEASE_ON, MSG_MOUSE_PRESS, STATELESS};
use crate::ui_string::UiString;

/// `ListenToMessage`'s global loop: a hidden consignment waits this long before it is destroyed.
const DORMANT_LIMIT: f64 = 120.0;
/// The rows in one page (`UpdatePosition`'s count of visible rows).
const PAGE: i32 = 7;
/// The row's press data that selects (0x40000004) and that examines (0x40000005).
const ROW_SELECT: i32 = 0x4000_0004;
const ROW_EXAMINE: i32 = 0x4000_0005;

/// What the consignment asks of the player: the `gmCWInterface` calls it makes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsignmentRequest {
    /// `gmCWInterface::RequestLeaveConsignment`.
    LeaveConsignment,
    /// `gmCWInterface::SetSelection(item)`: a sale was selected.
    Select(u64),
    /// `gmCWInterface::RequestCollect(target, profits)`: the selected sale's profits or items.
    Collect { target: u32, profits: bool },
    /// `gmCWInterface::RequestRestock(item, quantity, target)`.
    Restock {
        item: u64,
        quantity: u32,
        target: u32,
    },
    /// `CM_Store::SendNotice_InitNewSale(item, quantity)`: the new-sale dialog opens with the
    /// item (`INVALID` and 0 from the new button, which opens it empty).
    NewSale { item: u64, quantity: u32 },
    /// `CM_UI::SendNotice_OpenExaminationPanel`: a sale was examined.
    OpenExamination,
}

/// What the consignment reads from the client: the inventory and the strings it renders.
pub trait ConsignmentSource: InventorySource + ShopSource {
    /// The item's definition id, for `CanStackWith`'s match against a sale's item. `None` when the
    /// client does not know it, which makes no sale stack.
    fn item_did(&self, _item: u64) -> Option<u32> {
        None
    }
}

/// One row: its sale's id (the ordinal), its item and its element.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Row {
    pub sale: u32,
    pub item: u64,
    pub element: usize,
}

/// `gmShopConsignmentUI`'s state.
#[derive(Debug, Clone)]
pub struct ConsignmentUi {
    pub(crate) layout: Arc<LayoutDesc>,
    /// The panel's root, while its managed elements exist (`m_pShopConsignmentField`).
    pub(crate) field: Option<usize>,
    /// `IsShown`.
    pub(crate) shown: bool,
    /// `m_selectedElement`: the selected row.
    pub(crate) selected: Option<usize>,
    /// The rows, in the list's order, and the sales they show.
    pub(crate) rows: Vec<Row>,
    pub(crate) sales: Vec<ShopSale>,
    /// `m_chosenItem`, `m_chosenQuantity` and `m_chosenTarget` (the sale restocked; 0: none).
    pub(crate) item: u64,
    pub(crate) quantity: u32,
    pub(crate) target: u32,
    /// `m_stackContext` and `m_confirmationContext` (0: none).
    pub(crate) stack_context: u32,
    pub(crate) confirm_context: u32,
    /// `m_timeDormant`, on the global loop's clock.
    pub(crate) dormant_since: f64,
    requests: Vec<ConsignmentRequest>,
    stack_calls: Vec<ShopStackCall>,
}

impl ConsignmentUi {
    /// `gmShopConsignmentUI::gmShopConsignmentUI`: hidden, with no managed elements.
    pub fn create(gameplay: &Arc<LayoutDesc>) -> Self {
        Self {
            layout: Arc::clone(gameplay),
            field: None,
            shown: false,
            selected: None,
            rows: Vec::new(),
            sales: Vec::new(),
            item: INVALID,
            quantity: 0,
            target: 0,
            stack_context: 0,
            confirm_context: 0,
            dormant_since: 0.0,
            requests: Vec::new(),
            stack_calls: Vec::new(),
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
        let selected = self.selected?;
        self.rows
            .iter()
            .find(|row| row.element == selected)
            .map(|row| row.item)
    }

    /// The sales the list shows, in the list's order (their ids).
    pub fn sale_ids(&self) -> Vec<u32> {
        self.rows.iter().map(|row| row.sale).collect()
    }

    /// The requests the consignment made since the last call, for the client to send.
    pub fn take_requests(&mut self) -> Vec<ConsignmentRequest> {
        std::mem::take(&mut self.requests)
    }

    /// The stack dialog calls the consignment made since the last call.
    pub fn take_stack_calls(&mut self) -> Vec<ShopStackCall> {
        std::mem::take(&mut self.stack_calls)
    }

    /// The element `name` under the panel.
    fn child(&self, tree: &UiTree, name: &str) -> Option<usize> {
        tree.child_recursive(self.field?, id(name))
    }

    /// `gmShopConsignmentUI::CreateManagedElements`: the `Consignment` root, once.
    fn create_managed(&mut self, tree: &mut UiTree) -> bool {
        if self.field.is_none() {
            self.field = tree.create_root(Arc::clone(&self.layout), id("Consignment"));
        }
        self.field.is_some()
    }

    /// `gmShopConsignmentUI::DestroyManagedElements`: the root and the rows go.
    fn destroy_managed(&mut self, tree: &mut UiTree) {
        if let Some(field) = self.field.take() {
            tree.remove_root(field);
        }
        self.selected = None;
        self.rows.clear();
        self.sales.clear();
    }

    /// `gmShopConsignmentUI::Show`: shown makes the managed elements and shows them. Hidden closes
    /// the stack dialog and the confirmation, and waits to be destroyed.
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

    /// `gmShopConsignmentUI::RecvNotice_ViewConsignment`: the panel opens with the storekeeper's
    /// name and description, and its sales.
    pub fn recv_view_consignment(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        s: &dyn ConsignmentSource,
        shop: &ac2_weenie::Shop,
        now: f64,
    ) {
        self.show(tree, dialogs, true, now);
        if self.field.is_none() {
            return;
        }
        if let Some(name) = self.child(tree, "ConsignmentName") {
            tree.set_text(name, &s.render_wire(&shop.name));
        }
        if let Some(desc) = self.child(tree, "ConsignmentDesc") {
            tree.set_text(desc, &s.render_wire(&shop.desc));
        }
        self.update_sales(tree, s, &shop.sales);
    }

    /// `gmShopConsignmentUI::RecvNotice_UpdateConsignment`: while shown, the sales are replaced.
    pub fn recv_update_consignment(
        &mut self,
        tree: &mut UiTree,
        s: &dyn ConsignmentSource,
        shop: &Shop,
    ) {
        if self.shown {
            self.update_sales(tree, s, &shop.sales);
        }
    }

    /// The server's close: the panel hides.
    pub fn recv_leave_consignment(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        now: f64,
    ) {
        self.show(tree, dialogs, false, now);
    }

    /// `gmShopConsignmentUI::UpdateSales`: the list is rebuilt, the selection kept when its item
    /// is still listed, and the page buttons follow the list.
    pub fn update_sales(
        &mut self,
        tree: &mut UiTree,
        s: &dyn ConsignmentSource,
        sales: &[ShopSale],
    ) {
        let Some(list) = self.child(tree, "ItemList") else {
            return;
        };
        let kept = self.selected_item();
        if self.selected.is_some() {
            self.select(tree, None);
        }
        tree.list_flush(list);
        self.rows.clear();
        self.sales = sales.to_vec();
        tree.list_multi_op_start(list);
        for sale in sales {
            let Some(desc) = self.layout.element(id("Sale")).cloned() else {
                break;
            };
            let Some(row) = tree.create_element(Arc::clone(&self.layout), desc, false) else {
                continue;
            };
            self.post_sale(tree, s, sale, row);
            tree.list_add_item(list, row);
            self.rows.push(Row {
                sale: sale.ordinal,
                item: sale.item,
                element: row,
            });
            if kept == Some(sale.item) && sale.item != INVALID {
                self.select(tree, Some(row));
            }
        }
        tree.list_multi_op_complete(list);
        self.ghost_collect(tree);
        self.update_position(tree);
    }

    /// `gmShopConsignmentUI::PostSale`: a row's icon (with the item's description as its tooltip),
    /// its cost's icon and its cost. The `Sale` element has no name or status text, so the status
    /// (expired, sold or active) is kept on the row's data and not shown.
    fn post_sale(&self, tree: &mut UiTree, s: &dyn ConsignmentSource, sale: &ShopSale, row: usize) {
        if let Some(icon) = tree.child_recursive(row, id("ItemIcon")) {
            if sale.icon_did != 0 {
                tree.set_image(icon, sale.icon_did, 2);
            }
            tree.set_tooltip(icon, Some(s.render_wire(&sale.item_desc)));
        }
        if let Some(icon) = tree.child_recursive(row, id("CostIcon")) {
            if sale.cost_icon != 0 {
                tree.set_image(icon, sale.cost_icon, 2);
            }
            tree.set_tooltip(icon, Some(s.render_wire(&sale.cost_desc)));
        }
        if let Some(cost) = tree.child_recursive(row, id("CostQuantity")) {
            tree.set_text(cost, &format!("{}", sale.cost_quantity));
        }
        tree.set_attribute_id(row, id("Token_SaleID"), sale.ordinal);
        tree.set_attribute_id(row, id("Token_ItemID"), sale.item as u32);
    }

    /// `gmShopConsignmentUI::UpdatePosition`: the previous page is ghosted at the top of the list,
    /// the next page at its end.
    fn update_position(&self, tree: &mut UiTree) {
        let Some(list) = self.child(tree, "ItemList") else {
            return;
        };
        let top = tree.list_top_row(list);
        let count = self.rows.len() as i32;
        if let Some(prev) = self.child(tree, "PrevButton") {
            tree.set_bool(prev, BUTTON_GHOSTED, top == 0);
        }
        if let Some(next) = self.child(tree, "NextButton") {
            tree.set_bool(next, BUTTON_GHOSTED, count - PAGE <= top);
        }
    }

    /// The collect button is ghosted while no sale is selected.
    fn ghost_collect(&self, tree: &mut UiTree) {
        if let Some(collect) = self.child(tree, "CollectButton") {
            tree.set_bool(collect, BUTTON_GHOSTED, self.selected.is_none());
        }
    }

    /// `gmShopConsignmentUI::Select`: the row's selection state moves and the item is selected for
    /// the player (`SetSelection`).
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
        self.ghost_collect(tree);
        if let Some(item) = self.selected_item() {
            self.requests.push(ConsignmentRequest::Select(item));
        }
    }

    /// `gmShopConsignmentUI::CanStackWith`: a sale takes a stack of an item of its kind, when the
    /// item stacks and the sale is not expired.
    fn can_stack_with(&self, s: &dyn ConsignmentSource, sale: u32, item: u64) -> bool {
        let Some(slot) = s.item(item) else {
            return false;
        };
        let Some(sold) = self.sales.iter().find(|sold| sold.ordinal == sale) else {
            return false;
        };
        slot.stackable
            && slot.max_stack > 1
            && sold.status != 2
            && s.item_did(item) == Some(sold.item_did)
    }

    /// `gmShopConsignmentUI::HandleListBoxDrop`: an item dropped on the list is restocked into the
    /// sale under the mouse when it stacks with it, else it asks for a new sale. A stack of more
    /// than one asks how many first.
    pub fn handle_list_box_drop(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        s: &dyn ConsignmentSource,
        copy: usize,
    ) {
        let Some(dragged) = dragged_item(tree, copy) else {
            return;
        };
        let Some(slot) = s.item(dragged.item) else {
            return;
        };
        let under = tree
            .child_recursive(self.field.unwrap_or(0), id("ItemList"))
            .and_then(|list| tree.list_item_under_mouse(list))
            .and_then(|row| self.rows.iter().find(|r| r.element == row).copied());
        let target = match under {
            Some(row) if self.can_stack_with(s, row.sale, dragged.item) => row.sale,
            _ => 0,
        };
        self.item = dragged.item;
        self.target = target;
        self.stack_context = tree.request_stack_context();
        if slot.quantity == 1 {
            self.recv_stack_ui_done(tree, dialogs, s, self.stack_context, 1);
            return;
        }
        let string = if target != 0 {
            "ID_Store_ConsignmentRestockStack"
        } else {
            "ID_Store_ConsignmentSellStack"
        };
        let text = s.render_string(&UiString::ui(id(string), Vec::new()));
        self.stack_calls.push(ShopStackCall::Init {
            context: self.stack_context,
            text,
            max: slot.quantity,
        });
    }

    /// `gmShopConsignmentUI::RecvNotice_StackUIDone`: the stack's amount restocks the chosen sale
    /// after a confirmation (`ID_Store_ConsignmentConfirmRestock`); with no sale to restock, it
    /// opens the new-sale dialog with the item (the admin 7148 path; the Legions stack-done handler
    /// was not located in the decompile).
    pub fn recv_stack_ui_done(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        s: &dyn ConsignmentSource,
        context: u32,
        amount: u32,
    ) {
        if !self.shown || context != self.stack_context || amount == 0 {
            return;
        }
        self.quantity = amount;
        if self.target == 0 {
            self.requests.push(ConsignmentRequest::NewSale {
                item: self.item,
                quantity: amount,
            });
            return;
        }
        let text = s.render_string(&UiString::ui(
            id("ID_Store_ConsignmentConfirmRestock"),
            Vec::new(),
        ));
        self.confirm_context = dialogs.make_dialog(tree, &text, DIALOG_CONFIRMATION, false);
    }

    /// `gmShopConsignmentUI::RecvNotice_CloseConfirmationDialog`: a yes to the restock asks for it.
    pub fn recv_close_confirmation_dialog(&mut self, context: u32, yes: bool) {
        if !self.shown || !yes || context != self.confirm_context {
            return;
        }
        self.confirm_context = 0;
        if self.item != INVALID && self.quantity != 0 && self.target != 0 {
            self.requests.push(ConsignmentRequest::Restock {
                item: self.item,
                quantity: self.quantity,
                target: self.target,
            });
        }
    }

    /// The confirmation's answer among the dialogs' outcomes.
    pub fn dialog_outcomes(&mut self, outcomes: &[DialogOutcome]) {
        for outcome in outcomes {
            if let DialogOutcome::Confirmation { context, yes } = *outcome {
                self.recv_close_confirmation_dialog(context, yes);
            }
        }
    }

    /// `gmShopConsignmentUI::ListenToMessage`: the close button leaves the consignment, the new
    /// button opens the new-sale dialog empty, the collect button collects the selected sale, the
    /// page buttons page, a press on a row selects it (and examines it on the second), and a drop
    /// on the list is handled.
    pub fn message(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        s: &dyn ConsignmentSource,
        message: &UiMessage,
        now: f64,
    ) {
        if !self.shown {
            return;
        }
        let element = message.element_id;
        if message.message == MSG_BUTTON_RELEASE_ON {
            if element == id("ShopCloseButton") {
                self.requests.push(ConsignmentRequest::LeaveConsignment);
                self.show(tree, dialogs, false, now);
            } else if element == id("NewButton") {
                self.requests.push(ConsignmentRequest::NewSale {
                    item: INVALID,
                    quantity: 0,
                });
            } else if element == id("CollectButton") {
                if let Some(row) = self
                    .selected
                    .and_then(|r| self.rows.iter().find(|x| x.element == r))
                    .copied()
                {
                    let profits = self
                        .sales
                        .iter()
                        .any(|sold| sold.ordinal == row.sale && sold.status == 3);
                    self.requests.push(ConsignmentRequest::Collect {
                        target: row.sale,
                        profits,
                    });
                }
            } else if element == id("NextButton") {
                self.page(tree, PAGE);
            } else if element == id("PrevButton") {
                self.page(tree, -PAGE);
            }
            return;
        }
        if element == id("Sale") && message.message == MSG_MOUSE_PRESS {
            if let Some(row) = message.element {
                if (message.data == ROW_SELECT || message.data == ROW_EXAMINE)
                    && self.selected != Some(row)
                {
                    self.select(tree, Some(row));
                }
                if message.data == ROW_EXAMINE {
                    self.requests.push(ConsignmentRequest::OpenExamination);
                }
            }
            return;
        }
        if element == id("ItemList") && message.message == MSG_STOP_DRAG_ITEM && message.data != 0 {
            if let Some(copy) = message.item {
                if is_item_copy(tree, copy) {
                    self.handle_list_box_drop(tree, dialogs, s, copy);
                }
            }
        }
    }

    /// Scrolls the list by `rows` and updates the page buttons.
    fn page(&mut self, tree: &mut UiTree, rows: i32) {
        let Some(list) = self.child(tree, "ItemList") else {
            return;
        };
        let top = tree.list_top_row(list);
        tree.list_scroll_to_row(list, top + rows);
        self.update_position(tree);
    }

    /// `ListenToMessage` for the global loop (`now`): a hidden panel is destroyed after its time.
    pub fn tick(&mut self, tree: &mut UiTree, now: f64) {
        if !self.shown && self.field.is_some() && DORMANT_LIMIT < now - self.dormant_since {
            self.destroy_managed(tree);
        }
    }
}
