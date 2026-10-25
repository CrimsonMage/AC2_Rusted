//! The consignments a storekeeper shows (`WM_Store` 0x53 `ViewConsignment`, 0x57
//! `UpdateConsignment`). The forwarders' body (`CStoreSystem` 0x17, the build at 0x118008) walks
//! its reference argument as an `RList` of `Consignment`s (package 0x3898, nine slots) and builds a
//! `UIShopSale` from each one's sale with the store's sale build (0x118530), so the sale is a
//! `GameSaleProfile` as [`GameSale`] packs it.
//!
//! A `Consignment`'s members, by its functions and the build's calls: slot 0 the sale (0x10), slot 3
//! the ordinal (0x5, `UIShopSale::SetOrdinal`), slot 4 the stock (0x3, `SetStockQuantity`; a
//! consignment with none is skipped), slot 8 the flags (bit 0 expired: 0x19, `SetExpired`). Slots
//! 1-2 (a long int), 5 (an int) and 6-7 (a time, -1.0 from the constructor `Consignment` 0x14;
//! functions 0x1E, 0x22 and 0x23 count it in days) are not read by the build and are packed as the
//! constructor leaves them.

use super::store::{find, heap};
use super::GameSale;
use crate::protocol::native::{NativeObject, ScriptList};
use crate::protocol::pack::{HeapObject, PackedReference};
use crate::protocol::ProtocolError;

/// `Consignment`'s package id.
const CONSIGNMENT_PKG: u32 = 0x3898;
/// `R_LIST`: the native type of the list (`gRList`).
const R_LIST: u32 = 0x65;
/// The reference id of the list.
const LIST_ID: u32 = 1;
/// The flag bit of an expired consignment (`Consignment` 0x19 tests bit 0 through 0xD).
const EXPIRED: u32 = 0x1;

/// The VM's tags (`tag::REF`, `tag::INT`, `tag::LONG_INT`, `tag::DOUBLE`); a 64-bit store tags its
/// first slot only, the second keeps the uninitialised fill.
const REF: u8 = 0x01;
const INT: u8 = 0x02;
const LONG_INT: u8 = 0x82;
const DOUBLE: u8 = 0x83;
const UNINIT: u8 = 0xCD;

/// One consignment as the view shows it.
#[derive(Debug, Clone, PartialEq)]
pub struct Consignment {
    /// Slot 0: the item for sale.
    pub sale: GameSale,
    /// Slot 3: the sale's ordinal.
    pub ordinal: u32,
    /// Slot 4: the stock; zero hides the consignment.
    pub stock: u32,
    /// Slot 8, bit 0.
    pub expired: bool,
}

/// The consignments of a `ViewConsignment` or `UpdateConsignment`, in list order.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConsignmentList {
    pub consignments: Vec<Consignment>,
}

/// The reference ids of consignment `index`: the consignment, its sale, the sale's description.
fn ids(index: usize) -> (u32, u32, u32) {
    let base = LIST_ID + 1 + 3 * index as u32;
    (base, base + 1, base + 2)
}

impl ConsignmentList {
    /// The list as `ReferenceTable::PackReference` writes it: the `RList`, then each consignment,
    /// its sale and the sale's description.
    pub fn to_reference(&self) -> PackedReference {
        let items = (0..self.consignments.len())
            .map(|i| u64::from(ids(i).0))
            .collect();
        let mut reference = PackedReference {
            ids: vec![LIST_ID],
            entries: vec![heap(HeapObject {
                pkg_state_id: 0,
                pkg_id: u32::MAX,
                native: Some(NativeObject::List(ScriptList {
                    native_type: R_LIST,
                    items,
                })),
                slots: Vec::new(),
                tags: Vec::new(),
            })],
        };
        for (i, consignment) in self.consignments.iter().enumerate() {
            let (own, sale, desc) = ids(i);
            let time = (-1.0f64).to_bits();
            let flags = if consignment.expired { EXPIRED } else { 0 };
            reference.ids.extend([own, sale, desc]);
            reference.entries.push(heap(HeapObject {
                pkg_state_id: 0,
                pkg_id: CONSIGNMENT_PKG,
                native: None,
                slots: vec![
                    sale,
                    0,
                    0,
                    consignment.ordinal,
                    consignment.stock,
                    0,
                    time as u32,
                    (time >> 32) as u32,
                    flags,
                ],
                tags: vec![REF, LONG_INT, UNINIT, INT, INT, INT, DOUBLE, UNINIT, INT],
            }));
            reference.entries.push(heap(consignment.sale.object(desc)));
            reference.entries.push(heap(HeapObject {
                pkg_state_id: 0,
                pkg_id: u32::MAX,
                native: Some(NativeObject::StringInfo(consignment.sale.item_desc.clone())),
                slots: Vec::new(),
                tags: Vec::new(),
            }));
        }
        reference
    }

    /// The list a server's reference holds (the inverse of [`ConsignmentList::to_reference`]).
    pub fn from_reference(reference: &PackedReference) -> Result<Self, ProtocolError> {
        let invalid = || ProtocolError::Invalid("consignment list");
        let root = match reference.ids.first().and_then(|id| find(reference, *id)) {
            Some(HeapObject {
                native: Some(NativeObject::List(list)),
                ..
            }) if list.native_type == R_LIST => list,
            _ => return Err(invalid()),
        };
        let consignments = root
            .items
            .iter()
            .map(|id| {
                let object = find(reference, *id as u32).ok_or_else(invalid)?;
                if object.pkg_id != CONSIGNMENT_PKG || object.slots.len() != 9 {
                    return Err(invalid());
                }
                let slots = &object.slots;
                Ok(Consignment {
                    sale: GameSale::from_reference(reference, slots[0])?,
                    ordinal: slots[3],
                    stock: slots[4],
                    expired: slots[8] & EXPIRED != 0,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { consignments })
    }
}
