//! The light sources of object FX (`CPartArray::CreateLightSource`). Each is a point light placed
//! on its object at the node's offset (`PHYSICSOBJREP::PrepareObjectForDrawing`: the object's
//! frame times the offset), fading in and out over its fade time, until its lifespan is spent.
//! A light with no lifespan lasts while its object is in the world (`DestroyAllTransientFX`
//! drops them with it). The light is a copy of the whole `LightSource` (`CPartArray::CreateLightSource`),
//! so its range and diffuse intensity are the animated waveforms `FillRenderLight` samples at the
//! client time of each frame.

use std::collections::HashMap;

use ac2_assets::{Ac2FxLight, Ac2LightSource};
use ac2_math::{Frame, Vec3};

use super::entity_lights::EntityLight;
use super::{BlockKey, ObjectPlacement};
use crate::object::{PointLight, MAX_POINT_LIGHTS};
use crate::particles::ParticleLight;

/// `LightSource::m_Type` of a directional light (`0x40000002`) and of a sunlight-type light
/// (`0x40000003`).
const LIGHT_DIRECTIONAL: u32 = 0x4000_0002;
const LIGHT_SUNLIGHT: u32 = 0x4000_0003;

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

/// One startup light of a landblock entity: it stands where the entity does while the block is
/// resident.
#[derive(Debug, Clone, Copy, PartialEq)]
struct EntityFx {
    block: BlockKey,
    light: Ac2FxLight,
    origin: Vec3,
    created: f64,
}

/// The FX lights of the scene and the last frame each emitter stood at.
#[derive(Debug, Default)]
pub(super) struct ObjectLights {
    lights: Vec<FxLight>,
    /// The startup lights of the resident blocks' entities (outdoors).
    entities: Vec<EntityFx>,
    /// `Render::m_Prefs.LightSourceEffects`: whether the entities' lights light the objects.
    entities_lit: bool,
    /// The frame of each emitter that has been placed (kept while its lights run).
    frames: HashMap<u64, Frame>,
    /// Whether each placed emitter stands in an interior cell: `CPartArray::CreateLightSource`'s
    /// light carries its object's cell (`m_CellID`), which `SceneTool::AddDynamicLights` compares
    /// with the focus's side.
    indoor: HashMap<u64, bool>,
}

