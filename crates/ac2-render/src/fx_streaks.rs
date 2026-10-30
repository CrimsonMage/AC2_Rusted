//! `FXManager::DoStreakFX` and the streak flags of `FXManager::DoTargettedFX`: an FX's nodes make
//! the streaks their emitter trails (`PHYSICSOBJREP::CreateStreakEffect`), and start or stop the
//! emitter's streaks (`PHYSICSOBJREP::StartStreak` / `StopStreak`). Those walk the emitter's
//! holding-point children too (`CPhysicsObj::StartStreak`). A node is gated first (player-only,
//! day or night, channels), as the node's other parts are. A running FX starts nothing again
//! (`DoTargettedFX`'s `m_current_fx` check), and `StopFX` does not touch streaks: they end when they
//! are stopped, when their lifespan is over, or when their emitter leaves the world. The calls queue
//! here (`FxStreaks`) and the renderer takes them ([`FxStreaks::take_calls`]).

use std::collections::BTreeSet;

use ac2_assets::Ac2VisualFxParticles;
use ac2_core::DataId;

use crate::streak::StreakSpec;

/// The holding tree is shallow; the bound only guards a corrupt parent link.
const MAX_HOLD_DEPTH: usize = 16;

/// What the streak part of `DoTargettedFX` reads beside the FX: the player (for a player-only
/// node), the time of day, which objects hang from which holding point, and the emitter's terrain.
#[derive(Debug, Clone, Copy)]
pub struct StreakContext<'a> {
    /// The player's object id.
    pub player: Option<u64>,
    /// `CalcTimeOfDay`'s night flag: a night node plays, a day node does.
    pub night: bool,
    /// `(parent, child, holding location)` for each object held at a holding point of its parent
    /// (`CHILDLIST`'s holding entries).
    pub holds: &'a [(u64, u64, u32)],
    /// The emitter's `FXManager::m_terrain` (`PHYSICSOBJREP::SetTerrainType`): the terrain whose
    /// nodes `FXData::GetData` selects.
    pub terrain: u32,
}

/// The FX streak state (`FXManager`'s streak part): the FX that are running their streak nodes,
/// the emitters that hold streaks, the calls not yet taken by the renderer, and `StreakEffects`.
#[derive(Debug, Default)]
pub struct FxStreaks {
    /// The FX whose start ran its streak nodes and has not stopped (`FXManager::m_current_fx`
    /// for the streak part of `DoTargettedFX`).
    pub(crate) streaking: BTreeSet<(u64, u32)>,
    /// The emitters that hold streaks.
    pub(crate) streaked: BTreeSet<u64>,
    /// The streak calls the renderer takes, in order.
    pub(crate) streak_calls: Vec<ObjectStreakCall>,
    /// `StreakEffects` is off: no streak is made (`DoStreakFX` returns without one).
    pub(crate) streaks_off: bool,
}

/// What the renderer does for an object's streaks, in the order the FX made them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ObjectStreakCall {
    /// `FXManager::DoStreakFX`: a streak `spec` on `emitter`.
    Create { emitter: u64, spec: StreakSpec },
    /// `PHYSICSOBJREP::StartStreak` (`on`) or `StopStreak`: the emitter's streaks activate or stop.
    Activate { emitter: u64, on: bool },
    /// The emitter left the world: its streaks are destroyed (`CPartArray`'s destructor).
    Gone { emitter: u64 },
}

/// A `StreakFX` as the streak manager is given it (`CPartArray::CreateStreakEffect`'s arguments).
fn spec(streak: &ac2_assets::Ac2StreakFx) -> StreakSpec {
    StreakSpec {
        start: streak.start,
        end: streak.end,
        lifespan: streak.lifespan,
        trail_duration: streak.trail_duration,
        min_segment_time: streak.min_segment_time,
        min_segment_distance: streak.min_segment_distance,
        color: streak.color,
        texture_scale: streak.texture_scale,
        material: DataId::new(streak.material),
    }
}

/// `DoTargettedFX`'s node gate: no channel mismatch (`FXManager::ChannelMissmatch` against the
/// FX channel preferences), `m_player_only` for the player, and the node's day or night flag for
/// the time of day (`(day_time || night) && (night_time || day)`).
pub(crate) fn node_plays(gate: &ac2_assets::Ac2FxNodeGate, is_player: bool, night: bool) -> bool {
    gate.plays(is_player, night)
}

