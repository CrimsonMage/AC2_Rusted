//! The double's blob reassembly (test tooling): the client's fragments are gathered per blob id
//! until every index arrived. A fragment that arrives twice is counted and ignored, so a test can
//! assert that the transport never delivers one twice.

use std::collections::BTreeMap;

use ac2_net::{Fragment, FRAGMENT_HEADER_LEN};

/// A `BlobFragHeader_t`: blob id, fragment count, size with this header, index, queue.
pub fn fragment_header_bytes(fragment: &Fragment) -> Vec<u8> {
    let mut out = fragment.blob_id.to_le_bytes().to_vec();
    out.extend(fragment.count.to_le_bytes());
    out.extend(((fragment.data.len() + FRAGMENT_HEADER_LEN) as u16).to_le_bytes());
    out.extend(fragment.index.to_le_bytes());
    out.extend(fragment.queue.to_le_bytes());
    out
}

#[derive(Debug, Default)]
struct Partial {
    queue: u16,
    count: u16,
    pieces: BTreeMap<u16, Vec<u8>>,
}

/// Blobs assembling from the client's fragments.
#[derive(Debug, Default)]
pub struct Assembler {
    partial: BTreeMap<u64, Partial>,
    /// Blob ids already completed.
    done: std::collections::BTreeSet<u64>,
    /// Fragments that arrived a second time (of a partial or a completed blob).
    pub duplicate_fragments: u32,
}

impl Assembler {
    /// Adds a fragment; returns the blob (id, queue, bytes) once its last piece arrived.
    pub fn add(&mut self, fragment: &Fragment) -> Option<(u64, u16, Vec<u8>)> {
        let id = fragment.blob_id;
        if self.done.contains(&id) {
            self.duplicate_fragments += 1;
            return None;
        }
        let partial = self.partial.entry(id).or_insert_with(|| Partial {
            queue: fragment.queue,
            count: fragment.count,
            pieces: BTreeMap::new(),
        });
        if partial.pieces.contains_key(&fragment.index) {
            self.duplicate_fragments += 1;
            return None;
        }
        partial.pieces.insert(fragment.index, fragment.data.clone());
        if partial.pieces.len() < usize::from(partial.count) {
            return None;
        }
        let partial = self.partial.remove(&id).expect("partial blob");
        self.done.insert(id);
        Some((
            id,
            partial.queue,
            partial.pieces.into_values().flatten().collect(),
        ))
    }
}
