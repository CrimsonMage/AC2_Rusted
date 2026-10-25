//! The store's messages (`WM_Store`, package 0x383A): the client's requests and the server's
//! answers. The requests are [`ScriptRequest`](super::ScriptRequest) variants whose stubs the
//! script library runs for `gmCWInterface`'s store calls; the answers are the forwarders the server
//! reaches, which call `CPlayer` and on to `CStoreSystem`, which raises the `CM_Store::SendNotice_*`
//! notices.
//!
//! As with the trade messages, a forwarder's parameters are packed last first (the forwarder passes
//! them to `CPlayer` in declared order, and the VM reads the first packed value as the last
//! parameter), so each message lists its values last parameter first.

use super::{function, Args, ConsignmentList};
use crate::protocol::native::{NativeObject, ScriptList};
use crate::protocol::pack::{entry_flags, HeapObject, PackValue, PackedEntry, PackedReference};
use crate::protocol::string_info::StringInfo;
use crate::protocol::ProtocolError;

/// A server answer to the store (`WM_Store` forwarders).
#[derive(Debug, Clone, PartialEq)]
pub enum StoreMessage {
    /// [`function::STORE_VIEW`]: the store opened (`CM_Store::SendNotice_ViewStore`), with the
    /// storekeeper.
    ViewStore { view: StoreView, vendor: u64 },
    /// [`function::STORE_UPDATE`]: the open store's sales changed (`CM_Store::SendNotice_UpdateStore`):
    /// the store as a [`StoreView`], with the storekeeper. The forwarder (`CPlayer` 0x1E2 ->
    /// `CStoreSystem` 0x8, body 0x117BA0) takes the same two values as `ViewStore`.
    UpdateStore { view: StoreView, vendor: u64 },
    /// [`function::STORE_LEFT`]: the storekeeper (`CM_Store::SendNotice_LeaveStore`).
    LeaveStore { vendor: u64 },
    /// [`function::STORE_UPDATE_SALE`]: an item's stock (`CM_Store::SendNotice_UpdateSale`).
    UpdateSale { vendor: u64, item: u64, stock: u32 },
    /// [`function::STORE_VIEW_CONSIGNMENT`]: the storekeeper's consignments opened
    /// (`CM_Store::SendNotice_ViewConsignment`), with the storekeeper.
    ViewConsignment {
        consignments: ConsignmentList,
        vendor: u64,
    },
    /// [`function::STORE_UPDATE_CONSIGNMENT`]: the open consignments changed
    /// (`CM_Store::SendNotice_UpdateConsignment`), with the storekeeper.
    UpdateConsignment {
        consignments: ConsignmentList,
        vendor: u64,
    },
}

impl StoreMessage {
    /// The function id the message is sent to.
    pub fn function(&self) -> u32 {
        match self {
            Self::ViewStore { .. } => function::STORE_VIEW,
            Self::UpdateStore { .. } => function::STORE_UPDATE,
            Self::LeaveStore { .. } => function::STORE_LEFT,
            Self::UpdateSale { .. } => function::STORE_UPDATE_SALE,
            Self::ViewConsignment { .. } => function::STORE_VIEW_CONSIGNMENT,
            Self::UpdateConsignment { .. } => function::STORE_UPDATE_CONSIGNMENT,
        }
    }

