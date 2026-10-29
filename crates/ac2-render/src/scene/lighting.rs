//! The scene's light and fog: the region's day at the scene's time, as the viewer sees it
//! (`LScapeLighting::UpdateLighting` through `crate::landscape_lighting`).

use ac2_math::Vec3;

use super::{WorldScene, DAY_SECONDS};
use crate::SceneLighting;

impl WorldScene {
    /// The region's lighting at the scene's time of day.
    pub fn lighting(&self) -> SceneLighting {
        self.world
            .region()
            .and_then(|region| region.sample_sky_environment(self.time_of_day))
            .map(|sample| SceneLighting::from_sky(&sample))
            .unwrap_or_default()
    }

    /// The region's lighting at the scene's time of day seen from `viewer`
    /// (`LScapeRender::SetViewerPosition`, `LScapeLighting::UpdateLighting`): the day's, blended
    /// with the fog areas at the corners of the viewer's landblock, and under its water the
    /// water's fog.
    pub fn lighting_at(&self, viewer: Vec3) -> SceneLighting {
        let Some(sample) = self
            .world
            .region()
            .and_then(|region| region.sample_sky_environment(self.time_of_day))
        else {
            return SceneLighting::default();
        };
        let day = SceneLighting::from_sky(&sample);
        let size = self.world.block_size();
        let block = (viewer.truncate() / size).floor();
        let in_range = |c: f32| (0.0..255.0).contains(&c);
        if !(in_range(block.x) && in_range(block.y)) {
            return day;
        }
        let corners = self
            .world
            .region_fog()
            .and_then(|fog| fog.corners(block.x as u32, block.y as u32));
        let at = ((viewer.truncate() - block * size) / size).to_array();
        // `LScapeRender::SetViewerPosition`: the viewer's block's water plane decides whether it
        // is under water.
        let key = (block.x as u8, block.y as u8);
        let water_depth = self
            .heights
            .get(&key)
            .and_then(|heights| heights.water)
            .map(|surface| surface - viewer.z)
            .filter(|depth| *depth > 0.0);
        let layers = water_depth.and_then(|_| {
            self.world
                .block_water_lighting(crate::scene::loader::block_id(key))
        });
        crate::landscape_lighting::update_lighting(
            day,
            &sample,
            crate::landscape_lighting::ViewerSurroundings {
                fog: corners.as_ref().map(|corners| (corners, at)),
                under_water: layers.as_deref().zip(water_depth),
            },
        )
    }

    /// Starts at the brightest daylight of the region's day.
    pub fn set_daytime(&mut self) {
        let Some(region) = self.world.region() else {
            return;
        };
        let brightness = |time: f64| {
            region
                .sample_sky_environment(time)
                .map(|sample| {
                    SceneLighting::from_sky(&sample)
                        .fog_color
                        .iter()
                        .sum::<f32>()
                })
                .unwrap_or(0.0)
        };
        self.time_of_day = (0..48)
            .map(|step| f64::from(step) * DAY_SECONDS / 48.0)
            .max_by(|a, b| brightness(*a).total_cmp(&brightness(*b)))
            .unwrap_or(0.0);
    }
}
