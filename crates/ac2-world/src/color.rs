//! AC2 `RGBAColor`: float RGBA in `[0, 1]`, unpacked from ARGB words, with the HSB conversions
//! the landscape tints and the sky interpolate through.

/// Float RGBA, in `RGBAColor` field order.
pub type Rgba = [f32; 4];

pub(crate) const WHITE: Rgba = [1.0; 4];

/// `RGBAColor::SetColor32`: an ARGB word.
pub(crate) fn set_color32(argb: u32) -> Rgba {
    let channel = |shift: u32| ((argb >> shift) & 0xFF) as f32 * 0.003_921_569;
    [channel(16), channel(8), channel(0), channel(24)]
}

/// `RGBAColor::RGBtoHSB`: hue in `[0, 6)`, saturation and brightness in `[0, 1]`.
pub(crate) fn rgb_to_hsb([r, g, b, _]: Rgba) -> (f32, f32, f32) {
    if r == b && r == g {
        return (0.0, 0.0, r);
    }
    if g <= r {
        if r < b {
            return ((r - g) / (b - g) + 4.0, 1.0 - g / b, b);
        }
        if b <= g {
            return ((g - b) / (r - b), 1.0 - b / r, r);
        }
        return (6.0 - (b - g) / (r - g), 1.0 - g / r, r);
    }
    if g < b {
        return (4.0 - (g - r) / (b - r), 1.0 - r / b, b);
    }
    if b <= r {
        return (2.0 - (r - b) / (g - b), 1.0 - b / g, g);
    }
    ((b - r) / (g - r) + 2.0, 1.0 - r / g, g)
}

/// `RGBAColor::HSBtoRGB`, clamped by `CapLow`/`CapHigh`; alpha is always 1.
pub(crate) fn hsb_to_rgb(hue: f32, saturate: f32, bright: f32) -> Rgba {
    let rgb = if saturate > 0.0 && bright > 0.0 {
        let low = (1.0 - saturate) * bright;
        let hue = hue.rem_euclid(6.0);
        let ramp = |t: f32| t * (bright - low) + low;
        match hue {
            h if h < 1.0 => [bright, ramp(h), low],
            h if h < 2.0 => [ramp(2.0 - h), bright, low],
            h if h < 3.0 => [low, bright, ramp(h - 2.0)],
            h if h < 4.0 => [low, ramp(4.0 - h), bright],
            h if h < 5.0 => [ramp(h - 4.0), low, bright],
            h => [bright, low, ramp(6.0 - h)],
        }
    } else {
        [bright; 3]
    };
    let [r, g, b] = rgb.map(|c| c.clamp(0.0, 1.0));
    [r, g, b, 1.0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsb_round_trips_every_hue_sector() {
        for rgb in [
            [0.8, 0.2, 0.1],
            [0.1, 0.7, 0.3],
            [0.2, 0.3, 0.9],
            [0.9, 0.1, 0.6],
        ] {
            let (h, s, b) = rgb_to_hsb([rgb[0], rgb[1], rgb[2], 1.0]);
            assert!((0.0..6.0).contains(&h));
            let back = hsb_to_rgb(h, s, b);
            for (a, e) in back.iter().zip(rgb) {
                assert!((a - e).abs() < 1e-5, "{rgb:?} -> {back:?}");
            }
        }
    }

    #[test]
    fn set_color32_reads_argb() {
        assert_eq!(
            set_color32(0x80FF_0000),
            [1.0, 0.0, 0.0, 128.0 * 0.003_921_569]
        );
    }
}
