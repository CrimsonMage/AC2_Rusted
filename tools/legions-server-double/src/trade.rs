//! The secure trade against the double: the player's trade-window requests answered as the
//! server's `WM_Trade` handlers answer them, and the partner's side (FELLOW) driven by a test
//! through [`State::partner_offer`] and [`State::partner_accept`].
//!
//! Each change that follows an accept first sends `BeAcceptanceCleared` (the acceptances
//! lapse); when both sides accept, the offered items swap containers and the trade closes with
//! `BeClosed` (kind [`TRADE_COMPLETED`]).

use ac2_net::protocol::{ScriptRequest, StringInfo, TradeMessage};

use crate::social::FELLOW;
use crate::State;

/// FELLOW's name as the partner's message carries it.
const PARTNER_NAME: &str = "Birch";

/// `BeClosed`'s kind when the player closes the trade without a deal.
pub const TRADE_CANCELLED: u32 = 0;
/// `BeClosed`'s kind when both sides accepted and the items swapped.
pub const TRADE_COMPLETED: u32 = 1;

/// One open trade: its partner, what each side offers and who has accepted.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Trade {
    /// The player's id (the side the player offers on).
    pub player: u64,
    /// The other party while a trade is open.
    pub partner: Option<u64>,
    /// The player's offers: item and amount.
    pub player_offers: Vec<(u64, u32)>,
    /// The partner's offers.
    pub partner_offers: Vec<(u64, u32)>,
    pub player_accepted: bool,
    pub partner_accepted: bool,
}

impl Trade {
    fn offers(&mut self, side: u64) -> &mut Vec<(u64, u32)> {
        if side == self.player {
            &mut self.player_offers
        } else {
            &mut self.partner_offers
        }
    }

    fn accepted(&mut self, side: u64) -> &mut bool {
        if side == self.player {
            &mut self.player_accepted
        } else {
            &mut self.partner_accepted
        }
    }
}

impl State {
    /// The requests the trade window sends (`WM_Trade` stubs), answered.
    pub(crate) fn answer_trade(&mut self, request: &ScriptRequest) {
        match request {
            ScriptRequest::OpenTrade(other) => self.open_trade(*other),
            ScriptRequest::CloseTrade => self.end_trade(self.player(), TRADE_CANCELLED),
            ScriptRequest::AcceptTrade(_) => self.accept_trade(self.player()),
            ScriptRequest::DeclineTrade => self.decline_trade(),
            ScriptRequest::OfferItem { item, amount } => self.offer(self.player(), *item, *amount),
            ScriptRequest::RevokeItem(item) => self.revoke(*item),
            _ => {}
        }
    }

    /// FELLOW offers `item` (`amount` of it) in the open trade; the partner's side of
    /// `WM_Trade` 0xD.
    pub fn partner_offer(&mut self, item: u64, amount: u32) {
        self.offer(FELLOW, item, amount);
    }

    /// FELLOW takes `item` back out of the open trade (`WM_Trade` 0x7 for the partner, which the
    /// server answers with `BeRevoked`).
    pub fn partner_revoke(&mut self, item: u64) {
        let Some(at) = self
            .trade
            .partner_offers
            .iter()
            .position(|(i, _)| *i == item)
        else {
            return;
        };
        self.clear_acceptance();
        self.trade.partner_offers.remove(at);
        self.send_trade(TradeMessage::Revoked {
            source: FELLOW,
            item,
        });
    }

    /// FELLOW accepts the open trade (`WM_Trade` 0xF for the partner).
    pub fn partner_accept(&mut self) {
        self.accept_trade(FELLOW);
    }

    /// `WM_Trade` 0x10 and 0x1: a trade with `other` opens, if FELLOW is who the player asks.
    fn open_trade(&mut self, other: u64) {
        if self.trade.partner.is_some() || other != FELLOW {
            return;
        }
        let player = self.player();
        self.trade = Trade {
            player,
            partner: Some(other),
            ..Trade::default()
        };
        self.send_trade(TradeMessage::Registered {
            initiator: self.player(),
            partner: other,
            partner_name: StringInfo {
                literal: Some(PARTNER_NAME.into()),
                ..StringInfo::default()
            },
        });
        self.send_trade(TradeMessage::Opened);
    }

    /// An offer by `side`: the item must be in that side's containers and not on offer yet.
    /// A change after an accept clears the acceptances first.
    fn offer(&mut self, side: u64, item: u64, amount: u32) {
        if self.trade.partner.is_none() || self.trade.offers(side).iter().any(|(i, _)| *i == item) {
            return;
        }
        let held = self
            .script_world
            .contents
            .get(&side)
            .is_some_and(|(_, items)| items.contains(&item));
        if !held {
            return;
        }
        self.clear_acceptance();
        self.trade.offers(side).push((item, amount));
        self.send_trade(TradeMessage::Offered {
            source: side,
            item,
            amount,
        });
    }

    /// `WM_Trade` 0x7: the player takes `item` back out of the trade.
    fn revoke(&mut self, item: u64) {
        let Some(at) = self
            .trade
            .player_offers
            .iter()
            .position(|(i, _)| *i == item)
        else {
            return;
        };
        self.clear_acceptance();
        self.trade.player_offers.remove(at);
        self.send_trade(TradeMessage::Revoked {
            source: self.player(),
            item,
        });
    }

    /// `WM_Trade` 0xF: `side` accepts; when both have, the trade completes.
    fn accept_trade(&mut self, side: u64) {
        if self.trade.partner.is_none() {
            return;
        }
        *self.trade.accepted(side) = true;
        self.send_trade(TradeMessage::Accepted(side));
        if self.trade.player_accepted && self.trade.partner_accepted {
            self.complete_trade();
        }
    }

    /// `WM_Trade` 0xC: the player declines; the trade ends with no exchange.
    fn decline_trade(&mut self) {
        if self.trade.partner.is_none() {
            return;
        }
        self.send_trade(TradeMessage::Declined(self.player()));
        self.trade = Trade::default();
    }

    /// `WM_Trade` 0xB: the acceptances lapse after a change.
    fn clear_acceptance(&mut self) {
        if self.trade.player_accepted || self.trade.partner_accepted {
            self.trade.player_accepted = false;
            self.trade.partner_accepted = false;
            self.send_trade(TradeMessage::AcceptanceCleared);
        }
    }

    /// `WM_Trade` 0xA: the trade closes (`kind`).
    fn end_trade(&mut self, source: u64, kind: u32) {
        if self.trade.partner.is_none() {
            return;
        }
        self.send_trade(TradeMessage::Closed { source, kind });
        self.trade = Trade::default();
    }

    /// Both accepted: each side's offers move into the other's container, both containers are
    /// listed again, and the trade closes.
    fn complete_trade(&mut self) {
        let player_offers = std::mem::take(&mut self.trade.player_offers);
        let partner_offers = std::mem::take(&mut self.trade.partner_offers);
        self.script_world.contents.entry(self.player()).or_default();
        self.script_world.contents.entry(FELLOW).or_default();
        for (item, _) in player_offers {
            self.inventory_move(item, FELLOW, -1);
        }
        for (item, _) in partner_offers {
            self.inventory_move(item, self.player(), -1);
        }
        self.send_contents(self.player());
        self.send_contents(FELLOW);
        self.end_trade(self.player(), TRADE_COMPLETED);
    }

    /// Queues a `WM_Trade` answer to the player's script.
    fn send_trade(&mut self, message: TradeMessage) {
        self.send_script_call(message.function(), &message.arguments());
    }
}
