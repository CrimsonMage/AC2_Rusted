# DAT read path: retail compared with AC2_Rusted

A study made on 2026-10-28: how the retail client reads the DATs, how AC2_Rusted does, what it
costs, and what is worth changing. The study code (read counters, a per-record trace, a replay
tool and the variants compared) is on the branch `dat-read-study`, not merged.

## Summary

DAT reading is not a bottleneck, and the read path already matches retail. With the files in the
OS page cache, every record a whole walk or crowd scene reads (about 14,000 records, 22 to 31 MB)
takes 20 to 28 ms on one thread. Two changes are worth making: read and decode the landblock
record once, and keep the number of DAT-reading threads at 2 to 4.

## How retail reads (Legions 7160)

- One file handle per DAT. `DiskDev::Open_File` (0x005FF970) opens it with `CreateFileA` and no
  special flags, so reads go through the OS cache. No read-ahead or prefetch of its own.
- `DiskDev::SyncRead` (0x005FF8B0) seeks, then reads. `CLBlockAllocator::Load_Data` (0x005FDC60)
  does one read per block, header included, following the chain into one buffer of size + 4.
- `DiskController::LoadData` (0x005FABC0): `BTree::Lookup`, then allocate and `Load_Data`.
  `BTree::LoadTree` (0x005FBE90) pins the root plus 100 cached nodes of 1228 bytes (about 125 KB);
  `BTree::FindNode` (0x005FC160) replaces the oldest node by timestamp.
- `CThreadsafeDiskController::LoadData` (0x004B3A40) holds `m_DiskLock` over the lookup and the
  payload read, because seek-then-read shares one file pointer.
- One worker thread. `ThreadedCache` (constructor 0x006594D0) has a single worker;
  `WorkerExecuteGetRequest` (0x00659170) only reads raw bytes; `ThreadedCache::UseTime`
  (0x006592F0) drains replies on the main thread within 25 ms; `AsyncCache::OnGetRequestFinished`
  (0x0065C130) decodes on the main thread, then queues sub-object IDs. `BlockingGetFromDisk`
  (0x0065A930) reads synchronously on the caller's thread.
- Landblock streaming: `LScapeManager::UpdateVisibleCells` / `UpdateBlocks` (0x0054B260 /
  0x0054AB50) shift the near grid when the player crosses a block and fill only the empty slots
  through `GetFreeBlock` (0x0054A830), which builds the mesh synchronously.
  `CellManager::AsyncGetCells` (0x0054B850) requests only the destination block asynchronously.
  Retail does no prefetch in the direction of movement.

## How AC2_Rusted reads

- `crates/ac2-dat/src/file_btree_runtime.rs`: one shared `File` per DAT, positioned reads
  (`read_at`), one per block like retail (`read_block_chain`).
- The root is pinned and 100 nodes are cached, with an LRU (`touch`).
- The lock covers only the lookup; payload reads run outside it (positioned reads share no file
  pointer), a harmless deviation from retail.
- Per record: one `Vec` of len + 4 and a memmove. No AC2 record is stored compressed.
- Decoding runs on worker threads, as `AGENTS.md` requires.
- DAT-reading threads: 2 `ac2-loader` (`crates/ac2-client/src/client.rs`), 1 `ac2-scene-loader`
  (`crates/ac2-render/src/scene/loader.rs`), 1 preparation thread, plus the UI image, HUD string
  and audio threads.
- Syscalls per record: Portal about 14, Cell about 6, Local about 1.3. A B-tree node miss costs 2
  reads in Portal and 5 in Cell (256-byte blocks).
- **The landblock record is read and decoded twice.** Cell `xxxxFFFF` is decoded as
  `Ac2LandBlockRecord` (`crates/ac2-world/src/ac2_world.rs`) and again as `Ac2SceneLandBlock`
  (`crates/ac2-world/src/ac2_scene/sound.rs`): 74 extra reads per walk and two decoded copies in
  memory. Retail keeps one object per ID.
- DIDs that do not exist are looked up again every time (Cell `xxxxFFFE` up to 5 times);
  negligible.
- The string-table preload (`crates/ac2-client/src/hud_view/build.rs`) is 76% of all reads (10,604
  Local records, 6.7 MB). Retail fetches strings on demand (`StringInfo::AsyncGetString`).

## Measured

Load average 9 to 15 during the runs, so timings are noisy.

| What | Result |
|---|---|
| Records read, walk bench | Portal 1,113 (14.8 MB), Cell 377, Local 12,383 |
| Records read, crowd 100 | Portal 1,532 (23.9 MB) |
| Syscalls, walk bench (strace) | 32,759 `pread64` |
| Block chains already contiguous on disk | Portal 96%, Cell 95% |
| Replay of the same reads, page cache warm | 20 to 28 ms on 1 thread, about 15 ms on 4 |
| Same, DAT pages evicted (NVMe) | 800 to 900 ms on 1 thread; 300 to 480 ms on 2 to 8 |
| Loader thread CPU per run | scene loader 30 to 70 ms, blocks 10 ms |
| Main thread CPU per run | 0.5 to 1.5 s (frames) |

Of the bench's 4 to 6 s wall time, about 3 s is the fixed `--warm-up` sleep and 0.2 to 0.8 s GPU
setup; the rest is frames.

## Recommendations, ranked

1. **Read and decode the landblock record once**, and build the ambient sound points from it.
   Removes the duplicate reads and memory; retail behaviour; low risk.
2. **Keep 2 to 4 DAT-reading threads**, not scaled with core count. 1 to 2-4 threads halves the
   cold-cache time; beyond 4 gains nothing. Retail had one.
3. Optional: **coalesce reads of adjacent blocks** (one read per contiguous run, up to 128 KiB).
   Cut reads by 58 to 64% (33k to 14k per walk) but wall time did not change on Linux, warm or
   cold, because the kernel already merges sequential reads. Likely helps Windows or spinning disks
   (not measured). Costs a scratch buffer of at most 128 KiB per thread. Still on-demand reads with
   no mmap or index, so allowed by `AGENTS.md`, though not retail's one read per block.
4. Optional: **load strings on demand** as retail does, if first-launch time or RAM matters.
5. Not worth it: a bigger node cache (1,000 nodes took node loads only from 618 to 582); removing
   the lookup lock (no clear gain, more duplicate node loads); prefetch along the movement (retail
   does not, and reads are cheap); mmap or a startup index (forbidden by `AGENTS.md`).

Measured: counts, timings, strace and per-thread CPU. Inferred, not measured: the Windows and
spinning-disk benefit of coalescing, and Windows cache read-ahead.

## Study tooling (branch `dat-read-study`)

- `AC2_DAT_STATS=1`: `world-viewer` prints read counts, timings, phase times and per-thread CPU.
- `AC2_DAT_TRACE=<file>`: logs every record read; `crates/ac2-dat/examples/dat_replay.rs` replays
  such a log.
- `AC2_DAT_COALESCE`, `AC2_DAT_NODE_CAP`, `AC2_DAT_NOLOCK`: the variants compared.
