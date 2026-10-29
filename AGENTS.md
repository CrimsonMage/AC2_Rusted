# AC2_Rusted Agent Rules

`ARCHITECTURE.md` is the ownership map: every crate is named after the Asheron's Call 2 client
subsystem it reproduces. This file states the working rules.

The goal is the retail client improved: retail behaviour is the baseline we port faithfully, and
the client improves on top of it (a modern, Unreal Engine-like look, performance, memory, and
quality-of-life features). An improvement builds on the retail behaviour instead of replacing it:
port the retail code first, then layer the improvement over it, and keep the network protocol,
game rules and server-visible behaviour exactly retail.

- `docs/retail-catalog/` lists every retail function by owning crate; it is the work list. Port
  from the decompile (not from Vitaeum-AC2), then mark the catalog with
  `scripts/retail_catalog.py mark done|partial|skip …` in the same commit.
- Behaviour follows the original AC2 client unless an improvement listed in this file says
  otherwise. Cite the class and function a change reproduces or builds on
  (`CTransition::step_down`, `FrillRender::RefreshFrillMesh`). Each improvement that changes what
  retail did gets an entry below saying what retail did and what the client does instead; add
  the entry in the same commit. Never commit decompiled code, game data or data-derived fixtures.
- Exception to "behaviour follows retail": the retail DB object cache (`DBOCache`, its free-object
  pool, and the 30 second idle eviction in `DBOCache::UseTime` / `DBCache::UseTime`) existed to hide
  slow disks and 2001-era decode cost. Do not reproduce its eviction, pooling or retention counts.
  Read records from the DAT on demand the way retail does (B-tree lookup, then positioned reads;
  `ac2-dat`, no mmap, no up-front index; one deliberate deviation: a chain's blocks that lie
  together in the file are read with one positioned read of up to 128 KiB, where
  `CLBlockAllocator::Load_Data` read each block on its own), decode on worker threads, share one `Arc` per DID with
  in-flight dedupe, and free on last drop. The typed cache keeps released values only while their
  bytes fit `ResourceCacheConfig::free_byte_budget` (`DEFAULT_FREE_CACHE_BUDGET_BYTES`), reaping
  the oldest released first on the inserting worker (`TypedResourceCache::trim_if_over_budget`),
  never on a frame-loop timer. The frame thread must never decode or read files. DAT-reading
  threads are fixed counts (four steady readers in the world), never scaled with the core count;
  `crates/ac2-db/src/loader_pool.rs` lists them and says why.
  Still reproduce: (1) sub-object resolution, where `DBOCache::GetIfInMemory` calls a vtable slot to
  load an unloaded object's dependent DIDs on first fetch; (2) the main-thread pump of finished
  async requests that `AsyncCache::UseTime` performs, as a per-frame time-budgeted drain of a
  channel fed by the workers. Verified (Legions 0x0065AE60): `AsyncCache::UseTime` is only that
  pump, a 25 ms `GetTickCount` budget over `LFQueue::Consume` of the pending callbacks, each run
  then released by refcount. Not yet verified: whether `AddToDataGraph` matters outside tools,
  and whether the Pyrealium server ever pushes updated portal objects (the retail on-disk
  `AsyncCache` / `CMissingIteration` path exists for that). Read those before deleting the
  related code, and record the findings here.
- Exception: the window resizes freely (a modern client). Retail drew its UI pixel for pixel on a
  fixed window authored at 1024 x 768; here `ac2_client::ui_scale::UiScale` lays the UI out on a
  logical display and draws it scaled by `min(width / 1024, height / 768)`. The HUD fills the
  window and the pregame screens stay 1024 x 768, centred. Layout and anchoring code stays retail.
- Exception: one modern shading pass, no retail look beside it. What is drawn, its colours,
  order, blend, fog and baked light come from the DATs and the decompile (`LScapeSky::Draw`,
  `CPartArray` colours, `CEnvCell` vertex light); the shading turns that into linear light.
  It covers lighting (GGX for lit objects), sky, clouds, weather and the tone map.
  Scenes draw unclamped into an `Rgba16Float` target (`ac2_gpu::HdrTarget`); one Khronos PBR Neutral tone map
  (`ac2_gpu::ToneMap`) reaches the window, and the UI draws after it. Before it, the world alone
  takes a post chain (`ac2_gpu` `post.rs`, `ao.rs`): ambient occlusion from depth, a dual-filter
  bloom, eye adaptation (half the metered difference from a daylight key, up to one stop darker
  and never brighter, so night stays dark) and a light vignette. UI viewport scenes skip the
  chain and composite over their panel where they drew. The sky's atmosphere and cloud deck
  (`sky.wgsl`) stand in for the backdrop dome, so they fade with the dome's authored opacity
  (`LScapeSky::UpdateTimeOfDay`): by night retail drew the night layers and stars over an empty
  background. Do not add a retail/modern switch.
