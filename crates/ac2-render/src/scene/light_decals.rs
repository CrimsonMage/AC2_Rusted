//! `WorldRender::CreateLandscapePointLightDecals`: each frame, every point light standing over the
//! landscape within its range of the ground pools its light there, as a decal drawn once
//! (`DecalEffectManager::CreateDecalEffect`) of a material chosen by the time of day. The lights
//! are the frame's FX lights (`SceneTool::m_aDynamicLights`) and the own lights of the interior
//! cells that draw (`CEnvCell::LinkStaticLights`); a light inside a building pools on the ground
//! under it when the ground is within its reach, as retail's `LandDefs::adjust_to_outside` lets it.

use std::collections::HashMap;

use ac2_core::DataId;
use ac2_math::Vec3;
use ac2_world::land_block_size;

use super::loader::BlockKey;
use super::WorldScene;
use crate::decal::DecalSpec;
use crate::object::PointLight;

/// The materials' enums (`FileArray::GetDIDFromEnum(.., 0x40000002)`) by day, dusk and night.
pub(crate) const LIGHT_DECAL_ENUMS: [u32; 3] = [0x4000_000D, 0x4000_002B, 0x4000_002C];

/// Which of `LIGHT_DECAL_ENUMS` the time of day (`GameTime::present_time_of_day`, a fraction of
/// the day) picks: night before 0.2 and after 0.95, dusk before 0.3 and after 0.85, else day.
pub(crate) fn material_index(time_of_day: f32) -> usize {
    if !(0.2..=0.95).contains(&time_of_day) {
        2
    } else if !(0.3..=0.85).contains(&time_of_day) {
        1
    } else {
        0
    }
}

/// The pool `light` casts on ground at height `ground` under it: a box twice its range across
/// around the light, its colour the light's diffuse (with intensity and opacity) faded by the
/// height over the range, clamped to 0..1 (`RGBAColor::CapHigh`). `None` when the light is not
/// above the ground or is out of reach of it.
pub(crate) fn light_pool(light: &PointLight, ground: f32, material: DataId) -> Option<DecalSpec> {
    if ground >= light.origin.z {
        return None;
    }
    let height = f64::from(light.origin.z) - f64::from(ground);
    let reach = 1.0 - height / f64::from(light.range);
    if reach <= 0.0 {
        return None;
    }
    let [r, g, b] = light
        .diffuse
        .map(|c| (f64::from(c) * reach).clamp(0.0, 1.0) as f32);
    Some(DecalSpec {
        material,
        lifespan: 1.0,
        size: light.range + light.range,
        direction: Vec3::Y,
        color: [r, g, b, 1.0],
        draw_once: true,
        origin: light.origin,
    })
}

impl WorldScene {
    /// Pools the frame's point lights on the ground under them (drawn this frame only), when
    /// light source effects are on and the materials have loaded.
    pub(super) fn pool_lights(
        &mut self,
        interiors: &[BlockKey],
        interior_cells: &HashMap<BlockKey, Vec<u32>>,
    ) {
        if !self.light_source_effects {
            return;
        }
        let Some(materials) = self.light_decal_materials else {
            return;
        };
        let time_of_day = self
            .world
            .region()
            .and_then(|region| {
                region
                    .sky_day_context_at(self.time_of_day)
                    .map(|(fraction, _)| fraction)
            })
            .unwrap_or(0.5);
        let material = materials[material_index(time_of_day)];
        if material.raw() == 0 {
            return;
        }
        let mut lights = self.object_lights.all(self.particles.now());
        for key in interiors {
            let Some(all) = self.blocks.get(key).and_then(|b| b.cell_lights.as_deref()) else {
                continue;
            };
            let drawn = interior_cells.get(key);
            lights.extend(
                all.iter()
                    .filter(|(cell, _)| drawn.is_none_or(|ids| ids.contains(cell)))
                    .map(|(_, light)| *light),
            );
        }
        let block_size = land_block_size();
        let now = self.particles.now();
        for light in &lights {
            let (x, y) = (light.origin.x, light.origin.y);
            let Some(ground) = self
                .heights_at(x, y, block_size)
                .and_then(|heights| heights.height_at(x, y))
            else {
                continue;
            };
            if let Some(spec) = light_pool(light, ground, material) {
                let ground_triangles = self.decal_ground(spec.origin);
                self.decals.create(spec, now, &ground_triangles);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn light(z: f32, range: f32) -> PointLight {
        PointLight {
            origin: Vec3::new(10.0, 20.0, z),
            range,
            diffuse: [1.5, 0.5, 0.0],
            specular: [1.0; 3],
            attenuation: [1.0, 0.0, 10.0 / (range * range)],
        }
    }

    #[test]
    fn the_time_of_day_picks_day_dusk_or_night() {
        assert_eq!(material_index(0.1), 2);
        assert_eq!(material_index(0.96), 2);
        assert_eq!(material_index(0.25), 1);
        assert_eq!(material_index(0.9), 1);
        assert_eq!(material_index(0.5), 0);
    }

    #[test]
    fn a_light_pools_by_its_height_over_the_ground_within_its_range() {
        let material = DataId::new(0x3200_0001);
        let pool = light_pool(&light(102.0, 8.0), 100.0, material).expect("in reach");
        assert_eq!(pool.size, 16.0);
        assert_eq!(pool.origin, Vec3::new(10.0, 20.0, 102.0));
        assert!(pool.draw_once);
        // 1 - 2 / 8 of the diffuse, clamped.
        assert_eq!(pool.color, [1.0, 0.375, 0.0, 1.0]);
        // Out of reach, at the ground, or under it: no pool.
        assert!(light_pool(&light(109.0, 8.0), 100.0, material).is_none());
        assert!(light_pool(&light(100.0, 8.0), 100.0, material).is_none());
        assert!(light_pool(&light(90.0, 8.0), 100.0, material).is_none());
    }
}
