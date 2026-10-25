//! The script hashes and hash sets at run time: `IntrusiveHashTable` (bucket chains, new
//! elements at the head of their bucket) behind `HashTable`/`HashSet`, so iteration order,
//! `num_buckets` and every resize match retail.
//!
//! A key's bucket: 32-bit keys `key % buckets`, 64-bit keys `key % buckets` (`_aullrem`), and
//! `ReferenceIDWrapper` keys `VM::HashNativeObject(id, buckets)` unless the wrapper carries a
//! cached bucket (`m_bUseCache`). Hashing and comparing native keys needs the reference table,
//! so it goes through [`NativeKeys`].

use super::containers::RefKey;

/// `g_bucketSizes` (Legions 0x009ECCC0..0x009ECD1C).
pub const BUCKET_SIZES: [u32; 23] = [
    11, 23, 47, 89, 191, 383, 761, 1531, 3067, 6143, 12281, 24571, 49139, 98299, 196597, 393209,
    786431, 1572853, 3145721, 6291449, 12582893, 25165813, 50331599,
];

/// The bucket count `IntrusiveHashTable(0x20)` starts with (every `NewNativeObject` hash).
pub const DEFAULT_BUCKETS: u32 = 47;

/// `_STL::lower_bound(g_bucketSizesBegin, g_bucketSizesEnd, n)`, the end clamped to the last
/// size (`get_bucket_size_pointer`).
pub fn bucket_size_index(n: u32) -> usize {
    BUCKET_SIZES
        .iter()
        .position(|&s| s >= n)
        .unwrap_or(BUCKET_SIZES.len() - 1)
}

/// Hashing and equality of `ReferenceIDWrapper` keys (`VM::HashNativeObject`,
/// `VM::IsNativeEqual`).
pub trait NativeKeys {
    fn hash(&self, id: u32, buckets: u32) -> u32;
    fn equal(&self, a: u32, b: u32) -> bool;
}

/// For tables without native keys.
pub struct PlainKeys;

impl NativeKeys for PlainKeys {
    fn hash(&self, _id: u32, _buckets: u32) -> u32 {
        0
    }
    fn equal(&self, a: u32, b: u32) -> bool {
        a == b
    }
}

/// One `HashTableData` / `HashSetData` element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashNode {
    /// A stable identity for iterators (retail's element pointer).
    pub id: u32,
    pub key: RefKey,
    /// The value (0 in a hash set).
    pub value: u64,
}

/// An `IntrusiveHashTable`: `buckets[i]` is bucket i's chain, head first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptHash {
    pub buckets: Vec<Vec<HashNode>>,
    next_id: u32,
}

impl Default for ScriptHash {
    fn default() -> Self {
        Self::with_buckets(DEFAULT_BUCKETS)
    }
}

/// An iterator position: the bucket and the element (`None` at the end).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HashPos {
    pub bucket: usize,
    pub node: Option<u32>,
}

impl ScriptHash {
    /// `IntrusiveHashTable::init(n)`.
    pub fn with_buckets(n: u32) -> Self {
        Self {
            buckets: vec![Vec::new(); n.max(1) as usize],
            next_id: 0,
        }
    }

    pub fn num_buckets(&self) -> u32 {
        self.buckets.len() as u32
    }

    pub fn num_elements(&self) -> u32 {
        self.buckets.iter().map(Vec::len).sum::<usize>() as u32
    }

    /// The elements in iteration order (buckets ascending, each chain head first).
    pub fn nodes(&self) -> impl Iterator<Item = &HashNode> {
        self.buckets.iter().flatten()
    }

    pub fn nodes_mut(&mut self) -> impl Iterator<Item = &mut HashNode> {
        self.buckets.iter_mut().flatten()
    }

