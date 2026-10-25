//! The store's messages (`WM_Store`, package 0x383A): the client's requests and the server's
//! answers. The requests are [`ScriptRequest`](super::ScriptRequest) variants whose stubs the
//! script library runs for `gmCWInterface`'s store calls; the answers are the forwarders the server
//! reaches, which call `CPlayer` and on to `CStoreSystem`, which raises the `CM_Store::SendNotice_*`
//! notices.
//!
//! As with the trade messages, a forwarder's parameters are packed last first (the forwarder passes
//! them to `CPlayer` in declared order, and the VM reads the first packed value as the last
//! parameter), so each message lists its values last parameter first.

use super::{function, Args};
use crate::protocol::pack::{entry_flags, PackValue, PackedEntry, PackedReference};
use crate::protocol::ProtocolError;

/// A server answer to the store (`WM_Store` forwarders).
#[derive(Debug, Clone, PartialEq)]
pub enum StoreMessage {
    /// [`function::STORE_VIEW`]: the store opened (`CM_Store::SendNotice_ViewStore`). `store` is
    /// the StoreTemplate's WState DID, sent as a package reference (see [`store_reference`]).
    ViewStore { store: u32, vendor: u64 },
    /// [`function::STORE_UPDATE`]: the store's sales changed (`CM_Store::SendNotice_UpdateStore`).
    UpdateStore { store: u32, vendor: u64 },
    /// [`function::STORE_LEFT`]: the storekeeper (`CM_Store::SendNotice_LeaveStore`).
    LeaveStore { vendor: u64 },
    /// [`function::STORE_UPDATE_SALE`]: an item's stock (`CM_Store::SendNotice_UpdateSale`).
    UpdateSale { vendor: u64, item: u64, stock: u32 },
}

impl StoreMessage {
    /// The function id the message is sent to.
    pub fn function(&self) -> u32 {
        match self {
            Self::ViewStore { .. } => function::STORE_VIEW,
            Self::UpdateStore { .. } => function::STORE_UPDATE,
            Self::LeaveStore { .. } => function::STORE_LEFT,
            Self::UpdateSale { .. } => function::STORE_UPDATE_SALE,
        }
    }

    /// The packed arguments, last parameter first (the wire order).
    pub fn arguments(&self) -> Vec<PackValue> {
        match self {
            Self::ViewStore { store, vendor } => vec![
                PackValue::Reference(store_reference(*store)),
                PackValue::LongInt(*vendor),
            ],
            Self::UpdateStore { store, vendor } => {
                vec![PackValue::LongInt(*vendor), PackValue::Int(*store)]
            }
            Self::LeaveStore { vendor } => vec![PackValue::LongInt(*vendor)],
            Self::UpdateSale {
                vendor,
                item,
                stock,
            } => vec![
                PackValue::LongInt(*vendor),
                PackValue::LongInt(*item),
                PackValue::Int(*stock),
            ],
        }
    }

    /// The server's side: a store message from its function id and packed arguments, or `None`
    /// for another function id.
    pub fn decode(function_id: u32, arguments: &[u8]) -> Option<Result<Self, ProtocolError>> {
        let decode = |read: fn(&mut Args) -> Result<Self, ProtocolError>| {
            let mut args = Args::new(arguments)?;
            let message = read(&mut args)?;
            args.finish()?;
            Ok(message)
        };
        Some(match function_id {
            function::STORE_VIEW => decode(|a| {
                let store = store_did(&a.reference()?)?;
                Ok(Self::ViewStore {
                    store,
                    vendor: a.long()?,
                })
            }),
            function::STORE_UPDATE => decode(|a| {
                let vendor = a.long()?;
                Ok(Self::UpdateStore {
                    store: a.int()?,
                    vendor,
                })
            }),
            function::STORE_LEFT => decode(|a| Ok(Self::LeaveStore { vendor: a.long()? })),
            function::STORE_UPDATE_SALE => decode(|a| {
                let vendor = a.long()?;
                let item = a.long()?;
                Ok(Self::UpdateSale {
                    vendor,
                    item,
                    stock: a.int()?,
                })
            }),
            _ => return None,
        })
    }
}

/// The store as the forwarder reads it: a package reference to the StoreTemplate's singleton,
/// packed as its WState DID (`PackedEntry::Singleton`, the entry the VM unpacks with
/// `GetWStateReference`). The forwarder's first parameter is a reference: the script compares it
/// with the null reference and calls the StoreTemplate's functions on it (`CStoreSystem`'s
/// ViewStore body, traced in the client's library).
pub fn store_reference(did: u32) -> PackedReference {
    PackedReference {
        ids: vec![1],
        entries: vec![PackedEntry::Singleton {
            flags: entry_flags::KIND_HEAP | entry_flags::SINGLETON,
            did,
        }],
    }
}

