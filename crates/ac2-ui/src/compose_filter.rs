//! Resampling a UI image drawn at another size (an improvement over retail's `SurfaceWindow::Blit`,
//! which picked the nearest source pixel and so showed stretched art as blocks and shrunk art
//! with dropped rows). Each axis has its taps: a shrunk axis averages the source pixels each
//! destination pixel covers (a box filter, weighted by the part of each it covers), a grown one
//! interpolates between the two nearest pixel centres (bilinear). Pixels are mixed in
//! premultiplied alpha, so a transparent pixel's colour never bleeds into its neighbours.

use crate::compose::UiPixels;

/// One destination pixel's source pixels along an axis, with their weights (summing to 1).
pub(crate) type Taps = Vec<(i32, f32)>;

/// The taps of destination pixels `from..from + count` of an axis `dst` long drawn from the
/// source pixels `s0..s0 + src` (never outside them).
pub(crate) fn taps(s0: i32, src: i32, dst: i32, from: i32, count: i32) -> Vec<Taps> {
    let scale = f64::from(src) / f64::from(dst);
    let last = src - 1;
    (from..from + count)
        .map(|d| {
            if scale > 1.0 {
                let (a, b) = (f64::from(d) * scale, f64::from(d + 1) * scale);
                let mut taps = Vec::new();
                let mut i = a.floor() as i32;
                while f64::from(i) < b && i <= last {
                    let cover = (f64::from(i + 1).min(b) - f64::from(i).max(a)) / scale;
                    if cover > 0.0 {
                        taps.push((s0 + i, cover as f32));
                    }
                    i += 1;
                }
                taps
            } else {
                let c = ((f64::from(d) + 0.5) * scale - 0.5).clamp(0.0, f64::from(last));
                let i = c.floor() as i32;
                let t = (c - f64::from(i)) as f32;
                if t > 0.0 && i < last {
                    vec![(s0 + i, 1.0 - t), (s0 + i + 1, t)]
                } else {
                    vec![(s0 + i, 1.0)]
                }
            }
        })
        .collect()
}

/// The pixel the taps mix, straight alpha again.
pub(crate) fn sample(image: &UiPixels, xs: &Taps, ys: &Taps) -> [u8; 4] {
    let width = image.width as usize;
    let mut sum = [0.0f32; 4];
    for &(y, wy) in ys {
        for &(x, wx) in xs {
            let at = (y as usize * width + x as usize) * 4;
            let p = &image.rgba[at..at + 4];
            let w = wx * wy * f32::from(p[3]) / 255.0;
            for c in 0..3 {
                sum[c] += f32::from(p[c]) * w;
            }
            sum[3] += w;
        }
    }
    if sum[3] <= 0.0 {
        return [0; 4];
    }
    [
        (sum[0] / sum[3]).round().min(255.0) as u8,
        (sum[1] / sum[3]).round().min(255.0) as u8,
        (sum[2] / sum[3]).round().min(255.0) as u8,
        (sum[3] * 255.0).round().min(255.0) as u8,
    ]
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    fn image(width: u32, height: u32, rgba: Vec<u8>) -> UiPixels {
        UiPixels {
            width,
            height,
            rgba: Arc::from(rgba),
        }
    }

    #[test]
    fn every_pixels_taps_weigh_one_and_stay_inside_the_source() {
        for (src, dst) in [(4, 10), (10, 4), (7, 7), (1, 9), (9, 1)] {
            for t in taps(2, src, dst, 0, dst) {
                let sum: f32 = t.iter().map(|&(_, w)| w).sum();
                assert!((sum - 1.0).abs() < 1e-4, "{src}->{dst}: {sum}");
                assert!(t.iter().all(|&(i, _)| (2..2 + src).contains(&i)));
            }
        }
    }

    #[test]
    fn a_same_size_axis_takes_each_pixel_once() {
        let t = taps(0, 5, 5, 0, 5);
        assert!(t
            .iter()
            .enumerate()
            .all(|(i, t)| *t == vec![(i as i32, 1.0)]));
    }

    #[test]
    fn a_grown_edge_blends_and_a_transparent_neighbour_does_not_tint_it() {
        // A red opaque pixel beside a clear green one: the blend fades the alpha, never the hue.
        let img = image(2, 1, vec![255, 0, 0, 255, 0, 255, 0, 0]);
        let xs = taps(0, 2, 4, 0, 4);
        let ys = taps(0, 1, 1, 0, 1);
        let row: Vec<[u8; 4]> = xs.iter().map(|x| sample(&img, x, &ys[0])).collect();
        assert_eq!(row[0], [255, 0, 0, 255]);
        assert!(row[2][3] > 0 && row[2][3] < 255, "{row:?}");
        assert_eq!(&row[2][..3], &[255, 0, 0], "no green bleeds in: {row:?}");
        assert_eq!(row[3][3], 0);
    }
}
