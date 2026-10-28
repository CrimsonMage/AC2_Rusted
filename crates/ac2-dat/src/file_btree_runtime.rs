use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::btree::{DatBTreeNode, DatHeader, DAT_HEADER_SIZE};
use crate::btree_profile::{DatBtreeProfile, DatBtreeProfileSnapshot};
use crate::entry::DatRawFileEntry;
use crate::error::{DatError, DatResult};

mod chain;

/// Retail keeps the root separately plus 100 timestamp-replaceable `BTMemNode`s.
const REPLACEABLE_NODE_CACHE_LIMIT: usize = 100;

/// Retail-shaped DAT read owner: one shared file handle, positioned reads, and a bounded parsed
/// B-tree-node cache. It deliberately owns neither decoded objects nor raw record-byte caching.
#[derive(Debug)]
pub(crate) struct FileBtreeRuntime {
    file: File,
    path: PathBuf,
    header: DatHeader,
    file_len: usize,
    root: Option<Arc<DatBTreeNode>>,
    /// The `CThreadsafeDiskController::m_DiskLock` lookup section: DID lookup and B-tree node
    /// loading for one request run under it. Payload blocks are read afterwards with positioned
    /// reads on the shared read-only handle, so concurrent loaders overlap their disk reads.
    read_path: Mutex<()>,
    nodes: Mutex<NodeLru>,
    profile: DatBtreeProfile,
}

#[derive(Debug, Default)]
struct NodeLru {
    entries: HashMap<usize, Arc<DatBTreeNode>>,
    order: VecDeque<usize>,
}

#[derive(Debug, Clone, Copy)]
enum ReadClass {
    BtreeNode,
    Payload,
}

impl FileBtreeRuntime {
    pub(crate) fn open(path: impl AsRef<Path>) -> DatResult<Self> {
        let path = path.as_ref().to_path_buf();
        let file = File::open(&path)?;
        let file_len = usize::try_from(file.metadata()?.len())
            .map_err(|_| DatError::Io(io::Error::other("DAT file length does not fit usize")))?;
        let mut header_bytes = [0u8; DAT_HEADER_SIZE];
        read_exact_at(&file, &mut header_bytes, 0)?;
        let header = DatHeader::parse(&header_bytes, &path)?;
        let profile = DatBtreeProfile::default();
        let root = if header.root_block > 0 {
            let root_offset = header.root_block as usize;
            let root_bytes = read_block_chain_from_file(
                &file,
                &path,
                header,
                file_len,
                root_offset,
                header.layout.node_size(),
            )?;
            profile.record_node_load();
            record_root_positioned_reads(
                &profile,
                header.layout.node_size(),
                header.block_data_size(&path)?,
            );
            Some(Arc::new(DatBTreeNode::parse(
                &root_bytes,
                &path,
                root_offset,
                header.layout,
            )?))
        } else {
            None
        };
        Ok(Self {
            file,
            path,
            header,
            file_len,
            root,
            read_path: Mutex::new(()),
            nodes: Mutex::new(NodeLru::default()),
            profile,
        })
    }

    pub(crate) fn header(&self) -> DatHeader {
        self.header
    }

    pub(crate) fn find_file_entry(&self, did: u32) -> DatResult<Option<DatRawFileEntry>> {
        let _read_path = self.read_path.lock().expect("DAT read-path lock poisoned");
        self.find_file_entry_locked(did)
    }

    /// The `DiskController::LoadDataEx` equivalent. The lookup keeps the DAT critical section,
    /// so concurrent first readers never independently load the same parsed B-tree node; the
    /// payload chain is read after it is released. The DAT is opened read-only and never
    /// written, so a found entry's blocks cannot change underneath the read.
    pub(crate) fn read_record(&self, did: u32) -> DatResult<Option<(DatRawFileEntry, Vec<u8>)>> {
        let Some(entry) = self.find_file_entry(did)? else {
            return Ok(None);
        };
        let bytes = self.read_payload(entry)?;
        Ok(Some((entry, bytes)))
    }

