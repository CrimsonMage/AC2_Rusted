use super::collections::ArchiveCollectionLimits;
use super::ArchiveCursor;
use crate::ArchiveError;

mod collection_tail;

#[test]
fn count_helpers_promote_to_usize() {
    let bytes = [3, 4, 0, 5, 0, 0, 0, 0x82, 0x34, 0x56];
    let mut cursor = ArchiveCursor::new(&bytes);

    assert_eq!(cursor.read_u8_count().unwrap(), 3);
    assert_eq!(cursor.read_u16_count().unwrap(), 4);
    assert_eq!(cursor.read_u32_count().unwrap(), 5);
    assert_eq!(cursor.read_retail_compressed_u32_count().unwrap(), 564);
}

#[test]
fn signed_count_helper_rejects_negative_values() {
    let bytes = (-1_i32).to_le_bytes();
    let mut cursor = ArchiveCursor::new(&bytes);

    assert!(matches!(
        cursor.read_i32_non_negative_count(),
        Err(ArchiveError::NegativeCount {
            context: "i32 item count",
            value: -1
        })
    ));
}

#[test]
fn unbounded_counted_items_do_not_reserve_from_a_truncated_count() {
    let bytes = u32::MAX.to_le_bytes();
    let mut cursor = ArchiveCursor::new(&bytes);

    assert!(matches!(
        cursor.read_counted_items_u32::<_, ArchiveError>(|cursor| cursor.read_u8()),
        Err(ArchiveError::UnexpectedEof {
            position: 4,
            requested: 1,
            len: 4
        })
    ));
}

#[test]
fn counted_vec_helpers_decode_items_once() {
    let bytes = [0x02, 0x00, 0x10, 0x00, 0x20, 0x00, 0x02, 0xAA, 0xBB];
    let mut cursor = ArchiveCursor::new(&bytes);

    let u16_values = cursor
        .read_counted_vec_u16::<_, ArchiveError>(|cursor| cursor.read_u16())
        .unwrap();
    let u8_values = cursor
        .read_counted_vec_u8::<_, ArchiveError>(|cursor| cursor.read_u8())
        .unwrap();

    assert_eq!(u16_values, vec![0x0010, 0x0020]);
    assert_eq!(u8_values, vec![0xAA, 0xBB]);
    assert_eq!(cursor.remaining(), 0);
}

#[test]
fn counted_item_helper_uses_caller_supplied_count() {
    let mut cursor = ArchiveCursor::new(&[1, 2, 3, 4]);

    let values = cursor
        .read_counted_items::<_, ArchiveError>(3, |cursor| cursor.read_u8())
        .unwrap();

    assert_eq!(values, vec![1, 2, 3]);
    assert_eq!(cursor.position(), 3);
}

#[test]
fn bounded_counted_item_helper_checks_limit_and_minimum_bytes_before_allocation() {
    let mut cursor = ArchiveCursor::new(&[1, 2, 3]);
    let limits = ArchiveCollectionLimits::new(4, 1);

    assert!(matches!(
        cursor.read_counted_items_with_limits::<_, ArchiveError>(
            4,
            limits,
            "bounded test items",
            |cursor| cursor.read_u8(),
        ),
        Err(ArchiveError::UnexpectedEof {
            position: 0,
            requested: 4,
            len: 3,
        })
    ));
    assert_eq!(cursor.position(), 0);

    assert!(matches!(
        cursor.read_counted_items_with_limits::<_, ArchiveError>(
            5,
            limits,
            "bounded test items",
            |cursor| cursor.read_u8(),
        ),
        Err(ArchiveError::CollectionLimitExceeded {
            context: "bounded test items",
            count: 5,
            max: 4,
        })
    ));
    assert_eq!(cursor.position(), 0);
}

