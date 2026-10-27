//! `FXManager::DoParticleFX` and `FXManager::DoLightSourceFX`: the particle systems and lights an
//! object's FX starts. Each node of the effect's visual FX entry starts one system (the node's
//! `PSDesc` list picked at random, `FXNode::GetParticleID`) attached where the node says
//! (`PHYSICSOBJREP::CreateParticleSystem`, or `FireParticleFX` for a fire-and-forget node), and
//! places its light at the node's offset (`CPartArray::CreateLightSource`). The renderer starts
//! them on the object and poses their attachment with the object's bones each frame.

use ac2_assets::{Ac2FxLight, Ac2VisualFxParticles};
use ac2_core::{DataId, Random};
use ac2_render::scene::ParticleAnchor;

/// A particle system an object's FX started: the handle that names it until it is stopped, the
/// emitter (object) id, the FX, the chosen `PSDesc`, and where the system attaches on its object.
/// A fire-and-forget node's system is placed once where its object stands.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObjectParticleStart {
    pub handle: u64,
    pub emitter: u64,
    pub fx: u32,
    pub desc: DataId,
    pub anchor: ParticleAnchor,
    pub fire_and_forget: bool,
    /// `FXManager::ParticleIsLooping` of the chosen `PSDesc`: only a looping system is kept on
    /// the FX (and so stopped with it).
    pub looping: bool,
}

/// A light an object's FX placed (`DoLightSourceFX`) on `emitter`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObjectLightStart {
    pub emitter: u64,
    pub light: Ac2FxLight,
}

/// The systems `fx` starts on `emitter`, one per node of the visual's entry for it that has a
/// `PSDesc`. A node that attaches to a bone the object lacks starts none (`DoParticleFX`'s
/// `CreateParticleSystem` returns zero). `water_z` is the water surface under the object now
/// (`FXManager::FireParticleFX`'s water branch). The handles are left at zero for the caller.
pub(crate) fn particle_starts(
    emitter: u64,
    fx: u32,
    table: &Ac2VisualFxParticles,
    random: &mut Random,
    water_z: Option<f32>,
) -> Vec<ObjectParticleStart> {
    let Some(nodes) = table.get(fx) else {
        return Vec::new();
    };
    nodes
        .iter()
        .filter(|node| !node.particle_descs.is_empty())
        .map(|node| {
            let pick = random.roll_dice(0, node.particle_descs.len() as i32 - 1) as usize;
            ObjectParticleStart {
                handle: 0,
                emitter,
                fx,
                desc: node.particle_descs[pick],
                anchor: ParticleAnchor {
                    anchor: node.anchor,
                    remove_rotation: node.remove_rotation,
                    water_plane: node.water_plane,
                    water_z,
                },
                fire_and_forget: node.fire_and_forget,
                looping: node.looping.get(pick).copied().unwrap_or(false),
            }
        })
        .collect()
}

/// The lights `fx` places on `emitter`, one per node of the visual's entry that carries one, in
/// the entry's order.
pub(crate) fn light_starts(
    emitter: u64,
    fx: u32,
    table: &Ac2VisualFxParticles,
) -> Vec<ObjectLightStart> {
    let Some(nodes) = table.get(fx) else {
        return Vec::new();
    };
    nodes
        .iter()
        .filter_map(|node| node.light)
        .map(|light| ObjectLightStart { emitter, light })
        .collect()
}

/// The lights an object's startup FX place when the object is created
/// (`CPhysicsObj::InitObjectEnd` -> `PHYSICSOBJREP::ApplyStartupFX(rep, object)` ->
/// `FXManager::DoStartupFX`, whose object branch runs `DoLightSourceFX` for each node of each
/// startup FX that carries a light). The particle systems of the same nodes start with the
/// shared rep (`ApplyStartupFX(rep, NULL)`), not here.
pub(crate) fn startup_light_starts(
    emitter: u64,
    table: &Ac2VisualFxParticles,
) -> Vec<ObjectLightStart> {
    table
        .startup()
        .iter()
        .flat_map(|fx| light_starts(emitter, *fx, table))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use ac2_anim::Ac2Transform;
    use ac2_assets::{Ac2LightSource, Ac2ParticleAnchor, Ac2ParticleAttachment};
    use ac2_math::Vec3;

    use super::*;

    fn attachment(descs: &[u32], light: Option<Ac2FxLight>) -> Ac2ParticleAttachment {
        Ac2ParticleAttachment {
            placement: Ac2Transform::IDENTITY,
            anchor: Ac2ParticleAnchor::Object { offset: Vec3::ZERO },
            remove_rotation: false,
            particle_descs: descs.iter().map(|d| DataId::new(*d)).collect::<Arc<[_]>>(),
            looping: vec![true; descs.len()].into(),
            water_plane: false,
            fire_and_forget: false,
            light,
            object: Default::default(),
        }
    }

    fn light(lifespan: f32) -> Ac2FxLight {
        Ac2FxLight {
            offset: Vec3::new(0.0, 0.0, 1.0),
            source: Ac2LightSource::default(),
            lifespan,
            fade_time: 0.25,
        }
    }

    /// `FXManager::DoParticleFX`: a non-looping system is started but not kept on the FX, so the
    /// FX's stop leaves it to run out (only a looping system carries its handle).
    #[test]
    fn a_system_is_kept_on_the_fx_only_when_it_loops() {
        let mut looping = attachment(&[9], None);
        looping.looping = vec![true].into();
        let mut finite = attachment(&[9], None);
        finite.looping = vec![false].into();
        let table = Ac2VisualFxParticles::from_entries([(7, vec![looping]), (8, vec![finite])]);
        let mut random = Random::default();
        assert!(particle_starts(3, 7, &table, &mut random, None)[0].looping);
        assert!(!particle_starts(3, 8, &table, &mut random, None)[0].looping);
    }

    /// `DoStartupFX` with an object: the light-only node of each startup FX places its light at
    /// creation, and an FX that is not a startup one places none.
    #[test]
    fn startup_fx_place_their_lights_when_the_object_is_created() {
        let table = Ac2VisualFxParticles::from_entries([
            (7, vec![attachment(&[], Some(light(2.0)))]),
            (8, vec![attachment(&[], Some(light(3.0)))]),
        ])
        .with_startup(vec![7]);
        assert_eq!(
            startup_light_starts(3, &table),
            vec![ObjectLightStart {
                emitter: 3,
                light: light(2.0),
            }],
            "only the startup FX's light starts"
        );
        let no_startup =
            Ac2VisualFxParticles::from_entries([(7, vec![attachment(&[], Some(light(2.0)))])]);
        assert!(startup_light_starts(3, &no_startup).is_empty());
    }

    #[test]
    fn a_light_only_node_places_its_light_and_starts_no_system() {
        let table = Ac2VisualFxParticles::from_entries([(
            7,
            vec![attachment(&[], Some(light(2.0))), attachment(&[9], None)],
        )]);
        let mut random = Random::default();
        assert_eq!(particle_starts(3, 7, &table, &mut random, None).len(), 1);
        assert_eq!(
            light_starts(3, 7, &table),
            vec![ObjectLightStart {
                emitter: 3,
                light: light(2.0),
            }]
        );
        assert!(light_starts(3, 8, &table).is_empty());
    }
}
