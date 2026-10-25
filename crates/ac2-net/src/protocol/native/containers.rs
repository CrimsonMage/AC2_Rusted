//! The script containers (`gAList`, `gRList`, `gLList`, `gAArray`, ..., `gAAHash`, `gARHash`,
//! `gLAHash`, `gNRHash`, `gAAMultiHash`, `gAHashSet`, `gLAHashSet`) as
//! `CNativeFactory::PackNativeObject` packs them: letters name the key and value kinds, `A` a
//! 32-bit atom, `L` a 64-bit long, `R` a reference id, `N` a `ReferenceIDWrapper` key (the
//! reference id, its cached hash and the use-cache flag).
//!
//! Lists and arrays are a u32 count and the elements (`BaseList::Pack`, `PSmartArray`); hashes
//! and hash sets are the `PHashTable::StreamPack` header (the count, the bucket-size index
//! `<< 24`) and the pairs or keys, in table order; `BaseMultiHash<unsigned long,unsigned
//! long,4>::Pack` is a plain u32 count and the pairs. Reference ids are the sender's: the
//! receiver maps them with the packed graph's ids.

use super::ntype;
use crate::protocol::reader::{Reader, Writer, MAX_ITEMS};
use crate::protocol::ProtocolError;

/// An element or key width.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Width {
    U32,
    U64,
    /// `ReferenceIDWrapper::StreamPack`: three words.
    Wrapper,
}

impl Width {
    fn bytes(self) -> usize {
        match self {
            Self::U32 => 4,
            Self::U64 => 8,
            Self::Wrapper => 12,
        }
    }
}

/// The container shape a native type packs as: (key, value) widths and the header kind.
enum Shape {
    List(Width),
    Hash(Width, Option<Width>),
    MultiHash,
}

fn shape(native_type: u32) -> Option<Shape> {
    use ntype::*;
    use Width::*;
    Some(match native_type {
        A_LIST | R_LIST | A_ARRAY | R_ARRAY => Shape::List(U32),
        L_LIST | L_ARRAY => Shape::List(U64),
        AA_HASH | AR_HASH => Shape::Hash(U32, Some(U32)),
        AL_HASH => Shape::Hash(U32, Some(U64)),
        LA_HASH | LR_HASH => Shape::Hash(U64, Some(U32)),
        LL_HASH => Shape::Hash(U64, Some(U64)),
        NA_HASH | NR_HASH => Shape::Hash(Wrapper, Some(U32)),
        A_HASH_SET => Shape::Hash(U32, None),
        LA_HASH_SET => Shape::Hash(U64, None),
        AA_MULTI_HASH => Shape::MultiHash,
        _ => return None,
    })
}

/// Whether `native_type` is one of the containers.
pub(super) fn is_container(native_type: u32) -> bool {
    shape(native_type).is_some()
}

/// A list or array; elements widened to u64 (the native type says how they pack).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptList {
    pub native_type: u32,
    pub items: Vec<u64>,
}

/// A hash key: a 32/64-bit value, or a `ReferenceIDWrapper`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HashKey {
    Value(u64),
    Ref {
        id: u32,
        cached_hash: u32,
        use_cache: u32,
    },
}

/// A hash, multi-hash or hash set (`values` empty for a set), in packed order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptHash {
    pub native_type: u32,
    /// The `g_bucketSizes` index from the `PHashTable` header (0 for a multi-hash).
    pub bucket_index: u8,
    pub keys: Vec<HashKey>,
    pub values: Vec<u64>,
}

fn read_width(width: Width, r: &mut Reader<'_>) -> Result<u64, ProtocolError> {
    Ok(match width {
        Width::U32 => u64::from(r.u32()?),
        _ => r.u64()?,
    })
}

fn write_width(width: Width, value: u64, w: &mut Writer) {
    match width {
        Width::U32 => w.u32(value as u32),
        _ => w.u64(value),
    };
}

fn checked_count(count: usize, item: usize, r: &Reader<'_>) -> Result<usize, ProtocolError> {
    if count > MAX_ITEMS || count.saturating_mul(item) > r.remaining() {
        return Err(ProtocolError::Invalid("container count"));
    }
    Ok(count)
}

