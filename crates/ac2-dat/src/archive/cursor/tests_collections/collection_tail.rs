use super::super::ArchiveCursor;
use crate::ArchiveError;

#[test]
fn packable_hash_helper_rejects_entries_without_buckets() {
    let mut cursor = ArchiveCursor::new(&[0x03, 0x00, 0x00, 0x00]);
    assert!(matches!(
        cursor.read_packable_hash_entries::<u16, u16, ArchiveError>(
            |cursor| cursor.read_u16(),
            |cursor| cursor.read_u16(),
        ),
        Err(ArchiveError::InvalidPackableHashHeader {
            bucket_count: 0,
            entry_count: 3
        })
    ));
}