/// `LandDefs::first_envcell_id`: the cells from here up are interior ones.
const FIRST_ENV_CELL: u32 = 0x100;

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

    /// The startup lights of the entities of `block`, which became resident at `now`.
    pub(super) fn add_block(&mut self, block: BlockKey, lights: &[EntityLight], now: f64) {
        self.entities.extend(lights.iter().map(|entity| EntityFx {
            block,
            light: entity.light,
            origin: entity.origin,
            created: now,
        }));
    }

    /// `LightSourceEffects` changed: the entities' lights light the objects only while it is on.
    pub(super) fn set_entities_lit(&mut self, lit: bool) {
        self.entities_lit = lit;
    }

    /// The block's entities left the world.
    pub(super) fn remove_block(&mut self, block: &BlockKey) {
        self.entities.retain(|entity| entity.block != *block);
    }

    /// `CPartArray::DestroyAllTransientFX`: the object left the world, so its lights go.
    pub(super) fn remove_emitter(&mut self, emitter: u64) {
        self.lights.retain(|light| light.emitter != emitter);
        self.frames.remove(&emitter);
        self.indoor.remove(&emitter);
    }

    /// The frame's object placements: each placed emitter's frame is remembered, and spent lights
    /// are dropped.
    pub(super) fn place(&mut self, placements: &[ObjectPlacement], now: f64) {
        for placement in placements {
            self.frames.insert(placement.emitter, placement.frame);
            self.indoor
                .insert(placement.emitter, placement.cell & 0xFFFF >= FIRST_ENV_CELL);
        }
        self.prune(now);
    }

    /// How many lights are running.
    pub(super) fn count(&self) -> usize {
        self.lights.len()
    }

    /// The point lights an object at `position` with bounding radius `radius` sees at `now`:
    /// lights on its side (`indoor`) in range (`distance < max range + radius`), heaviest first,
    /// at most the device's point lights.
    pub(super) fn point_lights(
        &self,
        now: f64,
        position: Vec3,
        radius: f32,
        indoor: bool,
    ) -> Vec<PointLight> {
        let mut candidates: Vec<(f32, PointLight)> = self
            .in_range(now, position, radius, indoor)
            .map(|(weight, point, _)| (weight, point))
            .collect();
        // `_STL::sort` with `GetWeight`: heaviest first.
        candidates.sort_by(|a, b| b.0.total_cmp(&a.0));
        candidates
            .into_iter()
            .take(MAX_POINT_LIGHTS)
            .map(|(_, point)| point)
            .collect()
    }

    /// `SceneTool::AddDynamicLights` for `FillVerts::GetLighting` (radius 0): the lights in range
    /// on the focus's side with their `GetWeight`, as the particles read them: `GetDiffuse` times
    /// the diffuse intensity now, without the light's fade (`CalculateAttenuation` does not read
    /// `m_fOpacity`).
    pub(super) fn particle_lights(
        &self,
        now: f64,
        position: Vec3,
        indoor: bool,
    ) -> Vec<(f32, ParticleLight)> {
        self.in_range(now, position, 0.0, indoor)
            .map(|(weight, point, source)| {
                let intensity = source.intensity_at(now);
                let (diffuse, range) = match source.kind {
                    // A directional light is not attenuated.
                    LIGHT_DIRECTIONAL => (source.diffuse.map(|c| c * intensity), None),
                    // `GetDiffuse` of a sunlight-type light is the sun's (black) ambient.
                    LIGHT_SUNLIGHT => ([0.0; 3], Some(source.range_at(now))),
                    _ => (
                        source.diffuse.map(|c| c * intensity),
                        Some(source.range_at(now)),
                    ),
                };
                (
                    weight,
                    ParticleLight {
                        origin: point.origin,
                        diffuse,
                        range,
                    },
                )
            })
            .collect()
    }

    /// The placed lights on `indoor`'s side within reach of an object at `position` with bounding
    /// radius `radius` (`distance < max range + radius`), each with its `GetWeight`, its render
    /// light and its source.
    fn in_range(
        &self,
        now: f64,
        position: Vec3,
        radius: f32,
        indoor: bool,
    ) -> impl Iterator<Item = (f32, PointLight, &Ac2LightSource)> + '_ {
        let reaches = move |light: &Ac2FxLight, point: &PointLight| {
            // `LightSource::RefreshMaxRange`: the range waveform's upper bound.
            let reach = light.source.range_wave.upper_bound().unwrap_or(0.0);
            let distance = point.origin.distance(position);
            (distance < reach + radius).then(|| weight(point, distance))
        };
        let placed = self.lights.iter().filter_map(move |fx| {
            let frame = self.frames.get(&fx.emitter)?;
            if self.indoor.get(&fx.emitter) != Some(&indoor) {
                return None;
            }
            let opacity = light_opacity(&fx.light, now - fx.created)?;
            let origin = frame.local_to_global_point(fx.light.offset);
            let point = fill(&fx.light.source, origin, opacity, now);
            Some((reaches(&fx.light, &point)?, point, &fx.light.source))
        });
        // The entities stand outdoors (`CLandBlock` objects).
        let entities = self
            .entities
            .iter()
            .filter(move |_| !indoor && self.entities_lit)
            .filter_map(move |fx| {
                let opacity = light_opacity(&fx.light, now - fx.created)?;
                let point = fill(&fx.light.source, fx.origin, opacity, now);
                Some((reaches(&fx.light, &point)?, point, &fx.light.source))
            });
        placed.chain(entities)
    }

    /// Every placed point light at `now` (`SceneTool::m_aDynamicLights` of the frame).
    pub(super) fn all(&self, now: f64) -> Vec<PointLight> {
        let entities = self.entities.iter().filter_map(|fx| {
            let opacity = light_opacity(&fx.light, now - fx.created)?;
            (fx.light.source.kind == ac2_assets::LIGHT_POINT)
                .then(|| fill(&fx.light.source, fx.origin, opacity, now))
        });
        self.lights
            .iter()
            .filter(|fx| fx.light.source.kind == ac2_assets::LIGHT_POINT)
            .filter_map(|fx| {
                let frame = self.frames.get(&fx.emitter)?;
                let opacity = light_opacity(&fx.light, now - fx.created)?;
                Some(fill(
                    &fx.light.source,
                    frame.local_to_global_point(fx.light.offset),
                    opacity,
                    now,
                ))
            })
            .chain(entities)
            .collect()
    }

    /// Drops the lights that are spent at `now`.
    fn prune(&mut self, now: f64) {
        self.lights
            .retain(|fx| light_opacity(&fx.light, now - fx.created).is_some());
        self.entities
            .retain(|fx| light_opacity(&fx.light, now - fx.created).is_some());
    }
}

/// `LightSource::FillRenderLight` for an FX light at `origin` with its `opacity` at client time
/// `now`: the range and the intensity are the waveforms' values then; diffuse is the colour times
/// the intensity and the opacity; the attenuation is `1 / (10 d^2 / range^2 + 1)`.
fn fill(source: &Ac2LightSource, origin: Vec3, opacity: f32, now: f64) -> PointLight {
    let range = source.range_at(now).max(MIN_RANGE);
    let intensity = source.intensity_at(now);
    PointLight {
        origin,
        range,
        diffuse: source.diffuse.map(|c| c * intensity * opacity),
        specular: source.specular.map(|c| c * opacity),
        attenuation: [1.0, 0.0, 10.0 / (range * range)],
    }
}

