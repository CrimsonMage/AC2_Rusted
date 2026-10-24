//! Blobs: ids (`NetBlobIDUtils`), fragmentation (`NetBlob::Fragmentize`) and reassembly
//! (`Indicator::CheckInPacket`, `NetBlob::ReceiveAddFragment`).
//!
//! A blob id's high word holds the ephemeral (bit 31), cell (30) and out-of-world (29) flags,
//! the ordering type (bits 24..28: 1 ordered, 2 visual ordered, 3 unordered), the server id
//! (bits 16..23) and a 16-bit ordering stamp; the low word is the non-ephemeral sequence.

use std::collections::{BTreeMap, HashMap};

use crate::packet::Fragment;

/// Fragment payload size (`NetBlob::Fragmentize`, `0x1c0`).
pub const FRAGMENT_DATA: usize = 0x1C0;

pub const ORDERING_ORDERED: u32 = 0x0100_0000;
pub const ORDERING_VISUAL: u32 = 0x0200_0000;
pub const ORDERING_UNORDERED: u32 = 0x0300_0000;

/// `NetBlobIDUtils::IsEphemeralFlagSet`.
pub fn is_ephemeral(id: u64) -> bool {
    id & 0x8000_0000_0000_0000 != 0
}

/// `NetBlobIDUtils::IsCellFlagSet`.
pub fn is_cell(id: u64) -> bool {
    id & 0x4000_0000_0000_0000 != 0
}

/// `NetBlobIDUtils::IsOutOfWorldFlagSet`.
pub fn is_out_of_world(id: u64) -> bool {
    id & 0x2000_0000_0000_0000 != 0
}

/// `NetBlobIDUtils::GetOrderingType`, as the high-word bits `0x1f000000`.
pub fn ordering_type(id: u64) -> u32 {
    (id >> 32) as u32 & 0x1F00_0000
}

/// `NetBlobIDUtils::GetOrderingStamp`.
pub fn ordering_stamp(id: u64) -> u16 {
    (id >> 32) as u16
}

/// `NetBlobIDUtils::GetSequenceID`: the server bits and the low word.
pub fn sequence_id(id: u64) -> u64 {
    id & 0x00FF_0000_FFFF_FFFF
}

/// `NetBlobIDUtils::MakeNetBlobID`: event-type high byte, ordering stamp, and the server bits
/// and low word of a non-ephemeral sequence.
pub fn make_blob_id(event_type: u32, stamp: u16, sequence: u64) -> u64 {
    let high =
        (event_type & 0xFF00_0000) | (((sequence >> 32) as u32) & 0x00FF_0000) | u32::from(stamp);
    (u64::from(high) << 32) | (sequence & 0xFFFF_FFFF)
}

/// `NetBlobIDUtils::NextNonEphemeralSequenceID`.
pub fn next_non_ephemeral(sequence: u64) -> u64 {
    let next = sequence.wrapping_add(1);
    let high = ((next >> 32) as u32) & 0x00FF_0000;
    (u64::from(high) << 32) | (next & 0xFFFF_FFFF)
}

/// `NetBlobIDUtils::LHSNewerOrderingStamp`: 16-bit stamps compared across wraparound.
pub fn newer_stamp(lhs: u64, rhs: u64) -> bool {
    stamp_newer(ordering_stamp(lhs), ordering_stamp(rhs))
}

/// `TimeStampUtils::lhs_newer` for 16-bit stamps.
pub fn stamp_newer(lhs: u16, rhs: u16) -> bool {
    if lhs == rhs {
        return false;
    }
    let (diff, sign) = if lhs < rhs {
        (rhs - lhs, -1)
    } else {
        (lhs - rhs, 1)
    };
    let sign = if diff > 0x7FFF { -sign } else { sign };
    sign > 0
}

