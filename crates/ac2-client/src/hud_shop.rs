//! The shop panel's glue (`gmShopUI`): the store's notices and the presses go to
//! [`ac2_ui::ShopUi`], its requests become the gameplay commands ([`shop_commands`]), and its
//! stack dialog calls go to the stack dialog ([`shop_stack_calls`]). The frames call these from
//! `hud_player`, which routes the notices and the messages in the order the frameworks get them.

use ac2_ui::{
    CatalogRequest, CatalogUi, ConsignmentRequest, ConsignmentSource, ConsignmentUi, DialogFactory,
    NewSaleRequest, NewSaleSource, NewSaleUi, ShopRequest, ShopSource, ShopStackCall, ShopUi,
    StackUi, UiTree,
};
use ac2_weenie::GameplayNotice;

use crate::gameplay::Command;
#[cfg(feature = "window")]
use crate::ui_sources::ClientPlayer;

/// `INVALID_IID`: a stack's from and target when it splits a store's sale.
const INVALID: u64 = 0;

#[cfg(feature = "window")]
impl ConsignmentSource for ClientPlayer<'_> {}

#[cfg(feature = "window")]
impl ShopSource for ClientPlayer<'_> {
    fn render_wire(&self, info: &ac2_net::protocol::StringInfo) -> String {
        self.render_info(info)
    }
}

/// One of the store's notices for the shop panel (`gmShopUI::RecvNotice_*`); `now` is the frame's
/// clock. The notices the panel does not take pass by.
pub fn shop_notice(
    panel: &mut ShopUi,
    tree: &mut UiTree,
    dialogs: &mut DialogFactory,
    source: &dyn ShopSource,
    now: f64,
    notice: &GameplayNotice,
) {
    match notice {
        GameplayNotice::ViewStore(shop) => panel.recv_view_store(tree, dialogs, source, shop, now),
        GameplayNotice::UpdateStore(shop) => panel.recv_update_store(tree, source, shop),
        GameplayNotice::UpdateSale { item, stock } => {
            panel.recv_update_sale(tree, source, *item, *stock)
        }
        GameplayNotice::LeaveStore => panel.recv_leave_store(tree, dialogs, now),
        GameplayNotice::ConfirmPurchase {
            product,
            quantity,
            trade_name,
            cost,
        } => panel
            .recv_confirm_purchase(tree, dialogs, source, product, *quantity, trade_name, *cost),
        GameplayNotice::SelectionChanged(new) => panel.recv_selection_changed(tree, *new),
        _ => {}
    }
}

/// One of the catalog's notices (`gmShopCatalogUI::RecvNotice_ViewCatalog` and `_LeaveCatalog`);
/// `now` is the frame's clock. The notices the catalog does not take pass by.
pub fn catalog_notice(
    panel: &mut CatalogUi,
    tree: &mut UiTree,
    source: &dyn ShopSource,
    now: f64,
    notice: &GameplayNotice,
) {
    match notice {
        GameplayNotice::ViewCatalog { header, shops } => {
            panel.recv_view_catalog(tree, source, header, shops, now)
        }
        GameplayNotice::LeaveCatalog => panel.recv_leave_catalog(tree, now),
        _ => {}
    }
}

/// The catalog's requests as the gameplay commands (`gmCWInterface`'s calls).
pub fn catalog_commands(requests: Vec<CatalogRequest>) -> Vec<Command> {
    requests
        .into_iter()
        .map(|request| match request {
            CatalogRequest::LeaveCatalog => Command::LeaveCatalog,
            CatalogRequest::EnterStore(store) => Command::EnterStore(store),
            CatalogRequest::EnterConsignment => Command::EnterConsignment,
        })
        .collect()
}

/// The new-sale dialog's requests as the gameplay commands (`gmCWInterface`'s calls): the sale is
/// `RequestSell`, the close button `RequestLeaveStore`.
pub fn new_sale_commands(requests: Vec<NewSaleRequest>) -> Vec<Command> {
    requests
        .into_iter()
        .map(|request| match request {
            NewSaleRequest::Sell {
                item,
                quantity,
                price,
            } => Command::RequestSell {
                item,
                quantity,
                price,
            },
            NewSaleRequest::LeaveStore => Command::LeaveStore,
        })
        .collect()
}

