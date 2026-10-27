# Performance baseline (Phase 0)

Measurement only: this baseline changes no behaviour and no rendered output. It records the
client's frame cost before the performance pass, so later changes can be compared against it.

## What the bench measures

`world-viewer --bench N` (in `tools/world-viewer/src/bench.rs`) runs the viewer's headless scene
(the same walk or fly setup as `--snapshot`, on the `ac2-gpu` headless device), then times N more
frames. Each frame runs `ClientView::update`, `ClientView::prepare` and `ClientView::render`, then
waits for the GPU to finish (`device.poll(wait_indefinitely)`). The render time includes that wait.
The frame time is the sum of the three steps. The bench does not read the PNG back.

Output, one line on stdout:

- `frames`: timed frames (N).
- `p50_ms`, `p95_ms`, `max_ms`: frame time percentiles (nearest rank) and the worst frame.
- `update_p50_ms`, `prepare_p50_ms`, `render_p50_ms`: per-step medians.
- `peak_rss_kib`: `VmHWM` from `/proc/self/status` (0 where there is no procfs).
- `cache_bytes`, `cache_entries`: the session typed cache's retained bytes and entry count
  (`ResourceStore::cache_stats`) after the run.

## Commands

Build the viewer in the `profiling` profile (release code, thin LTO, line tables). Every cargo
command runs under the shared lock and memory cap:

```
cd AC2_Rusted-perf-measure
flock /tmp/ac2-cargo.lock systemd-run --user --scope -p MemoryMax=8G -p MemorySwapMax=2G --quiet cargo build --profile profiling -p world-viewer -j 8
export AC2_DAT_DIR=/home/jokerfactor/Downloads/AC2_Legions-DVD-analysis/dataset-ac2
target/profiling/world-viewer --walk 4 --bench 300
target/profiling/world-viewer --fly --bench 300
target/profiling/world-viewer --walk 60 --bench 300
```

The bench is built without the `profiling` feature, so the puffin scopes are compiled out. To
profile the client with puffin, build `ac2` with `--features profiling` and connect a puffin
viewer to `127.0.0.1:8585` while it runs.

## Results

Measured 2026-10-27 at commit `cbf485b` (the `perf-measure` base, `main` HEAD at the time) with the
changes in this commit applied. Headless Vulkan device on the measuring machine. Each run is a
single process.

| Scenario | Bench line |
|---|---|
| Default start, `--walk 4 --bench 300` | `bench frames=300 p50_ms=0.820 p95_ms=0.949 max_ms=4.711 update_p50_ms=0.062 prepare_p50_ms=0.140 render_p50_ms=0.617 peak_rss_kib=239020 cache_bytes=22631582 cache_entries=11900` |
| Fly, `--fly --bench 300` | `bench frames=300 p50_ms=0.737 p95_ms=0.870 max_ms=5.558 update_p50_ms=0.062 prepare_p50_ms=0.133 render_p50_ms=0.535 peak_rss_kib=214972 cache_bytes=20252021 cache_entries=11270` |
| Long walk, `--walk 60 --bench 300` | `bench frames=300 p50_ms=0.871 p95_ms=1.138 max_ms=10.791 update_p50_ms=0.064 prepare_p50_ms=0.144 render_p50_ms=0.664 peak_rss_kib=249308 cache_bytes=23266326 cache_entries=11963` |

A repeat of the first scenario gave p50 0.834 ms and an identical cache size (22631582 bytes,
11900 entries), so the medians vary by about 2 percent between runs.

## Known gaps

- The long-walk scenario did not travel far. The walker ended near its start (origin
  `[76.2, 90.0]` after 60 simulated seconds, against `[79.7, 104.4]` after 4 seconds), so it does
  not show RAM growth over a long traverse. A route that covers more blocks is needed for that.
- Only the headless viewer is measured. The windowed client's present cost, the network loop and
  the script VM are not in the bench.
- The bench runs one scene per process; the three scenarios are not averaged over repeated runs.
