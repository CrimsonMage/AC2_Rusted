//! Statistics the snapshot tests check their frames with. Frames are RGBA8 with
//! alpha ignored; `width` is the frame's pixel width.

/// Mean RGB over the frame, 0 to 255.
pub fn mean_rgb(rgba: &[u8]) -> [f32; 3] {
    let mut sum = [0.0_f64; 3];
    for px in rgba.chunks_exact(4) {
        for c in 0..3 {
            sum[c] += f64::from(px[c]);
        }
    }
    let n = (rgba.len() / 4) as f64;
    sum.map(|s| (s / n) as f32)
}

/// Mean Rec. 709 luma over the frame (of the encoded values), 0 to 255.
pub fn mean_luma(rgba: &[u8]) -> f32 {
    let [r, g, b] = mean_rgb(rgba);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// The share of pixels with any channel at 255 (clipped highlights).
pub fn clipped_fraction(rgba: &[u8]) -> f32 {
    let clipped = rgba
        .chunks_exact(4)
        .filter(|px| px[..3].contains(&255))
        .count();
    clipped as f32 / (rgba.len() / 4) as f32
}

/// Mean colour of the rows `rows` (half-open) and columns `cols` (half-open) of a frame.
pub fn region_mean(
    rgba: &[u8],
    width: usize,
    rows: std::ops::Range<usize>,
    cols: std::ops::Range<usize>,
) -> [f32; 3] {
    let mut sum = [0.0_f64; 3];
    let mut n = 0.0_f64;
    for row in rows {
        for col in cols.clone() {
            let i = (row * width + col) * 4;
            for c in 0..3 {
                sum[c] += f64::from(rgba[i + c]);
            }
            n += 1.0;
        }
    }
    sum.map(|s| (s / n) as f32)
}

/// Mean chroma spread (max channel minus min channel) over the region: a saturated colour has a
/// wide spread, a fog-grey one a narrow spread.
pub fn region_saturation(
    rgba: &[u8],
    width: usize,
    rows: std::ops::Range<usize>,
    cols: std::ops::Range<usize>,
) -> f32 {
    let mut sum = 0.0_f64;
    let mut n = 0.0_f64;
    for row in rows {
        for col in cols.clone() {
            let i = (row * width + col) * 4;
            let px = &rgba[i..i + 3];
            let max = *px.iter().max().unwrap_or(&0);
            let min = *px.iter().min().unwrap_or(&0);
            sum += f64::from(max - min);
            n += 1.0;
        }
    }
    (sum / n) as f32
}

/// The share of a frame that may clip at 255: the HDR scene's single tone map rolls stacked
/// additive sky layers off below white, so little should.
pub const MAX_CLIPPED: f32 = 0.03;

/// A frame of a shot: its brightness is in a sane band and few highlights clip. Returns its mean
/// luma.
pub fn check_frame(name: &str, frame: &[u8]) -> f32 {
    let luma = mean_luma(frame);
    assert!(
        (40.0..200.0).contains(&luma),
        "{name}: mean luma {luma:.1} outside the sane band"
    );
    let clipped = clipped_fraction(frame);
    assert!(
        clipped < MAX_CLIPPED,
        "{name}: {:.2}% of pixels clip at 255",
        clipped * 100.0
    );
    luma
}

/// The sky of an outdoor frame, its top right, is a gradient (higher pixels bluer and brighter
/// than the band near the horizon) and saturated, not the fog grey the retail sky shows.
pub fn check_sky_gradient(name: &str, modern: &[u8], width: usize) {
    let half = width / 2..width;
    let top = region_mean(modern, width, 0..40, half.clone());
    let near_horizon = region_mean(modern, width, 100..140, half.clone());
    let gradient = (mean_of(top) - mean_of(near_horizon)).abs();
    assert!(
        gradient > 2.0,
        "{name}: the sky has no vertical gradient ({gradient:.1})"
    );
    let saturation = region_saturation(modern, width, 0..40, half);
    assert!(
        saturation > 6.0,
        "{name}: the sky is fog grey (spread {saturation:.1})"
    );
}

fn mean_of(rgb: [f32; 3]) -> f32 {
    0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2]
}
