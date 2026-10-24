# AC2_Rusted

A Rust client for **Asheron's Call 2**, built end to end. It reads the game's own DAT files and
speaks the AC2 network protocol. It contains no game data: point it at your own installation with
`AC2_DAT_DIR` (the directory holding `portal.dat`, `cell_1.dat` and `local_English.dat`).

See [ARCHITECTURE.md](ARCHITECTURE.md) for the crate layout and milestones, and
[docs/retail-catalog](docs/retail-catalog/README.md) for the work list: every function of the
retail client, by owning crate, with what is done.

```bash
export AC2_DAT_DIR=/path/to/AsheronsCall2
cargo test --workspace
# Headless log-on check against a server:
cargo run -p ac2-weenie --example online_probe -- --server 127.0.0.1:9000 --account local
# Offline world viewer (never connects): a window, or a PNG with --snapshot
cargo run --release -p world-viewer -- --block 7F7F --radius 3
cargo run --release -p world-viewer -- --block CB4D --snapshot cb4d.png
```

Viewer controls: hold the left mouse button to look, WASD to move, Space/C up and down, Shift to
go fast, `[` and `]` to step the time of day.

Licensed AGPL-3.0-only; see [LICENSE](LICENSE) and [NOTICE.md](NOTICE.md).