impl FxStreaks {
    /// `FXManager::DoTargettedFX`'s streak part for an FX that starts on `emitter`: each node that
    /// plays (see [`node_plays`]) in order makes its streak (`DoStreakFX`), then starts
    /// (`start_streak`) or stops the emitter's streaks and those of its holding-point children.
    /// Nothing is made while `StreakEffects` is off, but a stop still applies. An FX that is already
    /// running starts nothing.
    pub fn start_streaks(
        &mut self,
        emitter: u64,
        fx: u32,
        table: &Ac2VisualFxParticles,
        ctx: &StreakContext<'_>,
    ) {
        if !self.streaking.insert((emitter, fx)) {
            return;
        }
        let Some(nodes) = table.get_on(fx, ctx.terrain) else {
            return;
        };
        let on = !self.streaks_off;
        let is_player = ctx.player == Some(emitter);
        for node in nodes
            .iter()
            .filter(|node| node_plays(&node.object.gate, is_player, ctx.night))
        {
            let holding = node.object.holding_location;
            if let (Some(streak), true) = (node.object.streak.as_ref(), on) {
                self.streaked.insert(emitter);
                self.streak_calls.push(ObjectStreakCall::Create {
                    emitter,
                    spec: spec(streak),
                });
            }
            if node.object.start_streak && on {
                self.activate_tree(emitter, true, holding, ctx.holds, 0);
            }
            if node.object.stop_streak {
                self.activate_tree(emitter, false, holding, ctx.holds, 0);
            }
        }
    }

    /// `FXManager::DoStartupFX`'s streak part: every node of a startup FX that carries a streak
    /// makes it (`CreateStreakEffect` at the FX's scalar, no node gate), while `StreakEffects` is on.
    /// Called once per object, when its startup FX run.
    pub fn start_startup(&mut self, emitter: u64, nodes: &[ac2_assets::Ac2ParticleAttachment]) {
        if self.streaks_off {
            return;
        }
        for streak in nodes.iter().filter_map(|node| node.object.streak.as_ref()) {
            self.streaked.insert(emitter);
            self.streak_calls.push(ObjectStreakCall::Create {
                emitter,
                spec: spec(streak),
            });
        }
    }

    /// `CPhysicsObj::StartStreak(0)` / `StopStreak(0)` called on the object itself (a missile's
    /// flight): its own streaks and every holding-point child's activate or stop.
    pub fn set_object_streaks(&mut self, emitter: u64, on: bool, ctx: &StreakContext<'_>) {
        self.activate_tree(emitter, on, 0, ctx.holds, 0);
    }

    /// `CPhysicsObj::StartStreak` / `StopStreak`: the object's own streaks activate (or stop), then
    /// each holding-point child at `holding` (every child when it is zero) does the same, down its
    /// own children.
    fn activate_tree(
        &mut self,
        emitter: u64,
        on: bool,
        holding: u32,
        holds: &[(u64, u64, u32)],
        depth: usize,
    ) {
        self.streak_calls
            .push(ObjectStreakCall::Activate { emitter, on });
        if depth >= MAX_HOLD_DEPTH {
            return;
        }
        for &(parent, child, location) in holds {
            if parent == emitter && (holding == 0 || location == holding) {
                self.activate_tree(child, on, holding, holds, depth + 1);
            }
        }
    }

    /// `StopFX` ends the FX's running state, so the FX can start (and make its streaks) again.
    pub fn stop_streaks(&mut self, emitter: u64, fx: u32) {
        self.streaking.remove(&(emitter, fx));
    }

    /// The emitter left the world: its streaks are destroyed once, if it had any.
    pub fn remove_streaks(&mut self, emitter: u64) {
        if self.streaked.remove(&emitter) {
            self.streak_calls.push(ObjectStreakCall::Gone { emitter });
        }
    }

    /// The emitters that hold streaks, in id order.
    pub fn streaked_emitters(&self) -> Vec<u64> {
        self.streaked.iter().copied().collect()
    }

    /// The streak calls the FX started since the renderer last took them, in order.
    pub fn take_calls(&mut self) -> Vec<ObjectStreakCall> {
        std::mem::take(&mut self.streak_calls)
    }

    /// `StreakEffects` turned on or off. Off makes no new streak and starts none, and the view
    /// stops drawing the streaks; the existing ones keep thinking, so they resume where they are.
    pub fn set_enabled(&mut self, on: bool) {
        self.streaks_off = !on;
    }

    /// Whether the FX make and draw streaks (`StreakEffects` is on).
    pub fn enabled(&self) -> bool {
        !self.streaks_off
    }

    /// The emitter's running FX are forgotten with it (`FXManager::DestroyAllTransientFX`), so
    /// its FX can start again if it returns.
    pub fn forget_emitter(&mut self, emitter: u64) {
        self.streaking.retain(|(id, _)| *id != emitter);
    }
}

#[cfg(test)]
#[path = "fx_streaks_tests.rs"]
mod tests;