    /// Same shape as [`Self::read_record`]: locked lookup, then an unlocked read of the range's
    /// backing payload chain.
    pub(crate) fn read_payload_range_for_did(
        &self,
        did: u32,
        offset: u64,
        len: usize,
    ) -> DatResult<Option<Vec<u8>>> {
        let Some(entry) = self.find_file_entry(did)? else {
            return Ok(None);
        };
        let offset = usize::try_from(offset).map_err(|_| DatError::RangeOutOfBounds {
            did,
            offset,
            len,
            packed_len: entry.size,
        })?;
        self.read_payload_range(entry, offset, len).map(Some)
    }

    fn find_file_entry_locked(&self, did: u32) -> DatResult<Option<DatRawFileEntry>> {
        if self.header.root_block <= 0 {
            return Ok(None);
        }
        self.profile.record_lookup();
        match self.find_in_node(self.header.root_block as usize, did) {
            Ok(Some(entry)) => {
                self.profile.record_lookup_found();
                Ok(Some(entry))
            }
            Ok(None) => {
                self.profile.record_lookup_missing();
                Ok(None)
            }
            Err(error) => {
                self.profile.record_lookup_failure();
                Err(error)
            }
        }
    }

    pub(crate) fn record_metadata_only_request(&self) {
        self.profile.record_metadata_only_request();
    }

    pub(crate) fn record_range_request(&self, bytes: usize) {
        self.profile.record_range_request(bytes);
    }

    pub(crate) fn profile_snapshot(&self) -> DatBtreeProfileSnapshot {
        let entries = self
            .nodes
            .lock()
            .expect("B-tree node cache lock poisoned")
            .entries
            .len();
        self.profile.snapshot(entries, REPLACEABLE_NODE_CACHE_LIMIT)
    }

    /// `BTree::Get_Id_List_Range` / `BTree::Build_Id_List(MinID, MaxID)`: visits only the subtrees
    /// that can hold `first..=last`, through the pinned root and the same 100-node cache as a
    /// lookup, and stops at the first entry past `last`. Retail appends a node's entry before
    /// descending into the child left of it; this walk descends first so DIDs come out ascending.
    pub(crate) fn collect_entries_in_range(
        &self,
        first: u32,
        last: u32,
    ) -> DatResult<Vec<DatRawFileEntry>> {
        let mut out = Vec::new();
        if self.header.root_block > 0 && first <= last {
            self.walk_range(self.header.root_block as usize, first, last, &mut out)?;
        }
        Ok(out)
    }

    fn read_payload(&self, entry: DatRawFileEntry) -> DatResult<Vec<u8>> {
        self.read_payload_prefix(entry, entry.size as usize)
    }

    /// Retail `dclfShortRead` leaves the caller's request buffer size intact. The allocator then
    /// loads only that packed-body prefix (plus one link header for every touched block). Rust
    /// represents the same narrow request as an owned prefix, not as a mutable cache buffer.
    fn read_payload_prefix(&self, entry: DatRawFileEntry, prefix_len: usize) -> DatResult<Vec<u8>> {
        let bytes = self.read_block_chain(
            entry.offset.max(0) as usize,
            prefix_len,
            Some(entry.id),
            "file payload",
            ReadClass::Payload,
        )?;
        self.profile.record_payload_read(
            block_count(prefix_len, self.header.block_data_size(&self.path)?),
            entry.flags & 1 != 0,
        );
        Ok(bytes)
    }

    fn read_payload_range(
        &self,
        entry: DatRawFileEntry,
        offset: usize,
        len: usize,
    ) -> DatResult<Vec<u8>> {
        let packed_len = entry.size as usize;
        let end = offset.checked_add(len).ok_or(DatError::RangeOutOfBounds {
            did: entry.id,
            offset: offset as u64,
            len,
            packed_len: entry.size,
        })?;
        if end > packed_len {
            return Err(DatError::RangeOutOfBounds {
                did: entry.id,
                offset: offset as u64,
                len,
                packed_len: entry.size,
            });
        }
        if len == 0 {
            return Ok(Vec::new());
        }
        let prefix = self.read_payload_prefix(entry, end)?;
        if offset == 0 {
            return Ok(prefix);
        }
        Ok(prefix[offset..].to_vec())
    }