#[test]
fn bounded_count_header_helpers_preflight_before_decoding_items() {
    let limits = ArchiveCollectionLimits::new(2, 1);
    let mut u32_counted = ArchiveCursor::new(&[2, 0, 0, 0, 0x11, 0x22]);
    assert_eq!(
        u32_counted
            .read_counted_items_u32_with_limits(limits, "u32 test items", |cursor| {
                cursor.read_u8()
            })
            .unwrap(),
        vec![0x11, 0x22]
    );

    let mut retail_counted = ArchiveCursor::new(&[2, 0x33, 0x44]);
    assert_eq!(
        retail_counted
            .read_counted_items_retail_compressed_u32_with_limits(
                limits,
                "retail test items",
                |cursor| cursor.read_u8(),
            )
            .unwrap(),
        vec![0x33, 0x44]
    );

    let mut rejected = ArchiveCursor::new(&[2, 0, 0, 0, 0x55, 0x66]);
    assert!(matches!(
        rejected.read_counted_map_u32_with_limits(
            ArchiveCollectionLimits::new(1, 1),
            "u32 test map",
            |cursor| cursor.read_u8(),
            |cursor| cursor.read_u8(),
        ),
        Err(ArchiveError::CollectionLimitExceeded {
            count: 2,
            max: 1,
            ..
        })
    ));
    assert_eq!(rejected.position(), 4);
}

#[test]
fn fixed_width_item_helpers_preflight_and_decode_all_primitive_families() {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0x11, 0x22]);
    bytes.extend_from_slice(&0x3344_u16.to_le_bytes());
    bytes.extend_from_slice(&(-55_i32).to_le_bytes());
    bytes.extend_from_slice(&(-66_i64).to_le_bytes());
    bytes.extend_from_slice(&1.5_f32.to_le_bytes());
    bytes.extend_from_slice(&(-2.5_f64).to_le_bytes());
    let mut cursor = ArchiveCursor::new(&bytes);

    assert_eq!(cursor.read_u8_items(2).unwrap(), vec![0x11, 0x22]);
    assert_eq!(cursor.read_u16_items(1).unwrap(), vec![0x3344]);
    assert_eq!(cursor.read_i32_items(1).unwrap(), vec![-55]);
    assert_eq!(cursor.read_i64_items(1).unwrap(), vec![-66]);
    assert_eq!(cursor.read_f32_items(1).unwrap(), vec![1.5]);
    assert_eq!(cursor.read_f64_items(1).unwrap(), vec![-2.5]);
    assert_eq!(cursor.remaining(), 0);

    let mut truncated = ArchiveCursor::new(&[1, 0, 0]);
    assert!(matches!(
        truncated.read_i32_items(1),
        Err(ArchiveError::UnexpectedEof {
            position: 0,
            requested: 4,
            len: 3,
        })
    ));
    assert_eq!(truncated.position(), 0);
}

#[test]
fn sized_slice_helper_reads_exact_bytes_and_rejects_overflow() {
    let mut cursor = ArchiveCursor::new(&[1, 2, 3, 4, 5, 6]);
    assert_eq!(
        cursor
            .read_sized_slice(2, 3, "slice byte length overflow")
            .unwrap(),
        &[1, 2, 3, 4, 5, 6]
    );

    let mut overflow = ArchiveCursor::new(&[]);
    assert!(matches!(
        overflow.read_sized_slice(2, usize::MAX, "slice byte length overflow"),
        Err(ArchiveError::ValueOutOfRange {
            context: "slice byte length overflow",
            value: 2,
            ..
        })
    ));
}

#[test]
fn fixed_width_fit_helper_preserves_position_and_rejects_overflow_and_truncation() {
    let cursor = ArchiveCursor::new(&[1, 2, 3, 4, 5, 6]);
    let start = cursor.position();
    cursor
        .ensure_items_fit_remaining(2, 3, "fit byte length overflow")
        .unwrap();
    assert_eq!(cursor.position(), start);

    assert!(matches!(
        cursor.ensure_items_fit_remaining(2, 4, "fit byte length overflow"),
        Err(ArchiveError::UnexpectedEof {
            position: 0,
            requested: 8,
            len: 6
        })
    ));

    let overflow = ArchiveCursor::new(&[]);
    assert!(matches!(
        overflow.ensure_items_fit_remaining(2, usize::MAX, "fit byte length overflow"),
        Err(ArchiveError::ValueOutOfRange {
            context: "fit byte length overflow",
            value: 2,
            ..
        })
    ));
}

#[test]
fn aligned_u32_array_helper_reads_counted_values() {
    let bytes = [0xFF, 0x00, 0x00, 0x00, 2, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0];
    let mut cursor = ArchiveCursor::new(&bytes);
    cursor.set_position(1).unwrap();

    assert_eq!(
        cursor
            .read_u32_array_with_alignment(Some(4), Some(4))
            .unwrap(),
        vec![1, 2]
    );
    assert_eq!(cursor.position(), bytes.len());
}

