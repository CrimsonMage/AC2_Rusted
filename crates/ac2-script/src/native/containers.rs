//! The script containers (`gAList`, `gRList`, `gLList`, `gAArray`, ..., `gAAHash`, `gARHash`,
//! `gLAHash`, `gNRHash`, `gAAMultiHash`, `gAHashSet`, `gLAHashSet`): letters name the key and
//! value kinds, `A` a 32-bit atom, `L` a 64-bit long, `R` a reference id, `N` a reference id
//! hashed by its native value (`ReferenceIDWrapper`).
//!
//! Packed forms: lists and arrays are a u32 count and the elements (`LList::UnPack`); hashes
//! and hash sets are a `PHashTable` header (count | bucket index << 24) and the pairs or keys;
//! `BaseMultiHash<unsigned long,unsigned long,4>::UnPack` is a plain u32 count and the pairs.

use crate::unpack::{Reader, UnpackResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Elem {
    U32,
    U64,
}

impl Elem {
    fn width(self) -> usize {
        match self {
            Self::U32 => 4,
            Self::U64 => 8,
        }
    }

    fn read(self, r: &mut Reader<'_>) -> UnpackResult<u64> {
        match self {
            Self::U32 => Ok(u64::from(r.u32()?)),
            Self::U64 => r.u64(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Key {
    U32,
    U64,
    /// `ReferenceIDWrapper::StreamPack`: id, cached hash, use-cache flag.
    Ref,
}

/// A hash key: a 32/64-bit value, or a `ReferenceIDWrapper`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RefKey {
    Value(u64),
    Ref {
        id: u32,
        cached_hash: u32,
        use_cache: u32,
    },
}

/// A list or array: elements widened to u64 (`wide` says they were 64-bit).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScriptList {
    pub wide: bool,
    pub items: Vec<u64>,
}

impl ScriptList {
    pub(super) fn unpack(r: &mut Reader<'_>, elem: Elem) -> UnpackResult<Self> {
        let n = r.count(elem.width(), "list count")?;
        Ok(Self {
            wide: elem == Elem::U64,
            items: (0..n).map(|_| elem.read(r)).collect::<UnpackResult<_>>()?,
        })
    }
}

/// A hash, multi-hash or hash set (`values` is empty for a set), in packed order.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScriptHash {
    /// `g_bucketSizes` index from the `PHashTable` header (0 for a multi-hash).
    pub bucket_index: u8,
    pub keys: Vec<RefKey>,
    pub values: Vec<u64>,
}

impl ScriptHash {
    pub(super) fn unpack(r: &mut Reader<'_>, key: Key, value: Option<Elem>) -> UnpackResult<Self> {
        let item = match key {
            Key::U32 => 4,
            Key::U64 => 8,
            Key::Ref => 12,
        } + value.map_or(0, Elem::width);
        let (n, bucket_index) = r.hash_header(item, "hash count")?;
        let mut keys = Vec::with_capacity(n);
        let mut values = Vec::with_capacity(if value.is_some() { n } else { 0 });
        for _ in 0..n {
            keys.push(match key {
                Key::U32 => RefKey::Value(u64::from(r.u32()?)),
                Key::U64 => RefKey::Value(r.u64()?),
                Key::Ref => RefKey::Ref {
                    id: r.u32()?,
                    cached_hash: r.u32()?,
                    use_cache: r.u32()?,
                },
            });
            if let Some(value) = value {
                values.push(value.read(r)?);
            }
        }
        Ok(Self {
            bucket_index,
            keys,
            values,
        })
    }

    /// `BaseMultiHash<unsigned long,unsigned long,4>::UnPack` (`AAMultiHash`).
    pub(super) fn unpack_multi(r: &mut Reader<'_>) -> UnpackResult<Self> {
        let n = r.count(8, "multi-hash count")?;
        let mut hash = Self::default();
        for _ in 0..n {
            hash.keys.push(RefKey::Value(u64::from(r.u32()?)));
            hash.values.push(u64::from(r.u32()?));
        }
        Ok(hash)
    }
}
