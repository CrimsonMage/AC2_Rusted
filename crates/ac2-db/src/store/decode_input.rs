use ac2_dat::DatEntryMeta;

/// The resolved metadata and owned temporary bytes for one synchronous typed-resource decode.
///
/// The metadata comes from the same DAT operation as the packed bytes, preventing body decoders
/// from repeating a DID lookup merely to recover the pack version. The bytes are decode-local and
/// must be dropped after the destination decoder returns; they are never resource-cache entries.
#[derive(Debug)]
pub(crate) struct ResourceDecodeInput {
    entry: DatEntryMeta,
    bytes: Vec<u8>,
}

impl ResourceDecodeInput {
    pub(crate) fn packed(entry: DatEntryMeta, bytes: Vec<u8>) -> Self {
        Self { entry, bytes }
    }

    pub(crate) fn decompressed(entry: DatEntryMeta, bytes: Vec<u8>) -> Self {
        Self { entry, bytes }
    }

    pub(crate) fn as_slice(&self) -> &[u8] {
        &self.bytes
    }

    pub(crate) const fn entry(&self) -> DatEntryMeta {
        self.entry
    }
}
