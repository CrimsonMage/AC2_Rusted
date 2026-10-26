//! The shared helpers of the world shaders (`shading.wgsl`, `object_modern.wgsl`, the terrain
//! shaders). Colour inputs are sRGB-encoded in the data; the shaders decode them, light and fog in
//! linear light and write linear radiance into the HDR scene, which `tonemap.wgsl` encodes once.
//! The Rust copies below exist only so the unit tests can pin the curves to their known values.

/// The frames' `shading` vec4, kept in the uniform layouts: x 1 (one shading path), y exposure,
/// z saturation, w contrast (the tone map's constants, read nowhere else now).
pub(crate) const FRAME_SHADING: [f32; 4] = [1.0, 1.0, 1.05, 1.05];

/// The sRGB electro-optical transfer function: an encoded channel to linear light.
pub fn srgb_to_linear(encoded: f32) -> f32 {
    if encoded <= 0.04045 {
        encoded / 12.92
    } else {
        ((encoded + 0.055) / 1.055).powf(2.4)
    }
}

#[cfg(test)]
/// The inverse transfer: linear light to an sRGB-encoded channel.
fn linear_to_srgb(linear: f32) -> f32 {
    if linear <= 0.0031308 {
        linear * 12.92
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
    }
}

/// `MODERN_FOG_HEIGHT_FALLOFF` in `shading.wgsl`: how fast distance fog thins per metre of altitude.
#[cfg(test)]
const FOG_HEIGHT_FALLOFF: f32 = 0.01;

/// The fogged fraction (0 clear, 1 fully fogged) of a point `distance` metres from the eye at
/// `height` metres of altitude, over the region's linear fog `range` (start, end). `modern_fog`
/// in `shading.wgsl`.
#[cfg(test)]
pub(crate) fn modern_fog(distance: f32, height: f32, range: (f32, f32)) -> f32 {
    let span = (range.1 - range.0).max(1e-3);
    let ramp = ((distance - range.0) / span).clamp(0.0, 1.0);
    ramp * (-FOG_HEIGHT_FALLOFF * height.max(0.0)).exp()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32, tolerance: f32) -> bool {
        (a - b).abs() <= tolerance
    }

    #[test]
    fn srgb_transfer_matches_known_values() {
        assert_eq!(srgb_to_linear(0.0), 0.0);
        assert!(close(srgb_to_linear(1.0), 1.0, 1e-6));
        // The linear segment ends at 0.04045 / 12.92 = 0.0031308 in linear light.
        assert!(close(srgb_to_linear(0.04045), 0.04045 / 12.92, 1e-7));
        // Mid grey: sRGB 0.5 is linear 0.214041...
        assert!(close(srgb_to_linear(0.5), 0.214_041_14, 1e-5));
        // sRGB 0.735357 is linear 0.5 (the point Unreal's 18 % grey sits near).
        assert!(close(srgb_to_linear(0.735_357), 0.5, 1e-4));
        assert!(close(linear_to_srgb(0.214_041_14), 0.5, 1e-5));
        assert!(close(linear_to_srgb(0.0031308), 0.0031308 * 12.92, 1e-6));
        assert!(close(linear_to_srgb(1.0), 1.0, 1e-6));
    }

    #[test]
    fn srgb_round_trips_across_the_range() {
        for step in 0..=255 {
            let encoded = step as f32 / 255.0;
            let back = linear_to_srgb(srgb_to_linear(encoded));
            assert!(close(back, encoded, 1e-5), "{encoded} -> {back}");
        }
    }

    #[test]
    fn fog_ramps_linearly_over_the_region_range_at_ground_level() {
        let range = (100.0, 1100.0);
        assert_eq!(modern_fog(50.0, 0.0, range), 0.0);
        assert!(close(modern_fog(600.0, 0.0, range), 0.5, 1e-5));
        assert_eq!(modern_fog(5000.0, 0.0, range), 1.0);
    }

    #[test]
    fn fog_thins_with_altitude() {
        let range = (0.0, 1000.0);
        let ground = modern_fog(800.0, 0.0, range);
        let high = modern_fog(800.0, 300.0, range);
        assert!(high < ground * 0.1, "{high} vs {ground}");
        assert_eq!(modern_fog(800.0, -50.0, range), ground);
    }
}