    fn bucket_of(&self, key: &RefKey, keys: &dyn NativeKeys) -> usize {
        let n = self.num_buckets();
        match *key {
            RefKey::Value(v) => (v % u64::from(n)) as usize,
            RefKey::Ref {
                id,
                cached_hash,
                use_cache,
            } => {
                let h = if use_cache != 0 {
                    cached_hash
                } else {
                    keys.hash(id, n)
                };
                // Retail indexes with a stale cached bucket unchecked; keep it in range.
                (h % n) as usize
            }
        }
    }

    fn same(a: &RefKey, b: &RefKey, keys: &dyn NativeKeys) -> bool {
        match (a, b) {
            (RefKey::Ref { id: x, .. }, RefKey::Ref { id: y, .. }) => keys.equal(*x, *y),
            _ => a == b,
        }
    }

    /// `find_internal`: the bucket and the element's index in its chain.
    fn locate(&self, key: &RefKey, keys: &dyn NativeKeys) -> (usize, Option<usize>) {
        let b = self.bucket_of(key, keys);
        let i = self.buckets[b]
            .iter()
            .position(|n| Self::same(&n.key, key, keys));
        (b, i)
    }

    pub fn find(&self, key: &RefKey, keys: &dyn NativeKeys) -> Option<u64> {
        let (b, i) = self.locate(key, keys);
        i.map(|i| self.buckets[b][i].value)
    }

    pub fn contains(&self, key: &RefKey, keys: &dyn NativeKeys) -> bool {
        self.locate(key, keys).1.is_some()
    }

    /// Links a new element at its bucket's head; a native key keeps the bucket it hashed to
    /// (`m_CachedHashValue`, what `Pack` writes).
    fn push_head(&mut self, bucket: usize, mut key: RefKey, value: u64) {
        if let RefKey::Ref { cached_hash, .. } = &mut key {
            *cached_hash = bucket as u32;
        }
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        self.buckets[bucket].insert(0, HashNode { id, key, value });
    }

    /// `HashTable::add` / `HashSet::add`: false when the key is present.
    pub fn add(&mut self, key: RefKey, value: u64, keys: &dyn NativeKeys) -> bool {
        let (b, i) = self.locate(&key, keys);
        if i.is_some() {
            return false;
        }
        self.push_head(b, key, value);
        true
    }

    /// `HashTable::set`: replaces the value or adds; always true.
    pub fn set(&mut self, key: RefKey, value: u64, keys: &dyn NativeKeys) -> bool {
        match self.locate(&key, keys) {
            (b, Some(i)) => self.buckets[b][i].value = value,
            (b, None) => self.push_head(b, key, value),
        }
        true
    }

    /// `IntrusiveHashTable::remove(key)`: the removed element's value.
    pub fn remove(&mut self, key: &RefKey, keys: &dyn NativeKeys) -> Option<u64> {
        let (b, i) = self.locate(key, keys);
        i.map(|i| self.buckets[b].remove(i).value)
    }

    /// `delete_contents`.
    pub fn flush(&mut self) {
        for b in &mut self.buckets {
            b.clear();
        }
    }

    /// `IntrusiveHashTable::resize_internal(n)`: every element is unlinked in iteration order
    /// onto a stack, then re-added (at its new bucket's head) from the stack's top.
    pub fn resize_internal(&mut self, n: u32, keys: &dyn NativeKeys) -> bool {
        if n == self.num_buckets() {
            return false;
        }
        let pulled: Vec<HashNode> = self.buckets.drain(..).flatten().collect();
        self.buckets = vec![Vec::new(); n.max(1) as usize];
        for node in pulled.into_iter().rev() {
            let b = self.bucket_of(&node.key, keys);
            self.buckets[b].insert(0, node);
        }
        true
    }

    /// `HashTable::grow`: the size after `lower_bound(num_buckets)` (retail reads past the
    /// table's end at the last size; the port stays there).
    pub fn grow(&mut self, keys: &dyn NativeKeys) -> bool {
        let i = (bucket_size_index(self.num_buckets()) + 1).min(BUCKET_SIZES.len() - 1);
        self.resize_internal(BUCKET_SIZES[i], keys)
    }

