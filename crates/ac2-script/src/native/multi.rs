//! The multi-containers at run time: `BaseMultiHash` over STLport's `hash_multimap`
//! (`_STL::hashtable`: prime bucket counts, `insert_equal_noresize` next to an equal key or at
//! the bucket head, `resize` relinking each chain head first) and `BaseMultiMap` over
//! `_STL::multimap` (sorted by key, a new key after the equal ones).
//!
//! Keys hash as `_STL::hash<unsigned long>` (the key) or `hash<unsigned __int64>` (its low
//! word). Elements carry an id that stands for retail's node pointer in iterators.

/// `_Stl_prime<bool>::_M_list` (Legions 0x009DA7E8); past its end STLport uses 0xFFFFFFFB.
pub const STL_PRIMES: [u32; 28] = [
    53, 97, 193, 389, 769, 1543, 3079, 6151, 12289, 24593, 49157, 98317, 196613, 393241, 786433,
    1572869, 3145739, 6291469, 12582917, 25165843, 50331653, 100663319, 201326611, 402653189,
    805306457, 1610612741, 3221225473, 4294967291,
];

/// `hashtable::_M_next_size`.
fn next_size(n: u32) -> u32 {
    STL_PRIMES
        .iter()
        .copied()
        .find(|&p| p >= n)
        .unwrap_or(0xFFFF_FFFB)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiNode {
    pub id: u32,
    pub key: u64,
    pub value: u64,
}

/// `BaseMultiHash`: `buckets[i]` is bucket i's chain, head first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptMultiHash {
    pub buckets: Vec<Vec<MultiNode>>,
    next_id: u32,
}

impl Default for ScriptMultiHash {
    /// `hash_multimap()`: `_M_initialize_buckets(100)`, 193 buckets.
    fn default() -> Self {
        Self {
            buckets: vec![Vec::new(); next_size(100) as usize],
            next_id: 0,
        }
    }
}

impl ScriptMultiHash {
    pub fn len(&self) -> u32 {
        self.buckets.iter().map(Vec::len).sum::<usize>() as u32
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn bucket(&self, key: u64) -> usize {
        ((key as u32) % self.buckets.len() as u32) as usize
    }

    pub fn nodes(&self) -> impl Iterator<Item = &MultiNode> {
        self.buckets.iter().flatten()
    }

    pub fn nodes_mut(&mut self) -> impl Iterator<Item = &mut MultiNode> {
        self.buckets.iter_mut().flatten()
    }

    /// `hashtable::resize(hint)`: a larger prime when `hint` exceeds the bucket count; each old
    /// chain is relinked head first onto the new chains' heads.
    fn resize(&mut self, hint: u32) {
        let old = self.buckets.len() as u32;
        if hint <= old {
            return;
        }
        let n = next_size(hint);
        if n <= old {
            return;
        }
        let mut fresh: Vec<Vec<MultiNode>> = vec![Vec::new(); n as usize];
        for chain in std::mem::take(&mut self.buckets) {
            for node in chain {
                let b = ((node.key as u32) % n) as usize;
                fresh[b].insert(0, node);
            }
        }
        self.buckets = fresh;
    }

    /// `resize(size + 1)` then `insert_equal_noresize`: after the first element with the same
    /// key, else at the bucket's head. Returns the new element's id.
    pub fn add(&mut self, key: u64, value: u64) -> u32 {
        self.resize(self.len() + 1);
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        let node = MultiNode { id, key, value };
        let b = self.bucket(key);
        match self.buckets[b].iter().position(|n| n.key == key) {
            Some(i) => self.buckets[b].insert(i + 1, node),
            None => self.buckets[b].insert(0, node),
        }
        id
    }

    /// `hashtable::erase(key)`: how many were removed.
    pub fn remove(&mut self, key: u64) -> u32 {
        let b = self.bucket(key);
        let before = self.buckets[b].len();
        self.buckets[b].retain(|n| n.key != key);
        (before - self.buckets[b].len()) as u32
    }

    /// `hashtable::erase(iterator)`.
    pub fn erase(&mut self, id: u32) -> bool {
        for chain in &mut self.buckets {
            if let Some(i) = chain.iter().position(|n| n.id == id) {
                chain.remove(i);
                return true;
            }
        }
        false
    }

    pub fn clear(&mut self) {
        for chain in &mut self.buckets {
            chain.clear();
        }
    }

    pub fn count(&self, key: u64) -> u32 {
        self.buckets[self.bucket(key)]
            .iter()
            .filter(|n| n.key == key)
            .count() as u32
    }

    /// `find`: the first element with `key` in its bucket.
    pub fn find(&self, key: u64) -> Option<u32> {
        self.buckets[self.bucket(key)]
            .iter()
            .find(|n| n.key == key)
            .map(|n| n.id)
    }

    fn first_from(&self, bucket: usize) -> Option<u32> {
        self.buckets
            .iter()
            .skip(bucket)
            .find_map(|c| c.first().map(|n| n.id))
    }

    pub fn begin(&self) -> Option<u32> {
        self.first_from(0)
    }

    /// `equal_range(key).second`: the element after the last one with `key` (the next one in
    /// the chain, else the first of a later bucket), or the end.
    pub fn equal_range_end(&self, key: u64) -> Option<u32> {
        let b = self.bucket(key);
        let chain = &self.buckets[b];
        let first = chain.iter().position(|n| n.key == key)?;
        match chain[first + 1..].iter().find(|n| n.key != key) {
            Some(n) => Some(n.id),
            None => self.first_from(b + 1),
        }
    }

    pub fn node(&self, id: u32) -> Option<&MultiNode> {
        self.nodes().find(|n| n.id == id)
    }

    /// `_Ht_iterator::operator++`: the next element in the chain, else the first of a later
    /// bucket. `None` when `id` is gone.
    pub fn next(&self, id: u32) -> Option<Option<u32>> {
        let node = self.node(id)?;
        let b = self.bucket(node.key);
        let chain = &self.buckets[b];
        let i = chain.iter().position(|n| n.id == id)?;
        Some(match chain.get(i + 1) {
            Some(n) => Some(n.id),
            None => self.first_from(b + 1),
        })
    }
}

/// `BaseMultiMap`: (key, id, value) sorted by key, equal keys in insertion order.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScriptMultiMap {
    pub entries: Vec<MultiNode>,
    next_id: u32,
}

