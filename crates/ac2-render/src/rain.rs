//! Rain (`LScapeRender::UpdateRain`, `LScapeRender::FillRainVertices`, `LScapeRender::DrawRain`):
//! a pool of 2500 streaks around the viewer, each standing on the terrain (or the water) at a
//! random point within 130 m and falling from up to 80 m at 150 m/s along gravity plus the
//! rain's wind. `m_fRainAmount` (`CDayDesc::GetRain`) picks how many of the pool fall;
//! `WorldRender::Render` draws them only above 0.4, with the material
//! `DBObj::GetByEnum(0x4000000F, 0x40000002)` (`LScapeRender::Init`).
//!
//! A streak in the last 8 m of its fall discharges a splash (`m_pRainVaporParticleSystem`'s
//! second emitter) where it lands, and a water ripple (`m_pWaterRipplesParticleSystem`'s) when
//! it stands on water within 65 m of the viewer: [`RainField::splashes`].

use ac2_core::Random;
use ac2_math::Vec3;

use crate::object::ObjectVertex;
use crate::particles::{CameraBasis, ParticleGeometry};

/// `LScapeRender::Init`: `m_Rain` holds this many `RainSegment`s.
pub const RAIN_SEGMENTS: usize = 2500;
/// `LScapeRender::Init`'s rain material: `DBObj::GetByEnum(0x4000000F, 0x40000002, ...)`.
pub const RAIN_MATERIAL_ENUM: u32 = 0x4000_000F;
pub const RAIN_MATERIAL_GROUP: u32 = 0x4000_0002;
/// `WorldRender::Init`'s rain particle systems, enums in the same group: the splash
/// (`m_pRainVaporParticleSystem`) and the water ripples (`m_pWaterRipplesParticleSystem`).
pub const RAIN_SPLASH_ENUM: u32 = 0x4000_0010;
pub const RAIN_RIPPLES_ENUM: u32 = 0x4000_0012;
/// `WorldRender::Render` draws rain only above this amount.
pub const RAIN_DRAW_THRESHOLD: f32 = 0.4;
/// `RAIN_cColor` (admin 7148 `.data`): the streak's foot; its head is the same colour at alpha 0.
const RAIN_COLOR: [f32; 4] = [0.9, 0.9, 1.0, 0.45];
/// How far from the viewer a streak stands (each axis).
const RAIN_RANGE: f32 = 130.0;
/// A streak's fall: its height at phase 1 and its speed.
const FALL_HEIGHT: f32 = 80.0;
const FALL_SPEED: f32 = 150.0;
/// A streak's length, and the half-width along the camera's right axis.
const SEGMENT_LENGTH: f32 = 8.0;
const HALF_WIDTH: f32 = 0.05;
/// A streak whose block is more than this many blocks from the viewer's stands anew.
const REGEN_BLOCKS: i32 = 5;
/// A streak landing ripples the water only within this distance of the viewer (`4225` m²).
const RIPPLE_RANGE: f32 = 65.0;
/// `PhysicsGlobals::gravity`.
const GRAVITY: Vec3 = Vec3::new(0.0, 0.0, -9.8);

/// The ground under a point: the terrain height and the block's water surface, when the
/// point's block is resident.
pub type Ground<'a> = dyn Fn(f32, f32) -> Option<(f32, Option<f32>)> + 'a;

/// `RainSegment`.
#[derive(Debug, Clone, Copy, Default)]
struct RainSegment {
    /// `m_vPhysicsOrigin`: where it stands on the ground.
    origin: Vec3,
    block: (i32, i32),
    on_water: bool,
    need_regen: bool,
    /// Where this fall lands (the origin, jittered).
    landing: Vec3,
    direction: Vec3,
    /// 1 at the top of the fall, 0 on the ground.
    phase: f32,
}

/// Where a streak lands this update: a splash there, and a ripple when `ripple`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Splash {
    pub at: Vec3,
    pub ripple: bool,
}

/// The rain around the viewer.
pub struct RainField {
    segments: Vec<RainSegment>,
    splashes: Vec<Splash>,
    random: Random,
    last_think: Option<f64>,
    center: Option<(i32, i32)>,
}

impl Default for RainField {
    fn default() -> Self {
        Self {
            segments: vec![
                RainSegment {
                    need_regen: true,
                    ..RainSegment::default()
                };
                RAIN_SEGMENTS
            ],
            splashes: Vec::new(),
            random: Random::default(),
            last_think: None,
            center: None,
        }
    }
}

