use super::ArchiveCursor;
use crate::{ArchiveError, ArchiveResult};

/// Caller-owned bounds for decoding a counted archive collection.
///
/// Archive owns the byte and allocation safety check; typed decoders choose limits appropriate
/// for their resource family instead of inheriting a global policy from this substrate crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArchiveCollectionLimits {
    max_items: usize,
    minimum_item_bytes: usize,
}

impl ArchiveCollectionLimits {
    pub const fn new(max_items: usize, minimum_item_bytes: usize) -> Self {
        Self {
            max_items,
            minimum_item_bytes,
        }
    }

    pub const fn max_items(self) -> usize {
        self.max_items
    }

    pub const fn minimum_item_bytes(self) -> usize {
        self.minimum_item_bytes
    }
}

impl<'a> ArchiveCursor<'a> {
    fn checked_item_span(
        count: usize,
        item_size: usize,
        overflow_context: &'static str,
    ) -> ArchiveResult<usize> {
        count
            .checked_mul(item_size)
            .ok_or_else(|| ArchiveError::ValueOutOfRange {
                context: overflow_context,
                value: count as u64,
                max: (usize::MAX / item_size.max(1)) as u64,
            })
    }

    pub fn read_counted_items<T, E>(
        &mut self,
        count: usize,
        mut decode: impl FnMut(&mut Self) -> Result<T, E>,
    ) -> Result<Vec<T>, E> {
        // Do not reserve from an untrusted count before decoding establishes that items exist.
        // Callers that know a format-specific bound should use `read_counted_items_with_limits`.
        let mut values = Vec::new();
        for _ in 0..count {
            values.push(decode(self)?);
        }
        Ok(values)
    }

    pub fn read_counted_items_with_limits<T, E>(
        &mut self,
        count: usize,
        limits: ArchiveCollectionLimits,
        context: &'static str,
        decode: impl FnMut(&mut Self) -> Result<T, E>,
    ) -> Result<Vec<T>, E>
    where
        E: From<ArchiveError>,
    {
        self.ensure_collection_limits(count, limits, context)
            .map_err(E::from)?;
        self.read_counted_items(count, decode)
    }

    pub fn ensure_collection_limits(
        &self,
        count: usize,
        limits: ArchiveCollectionLimits,
        context: &'static str,
    ) -> ArchiveResult<()> {
        if count > limits.max_items {
            return Err(ArchiveError::CollectionLimitExceeded {
                context,
                count,
                max: limits.max_items,
            });
        }
        if limits.minimum_item_bytes != 0 {
            self.ensure_items_fit_remaining(count, limits.minimum_item_bytes, context)?;
        }
        Ok(())
    }

