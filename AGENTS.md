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
  and whether the AC2e server ever pushes updated portal objects (the retail on-disk
  `AsyncCache` / `CMissingIteration` path exists for that). Read those before deleting the
  related code, and record the findings here.
- Exception: the window resizes freely (a modern client), and the UI has a scale setting.
  Retail drew its UI pixel for pixel on the device's display, authored at 1024 x 768: every root
  is anchored to `RenderDevice::GetDisplayWidth` / `GetDisplayHeight`
  (`UIElement::UpdateSizeAndPosition`, run for each root by `UIElementManager::RefreshEvent`), so
  a bigger display held a pixel-sized UI covering less of the screen. That is the default here
  (scale 1): the HUD lays out on the real window size in UI pixels (2560 x 1440 at 2560 x 1440;
  the vitals, radar and chat keep their authored places and sizes, the toolbar sits on the
  bottom edge at its native width) and the world gets the room. The improvement is the
  client-only `UiScale` preference (`ac2_ui::ui_scale_pref`, key `0x7E000002` in the `Misc`
  section beside `ClassicHud`, a drop-down on the options page, stored as hundredths so
  `UiScale=250` is 2.5x): 1x (retail size, the default), 1.25x, 1.5x, 1.75x, 2x, 2.5x, 3x, and
  Fill. `ac2_client::ui_scale::UiScale` lays the HUD out on a logical display of
  `window / scale` UI pixels and draws it scaled (`UiScale::hud`); a scale that would leave the
  display under 800 x 600 (retail's smallest mode) is held at the one that fits. Fill is the
  first modern client's behaviour, `min(width / 1024, height / 768)`, the largest scale that
  keeps 1024 x 768 inside the window. Applying the setting (`OptionsEffect::UiScale`,
  `ClientView::set_ui_scale`) lays the HUD out again on the new display without rebuilding it:
  `UiTree::set_display` places every root and a saved window place, which records the display it
  was saved on, keeps its size and its distance from the nearest edges (`window_positions.rs`
  `anchored`), so switching scale does not scatter the windows. The scaled HUD is drawn
  stretched (smooth, `UiOverlay::set_smooth`; its radar and bandwidth graph draw into the
  surfaces in UI pixels); at 1x nothing is filtered. Retail's `DisplayResolution`
  (`Device::InitializeRenderingPreferences`: the device's modes over 799 x 599, 1024 x 768
  always listed and the default) switched the display mode; here it lists the monitor's modes
  the same way and sets the window's inner size, at start and when the player picks a mode (a
  window resized by hand keeps its size until then; FullScreen stays the borderless switch).
  The pregame screens (connecting, character select and creation) are one 1024 x 768 panel
  scaled to fit the window and centred (`UiScale::fit`, `min(width / 1024, height / 768)`; the
  user's choice: retail showed them at 1024 x 768, and the UI Scale setting applies to the HUD
  only). They are composed at that scale, not stretched from 1024 x 768 (`UiTree::set_render_scale`,
  `ac2_ui::compose_scaled`, `UiOverlay::set_density`): each root's surface is its scaled size,
  every image is filtered once from its source to its scaled box (Catmull-Rom where it grows),
  each glyph is grown from the DAT font's coverage with its edge narrowed to a window pixel, and
  the surfaces are drawn texel for texel. Layout and anchoring code stays retail.
- Exception: a window nobody placed takes its authored place proportionally. Retail applied a
  window location only where the profile held one (`UIMainFramework::ApplyLocations` for the
  entries of the player's `UISaveLocations`, run on `gmGamePlayUI::ListenToMessage` `0xffff0007`
  after `UIElementManager::RefreshEvent` and on `RecvNotice_ReloadOptions`; `UIElement::ApplyLocation`
  maps the stored clip-space corners to the display and resizes only an `allow_resize` window).
  A `save_loc` window with none stayed where `UIElement::UpdateSizeAndPosition` left it, which for
  the layouts' fixed-edge windows (the radar, the four chat windows, most panels) is their
  authored pixel place, so on a bigger display they sat near the top left.
  `UIElementManager::CreateRootElementByDataID` did record each such root's creation-time clip box
  (`UIElement::SetDefaultLocation` into `m_defaultLocation`), but nothing reads it. Here that box
  is used (`ac2_ui` `default_locations.rs`): a `save_loc` root whose four edges are fixed, with no
  place saved by the player (`window_positions`) and none from the server, and that nothing
  else has moved, is put where its authored box lies in clip space over the layout's display,
  mapped onto the current display, by `ApplyLocation`'s rule (the same size, unless it allows
  resizing, then the far corner is mapped too, `max_width` and the like still clamping). It runs
  on every display change (`UiTree::set_display`: start, resize, the UI scale), so the radar sits
  at the same fraction of the screen at every scale. A server location (`apply_locations`) is
  kept and placed again on a display change; the player's saved places still keep their distance
  from the nearest edges, as above. Edge-anchored windows are retail's as before, and the modern
  style's own placed parts are not moved.
- Exception: one modern shading pass, no retail look beside it. What is drawn, its colours,
  order, blend, fog and baked light come from the DATs and the decompile (`LScapeSky::Draw`,
  `CPartArray` colours, `CEnvCell` vertex light); the shading turns that into linear light.
  It covers lighting (GGX for lit objects), sky, clouds, weather and the tone map.
  Scenes draw unclamped into an `Rgba16Float` target (`ac2_gpu::HdrTarget`); one Khronos PBR Neutral tone map
  without its toe (values under the shoulder pass unchanged, so dark tones and small channels keep
  their level as an untone-mapped image does; the shoulder starts at 0.80)
  (`ac2_gpu::ToneMap`) reaches the window, and the UI draws after it. Before it, the world alone
  takes a post chain (`ac2_gpu` `post.rs`, `ao.rs`): ambient occlusion from depth (GTAO, Jimenez et al. 2016, at half size with a depth-aware blur,
  temporally accumulated under the TAA's camera reprojection; a client deviation, retail had no
  ambient occlusion; the tier follows the overall graphics quality: level 1 off, 2-3 two slices of
  four steps, 4-5 three slices of six, `SceneOptions::ao_quality`; it multiplies the tone map's
  direct light too, as a forward pass without a prepass cannot apply it to ambient alone), a dual-filter
  bloom, eye adaptation (half the metered difference from a daylight key, up to one stop darker
  and never brighter, so night stays dark) and a light vignette. UI viewport scenes skip the
  chain and composite over their panel where they drew. The sky's atmosphere and cloud deck
  (`sky.wgsl`) stand in for the backdrop dome, so they fade with the dome's authored opacity
  (`LScapeSky::UpdateTimeOfDay`): by night retail drew the night layers and stars over an empty
  background. The sky draws camera-relative (the atmosphere's rays and the DAT sky objects,
  which ride the viewer as `LScapeSky::SetViewerPosition` places them, with the eye at the origin):
  in f32 at AC2's world coordinates the absolute view made the sky jitter as the eye moved. The screen brightness (`ScreenBrightness`) scales the world's display levels by
  `1 + 2 * brightness` after the tone map, as retail's gamma ramp did
  (`RenderDeviceD3D::SetGamma`), but the UI keeps its authored levels (retail's ramp brightened
  the whole screen). Do not add a retail/modern switch to the shading (the HUD's two styles, below,
  are the only such switch).
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
  map seamlessly (the static casters draw as the map's do, `m_bRenderingToShadowMap` clearing
  `CPartArray::Draw`'s degrade steps: every placement undegraded, unfaded and not distance-culled;
  bodies and world objects keep their own detail, as retail's volumes did); the terrain takes the darker of the two, and a frill already darkened by the
  landscape map is not darkened twice. The landscape map stays as retail draws it past the
  cascades. Level 0 draws none, as retail. UI viewport scenes use them too: `UIScene::Render` drew
  its entities' stencil volumes at `m_cVolumetricShadowStrength` 0.75 from level 2 up (the
  default sun of a scene without lights; levels 0 and 1 draw none there), so the scene's objects
  cast into one 1024 x 1024 cascade fitted to their bounding sphere (an orthographic camera sits
  10 km back, so the view's frustum is no basis) and the objects receive it (`ac2_gpu`
  `ui_scenes.rs`, `SceneRenderer::set_viewport_scene`, `fit_viewport_cascade`).
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
- Exception: the water reflects in screen space. Retail's `WorldRender::RenderReflectionMap`
  drew the sky, the terrain and the objects above the frame's water plane
  (`LScapeRender::FindReflectionPlane`) from a camera mirrored about it into a target, and
  `LScapeTerrainMesh::DrawWater` blended that target over the stencilled water pixels by their
  destination alpha, rippled by a bump map at `WaterReflections` 2. Here the same plane is found
  (the viewer's block's water below it, else the first resident block's), and after the world pass
  each pixel whose view ray meets that plane over one of its water blocks marches the mirrored
  ray through the world's depth (`ac2_gpu` `water_reflection.rs`): what it meets on the screen,
  else the world's colour where the mirrored direction reaches the screen, fading to the horizon's
  colour towards the screen's edges and beyond them, blended in by a Fresnel weight, with
  procedural ripples at level 2. What is off the screen is not reflected.
- Exception: `Antialiasing` drives temporal anti-aliasing and sharpening, and defaults on. Retail's
  `Antialiasing` device preference (`ID_Rendering_Antialiasing`, key 0x4000_0060, default off) became
  the D3D presentation's `MultiSampleType` (`RenderDeviceD3D::SetMultiSampleAntialias`,
  `D3DRS_MULTISAMPLEANTIALIAS`). Here the scene targets stay single-sampled (the post chain, the
  depth reads and the GL fallback need that): every world draw (objects with their particles,
  frills, decals and streaks; the near and far terrain; the sky and its objects; the water's
  reflection march) shifts by an 8-phase Halton(2,3) sub-pixel jitter, and one full-screen resolve
  after the world's last contribution (`ac2_gpu` `taa.rs`, `taa.wgsl`) blends it into a history
  reprojected from the depth buffer and the camera's motion (eye-relative matrices plus the eye's
  step taken in f64; the sky as a direction), with a Catmull-Rom history filter, a variance clip in
  YCoCg, Karis luminance weights and a blend of 0.1 rising to 0.25 with motion. The history restarts
  on a resize, a camera cut (the eye over 5 m in a frame, or a field of view change), while the
  portal space shows (it draws after the resolve, unjittered) and when the preference is turned
  off and on. A contrast adaptive sharpen (CAS, strength 0.25) in the tone map runs only while it
  is on. Off draws exactly as before: no jitter, no resolve, no sharpen. UI viewport scenes, the
  sun shadow cascades, the landscape shadow map and the lens flares are not jittered. Moving
  objects and bodies carry motion vectors: the world pass has a second colour attachment
  (`Rg16Float`, cleared to a "no object" sentinel) that the depth-writing object layers (objects,
  bodies, frills) fill with the screen step of the surface from its previous placement to its
  current one, both unjittered. The vertex stage takes the previous placement from the instance
  (the part's previous matrix relative to the previous eye, taken off in f64 on the CPU), the
  previous frame's bone matrices and the previous eye-relative view-projection, and the grass wind
  at the previous time; `SceneRenderer` keeps the previous frame's object list for it, so an object
  not drawn on the previous frame, one that jumped over 10 m, or whose rig changed has no record and
  writes the sentinel. The resolve reads the nearest-depth neighbour's velocity and falls back to
  the depth reprojection where it holds the sentinel. Terrain, sky and the other pipelines declare
  the target with an empty write mask. With the anti-aliasing off no previous placement is laid out
  and the velocity is discarded. Known gaps: the grass push and the pose blend use this frame's
  inputs for the previous position.
- Exception: a particle repulsor acts in the space of the system's particles. Retail's
  `ParticleEmitter::DoRepulse` subtracted the repulsor's `Position` origin (the object's frame
  origin) from each particle's origin as they stood, which only agrees for a world-space system;
  a system whose particles are relative to its own origin was compared against an unrelated
  origin. Here the repulsor's place is taken relative to the system's origin for such a system
  (`ParticleSystem::think_with`), so a repulsor pushes it as it does a world-space one.
