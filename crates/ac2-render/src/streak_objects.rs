//! The streaks an object's FX started, as the scene holds them. `FXManager::DoStreakFX` makes one
//! on the emitter (`PHYSICSOBJREP::CreateStreakEffect` -> `CPartArray::CreateStreakEffect`, which
//! keeps it on the part array); `FXManager::DoTargettedFX`'s start and stop flags activate and
//! deactivate the emitter's streaks (`PHYSICSOBJREP::StartStreakInternal` /
//! `StopStreakInternal` -> `CPartArray::ActivateStreakEffects` / `DeactivateStreakEffects`). Each
//! frame the emitter's part array places them (`CPartArray::Draw` -> `StreakEffect::SetPosition`
//! at the object's world matrix), the manager thinks them, and they draw. A streak goes when its
//! emitter is destroyed (`CPartArray`'s destructor, `StreakEffectManager::DestroyStreakEffect`).

use std::collections::HashMap;

use ac2_core::DataId;
use ac2_math::Frame;

use crate::particles::ParticleGeometry;
use crate::streak::{StreakEffects, StreakSpec};

/// The emitters' streaks, by emitter, over the manager that thinks and draws them.
#[derive(Debug, Clone, Default)]
pub struct ObjectStreaks {
    effects: StreakEffects,
    by_emitter: HashMap<u64, Vec<u64>>,
    /// `SceneTool::sceneTimestamp` of the last placement: only streaks placed in it draw.
    stamp: u64,
}

impl ObjectStreaks {
    /// `CPartArray::CreateStreakEffect`: a streak on `emitter`, made at client time `now`.
    pub fn start(&mut self, emitter: u64, spec: StreakSpec, now: f64) -> u64 {
        let id = self.effects.create(spec, now);
        self.by_emitter.entry(emitter).or_default().push(id);
        id
    }

    /// `CPartArray::ActivateStreakEffects` (`on`) and `DeactivateStreakEffects` (off) for every
    /// streak of `emitter`.
    pub fn set_active(&mut self, emitter: u64, on: bool) {
        for id in self.by_emitter.get(&emitter).into_iter().flatten() {
            if let Some(effect) = self.effects.get_mut(*id) {
                if on {
                    effect.activate();
                } else {
                    effect.deactivate();
                }
            }
        }
    }

    /// `CPartArray`'s destructor: the emitter's streaks are destroyed.
    pub fn remove_emitter(&mut self, emitter: u64) {
        for id in self.by_emitter.remove(&emitter).into_iter().flatten() {
            self.effects.destroy(id);
        }
    }

    /// `CPartArray::Draw`'s `StreakEffect::SetPosition` for each placed emitter, under a new
    /// stamp: `(emitter, cell, frame, scale)`, the scale the largest of the object's.
    pub fn place(&mut self, placements: impl IntoIterator<Item = (u64, u32, Frame, f32)>) {
        self.stamp += 1;
        for (emitter, cell, frame, scale) in placements {
            for id in self.by_emitter.get(&emitter).into_iter().flatten() {
                if let Some(effect) = self.effects.get_mut(*id) {
                    effect.set_position(self.stamp, cell, frame, scale);
                }
            }
        }
    }

    /// `StreakEffectManager::UpdateStreakEffects` at client time `now`.
    pub fn think(&mut self, now: f64) {
        self.effects.think(now);
    }

    /// `StreakEffectManager::RenderStreakEffects`: the ribbons placed in the last stamp, one
    /// geometry per material.
    pub fn geometry(&self) -> Vec<(DataId, ParticleGeometry)> {
        self.effects.geometry(self.stamp)
    }

    /// How many streaks the emitter holds.
    pub fn count(&self, emitter: u64) -> usize {
        self.by_emitter.get(&emitter).map_or(0, Vec::len)
    }
}

#[cfg(test)]
#[path = "streak_objects_tests.rs"]
mod tests;
