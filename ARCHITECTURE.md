# AC2_Rusted Architecture (draft for review)

A Rust Asheron's Call 2 client, built end to end. The crates follow the original client's own
subsystems, as the decompile index groups them (`INDEX` packs 03-16), so each AC2 class has one
obvious home crate. Nothing is inherited from the Asheron's Call 1 port; working AC2 code is
brought over from Vitaeum-AC2 (commit `b2963be`) into the crate that owns it.

- `crates/` is the client: libraries plus the `ac2-client` program.
- `tools/` is test tooling only. `world-viewer` is an offline harness that drives the same crates
  without a server and never connects.

## Crates

| Crate | AC2 subsystem (index pack) | Owns | Brought from Vitaeum-AC2 |
|---|---|---|---|
| `ac2-core` | Core runtime (03), shared types (00) | ids (`DataId`, `CellId`, `InstanceID`), scalars, shared enums and flags, timers | foundation, enums |
| `ac2-math` | Math and geometry (04) | `Vector`, `Frame`, `Plane`, bounds, `Heading`, cell `Position` and its offset rules | math, world `coordinates` |
| `ac2-dat` | Database access (06) | DAT mounting, B-tree lookup, record reads, the binary unpack cursor | dat, archive cursor |
| `ac2-db` | Database objects (06) | typed `DBObj` decode cache, `MasterProperty`/`PropertyCollection`, `EnumIDMap` | resources (cache core only), render-codec `ac2_property`, `EnumIdMap` |
| `ac2-net` | Network and protocol (07) | transport (`ProtoHeader`, checksums, ISAAC, fragments, flow, logon), event codecs | net, protocol |
| `ac2-anim` | Animation (08 `animsys`) | `CSetup`, `AnimMap`, `DBAnimator`/`AnimCube`/`AMHuman`, locomotion sliders | render-codec `ac2_animation`/`ac2_animator`/`ac2_setup`, body locomotion |
| `ac2-assets` | Render asset codecs (12) | `RenderSurface`/`RenderTexture`, materials and modifiers, `CDataMesh`, `VisualDesc`, `PSDesc`, `DBFXTable` | render-codec `ac2_*` decoders |
| `ac2-world` | Regions, cells (08 `region`, `cell`) | `LandscapeDefs`, landblocks, terrain/surface descs, scenery and grass placement, `CEnvCell`/`CCellMesh`, lights, water, region sky and game time | world `ac2_*` modules |
| `ac2-physics` | Physics objects (08 `physobj`, `path`) | `CPhysicsObj` step, `CTransition` collision, `CMotionInterp` and `MotionInterpDesc`, body shapes | physics (transition core), live-authority AC2 motion, body profile |
| `ac2-smartbox` | SmartBox and object maintenance (08 `smartbox`, `objects`) | landblock and cell residency around the player, interior scope, camera, the object table (`CObjectMaint`) | rebuilt; keeps the AC2 interior scope |
| `ac2-weenie` | Client core (09 `weenie`, `events`, `action`) | weenie objects, event dispatch, player actions, the online session flow | session |
| `ac2-render` | Render pipeline (13 `render`, `scenerender`) | scene assembly: landscape, objects, frills, water, interiors, particles, sky; backend-neutral draw lists | rebuilt; brings the AC2 terrain blend, frill, water, interior, particle and body visuals |
| `ac2-gpu` | Render device (13 `device`) | the `wgpu` backend: device, pipelines, buffers, textures, shaders | render-wgpu core (no "Enhanced" effects) |
| `ac2-ui` | UI framework and modules (14, 15) | input maps, UI elements, `LayoutDesc`, game panels | later |
| `ac2-script` | WSL interpreter (10) | script VM and native glue (AC2 UI and much gameplay run on it) | later |
| `ac2-gameplay` | Gameplay (11) | characters, quests, fellowships, crafting, examination | later |
| `ac2-audio` | Audio (16) | sound tables and playback | later |
| `ac2-client` | Client entry (09 `main`) | the program: window, input, login screen, world loop | new |

## Dependency order

Each crate may depend only on crates earlier in this list:

```
ac2-core → ac2-math → ac2-dat → ac2-db → ac2-net
                                ac2-db → ac2-anim → ac2-assets → ac2-world → ac2-physics
         → ac2-smartbox → ac2-weenie → ac2-render → ac2-gpu → ac2-ui → ac2-client
```

`ac2-net` depends only on `ac2-core`. `ac2-script`, `ac2-gameplay` and `ac2-audio` slot in when
they are built: script after `ac2-weenie`, gameplay after script, audio after `ac2-world`.

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

- Every `.rs` file stays under 800 lines. The dependency graph is one-way.
- Behaviour follows the original client, cited by class and function name. No decompiled code,
  game data or data-derived fixtures are committed. Tests that need data read
  `AC2_DAT_DIR`.
- AGPL-3.0-only; copyright CrimsonMage.