#[test]
fn aligned_u16_array_helper_reads_zero_count() {
    let bytes = [0xAA, 0x00, 0x00, 0x00, 0x00, 0x00];
    let mut cursor = ArchiveCursor::new(&bytes);
    cursor.set_position(1).unwrap();

    let values = cursor
        .read_u16_array_with_alignment(Some(4), Some(2))
        .unwrap();
    assert!(values.is_empty());
}

#[test]
fn aligned_array_helpers_reject_truncated_items() {
    let bytes = [2, 0, 0, 0, 1, 0, 0, 0];
    let mut cursor = ArchiveCursor::new(&bytes);

    assert!(matches!(
        cursor.read_u32_array_with_alignment(None, None),
        Err(ArchiveError::UnexpectedEof {
            position: 8,
            requested: 4,
            len: 8
        })
    ));
}

#[test]
fn aligned_array_helpers_reject_invalid_alignment() {
    let bytes = [0x01, 0x00, 0x10, 0x00];
    let mut cursor = ArchiveCursor::new(&bytes);

    assert!(matches!(
        cursor.read_u16_array_with_alignment(Some(3), None),
        Err(ArchiveError::InvalidAlignment { align: 3 })
    ));
}

#[test]
fn counted_u32_item_helper_reads_zero_count() {
    let bytes = [0, 0, 0, 0];
    let mut cursor = ArchiveCursor::new(&bytes);

    let values = cursor
        .read_counted_items_u32::<_, ArchiveError>(|cursor| cursor.read_u8())
        .unwrap();

    assert!(values.is_empty());
    assert_eq!(cursor.remaining(), 0);
}

#[test]
fn counted_u32_item_helper_reads_items_once() {
    let bytes = [2, 0, 0, 0, 0x10, 0x00, 0x20, 0x00];
    let mut cursor = ArchiveCursor::new(&bytes);

    let values = cursor
        .read_counted_items_u32::<_, ArchiveError>(|cursor| cursor.read_u16())
        .unwrap();

    assert_eq!(values, vec![0x0010, 0x0020]);
    assert_eq!(cursor.remaining(), 0);
}

#[test]
fn counted_u32_item_helper_honors_alignment() {
    let bytes = [0xFF, 0x00, 0x00, 0x00, 2, 0, 0, 0, 1, 0, 2, 0];
    let mut cursor = ArchiveCursor::new(&bytes);
    cursor.set_position(1).unwrap();

    let values = cursor
        .read_counted_items_u32_with_alignment::<_, ArchiveError>(Some(4), Some(2), |cursor| {
            cursor.read_u16()
        })
        .unwrap();

    assert_eq!(values, vec![1, 2]);
    assert_eq!(cursor.position(), bytes.len());
}

#[test]
fn counted_u32_item_helper_rejects_truncated_items() {
    let bytes = [2, 0, 0, 0, 1, 0, 0, 0];
    let mut cursor = ArchiveCursor::new(&bytes);

    assert!(matches!(
        cursor.read_counted_items_u32::<_, ArchiveError>(|cursor| cursor.read_u32()),
        Err(ArchiveError::UnexpectedEof {
            position: 8,
            requested: 4,
            len: 8
        })
    ));
}

#[test]
fn counted_u32_item_helper_rejects_invalid_alignment() {
    let bytes = [0x01, 0x00, 0x10, 0x00];
    let mut cursor = ArchiveCursor::new(&bytes);

    assert!(matches!(
        cursor.read_counted_items_u32_with_alignment::<_, ArchiveError>(Some(3), None, |cursor| {
            cursor.read_u16()
        }),
        Err(ArchiveError::InvalidAlignment { align: 3 })
    ));
}

#[test]
fn counted_u64_item_helper_reads_items_and_rejects_truncated_payload() {
    let bytes = [1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0];
    let mut cursor = ArchiveCursor::new(&bytes);
    assert_eq!(cursor.read_u64_items(2).unwrap(), vec![1, 2]);

    let mut truncated = ArchiveCursor::new(&bytes[..12]);
    assert!(matches!(
        truncated.read_u64_items(2),
        Err(ArchiveError::UnexpectedEof {
            position: 0,
            requested: 16,
            len: 12
        })
    ));
    assert_eq!(truncated.position(), 0);
}

