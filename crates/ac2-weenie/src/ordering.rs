//! Event ordering: `TSRecv` (the player's ordered events, blocking mode) and
//! `VisualOrderingQueue` (each object's visual-ordered events).

use ac2_net::Blob;

/// `TimeStampUtils::lhs_newer` for 16-bit stamps.
pub(crate) fn newer16(lhs: u16, rhs: u16) -> bool {
    if lhs == rhs {
        return false;
    }
    let (diff, sign) = if lhs < rhs {
        (rhs - lhs, -1)
    } else {
        (lhs - rhs, 1)
    };
    (if diff > 0x7FFF { -sign } else { sign }) > 0
}

/// `TSRecv` in `TSRECV_BLOCK` mode: entries leave in stamp order; one that skips ahead waits
/// until the gap fills, or until more than 20 wait for more than 300 s.
#[derive(Debug, Default)]
pub(crate) struct TsRecv {
    received_first: bool,
    highest: u32,
    /// `head_` list: blocked entries in ascending stamp order.
    blocked: Vec<(u32, Blob)>,
    blocked_since: f64,
}

/// `TSRecv::AddAndCheck` result.
#[derive(Debug, PartialEq)]
pub(crate) enum TsResult {
    /// `1`: deliver now.
    Ready(Blob),
    /// `2`: held.
    Blocked,
    /// `3`: older than what was delivered.
    Rejected,
}

impl TsRecv {
    /// `TSRecv::AddEntryBlocking`.
    pub fn add_and_check(&mut self, stamp: u32, blob: Blob, cur_time: f64) -> TsResult {
        if !self.received_first {
            self.received_first = true;
        } else if stamp <= self.highest {
            return TsResult::Rejected;
        }
        if stamp == self.highest || stamp == self.highest.wrapping_add(1) {
            self.highest = stamp;
            return TsResult::Ready(blob);
        }
        let was_blocked = self.blocked.len();
        self.add_entry(stamp, blob);
        if was_blocked == 0 && self.blocked.len() == 1 {
            self.blocked_since = cur_time;
        }
        if self.blocked.len() > 0x13
            && self.blocked_since != 0.0
            && 300.0 < cur_time - self.blocked_since
        {
            // Skip the missing stamp: deliver what follows it.
            self.highest = self.highest.wrapping_add(1);
            if let Some(next) = self.next_ready(cur_time) {
                return TsResult::Ready(next);
            }
        }
        TsResult::Blocked
    }

    /// `TSRecv::AddEntry`: insert before the first entry not older than `stamp`.
    fn add_entry(&mut self, stamp: u32, blob: Blob) {
        let at = self
            .blocked
            .iter()
            .position(|(s, _)| *s == stamp || !crate::ordering::newer32(stamp, *s))
            .unwrap_or(self.blocked.len());
        self.blocked.insert(at, (stamp, blob));
    }

    /// `TSRecv::GetNextReadyEntry`.
    pub fn next_ready(&mut self, cur_time: f64) -> Option<Blob> {
        let &(stamp, _) = self.blocked.first()?;
        if stamp == self.highest || stamp == self.highest.wrapping_add(1) {
            let (stamp, blob) = self.blocked.remove(0);
            self.highest = stamp;
            if self.blocked.is_empty() {
                self.blocked_since = 0.0;
            }
            return Some(blob);
        }
        self.blocked_since = cur_time;
        None
    }
}

/// `TimeStampUtils::lhs_newer` for 32-bit stamps.
pub(crate) fn newer32(lhs: u32, rhs: u32) -> bool {
    if lhs == rhs {
        return false;
    }
    let (diff, sign) = if lhs < rhs {
        (rhs - lhs, -1)
    } else {
        (lhs - rhs, 1)
    };
    (if diff > 0x7FFF_FFFF { -sign } else { sign }) > 0
}

/// `VisualOrderingQueue`: a min-heap of (stamp, blob) under 16-bit wraparound order, released
/// one stamp at a time.
#[derive(Debug, Default)]
pub(crate) struct VisualOrderingQueue {
    heap: Vec<(u16, Blob)>,
    current: u16,
    /// `m_lastBlockingTime` (`cur_time`), `None` for `-1`.
    blocking_since: Option<f64>,
}

impl VisualOrderingQueue {
    /// `VisualOrderingQueue::HandleVisualOrderedEvent` (`priority_queue::push`).
    pub fn push(&mut self, stamp: u16, blob: Blob) {
        self.heap.push((stamp, blob));
        let hole = self.heap.len() - 1;
        self.push_heap(hole, 0);
    }

