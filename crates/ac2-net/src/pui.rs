//! `Proto_UI`: the client's blob sends, their ids and the connection each goes to.

use super::ClientNet;
use crate::blob::{make_blob_id, next_non_ephemeral, ORDERING_ORDERED, ORDERING_UNORDERED};
use crate::flow::PROTO_UI_PRIORITY;

/// Event-type bits of the client's data-download events (`CCliDatEvent::CCliDatEvent`).
const DAT_EVENT_TYPE: u32 = 0x2300_0000;

impl ClientNet {
    /// `ClientNet::EnterWorld`: ordered stamps restart (`Proto_UI::SetOrderingStamp(0)`).
    pub fn enter_world(&mut self) {
        self.ordered = 0;
        self.in_world = true;
    }

    /// `PacketController::GetNonEphemeralID`: the current id, then advance.
    fn next_non_ephemeral(&mut self) -> u64 {
        let id = self.non_ephemeral;
        self.non_ephemeral = next_non_ephemeral(id);
        id
    }

    /// `Proto_UI::SendBuff`: queues 3 and 7 are ordered under `orderedNumber_`, the rest
    /// unordered under `unorderedNumber_`; `Proto_UI::SendBlob` routes queues 4, 5 and 8 to the
    /// logon connection and the rest to the world connection. Returns the blob id when queued.
    pub fn send_buff(&mut self, queue: u16, data: Vec<u8>) -> Option<u64> {
        let ordered = matches!(queue, 3 | 7);
        let stamp = if ordered {
            self.ordered = self.ordered.wrapping_add(1);
            self.ordered
        } else {
            self.unordered = self.unordered.wrapping_add(1);
            self.unordered
        };
        let event_type = if ordered {
            ORDERING_ORDERED
        } else {
            ORDERING_UNORDERED
        };
        let sequence = self.next_non_ephemeral();
        let id = make_blob_id(event_type, stamp, sequence);
        let sent = self.send_blob(id, queue, data);
        if !sent && ordered {
            self.ordered = self.ordered.wrapping_sub(1);
        }
        sent.then_some(id)
    }

    /// A data-download event (`CCliDatEvent`): unordered and out of world, stamped by its own
    /// counter, on queue 5.
    pub fn send_dat_event(&mut self, data: Vec<u8>) -> Option<u64> {
        let stamp = self.dat_stamp;
        self.dat_stamp = self.dat_stamp.wrapping_add(1);
        let sequence = self.next_non_ephemeral();
        let id = make_blob_id(DAT_EVENT_TYPE, stamp, sequence);
        self.send_blob(id, 5, data).then_some(id)
    }

    /// `ChatInterfaceProvider::SendData` -> `CChatEvent::CreateForSend`: `0x100D3`, the length,
    /// the chat client's bytes; a `CCliDatEvent` id (its own stamp counter) on queue 4.
    pub fn send_chat(&mut self, data: &[u8]) -> Option<u64> {
        let mut blob = crate::protocol::Opcode::CHAT.to_le_bytes().to_vec();
        blob.extend((data.len() as u32).to_le_bytes());
        blob.extend_from_slice(data);
        let stamp = self.dat_stamp;
        self.dat_stamp = self.dat_stamp.wrapping_add(1);
        let sequence = self.next_non_ephemeral();
        let id = make_blob_id(DAT_EVENT_TYPE, stamp, sequence);
        self.send_blob(id, 4, blob).then_some(id)
    }

    /// `Proto_UI::SendBlob` and `NetBlob::Send` (priority 5). Queues 6, 7 and 8 are
    /// encrypted in place with the session key first (`CryptEncrypt(hSessionKey_, 0, TRUE)`);
    /// without one they go in the clear.
    fn send_blob(&mut self, id: u64, queue: u16, mut data: Vec<u8>) -> bool {
        if self.world_rec == 0 {
            return false;
        }
        if (6..9).contains(&queue) {
            if let Some(key) = &self.session_key {
                key.encrypt(&mut data);
            }
        }
        let recipient = if matches!(queue, 4 | 5 | 8) {
            self.logon_rec
        } else {
            self.world_rec
        };
        self.controller
            .send_blob(recipient, id, queue, data, PROTO_UI_PRIORITY)
    }
}
