//! The light sources of object FX (`CPartArray::CreateLightSource`). Each is a point light placed
//! on its object at the node's offset (`PHYSICSOBJREP::PrepareObjectForDrawing`: the object's
//! frame times the offset), fading in and out over its fade time, until its lifespan is spent.
//! A light with no lifespan lasts while its object is in the world (`DestroyAllTransientFX`
//! drops them with it).

use std::collections::HashMap;

use ac2_assets::{Ac2FxLight, Ac2LightSource};
use ac2_math::{Frame, Vec3};

use super::ObjectPlacement;
use crate::object::{PointLight, MAX_POINT_LIGHTS};

/// `LightSource::FillRenderLight`'s smallest range.
const MIN_RANGE: f32 = 0.0002;
/// `PrepareObjectForDrawing`'s fade threshold (`0.0002`).
const FADE_EPSILON: f32 = 0.0002;

/// One light an object's FX placed.
#[derive(Debug, Clone, Copy, PartialEq)]
struct FxLight {
    emitter: u64,
    light: Ac2FxLight,
    /// The client time it was created at.
    created: f64,
}

/// The FX lights of the scene and the last frame each emitter stood at.
#[derive(Debug, Default)]
pub(super) struct ObjectLights {
    lights: Vec<FxLight>,
    /// The frame of each emitter that has been placed (kept while its lights run).
    frames: HashMap<u64, Frame>,
}

/// `PrepareObjectForDrawing`'s opacity for a light `elapsed` seconds old: `None` once its
/// lifespan is spent, else a fade in over the first fade time and a fade out over the last.
pub(crate) fn light_opacity(light: &Ac2FxLight, elapsed: f64) -> Option<f32> {
    let lifespan = light.lifespan;
    if lifespan != 0.0 && elapsed >= f64::from(lifespan) {
        return None;
    }
    let elapsed = elapsed as f32;
    if light.fade_time <= FADE_EPSILON {
        return Some(1.0);
    }
    // The fade in ends at `in_end`; the fade out starts at `out_start`.
    let in_end = lifespan.min(light.fade_time);
    let out_start = (lifespan - light.fade_time).max(in_end);
    if in_end >= out_start {
        return Some(1.0);
    }
    if elapsed < 0.0 || elapsed > in_end {
        if out_start <= elapsed && elapsed < lifespan {
            return Some(1.0 - (elapsed - out_start) / (lifespan - out_start));
        }
        return Some(1.0);
    }
    Some((elapsed / in_end).clamp(0.0, 1.0))
}

impl ObjectLights {
    /// `CPartArray::CreateLightSource`: a light on `emitter`, created at `now`.
    pub(super) fn start(&mut self, emitter: u64, light: Ac2FxLight, now: f64) {
        self.prune(now);
        self.lights.push(FxLight {
            emitter,
            light,
            created: now,
        });
    }

    /// `CPartArray::DestroyAllTransientFX`: the object left the world, so its lights go.
    pub(super) fn remove_emitter(&mut self, emitter: u64) {
        self.lights.retain(|light| light.emitter != emitter);
        self.frames.remove(&emitter);
    }

    /// The frame's object placements: each placed emitter's frame is remembered, and spent lights
    /// are dropped.
    pub(super) fn place(&mut self, placements: &[ObjectPlacement], now: f64) {
        for placement in placements {
            self.frames.insert(placement.emitter, placement.frame);
        }
        self.prune(now);
    }

    /// How many lights are running.
    pub(super) fn count(&self) -> usize {
        self.lights.len()
    }

    /// The point lights an object at `position` with bounding radius `radius` sees at `now`:
    /// lights in range (`distance < range + radius`), heaviest first, at most the device's
    /// point lights.
    pub(super) fn point_lights(&self, now: f64, position: Vec3, radius: f32) -> Vec<PointLight> {
        let mut candidates: Vec<(f32, PointLight)> = self
            .lights
            .iter()
            .filter_map(|fx| {
                let frame = self.frames.get(&fx.emitter)?;
                let opacity = light_opacity(&fx.light, now - fx.created)?;
                let origin = frame.local_to_global_point(fx.light.offset);
                let point = fill(&fx.light.source, origin, opacity);
                let distance = origin.distance(position);
                (distance < point.range + radius).then(|| (weight(&point, distance), point))
            })
            .collect();
        // `_STL::sort` with `GetWeight`: heaviest first.
        candidates.sort_by(|a, b| b.0.total_cmp(&a.0));
        candidates
            .into_iter()
            .take(MAX_POINT_LIGHTS)
            .map(|(_, point)| point)
            .collect()
    }

