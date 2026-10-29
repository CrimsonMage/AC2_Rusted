//! `WorldRender::RenderSkeletonEffects`: the bodies drawn outdoors give off rain vapour where the
//! rain strikes their bones, bubbles from the bones under water and ripples where a bone crosses
//! the water's surface, discharged into the scene's shared systems (`m_pRainVaporParticleSystem`,
//! `m_pWaterBubblesParticleSystem`, `m_pWaterRipplesParticleSystem`) at their first emitter. The
//! skeleton points are the root group's posed bones with their parents
//! (`CPartArray::CalculateSkeletonPoints`), in the body's frame and scale.

use ac2_math::Vec3;

use super::{ObjectPlacement, WorldScene};

/// `PhysicsGlobals::gravity`.
const GRAVITY: Vec3 = Vec3::new(0.0, 0.0, -9.8);

/// What one bone gives off this frame, at world points.
#[derive(Debug, Default, PartialEq)]
pub(super) struct BoneEffects {
    pub vapour: Vec<Vec3>,
    pub bubbles: Vec<Vec3>,
    pub ripples: Vec<Vec3>,
}

/// Which effects a body takes: rain vapour along `rain` (the direction rain falls, world
/// space) when it rains hard enough, and bubbles and ripples against the water at `water`.
#[derive(Debug, Clone, Copy)]
pub(super) struct Surroundings {
    pub rain: Option<Vec3>,
    pub water: Option<f32>,
}

/// `RenderSkeletonEffects` for one body: each bone with a parent and over 5 cm long rolls for
/// vapour (3 in 4), bubbles (4 in 5) and ripples (1 in 5). Vapour comes from a random point
/// along a bone across the rain (`1 - |bone . rain| > 0.4`) above the water; bubbles from a
/// random point 0.2 m under it; a ripple where a bone crosses it, at the middle of the bone.
/// `roll(lo, hi)` is `Random::RollDice`.
pub(super) fn bone_effects(
    body: &ObjectPlacement,
    around: Surroundings,
    roll: &mut dyn FnMut(f32, f32) -> f32,
) -> BoneEffects {
    let mut out = BoneEffects::default();
    let local_rain = around
        .rain
        .map(|rain| body.frame.global_to_local_vector(rain));
    let to_world = |p: Vec3| body.frame.local_to_global_point(p * body.scale);
    for (index, bone) in body.bones.iter().enumerate() {
        let Some(parent) = body.parents.get(index).copied().flatten() else {
            continue;
        };
        let Some(parent_bone) = body.bones.get(parent) else {
            continue;
        };
        let vapour = local_rain.is_some() && roll(0.0, 1.0) <= 0.75;
        let bubbles = roll(0.0, 1.0) <= 0.8;
        let ripples = roll(0.0, 1.0) <= 0.2;
        if !(vapour || bubbles || ripples) {
            continue;
        }
        let (from, to) = (parent_bone.offset, bone.offset);
        let span = to - from;
        let length = span.length();
        if length <= 0.05 {
            continue;
        }
        let along = span / length;
        if let (true, Some(rain)) = (vapour, local_rain) {
            if 1.0 - along.dot(rain).abs() > 0.4 {
                let at = to_world(from + along * roll(0.0, length));
                if around.water.is_none_or(|water| water < at.z) {
                    out.vapour.push(at);
                }
            }
        }
        let Some(water) = around.water else {
            continue;
        };
        if bubbles {
            let at = to_world(from + along * roll(0.0, length));
            if at.z < water - 0.2 {
                out.bubbles.push(at);
            }
        }
        if ripples {
            let (a, b) = (to_world(from), to_world(to));
            if (water < a.z && b.z < water) || (a.z < water && water < b.z) {
                let middle = (a + b) * 0.5;
                out.ripples.push(Vec3::new(middle.x, middle.y, water));
            }
        }
    }
    out
}