/// `LightSource::GetWeight` for a point light: `(range - d) / range * mean diffuse`, where the
/// diffuse already carries the intensity and the opacity; capped at 1.
pub(super) fn weight(point: &PointLight, distance: f32) -> f32 {
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
            cell: 0,
            frame: Frame::new(Vec3::new(x, 0.0, 0.0), ac2_math::Quat::IDENTITY),
            scale: Vec3::ONE,
            particle_scale: 1.0,
            bones: Vec::new(),
            named_bones: Vec::new(),
            parents: std::sync::Arc::from(Vec::new()),
            speed: 0.0,
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

    /// A light whose range and intensity are sine waveforms breathes with the client time:
    /// `GetAnimatedRange` and `FillRenderLight` sample them each frame.
    #[test]
    fn range_and_intensity_follow_their_waveforms_over_time() {
        use ac2_math::waveform::kind;
        let mut light = fx(0.0, 0.0);
        light.source.range_wave = ac2_assets::Ac2Waveform {
            kind: kind::SINE,
            base: 6.0,
            amplitude: 2.0,
            frequency: std::f32::consts::FRAC_PI_2,
            ..Default::default()
        };
        light.source.intensity_wave = ac2_assets::Ac2Waveform {
            kind: kind::SQUARE,
            base: 1.0,
            amplitude: 0.5,
            frequency: std::f32::consts::FRAC_PI_2,
            ..Default::default()
        };
        assert!(light.source.is_range_animating());
        let mut lights = ObjectLights::default();
        lights.start(7, light, 0.0);
        lights.place(&[placed(7, 0.0)], 0.0);
        let at = |t: f64| lights.all(t).remove(0);
        // t = 1: the range's sine peaks (6 + 2); the intensity's square is positive (1.5).
        let peak = at(1.0);
        assert!((peak.range - 8.0).abs() < 1e-4, "{}", peak.range);
        assert!((peak.diffuse[0] - 1.5).abs() < 1e-4);
        assert!((peak.attenuation[2] - 10.0 / 64.0).abs() < 1e-5);
        // t = 3: the range's sine bottoms out (6 - 2); the intensity's square is negative (0.5).
        let trough = at(3.0);
        assert!((trough.range - 4.0).abs() < 1e-4, "{}", trough.range);
        assert!((trough.diffuse[0] - 0.5).abs() < 1e-4);
        // A constant waveform is its base at any time: the light is not animated.
        assert!(!fx(0.0, 0.0).source.is_range_animating());
        let mut still = ObjectLights::default();
        still.start(7, fx(0.0, 0.0), 0.0);
        still.place(&[placed(7, 0.0)], 0.0);
        assert_eq!(still.all(0.0)[0].range, still.all(500.0)[0].range);
    }

    #[test]
    fn a_light_sits_at_the_object_frame_plus_its_offset() {
        let mut lights = ObjectLights::default();
        lights.start(7, fx(0.0, 0.0), 0.0);
        lights.place(&[placed(7, 10.0)], 0.0);
        let points = lights.point_lights(0.0, Vec3::new(10.0, 0.0, 0.0), 1.0, false);
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].origin, Vec3::new(10.0, 0.0, 1.0));
        assert_eq!(points[0].attenuation, [1.0, 0.0, 10.0 / 25.0]);
    }

    #[test]
    fn lights_out_of_range_or_of_unplaced_objects_do_not_light() {
        let mut lights = ObjectLights::default();
        lights.start(7, fx(0.0, 0.0), 0.0);
        // Emitter 7 never placed: no light.
        assert!(lights.point_lights(0.0, Vec3::ZERO, 1.0, false).is_empty());
        lights.place(&[placed(7, 100.0)], 0.0);
        assert!(lights.point_lights(0.0, Vec3::ZERO, 1.0, false).is_empty());
    }

    /// `SceneTool::AddDynamicLights`: an FX light carries its object's cell, so it lights only
    /// objects on the same side of the indoor/outdoor divide.
    #[test]
    fn an_fx_light_lights_only_objects_on_its_side() {
        let mut lights = ObjectLights::default();
        lights.start(7, fx(0.0, 0.0), 0.0);
        let mut outside = placed(7, 0.0);
        outside.cell = 0xCB4D_0025;
        lights.place(&[outside], 0.0);
        let at = Vec3::new(0.0, 0.0, 1.0);
        assert_eq!(lights.point_lights(0.0, at, 0.0, false).len(), 1);
        assert!(lights.point_lights(0.0, at, 0.0, true).is_empty());
        let mut inside = placed(7, 0.0);
        inside.cell = 0x02BB_0100;
        lights.place(&[inside], 0.0);
        assert!(lights.point_lights(0.0, at, 0.0, false).is_empty());
        assert_eq!(lights.point_lights(0.0, at, 0.0, true).len(), 1);
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
        let points = lights.point_lights(0.0, Vec3::ZERO, 0.0, false);
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
