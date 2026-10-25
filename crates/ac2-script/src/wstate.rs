//! `WState` (`DB_TYPE_WSTATE` 0x4000002F, DIDs 0x6F000000-0x7FFFFFFE in portal.dat): a saved
//! script reference graph (a package singleton, a weenie's script state, ...).
//!
//! `WState::UnPack`: `DBObj::UnPack_Type` (the DID), `PackageState::UnPack` (a u32 byte count
//! and that many bytes: `ReferenceTable::PackReference` output), then `m_pid`, the package id.

use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::DatFileKind;
use ac2_db::{
    CachedResource, ResourceError, ResourcePackedDecode, ResourceResult, ResourceStore,
    ResourceTypeId, TypedResourceRef,
};

use crate::reference::{PackedEntry, PackedReference};
use crate::unpack::{Reader, UnpackError};

pub const WSTATE_FIRST: u32 = 0x6F00_0000;
pub const WSTATE_LAST: u32 = 0x7FFF_FFFE;
/// `MasterDBMap::GetBaseDID(DB_TYPE_WSTATE)`.
pub const WSTATE_BASE_DID: u32 = 0x6F00_0000;

/// The WState holding package `pkg_id`'s singleton (`VM::GetSingletonPackageByID`:
/// `pid & 0xFFFF | base + 0x1000000`).
pub fn singleton_did(pkg_id: u32) -> u32 {
    (pkg_id & 0xFFFF) | (WSTATE_BASE_DID + 0x0100_0000)
}

/// The WState for a (category, type) pair (`WState::GetStateByType`:
/// `(category & 0xF) << 20 | type & 0xFFFFF | base + 0x3000000`).
pub fn state_by_type_did(category: u32, kind: u32) -> u32 {
    ((category & 0xF) << 20) | (kind & 0xF_FFFF) | (WSTATE_BASE_DID + 0x0300_0000)
}

/// One `WState`.
#[derive(Debug, Clone, PartialEq)]
pub struct WState {
    pub did: u32,
    /// `PackageState`'s bytes.
    pub package_state: Vec<u8>,
    /// `m_pid`. `None` when the record ends after the package state: an older layout (1,638
    /// Legions records, all 0x700xxxxx/0x710xxxxx) whose heap objects also lack `pkgStateID`
    /// and use older native ids; retail reads the pid past the record and then fails to
    /// unpack the references, so these states never load.
    pub pid: Option<u32>,
    /// The package state's reference graph, unpacked the way
    /// `ReferenceTable::UnPackReference` reads it.
    pub references: Result<PackedReference, UnpackError>,
}

impl WState {
    /// Unpacks a whole WState record (DID first).
    pub fn unpack(bytes: &[u8]) -> Result<Self, UnpackError> {
        let mut r = Reader::new(bytes);
        let did = r.u32()?;
        let n = r.count(1, "package state size")?;
        let package_state = r.bytes(n)?.to_vec();
        let pid = if r.remaining() >= 4 {
            Some(r.u32()?)
        } else {
            None
        };
        r.finish("bytes left after the WState")?;
        let references = PackedReference::unpack(&package_state);
        Ok(Self {
            did,
            package_state,
            pid,
            references,
        })
    }
}

impl ResourcePackedDecode for WState {
    const TYPE_NAME: &'static str = "AC2 WState";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(qdid: QualifiedDataId, _version: u16, bytes: &[u8]) -> ResourceResult<Self> {
        let bad = |context| ResourceError::MalformedTypedObject {
            qdid,
            type_name: Self::TYPE_NAME,
            context,
        };
        let state = Self::unpack(bytes).map_err(|e| bad(e.context))?;
        if state.did != qdid.data_id().raw() {
            return Err(bad("embedded DID did not match DAT identity"));
        }
        Ok(state)
    }

    fn cache_retained_bytes(&self) -> usize {
        let objects = self.references.as_ref().map_or(0, |refs| {
            refs.entries
                .iter()
                .map(|e| match e {
                    PackedEntry::Heap { object, .. } => {
                        object.slots.len() * 5 + std::mem::size_of_val(object) + 64
                    }
                    _ => std::mem::size_of_val(e),
                })
                .sum::<usize>()
        });
        std::mem::size_of::<Self>() + self.package_state.len() + objects
    }
}

/// The WState `did` from portal.dat (decoded on a worker thread by the store).
pub fn load_wstate(
    store: &ResourceStore,
    did: DataId,
) -> ResourceResult<Option<CachedResource<WState>>> {
    store.load_cached_packed::<WState>(TypedResourceRef::new(DatFileKind::Portal, did, 0))
}
