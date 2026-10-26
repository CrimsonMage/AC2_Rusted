//! `CDayDesc` sampling: the two `SkyTimeOfDay` keys around a time of day, their ratio, and the
//! interpolated `LightInfo`, light vector, `FogInfo` and rain (`CDayDesc::GetLighting`,
//! `CDayDesc::GetWorldFog`, `CDayDesc::GetRain`).

use super::{SkyEnvironmentSample, SkyFogDesc, SkyLightingDesc, SkyRain, SkyTimeOfDay};
use crate::color::{hsb_to_rgb, rgb_to_hsb, Rgba, WHITE};

/// Lighting and fog at `time_of_day` (a fraction of the day). A day without keys gets retail's
/// fallback light (ambient 0.7, direct 0.1 along `(1, 0, -0.4)`) and fog (0..10000, white).
pub(super) fn sample(keys: &[SkyTimeOfDay], time_of_day: f32) -> SkyEnvironmentSample {
    let (Some(before), Some(after)) = (
        before_time(keys, time_of_day),
        after_time(keys, time_of_day),
    ) else {
        return SkyEnvironmentSample {
            light: SkyLightingDesc {
                dir_brightness: 0.1,
                dir_heading: 0.0,
                dir_pitch: 0.0,
                dir_color: WHITE,
                ambient_brightness: 0.7,
                ambient_color: WHITE,
                shadow_color: WHITE,
            },
            light_vector: [1.0, 0.0, -0.4],
            world_fog: SkyFogDesc {
                min_distance: 0.0,
                max_distance: 10_000.0,
                sky: 0.0,
                color: WHITE,
            },
            rain: SkyRain::default(),
        };
    };
    let ratio = time_ratio(before.begin, after.begin, time_of_day);
    let lerp = |a: f32, b: f32| (b - a) * ratio + a;
    let (b, a) = (&before.lighting, &after.lighting);
    let light = SkyLightingDesc {
        dir_brightness: lerp(b.dir_brightness, a.dir_brightness),
        dir_heading: interp_angle_closest(b.dir_heading, a.dir_heading, ratio, 360.0),
        dir_pitch: interp_angle_closest(b.dir_pitch, a.dir_pitch, ratio, 360.0),
        dir_color: interp_color(b.dir_color, a.dir_color, ratio),
        ambient_brightness: lerp(b.ambient_brightness, a.ambient_brightness),
        ambient_color: interp_color(b.ambient_color, a.ambient_color, ratio),
        shadow_color: interp_color(b.shadow_color, a.shadow_color, ratio),
    };
    let (bf, af) = (&before.world_fog, &after.world_fog);
    SkyEnvironmentSample {
        light_vector: calc_light_vector(light.dir_heading, light.dir_pitch),
        light,
        world_fog: SkyFogDesc {
            min_distance: lerp(bf.min_distance, af.min_distance),
            max_distance: lerp(bf.max_distance, af.max_distance),
            sky: lerp(bf.sky, af.sky),
            color: interp_color(bf.color, af.color, ratio),
        },
        rain: SkyRain {
            amount: lerp(before.rain.amount, after.rain.amount),
            wind_x: lerp(before.rain.wind_x, after.rain.wind_x),
            wind_y: lerp(before.rain.wind_y, after.rain.wind_y),
        },
    }
}

/// `CDayDesc::GetBeforeTime`: the key whose span `[begin, next.begin]` holds the time, else the
/// last key.
pub(super) fn before_time(keys: &[SkyTimeOfDay], time: f32) -> Option<&SkyTimeOfDay> {
    let last = keys.last()?;
    Some(
        keys.windows(2)
            .find(|pair| pair[0].begin <= time && time <= pair[1].begin)
            .map_or(last, |pair| &pair[0]),
    )
}

/// `CDayDesc::GetAfterTime`: the first key beginning after the time, else the first key.
pub(super) fn after_time(keys: &[SkyTimeOfDay], time: f32) -> Option<&SkyTimeOfDay> {
    keys.iter().find(|key| time < key.begin).or(keys.first())
}

/// `CDayDesc::GetTimeRatio`. A forward span outside its keys answers 0.5; a span across midnight
/// wraps.
pub(super) fn time_ratio(before: f32, after: f32, time: f32) -> f32 {
    if before == after {
        return 0.0;
    }
    if before < after {
        return if before <= time && time <= after {
            (time - before) / (after - before)
        } else {
            0.5
        };
    }
    let mut elapsed = time - before;
    if elapsed < 0.0 {
        elapsed += 1.0;
    }
    let span = (after - before) + 1.0;
    if span != 0.0 {
        elapsed / span
    } else {
        0.0
    }
}

