# AC2_Rusted

A Rust client for **Asheron's Call 2**, built end to end. It reads the game's own DAT files and
speaks the AC2 network protocol. It contains no game data, executables or decompiled code: you
point it at your own installation.

Every behaviour is ported from the retail client (Legions build 7160, checked against the admin
7148 decompile) and cites the class and function it reproduces. See
[ARCHITECTURE.md](ARCHITECTURE.md) for the design and milestones, and
[docs/retail-catalog](docs/retail-catalog/README.md) for the work list: every retail function, by
owning crate, with what is done.

## Requirements

- **Rust 1.96.1** (pinned in `rust-toolchain.toml`; `rustup` installs it on first use) with
  `rustfmt` and `clippy`.
- **The game's DATs**: a directory holding `portal.dat`, `cell_1.dat` and `local_English.dat`
  (and `highres.dat` for the high-resolution textures). Point everything at it with
  `AC2_DAT_DIR`, or `--dat-dir` where a program takes it.
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
| `ac2-core` | ids (`DataId`, `CellId`), `Random`, shared enums and flags, timers |
| `ac2-math` | `Vector`, `Frame`, `Waveform`, bounds, cell `Position` offset rules |
| `ac2-dat` | DAT mounting: B-tree lookup, positioned record reads, the binary unpack cursor |
| `ac2-db` | typed `DBObj` decode cache, `MasterProperty`, enum id maps, string tables |
| `ac2-net` | transport (checksums, ISAAC, fragments, flow, logon) and the event codecs |
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
| `ac2-client` | the program: window, input, login and character screens, the world loop, `ClientView` |

`ac2-gameplay` (characters, quests, fellowships, crafting) is planned and has no crate yet; the
catalog already lists its functions.

`tools/` is test tooling only:

| Tool | Purpose |
|---|---|
| `world-viewer` | the offline world viewer (below); never connects, changes no client crate |
| `race-table` | runs the `CharacterGenSystem` script once and writes the playable race and sex models the viewer reads |
| `legions-server-double` | an in-process Legions 7160 server for the client's tests |

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
# The windowed client: log on, pick a character (or --character NAME), play
cargo run --release -p ac2-client --bin ac2 -- --dat-dir "$AC2_DAT_DIR" \
    --server 127.0.0.1:9000 --account local [--version 7160] [--client-exe PATH] [--radius 1]

# Headless: log on, enter, walk forward over the DATs' terrain, print the frames, log off
cargo run -p ac2-client --example headless_probe -- --dat-dir "$AC2_DAT_DIR" \
    [--server 127.0.0.1:9000] [--account local] [--seconds 4] [--walk 0.5] [--character NAME]

# Headless log-on and protocol check, with optional lossy-network simulation
cargo run -p ac2-weenie --example online_probe -- --server 127.0.0.1:9000 --account local \
    [--password PW] [--character NAME] [--seconds 10] [--walk 2.0] [--version 7160] \
    [--client-exe PATH] [--dat-dir PATH] [--loss DROP,DUPLICATE,HOLD,MAX_HOLD] [--seed N]
```

The Legions DVD client logs on as `7160`; servers that only take the admin build need
`--version 7148`. `--client-exe` reads the RSA key a retail `ac2client.exe` carries, as retail
does.

### The offline world viewer

Draws what the client draws, without a server: the viewer builds the client's own `ClientFrame`
and hands it to `ClientView`, so terrain, grass frills, water, sky, interiors, objects, particles
and the DAT-built HUD all appear exactly as in the client. A player walks the world with collision
and the retail camera; an egui panel swaps the playable race and sex, teleports, scales the
movement speed and sets the time of day.

```bash
cargo build --release -p world-viewer -p race-table
target/release/world-viewer --dat-dir "$AC2_DAT_DIR" [--block 7F7F] [--at X,Y] [--radius 2] [--fly]

# One headless frame (player, HUD, panel) as a PNG
target/release/world-viewer --dat-dir "$AC2_DAT_DIR" --block 7567 --at 60,80 \
    --snapshot out.png [--model N] [--walk SECONDS] [--speed N] [--warm-up SECONDS] [--fly]
```

`--block` is the landblock `xxyy` in hex, `--at` the start position within it (0–160),
`--model` an index into the race table, `--fly` starts (or with `--snapshot` draws) the free
camera. Block `7567` (the village) is full of braziers and good for checking particles.

| Control | Walk mode | Fly mode (Tab) |
|---|---|---|
| Move | W/S walk, Q/E strafe, A/D turn, Space jump | WASD, Space/C up and down, Shift fast |
| Camera | hold the right button to look, wheel zooms, V first person, Home resets | hold the left button to look |
| Other | F1 hides the panel, `[` and `]` step the time of day | |

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
- Where admin 7148 and Legions 7160 differ, follow Legions.
- New code goes in the crate that owns the subsystem; the dependency graph is one-way (the order
  of the table above). Change `ARCHITECTURE.md` first when no crate fits.
- Every `.rs` file stays under 800 lines; keep public APIs small and plain-data.
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
