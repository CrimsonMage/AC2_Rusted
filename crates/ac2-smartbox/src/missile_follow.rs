//! What a missile's body asks of the world once it has detonated or run out: the object
//! maintenance side of `CPhysicsObj::DeleteSelf` (`CObjectMaint::FadeAndDestroyObject`) and the
//! sort of the rest of the queued [`MissileEvent`]s into what the client acts on.

use ac2_physics::MissileEvent;

use crate::remotes::Remotes;

/// `CObjectMaint::FadeAndDestroyObject`'s fade: `CPhysicsObj::DoSlider(0, 0x40000015, 0, fade)`
/// moves the opacity slider (`PHYSICSOBJREP::SLIDER_OPACITY`) to clear over the fade time.
pub const FADE_SLIDER: u32 = 0x4000_0015;

/// The queued events of one quantum, sorted by who answers them.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct MissileFollow {
    /// `(object, seconds)`: `CObjectMaint::FadeAndDestroyObject` ran on the object (its fade is
    /// already moving); the object is deleted that many seconds from now.
    pub destroy: Vec<(u64, f64)>,
    /// `(missile, object)`: `CPhysicsObj::StickySetParent(object)` on the missile.
    pub stick: Vec<(u64, u64)>,
    /// `(object, on)`: `CPhysicsObj::StartStreak` / `StopStreak`.
    pub streaks: Vec<(u64, bool)>,
    /// The callbacks to the object's weenie (`MissileReportOutOfFuel` slot `0x80`,
    /// `MissileReportExpired` `0x84`, an error code `0x8c`).
    pub weenie: Vec<(u64, MissileEvent)>,
}

impl<B> Remotes<B> {
    /// `CPhysicsObj::DeleteSelf(fade, delay)` -> `CObjectMaint::FadeAndDestroyObject`: the body's
    /// opacity slider moves to clear over `fade` seconds, and the object is deleted `delay`
    /// seconds on, or at the end of the fade when `delay` is not positive. Returns that time.
    pub fn delete_self(&mut self, id: u64, fade: f64, delay: f64) -> f64 {
        let destruction = if delay > 0.0 { delay } else { fade };
        if let Some(body) = self.body_mut(id) {
            body.do_slider(0, FADE_SLIDER, 0.0, fade as f32);
        }
        destruction
    }

    /// Acts on the events the missiles queued in a quantum (`Remotes::step_quantum`): a
    /// `DeleteSelf` fades the body and says when the object goes.
    pub fn follow_missiles(&mut self, events: &[(u64, MissileEvent)]) -> MissileFollow {
        let mut out = MissileFollow::default();
        for &(id, event) in events {
            match event {
                MissileEvent::Streak { on } => out.streaks.push((id, on)),
                MissileEvent::DeleteSelf { fade, delay } => {
                    let destruction = self.delete_self(id, fade, delay);
                    out.destroy.push((id, destruction));
                }
                MissileEvent::StickTo(object) => out.stick.push((id, object)),
                MissileEvent::OutOfFuel | MissileEvent::Expired | MissileEvent::Error(_) => {
                    out.weenie.push((id, event));
                }
            }
        }
        out
    }
}

#[cfg(test)]
#[path = "missile_follow_tests.rs"]
mod tests;
