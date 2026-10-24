use std::fmt;
use std::io;
use std::path::PathBuf;

use crate::entry::DatFileKind;

pub type DatResult<T> = Result<T, DatError>;

#[derive(Debug)]
pub enum DatError {
    Io(io::Error),
    MissingDatFile {
        kind: DatFileKind,
        path: PathBuf,
    },
    InvalidHeaderMagic {
        path: PathBuf,
        magic: i32,
    },
    InvalidDatType {
        path: PathBuf,
        kind: DatFileKind,
        raw_type: u32,
    },
    InvalidBlockSize {
        path: PathBuf,
        block_size: i32,
    },
    InvalidBlockOffset {
        path: PathBuf,
        block_offset: usize,
        context: &'static str,
    },
    InvalidBlockChain {
        path: PathBuf,
        did: Option<u32>,
        block_offset: usize,
        context: &'static str,
    },
    InvalidBTreeNode {
        path: PathBuf,
        block_offset: usize,
        context: &'static str,
    },
    RangeOutOfBounds {
        did: u32,
        offset: u64,
        len: usize,
        packed_len: u32,
    },
    StoredCompressedEntry {
        kind: DatFileKind,
        did: u32,
    },
    InvalidDddIterationData {
        did: u32,
        reason: &'static str,
    },
    RewriteMissingFile {
        kind: DatFileKind,
    },
    RewriteWouldOverwriteSource {
        kind: DatFileKind,
        source_path: PathBuf,
        output_path: PathBuf,
    },
    RewriteImageTooLarge {
        kind: DatFileKind,
        rebuilt_size: u64,
        source_size_limit: u64,
    },
    RewriteValidationFailed {
        kind: DatFileKind,
        did: Option<u32>,
        reason: String,
    },
}

impl fmt::Display for DatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::MissingDatFile { kind, path } => {
                write!(f, "missing {:?} DAT file at {}", kind, path.display())
            }
            Self::InvalidHeaderMagic { path, magic } => write!(
                f,
                "invalid DAT header magic {magic:#010X} for {}",
                path.display()
            ),
            Self::InvalidDatType {
                path,
                kind,
                raw_type,
            } => write!(
                f,
                "unexpected DAT header type {raw_type} for {:?} file {}",
                kind,
                path.display()
            ),
            Self::InvalidBlockSize { path, block_size } => write!(
                f,
                "invalid DAT block size {block_size} for {}",
                path.display()
            ),
            Self::InvalidBlockOffset {
                path,
                block_offset,
                context,
            } => write!(
                f,
                "invalid DAT block offset {block_offset} while reading {context} in {}",
                path.display()
            ),
            Self::InvalidBlockChain {
                path,
                did,
                block_offset,
                context,
            } => {
                if let Some(did) = did {
                    write!(
                        f,
                        "invalid DAT block chain for DID 0x{did:08X} at block offset {block_offset} while reading {context} in {}",
                        path.display()
                    )
                } else {
                    write!(
                        f,
                        "invalid DAT block chain at block offset {block_offset} while reading {context} in {}",
                        path.display()
                    )
                }
            }
            Self::InvalidBTreeNode {
                path,
                block_offset,
                context,
            } => write!(
                f,
                "invalid DAT B-tree node at block offset {block_offset} in {}: {context}",
                path.display()
            ),
            Self::RangeOutOfBounds {
                did,
                offset,
                len,
                packed_len,
            } => write!(
                f,
                "range offset {offset} len {len} is out of bounds for DID 0x{did:08X} packed length {packed_len}",
            ),
            Self::StoredCompressedEntry { kind, did } => write!(
                f,
                "{kind:?} DID 0x{did:08X} is stored compressed; use read_packed and decompress above the DAT layer"
            ),
            Self::InvalidDddIterationData { did, reason } => write!(
                f,
                "invalid DDD iteration payload for DID 0x{did:08X}: {reason}"
            ),
            Self::RewriteMissingFile { kind } => {
                write!(f, "rewrite plan referenced missing {:?} DAT file", kind)
            }
            Self::RewriteWouldOverwriteSource {
                kind,
                source_path,
                output_path,
            } => write!(
                f,
                "refusing to rebuild {:?} DAT copy over source path {} -> {}",
                kind,
                source_path.display(),
                output_path.display()
            ),
            Self::RewriteImageTooLarge {
                kind,
                rebuilt_size,
                source_size_limit,
            } => write!(
                f,
                "rebuilt {:?} DAT image size {} exceeds source size limit {}",
                kind, rebuilt_size, source_size_limit
            ),
            Self::RewriteValidationFailed { kind, did, reason } => {
                if let Some(did) = did {
                    write!(
                        f,
                        "rebuilt {:?} DAT validation failed for DID 0x{did:08X}: {}",
                        kind, reason
                    )
                } else {
                    write!(f, "rebuilt {:?} DAT validation failed: {}", kind, reason)
                }
            }
        }
    }
}

impl std::error::Error for DatError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for DatError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
