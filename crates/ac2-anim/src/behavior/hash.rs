//! The animsys `PHashTable<unsigned long, T>`s as their `IntrusiveHashTable` lays them out:
//! the bucket count is the first entry of `g_bucketSizes` not below the requested 0x20 (47),
//! a key hashes to `key % 47`, `add` links a new node at the head of its chain, `set` replaces
//! the data of an existing key in place, and iteration walks the buckets in order and each chain
//! from its head. Retail iterates behaviour tracks in this order, so callbacks and blends do too.

/// `lower_bound(g_bucketSizes, 0x20)`: every animsys table asks for 0x20 buckets.
const BUCKETS: usize = 47;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RetailHash<V> {
    buckets: Vec<Vec<(u32, V)>>,
}

impl<V> Default for RetailHash<V> {
    fn default() -> Self {
        Self {
            buckets: (0..BUCKETS).map(|_| Vec::new()).collect(),
        }
    }
}

impl<V> RetailHash<V> {
    fn chain(&self, key: u32) -> &Vec<(u32, V)> {
        &self.buckets[key as usize % BUCKETS]
    }

    fn chain_mut(&mut self, key: u32) -> &mut Vec<(u32, V)> {
        &mut self.buckets[key as usize % BUCKETS]
    }

    pub(crate) fn get(&self, key: u32) -> Option<&V> {
        self.chain(key)
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| v)
    }

    pub(crate) fn get_mut(&mut self, key: u32) -> Option<&mut V> {
        self.chain_mut(key)
            .iter_mut()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| v)
    }

    pub(crate) fn contains(&self, key: u32) -> bool {
        self.get(key).is_some()
    }

    /// `HashTable::add`: links at the chain head; refused (false) when the key exists.
    pub(crate) fn add(&mut self, key: u32, value: V) -> bool {
        if self.contains(key) {
            return false;
        }
        self.chain_mut(key).insert(0, (key, value));
        true
    }

    /// `HashTable::set`: replaces in place, else adds at the chain head.
    pub(crate) fn set(&mut self, key: u32, value: V) {
        match self.get_mut(key) {
            Some(slot) => *slot = value,
            None => self.chain_mut(key).insert(0, (key, value)),
        }
    }

    /// `HashTable::remove` / `erase`.
    pub(crate) fn remove(&mut self, key: u32) -> Option<V> {
        let chain = self.chain_mut(key);
        let index = chain.iter().position(|(k, _)| *k == key)?;
        Some(chain.remove(index).1)
    }

    /// Keys in retail iteration order.
    pub(crate) fn keys(&self) -> Vec<u32> {
        self.iter().map(|(k, _)| k).collect()
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (u32, &V)> {
        self.buckets
            .iter()
            .flat_map(|chain| chain.iter().map(|(k, v)| (*k, v)))
    }
}
