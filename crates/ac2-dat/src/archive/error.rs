use std::fmt;

pub type ArchiveResult<T> = Result<T, ArchiveError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArchiveError {
    UnexpectedEof {
        position: usize,
        requested: usize,
        len: usize,
    },
    InvalidSeek {
        position: usize,
        len: usize,
    },
    AlignmentOverflow {
        position: usize,
        align: usize,
    },
    InvalidAlignment {
        align: usize,
    },
    InvalidBoolFlag {
        value: u8,
    },
    InvalidBool32Flag {
        value: u32,
    },
    InvalidUtf8String,
    InvalidUtf16String,
    NegativeCount {
        context: &'static str,
        value: i32,
    },
    ValueOutOfRange {
        context: &'static str,
        value: u64,
        max: u64,
    },
    CollectionLimitExceeded {
        context: &'static str,
        count: usize,
        max: usize,
    },
    ParseLimitExceeded {
        context: &'static str,
        limit: usize,
    },
    ErrorState,
    MissingVersionStack,
    UnknownVersionHandle {
        handle: u32,
    },
    VersionConflict {
        token: u32,
        existing: u32,
        requested: u32,
    },
    InvalidVersionHeader {
        value: u32,
    },
    InvalidVersionFooter,
    Io {
        operation: &'static str,
        path: String,
        message: String,
    },
    InvalidTextEncoding {
        filename: String,
        position: usize,
    },
    ParseError {
        message: String,
        filename: String,
        line: usize,
        column: usize,
    },
    InvalidFileNodeBinary {
        message: String,
    },
    MissingFileNodeName,
    FileNodeNameTypeMismatch {
        expected: &'static str,
    },
    FileNodeNameNotString,
    ExpectedLeafFileNode,
    UnknownFileNodeEnumName {
        name: String,
    },
    UnknownFileNodeEnumValue {
        value: u32,
    },
    InvalidPackableHashHeader {
        bucket_count: u32,
        entry_count: usize,
    },
    InvalidIntrusiveHashListHeader {
        bucket_count: u32,
        entry_count: usize,
    },
    InvalidIntrusiveHashHeader {
        bucket_count: u8,
        entry_count: usize,
    },
    TrailingBytes {
        position: usize,
        remaining: usize,
    },
    UnsupportedOperation(&'static str),
}

impl fmt::Display for ArchiveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEof {
                position,
                requested,
                len,
            } => write!(
                f,
                "archive read at offset {position} requested {requested} bytes past buffer length {len}"
            ),
            Self::InvalidSeek { position, len } => {
                write!(
                    f,
                    "archive seek to offset {position} past buffer length {len}"
                )
            }
            Self::AlignmentOverflow { position, align } => write!(
                f,
                "archive alignment from offset {position} to boundary {align} overflowed"
            ),
            Self::InvalidAlignment { align } => {
                write!(f, "archive alignment {align} is not a nonzero power of two")
            }
            Self::InvalidBoolFlag { value } => {
                write!(f, "archive boolean flag byte {value} was not 0 or 1")
            }
            Self::InvalidBool32Flag { value } => {
                write!(f, "archive 32-bit boolean flag value {value} was not 0 or 1")
            }
            Self::InvalidUtf8String => write!(f, "archive string payload was not valid UTF-8"),
            Self::InvalidUtf16String => write!(f, "archive string payload was not valid UTF-16"),
            Self::NegativeCount { context, value } => {
                write!(f, "archive count {value} for {context} was negative")
            }
            Self::ValueOutOfRange {
                context,
                value,
                max,
            } => write!(
                f,
                "archive value {value} for {context} exceeded maximum {max}"
            ),
            Self::CollectionLimitExceeded {
                context,
                count,
                max,
            } => write!(
                f,
                "archive collection count {count} for {context} exceeded caller limit {max}"
            ),
            Self::ParseLimitExceeded { context, limit } => {
                write!(f, "archive parse limit {limit} for {context} was exceeded")
            }
            Self::ErrorState => write!(f, "archive is already in an error state"),
            Self::MissingVersionStack => write!(f, "archive version stack is not initialized"),
            Self::UnknownVersionHandle { handle } => {
                write!(f, "unknown archive version handle 0x{handle:08X}")
            }
            Self::VersionConflict {
                token,
                existing,
                requested,
            } => write!(
                f,
                "archive version token 0x{token:08X} already has version {existing}, not {requested}"
            ),
            Self::InvalidVersionHeader { value } => {
                write!(f, "invalid archive version header 0x{value:08X}")
            }
            Self::InvalidVersionFooter => write!(f, "invalid archive version footer"),
            Self::Io {
                operation,
                path,
                message,
            } => write!(f, "archive {operation} failed for {path}: {message}"),
            Self::InvalidTextEncoding { filename, position } => write!(
                f,
                "archive text input {filename} is not valid UTF-8 at byte offset {position}"
            ),
            Self::ParseError {
                message,
                filename,
                line,
                column,
            } => write!(
                f,
                "archive parse error in {filename} at line {line}, column {column}: {message}"
            ),
            Self::InvalidFileNodeBinary { message } => {
                write!(f, "invalid binary file-node payload: {message}")
            }
            Self::MissingFileNodeName => write!(f, "file-node was missing a name"),
            Self::FileNodeNameTypeMismatch { expected } => {
                write!(f, "file-node name was not a {expected} token")
            }
            Self::FileNodeNameNotString => {
                write!(f, "file-node name was not a string token")
            }
            Self::ExpectedLeafFileNode => {
                write!(f, "file-node did not contain exactly one leaf child")
            }
            Self::UnknownFileNodeEnumName { name } => {
                write!(f, "file-node enum token '{name}' was not recognized")
            }
            Self::UnknownFileNodeEnumValue { value } => {
                write!(f, "file-node enum value {value} could not be stringified")
            }
            Self::InvalidPackableHashHeader {
                bucket_count,
                entry_count,
            } => write!(
                f,
                "packable hash header declared {entry_count} entries with bucket count {bucket_count}"
            ),
            Self::InvalidIntrusiveHashListHeader {
                bucket_count,
                entry_count,
            } => write!(
                f,
                "intrusive hash-list header declared {entry_count} entries with bucket count {bucket_count}"
            ),
            Self::InvalidIntrusiveHashHeader {
                bucket_count,
                entry_count,
            } => write!(
                f,
                "intrusive hash header declared {entry_count} entries with bucket count {bucket_count}"
            ),
            Self::TrailingBytes {
                position,
                remaining,
            } => write!(
                f,
                "archive decode stopped at offset {position} with {remaining} trailing unread bytes"
            ),
            Self::UnsupportedOperation(operation) => {
                write!(f, "unsupported archive operation: {operation}")
            }
        }
    }
}

impl std::error::Error for ArchiveError {}