- Exception: bodies and objects cast sun shadows from depth maps. Retail's `DynamicObjectShadows`
  drew a dark disc decal under each moving object (1, `WorldRender::CreateDiscShadows`) or stencil
  shadow volumes of the moving objects along `SceneTool::m_vShadowDirection` (2, and 3
  self-shadowing; `SceneTool::GenerateGlobalShadowVolume`, at most 100), darkened by a screen quad
  of `m_cVolumetricShadowStrength` (`RenderGlobalVolumetricShadows`); static objects shadowed only
  the terrain, through `LScapeRender::RefreshShadowMap`'s 512 x 512 map. Here every level above 0
  draws cascaded sun depth maps (`ac2_gpu` `sun_shadow.rs`, `object_sun_casters.rs`; level 1:
  2 x 1024 within 60 m, 2: 2 x 2048 within 80 m, 3: 3 x 2048 within 150 m) that the near static
  objects (no frills) and every body and world object cast into, posed and skinned. Terrain,
  objects and bodies receive them with 3 x 3 PCF and keep `mix(1 - strength, 1, lit)` of their
  colour at the static strength (`m_cStaticShadowStrength`), so the cascades meet the landscape
  map seamlessly; the terrain takes the darker of the two, and a frill already darkened by the
  landscape map is not darkened twice. The landscape map stays as retail draws it past the
  cascades. Level 0 draws none, as retail.
- Exception: lens flares fade by the world's depth. Retail's `LensFlareEffect::Draw` drew a
  triangle at the light after the world and read one back-buffer pixel the next frame
  (`SceneTool::RequestPixelScan`, `GetPixelScanResult`), raising `m_fOpacity` by the elapsed time
  times `m_fFadeSpeed` only while it showed. Here a pass after the world pass steps the opacity
  the same way towards the fraction of a 5 x 5 patch around the light that shows the cleared far
  depth (`ac2_gpu` `lens_flare.rs`), on the GPU, and the sprites read it a frame late as retail's
  did. The screen falloff stays at its fifth for a light past the screen's corners (retail's
  square rose again there while the flare faded), and a sky light's direction is taken from the
  viewer (retail transformed the point 1000 out without the viewer's origin). A cell light's
  corona (`CEnvCell::RenderLightCoronas`) is seen where nothing is nearer than the light, moved
  0.2 m towards the camera (retail's scan triangle sat at a fixed depth of 0.9999); the 64
  nearest flares in view draw, each keeping its fade while it stays in view.
- Exception: the portal space is a scene. Retail's `PortalSpace::Effect::Render` drew the fader, the
  background mesh, the tunnel rings and the cloud particles straight into the back buffer. Here
  they draw in retail's order into the `Rgba16Float` scene target from the portal camera over a
  cleared depth (after the world while it fades), and take the world's post chain and the one tone
  map; the fader's colour is decoded to linear light. The world's ambient occlusion is recorded
  before the portal pass clears the depth, so the world keeps it while it fades; the portal space
  itself takes none.
- Exception: every HUD window can be moved and keeps its place. Retail moved only windows with a
  drag bar and saved only `save_loc` windows to the server (`gmCWInterface::SetUISaveLocations`).
  Here a window with a drag bar still moves by it, and any other top-level HUD window moves by
  Alt + left drag anywhere on it. Every window's position is saved locally per character (the
  client's config directory, `ac2_prefs::paths`) and restored at login, over the server's
  locations; `save_loc` windows still upload to the server as retail does. The HUD's chrome
  (bars, radar, selection field, toolbar and its side panels) is windows too. Not moved: the
  world's overlays (targeting reticle, name plates, damage text), the heading ring (it follows the
  radar), the doll's 3D viewport (docked to the inventory), full-display backdrops and tooltips.
- Exception: text fields support modern selection and the clipboard: mouse drag and Shift+arrow
  selection, highlighted, with Ctrl+A / Ctrl+C / Ctrl+X / Ctrl+V (the OS clipboard). Password
  fields never copy out. Retail's own text editing rules (max length, enter/escape) still apply.
  Read-only selectable text (chat log, tells, books) takes Ctrl+A / Ctrl+C too, and a drag
  selection auto-scrolls at the text's edge. On macOS the shortcuts are Cmd+A/C/X/V.
- Exception: the mouse wheel scrolls every panel that has a scroll bar (texts, lists, trees, and
  the bar itself), from the element under the pointer up to its first scrollable ancestor. A
  wheel turn over a bar's thumb or arrow buttons reaches the bar; retail's buttons swallowed it.
  Pixel deltas (trackpads, smooth wheels) accumulate into whole notches. All of this (wheel,
  clipboard, Alt-move) must work on Windows, macOS, Linux X11 and Linux Wayland.
- Exception: help pages are drawn in the client on every platform. Retail's KSML help viewer was
  `AC2HelpPlugin.dll`, a Windows-only KeyStone plugin. `KeyStone::OpenHelp` pages (`ac2_ui::help`
  resolves the page, `ac2_ui::help_view` lays it out) are shown in the client's own window
  (`ac2_ui::help_window`: a second `Book` of the gameplay layout, made as its own root), with the
  same pages, links, tabs and frames. The book's text area has one font, so bold, italic and
  underline are drawn plain.
- Exception: a first run without a saved graphics quality starts at a level chosen from the GPU
  (`ac2_gpu::suggested_quality_level`: 5 on a discrete or Apple GPU, 3 otherwise), applied but not
  saved. Retail's `Render::InitRenderingPreferences` applied a saved level 1 to 5 (still done, by
  `ClientPrefs::apply_startup_quality`) and otherwise kept the registered defaults, which are
  level 3's medium model and scenery detail, sized for 2002 hardware.
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
