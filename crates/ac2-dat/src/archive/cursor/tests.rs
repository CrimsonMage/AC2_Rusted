use super::ArchiveCursor;
use crate::ArchiveError;

#[test]
fn reads_little_endian_primitives_and_advances() {
    let mut cursor = ArchiveCursor::new(&[1, 2, 3, 4, 0, 0, 128, 63]);

    assert_eq!(cursor.read_u16().unwrap(), 0x0201);
    assert_eq!(cursor.position(), 2);
    assert_eq!(cursor.read_u16().unwrap(), 0x0403);
    assert_eq!(cursor.read_f32().unwrap(), 1.0);
    assert_eq!(cursor.remaining(), 0);
}

#[test]
fn borrowed_reads_do_not_allocate() {
    let bytes = [10, 11, 12, 13];
    let mut cursor = ArchiveCursor::new(&bytes);

    assert_eq!(cursor.read_bytes(3).unwrap(), &[10, 11, 12]);
    assert_eq!(cursor.peek_bytes_at(1, 2).unwrap(), &[11, 12]);
}

#[test]
fn bounds_errors_report_position() {
    let mut cursor = ArchiveCursor::new(&[1, 2]);

    assert!(matches!(
        cursor.read_u32(),
        Err(ArchiveError::UnexpectedEof {
            position: 0,
            requested: 4,
            len: 2
        })
    ));
}

#[test]
fn alignment_moves_to_next_boundary() {
    let mut cursor = ArchiveCursor::new(&[0; 8]);
    cursor.set_position(3).unwrap();
    cursor.align_to(4).unwrap();

    assert_eq!(cursor.position(), 4);
}

#[test]
fn reads_archive_format_lengths_and_strings() {
    let bytes = [
        0x01, 0x01, 0x00, 0x00, 0x00, 0x34, 0x12, 0x00, 0x80, 0x78, 0x56, 0x02, 0x00, 0x41, 0x00,
        0xA9, 0x03, 0x05, 0x00, b'h', b'e', b'l', b'l', b'o', 0x02, 0x00, b'o', b'k', 0x01, 0x00,
        0x00, 0x00,
    ];
    let mut cursor = ArchiveCursor::new(&bytes);

    assert!(cursor.read_bool_flag().unwrap());
    assert!(cursor.read_bool32_flag().unwrap());
    assert_eq!(cursor.read_compressed_u32().unwrap(), 0x1234);
    assert_eq!(cursor.read_compressed_u32().unwrap(), 0x5678);
    assert_eq!(cursor.read_utf16_string().unwrap(), "A\u{03A9}");
    assert_eq!(cursor.read_pstring().unwrap(), "hello");
    assert_eq!(cursor.read_packed_string().unwrap(), "ok");
    assert_eq!(cursor.read_packed_string().unwrap(), "");
    assert_eq!(cursor.remaining(), 0);
}

#[test]
fn reads_retail_format_lengths_and_strings() {
    let bytes = vec![
        0x01, 0x00, 0x00, 0x00, 0x7F, 0xBF, 0xFF, 0xC0, 0x01, 0x02, 0x00, 0x02, 0x41, 0x00, 0xA9,
        0x03, 0x05, b'h', b'e', b'l', b'l', b'o', 0x02, 0x00, b'o', b'k', 0x00, 0x00, 0x03, 0x00,
        b'r', b'a', b'w', 0x04, 0x00, 0x63, 0x61, 0x66, 0xE9, 0x00,
    ];
    let mut cursor = ArchiveCursor::new(&bytes);

    assert!(cursor.read_bool32_flag().unwrap());
    assert_eq!(cursor.read_retail_compressed_u32().unwrap(), 0x7F);
    assert_eq!(cursor.read_retail_compressed_u32().unwrap(), 0x3FFF);
    assert_eq!(cursor.read_retail_compressed_u32().unwrap(), 0x0001_0002);
    assert_eq!(cursor.read_retail_utf16_string().unwrap(), "A\u{03A9}");
    assert_eq!(cursor.read_retail_pstring().unwrap(), "hello");
    assert_eq!(
        cursor.read_packed_string_with_alignment(Some(4)).unwrap(),
        "ok"
    );
    assert_eq!(
        cursor.read_packed_string_with_alignment(None).unwrap(),
        "raw"
    );
    assert_eq!(cursor.read_windows_1252_packed_string().unwrap(), "café");
    assert_eq!(cursor.remaining(), 0);
}

#[test]
fn rejects_invalid_archive_bool_and_string_payloads() {
    let mut bool_cursor = ArchiveCursor::new(&[2]);
    assert!(matches!(
        bool_cursor.read_bool_flag(),
        Err(ArchiveError::InvalidBoolFlag { value: 2 })
    ));

    let bool32_bytes = 2u32.to_le_bytes();
    let mut bool32_cursor = ArchiveCursor::new(&bool32_bytes);
    assert!(matches!(
        bool32_cursor.read_bool32_flag(),
        Err(ArchiveError::InvalidBool32Flag { value: 2 })
    ));

    let mut utf16_cursor = ArchiveCursor::new(&[0x01, 0x00, 0x00, 0xD8]);
    assert!(matches!(
        utf16_cursor.read_utf16_string(),
        Err(ArchiveError::InvalidUtf16String)
    ));

    let mut utf8_cursor = ArchiveCursor::new(&[0x01, 0x00, 0xFF]);
    assert!(matches!(
        utf8_cursor.read_pstring(),
        Err(ArchiveError::InvalidUtf8String)
    ));
}

#[test]
fn rejects_invalid_retail_string_payloads() {
    let mut utf16_cursor = ArchiveCursor::new(&[0x01, 0x00, 0xD8]);
    assert!(matches!(
        utf16_cursor.read_retail_utf16_string(),
        Err(ArchiveError::InvalidUtf16String)
    ));

    let mut utf8_cursor = ArchiveCursor::new(&[0x01, 0xFF]);
    assert!(matches!(
        utf8_cursor.read_retail_pstring(),
        Err(ArchiveError::InvalidUtf8String)
    ));
}

#[test]
fn windows_1252_packed_string_respects_alignment() {
    let bytes = [0x04, 0x00, 0x63, 0x61, 0x66, 0xE9, 0x12, 0x34];
    let mut cursor = ArchiveCursor::new(&bytes);

    assert_eq!(cursor.read_windows_1252_packed_string().unwrap(), "café");
    assert_eq!(cursor.position(), 8);
}