/// `CDayDesc::InterpAngleClosest`: interpolate the short way round a circle of `wrap`.
fn interp_angle_closest(before: f32, after: f32, ratio: f32, wrap: f32) -> f32 {
    let delta = after - before;
    if !(before < after) {
        if delta + wrap <= before - after {
            let angle = ((after + wrap) - before) * ratio + before;
            return if angle < wrap { angle } else { angle - wrap };
        }
    } else if (before - after) + wrap <= delta {
        let angle = ((after - wrap) - before) * ratio + before;
        return if angle >= 0.0 { angle } else { angle + wrap };
    }
    delta * ratio + before
}

/// `CDayDesc::InterpColor`: interpolate in HSB, hue the short way round.
pub(super) fn interp_color(before: Rgba, after: Rgba, ratio: f32) -> Rgba {
    let (h1, s1, b1) = rgb_to_hsb(before);
    let (h2, s2, b2) = rgb_to_hsb(after);
    hsb_to_rgb(
        interp_angle_closest(h1, h2, ratio, 6.0),
        (s2 - s1) * ratio + s1,
        (b2 - b1) * ratio + b1,
    )
}

/// `CDayDesc::CalcLightVector(1, heading, pitch)`: `-Vector::FromSpherical(1, heading, pitch)`.
fn calc_light_vector(heading: f32, pitch: f32) -> [f32; 3] {
    let (sin_h, cos_h) = heading.to_radians().sin_cos();
    let (sin_p, cos_p) = pitch.to_radians().sin_cos();
    [-(sin_h * cos_p), -(cos_h * cos_p), -sin_p]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(begin: f32, brightness: f32, heading: f32) -> SkyTimeOfDay {
        SkyTimeOfDay {
            begin,
            lighting: SkyLightingDesc {
                dir_brightness: brightness,
                dir_heading: heading,
                dir_pitch: 45.0,
                dir_color: WHITE,
                ambient_brightness: brightness,
                ambient_color: WHITE,
                shadow_color: WHITE,
            },
            world_fog: SkyFogDesc {
                min_distance: 0.0,
                max_distance: 100.0,
                sky: 0.0,
                color: WHITE,
            },
            replacements: std::sync::Arc::from([]),
            rain: SkyRain {
                amount: brightness,
                wind_x: heading,
                wind_y: 0.0,
            },
        }
    }

    #[test]
    fn keys_interpolate_forward_and_wrap_past_midnight() {
        let keys = [key(0.25, 0.0, 350.0), key(0.75, 1.0, 10.0)];
        let midday = sample(&keys, 0.5);
        assert!((midday.directional_brightness() - 0.5).abs() < 1e-6);
        // The heading takes the short way through north.
        assert!(midday.directional_heading_degrees().abs() < 1e-3);
        // 0.875: from the 0.75 key towards the 0.25 key across midnight, a quarter of the way.
        let late = sample(&keys, 0.875);
        assert!((late.directional_brightness() - 0.75).abs() < 1e-6);
    }

    #[test]
    fn rain_blends_linearly_and_is_dry_without_keys() {
        // `CDayDesc::GetRain`: amount and wind lerp straight (no angle wrap), like the fog.
        let keys = [key(0.25, 0.2, 350.0), key(0.75, 1.0, 10.0)];
        let rain = sample(&keys, 0.5).rain();
        assert!((rain.amount() - 0.6).abs() < 1e-6);
        assert!((rain.wind()[0] - 180.0).abs() < 1e-3);
        assert_eq!(sample(&[], 0.5).rain(), SkyRain::default());
    }

    #[test]
    fn a_forward_span_outside_its_keys_takes_the_midpoint() {
        assert_eq!(time_ratio(0.25, 0.75, 0.9), 0.5);
        assert_eq!(time_ratio(0.25, 0.25, 0.9), 0.0);
    }

    #[test]
    fn light_travels_away_from_the_sun() {
        let [x, y, z] = calc_light_vector(90.0, 30.0);
        assert!(x < -0.8 && y.abs() < 1e-6 && (z + 0.5).abs() < 1e-6);
        assert_eq!(sample(&[], 0.3).light_vector(), [1.0, 0.0, -0.4]);
    }
}
