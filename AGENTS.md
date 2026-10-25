# AC2_Rusted Agent Rules

`ARCHITECTURE.md` is the ownership map: every crate is named after the Asheron's Call 2 client
subsystem it reproduces. This file states the working rules.

- `docs/retail-catalog/` lists every retail function by owning crate; it is the work list. Port
  from the decompile (not from Vitaeum-AC2), then mark the catalog with
  `scripts/retail_catalog.py mark done|partial|skip …` in the same commit.
- Behaviour follows the original AC2 client. Cite the class and function a change reproduces
  (`CTransition::step_down`, `FrillRender::RefreshFrillMesh`). Never commit decompiled code,
  game data or data-derived fixtures.
- Exception to "behaviour follows retail": the retail DB object cache (`DBOCache`, its free-object
  pool, and the 30 second idle eviction in `DBOCache::UseTime` / `DBCache::UseTime`) existed to hide
  slow disks and 2001-era decode cost. Do not reproduce its eviction, pooling or retention counts
  (`retail_cache_retention.rs` and the timer-driven pass in `typed_cache_maintenance.rs` are
  candidates for removal). Read records from the DAT on demand the way retail does (B-tree lookup,
  then positioned reads; `ac2-dat`, no mmap, no up-front index), decode on worker threads, share one
  `Arc` per DID with in-flight dedupe, and free on last drop; cap memory by byte budget on the
  worker side, never on a frame-loop timer. The frame thread must never decode or read files.
  Still reproduce: (1) sub-object resolution, where `DBOCache::GetIfInMemory` calls a vtable slot to
  load an unloaded object's dependent DIDs on first fetch; (2) the main-thread pump of finished
  async requests that `AsyncCache::UseTime` performs, as a per-frame time-budgeted drain of a
  channel fed by the workers. Not yet verified: what `AsyncCache::UseTime` does beyond that, whether
  `AddToDataGraph` matters outside tools, and whether the Pyrealium server ever pushes updated
  portal objects (the retail on-disk `AsyncCache` / `CMissingIteration` path exists for that). Read
  those before deleting the related code, and record the findings here.
- Where admin 7148 and Legions 7160 differ, follow Legions (the DVD client the DATs ship with):
  read `decompiled-pdb/legions7160/source_tree/` beside the admin tree and note the difference.
- Numeric fidelity: retail ran x87 at 53-bit precision (`_setdefaultprecision`, and
  `D3DCREATE_FPU_PRESERVE` on every `CreateDevice`), rounding to f32 only when storing into
  `float` memory. Evaluate multi-operation expressions in f64 (scalar or f64 SIMD lanes) and round
  to f32 where retail stores a `float`; f32 SIMD is exact only for a single operation between f32
  values stored straight back. Keep hot loops SIMD-friendly (struct-of-arrays, no per-element
  branching) without dropping below that precision.
- New code goes in the crate that owns that AC2 subsystem. If no crate fits, revise
  `ARCHITECTURE.md` first.
- The dependency graph is one-way in the order `ARCHITECTURE.md` lists.
- Every `.rs` file stays under 800 lines; split by responsibility.
- Keep public APIs small: plain data and narrow entry points.
- Tests that need game data read `AC2_DAT_DIR` and are skipped when it is unset.
- `crates/` is the client. `tools/` holds test tooling only; the world viewer is an offline
  harness and never connects to a server.
- Close-out reports list `Retail Behavior traced:`, `Verification:` and `Known Gaps:`.