    /// The packed arguments, last parameter first (the wire order).
    pub fn arguments(&self) -> Vec<PackValue> {
        match self {
            Self::ViewStore { view, vendor } => vec![
                PackValue::Reference(view.to_reference()),
                PackValue::LongInt(*vendor),
            ],
            Self::UpdateStore { view, vendor } => vec![
                PackValue::Reference(view.to_reference()),
                PackValue::LongInt(*vendor),
            ],
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
            Self::ViewConsignment {
                consignments,
                vendor,
            }
            | Self::UpdateConsignment {
                consignments,
                vendor,
            } => vec![
                PackValue::Reference(consignments.to_reference()),
                PackValue::LongInt(*vendor),
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
                let view = StoreView::from_reference(&a.reference()?)?;
                Ok(Self::ViewStore {
                    view,
                    vendor: a.long()?,
                })
            }),
            function::STORE_UPDATE => decode(|a| {
                let view = StoreView::from_reference(&a.reference()?)?;
                Ok(Self::UpdateStore {
                    view,
                    vendor: a.long()?,
                })
            }),
            function::STORE_LEFT => decode(|a| Ok(Self::LeaveStore { vendor: a.long()? })),
            function::STORE_VIEW_CONSIGNMENT => decode(|a| {
                let consignments = ConsignmentList::from_reference(&a.reference()?)?;
                Ok(Self::ViewConsignment {
                    consignments,
                    vendor: a.long()?,
                })
            }),
            function::STORE_UPDATE_CONSIGNMENT => decode(|a| {
                let consignments = ConsignmentList::from_reference(&a.reference()?)?;
                Ok(Self::UpdateConsignment {
                    consignments,
                    vendor: a.long()?,
                })
            }),
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

/// `StoreView` (package 0x385D): the store the server opens, as the shop build reads it
/// (`CStoreSystem` 0x1170E8's body and the build at 0x117D64). Its members, by the calls the build
/// makes on them: member 0 the sales (a native `RList` of [`GameSale`]s), member 1 the
/// StoreTemplate's WState DID (the build passes `StoreView` function 7's result to
/// `GetPackageReference(did, 0x3810)`), member 2 the store size (function 0x10 feeds
/// `gUIShop::SetStoreSize`), member 3 the position (function 0xE feeds `gUIShop::SetPosition`).
#[derive(Debug, Clone, PartialEq)]
pub struct StoreView {
    /// Member 1: the StoreTemplate's WState DID.
    pub store: u32,
    /// Member 2: `gUIShop::SetStoreSize`'s value.
    pub store_size: u32,
    /// Member 3: `gUIShop::SetPosition`'s value.
    pub position: u32,
    /// Member 0: the sales, in list order.
    pub sales: Vec<GameSale>,
}

/// One sale as a `GameSaleProfile` (package 0x382E, eleven members): the store's `SaleProfile`
/// (0x37FD) methods the shop build calls on it, in the sale build at 0x118530.
#[derive(Debug, Clone, PartialEq)]
pub struct GameSale {
    /// Member 0: the item's DID (`SaleProfile` 0x15, `UIShopSale::SetItemDID`).
    pub item_did: u32,
    /// Members 2-3: the item's IID (`SaleProfile` 0x1A, `UIShopSale::SetItemIID`).
    pub item: u64,
    /// Member 4: the icon's DID (`SaleProfile` 0x2D, `UIShopSale::SetItemIconID`).
    pub icon_did: u32,
    /// Member 5: the item's description (`SaleProfile` 0x1B, `UIShopSale::SetItemDesc`).
    pub item_desc: StringInfo,
    /// Member 6: the cost quantity, the price (`SaleProfile` 0x17, `SetCostQuantity`).
    pub cost: f32,
    /// Member 7: the DID of the cost's entity (`SaleProfile` 0x19); it must be an entity
    /// (`DB_TYPE_ENTITYDESC`) or the sale build skips the sale.
    pub cost_did: u32,
    /// Member 8: the sold flag (`SaleProfile` 0x2A and 0x30).
    pub sold: u32,
}

/// A heap entry's kind and loaded flags (`PackedEntry::Heap`, as the server packs them).
const HEAP_FLAGS: u32 = entry_flags::KIND_HEAP | entry_flags::LOADED | entry_flags::RECURSE;
/// `StoreView`'s package id (`StoreView`, 0x385D).
const STORE_VIEW_PKG: u32 = 0x385D;
/// `GameSaleProfile`'s package id (0x382E).
const GAME_SALE_PKG: u32 = 0x382E;
/// `R_LIST`: the native type a `StoreView`'s sales are packed as (`gRList`).
const R_LIST: u32 = 0x65;
/// The reference ids of the view, its sales list and the sales (the sender's table indices).
const VIEW_ID: u32 = 1;
const LIST_ID: u32 = 2;

/// The VM's tags for the slots (`tag::REF`, `tag::INT`, `tag::FLOAT`, `tag::LONG_INT`).
const TAG_REF: u8 = 0x01;
const TAG_INT: u8 = 0x02;
const TAG_FLOAT: u8 = 0x03;
const TAG_LONG_INT: u8 = 0x82;

/// The reference id of sale `index`'s `GameSaleProfile`; its description's id follows it.
fn sale_id(index: usize) -> u32 {
    3 + 2 * index as u32
}

impl StoreView {
    /// The view as `ReferenceTable::PackReference` writes it: the root `StoreView`, its sales'
    /// list, then each sale and its description, in reference-id order.
    pub fn to_reference(&self) -> PackedReference {
        let mut ids = vec![VIEW_ID, LIST_ID];
        let mut entries = Vec::with_capacity(2 + 2 * self.sales.len());
        let view = HeapObject {
            pkg_state_id: 0,
            pkg_id: STORE_VIEW_PKG,
            native: None,
            slots: vec![LIST_ID, self.store, self.store_size, self.position],
            tags: vec![TAG_REF, TAG_INT, TAG_INT, TAG_INT],
        };
        entries.push(heap(view));
        let items = (0..self.sales.len())
            .map(|i| u64::from(sale_id(i)))
            .collect();
        let list = HeapObject {
            pkg_state_id: 0,
            pkg_id: u32::MAX,
            native: Some(NativeObject::List(ScriptList {
                native_type: R_LIST,
                items,
            })),
            slots: Vec::new(),
            tags: Vec::new(),
        };
        entries.push(heap(list));
        for (i, sale) in self.sales.iter().enumerate() {
            let desc_id = sale_id(i) + 1;
            ids.push(sale_id(i));
            entries.push(heap(sale.object(desc_id)));
            ids.push(desc_id);
            entries.push(heap(HeapObject {
                pkg_state_id: 0,
                pkg_id: u32::MAX,
                native: Some(NativeObject::StringInfo(sale.item_desc.clone())),
                slots: Vec::new(),
                tags: Vec::new(),
            }));
        }
        PackedReference { ids, entries }
    }

    /// The view a server's packed reference holds (the inverse of [`StoreView::to_reference`]).
    pub fn from_reference(reference: &PackedReference) -> Result<Self, ProtocolError> {
        let invalid = || ProtocolError::Invalid("store view");
        let view = find(reference, VIEW_ID).ok_or_else(invalid)?;
        let slots = &view.slots;
        if view.pkg_id != STORE_VIEW_PKG || slots.len() != 4 {
            return Err(invalid());
        }
        let list = match find(reference, slots[0]).and_then(|o| o.native.as_ref()) {
            Some(NativeObject::List(list)) if list.native_type == R_LIST => list,
            _ => return Err(invalid()),
        };
        let sales = list
            .items
            .iter()
            .map(|id| GameSale::from_reference(reference, *id as u32))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            store: slots[1],
            store_size: slots[2],
            position: slots[3],
            sales,
        })
    }
}

