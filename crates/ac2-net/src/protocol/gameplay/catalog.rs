//! The store catalog a storekeeper opens (`WM_Store` 0x52 -> `CPlayer` 0x1F3 -> `CStoreSystem`
//! 0xF, `ViewCatalog`): a `StoreGroup` WState (package 0x3858) by DID, whose stores the body lists
//! (the build at 0x117914 reads each store's StoreTemplate and raises
//! `CM_Store::SendNotice_ViewCatalog`), and an `AAHash` from a store's DID to its size, which the
//! build gives `gUIShop::SetStoreSize`. The body sets the group on the storekeeper's shopper
//! (`CShopperContext` 0x2, slot 0); a consignment sale needs it (`RequestSell`'s body reads it back
//! with `CShopperContext` 0xA and refuses with 0xAFD1 without one).
//!
//! The forwarder's parameters are the hash, the group and the storekeeper; they are packed in that
//! order (the first packed value is the first parameter's slot).

use super::Args;
use crate::protocol::native::{HashKey, NativeObject, ScriptHash};
use crate::protocol::pack::PackValue;
use crate::protocol::ProtocolError;

/// `AAHash`'s native type, and the default table's bucket index.
const AA_HASH: u32 = 0x01;
const DEFAULT_BUCKET_INDEX: u8 = 2;

/// A storekeeper's catalog: the store group and the size of each of its stores.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StoreCatalog {
    /// The `StoreGroup` WState's DID.
    pub group: u32,
    /// (store DID, store size), in the hash's insertion order; a store without one shows size 0.
    pub store_sizes: Vec<(u32, u32)>,
}

impl StoreCatalog {
    /// The packed values ahead of the storekeeper: the size hash, then the group.
    pub(super) fn values(&self) -> [PackValue; 2] {
        let hash = ScriptHash {
            native_type: AA_HASH,
            bucket_index: DEFAULT_BUCKET_INDEX,
            keys: self
                .store_sizes
                .iter()
                .map(|(did, _)| HashKey::Value(u64::from(*did)))
                .collect(),
            values: self
                .store_sizes
                .iter()
                .map(|(_, size)| u64::from(*size))
                .collect(),
        };
        [
            PackValue::native(NativeObject::Hash(hash)),
            PackValue::Int(self.group),
        ]
    }

    /// The catalog from the packed hash and group.
    pub(super) fn read(args: &mut Args) -> Result<Self, ProtocolError> {
        let invalid = || ProtocolError::Invalid("store catalog");
        let reference = args.reference()?;
        let Some(NativeObject::Hash(hash)) = reference.root_native() else {
            return Err(invalid());
        };
        if hash.native_type != AA_HASH {
            return Err(invalid());
        }
        let store_sizes = hash
            .keys
            .iter()
            .zip(&hash.values)
            .map(|(key, size)| match key {
                HashKey::Value(did) => Ok((*did as u32, *size as u32)),
                _ => Err(invalid()),
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            group: args.int()?,
            store_sizes,
        })
    }
}
