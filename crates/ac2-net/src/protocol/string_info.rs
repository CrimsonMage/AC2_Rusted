//! `StringInfo::UnPack` / `StringInfo::Pack`: a string-table reference, an optional literal and
//! typed substitution variables.
//!
//! Layout: u32 string token, u32 table id, i16 variable count, i16 literal flag, the literal as a
//! packed UTF-16 string when flagged, then each variable: u32 id, u16 type, alignment, the
//! type's value, alignment; then a final alignment.

use crate::protocol::reader::Reader;
#[cfg(test)]
use crate::protocol::reader::Writer;
use crate::protocol::ProtocolError;

/// Variables accepted in one `StringInfo` (and nested ones).
const MAX_VARIABLES: i16 = 256;
const MAX_DEPTH: usize = 8;

/// One substitution variable (`StringInfoData` subclasses), by its stream type.
#[derive(Debug, Clone, PartialEq)]
pub enum StringVariable {
    /// Types 2 and 6: one 32-bit value.
    Word { kind: u16, value: u32 },
    /// Types 3 and 7: two words and a short.
    WordsShort { kind: u16, a: u32, b: u32, c: u16 },
    /// Type 4: a 64-bit value and a short.
    LongShort { value: u64, c: u16 },
    /// Type 5: a 64-bit value, a short, a word and a short.
    Long5 { value: u64, c: u16, d: u32, e: u16 },
    /// Type 9 (`SID_StringInfo`): a nested `StringInfo`.
    Nested(Box<StringInfo>),
    /// Types 10 and 12: a 64-bit value.
    Long { kind: u16, value: u64 },
    /// Types 11 and 13: a 64-bit value, a word and a short.
    LongWordShort {
        kind: u16,
        value: u64,
        d: u32,
        e: u16,
    },
    /// Any other type carries no value.
    Empty { kind: u16 },
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct StringInfo {
    pub string_id: u32,
    pub table_id: u32,
    pub literal: Option<String>,
    /// Variable id and value, in stream order.
    pub variables: Vec<(u32, StringVariable)>,
}

impl StringInfo {
    pub(crate) fn read(reader: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        Self::read_depth(reader, 0)
    }

    fn read_depth(reader: &mut Reader<'_>, depth: usize) -> Result<Self, ProtocolError> {
        if depth > MAX_DEPTH {
            return Err(ProtocolError::Invalid("StringInfo nesting"));
        }
        let string_id = reader.u32()?;
        let table_id = reader.u32()?;
        let count = reader.i16()?;
        let literal_flag = reader.i16()?;
        if count > MAX_VARIABLES {
            return Err(ProtocolError::Invalid("StringInfo variables"));
        }
        let literal = if literal_flag != 0 {
            Some(reader.wide_string()?)
        } else {
            None
        };
        let mut variables = Vec::new();
        for _ in 0..count.max(0) {
            let id = reader.u32()?;
            let kind = reader.u16()?;
            reader.align()?;
            let value = match kind {
                2 | 6 => StringVariable::Word {
                    kind,
                    value: reader.u32()?,
                },
                3 | 7 => StringVariable::WordsShort {
                    kind,
                    a: reader.u32()?,
                    b: reader.u32()?,
                    c: reader.u16()?,
                },
                4 => StringVariable::LongShort {
                    value: reader.u64()?,
                    c: reader.u16()?,
                },
                5 => StringVariable::Long5 {
                    value: reader.u64()?,
                    c: reader.u16()?,
                    d: reader.u32()?,
                    e: reader.u16()?,
                },
                9 => StringVariable::Nested(Box::new(Self::read_depth(reader, depth + 1)?)),
                10 | 12 => StringVariable::Long {
                    kind,
                    value: reader.u64()?,
                },
                11 | 13 => StringVariable::LongWordShort {
                    kind,
                    value: reader.u64()?,
                    d: reader.u32()?,
                    e: reader.u16()?,
                },
                _ => StringVariable::Empty { kind },
            };
            reader.align()?;
            variables.push((id, value));
        }
        reader.align()?;
        Ok(Self {
            string_id,
            table_id,
            literal,
            variables,
        })
    }

    /// `StringInfo::Pack`.
    #[cfg(test)]
    pub(crate) fn write(&self, writer: &mut Writer) {
        writer
            .u32(self.string_id)
            .u32(self.table_id)
            .u16(self.variables.len() as u16)
            .u16(u16::from(self.literal.is_some()));
        if let Some(literal) = &self.literal {
            writer.wide_string(literal);
        }
        for (id, value) in &self.variables {
            let kind = match value {
                StringVariable::Word { kind, .. }
                | StringVariable::WordsShort { kind, .. }
                | StringVariable::Long { kind, .. }
                | StringVariable::LongWordShort { kind, .. }
                | StringVariable::Empty { kind } => *kind,
                StringVariable::LongShort { .. } => 4,
                StringVariable::Long5 { .. } => 5,
                StringVariable::Nested(_) => 9,
            };
            writer.u32(*id).u16(kind).align();
            match value {
                StringVariable::Word { value, .. } => {
                    writer.u32(*value);
                }
                StringVariable::WordsShort { a, b, c, .. } => {
                    writer.u32(*a).u32(*b).u16(*c);
                }
                StringVariable::LongShort { value, c } => {
                    writer.u64(*value).u16(*c);
                }
                StringVariable::Long5 { value, c, d, e } => {
                    writer.u64(*value).u16(*c).u32(*d).u16(*e);
                }
                StringVariable::Nested(inner) => inner.write(writer),
                StringVariable::Long { value, .. } => {
                    writer.u64(*value);
                }
                StringVariable::LongWordShort { value, d, e, .. } => {
                    writer.u64(*value).u32(*d).u16(*e);
                }
                StringVariable::Empty { .. } => {}
            }
            writer.align();
        }
        writer.align();
    }
}

#[cfg(test)]
mod tests {
    use super::{StringInfo, StringVariable};
    use crate::protocol::reader::Reader;
    #[cfg(test)]
    use crate::protocol::reader::Writer;

    #[test]
    fn a_literal_name_round_trips_with_its_alignment() {
        let info = StringInfo {
            string_id: 0,
            table_id: 0,
            literal: Some("Explorer".into()),
            variables: vec![],
        };
        let mut writer = Writer { bytes: Vec::new() };
        info.write(&mut writer);
        // 12 header bytes, then the 8-unit UTF-16 literal: 2 + 16 bytes, padded to 20.
        assert_eq!(writer.bytes.len(), 12 + 20);
        assert_eq!(&writer.bytes[8..12], &[0, 0, 1, 0]);
        let mut reader = Reader::new(&writer.bytes);
        assert_eq!(StringInfo::read(&mut reader).unwrap(), info);
        reader.finish().unwrap();
    }

    #[test]
    fn variables_take_their_type_sizes() {
        let info = StringInfo {
            string_id: 0x1234,
            table_id: 0x2300_0001,
            literal: None,
            variables: vec![
                (1, StringVariable::Word { kind: 2, value: 7 }),
                (
                    2,
                    StringVariable::Long5 {
                        value: 9,
                        c: 1,
                        d: 2,
                        e: 3,
                    },
                ),
                (3, StringVariable::Empty { kind: 8 }),
            ],
        };
        let mut writer = Writer { bytes: Vec::new() };
        info.write(&mut writer);
        assert_eq!(writer.bytes.len(), 12 + (8 + 4) + (8 + 16) + 8);
        let mut reader = Reader::new(&writer.bytes);
        assert_eq!(StringInfo::read(&mut reader).unwrap(), info);
    }
}