#[test]
fn multiplied_u32_item_helper_reads_items_and_rejects_overflow() {
    let bytes = [1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0];
    let mut cursor = ArchiveCursor::new(&bytes);
    assert_eq!(
        cursor
            .read_multiplied_u32_items(2, 2, "u32 item total overflow")
            .unwrap(),
        vec![1, 2, 3, 4]
    );

    let mut overflow = ArchiveCursor::new(&[]);
    assert!(matches!(
        overflow.read_multiplied_u32_items(2, usize::MAX, "u32 item total overflow"),
        Err(ArchiveError::ValueOutOfRange {
            context: "u32 item total overflow",
            value: 2,
            ..
        })
    ));
}

#[test]
fn counted_u32_map_helper_reads_zero_count() {
    let bytes = [0, 0, 0, 0];
    let mut cursor = ArchiveCursor::new(&bytes);

    let values = cursor
        .read_counted_map_u32::<_, _, ArchiveError>(
            |cursor| cursor.read_u8(),
            |cursor| cursor.read_u16(),
        )
        .unwrap();

    assert!(values.is_empty());
    assert_eq!(cursor.remaining(), 0);
}

#[test]
fn counted_u32_map_helper_reads_entries_once() {
    let bytes = [2, 0, 0, 0, 1, 0, 0x10, 0, 2, 0, 0x20, 0];
    let mut cursor = ArchiveCursor::new(&bytes);

    let values = cursor
        .read_counted_map_u32::<_, _, ArchiveError>(
            |cursor| cursor.read_u16(),
            |cursor| cursor.read_u16(),
        )
        .unwrap();

    assert_eq!(values, vec![(1, 0x0010), (2, 0x0020)]);
    assert_eq!(cursor.remaining(), 0);
}

#[test]
fn counted_u32_map_helper_honors_alignment() {
    let bytes = [
        0xFF, 0x00, 0x00, 0x00, 2, 0, 0, 0, 1, 0, 0x10, 0, 2, 0, 0x20, 0,
    ];
    let mut cursor = ArchiveCursor::new(&bytes);
    cursor.set_position(1).unwrap();

    let values = cursor
        .read_counted_map_u32_with_alignment::<_, _, ArchiveError>(
            Some(4),
            Some(2),
            |cursor| cursor.read_u16(),
            |cursor| cursor.read_u16(),
        )
        .unwrap();

    assert_eq!(values, vec![(1, 0x0010), (2, 0x0020)]);
    assert_eq!(cursor.position(), bytes.len());
}

#[test]
fn counted_u32_map_helper_rejects_truncated_value_payload() {
    let bytes = [1, 0, 0, 0, 1, 0];
    let mut cursor = ArchiveCursor::new(&bytes);

    assert!(matches!(
        cursor.read_counted_map_u32::<_, _, ArchiveError>(
            |cursor| cursor.read_u16(),
            |cursor| cursor.read_u16()
        ),
        Err(ArchiveError::UnexpectedEof {
            position: 6,
            requested: 2,
            len: 6
        })
    ));
}

#[test]
fn counted_u32_map_helper_rejects_invalid_alignment() {
    let bytes = [0x01, 0x00, 0x10, 0x00];
    let mut cursor = ArchiveCursor::new(&bytes);

    assert!(matches!(
        cursor.read_counted_map_u32_with_alignment::<_, _, ArchiveError>(
            Some(3),
            None,
            |cursor| cursor.read_u16(),
            |cursor| cursor.read_u16()
        ),
        Err(ArchiveError::InvalidAlignment { align: 3 })
    ));
}

#[test]
fn counted_retail_u32_item_helper_reads_zero_count() {
    let bytes = [0];
    let mut cursor = ArchiveCursor::new(&bytes);

    let values = cursor
        .read_counted_items_retail_compressed_u32::<_, ArchiveError>(|cursor| cursor.read_u8())
        .unwrap();

    assert!(values.is_empty());
    assert_eq!(cursor.remaining(), 0);
}

#[test]
fn counted_retail_u32_item_helper_reads_items_once() {
    let bytes = [0x02, 0x10, 0x00, 0x20, 0x00];
    let mut cursor = ArchiveCursor::new(&bytes);

    let values = cursor
        .read_counted_items_retail_compressed_u32::<_, ArchiveError>(|cursor| cursor.read_u16())
        .unwrap();

    assert_eq!(values, vec![0x0010, 0x0020]);
    assert_eq!(cursor.remaining(), 0);
}

