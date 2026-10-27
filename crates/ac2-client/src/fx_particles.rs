//! `FXManager::DoParticleFX` and `FXManager::DoLightSourceFX`: the particle systems and lights an
//! object's FX starts. Each node of the effect's visual FX entry starts one system (the node's
//! `PSDesc` list picked at random, `FXNode::GetParticleID`) placed where the node says
//! (`PHYSICSOBJREP::CreateParticleSystem`), and places its light at the node's offset
//! (`CPartArray::CreateLightSource`). The renderer starts them on the object.

use ac2_anim::Ac2Transform;
use ac2_assets::{Ac2FxLight, Ac2VisualFxParticles};
use ac2_core::{DataId, Random};

/// A particle system an object's FX started: the handle that names it until it is stopped, the
/// emitter (object) id, the FX, the chosen `PSDesc`, and the placement and water-plane flag of its
/// node. A fire-and-forget node's system is placed once where its object stands.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObjectParticleStart {
    pub handle: u64,
    pub emitter: u64,
    pub fx: u32,
    pub desc: DataId,
    pub placement: Ac2Transform,
    pub water_plane: bool,
    pub fire_and_forget: bool,
}

/// A light an object's FX placed (`DoLightSourceFX`) on `emitter`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObjectLightStart {
    pub emitter: u64,
    pub light: Ac2FxLight,
}

/// The systems `fx` starts on `emitter`, one per node of the visual's entry for it that has a
/// `PSDesc`. The handles are left at zero for the caller to number.
pub(crate) fn particle_starts(
    emitter: u64,
    fx: u32,
    table: &Ac2VisualFxParticles,
    random: &mut Random,
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
                placement: node.placement,
                water_plane: node.water_plane,
                fire_and_forget: node.fire_and_forget,
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use ac2_assets::{Ac2LightSource, Ac2ParticleAttachment};
    use ac2_math::Vec3;

    use super::*;

    fn attachment(descs: &[u32], light: Option<Ac2FxLight>) -> Ac2ParticleAttachment {
        Ac2ParticleAttachment {
            placement: Ac2Transform::IDENTITY,
            particle_descs: descs.iter().map(|d| DataId::new(*d)).collect::<Arc<[_]>>(),
            water_plane: false,
            fire_and_forget: false,
            light,
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

    #[test]
    fn a_light_only_node_places_its_light_and_starts_no_system() {
        let table = Ac2VisualFxParticles::from_entries([(
            7,
            vec![attachment(&[], Some(light(2.0))), attachment(&[9], None)],
        )]);
        let mut random = Random::default();
        assert_eq!(particle_starts(3, 7, &table, &mut random).len(), 1);
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
