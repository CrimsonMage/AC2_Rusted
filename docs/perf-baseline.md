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

## Crowd

`--crowd N` (`tools/world-viewer/src/crowd.rs`) adds N idle objects to the headless scene: a
square grid 3 m apart centred on the player, cycling through the race table's models (every race,
male and female), each at the terrain height under it and a fixed heading. They are remote objects
in the frame, in the default bind pose (no animation), so they measure per-object draw and prepare
cost. `--crowd` works with `--snapshot` and `--bench`; the bench times frames after the models have
loaded (`settle`).

Measured 2026-10-27 at commit `1d1ab40` plus the `--crowd` change (this commit), `profiling`
profile, the three runs back to back. The machine's load average was 9 to 10 from other sessions,
so these medians run above the Results table; compare the three rows with each other only. Repeating
`--crowd 0` on the same machine gave p50 1.368 and 1.383 ms.

| Scenario | Bench line |
|---|---|
| `--walk 4 --bench 300 --crowd 0` | `bench frames=300 p50_ms=1.322 p95_ms=1.620 max_ms=6.085 update_p50_ms=0.145 prepare_p50_ms=0.277 render_p50_ms=0.892 peak_rss_kib=240380 cache_bytes=22631582 cache_entries=11900` |
| `--walk 4 --bench 300 --crowd 100` | `bench frames=300 p50_ms=4.421 p95_ms=4.709 max_ms=8.404 update_p50_ms=0.283 prepare_p50_ms=1.616 render_p50_ms=2.526 peak_rss_kib=281304 cache_bytes=31031398 cache_entries=12290` |
| `--walk 4 --bench 300 --crowd 400` | `bench frames=300 p50_ms=8.089 p95_ms=8.872 max_ms=16.446 update_p50_ms=0.234 prepare_p50_ms=3.502 render_p50_ms=4.313 peak_rss_kib=296080 cache_bytes=31031398 cache_entries=12290` |

The 100 and 400 runs cache the same bytes and entries: the 400 objects reuse the models the 100
already loaded. From 100 to 400 objects, frame p50 grows by 3.7 ms (about 1.2 ms per 100 objects;
prepare +1.9 ms, render +1.8 ms).

Snapshot `--snapshot /tmp/crowd-100.png --walk 4 --crowd 100` shows the crowd: the race models stand
in their bind pose (arms out) on the terrain in a grid around the player, several races and both
sexes visible, the player in the middle.

## GPU pass times and VRAM

The bench line ends with `vram_allocated_bytes` and `vram_reserved_bytes`: what the device's
allocator holds after the run (`Gpu::allocator_bytes`; Vulkan reports it, GL and Metal
print 0). `--gpu-times` also times each render pass with timestamp queries (`Gpu::headless_timed`,
`crates/ac2-gpu/src/timing.rs`) and adds `gpu_p50_ms` (the frame's summed passes) and one
`gpu_<label>_p50_ms` per pass label: `scene` (the HDR world), `ui` (tone map and HUD) and
`shadow map` (the static landscape map; 0 at p50 because it redraws only when its key changes).

`--gpu-times` is a separate flag because asking the device for `TIMESTAMP_QUERY` alone raises
the NVIDIA driver's resident memory by about 130 MB, queries issued or not: peak RSS 355 to 416 MB
with the feature against 222 to 224 MB without it, same binary, three runs each. Compare
`peak_rss_kib` only between runs without `--gpu-times`. The client never asks for the feature.

Measured 2026-10-28 on an RTX 5060 Ti (NVIDIA driver, Vulkan), `profiling` profile, load average
14 to 18 from other sessions:

| Scenario | Bench line (tail) |
|---|---|
| `--walk 4 --bench 300` | `p50_ms=0.764 render_p50_ms=0.631 peak_rss_kib=223112 draws=35 vram_allocated_bytes=36528128 vram_reserved_bytes=201326592` |
| `--walk 4 --bench 300 --gpu-times` | `p50_ms=0.750 render_p50_ms=0.624 gpu_p50_ms=0.407 gpu_scene_p50_ms=0.390 gpu_ui_p50_ms=0.018` |
| `--walk 4 --crowd 400 --bench 300 --gpu-times` | `p50_ms=6.116 prepare_p50_ms=3.492 render_p50_ms=2.476 draws=2308 gpu_p50_ms=0.867 gpu_scene_p50_ms=0.841 vram_allocated_bytes=50174976` |

The crowd's cost is on the CPU: of the 2.5 ms `render` step the GPU's passes take 0.87 ms, so the
rest is recording and submitting 2308 draws, and `prepare` takes 3.5 ms more. Instanced draws
(fewer, larger calls) are the first lever, ahead of any GPU-side work.

## Frame workers and instanced object draws

Two changes cut the crowd's CPU cost; the image is unchanged (see below).

- **Frame workers.** `ac2_db::install_frame_workers` (called first in the `ac2` binary and
  `world-viewer`) builds rayon's global pool with `frame_worker_count` threads (logical cores
  minus one, at most 8) and installs its `WorkerLanes`. Until now nothing installed them, so
  `frame_map` always ran inline. `ClientView::prepare` now poses the drawn objects through
  `frame_map` (`placed`, `CPartArray::UpdateBones` per object): 2.8 ms to 0.45 ms per frame for
  the 400-figure crowd.
- **Instanced draws.** The object pass splits its per-draw uniform: what a layer's instances share
  (material, stages, point lights) stays a dynamic-offset uniform, and what each instance owns
  (model matrix, skin, frill normal and shade) moves to a storage array the vertex stage reads at
  `instance_index`. A group's depth-writing layers are laid out by model, part, batch and layer,
  and consecutive layers with the same pipeline, part and uniform become one instanced draw. Each
  layer's uniform is built once at upload (animated and tinted layers still rebuild per frame).
  The crowd's 2308 draws become 665.

Changing the order between objects in the depth-writing queue changes only partly transparent
texels where two objects overlap: snapshots of `--crowd 100` and `--crowd 400` differ from the
previous build on 59 and 16 of 921600 pixels, by at most 1 of 255; `--fly` is identical.

Measured 2026-10-28, `profiling` profile, load average 12 to 20 from other sessions (compare the
rows with each other only):

| `--walk 4 --crowd 400 --bench 600` | p50_ms | prepare_p50_ms | render_p50_ms | draws |
|---|---|---|---|---|
| before | 7.15 to 7.93 | 4.15 to 4.58 | 2.84 to 3.15 | 2308 |
| after | 4.39 to 4.99 | 2.37 to 2.75 | 1.80 to 1.97 | 665 |

Pinned to two cores (`taskset -c 0-1`) the frame workers alone took prepare from 4.58 to 3.64 ms,
and on four cores from 4.30 to 3.29 ms: `available_parallelism` follows the affinity mask, so the
pool shrinks with it and a small machine does not get slower.

## Known gaps

- The long-walk scenario did not travel far. The walker ended near its start (origin
  `[76.2, 90.0]` after 60 simulated seconds, against `[79.7, 104.4]` after 4 seconds), so it does
  not show RAM growth over a long traverse. A route that covers more blocks is needed for that.
- Only the headless viewer is measured. The windowed client's present cost, the network loop and
  the script VM are not in the bench.
- The bench runs one scene per process; the three scenarios are not averaged over repeated runs.
