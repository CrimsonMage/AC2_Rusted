//! GPU time per render pass, from timestamp queries. A measuring aid with no retail counterpart:
//! only on a device made by [`crate::Gpu::headless_timed`] (the world viewer's `--gpu-times`) that
//! has `TIMESTAMP_QUERY`. Each timed pass takes a start and end slot; a frame's
//! slots are resolved and read back together, then summed per label.

use std::sync::Mutex;

/// Timed passes per frame; a pass past this many draws untimed.
const MAX_PASSES: u32 = 32;
const QUERY_BYTES: u64 = std::mem::size_of::<u64>() as u64;

pub(crate) struct PassTimer {
    queries: wgpu::QuerySet,
    resolve: wgpu::Buffer,
    readback: wgpu::Buffer,
    /// The label of each pass given slots since the last read, in slot order.
    labels: Mutex<Vec<&'static str>>,
    /// Nanoseconds per timestamp tick (`Queue::get_timestamp_period`).
    period_ns: f64,
}

impl PassTimer {
    pub(crate) fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let count = MAX_PASSES * 2;
        let size = u64::from(count) * QUERY_BYTES;
        Self {
            queries: device.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some("ac2 pass timestamps"),
                ty: wgpu::QueryType::Timestamp,
                count,
            }),
            resolve: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ac2 pass timestamps resolve"),
                size,
                usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            }),
            readback: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ac2 pass timestamps readback"),
                size,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            labels: Mutex::new(Vec::new()),
            period_ns: f64::from(queue.get_timestamp_period()),
        }
    }

    /// The next pass's start and end slots under `label`, or `None` once the frame's slots are used.
    pub(crate) fn writes(
        &self,
        label: &'static str,
    ) -> Option<wgpu::RenderPassTimestampWrites<'_>> {
        let mut labels = self.labels.lock().ok()?;
        let index = u32::try_from(labels.len())
            .ok()
            .filter(|&i| i < MAX_PASSES)?;
        labels.push(label);
        Some(wgpu::RenderPassTimestampWrites {
            query_set: &self.queries,
            beginning_of_pass_write_index: Some(index * 2),
            end_of_pass_write_index: Some(index * 2 + 1),
        })
    }

    /// Resolves the slots given out since the last read, waits for them, and returns each
    /// label's summed milliseconds in first-use order. Blocks on the GPU.
    pub(crate) fn read(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Vec<(&'static str, f64)> {
        let labels = match self.labels.lock() {
            Ok(mut labels) => std::mem::take(&mut *labels),
            Err(_) => return Vec::new(),
        };
        if labels.is_empty() {
            return Vec::new();
        }
        let count = labels.len() as u32 * 2;
        let bytes = u64::from(count) * QUERY_BYTES;
        let mut encoder = device.create_command_encoder(&Default::default());
        encoder.resolve_query_set(&self.queries, 0..count, &self.resolve, 0);
        encoder.copy_buffer_to_buffer(&self.resolve, 0, &self.readback, 0, bytes);
        queue.submit([encoder.finish()]);
        let slice = self.readback.slice(..bytes);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        if device.poll(wgpu::PollType::wait_indefinitely()).is_err() {
            return Vec::new();
        }
        let ticks: Vec<u64> = match slice.get_mapped_range() {
            Ok(view) => bytemuck::cast_slice(&view).to_vec(),
            Err(_) => Vec::new(),
        };
        self.readback.unmap();
        sum_by_label(&labels, &ticks, self.period_ns)
    }
}

/// Each label's summed pass time in milliseconds, in first-use order. `ticks` holds a start and an
/// end per label; a pass whose end precedes its start (a reset counter) counts as zero.
fn sum_by_label(
    labels: &[&'static str],
    ticks: &[u64],
    period_ns: f64,
) -> Vec<(&'static str, f64)> {
    let mut sums: Vec<(&'static str, f64)> = Vec::new();
    for (label, pair) in labels.iter().zip(ticks.chunks_exact(2)) {
        let ms = pair[1].saturating_sub(pair[0]) as f64 * period_ns / 1.0e6;
        match sums.iter_mut().find(|(l, _)| l == label) {
            Some((_, total)) => *total += ms,
            None => sums.push((label, ms)),
        }
    }
    sums
}

#[cfg(test)]
mod tests {
    use super::sum_by_label;

    #[test]
    fn sums_repeated_labels_in_first_use_order_and_floors_reset_pairs() {
        let labels = ["scene", "ui", "scene", "shadow"];
        let ticks = [0, 1_000_000, 5, 6, 10, 2_000_010, 9, 3];
        let sums = sum_by_label(&labels, &ticks, 1.0);
        assert_eq!(
            sums,
            vec![("scene", 3.0), ("ui", 0.000_001), ("shadow", 0.0)]
        );
    }
}
