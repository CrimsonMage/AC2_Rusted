use ac2_core::QualifiedDataId;

use crate::{ResourceResult, ResourceTypeId};

/// Narrow destination-owned decoder for a standalone packed value without a DBObj header.
///
/// Resources validates the qualified typed identity, handles mounted-record decompression, and
/// retains the decoded result. The destination receives only the qualified identity, record
/// version, and packed value payload.
pub trait ResourcePackedDecode: Sized + Send + Sync + 'static {
    const TYPE_NAME: &'static str;
    const RESOURCE_TYPE: ResourceTypeId;

    fn decode_packed(
        qdid: QualifiedDataId,
        pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self>;

    fn cache_retained_bytes(&self) -> usize;
}