#[test]
fn counted_retail_u32_item_helper_honors_alignment() {
    let bytes = [0xFF, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01, 0x00, 0x02, 0x00];
    let mut cursor = ArchiveCursor::new(&bytes);
    cursor.set_position(1).unwrap();

    let values = cursor
        .read_counted_items_retail_compressed_u32_with_alignment::<_, ArchiveError>(
            Some(4),
            Some(2),
            |cursor| cursor.read_u16(),
        )
        .unwrap();

    assert_eq!(values, vec![1, 2]);
    assert_eq!(cursor.position(), bytes.len());
}

#[test]
fn counted_retail_u32_item_helper_rejects_truncated_count() {
    let bytes = [0xC0, 0x01, 0x02];
    let mut cursor = ArchiveCursor::new(&bytes);

    assert!(matches!(
        cursor
            .read_counted_items_retail_compressed_u32::<_, ArchiveError>(|cursor| cursor.read_u8()),
        Err(ArchiveError::UnexpectedEof {
            position: 2,
            requested: 2,
            len: 3
        })
    ));
}

#[test]
fn counted_retail_u32_map_helper_reads_zero_count() {
    let bytes = [0];
    let mut cursor = ArchiveCursor::new(&bytes);

    let values = cursor
        .read_counted_map_retail_compressed_u32::<_, _, ArchiveError>(
            |cursor| cursor.read_u8(),
            |cursor| cursor.read_u16(),
        )
        .unwrap();

    assert!(values.is_empty());
    assert_eq!(cursor.remaining(), 0);
}

#[test]
fn counted_retail_u32_map_helper_reads_entries_once() {
    let bytes = [0x02, 1, 0x10, 0, 2, 0x20, 0];
    let mut cursor = ArchiveCursor::new(&bytes);

    let values = cursor
        .read_counted_map_retail_compressed_u32::<_, _, ArchiveError>(
            |cursor| cursor.read_u8(),
            |cursor| cursor.read_u16(),
        )
        .unwrap();

    assert_eq!(values, vec![(1, 0x0010), (2, 0x0020)]);
    assert_eq!(cursor.remaining(), 0);
}

#[test]
fn counted_retail_u32_map_helper_honors_alignment() {
    let bytes = [
        0xFF, 0x00, 0x00, 0x00, 0x02, 0x00, 1, 0x10, 0, 0x00, 2, 0x20, 0,
    ];
    let mut cursor = ArchiveCursor::new(&bytes);
    cursor.set_position(1).unwrap();

    let values = cursor
        .read_counted_map_retail_compressed_u32_with_alignment::<_, _, ArchiveError>(
            Some(4),
            Some(2),
            |cursor| cursor.read_u8(),
            |cursor| cursor.read_u16(),
        )
        .unwrap();

    assert_eq!(values, vec![(1, 0x0010), (2, 0x0020)]);
    assert_eq!(cursor.position(), bytes.len());
}

#[test]
fn counted_retail_u32_map_helper_rejects_truncated_count() {
    let bytes = [0xC0, 0x01, 0x02];
    let mut cursor = ArchiveCursor::new(&bytes);

    assert!(matches!(
        cursor.read_counted_map_retail_compressed_u32::<_, _, ArchiveError>(
            |cursor| cursor.read_u8(),
            |cursor| cursor.read_u16()
        ),
        Err(ArchiveError::UnexpectedEof {
            position: 2,
            requested: 2,
            len: 3
        })
    ));
}

#[test]
fn counted_retail_u32_item_helper_rejects_truncated_items() {
    let bytes = [0x02, 0x10, 0x00];
    let mut cursor = ArchiveCursor::new(&bytes);

    assert!(matches!(
        cursor.read_counted_items_retail_compressed_u32::<_, ArchiveError>(
            |cursor| cursor.read_u16()
        ),
        Err(ArchiveError::UnexpectedEof {
            position: 3,
            requested: 2,
            len: 3
        })
    ));
}

#[test]
fn counted_retail_u32_item_helper_rejects_invalid_alignment() {
    let bytes = [0x01, 0x10, 0x00];
    let mut cursor = ArchiveCursor::new(&bytes);

    assert!(matches!(
        cursor.read_counted_items_retail_compressed_u32_with_alignment::<_, ArchiveError>(
            Some(3),
            None,
            |cursor| cursor.read_u16(),
        ),
        Err(ArchiveError::InvalidAlignment { align: 3 })
    ));
}

