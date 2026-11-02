# AC2_Rusted

A Rust client for **Asheron's Call 2**, built end to end. It reads the game's own DAT files and
speaks the AC2 network protocol. It contains no game data, executables or decompiled code: you
point it at your own installation.

**DAT target: the 7230 release** (client 02.00.20.7230, portal iteration 5064 and later), the
data the AC2e server serves. Every behaviour is ported from the retail client (Legions build 7160,
checked against the admin 7148 decompile; the 7230 executable where 7230 changed a packed layout
or added script calls) and cites the class and function it reproduces. See
[ARCHITECTURE.md](ARCHITECTURE.md) for the design and milestones, and
[docs/retail-catalog](docs/retail-catalog/README.md) for the work list: every retail function, by
owning crate, with what is done.

## Requirements

- **Rust 1.96.1** (pinned in `rust-toolchain.toml`; `rustup` installs it on first use) with
  `rustfmt` and `clippy`.
- **The game's DATs, 7230 release**: a directory holding `portal.dat`, `cell_1.dat` and
  `local_English.dat` (and `highres.dat` for the high-resolution textures), from the 2012 7230
  client package. Point everything at it with `AC2_DAT_DIR`, or `--dat-dir` where a program
  takes it. The client finds the release from the portal iteration (`ac2_core::DataRelease`), so
  the Legions DVD's 7160 DATs and the end-of-retail 0930 set (client build 0930, 2013) also read,
  but 7230 is the supported target: tests that pin record counts pin each release, and live
  testing runs on 7230.
- **Linux build libraries** for the window programs: ALSA headers (audio output) and a Vulkan or
  GL driver (wgpu 30). A missing GPU only affects the programs that draw; the tests that need one
  skip themselves.
- Optional: `AC2_GM_DLS` names a General MIDI `gm.dls` for the music synthesizer.

```bash
export AC2_DAT_DIR=/path/to/AsheronsCall2
```

## Crate ownership

Each crate is named for the retail subsystem it reproduces (INDEX packs in parentheses), so every
AC2 class has one home. A crate may depend only on crates above it.

| Crate | Owns |
|---|---|
| `ac2-core` | ids (`DataId`, `CellId`), `Random`, shared enums and flags, timers, the data release |
| `ac2-prefs` | `Preferences.ini` and `PreferenceBank`: the preferences, their sections and defaults |
| `ac2-math` | `Vector`, `Frame`, `Waveform`, bounds, cell `Position` offset rules |
| `ac2-dat` | DAT mounting: B-tree lookup, positioned record reads, the binary unpack cursor |
| `ac2-db` | typed `DBObj` decode cache, `MasterProperty`, enum id maps, string tables |
| `ac2-net` | transport (checksums, ISAAC, fragments, flow, logon) and the event codecs |
| `ac2-gun` | GUN, the MSN Gaming Zone service: the DirectPlay 8 client wire, GunDll's `CGun` and its `IGun*` interfaces, `NSTurbineGun::TurbineGun` |
| `ac2-anim` | `CSetup`, `AnimMap`, animators, locomotion sliders |
| `ac2-assets` | render asset codecs: surfaces, materials and modifiers, meshes, `VisualDesc`, `PSDesc`, FX tables |
| `ac2-world` | regions, landblocks, terrain and scenery, `CEnvCell` interiors, lights, water, sky, game time |
| `ac2-physics` | `CPhysicsObj` step, `CTransition` collision, `CMotionInterp`, body shapes |
| `ac2-smartbox` | `SmartBox`: cell residency and block loader threads, the player camera, the player body |
| `ac2-weenie` | the online session flow, weenie objects, event dispatch, player actions |
| `ac2-render` | scene assembly: landscape, objects, frills, water, interiors, sky, **particle systems**; backend-neutral |
| `ac2-gpu` | the wgpu backend: device, pipelines, shaders, textures, UI overlay |
| `ac2-ui` | input maps, UI elements and layouts from the DATs, the game panels |
| `ac2-script` | the WSL script VM and its native glue (much of the UI and gameplay runs on it) |
| `ac2-audio` | sound records, `SoundManager`, the music queue, the DLS synthesizer, mixer and output |
| `ac2-gameplay` | the rules for the quests, characters, crafting and conversations the client receives |
| `ac2-guide` | the in-client quest guide: the AC2Hero site baked in at build time (a modern addition) |
| `ac2-macro` | player automation (the Tracker), as pure policy (a modern addition) |
| `ac2-client` | the program: window, input, login and character screens, the world loop, `ClientView` |
| `ac2-launcher` | the `ac2-launcher` program: servers, sign-in and the client's start (a modern addition) |