impl WorldScene {
    /// The frame's skeleton effects for the bodies placed outdoors, discharged into the shared
    /// systems; rain vapour only when the rain is over 0.4.
    pub(super) fn skeleton_effects(&mut self, rain: f32, wind: [f32; 2]) {
        let rain_dir =
            (rain > 0.4).then(|| (GRAVITY + Vec3::new(wind[0], wind[1], 0.0)).normalize_or_zero());
        let now = self.started.elapsed().as_secs_f64();
        let block_size = self.world.block_size();
        let mut random = std::mem::take(&mut self.effects_random);
        let bodies = std::mem::take(&mut self.bodies);
        for body in &bodies {
            // `LandDefs::first_envcell_id`: outdoors only.
            if body.cell & 0xFFFF >= 0x100 || body.parents.is_empty() {
                continue;
            }
            let origin = body.frame.origin;
            let water = self
                .heights_at(origin.x, origin.y, block_size)
                .and_then(|heights| heights.water);
            let effects = bone_effects(
                body,
                Surroundings {
                    rain: rain_dir,
                    water,
                },
                &mut |lo, hi| random.roll_dice_f32(lo, hi),
            );
            let discharge = |system: &mut Option<crate::particles::ParticleSystem>,
                             points: &[Vec3]| {
                if let Some(system) = system.as_mut() {
                    for at in points {
                        system.discharge(0, now, *at);
                    }
                }
            };
            discharge(&mut self.rain_splash, &effects.vapour);
            discharge(&mut self.water_bubbles, &effects.bubbles);
            discharge(&mut self.rain_ripples, &effects.ripples);
        }
        self.bodies = bodies;
        self.effects_random = random;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ac2_anim::Ac2Transform;
    use ac2_math::{Frame, Quat};

    /// A standing body: hips at 1 m, a spine up to 1.6 m and a leg down to the ground.
    fn body(z: f32) -> ObjectPlacement {
        let bone = |z: f32| Ac2Transform {
            offset: Vec3::new(0.0, 0.0, z),
            scale: Vec3::ONE,
            rotation: Quat::IDENTITY,
        };
        ObjectPlacement {
            emitter: 1,
            cell: 0x7F7F_0001,
            frame: Frame::new(Vec3::new(10.0, 10.0, z), Quat::IDENTITY),
            scale: Vec3::ONE,
            bones: vec![bone(1.0), bone(1.6), bone(0.0)],
            named_bones: Vec::new(),
            parents: std::sync::Arc::from(vec![None, Some(0), Some(0)]),
        }
    }

    /// Every roll takes the low end: every effect is rolled, at the parent's end of the bone.
    fn low(lo: f32, _hi: f32) -> f32 {
        lo
    }

    #[test]
    fn a_body_wading_ripples_where_its_legs_cross_the_water_and_bubbles_under_it() {
        // Standing at 0 in water 0.5 deep: the leg (1 m to 0 m) crosses it, the spine does not.
        let effects = bone_effects(
            &body(0.0),
            Surroundings {
                rain: None,
                water: Some(0.5),
            },
            &mut low,
        );
        assert_eq!(effects.ripples, vec![Vec3::new(10.0, 10.0, 0.5)]);
        // Bubbles only from points 0.2 m under the water (the low rolls sit at the hips: none).
        assert!(effects.bubbles.is_empty());
        assert!(effects.vapour.is_empty());
        let deep = bone_effects(
            &body(-2.0),
            Surroundings {
                rain: None,
                water: Some(0.5),
            },
            &mut low,
        );
        assert_eq!(deep.bubbles.len(), 2, "both bones are under");
        assert!(deep.ripples.is_empty());
    }

    #[test]
    fn rain_steams_off_bones_across_it_above_the_water() {
        // Rain falling straight down runs along the upright bones: no vapour.
        let down = Surroundings {
            rain: Some(-Vec3::Z),
            water: None,
        };
        assert!(bone_effects(&body(0.0), down, &mut low).vapour.is_empty());
        // Driven sideways it strikes them.
        let sideways = Surroundings {
            rain: Some(Vec3::new(1.0, 0.0, -0.2).normalize()),
            water: None,
        };
        assert_eq!(bone_effects(&body(0.0), sideways, &mut low).vapour.len(), 2);
    }
}
