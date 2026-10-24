//! Point lights for one object. `SceneTool::CacheStaticLights` sorts the scene's static lights by
//! `LightSource::GetWeight` at their distance to the object (heaviest first), keeps up to six that
//! are in range (`distance < max range + object radius`) and on the same side of the
//! indoor/outdoor divide, and `SceneTool::ApplyLights` hands them to the device through
//! `LightSource::FillRenderLight`; `RenderDeviceD3D::SetupVertexShaderConstants` keeps the first
//! four point lights for the vertex program.

use ac2_math::Vec3;
use ac2_world::{Ac2LightKind, Ac2StaticLight};

/// `CacheStaticLights` stops once six lights are cached.
const MAX_CACHED_LIGHTS: usize = 6;
/// `VSLightState m_aPtLights[4]`.
pub const MAX_POINT_LIGHTS: usize = 4;

/// A point light as the vertex program's constants carry it, in world AC axes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointLight {
    pub origin: Vec3,
    /// `m_wRange` value (at least 0.0002).
    pub range: f32,
    /// `m_cDiffuse * m_wDiffuseIntensity * m_fOpacity`.
    pub diffuse: [f32; 3],
    /// `m_cSpecular * m_fOpacity` (white by default).
    pub specular: [f32; 3],
    /// `m_fAttenConstant`, `m_fAttenDistLinear`, `m_fAttenDistSquared` (`1, 0, 10 / range^2`).
    pub attenuation: [f32; 3],
}

/// A static light of the scene, placed in world space.
#[derive(Debug, Clone, PartialEq)]
pub struct SceneLight {
    pub light: Ac2StaticLight,
    /// World AC position of the light (its landblock origin plus its block-space origin).
    pub origin: Vec3,
    /// Whether the light belongs to an interior cell.
    pub indoor: bool,
}

/// The point lights the device gets for an object at `position` with bounding radius `radius`.
pub fn object_point_lights(
    scene: &[SceneLight],
    position: Vec3,
    radius: f32,
    indoor: bool,
) -> Vec<PointLight> {
    let mut candidates: Vec<(f32, f32, &SceneLight)> = scene
        .iter()
        .filter(|light| light.light.kind == Ac2LightKind::Point)
        .map(|light| {
            let distance = light.origin.distance(position);
            (weight(&light.light, distance), distance, light)
        })
        .collect();
    // `_STL::sort` with `GetWeight`: heaviest first.
    candidates.sort_by(|a, b| b.0.total_cmp(&a.0));
    candidates
        .into_iter()
        .filter(|(_, distance, light)| {
            light.indoor == indoor && *distance < max_range(&light.light) + radius
        })
        .take(MAX_CACHED_LIGHTS)
        .take(MAX_POINT_LIGHTS)
        .map(|(_, _, light)| fill_render_light(light))
        .collect()
}

/// `LightSource::RefreshMaxRange`: the upper bound of the range waveform.
fn max_range(light: &Ac2StaticLight) -> f32 {
    light.range
}

/// `LightSource::GetWeight` for a point light: `(range - d) / range * intensity * opacity *
/// mean diffuse`, capped at 1.
fn weight(light: &Ac2StaticLight, distance: f32) -> f32 {
    let mean = (light.color[2] + light.color[1] + light.color[0]) * 0.333_333_34;
    let range = light.range;
    (((range - distance) / range) * light.intensity * mean).min(1.0)
}

/// `LightSource::FillRenderLight` for a static point light.
fn fill_render_light(scene: &SceneLight) -> PointLight {
    let light = &scene.light;
    let range = light.range.max(0.0002);
    PointLight {
        origin: scene.origin,
        range,
        diffuse: light.color.map(|c| c * light.intensity),
        specular: [1.0; 3],
        attenuation: [1.0, 0.0, 10.0 / (range * range)],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ac2_core::CellId;

    fn light(x: f32, range: f32, indoor: bool) -> SceneLight {
        SceneLight {
            light: Ac2StaticLight {
                cell: CellId::new(0x005D_0100),
                kind: Ac2LightKind::Point,
                origin: Vec3::new(x, 0.0, 0.0),
                direction: Vec3::ZERO,
                range,
                intensity: 0.5,
                color: [1.0; 3],
                lit: Box::new([]),
            },
            origin: Vec3::new(x, 0.0, 0.0),
            indoor,
        }
    }

    #[test]
    fn lights_in_range_heaviest_first_capped_at_four() {
        let scene: Vec<_> = (0..8).map(|i| light(i as f32, 10.0, true)).collect();
        let lights = object_point_lights(&scene, Vec3::ZERO, 1.0, true);
        assert_eq!(lights.len(), 4);
        assert_eq!(lights[0].origin.x, 0.0);
        assert_eq!(lights[3].origin.x, 3.0);
        assert_eq!(lights[0].attenuation, [1.0, 0.0, 0.1]);
        assert_eq!(lights[0].diffuse, [0.5; 3]);
        // Out of range, or on the other side of the indoor divide.
        assert!(object_point_lights(&[light(20.0, 10.0, true)], Vec3::ZERO, 1.0, true).is_empty());
        assert!(object_point_lights(&[light(1.0, 10.0, true)], Vec3::ZERO, 1.0, false).is_empty());
    }
}