impl ScriptMultiMap {
    fn lower(&self, key: u64) -> usize {
        self.entries.partition_point(|n| n.key < key)
    }

    fn upper(&self, key: u64) -> usize {
        self.entries.partition_point(|n| n.key <= key)
    }

    fn id_at(&self, i: usize) -> Option<u32> {
        self.entries.get(i).map(|n| n.id)
    }

    fn index(&self, id: u32) -> Option<usize> {
        self.entries.iter().position(|n| n.id == id)
    }

    /// `_Rb_tree::insert_equal`: after the elements with the same key.
    pub fn insert(&mut self, key: u64, value: u64) {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        let at = self.upper(key);
        self.entries.insert(at, MultiNode { id, key, value });
    }

    /// `_Rb_tree::erase(key)`.
    pub fn remove(&mut self, key: u64) {
        self.entries.retain(|n| n.key != key);
    }

    pub fn erase(&mut self, id: u32) -> bool {
        match self.index(id) {
            Some(i) => {
                self.entries.remove(i);
                true
            }
            None => false,
        }
    }

    pub fn count(&self, key: u64) -> u32 {
        (self.upper(key) - self.lower(key)) as u32
    }

    /// `_Rb_tree::find`: the first element with `key`, or the end.
    pub fn find(&self, key: u64) -> Option<u32> {
        let i = self.lower(key);
        self.entries.get(i).filter(|n| n.key == key).map(|n| n.id)
    }

    pub fn lower_bound(&self, key: u64) -> Option<u32> {
        self.id_at(self.lower(key))
    }

    pub fn upper_bound(&self, key: u64) -> Option<u32> {
        self.id_at(self.upper(key))
    }

    pub fn begin(&self) -> Option<u32> {
        self.id_at(0)
    }

    pub fn node(&self, id: u32) -> Option<&MultiNode> {
        self.entries.iter().find(|n| n.id == id)
    }

    /// `_Rb_tree_iterator::operator++` (from the end: the last element, as the header's right
    /// link is the rightmost node).
    pub fn next(&self, at: Option<u32>) -> Option<u32> {
        match at {
            Some(id) => self.index(id).and_then(|i| self.id_at(i + 1)),
            None => self.entries.last().map(|n| n.id),
        }
    }

    /// `_Rb_tree_iterator::operator--` (from the end: the last element; from the first: the
    /// end).
    pub fn prev(&self, at: Option<u32>) -> Option<u32> {
        match at {
            Some(id) => self
                .index(id)
                .and_then(|i| i.checked_sub(1))
                .and_then(|i| self.id_at(i)),
            None => self.entries.last().map(|n| n.id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multi_hash_inserts_next_to_equal_keys_and_relinks_on_resize() {
        let mut h = ScriptMultiHash::default();
        assert_eq!(h.buckets.len(), 193);
        h.add(5, 1);
        h.add(198, 2);
        h.add(5, 3);
        let order: Vec<_> = h.nodes().map(|n| (n.key, n.value)).collect();
        assert_eq!(order, vec![(198, 2), (5, 1), (5, 3)]);
        assert_eq!(h.count(5), 2);
        assert_eq!(h.equal_range_end(5), None);
        assert_eq!(h.remove(5), 2);
        for k in 0..200 {
            h.add(k, k);
        }
        assert_eq!(h.buckets.len(), 389);
    }

    #[test]
    fn multi_map_keeps_keys_sorted_and_equal_keys_in_order() {
        let mut m = ScriptMultiMap::default();
        m.insert(7, 1);
        m.insert(3, 2);
        m.insert(7, 3);
        let order: Vec<_> = m.entries.iter().map(|n| (n.key, n.value)).collect();
        assert_eq!(order, vec![(3, 2), (7, 1), (7, 3)]);
        assert_eq!(m.count(7), 2);
        let first7 = m.find(7).unwrap();
        assert_eq!(m.node(first7).unwrap().value, 1);
        assert_eq!(m.upper_bound(7), None);
        assert_eq!(m.prev(None), m.node(m.entries[2].id).map(|n| n.id));
        assert_eq!(m.prev(m.begin()), None);
    }
}
