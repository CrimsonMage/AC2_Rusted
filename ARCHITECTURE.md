# AC2_Rusted Architecture (draft for review)

A Rust Asheron's Call 2 client, built end to end. The crates follow the original client's own
subsystems, as the decompile index groups them (`INDEX` packs 03-16), so each AC2 class has one
obvious home crate. Nothing is inherited from the Asheron's Call 1 port; working AC2 code is
brought over from Vitaeum-AC2 (commit `b2963be`) into the crate that owns it.

- `crates/` is the client: libraries plus the `ac2-client` program.
- `tools/` is test tooling only. `world-viewer` is an offline harness that drives the client's own
  view (`ac2_client::view::ClientView`) with a synthetic `ClientFrame` built from the DATs, so it
  draws what the client draws (terrain, frills, water, sky, objects, the HUD) without a server:
  it walks the world under the player camera (`ac2-smartbox`, `ac2-physics`) with an egui panel,
  and the HUD's session is the client logged on to `legions-server-double` in process (starter
  inventory, worn gear, skills), so the toolbar's panels work. It never opens a socket. `race-table` runs the `CharacterGenSystem`
  script once and writes the playable race and sex models the viewer reads. `ac2hero-guide`
  and `ac2hero-map` are standalone egui tools for the in-client quest guide: the guide `ac2-guide`
  bakes from the AC2Hero site mirror (`AC2HERO_DIR`; nothing from it is committed), browsed with
  clickable map coordinates, a target arrow and route, and shown as icons on a world map drawn
  from the landscape (`AC2_DAT_DIR`: per-vertex far colours, normals, water and roads, finer as
  you zoom). They only read the client's crates.

## Crates