    /// `_STL::__push_heap` with `greater<StampedNetBlob>`.
    fn push_heap(&mut self, mut hole: usize, top: usize) {
        let stamp = self.heap[hole].0;
        while hole > top {
            let parent = (hole - 1) / 2;
            let p = self.heap[parent].0;
            if p == stamp || !newer16(p, stamp) {
                break;
            }
            self.heap.swap(parent, hole);
            hole = parent;
        }
    }

    /// `priority_queue::pop`: the last node replaces the root through `_STL::__adjust_heap`.
    fn pop(&mut self) -> Option<(u16, Blob)> {
        if self.heap.is_empty() {
            return None;
        }
        let last = self.heap.len() - 1;
        self.heap.swap(0, last);
        let top = self.heap.pop();
        let len = self.heap.len();
        if len > 1 {
            // `__adjust_heap(first, 0, len, val)`: move the hole down to a leaf along the
            // older child (the right one on ties), then push the value back up.
            let mut hole = 0;
            let mut second = 2;
            while second < len {
                let (left, right) = (self.heap[second - 1].0, self.heap[second].0);
                if right != left && newer16(right, left) {
                    second -= 1;
                }
                self.heap.swap(hole, second);
                hole = second;
                second = 2 * second + 2;
            }
            if second == len {
                self.heap.swap(hole, second - 1);
                hole = second - 1;
            }
            self.push_heap(hole, 0);
        }
        top
    }

    /// `VisualOrderingQueue::ExtractNextEvent`: the next stamp, dropping stale ones.
    pub fn extract(&mut self, cur_time: f64) -> Option<Blob> {
        loop {
            let Some(&(stamp, _)) = self.heap.first() else {
                self.blocking_since = None;
                return None;
            };
            let next = self.current.wrapping_add(1);
            if stamp == next {
                self.current = stamp;
                self.blocking_since = None;
                return self.pop().map(|(_, blob)| blob);
            }
            if newer16(stamp, next) {
                self.blocking_since.get_or_insert(cur_time);
                return None;
            }
            self.pop();
            if self.heap.is_empty() {
                self.blocking_since = None;
                return None;
            }
        }
    }

    /// `VisualOrderingQueue::DiscardEvents`: a create sets the current stamp forward.
    pub fn discard_to(&mut self, stamp: u16) {
        if newer16(stamp, self.current) {
            self.current = stamp;
        }
    }

    /// `VisualOrderingQueue::IsHeavilyBlocked`: blocked for more than 20 s.
    pub fn heavily_blocked(&self, cur_time: f64) -> bool {
        self.blocking_since.is_some_and(|t| 20.0 < cur_time - t)
    }

    /// `VisualOrderingQueue::DiscardAllEvents`.
    pub fn discard_all(&mut self) {
        self.heap.clear();
        self.blocking_since = None;
    }
}

#[cfg(test)]
mod tests {
    use super::{TsRecv, TsResult, VisualOrderingQueue};
    use ac2_net::Blob;

    fn blob(n: u8) -> Blob {
        Blob {
            id: u64::from(n),
            queue: 1,
            data: vec![n],
        }
    }

    #[test]
    fn ordered_events_wait_for_the_missing_stamp() {
        let mut ts = TsRecv::default();
        assert_eq!(ts.add_and_check(1, blob(1), 0.0), TsResult::Ready(blob(1)));
        assert_eq!(ts.add_and_check(3, blob(3), 0.0), TsResult::Blocked);
        assert_eq!(ts.add_and_check(1, blob(1), 0.0), TsResult::Rejected);
        assert_eq!(ts.add_and_check(2, blob(2), 0.0), TsResult::Ready(blob(2)));
        assert_eq!(ts.next_ready(0.0), Some(blob(3)));
        assert_eq!(ts.next_ready(0.0), None);
    }

    #[test]
    fn visual_events_release_in_stamp_order_and_drop_stale_ones() {
        let mut q = VisualOrderingQueue::default();
        q.push(3, blob(3));
        q.push(1, blob(1));
        q.push(2, blob(2));
        assert_eq!(q.extract(0.0), Some(blob(1)));
        assert_eq!(q.extract(0.0), Some(blob(2)));
        assert_eq!(q.extract(0.0), Some(blob(3)));
        q.push(2, blob(2));
        q.push(5, blob(5));
        assert_eq!(q.extract(1.0), None);
        assert!(!q.heavily_blocked(20.0) && q.heavily_blocked(21.5));
        q.discard_to(4);
        assert_eq!(q.extract(2.0), Some(blob(5)));
    }
}