/// The StoreTemplate DID of a [`store_reference`]; anything else is not a store.
fn store_did(reference: &PackedReference) -> Result<u32, ProtocolError> {
    match reference.entries.as_slice() {
        [PackedEntry::Singleton { flags, did }]
            if flags & entry_flags::KIND_MASK == entry_flags::KIND_HEAP
                && flags & entry_flags::SINGLETON != 0 =>
        {
            Ok(*did)
        }
        _ => Err(ProtocolError::Invalid("store reference")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::pack::pack_values;

    fn words(w: &[u32]) -> Vec<u8> {
        w.iter().flat_map(|w| w.to_le_bytes()).collect()
    }

    /// `PackLInt`: tag 2, then the low and high words.
    fn long(v: u64) -> Vec<u32> {
        vec![2, v as u32, (v >> 32) as u32]
    }

    /// `PackInt`: tag 1, then the value.
    fn int(v: u32) -> Vec<u32> {
        vec![1, v]
    }

    /// `PackReference` of a one-entry singleton: tag 5, the id count, the id, the entry's flags
    /// (`KIND_HEAP | SINGLETON`) and its DID.
    fn singleton(did: u32) -> Vec<u32> {
        vec![5, 1, 1, 0x2000_0100, did]
    }

    #[test]
    fn view_store_packs_the_store_reference_then_the_storekeeper() {
        let message = StoreMessage::ViewStore {
            store: 0x6F00_33A3,
            vendor: 0x8000_0002_0000_0042,
        };
        let mut expected = singleton(0x6F00_33A3);
        expected.extend(long(0x8000_0002_0000_0042));
        assert_eq!(pack_values(&message.arguments()), words(&expected));
        assert_eq!(message.function(), 0xB83A_0013);
        assert_eq!(
            StoreMessage::decode(0xB83A_0013, &words(&expected)),
            Some(Ok(message))
        );
    }

    #[test]
    fn view_store_needs_a_store_reference_not_an_int() {
        let mut expected = int(0x7000_1234);
        expected.extend(long(0x8000_0002_0000_0042));
        assert!(StoreMessage::decode(0xB83A_0013, &words(&expected))
            .unwrap()
            .is_err());
    }

    #[test]
    fn update_store_has_the_view_store_layout() {
        let message = StoreMessage::UpdateStore {
            store: 3,
            vendor: 7,
        };
        let mut expected = long(7);
        expected.extend(int(3));
        assert_eq!(pack_values(&message.arguments()), words(&expected));
        assert_eq!(message.function(), 0xB83A_002A);
        assert_eq!(
            StoreMessage::decode(0xB83A_002A, &words(&expected)),
            Some(Ok(message))
        );
    }

    #[test]
    fn leave_store_is_the_storekeeper() {
        let message = StoreMessage::LeaveStore { vendor: 9 };
        assert_eq!(pack_values(&message.arguments()), words(&long(9)));
        assert_eq!(message.function(), 0xB83A_0010);
        assert_eq!(
            StoreMessage::decode(0xB83A_0010, &words(&long(9))),
            Some(Ok(message))
        );
        assert!(StoreMessage::decode(0xB83A_0010, &words(&int(9)))
            .unwrap()
            .is_err());
    }

    #[test]
    fn update_sale_packs_the_storekeeper_item_and_stock() {
        let message = StoreMessage::UpdateSale {
            vendor: 0x8000_0002_0000_0011,
            item: 0x8000_0002_0000_0021,
            stock: 4,
        };
        let mut expected = long(0x8000_0002_0000_0011);
        expected.extend(long(0x8000_0002_0000_0021));
        expected.extend(int(4));
        assert_eq!(pack_values(&message.arguments()), words(&expected));
        assert_eq!(message.function(), 0xB83A_0029);
        assert_eq!(
            StoreMessage::decode(0xB83A_0029, &words(&expected)),
            Some(Ok(message))
        );
    }

    /// The `InterpSEvent` header (0x10053, the function, the size) then the stub's pack.
    fn event_words(function: u32, arguments: &[u8]) -> Vec<u8> {
        let mut out = words(&[0x0001_0053, function, arguments.len() as u32]);
        out.extend(arguments);
        out
    }

    #[test]
    fn enter_and_leave_are_the_stubs_events() {
        use crate::protocol::ScriptRequest;
        let vendor = 0x8000_0002_0000_0042;
        let enter = ScriptRequest::EnterStore { store: 5, vendor };
        let mut expected = int(5);
        expected.extend(long(vendor));
        let (_, bytes) = enter.to_event().encode();
        assert_eq!(bytes, event_words(0xB83A_0044, &words(&expected)));
        assert_eq!(
            ScriptRequest::decode(0xB83A_0044, &words(&expected)),
            Some(Ok(enter))
        );
        let leave = ScriptRequest::LeaveStore(vendor);
        let (_, bytes) = leave.to_event().encode();
        assert_eq!(bytes, event_words(0xB83A_0003, &words(&long(vendor))));
        assert_eq!(
            ScriptRequest::decode(0xB83A_0003, &words(&long(vendor))),
            Some(Ok(leave))
        );
    }

    #[test]
    fn other_function_ids_are_not_store_messages() {
        assert!(StoreMessage::decode(0xB83A_0044, &[]).is_none());
        assert!(StoreMessage::decode(0x8215_0046, &[]).is_none());
    }
}