- Exception: the portal space is a scene. Retail's `PortalSpace::Effect::Render` drew the fader, the
  background mesh, the tunnel rings and the cloud particles straight into the back buffer. Here
  they draw in retail's order into the `Rgba16Float` scene target from the portal camera over a
  cleared depth (after the world while it fades), and take the world's post chain and the one tone
  map; the fader's colour is decoded to linear light. The world's ambient occlusion is recorded
  before the portal pass clears the depth, so the world keeps it while it fades; the portal space
  itself takes none.
- Exception: every HUD window can be moved and keeps its place. Retail moved only windows with a
  drag bar and saved only `save_loc` windows to the server (`gmCWInterface::SetUISaveLocations`).
  Here a window with a drag bar still moves by it, any other top-level HUD window moves by a
  left press in its header band (the top 24 pixels, `ac2_ui::window_locations::HEADER_BAND`;
  a press on a button, an editable or selectable text, a list, a scrollbar or a dragable item
  there stays with that control, a static title does not, and a resize grip keeps it only within
  6 pixels of the window edge it drags: the side panels' top grip spans the whole title, 45
  pixels tall) and
  by Alt + left drag anywhere on it; a locked window (`UiTree::set_locked`, the modern HUD's
  lock) moves by neither. Every window's position is saved locally per character (the
  client's config directory, `ac2_prefs::paths`) and restored at login, over the server's
  locations, and again after the window changes size (where retail's `RefreshEvent` re-applied
  the server's saved locations, scaled with the display). A saved place records the display it
  was saved on; on another display the window keeps its size and its distance from the edges it
  was nearest (left, right or centred; top, bottom or centred), so a bigger window does not drift
  windows inwards or stretch them (`window_positions.rs` `anchored`); `save_loc` windows still upload to the server as retail does. The HUD's chrome
  (bars, radar, selection field, toolbar and its side panels) is windows too. Not moved: the
  world's overlays (targeting reticle, name plates, damage text), the heading ring (it follows the
  radar), the doll's 3D viewport (docked to the inventory), the consignment's portrait (docked to
  its panel, kept at its `SyncPortrait` place each loop), full-display backdrops and tooltips.
- Exception: the classic inventory resizes vertically. Retail's `InventoryField` (in
  `RightSidePanel`, toolbar layout `0x41000003`) has no resize grip, no `allow_resize` and no
  `min_height` / `max_height`: the window had one size and the pack page showed three rows of six
  slots. Here the classic layout is copied with a `UIElement_Resizebar` along the window's top
  edge (cloned from the shortcut bar's top grip, `border_top`; the window sits on the display's
  bottom edge, so it grows upward), `allow_resize` and a height of at least the layout's own
  (414) and at most 24 rows more, and the page's parts anchored for a taller window
  (`UIElement::UpdateSizeAndPosition` edges on the references: the paper doll half and the page
  tabs keep their height, the bottom border follows the bottom edge, the pack half takes the
  growth), so the pack list (`UIElement_ListBox::ResizeTo`) shows more rows. The resize itself is
  retail's (`UIElement::MouseResizeElement`, `StopResizing`, the saved size applied by
  `ApplyLocation`); the one addition is that the height moves in whole pack rows (36 pixels, the
  window attribute `resize_height_step`, `inventory_resize.rs`), measured from the pointer where the
  resize started. The other right-side panels keep their height in the taller window, and the grip
  is only there while the inventory page is shown. The modern style's inventory is not changed.