/// The shop's requests as the gameplay commands (`gmCWInterface`'s calls). An examination
/// request is not a command: the frames open the examination panel for it.
pub fn shop_commands(requests: Vec<ShopRequest>) -> Vec<Command> {
    requests
        .into_iter()
        .filter_map(|request| {
            Some(match request {
                ShopRequest::Sales(index) => Command::RequestSales(index),
                ShopRequest::NextSales => Command::NextSales,
                ShopRequest::PrevSales => Command::PrevSales,
                ShopRequest::LeaveStore => Command::LeaveStore,
                ShopRequest::Select(item) => Command::Select(Some(item)),
                ShopRequest::Purchase {
                    item,
                    quantity,
                    slot,
                } => Command::PurchaseItem {
                    item,
                    quantity,
                    slot,
                },
                ShopRequest::PurchaseConfirmed(answer) => Command::PurchaseConfirmed(answer),
                ShopRequest::OpenExamination => return None,
            })
        })
        .collect()
}

/// The shop's stack dialog calls: a purchase of a stack asks how many (`InitStackUI`), and the
/// dialog closes with the shop (`CloseStackUI`).
pub fn shop_stack_calls(calls: Vec<ShopStackCall>, tree: &mut UiTree, stack: Option<&mut StackUi>) {
    let Some(stack) = stack else {
        return;
    };
    for call in calls {
        match call {
            ShopStackCall::Init { context, text, max } => {
                stack.init(tree, context, &text, 0, max, INVALID, INVALID)
            }
            ShopStackCall::Close(context) => stack.close_stack_ui(tree, context),
        }
    }
}

/// One of the consignment's notices (`gmShopConsignmentUI::RecvNotice_ViewConsignment`,
/// `_UpdateConsignment`, and the server's `LeaveConsignment`); `now` is the frame's clock.
pub fn consignment_notice(
    panel: &mut ConsignmentUi,
    tree: &mut UiTree,
    dialogs: &mut DialogFactory,
    source: &dyn ConsignmentSource,
    now: f64,
    notice: &GameplayNotice,
) {
    match notice {
        GameplayNotice::ViewConsignment(shop) => {
            panel.recv_view_consignment(tree, dialogs, source, shop, now)
        }
        GameplayNotice::UpdateConsignment(shop) => {
            panel.recv_update_consignment(tree, source, shop)
        }
        GameplayNotice::LeaveConsignment => panel.recv_leave_consignment(tree, dialogs, now),
        _ => {}
    }
}

/// The consignment's requests: the new-sale dialog opens for `NewSale` (`InitNewSale`), the
/// examination is asked for (returns true), and the rest are gameplay commands.
pub fn consignment_requests(
    requests: Vec<ConsignmentRequest>,
    new_sale: Option<&mut NewSaleUi>,
    tree: &mut UiTree,
    dialogs: &mut DialogFactory,
    source: &dyn NewSaleSource,
    now: f64,
    commands: &mut Vec<Command>,
) -> bool {
    let mut open_examination = false;
    let mut new_sale = new_sale;
    for request in requests {
        match request {
            ConsignmentRequest::NewSale { item, quantity } => {
                if let Some(panel) = new_sale.as_deref_mut() {
                    panel.recv_init_new_sale(tree, dialogs, source, item, quantity, now);
                }
            }
            ConsignmentRequest::OpenExamination => open_examination = true,
            ConsignmentRequest::LeaveConsignment => commands.push(Command::LeaveConsignment),
            ConsignmentRequest::Select(item) => commands.push(Command::Select(Some(item))),
            ConsignmentRequest::Collect { target, profits } => {
                commands.push(Command::RequestCollect { target, profits })
            }
            ConsignmentRequest::Restock {
                item,
                quantity,
                target,
            } => commands.push(Command::RequestRestock {
                item,
                quantity,
                target,
            }),
        }
    }
    open_examination
}