| Crate | AC2 subsystem (index pack) | Owns | Brought from Vitaeum-AC2 |
|---|---|---|---|
| `ac2-core` | Core runtime (03), shared types (00) | ids (`DataId`, `CellId`, `InstanceID`), scalars, shared enums and flags, timers | foundation, enums |
| `ac2-prefs` | Preferences (portal shared `prefs`, pack 06) | `Preferences.ini` (the Win32 INI calls, byte-exact), `PreferenceBank` (sections, keys, bool, float, enum and string preferences with defaults, ranges and choices, load and save, section apply and recall), the session `Preferences` cache; names come from the DAT through a trait | rebuilt from the decompile |
| `ac2-math` | Math and geometry (04) | `Vector`, `Frame`, `Plane`, bounds, `Heading`, cell `Position` and its offset rules | math, world `coordinates` |
| `ac2-dat` | Database access (06) | DAT mounting, B-tree lookup, record reads, the binary unpack cursor | dat, archive cursor |
| `ac2-db` | Database objects (06) | typed `DBObj` decode cache, `MasterProperty`/`PropertyCollection`, `EnumIDMap` | resources (cache core only), render-codec `ac2_property`, `EnumIdMap` |
| `ac2-net` | Network and protocol (07) | transport (`ProtoHeader`, checksums, ISAAC, fragments, flow, logon), event codecs | net, protocol |
| `ac2-anim` | Animation (08 `animsys`) | `CSetup`, `AnimMap`, `DBAnimator`/`AnimCube`/`AMHuman`, locomotion sliders | render-codec `ac2_animation`/`ac2_animator`/`ac2_setup`, body locomotion |
| `ac2-assets` | Render asset codecs (12) | `RenderSurface`/`RenderTexture`, materials and modifiers, `CDataMesh`, `VisualDesc`, `PSDesc`, `DBFXTable` | render-codec `ac2_*` decoders |
| `ac2-world` | Regions, cells (08 `region`, `cell`) | `LandscapeDefs`, landblocks, terrain/surface descs, scenery and grass placement, `CEnvCell`/`CCellMesh`, lights, water, region sky and game time | world `ac2_*` modules |
| `ac2-physics` | Physics objects (08 `physobj`, `path`) | `CPhysicsObj` step, `CTransition` collision, `CMotionInterp` and `MotionInterpDesc`, body shapes, the part-group tree (`CPartArray`/`CPartGroup`: bones, group matrices, spheres, cylspheres, holding locations, ray trace; drawing stays in `ac2-render`) | physics (transition core), live-authority AC2 motion, body profile |
| `ac2-smartbox` | SmartBox and object maintenance (08 `smartbox`, `objects`) | landblock and cell residency around the player, interior scope, camera, the object table (`CObjectMaint`) | rebuilt; keeps the AC2 interior scope |
| `ac2-weenie` | Client core (09 `weenie`, `events`, `action`) | weenie objects, event dispatch, player actions, the online session flow | session |
| `ac2-render` | Render pipeline (13 `render`, `scenerender`) | scene assembly: landscape, objects, frills, water, interiors, particles, sky; backend-neutral draw lists | rebuilt; brings the AC2 terrain blend, frill, water, interior, particle and body visuals |
| `ac2-gpu` | Render device (13 `device`) | the `wgpu` backend: device, pipelines, buffers, textures, shaders | render-wgpu core (no "Enhanced" effects) |
| `ac2-ui` | UI framework and modules (14, 15) | input maps, UI elements, `LayoutDesc`, game panels | later |
| `ac2-script` | WSL interpreter (10) | `WLib`/`ByteStream` and `WState` decode, the script VM and native glue (AC2 UI and much gameplay run on it) | rebuilt from the decompile |
| `ac2-gameplay` | Gameplay (11) | characters, quests, fellowships, crafting, examination; `quest` (`gmQuestInfo` status, time and extra-text rules) so far | started |
| `ac2-audio` | Audio (16) | sound records (`SoundInfo`, `MusicInfo`, `DBWave`) decoded on a loader thread, `SoundManager` (play requests, budget, channel volumes), the music queue (`DXMusicMessageThread`: primary and secondary segments over `MusicPerformance` and `StylePerformance`, events to an `EventSink`), the DLS software synthesizer (`synth`: voices from the band collections or the user's GM file, an `EventSink` that renders), the mixer and speaker output (cpal), the ambient driver, rain and thunder, the portal space's sounds and the screens' music (`SoundSpace`: the client answers the place questions over physics, so this crate names no physics or world type) | rebuilt from the decompile |
| `ac2-guide` | Quest guide (no retail subsystem: the AC2Hero site, a modern addition) | the AC2Hero mirror baked in at build time (`AC2HERO_DIR`, nothing from it committed): pages, quest facts, reward items, images and their decoding, map coordinates (`Position::FromRealMapCoords` forms), steering to them, map markers, browsing (tabs, search, chat share tokens) | new |
| `ac2-macro` | Player automation tools (no retail subsystem: a modern addition) | pure policy the client feeds and acts on: the Tracker (name patterns over the objects the client sees, Virindi Sense's `patterns.txt`; matches announced in chat, listed with distances, picked for the arrow); no files, drawing or network | new |
| `ac2-client` | Client entry (09 `main`) | the program: window, input, login screen, world loop | new |

## Dependency order

Each crate may depend only on crates earlier in this list:

```
ac2-core → ac2-prefs → ac2-math → ac2-dat → ac2-db → ac2-net
                                ac2-db → ac2-anim → ac2-assets → ac2-world → ac2-physics
         → ac2-smartbox → ac2-weenie → ac2-render → ac2-gpu → ac2-ui → ac2-client
```

`ac2-net` depends only on `ac2-core`. `ac2-prefs` has no crate dependencies (the names are passed in,
so it needs no DAT access). `ac2-script`, `ac2-gameplay` and `ac2-audio` slot in when
they are built: script after `ac2-weenie`, gameplay after script (it depends on `ac2-net` only), audio after `ac2-world`
(it needs only `ac2-db` so far; `ac2-client` drives it).
`ac2-script` also depends on `ac2-anim` (the latent mode and behaviour completions,
`ac2_script::host::latent::completions`), which precedes it in the order.
`ac2-ui` depends on `ac2-script` (the saved window locations are `UISaveLocations`, the
script's native layout, `ac2_ui::window_locations`); script sits before ui in the order.
`ac2-ui` also depends on `ac2-gameplay` (the quest journal's rules, `ac2_gameplay::quest`).
`ac2-ui` also depends on `ac2-render` (the client options page's scene options, `ac2_ui::OptionsPages`);
render sits before ui in the order.
`ac2-guide` depends only on `ac2-physics` (`LandDefs` cell addressing for its coordinates) and the
`image` decoder (the site's pictures), and sits after it; `ac2-ui` takes the guide from it (the navigation map's guide layers, the quest guide window).
`ac2-macro` depends only on `ac2-physics` (`LandDefs` offsets and map coordinates) and sits after it;
`ac2-ui` (the Tracker window) and `ac2-client` (observations and effects) take it.

## Milestones

1. **Login.** core, math, dat, db, net, weenie: headless log on, character list and world entry
   against a server (parity with the current `online_probe`).
2. **World on screen.** anim, assets, world, render, gpu and the world viewer: landscape,
   scenery, grass, water, interiors and particles drawn offline.
3. **Walking.** physics and smartbox: the AC2 body moves with collision; viewer parity with
   Vitaeum-AC2.
4. **Online client.** `ac2-client`: log in, enter at the server position, walk with movement
   reports, see other players.
5. **Game.** ui, script, gameplay and audio.

## Rules carried over

- `.rs` files: plan a split at 1000 lines, 1500 is a hard fail. The dependency graph is one-way.
- Behaviour follows the original client, cited by class and function name. No decompiled code,
  game data or data-derived fixtures are committed. Tests that need data read
  `AC2_DAT_DIR`.
- AGPL-3.0-only; copyright CrimsonMage.
