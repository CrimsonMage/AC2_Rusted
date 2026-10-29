//! `LScapeLighting::UpdateLighting`: the day's light and fog at the viewer, blended with the
//! region's fog areas. Each corner of the viewer's landblock carries a fog type
//! (`GetFogCorners`); each type's `LightFogInfo` is combined with the day's values by its rules
//! (`CalcCombineFog`), and the four results are blended by where the viewer stands in the block
//! (`CornerInterp`). Under the block's water the water's depth-keyed fog takes over the fog
//! (`CalcWaterLight`).

use ac2_math::Vec3;
use ac2_world::{
    BlockFogCorners, FogCombine, FogCombineType, LightFogInfo, Rgba, SkyEnvironmentSample,
    WaterLightInfo,
};

use crate::SceneLighting;

/// The day's light and fog as `LScapeLighting` keeps them (`m_finalDirBright`,
/// `m_finalDirColor`, `m_finalAmbBright`, `m_finalAmbColor`, `m_finalFogColor`, its range and
/// `m_finalFogSky`), with the light's travel direction.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Final {
    direction: Vec3,
    dir_bright: f32,
    dir_color: Rgba,
    amb_bright: f32,
    amb_color: Rgba,
    fog_color: Rgba,
    fog_min: f32,
    fog_max: f32,
    fog_sky: f32,
}

impl Final {
    fn from_sky(sample: &SkyEnvironmentSample) -> Self {
        let fog = sample.world_fog();
        Self {
            direction: Vec3::from_array(sample.light_vector()),
            dir_bright: sample.directional_brightness(),
            dir_color: sample.directional_color(),
            amb_bright: sample.ambient_brightness(),
            amb_color: sample.ambient_color(),
            fog_color: fog.color(),
            fog_min: fog.min_distance(),
            fog_max: fog.max_distance(),
            fog_sky: fog.sky(),
        }
    }
}

