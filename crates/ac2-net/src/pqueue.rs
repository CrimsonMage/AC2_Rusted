//! `PQueueArray<K, void*>`: the binary min-heap the flow queues order headers and blobs with
//! (keyed by header mask or blob priority) and the packet controller orders recipient timers
//! with. Equal keys leave in heap order, not insertion order, so the heap is reproduced exactly.

/// `PQueueArray`: a 1-based binary min-heap stored in an array.
#[derive(Debug, Clone)]
pub(crate) struct PQueueArray<K, T> {
    nodes: Vec<(K, T)>,
}

impl<K: PartialOrd + Copy, T> Default for PQueueArray<K, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: PartialOrd + Copy, T> PQueueArray<K, T> {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// The minimum key, if any.
    pub fn peek_key(&self) -> Option<K> {
        self.nodes.first().map(|(key, _)| *key)
    }

    /// `PQueueArray::Insert`: append, then move up while the parent's key is greater.
    pub fn insert(&mut self, key: K, data: T) {
        self.nodes.push((key, data));
        let mut i = self.nodes.len();
        while i > 1 {
            let parent = i >> 1;
            if self.nodes[parent - 1].0 <= key {
                break;
            }
            self.nodes.swap(parent - 1, i - 1);
            i = parent;
        }
    }

    /// `PQueueArray::RemoveMin`: the root leaves, the last node takes its place, `_Heapify(1)`.
    pub fn remove_min(&mut self) -> Option<(K, T)> {
        if self.nodes.is_empty() {
            return None;
        }
        let last = self.nodes.len() - 1;
        self.nodes.swap(0, last);
        let min = self.nodes.pop();
        self.heapify(1);
        min
    }

    /// `PQueueArray::_Heapify`: sift `root` down, preferring the left child on ties.
    fn heapify(&mut self, mut root: usize) {
        let n = self.nodes.len();
        loop {
            let left = root * 2;
            let right = left + 1;
            let mut smallest = root;
            if left <= n && self.nodes[left - 1].0 < self.nodes[root - 1].0 {
                smallest = left;
            }
            if right <= n && self.nodes[right - 1].0 < self.nodes[smallest - 1].0 {
                smallest = right;
            }
            if smallest == root {
                return;
            }
            self.nodes.swap(root - 1, smallest - 1);
            root = smallest;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PQueueArray;

    #[test]
    fn equal_keys_leave_in_heap_order() {
        let mut queue = PQueueArray::new();
        for (key, tag) in [(5u32, 'a'), (5, 'b'), (5, 'c'), (1, 'd'), (5, 'e')] {
            queue.insert(key, tag);
        }
        let order: Vec<char> = std::iter::from_fn(|| queue.remove_min().map(|(_, t)| t)).collect();
        // Heap after inserts: d(1) a b c e; removing d moves e to the root, and so on.
        assert_eq!(order, ['d', 'e', 'b', 'c', 'a']);
    }

    #[test]
    fn keys_come_out_ascending() {
        let mut queue = PQueueArray::new();
        for key in [9.5f64, 0.5, 3.0, 7.25, 0.25, 4.0] {
            queue.insert(key, ());
        }
        let keys: Vec<f64> = std::iter::from_fn(|| queue.remove_min().map(|(k, _)| k)).collect();
        assert_eq!(keys, [0.25, 0.5, 3.0, 4.0, 7.25, 9.5]);
    }
}
