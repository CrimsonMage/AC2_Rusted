//! Reading one record's (or B-tree node's) block chain.
//!
//! Retail `CLBlockAllocator::Load_Data` follows the chain with one `DiskDev::SyncRead` per block
//! (link header plus data). Here a deliberate deviation reads the chain in runs: one positioned
//! read covers the next few blocks in the file (at most [`MAX_COALESCED_READ`] bytes), and every
//! linked block that lies inside the bytes read, the next one or a short skip ahead, is taken
//! from them. Most chain links point at the next block, so a walk issues about half the reads
//! of the per-block path for about 12% more bytes. It is still an on-demand read of the
//! record's blocks (no mmap, no index, no record cache) and returns the same bytes as the
//! per-block path, which the tests check over the real DATs. The root node, read once when the
//! DAT opens (`read_block_chain_from_file`), keeps the per-block shape.

use std::cell::RefCell;

use super::{FileBtreeRuntime, ReadClass};
use crate::error::DatResult;

/// The most bytes one coalesced positioned read asks for (and so the most a thread's scratch
/// buffer holds).
pub(super) const MAX_COALESCED_READ: usize = 128 * 1024;

/// Blocks a record's first read covers. Many chains continue far from their first block, so
/// reads start small and double while the chain keeps running on in the file; a jump elsewhere
/// starts the window again. Over a walk's records this keeps the bytes read close to the
/// per-block path's while still merging most reads.
const FIRST_RUN_BLOCKS: usize = 4;

