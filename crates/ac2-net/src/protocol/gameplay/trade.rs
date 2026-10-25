//! The secure trade's server messages (`WM_Trade`, package 0x215): the forwarders the server's
//! answers reach in the Legions script library, which call `CPlayer` and raise the
//! `CM_Trade::SendNotice_Trade_*` notices. The client's requests are
//! [`ScriptRequest`](super::ScriptRequest) variants.
//!
//! The function ids are the library's own (package 0x215 maps them, `Resolver::GetFunctionOffset`):
//! each forwarder's body was traced to its `CPlayer` function and on to the notice's efun, and
//! the forwarders were then run on the VM (`CPlayer` answers each with its notice). The wire
//! packs a handler's arguments in reverse parameter order (`WSL` pushes the last parameter first,
//! as the stubs do for the requests), so each message lists its values last parameter first.

use super::{function, literal, Args};
use crate::protocol::pack::PackValue;
use crate::protocol::string_info::StringInfo;
use crate::protocol::ProtocolError;

/// A server message the player's script takes from `WM_Trade`.
#[derive(Debug, Clone, PartialEq)]
pub enum TradeMessage {
    /// [`function::TRADE_REGISTERED`]: the initiator, the partner and the partner's name
    /// (`CM_Trade::SendNotice_Trade_BeRegistered`).
    Registered {
        initiator: u64,
        partner: u64,
        partner_name: StringInfo,
    },
    /// [`function::TRADE_OPENED`] (`CM_Trade::SendNotice_Trade_BeOpened`).
    Opened,
    /// [`function::TRADE_CLOSED`]: the source and the end's kind (`BeClosed`).
    Closed { source: u64, kind: u32 },
    /// [`function::TRADE_ACCEPTED`]: who accepted (`BeAccepted`).
    Accepted(u64),
    /// [`function::TRADE_DECLINED`]: who declined (`BeDeclined`).
    Declined(u64),
    /// [`function::TRADE_RESET`] (`BeReset`).
    Reset,
    /// [`function::TRADE_ACCEPTANCE_CLEARED`] (`BeAcceptanceCleared`).
    AcceptanceCleared,
    /// [`function::TRADE_OFFERED`]: who offered `item` and the amount (`BeOffered`).
    Offered { source: u64, item: u64, amount: u32 },
    /// [`function::TRADE_REVOKED`]: who took `item` back (`BeRevoked`).
    Revoked { source: u64, item: u64 },
}

impl TradeMessage {
    /// The `WM_Trade` function the server calls the player's script with.
    pub fn function(&self) -> u32 {
        match self {
            Self::Registered { .. } => function::TRADE_REGISTERED,
            Self::Opened => function::TRADE_OPENED,
            Self::Closed { .. } => function::TRADE_CLOSED,
            Self::Accepted(_) => function::TRADE_ACCEPTED,
            Self::Declined(_) => function::TRADE_DECLINED,
            Self::Reset => function::TRADE_RESET,
            Self::AcceptanceCleared => function::TRADE_ACCEPTANCE_CLEARED,
            Self::Offered { .. } => function::TRADE_OFFERED,
            Self::Revoked { .. } => function::TRADE_REVOKED,
        }
    }

    /// The packed arguments, last parameter first (the wire order).
    pub fn arguments(&self) -> Vec<PackValue> {
        match self {
            Self::Registered {
                initiator,
                partner,
                partner_name,
            } => vec![
                literal(partner_name),
                PackValue::LongInt(*partner),
                PackValue::LongInt(*initiator),
            ],
            Self::Opened | Self::Reset | Self::AcceptanceCleared => Vec::new(),
            Self::Closed { source, kind } => {
                vec![PackValue::Int(*kind), PackValue::LongInt(*source)]
            }
            Self::Accepted(source) | Self::Declined(source) => {
                vec![PackValue::LongInt(*source)]
            }
            Self::Offered {
                source,
                item,
                amount,
            } => vec![
                PackValue::Int(*amount),
                PackValue::LongInt(*item),
                PackValue::LongInt(*source),
            ],
            Self::Revoked { source, item } => {
                vec![PackValue::LongInt(*item), PackValue::LongInt(*source)]
            }
        }
    }