/// What `LScapeLighting::UpdateLighting` blends over the day at the viewer: the fog areas at the
/// corners of its landblock with its place in the block (fractions of the edge), and, when it is
/// under the block's water, the water's depth-keyed light and fog with its depth.
#[derive(Debug, Clone, Copy, Default)]
pub struct ViewerSurroundings<'a> {
    pub fog: Option<(&'a BlockFogCorners, [f32; 2])>,
    pub under_water: Option<(&'a [WaterLightInfo], f32)>,
}

/// `LScapeLighting::UpdateLighting`: `lighting` (the day's, from `sample`) with the fog areas'
/// light and fog blended in, then, under water, the water's fog in place of the fog (the light
/// stays).
pub fn update_lighting(
    lighting: SceneLighting,
    sample: &SkyEnvironmentSample,
    around: ViewerSurroundings<'_>,
) -> SceneLighting {
    let mut last = Final::from_sky(sample);
    if let Some((corners, at)) = around.fog {
        let combined = [corners.sw, corners.se, corners.nw, corners.ne]
            .map(|corner| calc_combine_fog(&last, &corner));
        last.take(&corner_interp(&combined, at), true);
    }
    if let Some((layers, depth)) = around.under_water {
        if let Some(water) = calc_water_light(&last, layers, depth) {
            last.take(&water, false);
        }
    }
    let rgb = |c: Rgba| [c[0], c[1], c[2]];
    SceneLighting {
        light_dir: last.direction * last.dir_bright,
        dir_color: rgb(last.dir_color),
        dir_brightness: last.dir_bright,
        ambient: rgb(last.amb_color).map(|c| (f64::from(c) * f64::from(last.amb_bright)) as f32),
        fog_color: rgb(last.fog_color),
        fog_range: (last.fog_min, last.fog_max),
        fog_sky: last.fog_sky,
        ..lighting
    }
}

impl Final {
    /// The `m_final*` fields from a blended `LightFogInfo`: the fog always, the light with
    /// `light` (under water only the fog is taken).
    fn take(&mut self, info: &LightFogInfo, light: bool) {
        if light {
            self.dir_bright = info.light.dir_bright;
            self.dir_color = info.light.dir_color;
            self.amb_bright = info.light.amb_bright;
            self.amb_color = info.light.amb_color;
        }
        self.fog_color = info.fog.fog_color;
        self.fog_min = info.fog.fog_min;
        self.fog_max = info.fog.fog_max;
        self.fog_sky = info.fog.fog_sky;
    }
}

/// `LScapeLighting::CalcWaterLight` at `depth` under the surface: the layer pair the search
/// lands on, each combined over the last light (`CalcCombineFog`), blended by depth. The search
/// keeps the last layer deeper than `depth` (each match overwrites the pair: the layer and the
/// one before it); with none, the last layer alone. `None` without layers.
fn calc_water_light(last: &Final, layers: &[WaterLightInfo], depth: f32) -> Option<LightFogInfo> {
    let count = layers.len();
    if count == 0 {
        return None;
    }
    let (above, below) = layers
        .iter()
        .enumerate()
        .rfind(|(_, layer)| depth < layer.depth)
        .map_or((count - 1, count - 1), |(index, _)| {
            (index.saturating_sub(1), index)
        });
    let water_above = calc_combine_fog(last, &layers[above].water_light);
    if above == below {
        return Some(water_above);
    }
    let water_below = calc_combine_fog(last, &layers[below].water_light);
    let t = (f64::from(depth) - f64::from(layers[below].depth))
        / (f64::from(layers[above].depth) - f64::from(layers[below].depth));
    let mix = |a: f32, b: f32| (f64::from(a) * t + f64::from(b) * (1.0 - t)) as f32;
    let mix_color = |a: Rgba, b: Rgba| [mix(a[0], b[0]), mix(a[1], b[1]), mix(a[2], b[2]), b[3]];
    let mut out = water_below;
    out.light.dir_bright = mix(water_above.light.dir_bright, water_below.light.dir_bright);
    out.light.dir_color = mix_color(water_above.light.dir_color, water_below.light.dir_color);
    out.light.amb_bright = mix(water_above.light.amb_bright, water_below.light.amb_bright);
    out.light.amb_color = mix_color(water_above.light.amb_color, water_below.light.amb_color);
    out.fog.fog_type = ac2_world::LSCAPE_FOG_LINEAR;
    out.fog.fog_min = mix(water_above.fog.fog_min, water_below.fog.fog_min);
    out.fog.fog_max = mix(water_above.fog.fog_max, water_below.fog.fog_max);
    out.fog.fog_sky = mix(water_above.fog.fog_sky, water_below.fog.fog_sky);
    out.fog.fog_color = mix_color(water_above.fog.fog_color, water_below.fog.fog_color);
    Some(out)
}

/// `LScapeLighting::Combine` of one value.
fn combine(first: f32, second: f32, rule: &FogCombine) -> f32 {
    let (first64, second64) = (f64::from(first), f64::from(second));
    match rule.combine_type {
        FogCombineType::MultiplyColors => (first64 * second64) as f32,
        FogCombineType::AddColors => {
            (second64 * f64::from(rule.color_mult_second)
                + first64 * f64::from(rule.color_mult_first)) as f32
        }
        FogCombineType::UseFirstColor => first,
        FogCombineType::UseSecondColor => second,
        FogCombineType::UseMaximumColor => {
            if first <= second {
                second
            } else {
                first
            }
        }
        FogCombineType::UseMinimumColor => {
            if first <= second {
                first
            } else {
                second
            }
        }
    }
}

/// `LScapeLighting::Combine` of a colour, channel by channel.
fn combine_color(first: Rgba, second: Rgba, rule: &FogCombine) -> Rgba {
    [0, 1, 2, 3].map(|i| combine(first[i], second[i], rule))
}

/// `LScapeLighting::CalcDayLight`: the direct colour by the light's length plus the ambient
/// colour by its brightness, scaled down to its brightest channel when out of range
/// (`RGBAColor::IsValid`, `Normalize`); the direct colour's alpha.
fn calc_day_light(day: &Final) -> Rgba {
    let length = f64::from((day.direction * day.dir_bright).length());
    let mut color = [0, 1, 2].map(|i| {
        (f64::from(day.dir_color[i]) * length
            + f64::from(day.amb_color[i]) * f64::from(day.amb_bright)) as f32
    });
    let alpha = day.dir_color[3];
    let valid = color
        .iter()
        .chain(std::iter::once(&alpha))
        .all(|c| (-0.0002..=1.0002).contains(c));
    if !valid {
        let max = color.iter().copied().fold(f32::MIN, f32::max);
        if color.iter().any(|c| *c != 0.0) && max != 0.0 {
            let scale = 1.0 / f64::from(max);
            color = color.map(|c| (f64::from(c) * scale) as f32);
        }
    }
    [color[0], color[1], color[2], alpha]
}

/// `LScapeLighting::CalcCombineFog`: one corner's light and fog over the day's.
fn calc_combine_fog(day: &Final, fog_in: &LightFogInfo) -> LightFogInfo {
    let mut out = *fog_in;
    out.light.dir_bright = combine(
        day.dir_bright,
        fog_in.light.dir_bright,
        &fog_in.combine_direct_brightness,
    );
    out.light.dir_color = combine_color(
        day.dir_color,
        fog_in.light.dir_color,
        &fog_in.combine_direct_color,
    );
    out.light.amb_bright = combine(
        day.amb_bright,
        fog_in.light.amb_bright,
        &fog_in.combine_ambient_brightness,
    );
    out.light.amb_color = combine_color(
        day.amb_color,
        fog_in.light.amb_color,
        &fog_in.combine_ambient_color,
    );
    let day_color = calc_day_light(day);
    let day_fog = combine_color(day_color, day.fog_color, &fog_in.combine_day_fog);
    out.fog.fog_color = combine_color(day_fog, fog_in.fog.fog_color, &fog_in.combine_distance_fog);
    out
}

/// `LScapeLighting::CornerInterp`: the four corners (south-west, south-east, north-west,
/// north-east) blended by `at`, clamped to the block; the alphas are the south-west corner's.
fn corner_interp(corners: &[LightFogInfo; 4], at: [f32; 2]) -> LightFogInfo {
    let sw = &corners[0];
    let (x, y) = (
        f64::from(at[0].clamp(0.0, 1.0)),
        f64::from(at[1].clamp(0.0, 1.0)),
    );
    let weights = [(1.0 - x) * (1.0 - y), x * (1.0 - y), (1.0 - x) * y, x * y];
    let blend = |pick: &dyn Fn(&LightFogInfo) -> f32| {
        corners
            .iter()
            .zip(weights)
            .map(|(corner, weight)| f64::from(pick(corner)) * weight)
            .sum::<f64>() as f32
    };
    let blend_color = |pick: &dyn Fn(&LightFogInfo) -> Rgba, alpha: f32| {
        let color = [0, 1, 2].map(|i| blend(&|c| pick(c)[i]));
        [color[0], color[1], color[2], alpha]
    };
    let mut out = *sw;
    out.light.dir_bright = blend(&|c| c.light.dir_bright);
    out.light.dir_color = blend_color(&|c| c.light.dir_color, sw.light.dir_color[3]);
    out.light.amb_bright = blend(&|c| c.light.amb_bright);
    out.light.amb_color = blend_color(&|c| c.light.amb_color, sw.light.amb_color[3]);
    out.fog.fog_type = ac2_world::LSCAPE_FOG_LINEAR;
    out.fog.fog_min = blend(&|c| c.fog.fog_min);
    out.fog.fog_max = blend(&|c| c.fog.fog_max);
    out.fog.fog_sky = blend(&|c| c.fog.fog_sky);
    out.fog.fog_color = blend_color(&|c| c.fog.fog_color, sw.fog.fog_color[3]);
    out
}

#[cfg(test)]
#[path = "landscape_lighting_tests.rs"]
mod tests;