    /// Drops the lights that are spent at `now`.
    fn prune(&mut self, now: f64) {
        self.lights
            .retain(|fx| light_opacity(&fx.light, now - fx.created).is_some());
    }
}

/// `LightSource::FillRenderLight` for an FX light at `origin` with its `opacity`: diffuse is the
/// colour times the intensity and the opacity; the attenuation is `1 / (10 d^2 / range^2 + 1)`.
fn fill(source: &Ac2LightSource, origin: Vec3, opacity: f32) -> PointLight {
    let range = source.range.max(MIN_RANGE);
    PointLight {
        origin,
        range,
        diffuse: source.diffuse.map(|c| c * source.intensity * opacity),
        specular: source.specular.map(|c| c * opacity),
        attenuation: [1.0, 0.0, 10.0 / (range * range)],
    }
}

/// `LightSource::GetWeight` for a point light: `(range - d) / range * mean diffuse`, where the
/// diffuse already carries the intensity and the opacity; capped at 1.
fn weight(point: &PointLight, distance: f32) -> f32 {
    let mean = point.diffuse.iter().sum::<f32>() / 3.0;
    (((point.range - distance) / point.range) * mean).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fx(lifespan: f32, fade_time: f32) -> Ac2FxLight {
        Ac2FxLight {
            offset: Vec3::new(0.0, 0.0, 1.0),
            source: Ac2LightSource::default(),
            lifespan,
            fade_time,
        }
    }

    fn placed(emitter: u64, x: f32) -> ObjectPlacement {
        ObjectPlacement {
            emitter,
            frame: Frame::new(Vec3::new(x, 0.0, 0.0), ac2_math::Quat::IDENTITY),
            scale: Vec3::ONE,
            bones: Vec::new(),
        }
    }

    #[test]
    fn opacity_fades_in_then_out_and_ends_with_the_lifespan() {
        let light = fx(2.0, 0.25);
        assert_eq!(light_opacity(&light, 0.0), Some(0.0));
        assert_eq!(light_opacity(&light, 0.125), Some(0.5));
        assert_eq!(light_opacity(&light, 1.0), Some(1.0));
        let fading = light_opacity(&light, 1.9).expect("alive");
        assert!((fading - 0.4).abs() < 1e-5, "fade out at 1.9 s: {fading}");
        assert_eq!(light_opacity(&light, 2.0), None);
        // No lifespan lasts, and no fade time is full strength.
        assert_eq!(light_opacity(&fx(0.0, 0.25), 1e6), Some(1.0));
        assert_eq!(light_opacity(&fx(2.0, 0.0), 1.0), Some(1.0));
    }

    #[test]
    fn a_light_sits_at_the_object_frame_plus_its_offset() {
        let mut lights = ObjectLights::default();
        lights.start(7, fx(0.0, 0.0), 0.0);
        lights.place(&[placed(7, 10.0)], 0.0);
        let points = lights.point_lights(0.0, Vec3::new(10.0, 0.0, 0.0), 1.0);
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].origin, Vec3::new(10.0, 0.0, 1.0));
        assert_eq!(points[0].attenuation, [1.0, 0.0, 10.0 / 25.0]);
    }

    #[test]
    fn lights_out_of_range_or_of_unplaced_objects_do_not_light() {
        let mut lights = ObjectLights::default();
        lights.start(7, fx(0.0, 0.0), 0.0);
        // Emitter 7 never placed: no light.
        assert!(lights.point_lights(0.0, Vec3::ZERO, 1.0).is_empty());
        lights.place(&[placed(7, 100.0)], 0.0);
        assert!(lights.point_lights(0.0, Vec3::ZERO, 1.0).is_empty());
    }

    #[test]
    fn at_most_the_device_point_lights_and_heaviest_first() {
        let mut lights = ObjectLights::default();
        for emitter in 0..6 {
            lights.start(emitter, fx(0.0, 0.0), 0.0);
        }
        lights.place(
            &(0..6)
                .map(|e| placed(e, e as f32 * 0.5))
                .collect::<Vec<_>>(),
            0.0,
        );
        let points = lights.point_lights(0.0, Vec3::ZERO, 0.0);
        assert_eq!(points.len(), MAX_POINT_LIGHTS);
        assert!(points[0].origin.x <= points[1].origin.x);
    }

    #[test]
    fn spent_lights_are_pruned_and_removed_with_their_object() {
        let mut lights = ObjectLights::default();
        lights.start(1, fx(1.0, 0.0), 0.0);
        lights.start(2, fx(0.0, 0.0), 0.0);
        lights.place(&[placed(1, 0.0), placed(2, 0.0)], 1.5);
        assert_eq!(lights.count(), 1);
        lights.remove_emitter(2);
        assert_eq!(lights.count(), 0);
    }
}