impl GameSale {
    /// The `GameSaleProfile` object (its members as the struct documents them), with its
    /// description at reference `desc_id`.
    pub(super) fn object(&self, desc_id: u32) -> HeapObject {
        HeapObject {
            pkg_state_id: 0,
            pkg_id: GAME_SALE_PKG,
            native: None,
            slots: vec![
                self.item_did,
                u32::MAX,
                self.item as u32,
                (self.item >> 32) as u32,
                self.icon_did,
                desc_id,
                self.cost.to_bits(),
                self.cost_did,
                self.sold,
                0,
                0,
            ],
            tags: vec![
                TAG_INT,
                TAG_REF,
                TAG_LONG_INT,
                TAG_LONG_INT,
                TAG_INT,
                TAG_REF,
                TAG_FLOAT,
                TAG_INT,
                TAG_INT,
                TAG_INT,
                TAG_INT,
            ],
        }
    }

    pub(super) fn from_reference(
        reference: &PackedReference,
        id: u32,
    ) -> Result<Self, ProtocolError> {
        let invalid = || ProtocolError::Invalid("store sale");
        let sale = find(reference, id).ok_or_else(invalid)?;
        if sale.pkg_id != GAME_SALE_PKG || sale.slots.len() != 11 {
            return Err(invalid());
        }
        let slots = &sale.slots;
        let desc = match find(reference, slots[5]).and_then(|o| o.native.as_ref()) {
            Some(NativeObject::StringInfo(text)) => text.clone(),
            _ => return Err(invalid()),
        };
        Ok(Self {
            item_did: slots[0],
            item: u64::from(slots[2]) | u64::from(slots[3]) << 32,
            icon_did: slots[4],
            item_desc: desc,
            cost: f32::from_bits(slots[6]),
            cost_did: slots[7],
            sold: slots[8],
        })
    }
}

/// The purchase's `TransactionBlob` (package 0x3812, nine slots) as the client's
/// `PurchaseConfirmed` stub sends it (`CStoreSystem` 0x4, `WM_Store` 0x14): the item's IID
/// (slots 0-1, low word first), a reference list (slot 2), the quantity (slot 5; slot 3 reads 1
/// for every quantity and is not decoded) and the storekeeper's IID (slots 6-7). Slots 4 and 8
/// hold values the VM leaves uninitialised; the item and quantity were read on the VM (1 and 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Purchase {
    pub item: u64,
    pub quantity: u32,
    pub vendor: u64,
}

impl Purchase {
    /// The purchase a blob reference holds.
    pub fn from_reference(reference: &PackedReference) -> Result<Self, ProtocolError> {
        let root = reference
            .ids
            .first()
            .and_then(|id| find(reference, *id))
            .ok_or(ProtocolError::Invalid("transaction blob"))?;
        if root.pkg_id != TRANSACTION_BLOB_PKG || root.slots.len() != 9 {
            return Err(ProtocolError::Invalid("transaction blob"));
        }
        let slots = &root.slots;
        Ok(Self {
            item: u64::from(slots[0]) | u64::from(slots[1]) << 32,
            quantity: slots[5],
            vendor: u64::from(slots[6]) | u64::from(slots[7]) << 32,
        })
    }
}

