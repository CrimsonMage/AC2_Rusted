//! The store, catalog and consignment panels' notice glue (`gmShopUI::RecvNotice_*`,
//! `gmShopCatalogUI::RecvNotice_ViewCatalog` and `_LeaveCatalog`, `gmShopConsignmentUI`), and
//! the stack dialog calls a store's purchase asks for (`InitStackUI`, `CloseStackUI`). The client
//! hands each frame's notices here and turns the panels' requests into gameplay commands.

use ac2_weenie::GameplayNotice;

use crate::catalog::CatalogUi;
use crate::consignment::{ConsignmentSource, ConsignmentUi};
use crate::dialog::DialogFactory;
use crate::element::UiTree;
use crate::shop::{ShopSource, ShopStackCall, ShopUi, INVALID};
use crate::stack::StackUi;

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
