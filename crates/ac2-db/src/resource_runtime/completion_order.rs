//! Restores request order for resource requests decoded concurrently.
//!
//! Retail's single Worker 2 published completions in request order. Several workers may now
//! decode resource requests at once; each request takes a ticket when a worker claims it, and a
//! finished completion is released only after every earlier ticket, so consumers keep observing
//! FIFO completion order.

use std::collections::BTreeMap;

#[derive(Debug)]
pub(super) struct CompletionOrder<T> {
    next_ticket: u64,
    next_release: u64,
    finished: BTreeMap<u64, T>,
}

impl<T> Default for CompletionOrder<T> {
    fn default() -> Self {
        Self {
            next_ticket: 0,
            next_release: 0,
            finished: BTreeMap::new(),
        }
    }
}

impl<T> CompletionOrder<T> {
    /// The ticket for the next claimed request, in claim (FIFO) order.
    pub(super) fn issue(&mut self) -> u64 {
        let ticket = self.next_ticket;
        self.next_ticket += 1;
        ticket
    }

    /// Records `ticket` as finished and releases every completion that is now in order.
    pub(super) fn finish(&mut self, ticket: u64, value: T, mut release: impl FnMut(T)) {
        self.finished.insert(ticket, value);
        while let Some(value) = self.finished.remove(&self.next_release) {
            self.next_release += 1;
            release(value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CompletionOrder;

    #[test]
    fn out_of_order_finishes_release_in_ticket_order() {
        let mut order = CompletionOrder::default();
        let tickets: Vec<u64> = (0..4).map(|_| order.issue()).collect();
        let mut released = Vec::new();
        for ticket in [2, 0, 3, 1] {
            order.finish(tickets[ticket], ticket, |value| released.push(value));
            match ticket {
                2 => assert!(released.is_empty(), "ticket 0 is still running"),
                0 => assert_eq!(released, [0]),
                3 => assert_eq!(released, [0], "ticket 1 is still running"),
                _ => assert_eq!(released, [0, 1, 2, 3]),
            }
        }
    }

    #[test]
    fn in_order_finishes_release_immediately() {
        let mut order = CompletionOrder::default();
        let mut released = Vec::new();
        for value in 0..3 {
            let ticket = order.issue();
            order.finish(ticket, value, |value| released.push(value));
            assert_eq!(released.len(), value + 1);
        }
    }
}