/// `TransactionBlob`'s package id (0x3812).
const TRANSACTION_BLOB_PKG: u32 = 0x3812;

/// A heap entry, with the kind and loaded flags a server packs.
pub(super) fn heap(object: HeapObject) -> PackedEntry {
    PackedEntry::Heap {
        flags: HEAP_FLAGS,
        object,
    }
}

/// The heap object at reference id `id`.
pub(super) fn find(reference: &PackedReference, id: u32) -> Option<&HeapObject> {
    let index = reference.ids.iter().position(|i| *i == id)?;
    match reference.entries.get(index)? {
        PackedEntry::Heap { object, .. } => Some(object),
        _ => None,
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

    fn sale(item: u64, cost: f32) -> GameSale {
        GameSale {
            item_did: 0x4700_0001,
            item,
            icon_did: 0,
            item_desc: StringInfo::default(),
            cost,
            cost_did: 0x4700_0002,
            sold: 0,
        }
    }

    fn view(sales: Vec<GameSale>) -> StoreView {
        StoreView {
            store: 0x6F00_0001,
            store_size: 3,
            position: 5,
            sales,
        }
    }

    #[test]
    fn view_store_without_sales_packs_the_view_then_the_empty_list() {
        // The layout the VM reads, byte for byte: the view (pkg 0x385D, four slots: the list, the
        // StoreTemplate's DID, the size, the position), then the empty `RList`. The VM ran this
        // shape with the StoreTemplate DID 0x6F0033A3; the test uses a placeholder DID.
        let message = StoreMessage::ViewStore {
            view: StoreView {
                position: 0,
                ..view(Vec::new())
            },
            vendor: 0x8000_0002_0000_0042,
        };
        let mut expected = vec![5, 2, 1, 2];
        expected.extend([0x4400_0100, 0, 0x385D_0000, 4, 2, 0x6F00_0001, 3, 0]);
        // The tags (tag, INT, INT, INT) are bytes: one word.
        expected.push(0x0202_0201);
        expected.extend([0x4400_0100, 0, 0xFFFF_0065, 0, 0]);
        expected.extend(long(0x8000_0002_0000_0042));
        assert_eq!(pack_values(&message.arguments()), words(&expected));
        assert_eq!(message.function(), 0xB83A_0013);
        assert_eq!(
            StoreMessage::decode(0xB83A_0013, &words(&expected)),
            Some(Ok(message))
        );
    }

    #[test]
    fn view_store_round_trips_its_sales() {
        let item = 0x8000_0002_0000_0021;
        let message = StoreMessage::ViewStore {
            view: view(vec![sale(item, 500.0), sale(item + 1, 600.0)]),
            vendor: 0x8000_0002_0000_0042,
        };
        let bytes = pack_values(&message.arguments());
        assert_eq!(
            StoreMessage::decode(0xB83A_0013, &bytes),
            Some(Ok(message.clone()))
        );
        let StoreMessage::ViewStore { view, .. } = message else {
            unreachable!()
        };
        assert_eq!(view.sales[1].item, item + 1);
        assert_eq!(view.sales[0].cost, 500.0);
    }

    #[test]
    fn view_store_needs_a_store_view_not_an_int() {
        let mut expected = int(0x7000_1234);
        expected.extend(long(0x8000_0002_0000_0042));
        assert!(StoreMessage::decode(0xB83A_0013, &words(&expected))
            .unwrap()
            .is_err());
    }

    #[test]
    fn update_store_has_the_view_store_layout() {
        let message = StoreMessage::UpdateStore {
            view: view(vec![sale(0x8000_0002_0000_0021, 500.0)]),
            vendor: 0x8000_0002_0000_0042,
        };
        let bytes = pack_values(&message.arguments());
        assert_eq!(message.function(), 0xB83A_002A);
        assert_eq!(
            StoreMessage::decode(0xB83A_002A, &bytes),
            Some(Ok(message.clone()))
        );
        // The same first value as `ViewStore` (a reference, tag 5), then the long vendor.
        assert_eq!(&bytes[..4], &5u32.to_le_bytes());
        assert_eq!(
            &bytes[bytes.len() - 12..],
            &long(0x8000_0002_0000_0042)
                .iter()
                .flat_map(|w| w.to_le_bytes())
                .collect::<Vec<u8>>()[..]
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
