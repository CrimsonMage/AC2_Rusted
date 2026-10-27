//! `--bench N`: N timed headless frames after the warm-up. Each frame runs the view's update,
//! prepare and render steps, then waits for the GPU to finish it; the render time includes that
//! wait. Each step is also timed on its own.

use std::time::{Duration, Instant};

use ac2_client::view::ClientView;
use ac2_client::ClientFrame;
use ac2_db::ResourceStore;
use ac2_gpu::{FrameTarget, Gpu};

/// The load budget each frame's update takes from the scene's loader (as the window does).
const LOAD_BUDGET: Duration = Duration::from_millis(4);

/// The timings of one bench run, in milliseconds, and the memory it left behind.
pub struct Report {
    pub frames: usize,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub max_ms: f64,
    pub update_p50_ms: f64,
    pub prepare_p50_ms: f64,
    pub render_p50_ms: f64,
    pub peak_rss_kib: u64,
    pub cache_bytes: usize,
    pub cache_entries: usize,
    /// GPU texture bytes uploaded, and of those the BC blocks (`ac2_gpu::uploaded_texture_bytes`).
    pub texture_bytes: u64,
    pub texture_block_bytes: u64,
    /// Object draw calls the last frame laid out.
    pub draws: usize,
    /// World instances the last frame culled.
    pub culled: usize,
}

/// Renders `frames` frames of `frame` into `target` and reports their times.
pub fn run(
    view: &mut ClientView,
    gpu: &Gpu,
    store: &ResourceStore,
    frame: &ClientFrame,
    target: &FrameTarget,
    aspect: f32,
    frames: u32,
) -> Report {
    let mut total = Vec::with_capacity(frames as usize);
    let mut update = Vec::with_capacity(frames as usize);
    let mut prepare = Vec::with_capacity(frames as usize);
    let mut render = Vec::with_capacity(frames as usize);
    for index in 0..frames {
        let started = Instant::now();
        view.update(gpu, frame, LOAD_BUDGET);
        let after_update = Instant::now();
        view.prepare(gpu, frame, aspect, f64::from(index) / 60.0);
        let after_prepare = Instant::now();
        view.render(gpu, &target.color_view, &target.depth_view, true);
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        let finished = Instant::now();
        update.push(ms(after_update - started));
        prepare.push(ms(after_prepare - after_update));
        render.push(ms(finished - after_prepare));
        total.push(ms(finished - started));
    }
    let cache = store.cache_stats();
    Report {
        frames: total.len(),
        p50_ms: percentile(&mut total, 0.50),
        p95_ms: percentile(&mut total, 0.95),
        max_ms: percentile(&mut total, 1.0),
        update_p50_ms: percentile(&mut update, 0.50),
        prepare_p50_ms: percentile(&mut prepare, 0.50),
        render_p50_ms: percentile(&mut render, 0.50),
        peak_rss_kib: peak_rss_kib(),
        cache_bytes: cache.retained_bytes(),
        cache_entries: cache.entry_count(),
        texture_bytes: ac2_gpu::uploaded_texture_bytes().0,
        texture_block_bytes: ac2_gpu::uploaded_texture_bytes().1,
        draws: view.object_draw_count(),
        culled: view.culled_count(),
    }
}

impl Report {
    /// The one-line summary the baseline doc quotes.
    pub fn line(&self) -> String {
        format!(
            "bench frames={} p50_ms={:.3} p95_ms={:.3} max_ms={:.3} update_p50_ms={:.3} \
             prepare_p50_ms={:.3} render_p50_ms={:.3} peak_rss_kib={} cache_bytes={} \
             cache_entries={} texture_bytes={} texture_block_bytes={} draws={} culled={}",
            self.frames,
            self.p50_ms,
            self.p95_ms,
            self.max_ms,
            self.update_p50_ms,
            self.prepare_p50_ms,
            self.render_p50_ms,
            self.peak_rss_kib,
            self.cache_bytes,
            self.cache_entries,
            self.texture_bytes,
            self.texture_block_bytes,
            self.draws,
            self.culled,
        )
    }
}

fn ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

/// The nearest-rank percentile `p` (0 to 1) of `samples`; 0 when there are none. Sorts them.
fn percentile(samples: &mut [f64], p: f64) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    samples.sort_by(f64::total_cmp);
    let rank = ((p * samples.len() as f64).ceil() as usize).clamp(1, samples.len());
    samples[rank - 1]
}

/// `VmHWM` (the peak resident set, in KiB) from `/proc/self/status`; 0 where there is no procfs.
pub fn peak_rss_kib() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| {
            status
                .lines()
                .find_map(|line| line.strip_prefix("VmHWM:"))
                .and_then(|value| value.split_whitespace().next())
                .and_then(|kib| kib.parse().ok())
        })
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::percentile;

    #[test]
    fn nearest_rank_percentiles() {
        let mut samples: Vec<f64> = (1..=20).map(f64::from).collect();
        assert_eq!(percentile(&mut samples, 0.50), 10.0);
        assert_eq!(percentile(&mut samples, 0.95), 19.0);
        assert_eq!(percentile(&mut samples, 1.0), 20.0);
        assert_eq!(percentile(&mut [], 0.5), 0.0);
    }
}
