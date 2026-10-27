//! `FXManager::DoParticleFX`: the particle systems an effect starts on an object. Each node of
//! the effect's visual FX entry starts one system; the system's desc is one of the node's
//! `PSDesc` list, picked at random (`FXNode::GetParticleID`), and it is placed where the node
//! says (`PHYSICSOBJREP::CreateParticleSystem`). The systems are queued for the renderer, which
//! starts them on the object.

use ac2_anim::Ac2Transform;
use ac2_assets::Ac2VisualFxParticles;
use ac2_core::{DataId, Random};

/// A particle system an object's FX started: the handle that names it until it is stopped, the
/// emitter (object) id, the FX, the chosen `PSDesc`, and the placement and water-plane flag of its
/// node.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObjectParticleStart {
    pub handle: u64,
    pub emitter: u64,
    pub fx: u32,
    pub desc: DataId,
    pub placement: Ac2Transform,
    pub water_plane: bool,
}

/// The systems `fx` starts on `emitter`, one per node of the visual's entry for it. A node with
/// no `PSDesc` starts nothing.
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
            }
        })
        .collect()
}
