# AC2_Rusted

A Rust client for **Asheron's Call 2**, built end to end. It reads the game's own DAT files and
speaks the AC2 network protocol. It contains no game data: point it at your own installation with
`AC2_DAT_DIR` (the directory holding `portal.dat`, `cell_1.dat` and `local_English.dat`).

See [ARCHITECTURE.md](ARCHITECTURE.md) for the crate layout and milestones.

```bash
export AC2_DAT_DIR=/path/to/AsheronsCall2
cargo test --workspace
# Headless log-on check against a server:
cargo run -p ac2-weenie --example online_probe -- --server 127.0.0.1:9000 --account local
```

Licensed AGPL-3.0-only; see [LICENSE](LICENSE) and [NOTICE.md](NOTICE.md).
