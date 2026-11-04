//! Point lights for one object. `SceneTool::CacheStaticLights` sorts the scene's static lights by
//! `LightSource::GetWeight` at their distance to the object (heaviest first), keeps up to six that
//! are in range (`distance < max range + object radius`) and on the same side of the
//! indoor/outdoor divide, and `SceneTool::ApplyLights` hands them to the device through
//! `LightSource::FillRenderLight`; `RenderDeviceD3D::SetupVertexShaderConstants` keeps the first
//! four point lights for the vertex program.

use ac2_math::Vec3;
use ac2_world::{Ac2LightKind, Ac2StaticLight};

use crate::particles::ParticleLight;

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

/// The sun terms a sunlight-type light (`LightSource::m_Type` `0x40000003`) takes at run time
/// (`SceneTool::SetSunlight`, `SceneTool::SetOutdoorAmbient`): the sun's colour and diffuse
/// intensity, and the outdoor ambient (`WorldRender::SetupEnvironment`'s `sunAmbient`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SunTerms {
    pub color: [f32; 3],
    pub intensity: f32,
    pub ambient: [f32; 3],
}

/// What a pick of an object's lights reads besides the lights: the client time the waveforms are
/// sampled at and, when the scene has one, the sun a sunlight-type light follows (without it those
/// lights are left out).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LightContext {
    pub time: f64,
    pub sun: Option<SunTerms>,
}

/// The point lights the device gets for an object at `position` with bounding radius `radius`.
pub fn object_point_lights(
    scene: &[SceneLight],
    position: Vec3,
    radius: f32,
    indoor: bool,
) -> Vec<PointLight> {
    object_point_lights_in(
        scene.iter(),
        position,
        radius,
        indoor,
        LightContext::default(),
    )
}

/// [`object_point_lights`] over lights held in several places (the resident blocks'), read at
/// `context`'s time and sun. `SceneTool::CacheStaticLights` keeps the lights of the list on the
/// object's side of the indoor/outdoor divide (`SceneTool::LinkLight` files each by its cell);
/// a sunlight-type light only lights an object that stands inside.
pub fn object_point_lights_in<'a>(
    scene: impl Iterator<Item = &'a SceneLight>,
    position: Vec3,
    radius: f32,
    indoor: bool,
    context: LightContext,
) -> Vec<PointLight> {
    cache_static_lights(scene, position, radius, indoor, context)
        .into_iter()
        .take(MAX_POINT_LIGHTS)
        .map(|(_, light)| fill_render_light(light, context))
        .collect()
}

/// `SceneTool::CacheStaticLights` for `FillVerts::GetLighting` (`GetLightsForPosition` with
/// radius 0): the same pick, each light with its `GetWeight` as the particles read it.
pub fn particle_static_lights_in<'a>(
    scene: impl Iterator<Item = &'a SceneLight>,
    position: Vec3,
    indoor: bool,
    context: LightContext,
) -> Vec<(f32, ParticleLight)> {
    cache_static_lights(scene, position, 0.0, indoor, context)
        .into_iter()
        .map(|(weight, scene)| {
            let light = &scene.light;
            let diffuse = match light.kind {
                // `LightSource::GetDiffuse` of a sunlight-type light is the sun's `m_cAmbient`,
                // which nothing sets (black).
                Ac2LightKind::Sunlight => [0.0; 3],
                _ => light.color.map(|c| c * light.intensity_at(context.time)),
            };
            let range = match light.kind {
                Ac2LightKind::Directional => None,
                _ => Some(light.range_at(context.time)),
            };
            (
                weight,
                ParticleLight {
                    origin: scene.origin,
                    diffuse,
                    range,
                },
            )
        })
        .collect()
}

