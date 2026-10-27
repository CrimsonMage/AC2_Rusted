//! The shop panel's glue (`gmShopUI`): the store's notices and the presses go to
//! [`ac2_ui::ShopUi`], its requests become the gameplay commands ([`shop_commands`]), and its
//! stack dialog calls go to the stack dialog ([`shop_stack_calls`]). The frames call these from
//! `hud_player`, which routes the notices and the messages in the order the frameworks get them.

use ac2_ui::{
    CatalogRequest, CatalogUi, DialogFactory, ShopRequest, ShopSource, ShopStackCall, ShopUi,
    StackUi, UiTree,
};
use ac2_weenie::GameplayNotice;

use crate::gameplay::Command;
#[cfg(feature = "window")]
use crate::ui_sources::ClientPlayer;

/// `INVALID_IID`: a stack's from and target when it splits a store's sale.
const INVALID: u64 = 0;

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