    pub fn read_sized_slice(
        &mut self,
        count: usize,
        item_size: usize,
        overflow_context: &'static str,
    ) -> ArchiveResult<&'a [u8]> {
        let len = Self::checked_item_span(count, item_size, overflow_context)?;
        self.read_bytes(len)
    }

    pub fn ensure_items_fit_remaining(
        &self,
        count: usize,
        item_size: usize,
        overflow_context: &'static str,
    ) -> ArchiveResult<()> {
        let required = Self::checked_item_span(count, item_size, overflow_context)?;
        if required > self.remaining() {
            return Err(ArchiveError::UnexpectedEof {
                position: self.position(),
                requested: required,
                len: self.len(),
            });
        }
        Ok(())
    }

    pub fn read_u32_items(&mut self, count: usize) -> ArchiveResult<Vec<u32>> {
        self.read_fixed_items(count, std::mem::size_of::<u32>(), Self::read_u32)
    }

    pub fn read_u64_items(&mut self, count: usize) -> ArchiveResult<Vec<u64>> {
        self.read_fixed_items(count, std::mem::size_of::<u64>(), Self::read_u64)
    }

    pub fn read_u8_items(&mut self, count: usize) -> ArchiveResult<Vec<u8>> {
        self.read_fixed_items(count, std::mem::size_of::<u8>(), Self::read_u8)
    }

    pub fn read_u16_items(&mut self, count: usize) -> ArchiveResult<Vec<u16>> {
        self.read_fixed_items(count, std::mem::size_of::<u16>(), Self::read_u16)
    }

    pub fn read_i32_items(&mut self, count: usize) -> ArchiveResult<Vec<i32>> {
        self.read_fixed_items(count, std::mem::size_of::<i32>(), Self::read_i32)
    }

    pub fn read_i64_items(&mut self, count: usize) -> ArchiveResult<Vec<i64>> {
        self.read_fixed_items(count, std::mem::size_of::<i64>(), Self::read_i64)
    }

    pub fn read_f32_items(&mut self, count: usize) -> ArchiveResult<Vec<f32>> {
        self.read_fixed_items(count, std::mem::size_of::<f32>(), Self::read_f32)
    }

    pub fn read_f64_items(&mut self, count: usize) -> ArchiveResult<Vec<f64>> {
        self.read_fixed_items(count, std::mem::size_of::<f64>(), Self::read_f64)
    }

    pub fn read_multiplied_u32_items(
        &mut self,
        base_count: usize,
        multiplier: usize,
        overflow_context: &'static str,
    ) -> ArchiveResult<Vec<u32>> {
        let total = Self::checked_item_span(base_count, multiplier, overflow_context)?;
        self.read_u32_items(total)
    }

    pub fn read_counted_vec_u8<T, E>(
        &mut self,
        decode: impl FnMut(&mut Self) -> Result<T, E>,
    ) -> Result<Vec<T>, E>
    where
        E: From<ArchiveError>,
    {
        let count = self.read_u8_count().map_err(E::from)?;
        self.read_counted_items(count, decode)
    }

    pub fn read_counted_vec_u16<T, E>(
        &mut self,
        decode: impl FnMut(&mut Self) -> Result<T, E>,
    ) -> Result<Vec<T>, E>
    where
        E: From<ArchiveError>,
    {
        let count = self.read_u16_count().map_err(E::from)?;
        self.read_counted_items(count, decode)
    }

    pub fn read_counted_vec_u32<T, E>(
        &mut self,
        decode: impl FnMut(&mut Self) -> Result<T, E>,
    ) -> Result<Vec<T>, E>
    where
        E: From<ArchiveError>,
    {
        let count = self.read_u32_count().map_err(E::from)?;
        self.read_counted_items(count, decode)
    }

    pub fn read_counted_items_u32<T, E>(
        &mut self,
        decode: impl FnMut(&mut Self) -> Result<T, E>,
    ) -> Result<Vec<T>, E>
    where
        E: From<ArchiveError>,
    {
        let count = self.read_u32_count().map_err(E::from)?;
        self.read_counted_items(count, decode)
    }

    pub fn read_counted_items_u32_with_limits<T, E>(
        &mut self,
        limits: ArchiveCollectionLimits,
        context: &'static str,
        decode: impl FnMut(&mut Self) -> Result<T, E>,
    ) -> Result<Vec<T>, E>
    where
        E: From<ArchiveError>,
    {
        self.read_counted_items_u32_with_alignment_and_limits(None, None, limits, context, decode)
    }

    pub fn read_counted_map<K, V, E>(
        &mut self,
        count: usize,
        mut decode_key: impl FnMut(&mut Self) -> Result<K, E>,
        mut decode_value: impl FnMut(&mut Self) -> Result<V, E>,
    ) -> Result<Vec<(K, V)>, E> {
        self.read_counted_items(count, |cursor| {
            Ok((decode_key(cursor)?, decode_value(cursor)?))
        })
    }

    pub fn read_counted_map_u32<K, V, E>(
        &mut self,
        decode_key: impl FnMut(&mut Self) -> Result<K, E>,
        decode_value: impl FnMut(&mut Self) -> Result<V, E>,
    ) -> Result<Vec<(K, V)>, E>
    where
        E: From<ArchiveError>,
    {
        let count = self.read_u32_count().map_err(E::from)?;
        self.read_counted_map(count, decode_key, decode_value)
    }

    pub fn read_counted_map_u32_with_limits<K, V, E>(
        &mut self,
        limits: ArchiveCollectionLimits,
        context: &'static str,
        mut decode_key: impl FnMut(&mut Self) -> Result<K, E>,
        mut decode_value: impl FnMut(&mut Self) -> Result<V, E>,
    ) -> Result<Vec<(K, V)>, E>
    where
        E: From<ArchiveError>,
    {
        self.read_counted_items_u32_with_limits(limits, context, |cursor| {
            Ok((decode_key(cursor)?, decode_value(cursor)?))
        })
    }

    pub fn read_counted_map_u32_with_alignment<K, V, E>(
        &mut self,
        count_align: Option<usize>,
        item_align: Option<usize>,
        mut decode_key: impl FnMut(&mut Self) -> Result<K, E>,
        mut decode_value: impl FnMut(&mut Self) -> Result<V, E>,
    ) -> Result<Vec<(K, V)>, E>
    where
        E: From<ArchiveError>,
    {
        if let Some(align) = count_align {
            self.align_to(align).map_err(E::from)?;
        }
        let count = self.read_u32_count().map_err(E::from)?;
        self.read_counted_items(count, |cursor| {
            if let Some(align) = item_align {
                cursor.align_to(align).map_err(E::from)?;
            }
            Ok((decode_key(cursor)?, decode_value(cursor)?))
        })
    }

    pub fn read_counted_items_u32_with_alignment<T, E>(
        &mut self,
        count_align: Option<usize>,
        item_align: Option<usize>,
        mut decode: impl FnMut(&mut Self) -> Result<T, E>,
    ) -> Result<Vec<T>, E>
    where
        E: From<ArchiveError>,
    {
        if let Some(align) = count_align {
            self.align_to(align).map_err(E::from)?;
        }
        let count = self.read_u32_count().map_err(E::from)?;
        self.read_counted_items(count, |cursor| {
            if let Some(align) = item_align {
                cursor.align_to(align).map_err(E::from)?;
            }
            decode(cursor)
        })
    }

    pub fn read_counted_items_u32_with_alignment_and_limits<T, E>(
        &mut self,
        count_align: Option<usize>,
        item_align: Option<usize>,
        limits: ArchiveCollectionLimits,
        context: &'static str,
        mut decode: impl FnMut(&mut Self) -> Result<T, E>,
    ) -> Result<Vec<T>, E>
    where
        E: From<ArchiveError>,
    {
        if let Some(align) = count_align {
            self.align_to(align).map_err(E::from)?;
        }
        let count = self.read_u32_count().map_err(E::from)?;
        self.read_counted_items_with_limits(count, limits, context, |cursor| {
            if let Some(align) = item_align {
                cursor.align_to(align).map_err(E::from)?;
            }
            decode(cursor)
        })
    }

    pub fn read_counted_items_retail_compressed_u32<T, E>(
        &mut self,
        decode: impl FnMut(&mut Self) -> Result<T, E>,
    ) -> Result<Vec<T>, E>
    where
        E: From<ArchiveError>,
    {
        self.read_counted_items_retail_compressed_u32_with_alignment(None, None, decode)
    }

    pub fn read_counted_items_retail_compressed_u32_with_limits<T, E>(
        &mut self,
        limits: ArchiveCollectionLimits,
        context: &'static str,
        decode: impl FnMut(&mut Self) -> Result<T, E>,
    ) -> Result<Vec<T>, E>
    where
        E: From<ArchiveError>,
    {
        self.read_counted_items_retail_compressed_u32_with_alignment_and_limits(
            None, None, limits, context, decode,
        )
    }

    pub fn read_counted_items_retail_compressed_u32_with_alignment<T, E>(
        &mut self,
        count_align: Option<usize>,
        item_align: Option<usize>,
        mut decode: impl FnMut(&mut Self) -> Result<T, E>,
    ) -> Result<Vec<T>, E>
    where
        E: From<ArchiveError>,
    {
        if let Some(align) = count_align {
            self.align_to(align).map_err(E::from)?;
        }
        let count = self.read_retail_compressed_u32_count().map_err(E::from)?;
        self.read_counted_items(count, |cursor| {
            if let Some(align) = item_align {
                cursor.align_to(align).map_err(E::from)?;
            }
            decode(cursor)
        })
    }

    pub fn read_counted_items_retail_compressed_u32_with_alignment_and_limits<T, E>(
        &mut self,
        count_align: Option<usize>,
        item_align: Option<usize>,
        limits: ArchiveCollectionLimits,
        context: &'static str,
        mut decode: impl FnMut(&mut Self) -> Result<T, E>,
    ) -> Result<Vec<T>, E>
    where
        E: From<ArchiveError>,
    {
        if let Some(align) = count_align {
            self.align_to(align).map_err(E::from)?;
        }
        let count = self.read_retail_compressed_u32_count().map_err(E::from)?;
        self.read_counted_items_with_limits(count, limits, context, |cursor| {
            if let Some(align) = item_align {
                cursor.align_to(align).map_err(E::from)?;
            }
            decode(cursor)
        })
    }

    pub fn read_counted_map_retail_compressed_u32<K, V, E>(
        &mut self,
        decode_key: impl FnMut(&mut Self) -> Result<K, E>,
        decode_value: impl FnMut(&mut Self) -> Result<V, E>,
    ) -> Result<Vec<(K, V)>, E>
    where
        E: From<ArchiveError>,
    {
        self.read_counted_map_retail_compressed_u32_with_alignment(
            None,
            None,
            decode_key,
            decode_value,
        )
    }

    pub fn read_counted_map_retail_compressed_u32_with_limits<K, V, E>(
        &mut self,
        limits: ArchiveCollectionLimits,
        context: &'static str,
        mut decode_key: impl FnMut(&mut Self) -> Result<K, E>,
        mut decode_value: impl FnMut(&mut Self) -> Result<V, E>,
    ) -> Result<Vec<(K, V)>, E>
    where
        E: From<ArchiveError>,
    {
        self.read_counted_items_retail_compressed_u32_with_limits(limits, context, |cursor| {
            Ok((decode_key(cursor)?, decode_value(cursor)?))
        })
    }

    pub fn read_counted_map_retail_compressed_u32_with_alignment<K, V, E>(
        &mut self,
        count_align: Option<usize>,
        item_align: Option<usize>,
        mut decode_key: impl FnMut(&mut Self) -> Result<K, E>,
        mut decode_value: impl FnMut(&mut Self) -> Result<V, E>,
    ) -> Result<Vec<(K, V)>, E>
    where
        E: From<ArchiveError>,
    {
        if let Some(align) = count_align {
            self.align_to(align).map_err(E::from)?;
        }
        let count = self.read_retail_compressed_u32_count().map_err(E::from)?;
        self.read_counted_items(count, |cursor| {
            if let Some(align) = item_align {
                cursor.align_to(align).map_err(E::from)?;
            }
            Ok((decode_key(cursor)?, decode_value(cursor)?))
        })
    }

    pub fn read_counted_vec_retail_compressed_u32<T, E>(
        &mut self,
        decode: impl FnMut(&mut Self) -> Result<T, E>,
    ) -> Result<Vec<T>, E>
    where
        E: From<ArchiveError>,
    {
        self.read_counted_items_retail_compressed_u32(decode)
    }

    pub fn read_u16_array_with_alignment(
        &mut self,
        count_align: Option<usize>,
        item_align: Option<usize>,
    ) -> ArchiveResult<Vec<u16>> {
        if let Some(align) = count_align {
            self.align_to(align)?;
        }
        let count = self.read_u16_count()?;
        self.read_aligned_items(count, item_align, Self::read_u16)
    }

    pub fn read_u32_array_with_alignment(
        &mut self,
        count_align: Option<usize>,
        item_align: Option<usize>,
    ) -> ArchiveResult<Vec<u32>> {
        if let Some(align) = count_align {
            self.align_to(align)?;
        }
        let count = self.read_u32_count()?;
        self.read_aligned_items(count, item_align, Self::read_u32)
    }

    pub fn read_intrusive_hash_entry_count(&mut self) -> ArchiveResult<usize> {
        let _bucket_code = self.read_u8()?;
        self.read_retail_compressed_u32_count()
    }

    pub fn read_validated_intrusive_hash_entry_count(&mut self) -> ArchiveResult<usize> {
        let bucket_count = self.read_u8()?;
        let entry_count = self.read_retail_compressed_u32_count()?;
        if bucket_count == 0 && entry_count != 0 {
            return Err(ArchiveError::InvalidIntrusiveHashHeader {
                bucket_count,
                entry_count,
            });
        }
        Ok(entry_count)
    }

    pub fn read_intrusive_hash_len_entry_count(&mut self) -> ArchiveResult<usize> {
        let _bucket_code = self.read_u8()?;
        self.read_len()
    }

    pub fn read_retail_intrusive_hash_list_entry_count(&mut self) -> ArchiveResult<usize> {
        let _bucket_count = self.read_retail_len()?;
        self.read_retail_compressed_u32_count()
    }

    pub fn read_intrusive_hash_list_entry_count(&mut self) -> ArchiveResult<usize> {
        let bucket_count = self.read_compressed_u32()?;
        let entry_count = self.read_len()?;
        let bucket_count =
            usize::try_from(bucket_count).map_err(|_| ArchiveError::ValueOutOfRange {
                context: "intrusive hash-list bucket count",
                value: u64::from(bucket_count),
                max: usize::MAX as u64,
            })?;
        if bucket_count != 0 && entry_count > bucket_count.saturating_mul(2) {
            return Err(ArchiveError::InvalidIntrusiveHashListHeader {
                bucket_count: bucket_count as u32,
                entry_count,
            });
        }
        Ok(entry_count)
    }

    pub fn read_packable_hash_entries<K, V, E>(
        &mut self,
        mut decode_key: impl FnMut(&mut Self) -> Result<K, E>,
        mut decode_value: impl FnMut(&mut Self) -> Result<V, E>,
    ) -> Result<Vec<(K, V)>, E>
    where
        E: From<ArchiveError>,
    {
        let entry_count = self.read_packable_hash_entry_count().map_err(E::from)?;
        self.read_counted_items(entry_count, |cursor| {
            Ok((decode_key(cursor)?, decode_value(cursor)?))
        })
    }

    pub fn read_packable_hash_entry_count(&mut self) -> ArchiveResult<usize> {
        let header = self.read_u32()?;
        let bucket_count = header >> 16;
        let entry_count = usize::from((header & 0xFFFF) as u16);
        if bucket_count == 0 && entry_count != 0 {
            return Err(ArchiveError::InvalidPackableHashHeader {
                bucket_count,
                entry_count,
            });
        }
        Ok(entry_count)
    }

    fn read_aligned_items<T>(
        &mut self,
        count: usize,
        item_align: Option<usize>,
        mut read_item: impl FnMut(&mut Self) -> ArchiveResult<T>,
    ) -> ArchiveResult<Vec<T>> {
        let mut values = Vec::new();
        for _ in 0..count {
            if let Some(align) = item_align {
                self.align_to(align)?;
            }
            values.push(read_item(self)?);
        }
        Ok(values)
    }

    fn read_fixed_items<T>(
        &mut self,
        count: usize,
        item_size: usize,
        read_item: impl FnMut(&mut Self) -> ArchiveResult<T>,
    ) -> ArchiveResult<Vec<T>> {
        self.ensure_items_fit_remaining(count, item_size, "fixed-width archive items")?;
        self.read_counted_items(count, read_item)
    }
}
