//! The store against the double: the server's answers to the player's store calls (`WM_Store`).
//! A store is opened by the server's `ViewStore` (the store and the storekeeper), which the
//! player's script answers with the shop; the player's leave comes back as `LeaveStore`. The
//! stock is not the double's: the client's script builds it from the store's own script data.

use ac2_net::protocol::{PackedReference, Purchase, ScriptRequest, StoreMessage, StoreView};

use crate::items::{item_object_bytes, ItemSpawn};
use crate::{State, PLAYER};

/// The first id of an item a purchase gives the player (`answer_purchase` counts up from it).
const PURCHASED_BASE: u64 = 0x8000_0000_0000_0300;
/// `CItem`, the package the double's potions use.
const ITEM_PACKAGE: u32 = 0x1AA9;

impl State {
    /// Opens the store `view` (a `StoreView` whose sales the server built) with the storekeeper
    /// `vendor`, as the server does (`WM_Store` 0x13).
    pub fn open_store(&mut self, view: StoreView, vendor: u64) {
        self.script_world.open_store = Some(view.clone());
        self.send_store(&StoreMessage::ViewStore { view, vendor });
    }

    /// Updates the open store to `view` with the storekeeper `vendor`, as the server does
    /// (`WM_Store` 0x2A, `UpdateStore`).
    pub fn update_store(&mut self, view: StoreView, vendor: u64) {
        self.script_world.open_store = Some(view.clone());
        self.send_store(&StoreMessage::UpdateStore { view, vendor });
    }

    /// The server's answer to the player's leave (`WM_Store` 0x10, `LeaveStore`).
    pub(crate) fn answer_leave_store(&mut self, request: &ScriptRequest) {
        if let ScriptRequest::LeaveStore(vendor) = request {
            self.send_store(&StoreMessage::LeaveStore { vendor: *vendor });
        }
    }

    /// The server's side of a purchase (`WM_Store` 0x14's `TransactionBlob`): a sale of the open
    /// store for one item costs its price, which comes off the player's money, and the player
    /// gets a new item for it in the pack. The double's rule, not retail's: the purchased object
    /// is a generic item (its name is not read from the entity), and a quantity other than one is
    /// not modelled (no purchase).
    pub(crate) fn answer_purchase(&mut self, blob: &PackedReference) {
        let Ok(purchase) = Purchase::from_reference(blob) else {
            self.script_errors
                .push("a purchase's transaction blob".to_owned());
            return;
        };
        let sale = self.script_world.open_store.as_ref().and_then(|view| {
            view.sales
                .iter()
                .find(|sale| sale.item == purchase.item)
                .cloned()
        });
        let Some(sale) = sale else { return };
        let cost = sale.cost as i32;
        if purchase.quantity != 1 || self.script_world.character.money < cost {
            return;
        }
        self.give_money(-cost);
        self.script_world.purchases += 1;
        let item = ItemSpawn::new(
            PURCHASED_BASE + self.script_world.purchases,
            ITEM_PACKAGE,
            "Purchased item",
            PLAYER,
        );
        self.send_event(&item_object_bytes(&item));
        self.script_world
            .contents
            .entry(PLAYER)
            .or_default()
            .1
            .push(item.id);
        self.send_contents(PLAYER);
    }

    fn send_store(&mut self, message: &StoreMessage) {
        self.send_script_call(message.function(), &message.arguments());
    }
}
