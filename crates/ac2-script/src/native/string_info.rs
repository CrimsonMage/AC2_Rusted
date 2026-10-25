//! `StringInfo`: a string-table reference (string id + table DID) with typed variables, or a
//! literal string (`StringInfo::UnPack`, 0x00652DB0).

/// One `StringInfoData` variable.
#[derive(Debug, Clone, PartialEq)]
pub struct StringVar {
    pub id: u32,
    /// `StringInfoData::type`.
    pub kind: u16,
    pub value: StringVarValue,
}

/// A variable's value by `StringInfoData::type` (the `SID_*` classes).
#[derive(Debug, Clone, PartialEq)]
pub enum StringVarValue {
    /// 1, `SID_String` (made by script glue; `StringInfo::UnPack` reads no such variable).
    String(Vec<u16>),
    /// 2, `SID_Int`.
    Int(i32),
    /// 3, `SID_FormattedInt`.
    FormattedInt {
        value: i32,
        grouping: i32,
        base: u16,
    },
    /// 4, `SID_Float`.
    Float { value: f64, precision: u16 },
    /// 5, `SID_FormattedFloat`.
    FormattedFloat {
        value: f64,
        precision: u16,
        grouping: i32,
        base: u16,
    },
    /// 6, `SID_UInt`.
    UInt(u32),
    /// 7, `SID_FormattedUInt`.
    FormattedUInt {
        value: u32,
        grouping: i32,
        base: u16,
    },
    /// 9, `SID_StringInfo`.
    StringInfo(Box<StringInfo>),
    /// 10, `SID_LInt`.
    LInt(i64),
    /// 11, `SID_FormattedLInt`.
    FormattedLInt {
        value: i64,
        grouping: i32,
        base: u16,
    },
    /// 12, `SID_ULInt`.
    ULInt(u64),
    /// 13, `SID_FormattedULInt`.
    FormattedULInt {
        value: u64,
        grouping: i32,
        base: u16,
    },
    /// Any other type: `StringInfo::UnPack` reads no value and drops the variable.
    Skipped,
}

/// `StringInfo`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StringInfo {
    pub string_id: u32,
    pub table_id: u32,
    /// The variables `StringInfo::AddVar` kept, in packed order.
    pub variables: Vec<StringVar>,
    /// `m_LiteralValue` when `m_IsLiteral` was packed non-zero.
    pub literal: Option<Vec<u16>>,
}

/// `GetStringErrorCode` (`gsSuccess` is the `Ok` side).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum GetStringError {
    Uninitialized = 1,
    MissingString = 2,
    MissingTable = 3,
    MissingPrivateString = 4,
    MissingPrivateTable = 5,
    MissingVariable = 6,
    MetaLanguageError = 7,
    GenericError = 8,
}
