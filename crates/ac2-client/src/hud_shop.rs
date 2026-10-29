//! The shop panel's glue (`gmShopUI`): the requests of the store, the catalog, the consignment and
//! the new-sale dialog become the gameplay commands ([`shop_commands`] and the rest). The panels'
//! notices and stack calls are in [`ac2_ui::shop_notice`] and its siblings. The frames call these
//! from `hud_player_*`, which routes the notices and the messages in the order the frameworks get
//! them.

use ac2_ui::{CatalogRequest, ConsignmentSource, NewSaleRequest, ShopRequest, ShopSource};

use crate::gameplay::Command;
#[cfg(feature = "window")]
use crate::ui_sources::ClientPlayer;

#[cfg(feature = "window")]
impl ConsignmentSource for ClientPlayer<'_> {
    /// `CWeenieObject::m_initData.m_did`: the entity DID the object was created with.
    fn item_did(&self, item: u64) -> Option<u32> {
        self.client.borrow().object_entity(item)
    }
}

#[cfg(feature = "window")]
impl ShopSource for ClientPlayer<'_> {
    fn render_wire(&self, info: &ac2_net::protocol::StringInfo) -> String {
        self.render_info(info)
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