#[test]
fn packable_hash_helper_reads_entries_from_retail_header() {
    let bytes = [
        0x03, 0x00, 0x02, 0x00, 0x11, 0x00, 0x21, 0x00, 0x12, 0x00, 0x22, 0x00, 0x13, 0x00, 0x23,
        0x00,
    ];
    let mut cursor = ArchiveCursor::new(&bytes);

    let entries = cursor
        .read_packable_hash_entries::<_, _, ArchiveError>(
            |cursor| cursor.read_u16(),
            |cursor| cursor.read_u16(),
        )
        .unwrap();

    assert_eq!(
        entries,
        vec![(0x0011, 0x0021), (0x0012, 0x0022), (0x0013, 0x0023)]
    );
}

#[test]
fn intrusive_hash_helper_reads_retail_header() {
    let mut cursor = ArchiveCursor::new(&[0x2A, 0x03]);

    assert_eq!(cursor.read_intrusive_hash_entry_count().unwrap(), 3);
    assert_eq!(cursor.remaining(), 0);
}

#[test]
fn intrusive_hash_len_helper_reads_archive_header() {
    let mut cursor = ArchiveCursor::new(&[0x2A, 0x03, 0x00]);

    assert_eq!(cursor.read_intrusive_hash_len_entry_count().unwrap(), 3);
    assert_eq!(cursor.remaining(), 0);
}

#[test]
fn validated_intrusive_hash_helper_reads_nonzero_bucket_header() {
    let mut cursor = ArchiveCursor::new(&[0x01, 0x03]);

    assert_eq!(
        cursor.read_validated_intrusive_hash_entry_count().unwrap(),
        3
    );
    assert_eq!(cursor.remaining(), 0);
}

#[test]
fn validated_intrusive_hash_helper_accepts_empty_zero_bucket_header() {
    let mut cursor = ArchiveCursor::new(&[0x00, 0x00]);

    assert_eq!(
        cursor.read_validated_intrusive_hash_entry_count().unwrap(),
        0
    );
    assert_eq!(cursor.remaining(), 0);
}

#[test]
fn validated_intrusive_hash_helper_rejects_entries_without_buckets() {
    let mut cursor = ArchiveCursor::new(&[0x00, 0x03]);

    assert!(matches!(
        cursor.read_validated_intrusive_hash_entry_count(),
        Err(ArchiveError::InvalidIntrusiveHashHeader {
            bucket_count: 0,
            entry_count: 3
        })
    ));
}

#[test]
fn validated_intrusive_hash_helper_rejects_truncated_header() {
    let mut cursor = ArchiveCursor::new(&[0x2A]);

    assert!(matches!(
        cursor.read_validated_intrusive_hash_entry_count(),
        Err(ArchiveError::UnexpectedEof {
            position: 1,
            requested: 1,
            len: 1
        })
    ));
}

#[test]
fn retail_intrusive_hash_list_helper_reads_header() {
    let mut cursor = ArchiveCursor::new(&[0x01, 0x02]);

    assert_eq!(
        cursor
            .read_retail_intrusive_hash_list_entry_count()
            .unwrap(),
        2
    );
    assert_eq!(cursor.remaining(), 0);
}

#[test]
fn intrusive_hash_list_helper_reads_archive_header() {
    let mut cursor = ArchiveCursor::new(&[0x03, 0x00, 0x02, 0x00]);

    assert_eq!(cursor.read_intrusive_hash_list_entry_count().unwrap(), 2);
    assert_eq!(cursor.remaining(), 0);
}

#[test]
fn intrusive_hash_list_helper_rejects_entries_above_retail_bound() {
    let mut cursor = ArchiveCursor::new(&[0x01, 0x00, 0x03, 0x00]);

    assert!(matches!(
        cursor.read_intrusive_hash_list_entry_count(),
        Err(ArchiveError::InvalidIntrusiveHashListHeader {
            bucket_count: 1,
            entry_count: 3
        })
    ));
}

#[test]
fn intrusive_hash_helper_rejects_truncated_header() {
    let mut cursor = ArchiveCursor::new(&[0x2A]);

    assert!(matches!(
        cursor.read_intrusive_hash_entry_count(),
        Err(ArchiveError::UnexpectedEof {
            position: 1,
            requested: 1,
            len: 1
        })
    ));
}