    /// Returns `false` once an entry past `last` is reached, which ends the whole walk.
    fn walk_range(
        &self,
        offset: usize,
        first: u32,
        last: u32,
        out: &mut Vec<DatRawFileEntry>,
    ) -> DatResult<bool> {
        let node = self.node(offset)?;
        let files = &node.files[..node.file_count];
        // `HasEntry` positions the walk at the first entry not below `first`; the children left
        // of it hold only smaller DIDs, as does the child left of an entry equal to `first`.
        let start = files.partition_point(|entry| entry.id < first);
        let skip_start_child = files.get(start).is_some_and(|entry| entry.id == first);
        for index in start..=files.len() {
            if !(node.is_leaf() || index == start && skip_start_child) {
                let branch = node.branches[index];
                if branch <= 0 {
                    return self
                        .invalid_node(offset, "non-leaf node contains invalid child branch");
                }
                if !self.walk_range(branch as usize, first, last, out)? {
                    return Ok(false);
                }
            }
            let Some(entry) = files.get(index) else {
                break;
            };
            if entry.id > last {
                return Ok(false);
            }
            out.push(*entry);
        }
        Ok(true)
    }

    fn find_in_node(&self, offset: usize, did: u32) -> DatResult<Option<DatRawFileEntry>> {
        self.profile
            .record_node_visit(offset == self.header.root_block as usize);
        let node = self.node(offset)?;
        let mut first = 0usize;
        let mut last = node.file_count;
        while first < last {
            let middle = first + (last - first) / 2;
            self.profile.record_key_comparison();
            if node.files[middle].id < did {
                first = middle + 1;
            } else {
                last = middle;
            }
        }
        if first < node.file_count {
            self.profile.record_key_comparison();
            let entry = node.files[first];
            if entry.id == did {
                return Ok(Some(entry));
            }
        }
        if node.is_leaf() {
            return Ok(None);
        }
        let branch = node.branches[first];
        if branch <= 0 {
            return self.invalid_node(offset, "non-leaf node is missing its final branch");
        }
        self.find_in_node(branch as usize, did)
    }

    fn node(&self, offset: usize) -> DatResult<Arc<DatBTreeNode>> {
        if offset == self.header.root_block as usize {
            return Ok(Arc::clone(
                self.root
                    .as_ref()
                    .expect("positive DAT root block must have a parsed root"),
            ));
        }
        let mut cache = self.nodes.lock().expect("B-tree node cache lock poisoned");
        if let Some(node) = cache.entries.get(&offset).cloned() {
            touch(&mut cache.order, offset);
            self.profile.record_cache_hit();
            return Ok(node);
        }
        self.profile.record_cache_miss();
        drop(cache);
        let bytes = self.read_block_chain(
            offset,
            self.header.layout.node_size(),
            None,
            "B-tree node",
            ReadClass::BtreeNode,
        )?;
        self.profile.record_node_load();
        let node = Arc::new(DatBTreeNode::parse(
            &bytes,
            &self.path,
            offset,
            self.header.layout,
        )?);
        let mut cache = self.nodes.lock().expect("B-tree node cache lock poisoned");
        if let Some(existing) = cache.entries.get(&offset).cloned() {
            touch(&mut cache.order, offset);
            return Ok(existing);
        }
        cache.entries.insert(offset, Arc::clone(&node));
        cache.order.push_back(offset);
        if cache.order.len() > REPLACEABLE_NODE_CACHE_LIMIT {
            if let Some(evicted) = cache.order.pop_front() {
                cache.entries.remove(&evicted);
                self.profile.record_cache_eviction();
            }
        }
        Ok(node)
    }

    fn read_exact_into(
        &self,
        offset: usize,
        out: &mut [u8],
        did: Option<u32>,
        context: &'static str,
        read_class: ReadClass,
    ) -> DatResult<()> {
        let end = offset
            .checked_add(out.len())
            .filter(|end| *end <= self.file_len)
            .ok_or_else(|| self.invalid_chain(offset, did, context))?;
        read_exact_at(&self.file, out, offset as u64)?;
        match read_class {
            ReadClass::BtreeNode => self.profile.record_node_positioned_read(end - offset),
            ReadClass::Payload => self.profile.record_payload_positioned_read(end - offset),
        }
        Ok(())
    }

    fn invalid_node<T>(&self, offset: usize, context: &'static str) -> DatResult<T> {
        Err(DatError::InvalidBTreeNode {
            path: self.path.clone(),
            block_offset: offset,
            context,
        })
    }

