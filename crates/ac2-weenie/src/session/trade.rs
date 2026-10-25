//! The secure trade's entry points: the calls the trade window makes on the player's weenie
//! (`gmCWInterface::OpenTradeNegotiations`, `CloseTradeNegotiations`, `AcceptTrade`,
//! `DeclineTrade`, `OfferItemForTrade`, `RevokeItemFromTrade`), as their IFuns send them to the
//! `WM_Trade` stubs, and the server's `WM_Trade` answers as the player's script raises them
//! (`CM_Trade::SendNotice_Trade_*`).

use ac2_net::protocol::{PackedReference, ScriptRequest, TradeMessage};

use super::{OnlineSession, SessionEvent};
use crate::GameplayNotice;

impl OnlineSession {
    /// `gmCWInterface::OpenTradeNegotiations` (IFun 0xB400B8) -> `WM_Trade` stub 0x2F: ask
    /// `other` to trade.
    pub fn open_trade(&mut self, other: u64) {
        self.send(ScriptRequest::OpenTrade(other).to_event());
    }

    /// `gmCWInterface::CloseTradeNegotiations` (IFun 0xB400B9) -> stub 0x3E.
    pub fn close_trade(&mut self) {
        self.send(ScriptRequest::CloseTrade.to_event());
    }

    /// `gmCWInterface::AcceptTrade` (IFun 0xB400BA) -> stub 0x35. The stub packs the accepting
    /// side's script instance (`CWInterface`); with the script VM attached the client runs the
    /// IFun on the VM and this is not called. Without a VM the session has no such graph, so it
    /// sends an empty reference.
    pub fn accept_trade(&mut self) {
        let empty = PackedReference {
            ids: Vec::new(),
            entries: Vec::new(),
        };
        self.send(ScriptRequest::AcceptTrade(empty).to_event());
    }

    /// `gmCWInterface::DeclineTrade` (IFun 0xB400BB) -> stub 0x23.
    pub fn decline_trade(&mut self) {
        self.send(ScriptRequest::DeclineTrade.to_event());
    }

    /// `gmCWInterface::OfferItemForTrade` (IFun 0xB400BC) -> stub 0x15: offer `item` in the
    /// trade with `amount` of it.
    pub fn offer_item_for_trade(&mut self, item: u64, amount: u32) {
        self.send(ScriptRequest::OfferItem { item, amount }.to_event());
    }

    /// `gmCWInterface::RevokeItemFromTrade` (IFun 0xB400BD) -> stub 0x3F: take `item` back out
    /// of the trade.
    pub fn revoke_item_from_trade(&mut self, item: u64) {
        self.send(ScriptRequest::RevokeItem(item).to_event());
    }

    /// A `WM_Trade` answer the player's script takes; `None` for another function id.
    pub(super) fn receive_trade(
        &self,
        function: u32,
        arguments: &[u8],
    ) -> Option<Vec<SessionEvent>> {
        match TradeMessage::decode(function, arguments)? {
            Ok(message) => Some(vec![SessionEvent::Notice(trade_notice(message))]),
            Err(error) => Some(vec![SessionEvent::Rejected(format!(
                "script function {function:#010x}: {error}"
            ))]),
        }
    }
}

/// The `CM_Trade::SendNotice_Trade_*` notice a server message raises.
fn trade_notice(message: TradeMessage) -> GameplayNotice {
    match message {
        TradeMessage::Registered {
            initiator,
            partner,
            partner_name,
        } => GameplayNotice::TradeRegistered {
            initiator,
            partner,
            partner_name,
        },
        TradeMessage::Opened => GameplayNotice::TradeOpened,
        TradeMessage::Closed { source, kind } => GameplayNotice::TradeClosed { source, kind },
        TradeMessage::Accepted(source) => GameplayNotice::TradeAccepted(source),
        TradeMessage::Declined(source) => GameplayNotice::TradeDeclined(source),
        TradeMessage::Reset => GameplayNotice::TradeReset,
        TradeMessage::AcceptanceCleared => GameplayNotice::TradeAcceptanceCleared,
        TradeMessage::Offered {
            source,
            item,
            amount,
        } => GameplayNotice::TradeOffered {
            source,
            item,
            amount,
        },
        TradeMessage::Revoked { source, item } => GameplayNotice::TradeRevoked { source, item },
    }
}
