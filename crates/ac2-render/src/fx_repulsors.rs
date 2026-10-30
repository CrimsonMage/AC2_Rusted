//! `FXManager::DoRepulseFX`: an FX node with a `RepulsorFX` leaves a particle repulsor on its
//! emitter (`PHYSICSOBJREP::CreateParticleRepulsor` -> `CPartArray::CreateRepulsorEffect` ->
//! `PSManager::CreateRepulsor`). The repulsor lives by its own lifetime: stopping the FX does not
//! end it. The emitter's part array keeps it and places it every time the object draws
//! (`PHYSICSOBJREP::Draw`), and destroys it with the emitter (`CPartArray`'s destructor). A node
//! is gated as the node's other parts are, and a running FX makes nothing again
//! (`DoTargettedFX`'s `m_current_fx`). The calls queue here and the scene takes them
//! ([`FxRepulsors::take_calls`]).

use std::collections::BTreeSet;

use ac2_assets::Ac2VisualFxParticles;

use crate::fx_streaks::{node_plays, StreakContext};
use crate::particles::RepulsorSpec;

/// What the scene does for an object's repulsors, in the order the FX made them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ObjectRepulsorCall {
    /// `FXManager::DoRepulseFX`: a repulsor `spec` on `emitter`.
    Create { emitter: u64, spec: RepulsorSpec },
    /// The emitter left the world: its repulsors are destroyed.
    Gone { emitter: u64 },
}

/// The FX repulsor state: the FX that have started their repulsor nodes, the emitters holding
/// repulsors, and the calls not yet taken by the scene.
#[derive(Debug, Default)]
pub struct FxRepulsors {
    started: BTreeSet<(u64, u32)>,
    repulsed: BTreeSet<u64>,
    calls: Vec<ObjectRepulsorCall>,
}

/// A `RepulsorFX` as the particle manager is given it (`PSManager::CreateRepulsor`'s arguments).
fn repulsor_spec(repulsor: &ac2_assets::Ac2RepulsorFx) -> RepulsorSpec {
    RepulsorSpec {
        acceleration: repulsor.acceleration,
        range: repulsor.range,
        lifetime: repulsor.lifetime,
        minimum_velocity: repulsor.minimum_velocity,
        attractor: repulsor.attractor,
    }
}

impl FxRepulsors {
    /// `FXManager::DoTargettedFX`'s repulsor part for an FX starting on `emitter`: each node that
    /// plays (see [`node_plays`]) and carries a repulsor makes it, in order.
    pub fn start(
        &mut self,
        emitter: u64,
        fx: u32,
        table: &Ac2VisualFxParticles,
        ctx: &StreakContext<'_>,
    ) {
        if !self.started.insert((emitter, fx)) {
            return;
        }
        let Some(nodes) = table.get_on(fx, ctx.terrain) else {
            return;
        };
        let is_player = ctx.player == Some(emitter);
        for node in nodes
            .iter()
            .filter(|node| node_plays(&node.object.gate, is_player, ctx.night))
        {
            let Some(repulsor) = node.object.repulsor else {
                continue;
            };
            self.repulsed.insert(emitter);
            self.calls.push(ObjectRepulsorCall::Create {
                emitter,
                spec: repulsor_spec(&repulsor),
            });
        }
    }

    /// `FXManager::DoStartupFX`'s repulsor part: every node of a startup FX that carries a repulsor
    /// makes it (`CreateParticleRepulsor`, no node gate). Called once per object, when its startup
    /// FX run.
    pub fn start_startup(&mut self, emitter: u64, nodes: &[ac2_assets::Ac2ParticleAttachment]) {
        for repulsor in nodes.iter().filter_map(|node| node.object.repulsor) {
            self.repulsed.insert(emitter);
            self.calls.push(ObjectRepulsorCall::Create {
                emitter,
                spec: repulsor_spec(&repulsor),
            });
        }
    }

    /// `StopFX`: the FX can start (and make its repulsors) again; the repulsors made stay.
    pub fn stop(&mut self, emitter: u64, fx: u32) {
        self.started.remove(&(emitter, fx));
    }

    /// The emitter's running FX are forgotten with it, so they can start again if it returns.
    pub fn forget_emitter(&mut self, emitter: u64) {
        self.started.retain(|(id, _)| *id != emitter);
    }

    /// The emitter left the world: its repulsors are destroyed once, if it had any.
    pub fn remove_repulsors(&mut self, emitter: u64) {
        if self.repulsed.remove(&emitter) {
            self.calls.push(ObjectRepulsorCall::Gone { emitter });
        }
    }

    /// The emitters that hold repulsors, in id order.
    pub fn repulsed_emitters(&self) -> Vec<u64> {
        self.repulsed.iter().copied().collect()
    }

    /// The calls the FX made since the scene last took them, in order.
    pub fn take_calls(&mut self) -> Vec<ObjectRepulsorCall> {
        std::mem::take(&mut self.calls)
    }
}

#[cfg(test)]
#[path = "fx_repulsors_tests.rs"]
mod tests;