`tools/` is test tooling only:

| Tool | Purpose |
|---|---|
| `world-viewer` | the offline world viewer (below); never connects, changes no client crate |
| `race-table` | runs the `CharacterGenSystem` script once and writes the playable race and sex models the viewer reads |
| `legions-server-double` | an in-process Legions 7160 server for the client's tests (the 7160 logon path; live 7230 testing uses AC2e) |
| `ac2-login-probe` | the network logon alone against a real or local server, stage by stage |
| `dat-diff` | compares two DAT sets record by record |
| `ac2hero-guide` | the quest guide as a standalone egui window |
| `ac2hero-map` | the quest guide's places on a zoomable world map; exports the map for AC2e |

## Commands

### Build, test, lint

```bash
cargo build --workspace                     # everything
cargo test --workspace                      # tests that need DATs skip without AC2_DAT_DIR
cargo test -p ac2-physics                   # one crate
cargo clippy --workspace --all-targets      # must stay warning-free
cargo fmt --all
```

The workspace lints in `Cargo.toml` allow `neg_cmp_op_on_partial_ord`, `manual_range_contains`,
`too_many_arguments` and `type_complexity` on purpose: retail writes `!(a < b)` and
`t < 0.0 || t > 1.0`, which differ from the "simplified" forms for NaN, and keeps its signatures.
Building many crates at once is heavy; on a small machine use `-j 6` and cap memory
(`systemd-run --user --scope -p MemoryMax=16G cargo …`).

### The client

```bash
# Build the client and the launcher (both land in target/release/)
cargo build --release -p ac2-client --bin ac2 -p ac2-launcher

# The windowed client: log on, pick or create a character, play
cargo run --release -p ac2-client --bin ac2 -- --dat-dir "$AC2_DAT_DIR" \
    --host 127.0.0.1 --port 9000 --account local
```

