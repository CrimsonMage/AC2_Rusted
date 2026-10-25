//! `WLib` (`DB_TYPE_WLIB` 0x4000004F, DIDs 0x56000000-0x56FFFFFE in portal.dat): the compiled
//! script library, one `ByteStream`.

use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::DatFileKind;
use ac2_db::{
    CachedResource, ResourceError, ResourcePackedDecode, ResourceResult, ResourceStore,
    ResourceTypeId, TypedResourceRef,
};

use crate::bytestream::ByteStream;
use crate::unpack::Reader;

pub const WLIB_FIRST: u32 = 0x5600_0000;
pub const WLIB_LAST: u32 = 0x56FF_FFFE;
/// The library the Legions client runs: the only WLib in its portal.dat. Retail finds it by
/// file name through `DBFile2IDTable` (`CWAllocator::GetDBWLib`).
pub const LEGIONS_WLIB_DID: u32 = 0x5600_0005;

/// One `WLib`.
#[derive(Debug, Clone, PartialEq)]
pub struct WLib {
    pub did: u32,
    pub stream: ByteStream,
}

impl ResourcePackedDecode for WLib {
    const TYPE_NAME: &'static str = "AC2 WLib";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    /// `WLib::UnPack`: `DBObj::UnPack_Type` (the DID), then `ByteStream::UnPack` over the
    /// rest of the record.
    fn decode_packed(qdid: QualifiedDataId, _version: u16, bytes: &[u8]) -> ResourceResult<Self> {
        let bad = |context| ResourceError::MalformedTypedObject {
            qdid,
            type_name: Self::TYPE_NAME,
            context,
        };
        let mut r = Reader::new(bytes);
        let did = r.u32().map_err(|e| bad(e.context))?;
        if did != qdid.data_id().raw() {
            return Err(bad("embedded DID did not match DAT identity"));
        }
        let stream = ByteStream::unpack(&mut r).map_err(|e| bad(e.context))?;
        Ok(Self { did, stream })
    }

    fn cache_retained_bytes(&self) -> usize {
        let s = &self.stream;
        std::mem::size_of::<Self>()
            + s.opcodes.len()
            + s.string_literals
                .iter()
                .map(|l| l.len() * 2 + 24)
                .sum::<usize>()
            + s.string_literal_uses
                .iter()
                .map(|u| u.len() * 4 + 24)
                .sum::<usize>()
            + s.vtables
                .vtables
                .iter()
                .map(|v| v.len() * 4 + 24)
                .sum::<usize>()
            + s.vtables
                .func_mapper
                .iter()
                .map(|v| v.len() * 4 + 24)
                .sum::<usize>()
            + s.vtables.package_info.len() * 8
            + s.vtables.package_id_map.len() * 4
            + s.vtables
                .package_names
                .iter()
                .map(|n| n.0.len() + 32)
                .sum::<usize>()
            + s.exports.len() * std::mem::size_of::<crate::bytestream::ExportData>()
    }
}

/// The library `did` from portal.dat (decoded on a worker thread by the store).
pub fn load_wlib(
    store: &ResourceStore,
    did: DataId,
) -> ResourceResult<Option<CachedResource<WLib>>> {
    store.load_cached_packed::<WLib>(TypedResourceRef::new(DatFileKind::Portal, did, 0))
}