    fn invalid_chain(&self, offset: usize, did: Option<u32>, context: &'static str) -> DatError {
        DatError::InvalidBlockChain {
            path: self.path.clone(),
            did,
            block_offset: offset,
            context,
        }
    }
}

fn block_count(len: usize, block_data_size: usize) -> usize {
    len.div_ceil(block_data_size)
}

fn record_root_positioned_reads(profile: &DatBtreeProfile, len: usize, block_data_size: usize) {
    let blocks = block_count(len, block_data_size);
    for index in 0..blocks {
        let data_len = block_data_size.min(len - index * block_data_size);
        profile.record_node_positioned_read(data_len + 4);
    }
}

fn read_block_chain_from_file(
    file: &File,
    path: &Path,
    header: DatHeader,
    file_len: usize,
    start: usize,
    len: usize,
) -> DatResult<Vec<u8>> {
    if start == 0 || len == 0 {
        return Err(DatError::InvalidBlockChain {
            path: path.to_path_buf(),
            did: None,
            block_offset: start,
            context: "B-tree root",
        });
    }
    let block_bytes = header.block_data_size(path)?;
    let logical_len = file_len.min(header.file_size.max(0) as usize);
    let mut current = start;
    let mut read = 0usize;
    let output_len = len
        .checked_add(4)
        .ok_or_else(|| DatError::InvalidBlockChain {
            path: path.to_path_buf(),
            did: None,
            block_offset: start,
            context: "B-tree root",
        })?;
    let mut out = vec![0; output_len];
    while current != 0 && read < len {
        let count = block_bytes.min(len - read);
        let block_len = count
            .checked_add(4)
            .ok_or_else(|| DatError::InvalidBlockOffset {
                path: path.to_path_buf(),
                block_offset: current,
                context: "B-tree root",
            })?;
        let block_end =
            current
                .checked_add(block_len)
                .ok_or_else(|| DatError::InvalidBlockOffset {
                    path: path.to_path_buf(),
                    block_offset: current,
                    context: "B-tree root",
                })?;
        if block_end > logical_len {
            return Err(DatError::InvalidBlockChain {
                path: path.to_path_buf(),
                did: None,
                block_offset: current,
                context: "B-tree root",
            });
        }
        let output_end =
            read.checked_add(block_len)
                .ok_or_else(|| DatError::InvalidBlockChain {
                    path: path.to_path_buf(),
                    did: None,
                    block_offset: current,
                    context: "B-tree root",
                })?;
        let output = &mut out[read..output_end];
        let preserved_tail: [u8; 4] = output[..4].try_into().unwrap();
        read_exact_at(file, output, current as u64)?;
        let next = i32::from_le_bytes(output[..4].try_into().unwrap());
        output[..4].copy_from_slice(&preserved_tail);
        read += count;
        if read == len {
            break;
        }
        if next < 0 {
            return Err(DatError::InvalidBlockChain {
                path: path.to_path_buf(),
                did: None,
                block_offset: current,
                context: "B-tree root",
            });
        }
        current = next as usize;
    }
    if read == len {
        out.copy_within(4.., 0);
        out.truncate(len);
        Ok(out)
    } else {
        Err(DatError::InvalidBlockChain {
            path: path.to_path_buf(),
            did: None,
            block_offset: current,
            context: "B-tree root",
        })
    }
}

fn touch(order: &mut VecDeque<usize>, offset: usize) {
    if let Some(index) = order.iter().position(|entry| *entry == offset) {
        order.remove(index);
    }
    order.push_back(offset);
}

#[cfg(unix)]
fn read_exact_at(file: &File, mut target: &mut [u8], mut offset: u64) -> io::Result<()> {
    use std::os::unix::fs::FileExt;
    while !target.is_empty() {
        let count = file.read_at(target, offset)?;
        if count == 0 {
            return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
        }
        offset += count as u64;
        target = &mut target[count..];
    }
    Ok(())
}

#[cfg(windows)]
fn read_exact_at(file: &File, mut target: &mut [u8], mut offset: u64) -> io::Result<()> {
    use std::os::windows::fs::FileExt;
    while !target.is_empty() {
        let count = file.seek_read(target, offset)?;
        if count == 0 {
            return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
        }
        offset += count as u64;
        target = &mut target[count..];
    }
    Ok(())
}