- Exception: the HUD has two styles, modern and classic. Retail drew only its DAT layouts' HUD.
  Here the classic style is that HUD, unchanged; the modern style (the mockup "variant D" the
  user chose) builds the same frameworks over the same elements, restyling their layout
  descriptions before they are made (each restyle keeps every element id and state, so every
  framework's behaviour stays retail) and adding the client's own parts beside them, which call
  only the frameworks' existing entry points and send nothing new to the server. The choice is
  the client-only `ClassicHud` preference (`ac2_ui::hud_style`, key `0x7E000001` in the
  `Misc` section, listed on the options page by its key name); applying it rebuilds the HUD
  (`ClientView::set_hud_style`), whose panels come back from the player as after a relog. The
  world viewer takes `--hud-style modern|classic` and switches with F7. The modern style is the
  default (`ClassicHud` off); classic stays one switch away. Each modern change is listed here.
  Modern changes so far: scaled UI art is filtered (`ac2_ui` `compose_filter`: a box filter where
  it shrinks, bilinear where it grows, mixed in premultiplied alpha), where retail's
  `SurfaceWindow::Blit` took the nearest pixel (classic still does).
  The toolbar is the design's strip and dock (`ac2_ui::modern_toolbar`, art drawn by the client in
  `ac2_ui::modern_art` under its own DIDs `0x7E01xxxx`, composited with a source-over draw mode,
  `compose::DRAW_OVER`, as retail's modes keep the surface's alpha): retail's 798-pixel bar art is
  gone and `ToolbarField` spans the bottom edge without taking the pointer. The strip sits 6
  pixels up, centred in the room left of the inventory drawer, on a dark teal fade: the attack
  buttons side by side in round wells (retail ghosts the start button while the player attacks,
  drawn lit; and the stop button while the player does not, drawn dim), the set chip (arrows over
  and under the set's number, shown always where retail's `UpdateSetNumber` showed it only with
  numbers on), and the shortcut bar restyled as rows of 32-pixel glass wells 3 pixels apart under a 4-pixel band where the top edge's drag
  reaches retail's resize grip (`border_size` 4, so the rows still snap whole), an
  empty slot a faint line until the pointer is over the bar or something is dragged
  (`ac2_ui::modern_strip`). The dock in the bottom-right corner is a teal orb with a pip per panel
  (gold while it shows); the panel buttons fan out of it in round wells on a bevelled backing
  while the pointer is over the dock, or always once the orb is pressed; the dismount and
  consignments buttons stand beside it on their own backings while shown. No band is reserved
  for the strip: a window may be put over it (the design's reserved band, which kept windows off
  the strip, is not ported, at the user's request, as it limits customisation; retail's
  `PrecludeHeight`, which keeps only the targeting reticle and name plates clear of the toolbar,
  stays). Window places are kept per style
  (`WindowPositions-<character>-modern.txt` beside the classic file), and the modern style
  neither uploads its windows' places to the server nor applies the server's saved locations
  (their boxes are the classic layout's), so `gmCWInterface::SetUISaveLocations` runs only in the
  classic style.
  The inventory is the design's drawer (`ac2_ui::modern_inventory`): `RightSidePanel` is laid
  out by the display's edges, 316 pixels wide on the right edge from under the radar to over the
  strip, on bevelled glass. Retail's stone frames (`0x41007118`, `0x41007119`) are gone; the
  header is a bronze fall with the help button at its left and the close button at its right on
  round backings, the gold row is dark glass, the equipment's ten slots stand in two columns of
  five either side of the doll's frame (the 3D viewport fills it, where retail's
  `gmPaperDollUI` sat at its layout offset), and under retail's rail the pack buttons are a row of
  tabs over the page's grid, seven slots wide (`max_columns` 7, retail 6) of 36-pixel slots 4
  apart in glass wells (the backpack's `ItemBackground`, from a restyled copy of the toolbar
  layout so other containers keep retail's slot), the scroll bar at its right drawn thin (10
  pixels: a dark track, a bronze thumb with round ends, small chevron arrows, where retail drew
  15-pixel blue art; the same parts, states and behaviour). The drawer adds
  (`ac2_ui::modern_drawer_bar`) each tab's count of items over slots in the slot numbers' small
  font, a search field at the tabs' right (a muted "search" hint while it is empty) that veils the shown page's slots whose item's name
  lacks the text, filter chips under the tabs (All, Arms, Armor, Jewelry, Other, by the item's
  preferred equip location, the `*_InvLoc` bits, as retail has no item category; the user chose
  these over the design's Use and Parts) that veil the slots the chip leaves out (both view only:
  the slots, their order and drops are unchanged), and a footer naming the page shown with its
  count; retail showed none of these. An All tab after the pack tabs (`ac2_ui::modern_all_view`)
  shows the four pages at once, each under a heading with its slot numbers, in one view the
  wheel scrolls: the pages are retail's containers moved under that view and shown whole, so
  their slots, drops and messages stay retail's; a pack tab (`gmBackpackUI::ShowPack`) leaves it.
  The selected item's slot fans out a ring of round buttons (`ac2_ui::modern_item_ring`), each a
  shortcut to a gesture the item already takes and sending what that gesture sends: Use
  (`UseSelection`, the double-click), Examine (the right press), Split (the backpack's tab-drop
  stack question and move, onto the end of the item's own page, at most all but one), Melt
  (`gmMoneyBagUI::HandleMoneyBagDrop`'s stack question and confirmation), and Equip (`EquipItem`
  at the preferred location) or, for a worn item, Unequip (`UnEquipItem` into the first page with
  room); an action that does not apply is dimmed. Retail had no such ring. No button covers the
  slot the ring opened from (a slot in a short grid, whose bounds would clamp a button onto it,
  has its buttons placed outside the grid, in the drawer), so the double click that uses the item
  still reaches its icon.
  The stack dialog (`gmStackUI`'s `StackableField`) is the design's popover on the drawer's glass
  (`ac2_ui::modern_stack`): its title and the question at the left, a slider beside the amount's
  entry, the error line, then the all / half toggle, the most allowed (`max N`) and OK and Cancel.
  The slider is the client's: dragging it writes the amount into `AmountEntry` as the toggle does
  (`SetAmount`) and typing moves it, so OK still reads the entry (`wcstoul`, base 0) and answers
  with `StackUIDone`; retail had only the entry.
  The vitals are the design's panel (`ac2_ui::modern_vitals`): `StatusbarField` on teal glass,
  retail's stone frame (`0x41003A7D`..`0x41003A80`, the end cap `0x41006018`) gone. The health,
  vigor and focus bars sit in dark wells side by side over the experience bar (the default) or
  stacked over it, the effects under them in as many rows as they take, and the indicators (net,
  PK, vitae) in a column at their right, packed so a hidden one leaves no gap; retail kept them in
  a block left of the bars. A switch at the panel's corner, shown while the pointer is over the
  panel, changes the layout (kept in the HUD layout, `vitals.layout`). The panel is laid out by the
  display's top left (8 pixels in), not a saved window, moved by the Edit layout mode's offset;
  the indicators are drawn 16 x 25. The fields are retail's, moved up to the panel (the
  frame's own fields would clip them), so what they show and their presses stay retail's.
  At 30% health or less the health bar grows, its well pulses red, a LOW tag shows under the
  indicators and the screen's edge glows red (a full-display overlay root whose surface opacity
  pulses); while every vital is full and no PK damage shows, a veil dims the bars until the
  pointer comes over the panel. Retail had neither.
  The radar is the design's (`ac2_ui::modern_radar`): the blips on a dark teal disc 92 pixels
  across in a fine stone ring with the player's arrow at its centre (`RadarCenter` and
  `RadarRadius` set to the disc, so the blips' scale is the disc's; retail's ring was 120 pixels),
  the heading ring the four letters around it (still the turning `HeadingField`), the
  coordinates a pill under the disc, the Tracker eye and the coordinates button in round wells at
  the disc's lower left and right, and a chip at its upper right giving the range in metres
  (retail showed none). The art is restyled in the gameplay layout before the radar is made, as
  `BaseField`'s state changes make its children anew; the places are set again each update. The
  radar is laid out by the display's top right (12 and 8 pixels in), not a saved window, moved by
  the Edit layout mode's offset.
  The chat windows are the design's (`ac2_ui::modern_chat`): each of the four `ChatWindow`s on
  teal glass in a bronze line (gold while active) under a bronze header holding its tab (its
  window key and its name, `ID_Option_Chat1`..`4`), the other windows' toggles, an opacity slider
  (`op`) and the options button (`=`); the main window opens, until the player places it, at
  the display's bottom left over the strip, 370 pixels wide (retail's sat 500 wide at the bottom
  centre, where the strip is); the log on darker glass with the thin scroll bar; a channel
  pill before the entry line (SAY, or the command the line starts with; an addition); retail's
  stone frame, rule and vertical opacity field are gone. The slider is the client's: it sets the
  window's opacity as retail's did (`ChatWindow::SetOpacity`) and saves it on release
  (`SetWindowOpacity`), and the window's glass fades with it as retail's mode 3 frame did
  (`compose::DRAW_OVER_FADE`). The art is restyled in the chat's own copy of the toolbar layout.
  The options button (`=`) opens the window's settings (`ac2_ui::modern_chat_settings`; retail's
  button opened the field with its vertical opacity slider): a name for its tab, a font size step
  over the chat options' per-type sizes (small, medium, large; applied to new lines), whether its
  glass and the log's backing show and the backing's tint, the local time before each new line
  (`chrono`'s clock, `HH:MM` in the design's grey), old lines faded while the window is not in use
  (pointer away, entry idle; lines older than 12 seconds are not drawn), the most lines the log
  keeps (all, as retail, or 100 to 1000; the oldest leave), a lock (a locked window neither moves,
  by its bar or Alt + drag, nor resizes: `UiTree::set_locked`), the window's retail chat options
  (the text types it shows, its filter, and the type its entry line sends, its devoted type: a
  press sets the option on the chat options page and runs the page's Apply, so the server gets
  what the page's Apply sends), Hide window (`ChatWindow::Show(0)`,
  as its key does) and the chat options page. The choices, and the vitals' layout, are kept per
  character on this machine in `HudLayout-<character>.txt` beside the window positions
  (`ac2_ui::hud_layout`). Retail had none of these choices.
  Windows may dock together as tabs (`ac2_ui::modern_chat_tabs`; retail's windows were each their
  own): the header's `+` docks another window into the frame, the window shown carrying a tab per
  docked window (key, name, an unread dot while a hidden one takes lines); a tab shows its window
  in the frame's place (`ChatWindow::Show`, the shown one hidden), a docked window shown by its
  window key takes the frame, and the settings' Pop out / Dock into tabs moves a window out of or
  into the frame. The frame (`chat.group`, `chat.active`) is kept in the HUD layout; each window
  stands alone by default, as retail.
  A log the player scrolled up keeps its place as lines arrive (retail's `AllTextToLog` scrolled to
  the end unless the bar was held) and a pill over its foot counts them (`N new below`); a press
  on it, or scrolling back down, shows the end.
  Each tell also shows for three seconds as a toast in a lane under the vitals, fading at its end
  (`ac2_ui::modern_chat_toasts`; at most three, the oldest leaving), taking no pointer; the tell
  still reaches the windows as retail's did.
  The option panel has an Edit HUD layout button (a copy of the In-Game Wiki button under it;
  `ac2_ui::modern_layout_edit`) that turns on the design's Edit layout mode: a clear shield over
  the display takes every press, each movable part shows a dashed outline and its name, and a drag
  moves it. The parts laid out by the display's edges (vitals, radar, inventory drawer, shortcut
  strip, dock; the toolbar's field grows upward as far as the strip or the dock is raised) keep
  an offset from their place (`UiTree::hud_offset`); the chat windows move as windows. Its bar
  locks the whole HUD (`hud_locked`: no window or part moves or resizes; turning the mode on
  releases it), resets the offsets and the chat windows' choices and places, or closes the mode. The offsets
  and the lock are kept in the HUD layout (`offset.<part>`, `hud.locked`).
  The bar's second row saves the layout under a typed name and loads a saved one (the design's
  presets): a copy of the HUD layout's settings and the chat windows' places under
  `preset.<name>.` in the same file (`HudLayout::save_preset`, `load_preset`); loading applies them
  as at login.
  The loot window is the design's (its variant A, `ac2_ui::modern_loot`): `ExternalContainerField`
  at 214 by 262 with retail's frame (`0x41000D41`) drawn in nine pieces of their own sizes
  (`compose::NINE_SLICE_EDGES`; retail drew the frame at its 138 by 165) over teal glass, the title
  centred, a bronze close button, the list four slots wide (retail three) with the thin scroll
  bar, Loot All and Transmute All as pills holding their art and their names, and a note under
  them. The elements are retail's, so opening, slots, drops and the buttons stay retail's.
  The window opens above its container, its foot 28 pixels over the container's top (retail's
  `PositionWindow` centred it on the container), and a dashed gold leader runs from its foot to
  the container's top, ending in a dot, every frame while both show (`ac2_ui::modern_loot_leader`,
  from the frame's projected boxes); retail drew none.
  While the targeting pointer is on (`RecvNotice_SetTargetingPointer`), a gold banner at the
  display's top centre says to pick a target (`ac2_ui::modern_banner`, the design's variant B);
  retail showed only the pointer's cursor. It takes no pointer; the targeting stays retail's.
  Every other window takes the design's shared parts (`ac2_ui::modern_windows`, its "Other
  windows" component set): retail's shared images are swapped in the toolbar and gameplay layouts
  before the frameworks make their elements, so every element, state and message stays retail's.
  The stone frames (`0x41000970`, `0x410011FB`) become teal glass in a stone bevel under a dark
  stone header (the frame's top 31 rows); the help and close buttons, square dark buttons in a
  stone line; the tabs' caps, tabs in a fine line (gold for the chosen one); the four families of
  blue text buttons, bronze pills (gold when lit, faint when ghosted); the check box, a dark well
  with a gold tick; the expander and drop-down arrows, gold chevrons; the rule, a fine line; the
  wells' eight pieces, dark wells; the dialogs', pets' and fellows' translucent fill and bronze
  border, the glass in the bevel; the secure trade's frame, redrawn with a header over its title,
  its title strip and divider the header's stone and a fine rule; the fellows' health and vigor
  fills, the design's meter gradients, over its dark meter well; the whole-window frames of the
  store, consignments, catalog, new sale, ingredients and book, redrawn at their own sizes with
  their wells where retail's were (retail's light title boxes, which hold dark text, as parchment
  strips; the book's page as parchment); the green, gold and grey button families, pills too;
  the page, spinner and fold arrows, gold chevrons; the list rows, clear, the chosen one in a gold
  ring; the blue scroll bar, the thin one. Retail tiled its header bar
  (`0x410002CD`, `0x410011FC`) down the room under a side panel's window; that room is left
  empty. A window's title (the text, or retail's reference to its shared title text, in the
  header) is drawn in the design's gold (`#d9c27a`; retail's cream).
  Titles are also set in small caps with wide letter spacing (`ac2_ui::text_title`; the client's
  own `modern_title` text attribute, set on the title by `modern_windows` and on the inventory
  drawer's): each word's first letter capital, the rest lower case in the DATs' small-caps family
  (`0x4200000A`, `0x42000009`, `0x4200000B`, `0x4200001A`, 20 to 13 pixels), 0.14 em between
  glyphs, the largest size that fits the title's box and an ellipsis if even the smallest does
  not; retail set the title in the layout's decorative lowercase face ("sKiLLs") at one size. The inventory drawer, the
  loot window and the stack dialog keep their own design.
  The dialogs (`ConfirmationField`, `MessageField`, `WaitField`, `ResurrectField`, made by
  `DialogFactory`) get the design's header (`ac2_ui::modern_dialog`): a dark stone band across
  their top with a gold title centred (CONFIRM, MESSAGE, PLEASE WAIT, RESURRECTION), the body
  text moved down clear of it; retail's dialogs had no title band. The answers and messages are
  retail's.
  The arrows that point the way are drawn by the client (`ac2_ui::modern_art_pointer`): retail
  turned `OffScreenIndicator`'s small bitmap by its surface object's rotation, which the UI
  layer samples nearest, so it showed jagged at most angles. Here each arrow is drawn anti-aliased
  already turned, every 5 degrees, under the client's own DIDs, and the element shows the
  drawing for its angle unturned. The selection's off-screen indicator is a chevron disc (dark
  glass in a gold ring, a double chevron pointing out; `Reticle::set_pointer_art`) and the
  floating arrow (the Tracker's match, a map coordinate) a notched gold delta with a short
  fading trail (`GuideArrow::set_pointer_art`). Where they stand and which way they point stay
  as before.
  The selection's marks are the design's fine brackets (`ac2_ui::modern_target`,
  `modern_art_target`): retail turned one diagonal gem (`0x410016EB`) for each of the reticle's
  four corners and drew the name and its 5-pixel health and vigor meters bare over the scene.
  Here each corner shows its own drawn gold L bracket unturned, the name and meters sit in a dark
  glass card (`ModernSelectionCard`, the client's own root behind them; it counts in the box the
  name plates keep clear of), the meters are 134 pixels wide with an 8-pixel health bar over a
  2-pixel vigor line, and every other object's name plate is a smaller glass pill. Where they
  stand, what shows when and the names' look colours are retail's.
  The fellow mini panel's rows (`ac2_ui::modern_fellow_rows`) are 24 pixels tall with 10-pixel
  health and vigor bars (retail's: 15 tall, 5-pixel bars), their numbers shown from the start
  (retail's mini panel started them hidden until the page's number button), and a fine light rule
  along each row's foot parts the members at a glance.
  The pet panel is the design's, expanded or compact (`ac2_ui::modern_pets`, the user's choice):
  retail's `PetPanel` put its title and the four command buttons in a block over the list of
  pets, at one size; here the title and a fold chevron head the panel, the rows follow in their
  well and the commands stand under them, and the chevron folds the well to the pets listed or
  opens it to retail's height. The choice is kept in the HUD layout (`pets.compact`).
  The seven panel buttons' icons are one matching set in Asheron's Call 1's look (the user's
  choice): each panel's AC1 toolbar symbol (Examination its magnifier, Navigation its compass,
  Option its checklist, Inventory its backpack, Quest its leaf, Skill its flexed arm, Social its
  figures) drawn in AC2's colours, ivory on AC2's bronze orb, with a brighter rollover. Exception to
  "never commit game data", at the user's explicit request: these redrawn icons ship as PNGs in
  `crates/ac2-ui/assets/hud/` (`ac2_ui::ac1_buttons::shipped_icons`). They are not the DATs'
  images but new art made from them; regenerate them with `cargo run -p ac2-ui --example
  bake_panel_icons -- AC2_DAT_DIR AC1_DAT_DIR` (`ac2_assets::ac1_icons` reads AC1's container
  through `ac2_dat`'s retail layout).
- Exception: the modern style's skill panel lays its trees out top down and opens on a page.
  Retail's `UIElement_Tree` for `SkillTree` had no `orient_high`, so `UpdateLayout` put grid row 0
  at the bottom of the paper, grew the tree upward, and `UIElement_Tree::End` scrolled the view
  to the far corner (the bottom, the roots). Retail also loaded a page only when a tab was
  pressed (`gmSkillTreeUI::HandleElement`), so the panel opened empty. Here the modern style
  (`SkillTreeUi::set_modern`) sets `orient_high` on the tree (the retail layout path that puts row 0
  at the top, each parent above its children, the connector lines from a parent's foot to a
  child's head), scrolls to the top after a page loads (`UiTree::tree_view_top`), and the first
  time the panel is shown with nothing loaded asks for the melee page as its tab does
  (`SkillTreeUi::open_first_page`, the same `SetExaminationRequest` the tab makes). The classic
  style keeps retail's bottom-up layout and the empty first show.
- Exception: the radar shows the player's map coordinates under the compass by default.
  Retail's `gmRadarUI::ReloadOptions` opened `BaseField`'s coordinates box only while the options
  profile's `PROFILE_SHOW_RADAR_COORDS` bit (0x10) was set, and a new character's profile has it
  off. Here the box also opens while the player has not hidden it in this client
  (`Client::radar_coords_shown`): the radar's button or a change of the bit records the choice
  per character on this machine (`RadarCoordsHidden-<character>.txt` beside the window
  positions). The bit, the button's `SetShowRadarCoords` call and the profile upload stay retail.
- Exception: holding the right button over empty world looks around. Retail bound the right
  button only to the UI's `SelectRight` (a right click on an entity examines it,
  `gmSelectionUI::Finish3DSelection`) and put mouse look on the middle button
  (`CameraToggleMouseLook` 0x400000BA, a toggle) and a bindable hold action (`gmCameraUI::
  ListenToMessage`); no key map of 7160, 7230 or 0930 binds a right-hold look. Here a right
  press whose selection render picks nothing (the press landed on the world element, not on a
  panel) turns mouse look on (`Controls::start_right_look`, the camera UI's want state) until the
  button comes up; a right click on an entity or item still selects and examines it. Mouse look
  the player already toggled on is left alone.
- Exception: text fields support modern selection and the clipboard: mouse drag and Shift+arrow
  selection, highlighted, with Ctrl+A / Ctrl+C / Ctrl+X / Ctrl+V (the OS clipboard). Password
  fields never copy out. Retail's own text editing rules (max length, enter/escape) still apply.
  Read-only selectable text (chat log, tells, books) takes Ctrl+A / Ctrl+C too, and a drag
  selection auto-scrolls at the text's edge. On macOS the shortcuts are Cmd+A/C/X/V.
- Exception: the mouse wheel scrolls every panel that has a scroll bar (texts, lists, trees, and
  the bar itself), from the element under the pointer up to its first scrollable ancestor. A
  wheel turn over a bar's thumb or arrow buttons reaches the bar; retail's buttons swallowed it.
  Over any other part of a window (its frame, title, tabs, buttons, empty space) the turn scrolls
  that window's content: the nearest enclosing element with a shown scroll owner, and of its
  owners the one level with the pointer, else the largest (`window_wheel.rs`). Over no window the
  turn still zooms the camera; the guide, help and Tracker windows keep their own wheel handling.
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
- Exception: levels 4 and 5 of the graphics quality turn 16x anisotropic filtering on. Retail's
  `Render::SetOverallGraphicsQuality` left `AnisotropicFiltering` off at every level (the
  `QUALITY_PRESETS` table stays as retail has it) and `RenderDeviceD3D::SetStageFilterMode` made a
  LINEAR min or mag filter ANISOTROPIC only while the preference was on, at the caps'
  `MaxAnisotropy`, with a LINEAR mip filter only while `TrilinearFiltering` was on. `apply_quality`
  (`ac2_render::prefs`) sets `AnisotropicFiltering` after the preset at levels 4 and 5, and the
  samplers (`ac2_gpu::TextureFiltering`) clamp at 16 where the adapter has
  `ANISOTROPIC_FILTERING` (retail's caps check). wgpu allows a clamp above 1 only with min, mag
  and mip all linear, so a stage is anisotropic only when both its min and mag filters are linear
  (retail needed either), and an anisotropic stage takes a linear mip even with trilinear off.
- Exception: the near landscape draws its surfaces' detail textures. Every terrain `SurfaceInfo`
  names a detail `RenderMaterial` and its tile width (`detail_texture`, `detail_tiling`; a grey
  around 0.5, e.g. `0x2B000007`, 128 x 128 every 3 m), which `LScapeMaterialInfo::GetCombineMaterials`
  gathered into `MaterialCombiner::detailMaterials` and `CombineMaterials` never drew, so close
  ground showed only its base texture (512 x 512 every 20 m) magnified into a smear. Here each
  layer is modulated by twice its detail texel (a MODULATE2X stage over the layer, so mid grey
  leaves it unchanged), fading out between 25 and 60 m from the eye (`terrain.wgsl`
  `DETAIL_FADE`); a layer whose surface has none binds mid grey. The detail is part of the layer
  set (`TerrainLayer::detail`), as `LScapeMaterialInfo::CornerMatch` already compared it.
- Exception: the scenery reaches farther at quality levels 4 and 5. Retail kept a resident
  ring of one landblock around the camera's (3 x 3 blocks) and culled each static scenery
  object past its Setup's `DegradeInfo` visible distance times the `SceneryDrawDistance` scale
  (0.4, 0.7 or 1; `CPartArray::Draw`), so ridges a few hundred metres out stood bare. At levels 4
  and 5 the client keeps a ring of two (5 x 5 blocks, as the AC2e Unreal remake streams;
  `ac2_render::prefs::near_radius_for_level`, unless `--radius` is given) and multiplies the static
  scenery's distance scale by 2 (`DegradeOptions::scenery_reach`, `scenery_reach_for_level`), so
  those objects draw out to twice retail's distances at their cheapest degrade with retail's fade
  at the new edge; mobile objects keep retail's adaptive scale. Levels 1 to 3 stay retail.
- Exception: grass frills reach past retail's ring. Retail merged the frills within
  `(m_current_lscape_radius + 1) * square_length` of the camera cell (`FrillRender::RefreshFrillMesh`,
  still done) and faded them out between `radius * square_length - 8` and `radius * square_length`
  metres (`FrillRender::RenderFrillMesh`, 12 to 20 m; that fade is not drawn). The client adds the
  loaded blocks' other frills out to 160 m (`ac2_render::frill::select_far_frills`, 0.65 of it at
  the short `LandscapeFrillDistance`), thinned by a fixed per-frill hash from all of them to 30%,
  each growing in and out from its origin in the vertex stage (`grass_scale` in `object.wgsl`).
  Each frill of retail's ring is also drawn `FrillSettings::copies` times (1, 2, 4, 8 or 12 by
  the `LandscapeDrawDistance` choice, so 1 at quality level 1 is retail's count), each copy
  moved up to 1 m on each axis, turned and scaled 0.85 to 1.15 by a fixed hash of its frill's
  position and set on the ground there (`ac2_render::frill::grass_copy`, `scene/grass.rs`).
  Retail drew each DAT frill once. Every frill sways in a wind by the square of its height
  (`grass_wind`), and discards fragments whose alpha is at most 0.2 (`GRASS_ALPHA_CUT`): the
  DAT's grass layers blend with depth writes and an alpha test above 0, so overlapping copies'
  faint fringes hid the blades behind them. While the temporal anti-aliasing is on, a frill's
  alpha between that cut and 0.6 becomes coverage instead (`object_modern.wgsl` `grass_dither`:
  the fragment draws opaque where its alpha beats a per-pixel noise that moves with the jitter's
  phase), so the resolve gives soft, crisp blade edges and distant blades keep their mean
  coverage through the mips; off, the cut alone stands. The frills within 12 m of the camera draw back to
  front among themselves (`FRILL_SORT_DISTANCE`), so a near blade's soft edge no longer keeps
  out the blades and the body behind it. The grass parts around the nearest eight posed bodies
  within 60 m (`frill::grass_pushers`, `grass_push` in `object.wgsl`): the part of a blade within
  0.75 m (times the body's scale) of a body's feet is carried out to that reach and sinks as it
  bends, its base planted. Retail's frills stood still and the bodies walked through them.
  Frills keep retail's lit vertex program for their light (`object_modern.wgsl`
  `frill_retail_colour`): the sun's N.L, the point lights and the ambient summed and clamped in
  display space, times the stage's MODULATE2X albedo, decoded once, where other lit objects sum
  decoded lights in linear light, which left the meadows a quarter darker than retail and the AC2e
  remake drew them.
- Exception: the inventory paper doll frames the whole figure as admin 7148's
  `gmPaperDollUI::UpdatePaperDollPosition` does (avatar at -0.5 of its height, the camera that
  height tall); Legions 7160 framed a head-and-shoulders portrait (-0.875, a quarter).
- Exception: the Tracker (`ac2_macro::tracker`, a modern addition with no retail counterpart;
  Virindi Sense's behaviour). An eyeball button on the radar's edge (a copy of `CoordBoxButton`
  mirrored across the ring, with the `icon_it_blowgunsniper_eyeball` icon) opens the Tracker
  window (`ac2_ui::tracker_ui`): another `Book` of the gameplay layout made as its own root, with a
  pattern field and text buttons copied from the DAT layouts. Its name patterns (`patterns.txt` in
  the config directory's `tracker` folder; an older Vitaeum or Virindi list is imported once) are
  matched against the frame's world objects (`ac2_client::hud_tracker`); a match is announced in
  chat once, listed with its distance and coordinates, and picking it selects it and points the
  floating arrow at it; the radar brackets the matches in green, four pixels out (outside
  retail's selection box), and Enter in the pattern field adds the pattern (the chat does not take
  that Enter). The floating arrow (`ac2_ui::guide_arrow`: a second `OffScreenIndicator` and
  `SelectionName` from the gameplay layout, as overlays) turns at the screen's top towards its
  target relative to the camera's view, labelled with the target's coordinates and distance; a
  press on a map coordinate in read-only text (the chat log; the forms `Position::FromRealMapCoords`
  reads) points it at that place, which clears on arrival. It sends nothing to the server.
  Windows the client makes may draw a frame in nine pieces
  (`ac2_ui::compose::NINE_SLICE`: corners kept, edges and middle stretched) where retail drew a
  frame image only at its own size, stretched or tiled.
- Exception: the launcher (`crates/ac2-launcher`, a modern addition; retail's was the separate
  `ac2launcher.exe`). It works as ThwargLauncher does: a server is a name, a connect string
  (`host:port`) and a type (`emu`: `AC2E` or `Relaunch`), and the launcher builds the client's
  command line for that type and starts it (`docs/server-list.md`); the player never writes one.
  The player's servers (a local AC2e to start with; no community server is built in) and a
  published list share ThwargLauncher's `ServerItem` XML. An AC2E server with a login server
  (`LoginServerUrlKey`) signs in at its AC2e account service (`POST /api/login`: the game's
  ticket); the client takes the ticket from a file (`glsticket FILE`),
  and the launcher waits for that. Passwords are never kept.
  `scripts/ac2e-launcher-test.sh` runs that start against a local AC2e.
- Exception: the intro is skipped. Retail's `gmUIFlow::gmUIFlow` queued `gmIntroUI` first (two
  pictures four seconds each, with its music, before the data patch screen); the client opens the
  data patch (connecting) screen first (`PregameUi::skip_intro`) unless started with `--intro`.
- Exception: the quest guide (`ac2_guide`, a modern addition: the AC2Hero site baked in at build
  time from `AC2HERO_DIR`, nothing of it committed). The option panel's Assistance button
  (`gmOptionUI`, which opened the hidden `AssistanceTab` page for `gmAssistanceUI`'s calls) is
  the In-Game Wiki button: it opens and closes the guide window (`ac2_ui::guide_window`), and the
  assistance page is no longer reached (the servers do not answer its calls). The window is another
  `Book` of the gameplay layout made as its own root, its frame in nine pieces, resized by a copy
  of the chat strip's grip dragging two edges (the DATs' diagonal cursor, a drawn grip mark) and
  kept at its size with its place; its panes scroll by copies of the layouts' vertical scrollbar
  (`0x413DE718`). Tabs list quests (by level band; the search also matches rewards), NPCs,
  monsters, guides and reward items; a page shows its facts, rewards with their item icons, text
  with coloured runs (the book's text area drew one colour), links, map coordinates that point the
  floating arrow, and its pictures, decoded on a worker and dropped with the guide when the window
  closes. Share puts `Title [wiki:slug]` in the chat entry line (the player sends it, plain text
  to every other client); a press on such a token in read-only text opens the guide at that page.
  It sends nothing to the server itself.
- Exception: the navigation panel's map is drawn from the terrain. Retail's `gmNavigationUI`
  showed the painted region art (`CDetailMapDesc`): an overview image and fixed detail tiles picked
  by the overview's buttons, with the map notes (`AddMapNotes`), the server's markers and the
  player marker over them, and a return-to-overview X. Here `ac2_ui::world_map` draws the world
  from the landblocks' terrain (`ac2_world::map_raster`) into the same map area, zooming with the
  wheel (more detail as it zooms: per-block tiles past 4 pixels a cell) and panning by drag; the
  overview buttons, scroll buttons and X are hidden. The map area is taller (289 pixels; the
  panel's lower rows move down) and the time and date share one line. The retail notes are the
  default layers, denser kinds appearing as it zooms, with the server's markers and the player
  marker kept. A layers button in the coordinates row opens a list beside the panel
  (`ac2_ui::world_map_sidecar`, another `Book`, right of the panel or left when the right would
  leave the screen) that toggles each note kind, the AC2Hero guide's places and zone levels
  (`ac2_guide`, baked at build time; zone levels are the levels of the guide's quests starting in
  each 16 x 16 block square, as the game data has none), all off by default, and the painted region
  art as a background, also off. The terrain is read on its own store with an 8 MB free-object
  budget by fixed map threads (`crates/ac2-db/src/loader_pool.rs`), only once the panel is first
  shown. It sends nothing to the server.
- No admin tools in the client. The admin 7148 build is a reference for names and code, not for
  features: do not port its admin-only commands, debuggers, account overrides, admin entities
  (e.g. `wsl_butcherytool`) or admin UI; mark those catalog rows `skip` with the reason.
  Test-only code (`view::probe`) stays behind the `test-support` feature.
- Exception: drawn poses blend between the last two bone refreshes (`ac2_anim::blend_transform`;
  `Client::set_pose_blend`, on by default), so the skeleton moves at display rate. Retail draws the
  last refresh; turn the blend off to get it. Child part groups with their own `AnimMap`
  (`child_part_poses`) blend the same way. Drawn positions (`PhysicsBody::drawing_position`)
  follow retail's render interpolation and are not an exception.
- Exception: objects and bodies draw eye-relative. Retail's device took each part's world matrix
  and the camera's view at the landscape's absolute coordinates (tens of kilometres out), where
  an f32 vertex position rounds to 2 mm or more, so a posed body's vertices stepped by those
  millimetres as its pose moved and the texture crawled over the surface when it stood idle.
  `ObjectRenderer` builds the clip position and the view-space position from each part's origin
  taken off the eye on the CPU (`InstanceData::rel`, eye-relative `view_proj` and `view` in the
  frame uniform) plus the rotated mesh point; the per-pixel lighting and the sun shadow cascades
  take that eye-relative position too: casters and receivers (objects, bodies, near terrain) go
  through each cascade's `Cascade::clip_rel`, whose window is taken off the eye in f64, so shadow
  edges on and from a moving body do not step either. The water reflection's view and plane are
  eye-relative as well. The sky already drew this way.
- Exception: the camera follows the body where it is drawn, every frame. Retail's `SmartBox::UseTime`
  updated the camera every frame only while mouse look was on (`UpdateCamera(elapsed, 0)`), and
  otherwise once per physics quantum (`SmartBox::PlayerPhysicsUpdatedCallback`, then
  `SceneCamera::slerp`), in both cases against the player's current physics position, which steps
  a whole quantum (0.1 s) at a time while the body was drawn interpolated between quanta; running,
  the two jittered against each other by up to a third of a metre, and a turn reached the camera a
  quantum late. Here the camera updates every frame in every mode against the drawn position
  (`Client::update_camera`, `Player::drawn_camera_subject`).
  The collision stays retail's and applies on top of that: `SmartBox::UpdateCamera`'s line-of-sight
  pull-in (`CameraCollision::trace`, `SceneCamera::update_collided`) runs from the drawn player's
  origin where retail used the physics frame's, so the camera and the body it follows stay together.
  The first trace (to the point the camera looks at) starts at the physics position, as retail's
  does, and ends at the drawn body's point (`CameraCollision::set_trace_origin`): the drawn origin
  is interpolated between two quanta and sinks a few centimetres into the ground over a crease
  between terrain cells, and a trace started there stopped at once, dropping the camera onto the
  player for a frame.
  Exception: the camera's traces pass through the player they follow (`LosParameters::
  ignore_instance`, set by `CameraCollision::trace` on the head-to-camera ray and the wall
  stand-off). Retail's traces test every object in the cells with the camera-ethereal set
  (`CPhysicsObj::IsLOSBlocking`), and the player blocks unless its blocking mask has the
  camera-ethereal bit; the player's EntityDesc AC2e names (`0x4700110F`) sets no ethereal
  property, and the ray starts at the shoulder offset (radius 0.785 times 0.7, 0.549 m) at
  0.75 of the height, 0.551 m from the centre of the human Setup's upper sphere (radius 0.555),
  so it began inside the player and the camera was held in the model. Where retail's player
  took the bit is not traced. Every other object and the walls still block.
- Exception: a body's environment collision response does not wait for a collision manager. Retail's
  `CPhysicsObj::handle_all_collisions` returns at once for an object that has no
  `m_pCollisionManager`, which only `CPhysicsObj::SetCollisionState` makes, so such an object never
  reflects its velocity off a wall or tracks its stationary-fall frames. The client runs that
  response (`PhysicsObj::handle_all_collisions`) for every body, so an object no script gave a
  collision state still stops, slides and bounces; the object-table, collide-mask and
  other-object parts that need the manager (`has_collision_manager`) are gated as retail does.
- Exception: the player turns responsively. Retail stepped turns in 0.1 s physics quanta and drew
  the body slerped from the previous quantum to the current one, so a turn key showed up to two
  quanta late; and the DAT's air element has no turn acceleration (`MotionValues` 0x40000002), so
  the body could not turn mid-jump. Here the player's drawn heading is the latest quantum's turned
  on by what the next quantum's `CPhysicsObj::UpdateRotation` will turn over the elapsed part of it
  (the yaw rate and the acceleration the held input picks; `PhysicsBody::led_drawing_position`),
  so a turn shows the frame its key goes down and the next quantum lands where it was drawn; the
  origin still interpolates as retail. In the air the player takes the ground element's turn
  speeds, acceleration and brake (`MotionInterp::turn_in_air`). Other bodies are unchanged, and
  the physics (and the heading reported to the server) still step per quantum as retail.
- Exception: a story hides the gameplay panels. Retail's `gmStoryUI` draws its background and
  story fields over the screen and leaves the panels shown; here the panels leave at the story's
  begin and return at its hide (`HudRequest::StoryGameplay`, `ac2-ui` `frame_passes.rs`
  `story_pass`), so no window sits over the pictures.
- Exception: a notice raised while the player is out of the world waits for the player's return.
  Retail's `RecvNotice_*` receivers (`gmJournalQuestsUI::RecvNotice_UpdateQuests` among them) were
  called the moment the script raised the notice, in portal space too. The HUD's frame passes run
  only in the world (`CPhysicsObj::IsInWorld`), so the notices of the frames between a portal's
  forced move and the arrival are kept (`Hud::parked_notices`) and handed to the first frame back,
  ahead of its own: A Safe Journey's quest update arrives with the portal's move and the journal
  shows it. The shop's string variables are `Tokenize("quantity")`, `"product"`, `"trade"` and
  `"cost"` (`gmShopUI`'s `Token_*`, lower case), the names the UI string table's entries bind.
- Exception: a death flag that comes before the HUD waits for it. Retail's `gmGamePlayUI` existed
  before the login's `WM_Death` 0x33 (`CDeathSystem` 0x3 -> `CM_Character::SendNotice_UpdateDeathState`)
  reached `RecvNotice_UpdateDeathState`; here the HUD of a new session is built on workers and can
  come after it, so a character that enters the world dead found no dialogs and neither raised the
  resurrect dialog (`MakeResurrectDialog`) nor hid the gameplay panels (`gmGamePlayUI::Show(0)`).
  `DeathPanel::hold` keeps the latest flag and `death_notices` raises it when the dialogs exist
  (a newer flag in the same frame wins). Movement while dead is not a client rule: the keys stop
  through `PlayerMovementModule::LockActions`, set by a behaviour whose params lock actions; AC2e
  policy (no capture of the retail server is known) sends the player `DoBehavior` death into the
  dead mode with the lock at death and at entering dead, and `SetMode` peace plus `StopBehavior`
  death at the respawn.
- Exception: a book's text after its last `<BR>` is a page. Retail's `gmBookUI::ParseText` stops
  at the last `<BR>` and drops anything after it; here that tail is paginated too
  (`book.rs` `segments`), so a book without a closing `<BR>` is not shown empty.
- Exception: an open store is left as the session ends. Retail's store panel destructors
  (`gmShopUI::~gmShopUI`, `gmShopConsignmentUI::~gmShopConsignmentUI`: `RequestLeaveStore`;
  `gmShopCatalogUI::~gmShopCatalogUI`: `RequestLeaveCatalog`) called the player's script whenever
  the UI was torn down with the player present; here the log off and the quit ask first, once
  each and only while such a panel is open (`ac2-ui` `hud_tick.rs` `leave_open_stores`).
- Exception: a placement keeps a pending mode change and its cell. Retail's `CPhysicsObj::SetPlacementPosition`
  sets the mode tracker to `GetMode()` (the mode reached so far), which only matches the requested
  mode because retail places an object before its description's `DoMode`; here the object enters
  the world later, once its cells are resident, so the tracker is set to its desired mode
  (`PhysicsObj::set_placement_position`) and a door created shut is not reset to its default mode.
  Retail also passes the requested position, whose cell `AdjustPosition` may have corrected; when
  the body ended in another cell, the previous position takes the body's own position, so a drawn
  position never names a cell the body is not in.
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
- The DAT set is the 7230 release (newer than the Legions DVD's 7160 data): `AC2_DAT_DIR` is
  `../dataset-7230` in the analysis workspace. Tests that pin record counts pin per release
  (`test_support::pinned(legions, r7230, r0930)`). Porting still follows the Legions 7160 code
  (below); where 7230 changed a packed layout or added script calls, follow the 7230 executable
  (02.00.20.7230, from the archive's 2012 client package; no symbols, so cite its addresses). The
  layouts that differ (so far `SkillUINode`'s perk word) follow `ac2_core::data_release`, which
  `DatDataset::open` sets from the portal iteration (7230 is 5064; the
  DVD's 4862). The end-of-retail set of client build 0930 (`../dataset-0930`, iteration 137, below
  1000) is readable as `Release0930`, which reads 7230's layouts (`SkillUINode::StreamPack`
  0930 0x005D5780); its script library numbers some native types and efuns differently
  (`ac2_core::native_type_from_release`, `ac2-script` `vm/efun_ids.rs`).
- Live server testing is against the AC2e server (github.com/Thwargle/AC2e, run on the 7230
  DATs, which it serves as its 7230 profile). The client logs on as the 7230 client does (data
  version `7230`, `ac2_net::RELEASE_7230_VERSION`; `ac2_net::default_version` follows the mounted
  DATs, so the DVD's give `7160`). `scripts/ac2e-login-test.sh` starts a server, logs on through
  `ac2-login-probe` to the character set and stops it; `scripts/ac2e-server.sh` keeps one running
  for the client (`ac2 --server 127.0.0.1:9000 --account local`). The in-process
  `legions-server-double` stays the 7160 path's test peer.
- Effects (buffs and debuffs): the client is retail's. The server adds a timed effect to the player
  with `WM_Effect` 0x17 (an `EffectRecord` and its `Effect` state) and removes it with 0x16; the
  status bar's strip (`gmEffectsUI::Init`) lists it in order of expiry with its time meter running
  down on the global clock, and a press examines it (`gmEffectsUI::ListenToMessage` ->
  `SetExaminationRequest` type `0x40000001`): the request is not the server's, the player's script
  builds the profile (`CExaminationSystem`; name, description and the end time as a countdown,
  `gmExaminationUI::ProcessNode` type `0x40000008`, whose value is `TimeDisplay::Init`'s
  `m_endTime`), so `Client::examine_request` hands such a request, and `StartExamination` /
  `StopExamination` with it, to the VM beside the session's stand-in. What AC2e does with effects
  (a recast replaces the record, different skills stack, death takes them off, a creature's burn is
  text in its examination, a creature uses its harmful entity-record skill in place of a swing and
  then every 45 s, a logged-off player keeps its effects, and the server tells the client of them with `WM_Effect` 0x17 after the login, whose `EffectRegistry` (7230: 14 slots, record hash in slot 4, order hash in slot 11) is empty, since the registry's own add builds the indexes in slots 0 and 12 that its remove and `UpdateEffect` notice go by)
  is its own policy, not retail's. What the server puts in the `EffectRecord` (package 0x14E) is
  what the client's script reads: slots 2-3 the cast time, 6 the duration, 8 the caster's level in
  the skill (the examination's "Spellcraft", and the level the effect's stat tables are read at,
  so the skill panel's enchanted level moves by it: `InqSkillLevel`), 14 the `Effect` state, 20
  the flags (0x2 timed, 0x400 pulsed; a pulsed effect has the strip's `pulsed` state and no
  draining meter). AC2e flags health and vigor changes over time as pulsed, because the DAT
  `Effect` states of exactly those types carry flag bit 0x20000000 (an inference: no retail text
  says what the server set). `scripts/ac2e-live-test.sh` runs
  `ac2e_live_effects` (the creature test needs `AC2E_LIVE_THRASHER=1`, a Banderling Thrasher placed beside the start through `AC2E_EXTRA_WORLD`) against a server whose timed effects last `AC2E_DEV_EFFECT_SCALE` (0.1) of
  their record's time and whose new characters start with 2,000,000 experience.
- Story scenes (the Quest panel's Story page) are retail's client path, and AC2e's trigger is its own
  policy. Retail: the server calls `WM_Quest` 0x23 -> `CPlayer` 0x2A -> `CQuestSystem::UpdateStoryQuest_Done`
  with `(errorCode, bAdd, sceneId)` (stack order: the script branches on the second on add or
  remove and hands the third to `ActRegistry::AddSceneID`; errors 0xFA7, 0xFA8, 0xFA0 print the
  script's failure text), which finds the scene's act through the `MasterActList` singleton
  (`WState` 70001a89, a subclass of `MasterSceneList`; the base's own singleton 700001a5 is empty and
  unused), adds it to the player's `ActRegistry` (the login's `WM_Player` 0x27 argument 2) and
  raises `CM_Quest::SendNotice_UpdateStoryQuests`; `QuestSystem::InqActInfoList` then marks a
  scene playable only if the registry holds it (`ggmSceneInfo::SetPlayable`). The act and scene
  content (names, keyframes) is in the acts' `WState`s in the DATs. A scene row plays on the
  double click (`0x40000007`); the skip key finishes the story and `HandlePlayScenesDone` sends
  the played list (`WM_Quest` 0x22, 0x82130013). Not in the client or the data: what made the
  retail server grant a scene (`Eff_BestowScene*` carry a scene id and stat 4200
  `Quest_BestowedSceneID` exists; no script or AC2Hero page says when) and what it did with a
  played list. AC2e policy (`emulator/src/story.rs`): a character with no scene earned is
  granted the three scenes of the Prologue act (0x40000008) when it first enters the world; the
  login sends the registry empty, as for effects, and each earned scene is granted again after
  it; the played list is accepted and not acted on (AC2e decodes only that it is non-empty).
  Live: `ac2e_live_story`; double: `story_scenes_7160`. Both click the Story tab first, as a player
  does: the Story page is a hidden page of the Quest panel until then, and `UIRegion::NotifyMouseMove`
  (0x005e25c0) counts an element under the pointer only while it and its parents are shown, so
  before the tab only a leaf of the visible page (element 0x6a170865) sat under the list's box (the
  port matches retail here, `ac2-ui` `mouse_tests`). The double click then plays the scene
  (`0x40000007`) and the skip key (`0x4000001E`) ends it. `ac2e_live_story` proves the play step live.
  Exception: the Story page lists the scenes when the first frame in the world runs
  (`Hud::init_pass`). Retail's `gmStoryQuestsUI` is made before the login's `WM_Player` 0x27 raises
  `CM_Quest::SendNotice_UpdateStoryQuests`, so it receives that notice; the HUD here is built on
  workers and can come after it, and the notice then finds no panel (as for the journal's quest
  list). `init_pass` raises the list notice (code 0x40000002) itself, as `RecvNotice_UpdateStoryQuests`
  would, and the global loop asks the script for `InqActInfoList`.
- The inventory requests are the player's script's, as in retail. `gmCWInterface::MoveItem`,
  `EquipItem`, `UnEquipItem`, `LootAllFromContainer` and `TransmuteAllFromContainer` call their
  IFuns on the player's process (`ggmCWInterface::CallIFun_MoveItem` pushes each id as two words
  tagged 0x84 and each number tagged 4, and the `CContainer` / `CInventorySystem` code builds the
  `InvMoveDesc` / `InvEquipDesc` / `InvTakeAllDesc` and sends it: `ac2_client::inventory_commands`);
  `UpdateOpen` goes to the container's own process with the player as the updater and, as the
  wlib reads it, a nonzero argument closing (`gmExternalContainerUI::ListenToMessage` passes 0
  when its window shows). The pack panel lists what the script lists
  (`gmCWInterface::GetContentsInfo`); there is no mirror fallback. The packets are what the VM
  builds. Two points differ from a reading of the retail client's code:
  - Exception: a request leaves with its reference ids counted from 1 (`ac2_script::wire::
    as_collected`, applied to what `gCWInterface::SendServerEvt` sends). Retail wrote the sender's
    reference-table indices (`ReferenceTable::PackReferenceArray` packs the id array as it is), and
    its table hands out the lowest free index after each collection
    (`ReferenceTable::FinishGarbageCollection` resets `m_highWater` to 0,
    `FindNextReference` searches from 1), so a request made soon after one is numbered from 1; this
    port never collects, so its indices only grow (some 26,000 by the time the player acts). The
    receiver only maps the ids to its own (`UnPackReferenceArray`, `UpdateReferences`).
  - Exception: a drop with nothing picked under the pointer names the landblock the player stands
    in as its target (`ac2_client::view::hud_api`). Retail's selection render named the object
    under the pointer (`gm3DItemsUI::FinishFind3DTarget`), and the wlib turns a static object
    into its landblock (`LandDefs::InqLandblockIIDFromStaticObjectIID`); this port has no terrain
    or static-object pick yet.
  AC2e checks these requests against packets captured from clients whose scripts differ from the
  7230 wlib the DATs carry (ids 1; 0x82 on ids that `CallIFun_*` pushes tagged 0x84; icon,
  definition and physical typed 4; a ground drop typed 0 with the slots -1): it compares tags up to
  the integer kind and accepts a drop's move type 33 and slot 0
  (`inventory_request::tags_agree`).
- Exception: an object's class defaults arrive frames after its `CreateObject`, and the panels
  refresh then. Retail's `CBaseQualities::InitDefaults` read them synchronously when the object was
  made, so `ItemDisplay::Update` (backpack, `gmPaperDollUI`) never saw an item without its icon,
  name or locations. Here the loader thread reads them (`Request::ScriptState`), a pack or worn
  item drawn before they land had icon 0 (the "?" slot) and nothing redrew it. `Client::take_loaded`
  raises `GameplayNotice::ItemUpdate` for the object when its defaults are set, which the slots take
  as `RecvNotice_ItemUpdate`. `ac2e_live_item_icons` proves it with frames that do not wait for the
  loader (`LiveRig::new_unsettled`); the other live files settle the loads each frame and cannot see
  the race.
- Exception: an item's icon is its visual's total icon description, read on a loader thread. Retail's
  `ItemDisplay::Update` draws `VisualDesc::GetTotalIconDesc` of the object's local visual desc (the
  parent DAT chain's icon layers with the local ones on top) and only falls back to the `ICON`
  quality (`gmCWInterface::GetIconID`) when it has no layers. The tools and the starter apparel
  have `ICON` = `default_icon.tga` (0x4100034b, the "?") and carry the real icon as four layers in
  their `VisualDesc` (e.g. 0x1f00058e: 0x41005961, 0x410048b4, 0x410048b5, 0x41004314).
  `Request::TotalIcon` reads them (`ac2_assets::ac2_total_icon_desc`), `Client::total_icons` keeps
  them, `inventory_frame` puts them under the wire's local layers, and `ItemUpdate` redraws the slot.
- Exception: a craft query that waits for DAT data is asked again by its panel. Retail's
  `gmCWInterface::InqCraftSkillList` / `InqRecipeList` (and the other craft queries) ran the
  player's script, which read the craft records from the DATs synchronously. Here the frame thread
  never reads, so a query whose script stops for a record not yet resident (`DispatchOutcome::data_wait`)
  answers nothing at first; the host notes it (`ScriptState::waited_queries`) and, once the reads
  land and the waiting contexts have run, raises `CM_Craft::SendNotice_UpdateCraftPanel`
  (`gmCraftUI::RecvNotice_UpdateCraftPanel` -> `RefreshCraftPanel`), so the craft panel lists its
  rows as soon as the data is there (`script_data_loaded`).
- Tradeskills against AC2e (live: `ac2e_live_tradeskills*`, `ac2e_live_traders`): what the data
  settles, what AC2Hero settles and what is AC2e policy. Ranking: client and DATs, then the
  AC2Hero mirror (a 2013 player guide, untrusted; pages cited below), then AC2e policy.
  Data: a mote stack holds 1,000,000 (entity property `0x41000089`, description word `0x105`), a
  recipe slot takes one item that holds the slot's whole quantity (the client's `CheckIngredient`),
  so a 500 or 2500 refining is filled by one merged stack; coal and flawless pyreal are sold by
  the raw-material traders (`store_cragstone`, `store_ikeras`, `store_linvaktukal`: coal at 0.5;
  `store_cavendo`: pyreal at 50; `store_hakata` deruwood; `store_ondekodo` obsidian), and the
  WState `700005ec` is a craft trait of the pyreal mote, not a loot table. The 67
  `*butcheryprofile.wc` records are identical and empty, so what a creature gives is AC2Hero's.
  AC2Hero rules AC2e applies (each with an AC2e unit test, and the chain/butchery/spellbinding
  live files): `butcher.html` (a corpse eight or more levels below the butcher is refused),
  `traits.html` (the table of the two traits each creature family gives, keyed by the
  creature's `*butcheryprofile.wc` record: `butchery::LISTED`, ten units each; the training Idol
  Spawns give sinew and tallow, a Vermin Pilferer bone and hide), `refining.html` (a refining
  recipe returns 34 to 100 percent of its ingredients by the refining rank, 34 + 66 (rank - 1) /
  149; the Crucible and the Refiner raise it by amounts no page or DAT gives, so they are not
  modelled; the DAT already makes a Refining Workshop a requirement), `survey.html` (a mining strike
  adds up to 40 units at Surveying rank 150 to the deposit's ten; even steps, 40 rank / 150),
  `resource.html` (the traders' bundles equal the DAT's per-unit prices: coal 0.5, the other
  commodities 50, so 200 coal = 100 gp ... 50 pyreal = 2500 gp; the purchase dialog sells any
  count, a unit test checks every bundle; the traders file buys the 50-pyreal bundle for 2500).
  Not applied, with the reason: knife and tool durability (`butcher.html`: tools wear out and
  vanish; the starter tools carry `Durability_MaxLevel` 618 = 50 in the DAT, but
  `Usage_DurabilityLostOnUse` 325 is absent on them and no page gives the wear per use, so the
  rate would be invented), the better knives' bonus items (no count anywhere),
  `cartomancer.html` and `workshops.html` (AC2e's profile carries neither the cartomancers nor
  the workshops; the map travel stays the `@maptravel` test knob).
  Known AC2e gap, not diagnosed: a stack merge made after a log-in, of stacks created before it,
  is refused with `InvalidValue` (the chain file merges before it visits the traders).
  Policy still without a record: automatic rank-up, ten units a deposit strike before the bonus,
  certain success, one-time butchery of a body.
  Names: an item AC2e creates after the login carries its name in the `CreateObject`
  description (`gmWeenieDesc` data-mask bit 4, `Name_StringInfoStat`; AC2e `item_names.rs`, the
  entity's `Name` property `0x41000143`, else the name its qualities record embeds). Without it
  the client reads the name from the class defaults (`0x81000000 | definition`) only after its
  data load, and a pack slot created before that keeps "Error! Name_StringInfoStat missing";
  items in the pack at login do not show it. The live tests assert the crafted sword's, bracket's
  and charm's names. The knife is used as a player does: a double-click arms the targeting
  pointer (the client reads `Usage_NumberOfEntitiesUsedWith` 720, `Usage_InventoryRequired` 305 and
  `Usage_ValidTargetTypes` 722 = 8 from the DAT's defaults, nothing from AC2e), and the click on
  the body completes the use only if the body's `Usage_TargetType` (721) shares a bit with 722: a
  creature's DAT default is 4, so AC2e sends 8 on a creature at zero health
  (`butchery::BUTCHERABLE_TARGET`). Spellbinding: `rcp_spellbinding_practice` (wood 1, runewood 35,
  bone 10) makes one of the six `wsl_tradegoods_spellbinding_N` charms; `rcp_spellbinding_study`
  names no output and the DAT calls it "OBSOLETE - DNT: ... crafting and discarding sample totems.
  You will earn no profits", so AC2e uses its ingredients up for the experience and makes no item
  (`tradeskills::is_study`). Buying coal after mining works at any pack size; the earlier failure
  was the test server's `@maptravel` (a loopback test knob), which a second travel on one login does
  not carry out, so the tests log in again before each trader.
- Exception: GUN (the MSN Gaming Zone service: friends, ignore list, tells, room chat, tickets,
  support calls). Retail loaded Microsoft's `GunDll.dll` with `LoadLibraryA(-gunclientdll or
  "GunDll.dll")`, called its `DllGetClassObject`, `QueryInterface`d ten `IGun*` interfaces from the
  one `CGun` COM object (`NSTurbineGun::TurbineGun::Init`, Legions 0x005C6DA0), made `TurbineGun`
  with the global `S_pTurbineGun`, and ran `StartupGUN` as two blocking loops inside
  `gmClient::Connect`. The client has no DLL: `ac2_gun::gundll::CGun` (over `ac2_gun::dplay8`,
  DirectPlay 8 as `dpnet.dll`/`dpnwsock.dll` speak it) is the object `ac2_client::gun_env::
  OwnGunLoader` hands `TurbineGun::Init` as the "module" whatever the path, so `-gunclientdll` is
  parsed and ignored, and a module without `DllGetClassObject` is a false `Init` where retail
  went on with a NULL object and crashed. GUN is off exactly when `-gunserver` is empty (no
  default server, no switch), as in retail. Differences, all of them:
  (1) `TurbineGun::new` takes `GetUserDefaultLCID()` from the caller. `gun_env::default_lcid`
  maps the user's locale name (`sys-locale`: on Windows `GetUserDefaultLocaleName`, which names
  the locale `GetUserDefaultLCID` numbers, one to one; elsewhere `LC_ALL`/`LC_MESSAGES`/`LANG`)
  through Microsoft's [MS-LCID] section 2.2 table (version 16.0,
  `learn.microsoft.com/en-us/openspecs/windows_protocols/ms-lcid/63d3d639-7fd2-4afb-abbe-0d5b5551eef8`),
  mapping only the rows it lists (the table's neutral language ids when only a language is
  known), and an unmapped or `C` locale to 0x0409 en-US, the language of the shipped
  `local_English.dat`. Off Windows the POSIX UI-language chain is only the nearest analogue of
  the regional-format locale retail used.
  (2) A request's `void*` context is a `Context` id into `TurbineGun`'s map of `Rc` handlers
  instead of a pointer; listeners and contexts are `Rc`s and the ten interface pointers one
  `Rc<dyn GunObject>`; the statics (`S_pTurbineGun`, `S_fUseTurbineChatInsteadOfGUN`,
  `G_pGunTicketHandler`, the player interface) are per-thread (`ac2_weenie::gun::shared`), and
  `S_pTurbineGun` is cleared when the object goes (retail leaves it dangling). `Drop` releases
  everything, including the `m_pGunSupport` reference retail leaks, and unloads the module
  (retail never `FreeLibrary`s).
  (3) The Win32/COM seams are traits: `GunLoader`/`GunModule`/`ClassFactory`, `Dp8Factory`, the
  `GunRegistry` (retail's `HKCU\Software\Microsoft\Microsoft Games` values `LaunchTicketData`,
  `LaunchTicketSize`, `Email` are files in `<config>/AC2/MicrosoftGames`; `Email` and
  `LaunchTicketData` are binary values), the random source (the operating system's for
  `CryptGenRandom`: there is no CryptoAPI provider or container, the key exchange is the `rsa`,
  MD5 and RC4 crates' work), and the local-port probe. The registry read retries as the
  decompile does; DP8's adapter order falls back as `dplay8` documents; every DP8 send failure
  maps to 0xC1120405; `CGun::DoWork`'s return value is not used; a hostname is cut at 63
  characters (`MultiByteToWideChar` into 0x40); NULL string members of a message reach the
  client as empty strings; the thread pool is per `CGun` slot, not a process singleton.
  The DP8 thread pool is made by the first `Connect` (the handler constructor `FUN_10035ae0` runs
  `FUN_10003810`), not by `Init`, and a failed creation is tried again by every later `Connect`.
  `Disconnect` is retail's order (`FUN_10003ae0`): the DP8 client is reset first (`Close`,
  re-create, `Initialize`, a synthetic `USERCANCEL` terminate written into Q1), then Q1 is drained
  with the shutting-down flag, which the terminate slot ignores, so the next `DispatchQueue`
  raises a `DisconnectEv` (0x1301), then the service object's destructor resets and closes again
  (three `Close` calls); a `Disconnect` made while Q1 drains is deferred until the drain ends. A
  failed `CONNECT_COMPLETE` or a `TERMINATE_SESSION` destroys the same object from its own sink
  slot. Control frames of type 0 call `GetConnectionInfo` and type 1 reset the keep-alive
  (`0xFFFFFFFF` or 15000) as retail does; their sinks are stubs, and the elapsed-time doubles
  `FUN_1003da50` keeps are not modelled. `GetTicket` after a failed `LaunchTicketData` read: retail
  keeps a half-filled cache (the next call encrypts uninitialised bytes and answers `S_OK`), the
  port reads the registry again. The private interfaces `IGunUnknown_eeeeee28` and
  `b0a7e593` (`QueryInterface`), the queue's event handle (`FUN_10039780`, never signalled) and
  the thread pool's `Close` wrapper (0x1003b620, no caller) are not answered or ported. The
  `CONNECT_COMPLETE` reply-data replay (`reply.vtable[0]`) stays a no-op.
  (4) Crash paths of retail are defined: list rows (15 classes: a row missing an attribute leaves
  retail a NULL array and a stale count), a `HostData`/`MHostData` `Item` without `ItemId` or `Num`
  (retail's render dereferences the NULL block; the port prints the empty block), events with `m_bInited` 0 (retail reads
  uninitialised stack), a NULL ticket or context, a ticket shorter than 0xCC, a NULL extra blob in
  `HearTell`, and the 0xFFFE/0xFFFF send-length wrap are handled instead of read through; a
  control frame of type 2..998 is dropped where retail hangs. A tell or room event
  without `Extra` (retail's `HearTell`/`HearFromRoom` dereference NULL) is taken as no metas; a
  `PresentTicket` ticket under 0xCC bytes (retail reads the two words that decide "encrypt again"
  past its heap block) reads them as zero, so it is encrypted (a ticket `GetTicket` already
  encrypted goes out as its plaintext); a result whose `Cx` matches no pending context is ignored
  where retail would use or release a pointer.
  (5) `GetNews` and `CreateGame` are not ported (`E_NOTIMPL`; the client never calls them).
  (6) Pumping: retail's `CheckForMessages` (`gmClient::UseTime`, every frame) drives
  `CGun::DispatchQueue`, which calls `DoWork` on the DP8 thread pool; DP8's own timer wheel ran
  every 4 ms on its worker threads, here its timers advance only when `DispatchQueue` runs, so
  their granularity is the frame time (no separate pump thread).
  (7) `StartupGUN`'s two wait loops (no timeout, `KeepUIAlive` + `CheckForMessages`) are
  `GunClient::connect`, one iteration per frame while the pregame screen runs; the game logon is
  held (`OnlineSession::connect_held`) and starts, with `InitAuth`'s credential, when it answers
  (`gmClient::Connect`'s order). A failed start stays failed: the `NetError` goes to the
  connecting screen once and the logon never starts, as `Connect` returns before
  `Turbine::Client::Connect`; a false `InitAuth` is `ID_ConnectionError_Generic` (Legions
  `DAT_00AF596C/70`). Retail's `UseTime` poll does not run while the loops spin and does not
  here. `ui_alive` (`RealClient::KeepUIAlive`) is always true: closing the window ends the
  process, so the `!KeepUIAlive` failure branches cannot be reached.
  (8) What the `CCli*` handlers do inside `DispatchQueue` is split. The `CM_UI` notices that
  have a gmClient receiver (`GunReconnected` 0x46B, `GunAuthenticated` 0x46C) run at once, so
  the re-authentication happens inside `DispatchQueue` as in retail. The calls into the player's
  script (`Result_*`, `GUNEvent_*`, `HearTell`, `GUNEvent_HearFromRoom`,
  `GUNHandleReauthentication`) and the lines of `gmUIFlow`/`gmClient` are recorded
  (`GunOutput`) and carried out right after `CheckForMessages` returns, in order, in the same
  frame. Retail makes them synchronously from inside the handler; the handler is reached from
  `Client::use_time` holding `&mut Client`, and running the VM needs that whole client (session,
  script state, loader), so a handler cannot be given it without `unsafe` (the crate forbids
  it) or sharing every one of those through `Rc<RefCell<..>>`. `GunConnected`
  (0x469, no handler) and the `ConnectionLost` notice are recorded likewise.
  (9) `RequestGUN_SendToRoom` with Turbine chat (`CharacterSet.m_fUseTurbineChat` or
  `-tchat`; the client no longer forces Turbine chat on, as retail does not) records the
  `CSendToTurbineRoomByID` call (the chat client is the session's) and returns 1 when the chat
  client exists (in the world) and 0 otherwise; the session's send cannot fail. The context is
  kept and answered by the chat reply (`OnSendToRoomByIDResult` -> `Result_SendToRoom`). With no
  `TurbineGun` and no Turbine chat it returns a non-zero stand-in for retail's `arg2` pointer.
  (10) Options: `-gunport` is `sscanf("%hu")` as retail's type word 0x62 (decimal, no digits is
  an error). `-tchat` (also `Client+0xB0`) and `-tchatdll` (`Client+0xB4`) are
  `Turbine::Client` options; `-tchatdll` is stored but there is no chat DLL to load (Turbine chat
  is built in). `gmClient::Init`'s registry billing-migration check (`CheckForBillingMigration`
  against `-migrationurl` with `-migration1Timeout`/`-migration2Timeout`) is not ported
  (Turbine's billing service is gone); the options are parsed and kept. The local GUN port is
  `Preferences::InqBool(0x4000000F, 0x41000001)` read from the user's preferences file, absent
  being 0. `gmClient::RecvNotice_CharacterSet`'s trailing virtual call (`+0x9C`, after it sets
  `S_fUseTurbineChatInsteadOfGUN`) is not modelled.
  (11) Scripts' `gTurbineGunUtils::GetWorldName`/`GetWorldSuffix` read `S_pTurbineGun`'s names
  as retail's do, so without `-gunserver` (the default) they are empty strings (the client's own
  `world_name` still feeds the assistance page). The GUN failure strings
  (`ID_DataPatch_GUNFailed`, `ID_DataPatch_GUNAccountBanned`, table 0x41000018) reach the
  connecting screen as a `NetError::Server`; the lines of `gmClient`/`gmUIFlow`
  (`ID_GunReAuthenticationFailed`, `ID_CHAR_ERROR_GUN_SERVER_DOWN`/`_LOST`) are shown as chat
  lines (the retail UI sink is unresolved, and the flow state "data patch" is read as "the logon
  has not connected"). `ID_GunReAuthenticationFailed` is preferred when it is in the DATs
  (`StringInfo::IsOnDisk`).
- Exception: `/saveui [name]` and `/loadui [name]` come from the 2013 relaunch client (build 0930)
  and are added to this 7230 client natively; 7230's player script (`CPlayer` 0x121) and DATs
  have neither. Retail 0930 registered them in the player script's command registry (the
  function that registers `/loc`; `AddShellCmdByStringID` 0x7f003c / `AddShellHelpByStringID`
  0x7f003e, commands table 0x250007c7): `saveui` (string 0xAEA028AE) -> `CPlayer` 0x1F5, help
  0x1F6; `loadui` (0x6B60A95F) -> 0x1F8, help 0x1F7. Those ids mean other functions in 7230's
  wlib, so the commands use their own handler ids (`ac2_weenie::chat_handler::SAVE_UI` /
  `LOAD_UI`, outside the `CPlayer` range) and `ChatCommands::register_client_commands` adds the
  names and usage texts ("usage: /saveui < filename > (filename is optional)"). `saveautoui`
  and `loadautoui` are in 0930's table but never registered; not ported. The handlers take the
  name from one argument or use "UI-Default" (script literal 998), show "Saving UI to file
  <name>.txt." (string 0xC109BBD2) or, for `/loadui <name>`, "Loading UI from file <name>.txt."
  (0x6F243A7C) with display type 8, and run efun 0x85001C (save, notice 0x4A5) or 0x85001D
  (load); more than one argument shows the usage. Turbine's bug is kept: `/loadui` with no
  name prints the saving message. The listener is `gmGameplayUI` 0x442540 (path: `"%s%s.txt"`,
  directory and name; the unreachable empty-name form `"%sUI-%s-%s-%d-%d.txt"` is not
  ported), 0x443030 (save) and 0x442D40 (load), in `ac2_ui::ui_layout`. Save writes, for
  SocialField, QuestField, SkillField, NavigationField, ExaminationField and ChatWindow1..4 in
  that order, `"<SOCIA> X:%d Y: %d W: %d H: %d\n"` (tags <SOCIA> <QUEST> <SKILL> <NAVIG>
  <EXAMI> <CHAT1>..<CHAT4>) with `UIRegion::GetScreenX0` / `GetScreenY0` and the box width and
  height (x1 - x0 + 1, y1 - y0 + 1). Load reads each line by `sscanf("%s X:%d Y: %d W: %d H:
  %d")`, skips a line that does not give 5 values or whose tag is none of the nine, then
  `UIElement::ResizeTo(W, H)` and `UIElement::MoveTo(X, Y)`. Differences: (1) retail wrote and
  read beside the game's working folder; the client keeps the files in the `ui` folder of the
  player's AC2 directory (`ac2_prefs::paths::config_dir`, where the window positions live), and
  the name is made safe so it cannot leave that folder (path separators, drive colons, wildcard
  and control characters become `_`, leading dots too, so no `..`). (2) Retail looked each
  window up with no null check; a window the HUD does not have is skipped. (3) Retail's
  `MoveTo(X, Y)` of a window inside a panel took the saved screen place as the parent-relative
  box (`UIRegion::MoveTo` sets `m_box` in the parent's coordinates, `GetScreenX0` adds the
  parents), so a load moved such a field off by its panel's place; in 7230's HUD the social,
  quest, skill, navigation and examination fields are such children. The client moves a child
  to the screen place it was saved at; a root window is moved as retail did. (4) After a load
  the windows' places are recorded as a finished drag records them
  (`UiTree::record_window_positions`, and the location upload of a window that saves its
  location), so the per-character window positions keep them across a relog. A file error goes
  to stderr (retail printed nothing).
  Beyond 0930's nine lines the file holds the whole HUD layout of the style that wrote it
  (`ac2_ui::ui_layout_text`, `Hud::save_ui_layout` / `load_ui_layout`), in lines 0930's
  `sscanf` skips: `<STYLE> modern|classic`; a `<WIN> <layout>:<element> X:%d Y: %d W: %d H: %d`
  line for every other movable top-level window (ids in hex, the `Tokenize` hash of its element
  name, as the tree keeps no names; the pinned parts of the modern style are left out, their
  place being their offset); and, in the modern style, a `<SET> key=value` line for every `HudLayout`
  value an Edit layout preset saves (offsets, lock, vitals layout, chat settings, tabs and
  pets mode, not the stored presets), the value escaping backslash and line breaks. A load of a file of
  this style applies the settings as a preset load does (`HudLayout::replace_settings`, then
  each part's `apply_layout`), then the other windows, then the nine (fields inside a panel
  follow the panel), and keeps them like a drag and a preset load (window places recorded,
  `HudLayout` marked for writing). A file of the other style, or without a `<STYLE>` line but with
  window or setting lines, applies the nine and the windows this style also has, skips the
  `<SET>` lines, and prints one chat line (display type 8) that it skipped the other style's
  settings; a bare 0930 file loads silently. Unknown or malformed lines are skipped.
- `crates/` is the client. `tools/` holds test tooling only; the world viewer is an offline
  harness and never connects to a server.
- Close-out reports list `Retail Behavior traced:`, `Verification:` and `Known Gaps:`.
