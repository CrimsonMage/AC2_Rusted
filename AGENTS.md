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
  logical display and draws it scaled by `min(width / 1024, height / 768)`. Retail's
  `DisplayResolution` (`Device::InitializeRenderingPreferences`: the device's modes over 799 x
  599, 1024 x 768 always listed and the default) switched the display mode; here it lists the
  monitor's modes the same way and sets the window's inner size, at start and when the player
  picks a mode (a window resized by hand keeps its size until then; FullScreen stays the
  borderless switch). The HUD fills the window and the pregame screens stay 1024 x 768,
  centred. Layout and anchoring code stays retail.
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
  map seamlessly; the terrain takes the darker of the two, and a frill already darkened by the
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
  else the world's colour where the mirrored direction reaches the screen, blended in by a Fresnel
  weight, with procedural ripples at level 2. What is off the screen is not reflected.
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
  Here a window with a drag bar still moves by it, and any other top-level HUD window moves by
  Alt + left drag anywhere on it. Every window's position is saved locally per character (the
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
- Exception: the HUD has two styles, modern and classic. Retail drew only its DAT layouts' HUD.
  Here the classic style is that HUD, unchanged; the modern style (the mockup "variant D" the
  user chose) builds the same frameworks over the same elements, restyling their layout
  descriptions before they are made (each restyle keeps every element id and state, so every
  framework's behaviour stays retail) and adding the client's own parts beside them, which call
  only the frameworks' existing entry points and send nothing new to the server. The choice is
  the client-only `ClassicHud` preference (`ac2_ui::hud_style`, key `0x7E000001` in the
  `Misc` section, listed on the options page by its key name); applying it rebuilds the HUD
  (`ClientView::set_hud_style`), whose panels come back from the player as after a relog. The
  world viewer takes `--hud-style modern|classic` and switches with F7. The default stays classic
  until the modern style is complete. Each modern change is listed here as it lands.
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
  numbers on), and the shortcut bar restyled as rows of 32-pixel glass wells 3 pixels apart, an
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
  room); an action that does not apply is dimmed. Retail had no such ring.
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
  panel, changes the layout (not yet saved). The panel is laid out by the display's top left (8
  pixels in), not a saved window, until the design's Edit layout mode lands; the indicators are
  drawn 16 x 25. The fields are retail's, moved up to the panel (the
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
  radar is laid out by the display's top right (12 and 8 pixels in), not a saved window, until the
  design's Edit layout mode lands.
  The chat windows are the design's (`ac2_ui::modern_chat`): each of the four `ChatWindow`s on
  teal glass in a bronze line (gold while active) under a bronze header holding its tab (its
  window key and its name, `ID_Option_Chat1`..`4`), the other windows' toggles, an opacity slider
  (`op`) and the options button (`=`); the log on darker glass with the thin scroll bar; a channel
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
  by its bar or Alt + drag, nor resizes: `UiTree::set_locked`), Hide window (`ChatWindow::Show(0)`,
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
  releases it), resets the offsets and the chat windows' choices, or closes the mode. The offsets
  and the lock are kept in the HUD layout (`offset.<part>`, `hud.locked`).
  The loot window is the design's (its variant A, `ac2_ui::modern_loot`): `ExternalContainerField`
  at 214 by 262 with retail's frame (`0x41000D41`) drawn in nine pieces of their own sizes
  (`compose::NINE_SLICE_EDGES`; retail drew the frame at its 138 by 165) over teal glass, the title
  centred, a bronze close button, the list four slots wide (retail three) with the thin scroll
  bar, Loot All and Transmute All as pills holding their art and their names, and a note under
  them. The elements are retail's, so opening, slots, drops and the buttons stay retail's.
  While the targeting pointer is on (`RecvNotice_SetTargetingPointer`), a gold banner at the
  display's top centre says to pick a target (`ac2_ui::modern_banner`, the design's variant B);
  retail showed only the pointer's cursor. It takes no pointer; the targeting stays retail's.
  The seven panel buttons' icons are one matching set in Asheron's Call 1's look (the user's
  choice): each panel's AC1 toolbar symbol (Examination its magnifier, Navigation its compass,
  Option its checklist, Inventory its backpack, Quest its leaf, Skill its flexed arm, Social its
  figures) drawn in AC2's colours, ivory on AC2's bronze orb, with a brighter rollover. Exception to
  "never commit game data", at the user's explicit request: these redrawn icons ship as PNGs in
  `crates/ac2-ui/assets/hud/` (`ac2_ui::ac1_buttons::shipped_icons`). They are not the DATs'
  images but new art made from them; regenerate them with `cargo run -p ac2-ui --example
  bake_panel_icons -- AC2_DAT_DIR AC1_DAT_DIR` (`ac2_assets::ac1_icons` reads AC1's container
  through `ac2_dat`'s retail layout).
- Exception: the radar shows the player's map coordinates under the compass by default.
  Retail's `gmRadarUI::ReloadOptions` opened `BaseField`'s coordinates box only while the options
  profile's `PROFILE_SHOW_RADAR_COORDS` bit (0x10) was set, and a new character's profile has it
  off. Here the box also opens while the player has not hidden it in this client
  (`Client::radar_coords_shown`): the radar's button or a change of the bit records the choice
  per character on this machine (`RadarCoordsHidden-<character>.txt` beside the window
  positions). The bit, the button's `SetShowRadarCoords` call and the profile upload stay retail.
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
  faint fringes hid the blades behind them. The frills within 12 m of the camera draw back to
  front among themselves (`FRILL_SORT_DISTANCE`), so a near blade's soft edge no longer keeps
  out the blades and the body behind it. The grass parts around the nearest eight posed bodies
  within 60 m (`frill::grass_pushers`, `grass_push` in `object.wgsl`): the part of a blade within
  0.75 m (times the body's scale) of a body's feet is carried out to that reach and sinks as it
  bends, its base planted. Retail's frills stood still and the bodies walked through them.
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
- Exception: a book's text after its last `<BR>` is a page. Retail's `gmBookUI::ParseText` stops
  at the last `<BR>` and drops anything after it; here that tail is paginated too
  (`book.rs` `segments`), so a book without a closing `<BR>` is not shown empty.
- Exception: an open store is left as the session ends. Retail's store panel destructors
  (`gmShopUI::~gmShopUI`, `gmShopConsignmentUI::~gmShopConsignmentUI`: `RequestLeaveStore`;
  `gmShopCatalogUI::~gmShopCatalogUI`: `RequestLeaveCatalog`) called the player's script whenever
  the UI was torn down with the player present; here the log off and the quit ask first, once
  each and only while such a panel is open (`ac2-ui` `hud_tick.rs` `leave_open_stores`).
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
