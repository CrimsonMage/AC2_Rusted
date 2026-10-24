use std::fmt;

use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::ArchiveError;
use ac2_dat::{DatError, DatFileKind};

pub type ResourceResult<T> = Result<T, ResourceError>;

#[derive(Debug)]
pub enum ResourceError {
    Dat(DatError),
    Archive(ArchiveError),
    DecompressionIo {
        file_kind: DatFileKind,
        did: DataId,
        message: String,
    },
    DecompressionLengthMismatch {
        file_kind: DatFileKind,
        did: DataId,
        expected_len: usize,
        actual_len: usize,
    },
    InvalidData {
        file_kind: DatFileKind,
        did: DataId,
        context: &'static str,
    },
    MalformedTypedObject {
        qdid: QualifiedDataId,
        type_name: &'static str,
        context: &'static str,
    },
}

impl fmt::Display for ResourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dat(error) => write!(f, "{error}"),
            Self::Archive(error) => write!(f, "{error}"),
            Self::DecompressionIo {
                file_kind,
                did,
                message,
            } => write!(
                f,
                "failed to decompress {file_kind:?} DID 0x{did:08X}: {message}"
            ),
            Self::DecompressionLengthMismatch {
                file_kind,
                did,
                expected_len,
                actual_len,
            } => write!(
                f,
                "decompressed {file_kind:?} DID 0x{did:08X} to {actual_len} bytes, expected {expected_len}"
            ),
            Self::InvalidData {
                file_kind,
                did,
                context,
            } => write!(
                f,
                "invalid resource data in {file_kind:?} DID 0x{did:08X}: {context}"
            ),
            Self::MalformedTypedObject {
                qdid,
                type_name,
                context,
            } => write!(f, "malformed {type_name} payload for {qdid}: {context}"),
        }
    }
}

impl std::error::Error for ResourceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Dat(error) => Some(error),
            Self::Archive(error) => Some(error),
            _ => None,
        }
    }
}

impl ResourceError {
    pub(crate) const fn diagnostic_kind(&self) -> &'static str {
        match self {
            Self::Dat(_) => "dat",
            Self::Archive(_) => "archive",
            Self::DecompressionIo { .. } => "decompression_io",
            Self::DecompressionLengthMismatch { .. } => "decompression_length_mismatch",
            Self::InvalidData { .. } => "invalid_data",
            Self::MalformedTypedObject { .. } => "malformed_typed_object",
        }
    }
}

impl From<DatError> for ResourceError {
    fn from(value: DatError) -> Self {
        Self::Dat(value)
    }
}

impl From<ArchiveError> for ResourceError {
    fn from(value: ArchiveError) -> Self {
        Self::Archive(value)
    }
}