    /// `HashTable::shrink`: the size before `lower_bound(num_buckets)`.
    pub fn shrink(&mut self, keys: &dyn NativeKeys) -> bool {
        let i = bucket_size_index(self.num_buckets()).saturating_sub(1);
        self.resize_internal(BUCKET_SIZES[i], keys)
    }

    /// The glue's `resize(n)`: `lower_bound(n)`.
    pub fn resize(&mut self, n: u32, keys: &dyn NativeKeys) -> bool {
        self.resize_internal(BUCKET_SIZES[bucket_size_index(n)], keys)
    }

    /// `IntrusiveHashTable::optimize_size`.
    pub fn optimize_size(&mut self, keys: &dyn NativeKeys) -> bool {
        let (n, b) = (self.num_elements(), self.num_buckets());
        if n / 2 <= b && b <= n.wrapping_mul(2) {
            return false;
        }
        self.resize_internal(BUCKET_SIZES[bucket_size_index(n)], keys)
    }

    /// `too_big`: `num_elements * 2 < num_buckets`.
    pub fn too_big(&self) -> bool {
        self.num_elements().wrapping_mul(2) < self.num_buckets()
    }

    /// `too_small`: `num_buckets < num_elements / 2`.
    pub fn too_small(&self) -> bool {
        self.num_buckets() < self.num_elements() / 2
    }

    /// `IntrusiveHashTable::begin`.
    pub fn begin(&self) -> HashPos {
        self.first_from(0)
    }

    pub fn end(&self) -> HashPos {
        HashPos {
            bucket: self.buckets.len(),
            node: None,
        }
    }

    fn first_from(&self, bucket: usize) -> HashPos {
        (bucket..self.buckets.len())
            .find_map(|b| {
                self.buckets[b].first().map(|n| HashPos {
                    bucket: b,
                    node: Some(n.id),
                })
            })
            .unwrap_or(self.end())
    }

    pub fn node(&self, pos: HashPos) -> Option<&HashNode> {
        let id = pos.node?;
        self.buckets.get(pos.bucket)?.iter().find(|n| n.id == id)
    }

    /// `HashIterator::next`: the next element in the chain, else the next non-empty bucket.
    /// `None` when the iterator is at the end or its element is gone (retail reads freed or
    /// null memory there).
    pub fn next(&self, pos: HashPos) -> Option<HashPos> {
        let id = pos.node?;
        let chain = self.buckets.get(pos.bucket)?;
        let i = chain.iter().position(|n| n.id == id)?;
        Some(match chain.get(i + 1) {
            Some(n) => HashPos {
                bucket: pos.bucket,
                node: Some(n.id),
            },
            None => self.first_from(pos.bucket + 1),
        })
    }

    /// `IntrusiveHashIterator::remove`: steps to the next element, then unlinks this one.
    pub fn remove_at(&mut self, pos: HashPos) -> Option<HashPos> {
        let next = self.next(pos)?;
        let id = pos.node?;
        self.buckets[pos.bucket].retain(|n| n.id != id);
        Some(next)
    }