impl RainField {
    /// `UpdateRain` at client time `now` seen by `camera`: the streaks of `amount` falling in
    /// `wind` around `viewer`, as four vertices and two triangles each. `ground` answers the
    /// terrain under a point; `block_size` is the landblock edge.
    pub fn update(
        &mut self,
        now: f64,
        camera: &CameraBasis,
        amount: f32,
        wind: [f32; 2],
        block_size: f32,
        ground: &Ground<'_>,
    ) -> ParticleGeometry {
        let count = (RAIN_SEGMENTS as f32 * amount.clamp(0.0, 1.0)).floor() as usize;
        let mut out = ParticleGeometry::default();
        let viewer = camera.eye;
        let block = |p: Vec3| {
            (
                (p.x / block_size).floor() as i32,
                (p.y / block_size).floor() as i32,
            )
        };
        let center = block(viewer);
        if self.center != Some(center) {
            // The viewer changed cells: streaks left far behind stand anew.
            self.center = Some(center);
            for segment in self.segments.iter_mut().take(count) {
                let (dx, dy) = (segment.block.0 - center.0, segment.block.1 - center.1);
                if !segment.need_regen && (dx.abs() > REGEN_BLOCKS || dy.abs() > REGEN_BLOCKS) {
                    segment.need_regen = true;
                }
            }
        }
        let elapsed = self.last_think.map_or(0.0, |last| (now - last) as f32);
        self.last_think = Some(now);
        self.splashes.clear();
        if count == 0 {
            return out;
        }
        let wind = Vec3::new(wind[0], wind[1], 0.0);
        let right = camera.right * HALF_WIDTH;
        let head = [RAIN_COLOR[0], RAIN_COLOR[1], RAIN_COLOR[2], 0.0];
        let random = &mut self.random;
        let splashes = &mut self.splashes;
        for segment in self.segments.iter_mut().take(count) {
            if segment.need_regen {
                let x = random.roll_dice_f32(-RAIN_RANGE, RAIN_RANGE) + viewer.x;
                let y = random.roll_dice_f32(-RAIN_RANGE, RAIN_RANGE) + viewer.y;
                if let Some((height, water)) = ground(x, y) {
                    let mut origin = Vec3::new(x, y, height);
                    segment.on_water = false;
                    if let Some(water) = water.filter(|w| origin.z < *w) {
                        origin.z = water;
                        segment.on_water = true;
                    }
                    segment.origin = origin;
                    segment.block = block(origin);
                    segment.landing = origin;
                    // Retail leaves the direction of the first fall as the segment last held
                    // it (zero for a new pool); here it falls with the wind from the start.
                    segment.direction = (GRAVITY + wind).normalize_or(Vec3::NEG_Z);
                    segment.phase = random.roll_dice_f32(0.0, 1.0);
                    segment.need_regen = false;
                }
            }
            if segment.need_regen {
                continue;
            }
            if segment.phase <= 0.0 {
                // A new fall: wrap the phase, turn with the wind, land near the origin.
                segment.phase -= segment.phase.floor();
                let jitter_y = random.roll_dice_f32(-0.5, 0.5);
                let jitter_x = random.roll_dice_f32(-0.5, 0.5);
                let fall = GRAVITY + wind + Vec3::new(jitter_x, jitter_y, 0.0);
                segment.direction = fall.normalize_or(Vec3::NEG_Z);
                segment.landing = segment.origin
                    + Vec3::new(
                        random.roll_dice_f32(-0.8, 0.8),
                        random.roll_dice_f32(-0.8, 0.8),
                        0.0,
                    );
            }
            let height = segment.phase * FALL_HEIGHT;
            let length = if height <= SEGMENT_LENGTH {
                let near = (segment.landing.truncate() - viewer.truncate()).length_squared();
                splashes.push(Splash {
                    at: segment.landing,
                    ripple: segment.on_water && near < RIPPLE_RANGE * RIPPLE_RANGE,
                });
                height
            } else {
                SEGMENT_LENGTH
            };
            let top = segment.landing - segment.direction * height;
            let foot = top + segment.direction * length;
            let first = out.vertices.len() as u32;
            for (position, color, uv) in [
                (top - right, head, [0.0, 0.0]),
                (foot - right, RAIN_COLOR, [0.0, 1.0]),
                (foot + right, RAIN_COLOR, [1.0, 1.0]),
                (top + right, head, [1.0, 0.0]),
            ] {
                out.vertices.push(ObjectVertex {
                    position: position.to_array(),
                    normal: [0.0, 0.0, 1.0],
                    color,
                    uvs: [uv; 4],
                    tangents: [[0.0; 3]; 2],
                });
            }
            // `LScapeRender::Init`'s index list.
            out.indices
                .extend([first, first + 1, first + 2, first, first + 3, first + 2]);
            segment.phase -= elapsed * FALL_SPEED / FALL_HEIGHT;
        }
        out
    }