| Option | Meaning |
|---|---|
| `--dat-dir DIR` | the DATs (else `AC2_DAT_DIR`) |
| `--host NAME`, `--port N` | the server (retail's `-h` / `-p`); `--server HOST:PORT` gives both |
| `--account NAME` | the account (lower-cased, as retail) |
| `--glsticketdirect PASS` | password logon; `--glsticket FILE` reads the ticket from a file and deletes it (the launcher's way) |
| `--version V` | the logon's data version: by default the DATs' release (`7230` for the 7230 and 0930 data, `7160` for the DVD's); `7148` for servers that take only the admin build, `1802` for the relaunch servers |
| `--character NAME` | enter with this character, skipping character selection |
| `--intro` | show the intro's pictures before the connecting screen (skipped by default) |
| `--width N`, `--height N` | the window's first size |
| `--radius N` | the resident ring of landblocks (by the graphics quality without it) |
| `--prefs FILE` | `Preferences.ini` (default: the AC2 configuration directory) |
| `--language NAME` | the language, mounting its `local_NAME.dat` |
| `--content-dir DIR` | the help pages (else `Content` beside the DATs, then `AC2_CONTENT_DIR`) |
| `--chargen FILE` | also capture every created character to FILE |
| `--gunserver HOST`, `--gunport N` (default 2310), `--gunticket`, `--gunticketoverride T` | a GUN server, retail's arguments; GUN stays off without `--gunserver`, as in retail |
| `--tchat`, `--tchatdll NAME` | Turbine chat, retail's arguments |

### The launcher

The launcher stores the player's servers and starts the client with the right command line for
each, as ThwargLauncher does (`docs/server-list.md`). With no arguments it opens its window.

```bash
cargo run --release -p ac2-launcher                       # the window
target/release/ac2-launcher set dat-dir "$AC2_DAT_DIR"    # also: set client PATH, set list ADDRESS
target/release/ac2-launcher add "AC2e Test Server" 127.0.0.1:9000 AC2E [--login-server URL]
target/release/ac2-launcher list
target/release/ac2-launcher play "AC2e Test Server" [--account NAME]   # a password on stdin
```

The launcher finds `ac2` beside itself unless `set client` names it.

### Probes and test servers

```bash
# Log on alone and report each stage, quitting at character selection
cargo run --release -p ac2-login-probe -- --host 127.0.0.1 --port 9000 --account local \
    [--version 7230] [--glsticketdirect PASS] [--dat-dir DIR] [--timeout 45]

# Headless: log on, enter, walk forward over the DATs' terrain, print the frames, log off
cargo run -p ac2-client --example headless_probe -- --dat-dir "$AC2_DAT_DIR" \
    [--server 127.0.0.1:9000] [--account local] [--version V] [--client-exe PATH] \
    [--seconds 4] [--walk 0.5] [--character NAME]

# Headless log-on and protocol check, with optional lossy-network simulation
cargo run -p ac2-weenie --example online_probe -- --server 127.0.0.1:9000 --account local \
    [--password PW] [--character NAME] [--seconds 10] [--walk 2.0] [--version V] \
    [--client-exe PATH] [--dat-dir PATH] [--language NAME] \
    [--loss DROP,DUPLICATE,HOLD,MAX_HOLD] [--seed N]

# Sign in at an AC2e account service with the launcher's code (password on stdin)
cargo run -p ac2-launcher --example sign_in -- URL ACCOUNT
```

`--client-exe` reads the RSA key a retail `ac2client.exe` carries, as retail does.

Live testing is against a local AC2e server on the 7230 DATs. `AC2E_DIR` names the AC2e checkout
(default `../third_party/ac2e-src/repo`); `AC2E_BIND` and `AC2E_GUN_BIND` move its ports:

```bash
scripts/ac2e-server.sh                 # keep a server running on 127.0.0.1:9000
scripts/ac2e-login-test.sh             # start one, log on to the character set, stop it
scripts/ac2e-launcher-test.sh          # the launcher's sign-in and start against AC2e accounts
scripts/ac2e-live-test.sh [FILTER]     # the gameplay suite (quests, crafting, combat, effects)
                                       # on a fresh server of its own; AC2E_LIVE_TEST picks the target
```

The HUD is the client's modern style by default: the retail frameworks over the DAT layouts,
restyled (glass windows, the shortcut strip and dock, the inventory drawer, chat tabs) with the
same behaviour. The retail DAT HUD is one switch away: `ClassicHud` under `Misc` on the client
options page (or `ClassicHud=1` in `Preferences.ini`) rebuilds the HUD in the classic style.
`AGENTS.md` lists every modern change and what retail did.

### The offline world viewer

Draws what the client draws, without a server: the viewer builds the client's own `ClientFrame`
and hands it to `ClientView`, so terrain, grass frills, water, sky, interiors, objects, particles
and the DAT-built HUD (modern by default, `--hud-style classic` for retail's) all appear exactly
as in the client. A player walks the world with collision
and the retail camera; an egui panel swaps the playable race and sex, teleports, scales the
movement speed and sets the time of day.

```bash
cargo build --release -p world-viewer -p race-table
target/release/world-viewer --dat-dir "$AC2_DAT_DIR" [--block 7F7F] [--at X,Y] [--radius 2] [--fly]

# One headless frame (player, HUD, panel) as a PNG
target/release/world-viewer --dat-dir "$AC2_DAT_DIR" --block 7567 --at 60,80 \
    --snapshot out.png [--model N] [--walk SECONDS] [--speed N] [--warm-up SECONDS] [--fly]
```

HUD snapshots (modern style; the HUD is logged on to the in-process server double):
`--hud-panel NAME` opens a toolbar panel (Inventory, Skill, Quest, Social, Examination,
Navigation, Option) and `--hud-window NAME` a window that normally appears only during play
(confirm, message, wait, resurrect, loot, stack, trade, shop, catalog, consignment, new-sale,
ingredient, craft, book, help, guide, tracker, pets, fellowship, fellow-mini, chat-settings,
edit-layout, targeting; both repeatable). Each is raised as the game raises it (the notices the
server would send, the double's chest and craft registry, a real drag or press on the UI), with
made-up sample content. The help window needs `AC2_CONTENT_DIR` (or a `Content` folder beside the
DATs). One PNG per panel and window:

```bash
XDG_CONFIG_HOME=$(mktemp -d) target/debug/world-viewer --dat-dir "$AC2_DAT_DIR" --block CB4D \
    --hud-window loot --size 1365x768 --snapshot loot.png
tools/world-viewer/snapshot_all.sh OUT_DIR [target/debug/world-viewer]   # PANELS=/WINDOWS= pick a subset
```

`--block` is the landblock `xxyy` in hex, `--at` the start position within it (0–160),
`--model` an index into the race table, `--fly` starts (or with `--snapshot` draws) the free
camera. Block `7567` (the village) is full of braziers and good for checking particles.

| Control | Walk mode | Fly mode (Tab) |
|---|---|---|
| Move | W/S walk, Q/E strafe, A/D turn, Space jump | WASD, Space/C up and down, Shift fast |
| Camera | hold the right button to look, wheel zooms, V first person, Home resets | hold the left button to look |
| Other | F1 hides the panel, `[` and `]` step the time of day, F7 switches the HUD between modern and classic (`--hud-style`) | |

The race and sex table (`race<TAB>sex<TAB>phys_obj`, hex) is written by `race-table` to
`$AC2_RACE_TABLE`, else `$XDG_CACHE_HOME/ac2-world-viewer/race_table.tsv` (default
`~/.cache/…`). The viewer runs `race-table` from beside its own executable when the file is
missing; or write it yourself:

```bash
target/release/race-table --dat-dir "$AC2_DAT_DIR" --out ~/.cache/ac2-world-viewer/race_table.tsv
```

The HUD's vitals, skills and shortcut panels stay empty offline: they wait on the online
client's `frameworks` pass. The viewer links the client's libraries but never opens a socket and
takes no `--server` flag.

### Other tools

```bash
# Compare two DAT sets: records added, removed or changed, per file and record type
cargo run --release -p dat-diff -- OLD_DIR NEW_DIR [--tsv OUT.tsv]

# The quest guide (needs AC2HERO_DIR at build time, or --mirror at run time)
cargo run --release -p ac2hero-guide -- [--mirror DIR] [--page TITLE] [--stats]
cargo run --release -p ac2hero-map -- [--mirror DIR] [--dat-dir DIR] [--snapshot OUT.png] \
    [--screenshot OUT.png [--at COORD] [--zoom PX_PER_CELL]] [--size WxH] [--bare 1] \
    [--player "COORD@HEADING"] [--export DIR]

# Redraw the modern HUD's panel icons from AC2's and AC1's DATs
cargo run -p ac2-ui --example bake_panel_icons -- AC2_DAT_DIR AC1_DAT_DIR [OUT_DIR]

scripts/check_file_sizes.sh            # the 1000 / 1500 line gate
```

### The retail catalog

```bash
scripts/retail_catalog.py status
scripts/retail_catalog.py mark done 'CDayDesc::GetLighting'
scripts/retail_catalog.py mark partial 0058c8c0 --note 'no cloud shadow UVs'
scripts/retail_catalog.py mark skip 'FrillRender::*' --note 'why'
scripts/retail_catalog.py build            # regenerate docs/retail-catalog from the index
```

It reads the decompile index (`INDEX/ac2client_index.sqlite`, or `--index` / `AC2_INDEX`). Mark
the functions you port in the same commit.

## Rules for changes

- Port from the retail decompile and cite the class and function; never commit decompiled code,
  game data or data-derived fixtures. Tests that need data read `AC2_DAT_DIR` and skip without it.
- Where admin 7148 and Legions 7160 differ, follow Legions. Where the 7230 data changed a layout,
  follow the 7230 executable (`ac2_core::data_release`).
- New code goes in the crate that owns the subsystem; the dependency graph is one-way (the order
  of the table above). Change `ARCHITECTURE.md` first when no crate fits.
- A `.rs` file reaching 1000 lines is split by responsibility (`scripts/check_file_sizes.sh` fails at
  1500); keep public APIs small and plain-data.
- Retail ran x87 at 53-bit precision: evaluate multi-operation expressions in `f64` and round to
  `f32` where retail stores a `float`.
- The frame thread never decodes or reads files: DAT reads happen on loader threads.
- Networking matches retail byte for byte (opcodes, field order, sequencing).
- `tools/` stays tooling: the viewer drives client crates, it does not change them.
- Commit with the CLI; `main` stays linear (squash before pushing, integrate worker branches by
  cherry-pick).

See [AGENTS.md](AGENTS.md) for the full working rules.

## License

AGPL-3.0-only; see [LICENSE](LICENSE) and [NOTICE.md](NOTICE.md). Asheron's Call and Asheron's
Call 2 are trademarks of their respective owners; this project is not affiliated with them.
