//! `StringInfo`: a string-table reference (string id + table DID) with typed variables, or a
//! literal string (`StringInfo::UnPack`, 0x00652DB0).

use crate::unpack::{Reader, UnpackResult};

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

/// Nested `SID_StringInfo` deeper than this is treated as corrupt.
const MAX_DEPTH: usize = 32;

impl StringInfo {
    /// `StringInfo::UnPack`.
    pub fn unpack(r: &mut Reader<'_>) -> UnpackResult<Self> {
        Self::unpack_depth(r, 0)
    }

    fn unpack_depth(r: &mut Reader<'_>, depth: usize) -> UnpackResult<Self> {
        if depth > MAX_DEPTH {
            return Err(r.bad("StringInfo nesting"));
        }
        let string_id = r.u32()?;
        let table_id = r.u32()?;
        let num_vars = r.u16()? as i16;
        let is_literal = r.u16()? as i16;
        let literal = if is_literal != 0 {
            Some(r.wpstring()?)
        } else {
            None
        };
        let mut variables = Vec::new();
        for _ in 0..num_vars.max(0) {
            let id = r.u32()?;
            let kind = r.u16()?;
            r.align()?;
            let value = var_value(r, kind, depth)?;
            r.align()?;
            if value != StringVarValue::Skipped {
                variables.push(StringVar { id, kind, value });
            }
        }
        r.align()?;
        Ok(Self {
            string_id,
            table_id,
            variables,
            literal,
        })
    }
}

fn var_value(r: &mut Reader<'_>, kind: u16, depth: usize) -> UnpackResult<StringVarValue> {
    use StringVarValue as V;
    Ok(match kind {
        2 => V::Int(r.i32()?),
        3 => V::FormattedInt {
            value: r.i32()?,
            grouping: r.i32()?,
            base: r.u16()?,
        },
        4 => V::Float {
            value: r.f64()?,
            precision: r.u16()?,
        },
        5 => V::FormattedFloat {
            value: r.f64()?,
            precision: r.u16()?,
            grouping: r.i32()?,
            base: r.u16()?,
        },
        6 => V::UInt(r.u32()?),
        7 => V::FormattedUInt {
            value: r.u32()?,
            grouping: r.i32()?,
            base: r.u16()?,
        },
        9 => V::StringInfo(Box::new(StringInfo::unpack_depth(r, depth + 1)?)),
        10 => V::LInt(r.u64()? as i64),
        11 => V::FormattedLInt {
            value: r.u64()? as i64,
            grouping: r.i32()?,
            base: r.u16()?,
        },
        12 => V::ULInt(r.u64()?),
        13 => V::FormattedULInt {
            value: r.u64()?,
            grouping: r.i32()?,
            base: r.u16()?,
        },
        _ => V::Skipped,
    })
}