    /// The streaks that reached the ground in the last update (`FillRainVertices`' discharges).
    pub fn splashes(&self) -> &[Splash] {
        &self.splashes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn camera() -> CameraBasis {
        CameraBasis::look_at(Vec3::new(100.0, 100.0, 2.0), Vec3::new(100.0, 110.0, 2.0)).unwrap()
    }

    #[test]
    fn the_amount_picks_how_many_streaks_fall_on_the_ground() {
        let mut rain = RainField::default();
        let flat = |_: f32, _: f32| Some((5.0, None));
        let geometry = rain.update(0.0, &camera(), 0.5, [0.0; 2], 192.0, &flat);
        assert_eq!(geometry.vertices.len(), 1250 * 4);
        assert_eq!(geometry.indices.len(), 1250 * 6);
        // Every streak is above the ground it stands on and within 130 m of the viewer.
        for v in &geometry.vertices {
            assert!(v.position[2] >= 5.0 - 1e-3 && v.position[2] <= 5.0 + FALL_HEIGHT + 1.0);
            assert!((v.position[0] - 100.0).abs() <= RAIN_RANGE + 1.0);
        }
        assert!(rain
            .update(0.1, &camera(), 0.0, [0.0; 2], 192.0, &flat)
            .is_empty());
    }

    #[test]
    fn streaks_fall_at_150_metres_a_second_and_off_resident_ground_stand_nowhere() {
        let mut rain = RainField::default();
        let flat = |_: f32, _: f32| Some((0.0, None));
        rain.update(0.0, &camera(), 0.01, [0.0; 2], 192.0, &flat);
        // The phase steps after the streaks are laid out, as `FillRainVertices` does.
        let before = rain.update(0.01, &camera(), 0.01, [0.0; 2], 192.0, &flat);
        let after = rain.update(0.02, &camera(), 0.01, [0.0; 2], 192.0, &flat);
        let heads = |g: &ParticleGeometry| {
            g.vertices
                .iter()
                .step_by(4)
                .map(|v| v.position[2])
                .collect::<Vec<_>>()
        };
        let fell: Vec<f32> = heads(&before)
            .iter()
            .zip(heads(&after))
            .map(|(a, b)| a - b)
            .collect();
        // 10 ms falls 1.5 m (or wraps to a new fall).
        assert!(fell.iter().any(|d| (d - 1.5).abs() < 1e-3));
        assert!(!rain.splashes().is_empty() || fell.iter().any(|d| *d < 0.0));
        let mut dry = RainField::default();
        let nowhere = |_: f32, _: f32| None;
        assert!(dry
            .update(0.0, &camera(), 1.0, [0.0; 2], 192.0, &nowhere)
            .is_empty());
    }

    #[test]
    fn landing_streaks_splash_and_ripple_only_near_water() {
        let mut rain = RainField::default();
        // Water stands a metre above the ground everywhere.
        let wet = |_: f32, _: f32| Some((0.0, Some(1.0)));
        let mut seen = Vec::new();
        for step in 0..40 {
            rain.update(
                f64::from(step) * 0.02,
                &camera(),
                1.0,
                [0.0; 2],
                192.0,
                &wet,
            );
            seen.extend_from_slice(rain.splashes());
        }
        assert!(!seen.is_empty(), "streaks reach the ground");
        for splash in &seen {
            assert!((splash.at.z - 1.0).abs() < 1e-3, "on the water surface");
            let near = (splash.at.truncate() - camera().eye.truncate()).length() < RIPPLE_RANGE;
            assert_eq!(splash.ripple, near);
        }
        assert!(seen.iter().any(|s| s.ripple) && seen.iter().any(|s| !s.ripple));
    }
}
