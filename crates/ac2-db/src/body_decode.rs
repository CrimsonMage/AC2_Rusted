use ac2_core::QualifiedDataId;

use crate::{ResourceResult, ResourceTypeId};

/// Metadata-validated DBObj context presented to a domain-owned immutable decoder.
///
/// This is deliberately a value vocabulary rather than a DAT metadata view. Domain decoders may
/// make retail format decisions from the DBObj pack version, but cannot inspect storage details.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceDbObjDecodeContext {
    pub(crate) qdid: QualifiedDataId,
    pub(crate) dbobj_pack_version: u16,
}

impl ResourceDbObjDecodeContext {
    pub const fn qdid(self) -> QualifiedDataId {
        self.qdid
    }

    pub const fn dbobj_pack_version(self) -> u16 {
        self.dbobj_pack_version
    }
}

/// Metadata-validated DBObj payload presented to a domain-owned immutable decoder.
///
/// Resources owns the DID header rule; domain decoders receive only the verified body.
#[derive(Debug, Clone, Copy)]
pub struct ResourceDbObjBody<'a> {
    pub(crate) bytes: &'a [u8],
}

impl<'a> ResourceDbObjBody<'a> {
    pub const fn bytes(self) -> &'a [u8] {
        self.bytes
    }
}

/// Narrow extension point for immutable records whose semantic decode belongs above resources.
/// The resource store performs DAT access, header validation, and bounded `Arc` retention.
pub trait ResourceBodyDecode: Sized + Send + Sync + 'static {
    const TYPE_NAME: &'static str;
    const RESOURCE_TYPE: ResourceTypeId;

    fn decode_body(
        context: ResourceDbObjDecodeContext,
        body: ResourceDbObjBody<'_>,
    ) -> ResourceResult<Self>;

    /// Conservative charge for memory exclusively retained by this decoded cache value.
    ///
    /// This excludes transient unpacked DBObj bytes borrowed during decode; the store drops those
    /// bytes before the typed value enters its bounded cache.
    fn cache_retained_bytes(&self) -> usize;
}

/// DBObj body decoder whose destination-owned schema state was resolved before body loading.
///
/// The state may describe a stable companion schema from the same mounted dataset, but it must
/// not change the decoded resource's cache identity. Resources still validates the header,
/// owns storage access, and retains only the decoded value.
pub trait ResourceBodyDecodeWith: Sized + Send + Sync + 'static {
    type DecodeState: Send + Sync + ?Sized;

    const TYPE_NAME: &'static str;
    const RESOURCE_TYPE: ResourceTypeId;

    fn decode_body_with(
        state: &Self::DecodeState,
        context: ResourceDbObjDecodeContext,
        body: ResourceDbObjBody<'_>,
    ) -> ResourceResult<Self>;

    fn cache_retained_bytes(&self) -> usize;
}