thread_local! {
    /// Per-thread landing buffer of a coalesced read, at most [`MAX_COALESCED_READ`] bytes (or one
    /// block where a block is larger).
    static SCRATCH: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

impl FileBtreeRuntime {
    /// The `len` packed bytes of the chain starting at block `start`, read run by run.
    pub(super) fn read_block_chain(
        &self,
        start: usize,
        len: usize,
        did: Option<u32>,
        context: &'static str,
        read_class: ReadClass,
    ) -> DatResult<Vec<u8>> {
        if len == 0 {
            return Ok(Vec::new());
        }
        if start == 0 {
            return Err(self.invalid_chain(start, did, context));
        }
        let data_size = self.header.block_data_size(&self.path)?;
        let block_size = data_size + 4;
        let logical_len = self.file_len.min(self.header.file_size.max(0) as usize);
        // Whole blocks one read may span (at least one).
        let max_blocks = (MAX_COALESCED_READ / block_size).max(1);
        let mut out = Vec::with_capacity(len);
        let mut current = start;
        let mut window = FIRST_RUN_BLOCKS;
        SCRATCH.with(|scratch| {
            let mut scratch = scratch.borrow_mut();
            while out.len() < len {
                let remaining = len - out.len();
                // The block at `current` must fit in the file, as the per-block read requires.
                let first = data_size.min(remaining) + 4;
                if current
                    .checked_add(first)
                    .is_none_or(|end| end > logical_len)
                {
                    return Err(self.invalid_chain(current, did, context));
                }
                // The blocks this run may cover: what the record still needs, the read cap, and
                // the whole blocks left in the file (the last needed block may be partial).
                let needed = remaining.div_ceil(data_size);
                let in_file = ((logical_len - current) / block_size).max(1);
                let blocks = needed.min(window).min(max_blocks).min(in_file);
                let span = if blocks == needed {
                    remaining + 4 * blocks
                } else {
                    blocks * block_size
                };
                // Exact growth keeps the buffer at the largest span read, never past the cap.
                let held = scratch.len();
                if held < span {
                    scratch.reserve_exact(span - held);
                }
                scratch.resize(span, 0);
                self.read_exact_into(current, &mut scratch[..span], did, context, read_class)?;
                let run_start = current;
                let mut at = 0;
                loop {
                    let next = i32::from_le_bytes(scratch[at..at + 4].try_into().unwrap());
                    let count = data_size.min(len - out.len());
                    out.extend_from_slice(&scratch[at + 4..at + 4 + count]);
                    if out.len() == len {
                        break;
                    }
                    if next < 0 {
                        return Err(self.invalid_chain(current, did, context));
                    }
                    current = next as usize;
                    if current == 0 {
                        // The chain ended before `len` bytes.
                        return Err(self.invalid_chain(current, did, context));
                    }
                    // Keep going inside this read while the linked block, as far as the record
                    // needs it, lies in the bytes already read: the next block, or the short
                    // forward skips the allocator leaves; anything else starts the next read.
                    let wanted = data_size.min(len - out.len()) + 4;
                    match current.checked_sub(run_start) {
                        Some(offset) if offset + wanted <= span => {
                            #[cfg(test)]
                            if offset != at + block_size {
                                tests::IN_READ_SKIPS
                                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            }
                            at = offset;
                        }
                        _ => break,
                    }
                }
                // The chain ran on to the end of this read: read more next time; it jumped
                // elsewhere: start small again.
                window = if at + block_size >= span {
                    (window * 2).min(max_blocks)
                } else {
                    FIRST_RUN_BLOCKS
                };
            }
            // A large block size can grow the buffer past the cap once; never keep more.
            if scratch.capacity() > MAX_COALESCED_READ {
                scratch.truncate(0);
                scratch.shrink_to(MAX_COALESCED_READ);
            }
            Ok(())
        })?;
        Ok(out)
    }

    /// `CLBlockAllocator::Load_Data`'s own shape, one positioned read per block, kept as the
    /// reference the coalesced path is checked against.
    #[cfg(test)]
    fn read_block_chain_per_block(
        &self,
        start: usize,
        len: usize,
        did: Option<u32>,
        context: &'static str,
        read_class: ReadClass,
    ) -> DatResult<Vec<u8>> {
        if len == 0 {
            return Ok(Vec::new());
        }
        if start == 0 {
            return Err(self.invalid_chain(start, did, context));
        }
        let block_bytes = self.header.block_data_size(&self.path)?;
        let logical_len = self.file_len.min(self.header.file_size.max(0) as usize);
        let mut current = start;
        let mut read = 0usize;
        let mut out = vec![0; len + 4];
        while current != 0 && read < len {
            let count = block_bytes.min(len - read);
            let block_len = count + 4;
            let block_end = current.checked_add(block_len).ok_or_else(|| {
                crate::error::DatError::InvalidBlockOffset {
                    path: self.path.clone(),
                    block_offset: current,
                    context,
                }
            })?;
            if block_end > logical_len {
                return Err(self.invalid_chain(current, did, context));
            }
            let output = &mut out[read..read + block_len];
            let preserved_tail: [u8; 4] = output[..4].try_into().unwrap();
            self.read_exact_into(current, output, did, context, read_class)?;
            let next = i32::from_le_bytes(output[..4].try_into().unwrap());
            output[..4].copy_from_slice(&preserved_tail);
            read += count;
            if read == len {
                break;
            }
            if next < 0 {
                return Err(self.invalid_chain(current, did, context));
            }
            current = next as usize;
        }
        if read == len {
            out.copy_within(4.., 0);
            out.truncate(len);
            Ok(out)
        } else {
            Err(self.invalid_chain(current, did, context))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;
    use crate::entry::DatFileKind;

    /// Links followed inside one read to a block other than the next one.
    pub(super) static IN_READ_SKIPS: AtomicUsize = AtomicUsize::new(0);

    /// Every `stride`-th record of each DAT, plus the first records needing more than one
    /// 128 KiB read, read to the same bytes (and the same error) both ways, whole and as a short
    /// prefix. Skipped without `AC2_DAT_DIR`.
    #[test]
    fn coalesced_chains_match_per_block_reads_on_the_real_dats() {
        let Some(root) = std::env::var_os("AC2_DAT_DIR") else {
            return;
        };
        let root = std::path::PathBuf::from(root);
        for kind in [DatFileKind::Portal, DatFileKind::Cell, DatFileKind::Local] {
            let runtime =
                FileBtreeRuntime::open(root.join(kind.default_file_name())).expect("DAT opens");
            let entries = runtime
                .collect_entries_in_range(0, u32::MAX)
                .expect("B-tree walks");
            assert!(!entries.is_empty(), "{kind:?} has records");
            let stride = (entries.len() / 4000).max(1);
            let (mut compared, mut multi_block, mut bytes) = (0usize, 0usize, 0usize);
            let data_size = runtime.header.block_data_size(&runtime.path).unwrap();
            let mut large = 0usize;
            for (index, entry) in entries.iter().enumerate() {
                let size = entry.size as usize;
                let is_large = size > MAX_COALESCED_READ && large < 200;
                if index % stride != 0 && !is_large {
                    continue;
                }
                large += usize::from(size > MAX_COALESCED_READ);
                let read = |coalesced: bool| {
                    let args = (
                        entry.offset.max(0) as usize,
                        size,
                        Some(entry.id),
                        "file payload",
                        ReadClass::Payload,
                    );
                    if coalesced {
                        runtime.read_block_chain(args.0, args.1, args.2, args.3, args.4)
                    } else {
                        runtime.read_block_chain_per_block(args.0, args.1, args.2, args.3, args.4)
                    }
                };
                match (read(true), read(false)) {
                    (Ok(a), Ok(b)) => assert!(a == b, "{kind:?} {:08X} bytes differ", entry.id),
                    (a, b) => assert_eq!(
                        format!("{a:?}"),
                        format!("{b:?}"),
                        "{kind:?} {:08X}",
                        entry.id
                    ),
                }
                // A short prefix (a range read) stops mid-chain the same way.
                let prefix = size.min(data_size + 7);
                assert_eq!(
                    format!(
                        "{:?}",
                        runtime.read_block_chain(
                            entry.offset.max(0) as usize,
                            prefix,
                            Some(entry.id),
                            "file payload",
                            ReadClass::Payload
                        )
                    ),
                    format!(
                        "{:?}",
                        runtime.read_block_chain_per_block(
                            entry.offset.max(0) as usize,
                            prefix,
                            Some(entry.id),
                            "file payload",
                            ReadClass::Payload
                        )
                    ),
                    "{kind:?} {:08X} prefix",
                    entry.id
                );
                compared += 1;
                multi_block += usize::from(size > data_size);
                bytes += size;
            }
            eprintln!("{kind:?}: {compared} records ({multi_block} multi-block, {bytes} bytes)");
            assert!(compared > 1000 && multi_block > 0, "{kind:?} coverage");
            SCRATCH.with(|scratch| assert!(scratch.borrow().capacity() <= MAX_COALESCED_READ));
        }
        // Some chains skip ahead inside one read, and those came out the same too.
        assert!(IN_READ_SKIPS.load(Ordering::Relaxed) > 0);
    }
}
