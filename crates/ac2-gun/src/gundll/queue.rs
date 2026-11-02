//! GunDll's message queue (`GUNQueue`, vtable 0x10060850; `Write` `FUN_10039c00`, `Read`
//! `FUN_10039d10`, copy/keep rules `FUN_100398b0`), `0a_receive_messages.md` 1.4 (verified): FIFO,
//! except an entry written with `flags` bit 0 goes to the FRONT; `Read` removes the head, hands it to
//! the sink and answers 1 when the sink's result is negative, 0 otherwise; an empty queue answers
//! `E_PENDING` (0x8000000A).
//!
//! GunDll guards both with a spin lock; here the queue sits in a `RefCell` and [`read`] pops with the
//! borrow released before the sink runs, since the sink re-enters the object.

use std::cell::RefCell;
use std::collections::VecDeque;

use crate::Hresult;

/// `flags` bit 0: insert at the front (`FUN_10039c00`: `test [flags], 1`).
pub const QUEUE_FRONT: u32 = 1;

/// A queue of `T` with the `GUNQueueMessage` `flags`.
#[derive(Debug)]
pub struct Queue<T> {
    items: VecDeque<T>,
}

impl<T> Default for Queue<T> {
    fn default() -> Self {
        Queue {
            items: VecDeque::new(),
        }
    }
}

impl<T> Queue<T> {
    /// `Write` (vtable +0xc).
    pub fn write(&mut self, flags: u32, item: T) -> Hresult {
        if flags & QUEUE_FRONT != 0 {
            self.items.push_front(item);
        } else {
            self.items.push_back(item);
        }
        Hresult::S_OK
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// `Read(sink)` (vtable +0x10): `E_PENDING` for an empty queue; else the head goes to `sink` and the
/// result is 1 if the sink's result is below zero, 0 otherwise.
pub fn read<T>(queue: &RefCell<Queue<T>>, sink: impl FnOnce(T) -> Hresult) -> Hresult {
    let head = queue.borrow_mut().items.pop_front();
    match head {
        None => Hresult::E_PENDING,
        Some(item) => {
            if sink(item).succeeded() {
                Hresult::S_OK
            } else {
                Hresult(1)
            }
        }
    }
}