    /// `PHashTable::UnPack`: `g_bucketSizes[index]` buckets, then each element by
    /// `add_for_unpacking` (after the previous element when it shares its bucket, else as its
    /// bucket's only element). An unpacked `ReferenceIDWrapper` uses its packed bucket
    /// (`ReferenceIDWrapper::StreamPack` sets `m_bUseCache` on unpack).
    pub fn from_packed(index: u8, items: Vec<(RefKey, u64)>) -> Self {
        let size = BUCKET_SIZES[usize::from(index).min(BUCKET_SIZES.len() - 1)];
        let mut hash = Self::with_buckets(size);
        let mut prev: Option<usize> = None;
        for (mut key, value) in items {
            if let RefKey::Ref { use_cache, .. } = &mut key {
                *use_cache = 1;
            }
            let b = hash.bucket_of(&key, &PlainKeys);
            let id = hash.next_id;
            hash.next_id += 1;
            let node = HashNode { id, key, value };
            if prev == Some(b) {
                hash.buckets[b].push(node);
            } else {
                hash.buckets[b] = vec![node];
            }
            prev = Some(b);
        }
        hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn k(v: u64) -> RefKey {
        RefKey::Value(v)
    }

    fn order(h: &ScriptHash) -> Vec<u64> {
        h.nodes()
            .map(|n| match n.key {
                RefKey::Value(v) => v,
                RefKey::Ref { id, .. } => u64::from(id),
            })
            .collect()
    }

    #[test]
    fn new_elements_go_to_the_head_of_their_bucket_and_add_refuses_duplicates() {
        let mut h = ScriptHash::default();
        assert_eq!(h.num_buckets(), 47);
        for v in [1, 48, 95, 2] {
            assert!(h.add(k(v), v * 10, &PlainKeys));
        }
        assert!(!h.add(k(48), 0, &PlainKeys));
        assert_eq!(order(&h), vec![95, 48, 1, 2]);
        assert!(h.set(k(48), 7, &PlainKeys));
        assert_eq!(h.find(&k(48), &PlainKeys), Some(7));
        assert_eq!(h.remove(&k(1), &PlainKeys), Some(10));
        assert_eq!(order(&h), vec![95, 48, 2]);
        assert!(h.too_big() && !h.too_small());
    }

    #[test]
    fn resizing_reinserts_from_the_last_element_pulled() {
        let mut h = ScriptHash::with_buckets(11);
        for v in [0, 11, 22, 1, 253] {
            h.add(k(v), 0, &PlainKeys);
        }
        assert_eq!(order(&h), vec![253, 22, 11, 0, 1]);
        assert!(h.resize_internal(23, &PlainKeys));
        // Pulled 253, 22, 11, 0, 1; re-added from 1 back to 253, each at its bucket's head,
        // so 253 and 0 (both bucket 0 of 23) keep their chain order.
        assert_eq!(order(&h), vec![253, 0, 1, 11, 22]);
        assert!(!h.resize_internal(23, &PlainKeys));
        assert!(h.grow(&PlainKeys));
        assert_eq!(h.num_buckets(), 47);
        assert!(h.shrink(&PlainKeys));
        assert_eq!(h.num_buckets(), 23);
        h.resize(30, &PlainKeys);
        assert_eq!(h.num_buckets(), 47);
        assert!(h.optimize_size(&PlainKeys));
        assert_eq!(h.num_buckets(), 11);
        assert_eq!(bucket_size_index(u32::MAX), 22);
    }

    #[test]
    fn iterators_walk_buckets_in_order_and_survive_head_inserts() {
        let mut h = ScriptHash::with_buckets(11);
        for v in [3, 14, 5] {
            h.add(k(v), 0, &PlainKeys);
        }
        let first = h.begin();
        assert_eq!(h.node(first).map(|n| n.key), Some(k(14)));
        h.add(k(25), 0, &PlainKeys);
        let second = h.next(first).unwrap();
        assert_eq!(h.node(second).map(|n| n.key), Some(k(3)));
        let third = h.remove_at(second).unwrap();
        assert_eq!(h.node(third).map(|n| n.key), Some(k(5)));
        let end = h.next(third).unwrap();
        assert_eq!(end, h.end());
        assert_eq!(h.next(end), None);
        assert_eq!(order(&h), vec![25, 14, 5]);
    }

    #[test]
    fn unpacking_keeps_packed_order_and_the_packed_native_buckets() {
        let items = vec![(k(0), 1), (k(11), 2), (k(3), 3)];
        let h = ScriptHash::from_packed(0, items);
        assert_eq!(order(&h), vec![0, 11, 3]);
        let native = RefKey::Ref {
            id: 9,
            cached_hash: 4,
            use_cache: 0,
        };
        let h = ScriptHash::from_packed(0, vec![(native, 1)]);
        assert_eq!(h.buckets[4].len(), 1);
    }
}