/// `TimeStampUtils::lhs_newer` for 32-bit sequence ids.
pub fn sequence_newer(lhs: u32, rhs: u32) -> bool {
    if lhs == rhs {
        return false;
    }
    let (diff, sign) = if lhs < rhs {
        (rhs - lhs, -1)
    } else {
        (lhs - rhs, 1)
    };
    let sign = if diff > 0x7FFF_FFFF { -sign } else { sign };
    sign > 0
}

/// `NetBlob::Fragmentize`: `0x1c0`-byte pieces numbered from 0.
pub fn fragmentize(id: u64, queue: u16, data: &[u8]) -> Vec<Fragment> {
    let count = data.len().div_ceil(FRAGMENT_DATA) as u16;
    data.chunks(FRAGMENT_DATA)
        .enumerate()
        .map(|(index, part)| Fragment {
            blob_id: id,
            count,
            index: index as u16,
            queue,
            data: part.to_vec(),
        })
        .collect()
}

/// A completed blob: its id, the queue it arrived on and its bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Blob {
    pub id: u64,
    pub queue: u16,
    pub data: Vec<u8>,
}

/// `NetBlob` on the receive side.
#[derive(Debug)]
struct Assembly {
    id: u64,
    queue: u16,
    max_fragments: usize,
    received: usize,
    buf: Vec<u8>,
}

impl Assembly {
    fn new(fragment: &Fragment) -> Self {
        Self {
            id: fragment.blob_id,
            queue: fragment.queue,
            max_fragments: 0,
            received: 0,
            buf: Vec::new(),
        }
    }

    /// `NetBlob::ReceiveAddFragment`: pieces land at `index * 0x1c0`; one that disagrees with
    /// the blob or overruns it is ignored. The last piece fixes the size.
    fn add(&mut self, fragment: &Fragment) {
        let index = usize::from(fragment.index);
        if self.received == 0 {
            // Until a piece is accepted, each arrival sizes the buffer: a whole number of
            // pieces, or the exact size when it is the last piece.
            let count = usize::from(fragment.count);
            let size = if index + 1 == count {
                (count - 1) * FRAGMENT_DATA + fragment.data.len()
            } else {
                count * FRAGMENT_DATA
            };
            self.id = fragment.blob_id;
            self.queue = fragment.queue;
            self.max_fragments = count;
            self.buf = vec![0; size];
        }
        let len = fragment.data.len();
        let at = index * FRAGMENT_DATA;
        if usize::from(fragment.count) != self.max_fragments
            || fragment.queue != self.queue
            || index >= self.max_fragments
            || len > FRAGMENT_DATA
            || at + len > self.buf.len()
        {
            return;
        }
        self.buf[at..at + len].copy_from_slice(&fragment.data);
        self.received += 1;
        if self.max_fragments > 1 && self.max_fragments == index + 1 {
            self.buf
                .truncate((self.max_fragments - 1) * FRAGMENT_DATA + len);
        }
    }
}

/// `Indicator`: blobs assembling from one sender, and the newest id of each ephemeral sequence.
#[derive(Debug, Default)]
pub(crate) struct Indicator {
    waiting: HashMap<u64, Assembly>,
    /// `ArrivedEphInfo`: ephemeral sequence -> (latest id, `cur_time` it arrived).
    ephemeral: BTreeMap<u64, (u64, f64)>,
    flush_stamp: f64,
}

impl Indicator {
    /// `Indicator::CheckInPacket`: each fragment not made obsolete by a newer ephemeral blob is
    /// accepted; completed blobs come back in completion order.
    pub fn check_in(&mut self, fragments: &[Fragment], cur_time: f64) -> Vec<Blob> {
        let mut out = Vec::new();
        for fragment in fragments {
            if !self.obsolete_ephemeral(fragment.blob_id, cur_time) {
                if let Some(blob) = self.accept(fragment) {
                    out.push(blob);
                }
            }
        }
        out
    }

