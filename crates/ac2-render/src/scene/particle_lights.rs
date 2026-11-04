//! `SceneTool::GetLightsForPosition` (admin 0x005726B0, Legions 0x00542B60) as
//! `ParticleSystem::FillVertexBuffer` calls it for a lit system: radius 0 at the system's first
//! render position. `CacheStaticLights` picks the resident blocks' static lights on the focus's
//! side, `AddDynamicLights` adds the FX lights in range, `SortLights` orders them all by
//! `GetWeight` (heaviest first), and outside (the focus cell below `LandDefs::first_envcell_id`)
//! the sun (`LightSource::s_spSunlight`, a directional light, `SceneTool::Startup` type
//! `0x40000002`, coloured and scaled by `SceneTool::SetSunlight`) goes last.

use std::collections::HashMap;

use ac2_math::Vec3;

use super::object_lights::ObjectLights;
use super::{BlockInfo, BlockKey};
use crate::object::{particle_static_lights_in, LightContext, SunTerms};
use crate::particles::ParticleLight;

/// What the particle lighting reads of the scene this frame.
pub(super) struct SceneLights<'a> {
    pub(super) blocks: &'a HashMap<BlockKey, BlockInfo>,
    pub(super) object_lights: &'a ObjectLights,
    pub(super) sun: Option<SunTerms>,
    pub(super) now: f64,
}

impl SceneLights<'_> {
    /// The lights `GetLightsForPosition` lists for a focus at `position`, inside (`indoor`) or
    /// out, in the order `FillVerts::GetLighting` reads them.
    pub(super) fn at(&self, position: Vec3, indoor: bool) -> Vec<ParticleLight> {
        let context = LightContext {
            time: self.now,
            sun: self.sun,
        };
        let mut lights = particle_static_lights_in(
            self.blocks
                .values()
                .filter_map(|block| block.scene_lights.as_deref())
                .flatten(),
            position,
            indoor,
            context,
        );
        lights.extend(
            self.object_lights
                .particle_lights(self.now, position, indoor),
        );
        lights.sort_by(|a, b| b.0.total_cmp(&a.0));
        let mut lights: Vec<ParticleLight> = lights.into_iter().map(|(_, light)| light).collect();
        if let (false, Some(sun)) = (indoor, self.sun) {
            lights.push(ParticleLight {
                origin: Vec3::ZERO,
                diffuse: sun.color.map(|c| c * sun.intensity),
                range: None,
            });
        }
        lights
    }
}