    /// The server's side: a trade message from its function id and packed arguments, or `None`
    /// for another function id. The forwarder 0x4C raises the same notice as 0x51.
    pub fn decode(function_id: u32, arguments: &[u8]) -> Option<Result<Self, ProtocolError>> {
        let decode = |read: fn(&mut Args) -> Result<Self, ProtocolError>| {
            let mut args = Args::new(arguments)?;
            let message = read(&mut args)?;
            args.finish()?;
            Ok(message)
        };
        Some(match function_id {
            function::TRADE_REGISTERED => decode(|a| {
                let partner_name = a.string_info()?;
                Ok(Self::Registered {
                    partner: a.long()?,
                    initiator: a.long()?,
                    partner_name,
                })
            }),
            function::TRADE_OPENED => decode(|_| Ok(Self::Opened)),
            function::TRADE_CLOSED => decode(|a| {
                Ok(Self::Closed {
                    kind: a.int()?,
                    source: a.long()?,
                })
            }),
            function::TRADE_ACCEPTED => decode(|a| Ok(Self::Accepted(a.long()?))),
            function::TRADE_DECLINED => decode(|a| Ok(Self::Declined(a.long()?))),
            function::TRADE_RESET => decode(|_| Ok(Self::Reset)),
            function::TRADE_ACCEPTANCE_CLEARED | function::TRADE_ACCEPTANCE_CLEARED_ALT => {
                decode(|_| Ok(Self::AcceptanceCleared))
            }
            function::TRADE_OFFERED => decode(|a| {
                Ok(Self::Offered {
                    amount: a.int()?,
                    item: a.long()?,
                    source: a.long()?,
                })
            }),
            function::TRADE_REVOKED => decode(|a| {
                Ok(Self::Revoked {
                    item: a.long()?,
                    source: a.long()?,
                })
            }),
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::pack::pack_values;

    fn words(w: &[u32]) -> Vec<u8> {
        w.iter().flat_map(|w| w.to_le_bytes()).collect()
    }

    fn long(v: u64) -> Vec<u32> {
        vec![2, v as u32, (v >> 32) as u32]
    }

    #[test]
    fn offered_packs_the_amount_item_and_source() {
        let message = TradeMessage::Offered {
            source: 0x2000_0000_0000_0001,
            item: 0x8000_0002_0000_0021,
            amount: 3,
        };
        let mut expected = vec![1, 3];
        expected.extend(long(0x8000_0002_0000_0021));
        expected.extend(long(0x2000_0000_0000_0001));
        assert_eq!(pack_values(&message.arguments()), words(&expected));
        assert_eq!(message.function(), 0x8215_0044);
        assert_eq!(
            TradeMessage::decode(0x8215_0044, &words(&expected)),
            Some(Ok(message))
        );
    }

    #[test]
    fn revoked_and_closed_pack_their_values() {
        let revoked = TradeMessage::Revoked {
            source: 7,
            item: 0x8000_0002_0000_0022,
        };
        let mut expected = long(0x8000_0002_0000_0022);
        expected.extend(long(7));
        assert_eq!(pack_values(&revoked.arguments()), words(&expected));
        let closed = TradeMessage::Closed { source: 7, kind: 1 };
        let mut expected = vec![1, 1];
        expected.extend(long(7));
        assert_eq!(pack_values(&closed.arguments()), words(&expected));
        assert_eq!(
            TradeMessage::decode(0x8215_0053, &words(&expected)),
            Some(Ok(closed))
        );
    }

    #[test]
    fn registered_packs_the_partner_name_first() {
        let name = StringInfo {
            literal: Some("Birch".into()),
            ..StringInfo::default()
        };
        let message = TradeMessage::Registered {
            initiator: 1,
            partner: 2,
            partner_name: name.clone(),
        };
        let mut expected = pack_values(&[literal(&name)]);
        expected.extend(words(&long(2)));
        expected.extend(words(&long(1)));
        assert_eq!(pack_values(&message.arguments()), expected);
        assert_eq!(
            TradeMessage::decode(0x8215_0046, &expected),
            Some(Ok(message))
        );
    }

    #[test]
    fn nothing_arguments_and_extra_values() {
        assert_eq!(
            pack_values(&TradeMessage::Opened.arguments()),
            Vec::<u8>::new()
        );
        assert_eq!(
            TradeMessage::decode(0x8215_004E, &[]),
            Some(Ok(TradeMessage::Opened))
        );
        assert_eq!(
            TradeMessage::decode(0x8215_004C, &[]),
            Some(Ok(TradeMessage::AcceptanceCleared))
        );
        assert!(TradeMessage::decode(0x8215_004E, &words(&[1, 0]))
            .unwrap()
            .is_err());
        assert!(TradeMessage::decode(0x8215_0011, &[]).is_none());
    }
}
