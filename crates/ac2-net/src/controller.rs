//! `PacketController` and `RecipientData`: one flow queue and reassembler per server
//! connection, the recipients due a flush (`haveNaks_`, a stack), and the half-second flow
//! timers that queue every recipient regularly.

use std::collections::BTreeMap;

use crate::blob::{Blob, Indicator};
use crate::flow::{Destination, FlowQueue, FlushContext};
use crate::optional::OptionalHeader;
use crate::packet::Fragment;
use crate::pqueue::PQueueArray;
use crate::receiver::{ConnectionState, Receiver};

/// `RecipientData`.
#[derive(Debug)]
pub(crate) struct Recipient {
    pub flow: FlowQueue,
    indicator: Indicator,
    /// `flushNum_`: acknowledged below this sequence, applied at the next flush.
    flush_num: u32,
    pub state: ConnectionState,
    on_queue: bool,
}

/// `PacketController`.
#[derive(Debug, Default)]
pub(crate) struct PacketController {
    pub recipients: BTreeMap<u16, Recipient>,
    /// `FlowControlTimers`: next flush time (`cur_time`) per recipient.
    timers: PQueueArray<f64, u16>,
    /// `haveNaks_`: recipients due a flush; the last queued is flushed first.
    queued: Vec<u16>,
}

impl PacketController {
    /// `PacketController::AddRecipient`: its flow timer starts now.
    pub fn add_recipient(&mut self, id: u16, net_id: u16, cur_time: f64, local_time: f64) -> bool {
        if self.recipients.contains_key(&id) {
            return false;
        }
        self.recipients.insert(
            id,
            Recipient {
                flow: FlowQueue::new(net_id, local_time),
                indicator: Indicator::default(),
                flush_num: 0,
                state: ConnectionState::Disconnected,
                on_queue: false,
            },
        );
        self.timers.insert(cur_time, id);
        true
    }

    /// `PacketController::DeleteRecipient`.
    pub fn delete_recipient(&mut self, id: u16) {
        self.recipients.remove(&id);
        let kept: Vec<(f64, u16)> = std::iter::from_fn(|| self.timers.remove_min())
            .filter(|&(_, r)| r != id)
            .collect();
        for (key, r) in kept {
            self.timers.insert(key, r);
        }
        self.queued.retain(|&r| r != id);
    }

    /// `PacketController::EnqueueRecip`.
    pub fn enqueue_recip(&mut self, id: u16) {
        if let Some(recipient) = self.recipients.get_mut(&id) {
            if !recipient.on_queue {
                recipient.on_queue = true;
                self.queued.push(id);
            }
        }
    }

    /// `PacketController::EnqueueOptionalHeader`.
    pub fn enqueue_header(&mut self, id: u16, header: OptionalHeader, local_time: f64) -> bool {
        let Some(recipient) = self.recipients.get_mut(&id) else {
            return false;
        };
        recipient.flow.enqueue_header(header, local_time);
        self.enqueue_recip(id);
        true
    }

    /// `PacketController::SendBlob` -> `RecipientData::EnqueueBlob` -> `ClientFlowQueue::Enqueue`:
    /// queued while the connection is no further than connected; `true` whenever the recipient
    /// exists.
    pub fn send_blob(
        &mut self,
        id: u16,
        blob_id: u64,
        queue: u16,
        data: Vec<u8>,
        priority: u32,
    ) -> bool {
        let Some(recipient) = self.recipients.get_mut(&id) else {
            return false;
        };
        let accept = recipient.state <= ConnectionState::Connected;
        if accept {
            recipient.flow.enqueue_blob(blob_id, queue, data, priority);
        }
        self.enqueue_recip(id);
        true
    }

    /// `RecipientData::ProcessNaks`: queue the resends; everything below the first NAK is
    /// acknowledged.
    pub fn process_naks(&mut self, id: u16, ids: &[u32]) {
        let Some(recipient) = self.recipients.get_mut(&id) else {
            return;
        };
        recipient.flow.enqueue_acks(ids);
        if let Some(&first) = ids.first() {
            if crate::blob::sequence_newer(first, recipient.flush_num) {
                recipient.flush_num = first;
            }
        }
        self.enqueue_recip(id);
    }

    /// `RecipientData::QueueFlush`.
    pub fn queue_flush(&mut self, id: u16, num: u32) {
        if let Some(recipient) = self.recipients.get_mut(&id) {
            if num != 0 && crate::blob::sequence_newer(num, recipient.flush_num) {
                recipient.flush_num = num;
            }
        }
    }

    /// `RecipientData::IndicateReceivedPacket` -> `Indicator::CheckInPacket`.
    pub fn indicate(&mut self, id: u16, fragments: &[Fragment], cur_time: f64) -> Vec<Blob> {
        self.recipients
            .get_mut(&id)
            .map(|r| r.indicator.check_in(fragments, cur_time))
            .unwrap_or_default()
    }

    /// `PacketController::CheckFlowControlTimers`: each due timer queues its recipient and moves
    /// forward to the first half-second step after now.
    fn check_flow_timers(&mut self, cur_time: f64) {
        while let Some(key) = self.timers.peek_key() {
            if cur_time < key {
                return;
            }
            let (key, id) = self.timers.remove_min().expect("due timer");
            self.enqueue_recip(id);
            // `_ftol2((now - key) * -2.0)` truncates toward zero.
            let steps = ((cur_time - key) * -2.0).trunc() as i32;
            self.timers.insert(f64::from(1 - steps) * 0.5 + key, id);
        }
    }

    /// `PacketController::UseTime`: flush queued recipients, newest first, until none remain.
    /// A recipient that queues itself again while flushing (a header its own flush added) is
    /// flushed again in the same pass. Each flush reads its connection's `ReceiverData`.
    pub fn use_time(
        &mut self,
        clock: Clock,
        receivers: &mut BTreeMap<u16, Receiver>,
        mut sink: impl FnMut(u16, Destination, Vec<u8>),
    ) {
        self.check_flow_timers(clock.cur_time);
        while let Some(id) = self.queued.pop() {
            if id == 0 {
                break;
            }
            let Some(recipient) = self.recipients.get_mut(&id) else {
                continue;
            };
            recipient.on_queue = false;
            let Some(receiver) = receivers.get_mut(&id) else {
                continue;
            };
            let disconnect_received = (receiver.state == ConnectionState::DisconnectReceived)
                .then(|| clock.local_time - receiver.state_changed);
            let mut ctx = FlushContext {
                local_time: clock.local_time,
                server_time: clock.server_time,
                cur_time: clock.cur_time,
                iteration: receiver.iteration,
                outgoing: &mut receiver.outgoing,
                disconnect_received,
            };
            let before = recipient.flow.enqueued();
            for (to, datagram) in recipient.flow.empty(&mut ctx) {
                sink(id, to, datagram);
            }
            if recipient.flush_num != 0 {
                recipient.flow.flush_sent(recipient.flush_num, false);
                recipient.flush_num = 0;
            }
            recipient.indicator.flush_timed_out(clock.cur_time);
            if recipient.flow.enqueued() != before {
                self.enqueue_recip(id);
            }
        }
    }
}

/// The three clocks a frame runs on: `Timer::local_time`, `Timer::cur_time`, and
/// `Timer::get_elapsed_time() + Timer::base_time` for time-sensitive payloads.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Clock {
    pub local_time: f64,
    pub cur_time: f64,
    pub server_time: f64,
}
