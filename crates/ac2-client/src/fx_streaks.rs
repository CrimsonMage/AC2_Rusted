//! `FXManager::DoStreakFX` and the streak flags of `FXManager::DoTargettedFX`: an FX's nodes make
//! the streaks their emitter trails (`PHYSICSOBJREP::CreateStreakEffect`), and start or stop the
//! emitter's streaks (`PHYSICSOBJREP::StartStreak` / `StopStreak`). A running FX starts nothing
//! again (`DoTargettedFX`'s `m_current_fx` check), and `StopFX` does not touch streaks: they end
//! when they are stopped, when their lifespan is over, or when their emitter leaves the world. The
//! calls queue here and the renderer takes them (`Client::take_object_streak_calls`).

use ac2_assets::Ac2VisualFxParticles;
use ac2_core::DataId;
use ac2_render::streak::StreakSpec;

use crate::fx_objects::FxObjects;

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

impl FxObjects {
    /// `FXManager::DoTargettedFX`'s streak part for an FX that starts on `emitter`: each node in
    /// order makes its streak (`DoStreakFX`), then starts (`start_streak`) or stops the emitter's
    /// streaks. Nothing is made while `StreakEffects` is off, but a stop still applies. An FX that
    /// is already running starts nothing.
    pub(crate) fn start_streaks(&mut self, emitter: u64, fx: u32, table: &Ac2VisualFxParticles) {
        if !self.streaking.insert((emitter, fx)) {
            return;
        }
        let Some(nodes) = table.get(fx) else {
            return;
        };
        let on = !self.streaks_off;
        for node in nodes.iter() {
            if let (Some(streak), true) = (node.object.streak.as_ref(), on) {
                self.streaked.insert(emitter);
                self.streak_calls.push(ObjectStreakCall::Create {
                    emitter,
                    spec: spec(streak),
                });
            }
            if node.object.start_streak && on {
                self.streak_calls
                    .push(ObjectStreakCall::Activate { emitter, on: true });
            }
            if node.object.stop_streak {
                self.streak_calls
                    .push(ObjectStreakCall::Activate { emitter, on: false });
            }
        }
    }

    /// `StopFX` ends the FX's running state, so the FX can start (and make its streaks) again.
    pub(crate) fn stop_streaks(&mut self, emitter: u64, fx: u32) {
        self.streaking.remove(&(emitter, fx));
    }

    /// The emitter left the world: its streaks are destroyed once, if it had any.
    pub(crate) fn remove_streaks(&mut self, emitter: u64) {
        if self.streaked.remove(&emitter) {
            self.streak_calls.push(ObjectStreakCall::Gone { emitter });
        }
    }

    /// The emitters that hold streaks, in id order.
    pub(crate) fn streaked_emitters(&self) -> Vec<u64> {
        self.streaked.iter().copied().collect()
    }
}

impl crate::client::Client {
    /// The streak calls the FX started since the renderer last took them, in order.
    pub fn take_object_streak_calls(&mut self) -> Vec<ObjectStreakCall> {
        std::mem::take(&mut self.fx_objects.streak_calls)
    }

    /// `Render::m_Prefs.StreakEffects`: the streak preference the bank holds.
    pub fn apply_streak_preferences(&mut self, bank: &ac2_prefs::PreferenceBank) {
        self.fx_objects.streaks_off = !ac2_render::prefs::streak_effects(bank);
    }

    /// Whether the FX make and draw streaks (`StreakEffects` is on).
    pub fn streaks_enabled(&self) -> bool {
        !self.fx_objects.streaks_off
    }
}

#[cfg(test)]
#[path = "fx_streaks_tests.rs"]
mod tests;