/// `SceneTool::CacheStaticLights`: the lights of the list on the object's side of the
/// indoor/outdoor divide, heaviest first, in range, at most six, with their weights.
fn cache_static_lights<'a>(
    scene: impl Iterator<Item = &'a SceneLight>,
    position: Vec3,
    radius: f32,
    indoor: bool,
    context: LightContext,
) -> Vec<(f32, &'a SceneLight)> {
    let mut candidates: Vec<(f32, f32, &SceneLight)> = scene
        .filter(|light| match light.light.kind {
            Ac2LightKind::Point => true,
            Ac2LightKind::Sunlight => indoor && context.sun.is_some(),
            Ac2LightKind::Directional => false,
        })
        .map(|light| {
            let distance = light.origin.distance(position);
            (weight(&light.light, distance, context), distance, light)
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
        .map(|(weight, _, light)| (weight, light))
        .collect()
}

/// `LightSource::RefreshMaxRange`: the upper bound of the range waveform (zero where it has
/// none, the field's initial value).
fn max_range(light: &Ac2StaticLight) -> f32 {
    light.range_wave.upper_bound().unwrap_or(0.0)
}

/// `LightSource::GetWeight` for a point or sunlight-type light: `(range - d) / range` times the
/// intensity, the opacity and the mean diffuse, capped at 1. The range is the waveform's value now, the intensity
/// its upper bound (1 where it has none); a sunlight-type light takes the sun's colour and
/// multiplies in the sun's intensity bound.
fn weight(light: &Ac2StaticLight, distance: f32, context: LightContext) -> f32 {
    let (color, sun_intensity) = match (light.kind, context.sun) {
        (Ac2LightKind::Point, _) => (light.color, 1.0),
        (Ac2LightKind::Sunlight, Some(sun)) => (sun.color, sun.intensity),
        _ => return 1.0,
    };
    let mean = (color[2] + color[1] + color[0]) * 0.333_333_34;
    let range = light.range_at(context.time);
    let intensity = light.intensity_wave.upper_bound().unwrap_or(1.0);
    (((range - distance) / range) * sun_intensity * intensity * mean).min(1.0)
}

/// `LightSource::FillRenderLight` for a static point light.
fn fill_render_light(scene: &SceneLight, context: LightContext) -> PointLight {
    let mut point = static_point_light_at(&scene.light, scene.origin, context.time);
    if let (Ac2LightKind::Sunlight, Some(sun)) = (scene.light.kind, context.sun) {
        // The sun's colour times its intensity plus the outdoor ambient, then the light's own
        // intensity (`FillRenderLight`'s type `0x40000003` branch).
        let intensity = scene.light.intensity_at(context.time);
        point.diffuse =
            std::array::from_fn(|i| (sun.color[i] * sun.intensity + sun.ambient[i]) * intensity);
    }
    point
}

/// `LightSource::FillRenderLight` for a static point light standing at `origin` in the scene, at
/// the start of its waveforms (the lights cached with a cell or an object keep these values).
pub fn static_point_light(light: &Ac2StaticLight, origin: Vec3) -> PointLight {
    static_point_light_at(light, origin, 0.0)
}

/// `LightSource::FillRenderLight` at client time `time`: the range and the diffuse intensity are
/// the values of `m_wRange` and `m_wDiffuseIntensity` then, so a light whose waveforms animate
/// breathes from frame to frame.
pub fn static_point_light_at(light: &Ac2StaticLight, origin: Vec3, time: f64) -> PointLight {
    let range = light.range_at(time).max(0.0002);
    PointLight {
        origin,
        range,
        diffuse: light.color.map(|c| c * light.intensity_at(time)),
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
                range_wave: ac2_math::Waveform::constant(range),
                intensity_wave: ac2_math::Waveform::constant(0.5),
                color: [1.0; 3],
                lit: Box::new([]),
                corona: None,
            },
            origin: Vec3::new(x, 0.0, 0.0),
            indoor,
        }
    }

    /// A cell light whose waveforms animate fills a different render light at another time
    /// (`LightSource::FillRenderLight` samples `m_wRange` and `m_wDiffuseIntensity`); a constant
    /// one never changes.
    #[test]
    fn a_static_lights_range_and_intensity_follow_their_waveforms() {
        use ac2_math::waveform::kind;
        let mut animated = light(0.0, 6.0, true);
        animated.light.range_wave = ac2_math::Waveform {
            kind: kind::SINE,
            base: 6.0,
            amplitude: 2.0,
            frequency: std::f32::consts::FRAC_PI_2,
            ..Default::default()
        };
        animated.light.intensity_wave = ac2_math::Waveform {
            kind: kind::SINE,
            base: 0.5,
            amplitude: 0.25,
            frequency: std::f32::consts::FRAC_PI_2,
            ..Default::default()
        };
        assert!(animated.light.is_range_animating());
        let peak = static_point_light_at(&animated.light, animated.origin, 1.0);
        let trough = static_point_light_at(&animated.light, animated.origin, 3.0);
        assert!((peak.range - 8.0).abs() < 1e-4 && (trough.range - 4.0).abs() < 1e-4);
        assert!((peak.diffuse[0] - 0.75).abs() < 1e-4 && (trough.diffuse[0] - 0.25).abs() < 1e-4);
        let steady = light(0.0, 6.0, true);
        assert!(!steady.light.is_range_animating());
        assert_eq!(
            static_point_light_at(&steady.light, steady.origin, 0.0),
            static_point_light_at(&steady.light, steady.origin, 99.0)
        );
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

    /// `FillRenderLight`'s type `0x40000003` branch: a sunlight-type light is a point light of the
    /// sun's colour times its intensity plus the outdoor ambient, scaled by the light's own
    /// intensity; it lights only objects inside, and only where the scene has a sun.
    #[test]
    fn a_sunlight_type_light_follows_the_sun_inside() {
        let mut sun_light = light(1.0, 10.0, true);
        sun_light.light.kind = Ac2LightKind::Sunlight;
        let scene = [sun_light];
        let sun = SunTerms {
            color: [1.0, 0.5, 0.0],
            intensity: 0.8,
            ambient: [0.1; 3],
        };
        let at = |indoor, sun| {
            let context = LightContext { time: 0.0, sun };
            object_point_lights_in(scene.iter(), Vec3::ZERO, 1.0, indoor, context)
        };
        assert!(at(true, None).is_empty());
        assert!(at(false, Some(sun)).is_empty());
        let lit = at(true, Some(sun));
        assert_eq!(lit.len(), 1);
        // (colour * 0.8 + 0.1) * the light's intensity 0.5.
        for (got, want) in lit[0].diffuse.iter().zip([0.45, 0.25, 0.05]) {
            assert!((got - want).abs() < 1e-5, "{got} {want}");
        }
    }
}
