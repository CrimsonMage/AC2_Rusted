//! Entity description property values the script inquiries read (`EntityDesc::InqStringInfo`
//! and `InqDataID`, `portal/shared/src/entitydesc/entitydesc.cpp.c`).

use ac2_dat::ArchiveCursor;
use ac2_db::read_wide_pstring;

use crate::native::StringInfo;

/// A `StringInfo` property's packed value (`StringInfo::UnPack`): the string and table DIDs, the
/// variable and literal counts (low and high halves of one word), then the literal wide string.
/// Variables are not decoded yet, so a property with variables gives `None`.
pub fn string_info_property(bytes: &[u8]) -> Option<StringInfo> {
    let mut cursor = ArchiveCursor::new(bytes);
    let string_id = cursor.read_u32().ok()?;
    let table_id = cursor.read_u32().ok()?;
    let counts = cursor.read_u32().ok()?;
    let (variables, literal) = (counts & 0xFFFF, counts >> 16);
    if variables != 0 {
        return None;
    }
    let literal = if literal != 0 {
        Some(read_wide_pstring(&mut cursor).ok()?)
    } else {
        None
    };
    Some(StringInfo {
        string_id,
        table_id,
        variables: Vec::new(),
        literal,
    })
}

#[cfg(test)]
mod tests {
    use super::string_info_property;

    fn words(values: &[u32]) -> Vec<u8> {
        values.iter().flat_map(|w| w.to_le_bytes()).collect()
    }

    /// The string and table DIDs, no variables and no literal: the table's string.
    #[test]
    fn a_table_string_has_no_literal() {
        let info = string_info_property(&words(&[0x06E5_951F, 0x2500_7F28, 0])).unwrap();
        assert_eq!((info.string_id, info.table_id), (0x06E5_951F, 0x2500_7F28));
        assert!(info.variables.is_empty() && info.literal.is_none());
    }

    /// A literal (`m_IsLiteral` in the high half of the counts) is a wide string: a u16 length
    /// and its characters, aligned to four bytes (the empty literal has two bytes of padding).
    #[test]
    fn a_literal_is_a_wide_string() {
        let mut bytes = words(&[1, 2, 0x0001_0000]);
        bytes.extend([0, 0, 0, 0]);
        let info = string_info_property(&bytes).unwrap();
        assert_eq!(info.string_id, 1);
        assert_eq!(info.literal, Some(Vec::new()));
    }

    /// Variables are not decoded yet: a property with one gives no string.
    #[test]
    fn variables_are_not_decoded() {
        assert!(string_info_property(&words(&[1, 2, 1])).is_none());
    }
}
