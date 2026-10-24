//! `CObjectMaint`'s destruction queues (`PriorityHash<InstanceID, double>`):
//! `m_objectDestructionQueue` holds objects that left the client's view, due 25 s later
//! (`AddObjectToBeDestroyed`; cancelled when they come back, `RemoveObjectToBeDestroyed`);
//! `m_pendingDeletions` holds objects to delete at the next `CObjectMaint::UseTime`
//! (`DestroyObject`, and the destruction queue's due entries), which `HandlePendingDeletion`
//! removes (`DestroyObjectInternal`).

use std::collections::BTreeMap;

/// `CObjectMaint::AddObjectToBeDestroyed`: how long an out-of-view object is kept (seconds; a
/// temporary object, which the server never creates, would get 0.25).
pub(crate) const OUT_OF_VIEW_LIFETIME: f64 = 25.0;

/// One `PriorityHash<InstanceID, double>`: each id once, taken in order of its time.
#[derive(Debug, Default)]
pub(crate) struct PriorityQueue {
    times: BTreeMap<u64, f64>,
}

impl PriorityQueue {
    /// `PriorityHash::insert`: a second insert of the same id replaces its time.
    pub(crate) fn insert(&mut self, id: u64, time: f64) {
        self.times.insert(id, time);
    }

    pub(crate) fn remove(&mut self, id: u64) {
        self.times.remove(&id);
    }

    /// `remove_min` while the earliest entry is due (`priority <= cur_time`).
    pub(crate) fn take_due(&mut self, cur_time: f64) -> Option<u64> {
        let (&id, _) = self
            .times
            .iter()
            .filter(|(_, &time)| time <= cur_time)
            .min_by(|a, b| a.1.total_cmp(b.1))?;
        self.times.remove(&id);
        Some(id)
    }

    pub(crate) fn clear(&mut self) {
        self.times.clear();
    }

    pub(crate) fn contains(&self, id: u64) -> bool {
        self.times.contains_key(&id)
    }
}

/// Both queues.
#[derive(Debug, Default)]
pub(crate) struct Destruction {
    pub(crate) to_destroy: PriorityQueue,
    pub(crate) pending: PriorityQueue,
}

impl Destruction {
    pub(crate) fn clear(&mut self) {
        self.to_destroy.clear();
        self.pending.clear();
    }

    /// `DestroyObjectInternal`'s queue part: the object leaves both.
    pub(crate) fn forget(&mut self, id: u64) {
        self.to_destroy.remove(id);
        self.pending.remove(id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn due_entries_come_out_earliest_first() {
        let mut queue = PriorityQueue::default();
        queue.insert(1, 30.0);
        queue.insert(2, 10.0);
        queue.insert(3, 20.0);
        assert_eq!(queue.take_due(5.0), None);
        assert_eq!(queue.take_due(25.0), Some(2));
        assert_eq!(queue.take_due(25.0), Some(3));
        assert_eq!(queue.take_due(25.0), None);
        // A due time equal to now counts.
        assert_eq!(queue.take_due(30.0), Some(1));
    }
}