/// A container native, or `None` when `native_type` is not one.
pub(super) fn read(
    native_type: u32,
    r: &mut Reader<'_>,
) -> Option<Result<super::NativeObject, ProtocolError>> {
    let shape = shape(native_type)?;
    let read = |r: &mut Reader<'_>| -> Result<super::NativeObject, ProtocolError> {
        Ok(match shape {
            Shape::List(width) => {
                let n = checked_count(r.u32()? as usize, width.bytes(), r)?;
                let items = (0..n)
                    .map(|_| read_width(width, r))
                    .collect::<Result<_, _>>()?;
                super::NativeObject::List(ScriptList { native_type, items })
            }
            Shape::Hash(key, value) => {
                let header = r.u32()?;
                let item = key.bytes() + value.map_or(0, Width::bytes);
                let n = checked_count((header & 0x00FF_FFFF) as usize, item, r)?;
                let mut hash = ScriptHash {
                    native_type,
                    bucket_index: (header >> 24) as u8,
                    keys: Vec::with_capacity(n),
                    values: Vec::new(),
                };
                for _ in 0..n {
                    hash.keys.push(match key {
                        Width::Wrapper => HashKey::Ref {
                            id: r.u32()?,
                            cached_hash: r.u32()?,
                            use_cache: r.u32()?,
                        },
                        width => HashKey::Value(read_width(width, r)?),
                    });
                    if let Some(value) = value {
                        hash.values.push(read_width(value, r)?);
                    }
                }
                super::NativeObject::Hash(hash)
            }
            Shape::MultiHash => {
                let n = checked_count(r.u32()? as usize, 8, r)?;
                let mut hash = ScriptHash {
                    native_type,
                    bucket_index: 0,
                    keys: Vec::with_capacity(n),
                    values: Vec::with_capacity(n),
                };
                for _ in 0..n {
                    hash.keys.push(HashKey::Value(u64::from(r.u32()?)));
                    hash.values.push(u64::from(r.u32()?));
                }
                super::NativeObject::Hash(hash)
            }
        })
    };
    Some(read(r))
}

impl ScriptList {
    pub(super) fn write(&self, w: &mut Writer) {
        let width = match shape(self.native_type) {
            Some(Shape::List(width)) => width,
            _ => Width::U32,
        };
        w.u32(self.items.len() as u32);
        for item in &self.items {
            write_width(width, *item, w);
        }
    }
}

impl ScriptHash {
    pub(super) fn write(&self, w: &mut Writer) {
        let (key, value, multi) = match shape(self.native_type) {
            Some(Shape::Hash(key, value)) => (key, value, false),
            _ => (Width::U32, Some(Width::U32), true),
        };
        if multi {
            w.u32(self.keys.len() as u32);
        } else {
            w.u32(self.keys.len() as u32 & 0x00FF_FFFF | u32::from(self.bucket_index) << 24);
        }
        for (i, k) in self.keys.iter().enumerate() {
            match *k {
                HashKey::Ref {
                    id,
                    cached_hash,
                    use_cache,
                } => {
                    w.u32(id).u32(cached_hash).u32(use_cache);
                }
                HashKey::Value(v) => write_width(key, v, w),
            }
            if let Some(value) = value {
                write_width(value, self.values.get(i).copied().unwrap_or(0), w);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::NativeObject;
    use super::*;

    fn round_trip(object: NativeObject) -> Vec<u8> {
        let mut w = Writer::new();
        object.write(&mut w);
        let mut r = Reader::new(&w.bytes);
        assert_eq!(
            NativeObject::read(object.native_type(), &mut r).unwrap(),
            object
        );
        r.finish().unwrap();
        w.bytes
    }

    fn words(w: &[u32]) -> Vec<u8> {
        w.iter().flat_map(|w| w.to_le_bytes()).collect()
    }

    #[test]
    fn lists_are_a_count_and_their_elements() {
        let short = ScriptList {
            native_type: ntype::R_LIST,
            items: vec![4, 5],
        };
        assert_eq!(round_trip(NativeObject::List(short)), words(&[2, 4, 5]));
        let long = ScriptList {
            native_type: ntype::L_LIST,
            items: vec![0x1_0000_0002],
        };
        assert_eq!(round_trip(NativeObject::List(long)), words(&[1, 2, 1]));
    }

    #[test]
    fn hashes_keep_their_bucket_index_and_key_widths() {
        let ar = ScriptHash {
            native_type: ntype::AR_HASH,
            bucket_index: 3,
            keys: vec![HashKey::Value(7)],
            values: vec![9],
        };
        assert_eq!(
            round_trip(NativeObject::Hash(ar)),
            words(&[0x0300_0001, 7, 9])
        );
        let nr = ScriptHash {
            native_type: ntype::NR_HASH,
            bucket_index: 0,
            keys: vec![HashKey::Ref {
                id: 2,
                cached_hash: 0x55,
                use_cache: 1,
            }],
            values: vec![3],
        };
        assert_eq!(
            round_trip(NativeObject::Hash(nr)),
            words(&[1, 2, 0x55, 1, 3])
        );
        let set = ScriptHash {
            native_type: ntype::LA_HASH_SET,
            bucket_index: 1,
            keys: vec![HashKey::Value(0x2_0000_0001)],
            values: vec![],
        };
        assert_eq!(
            round_trip(NativeObject::Hash(set)),
            words(&[0x0100_0001, 1, 2])
        );
        let multi = ScriptHash {
            native_type: ntype::AA_MULTI_HASH,
            bucket_index: 0,
            keys: vec![HashKey::Value(1), HashKey::Value(1)],
            values: vec![2, 3],
        };
        assert_eq!(
            round_trip(NativeObject::Hash(multi)),
            words(&[2, 1, 2, 1, 3])
        );
        // A count past the data fails before allocating.
        let mut r = Reader::new(&[0xFF, 0xFF, 0xFF, 0x00]);
        assert!(NativeObject::read(ntype::AA_HASH, &mut r).is_err());
    }
}
