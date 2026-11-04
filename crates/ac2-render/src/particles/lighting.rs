//! `FillVerts::GetLighting` (admin 0x005D26D0, inlined into Legions 0x00595F90
//! `ParticleEmitter::FillVertexBuffer`): a system whose material's template asks for particle
//! lighting (`Ac2ParticleSystemDesc::particle_lighting`) has each particle's colour multiplied by
//! the lights `SceneTool::GetLightsForPosition` found at the system's first render position.
//!
//! The sum starts at `RGBAColor_Black` (0, 0, 0, 1) and adds, for each of the first five lights,
//! `LightSource::GetDiffuse` times `LightSource::CalculateAttenuation` at the particle, plus the
//! light's `m_cAmbient` (always black: no light property writes it). So a particle with no light
//! around it draws black, and its alpha is kept.

use ac2_math::Vec3;

/// `GetLighting` reads at most five lights (`while (i < 5)`).
const MAX_LIGHTS: usize = 5;

/// One light as `GetLighting` reads it, at the frame's client time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleLight {
    /// `m_vOrigin` in world AC axes.
    pub origin: Vec3,
    /// `LightSource::GetDiffuse` times the diffuse intensity now (`m_wDiffuseIntensity`): the
    /// light's `m_cDiffuse` for a point or directional light; for a sunlight-type light (type
    /// `0x40000003`) the sun's `m_cAmbient`, which is black.
    pub diffuse: [f32; 3],
    /// `m_wRange` now for a point or sunlight-type light (types `0x40000001` and `0x40000003`):
    /// the attenuation is `1 / (10 d^2 / range^2 + 1)`. `None` for a directional light (the sun,
    /// `0x40000002`), which is not attenuated.
    pub range: Option<f32>,
}

impl ParticleLight {
    /// `GetDiffuse() * CalculateAttenuation(distance)` at `at`.
    fn at(&self, at: Vec3) -> [f32; 3] {
        let attenuation = match self.range {
            Some(range) => {
                let distance = self.origin.distance(at);
                1.0 / ((distance / (range * range)) * distance * 10.0 + 1.0)
            }
            None => 1.0,
        };
        self.diffuse.map(|c| c * attenuation)
    }
}

/// `FillVerts::GetLighting`: `color`'s red, green and blue times the light the first five of
/// `lights` give a particle at `at`.
pub(crate) fn light_particle(lights: &[ParticleLight], at: Vec3, color: &mut [f32; 4]) {
    let mut sum = [0.0_f32; 3];
    for light in lights.iter().take(MAX_LIGHTS) {
        let lit = light.at(at);
        for (total, add) in sum.iter_mut().zip(lit) {
            *total += add;
        }
    }
    for (channel, light) in color.iter_mut().zip(sum) {
        *channel *= light;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(x: f32, diffuse: f32, range: f32) -> ParticleLight {
        ParticleLight {
            origin: Vec3::new(x, 0.0, 0.0),
            diffuse: [diffuse; 3],
            range: Some(range),
        }
    }

    /// No light: the particle is black, its alpha kept (`RGBAColor_Black` is opaque).
    #[test]
    fn an_unlit_particle_draws_black_with_its_alpha() {
        let mut color = [0.8, 0.6, 0.4, 0.5];
        light_particle(&[], Vec3::ZERO, &mut color);
        assert_eq!(color, [0.0, 0.0, 0.0, 0.5]);
    }

    /// A point light attenuates by `1 / (10 d^2 / r^2 + 1)`; the sun (directional) does not.
    #[test]
    fn point_lights_attenuate_and_the_sun_does_not() {
        let mut color = [1.0; 4];
        light_particle(&[point(5.0, 0.5, 5.0)], Vec3::ZERO, &mut color);
        // d = r: 1 / 11.
        assert!((color[0] - 0.5 / 11.0).abs() < 1e-6);
        let sun = ParticleLight {
            origin: Vec3::ZERO,
            diffuse: [0.9, 0.8, 0.7],
            range: None,
        };
        let mut color = [1.0; 4];
        light_particle(&[sun], Vec3::new(1000.0, 0.0, 0.0), &mut color);
        assert_eq!(color, [0.9, 0.8, 0.7, 1.0]);
    }

    /// Only the first five lights count.
    #[test]
    fn only_five_lights_count() {
        let lights = vec![point(0.0, 0.1, 5.0); 7];
        let mut color = [1.0; 4];
        light_particle(&lights, Vec3::ZERO, &mut color);
        assert!((color[0] - 0.5).abs() < 1e-6);
    }
}