    /// `Indicator::FragIsObsoleteEmphemeral`.
    fn obsolete_ephemeral(&mut self, id: u64, cur_time: f64) -> bool {
        if !is_ephemeral(id) {
            return false;
        }
        match self.ephemeral.get_mut(&sequence_id(id)) {
            None => {
                self.ephemeral.insert(sequence_id(id), (id, cur_time));
                false
            }
            Some((latest, stamp)) => {
                if id != *latest {
                    if newer_stamp(*latest, id) {
                        return true;
                    }
                    *latest = id;
                    *stamp = cur_time;
                }
                false
            }
        }
    }

    /// `Indicator::AcceptFrag`: ephemeral blobs assemble under their sequence, so a newer stamp
    /// replaces an older partial blob.
    fn accept(&mut self, fragment: &Fragment) -> Option<Blob> {
        let id = fragment.blob_id;
        let key = if is_ephemeral(id) {
            sequence_id(id)
        } else {
            id
        };
        if let Some(existing) = self.waiting.get(&key) {
            if existing.id != id {
                if !newer_stamp(existing.id, id) {
                    return None;
                }
                self.waiting.remove(&key);
            }
        }
        let assembly = self
            .waiting
            .entry(key)
            .or_insert_with(|| Assembly::new(fragment));
        assembly.add(fragment);
        if assembly.received == assembly.max_fragments {
            let done = self.waiting.remove(&key).expect("assembly present");
            return Some(Blob {
                id: done.id,
                queue: done.queue,
                data: done.buf,
            });
        }
        None
    }

    /// `Indicator::FlushTimedOutEphInfo`: every 5 s, forget ephemeral sequences idle for 5 s.
    pub fn flush_timed_out(&mut self, cur_time: f64) {
        if 5.0 <= cur_time - self.flush_stamp {
            self.flush_stamp = cur_time;
            self.ephemeral
                .retain(|_, (_, stamp)| !(5.0 < cur_time - *stamp));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        fragmentize, make_blob_id, next_non_ephemeral, sequence_newer, stamp_newer, Indicator,
        ORDERING_UNORDERED,
    };

    #[test]
    fn blob_ids_pack_like_netblobidutils() {
        let id = make_blob_id(ORDERING_UNORDERED, 7, 0x0012_0000_0000_0005);
        assert_eq!(id, 0x0312_0007_0000_0005);
        assert_eq!(make_blob_id(0x2300_0000, 0, 1), 0x2300_0000_0000_0001);
        assert_eq!(next_non_ephemeral(0), 1);
        assert_eq!(next_non_ephemeral(0x0000_0000_FFFF_FFFF), 0);
        assert!(stamp_newer(1, 0xFFFF) && !stamp_newer(0xFFFF, 1));
        assert!(sequence_newer(2, 1) && !sequence_newer(1, 1));
        assert!(sequence_newer(1, 0xFFFF_FFF0));
    }

    #[test]
    fn fragments_place_at_their_index_and_complete_in_any_order() {
        let data: Vec<u8> = (0..1000u32).map(|v| v as u8).collect();
        let pieces = fragmentize(0x0300_0001_0000_0000, 1, &data);
        assert_eq!(pieces.len(), 3);
        assert_eq!(
            pieces.iter().map(|p| p.data.len()).collect::<Vec<_>>(),
            [448, 448, 104]
        );
        let mut indicator = Indicator::default();
        assert!(indicator.check_in(&pieces[2..], 0.0).is_empty());
        assert!(indicator.check_in(&pieces[..1], 0.0).is_empty());
        let blobs = indicator.check_in(&pieces[1..2], 0.0);
        assert_eq!(blobs.len(), 1);
        assert_eq!(blobs[0].data, data);
    }

    #[test]
    fn a_newer_ephemeral_blob_obsoletes_the_older() {
        let older = 0x8300_0001_0000_0009u64;
        let newer = 0x8300_0002_0000_0009u64;
        let mut indicator = Indicator::default();
        assert_eq!(
            indicator
                .check_in(&fragmentize(newer, 1, b"new"), 0.0)
                .len(),
            1
        );
        assert!(indicator
            .check_in(&fragmentize(older, 1, b"old"), 0.0)
            .is_empty());
    }
}
