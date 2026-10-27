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
- Exception: the window resizes freely (a modern client). Retail drew its UI pixel for pixel on a
  fixed window authored at 1024 x 768; here `ac2_client::ui_scale::UiScale` lays the UI out on a
  logical display and draws it scaled by `min(width / 1024, height / 768)`. The HUD fills the
  window and the pregame screens stay 1024 x 768, centred. Layout and anchoring code stays retail.
- Exception: one modern shading pass, no retail look beside it. What is drawn, its colours,
  order, blend, fog and baked light come from the DATs and the decompile (`LScapeSky::Draw`,
  `CPartArray` colours, `CEnvCell` vertex light); the shading turns that into linear light.
  It covers lighting (GGX for lit objects), sky, clouds, weather and the tone map.
  Scenes draw unclamped into an `Rgba16Float` target (`ac2_gpu::HdrTarget`); one ACES tone map
  (`ac2_gpu::ToneMap`) reaches the window, and the UI draws after it. UI viewport scenes
  composite over their panel where they drew. Do not add a retail/modern switch.
- Exception: every HUD window can be moved and keeps its place. Retail moved only windows with a
  drag bar and saved only `save_loc` windows to the server (`gmCWInterface::SetUISaveLocations`).
  Here a window with a drag bar still moves by it, and any other top-level HUD window moves by
  Alt + left drag anywhere on it. Every window's position is saved locally per character (the
  client's config directory, `ac2_prefs::paths`) and restored at login, over the server's
  locations; `save_loc` windows still upload to the server as retail does.
- Exception: text fields support modern selection and the clipboard: mouse drag and Shift+arrow
  selection, highlighted, with Ctrl+A / Ctrl+C / Ctrl+X / Ctrl+V (the OS clipboard). Password
  fields never copy out. Retail's own text editing rules (max length, enter/escape) still apply.
- Exception: the inventory paper doll frames the whole figure as admin 7148's
  `gmPaperDollUI::UpdatePaperDollPosition` does (avatar at -0.5 of its height, the camera that
  height tall); Legions 7160 framed a head-and-shoulders portrait (-0.875, a quarter).
- No admin tools in the client. The admin 7148 build is a reference for names and code, not for
  features: do not port its admin-only commands, debuggers, account overrides, admin entities
  (e.g. `wsl_butcherytool`) or admin UI; mark those catalog rows `skip` with the reason.
  Test-only code (`view::probe`) stays behind the `test-support` feature.
- Exception: drawn poses blend between the last two bone refreshes (`ac2_anim::blend_transform`;
  `Client::set_pose_blend`, on by default), so the skeleton moves at display rate. Retail draws the
  last refresh; turn the blend off to get it. Drawn positions (`PhysicsBody::drawing_position`)
  follow retail's render interpolation and are not an exception.
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
- File size: plan a split by responsibility once a `.rs` file reaches 1000 lines; 1500 lines is a
  hard fail (`scripts/check_file_sizes.sh` fails the gate).
- Keep public APIs small: plain data and narrow entry points.
- Tests that need game data read `AC2_DAT_DIR` and are skipped when it is unset.
- `crates/` is the client. `tools/` holds test tooling only; the world viewer is an offline
  harness and never connects to a server.
- Close-out reports list `Retail Behavior traced:`, `Verification:` and `Known Gaps:`.
