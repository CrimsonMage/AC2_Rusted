//! The store against the double: the server's answers to the player's store calls (`WM_Store`).
//! A store is opened by the server's `ViewStore` (the store and the storekeeper), which the
//! player's script answers with the shop; the player's leave comes back as `LeaveStore`. The
//! stock is not the double's: the client's script builds it from the store's own script data.

use ac2_net::protocol::{ScriptRequest, StoreMessage};

use crate::State;

impl State {
    /// Opens `store` (a StoreTemplate's WState DID, sent as a package reference) with the
    /// storekeeper `vendor`, as the server does (`WM_Store` 0x13).
    pub fn open_store(&mut self, store: u32, vendor: u64) {
        self.send_store(&StoreMessage::ViewStore { store, vendor });
    }

    /// The server's answer to the player's leave (`WM_Store` 0x10, `LeaveStore`).
    pub(crate) fn answer_leave_store(&mut self, request: &ScriptRequest) {
        if let ScriptRequest::LeaveStore(vendor) = request {
            self.send_store(&StoreMessage::LeaveStore { vendor: *vendor });
        }
    }

    fn send_store(&mut self, message: &StoreMessage) {
        self.send_script_call(message.function(), &message.arguments());
    }
}
