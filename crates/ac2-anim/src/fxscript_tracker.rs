//! `FXScriptTracker`: the FX scripts a physics object is playing. `CPhysicsObj::DoFXScript`
//! (`PHYSICSOBJREP::DoFXScript`) adds a track for the script's DID; each animation update
//! (`PHYSICSOBJREP::UpdateFXScriptTracker`, at the refresh step) advances the track's time by
//! `quantum / period`, adds the effect and impulse triggers the time has reached to the object's
//! `TriggerUpdate`, and the animation's own trigger pass then fires them.
//!
//! Retail loads the script inside `FXTrack::FillCache` on the first use. Here the caller resolves
//! pending tracks with [`Ac2FxScriptTracker::resolve`] before the update (the script records are
//! read by the client's loader, never on the physics step); a track whose script cannot be found
//! is dropped, as `FXScriptTracker::Update` drops it.

use std::sync::Arc;

use crate::behavior::{Ac2BehaviorParams, Ac2BehaviorTrack, Ac2TriggerUpdate};
use crate::dbanimator::add_effects_and_impulses;
use crate::fxscript::Ac2FxScript;

/// The time a timeline past one period rests at: `FXScriptTracker::Update` sets `mCTime` to
/// -4 (`-0x40800000`) once it reaches 1, so the timeline runs again after four more periods.
const REST_TIME: f32 = -4.0;

/// One `FXTrack`: the script id, the script once resolved, and the `AnimTrack` that holds the
/// triggers it reached (`mCTime` is the timeline's time in periods, `mWeight` 1).
#[derive(Debug, Clone)]
struct FxTrack {
    id: u32,
    script: Option<Arc<Ac2FxScript>>,
    track: Ac2BehaviorTrack,
}

/// `FXScriptTracker`: the tracks of the FX scripts playing on one physics object.
#[derive(Debug, Clone, Default)]
pub struct Ac2FxScriptTracker {
    tracks: Vec<FxTrack>,
}

impl Ac2FxScriptTracker {
    /// `FXScriptTracker::DoFXScript`: a track for the script `id` at time 0 with weight 1. The
    /// script is attached by [`Self::resolve`]; a missing record drops the track there.
    pub fn do_fx_script(&mut self, id: u32) {
        let mut track = Ac2BehaviorTrack::new(Ac2BehaviorParams::new(0));
        track.ctime = 0.0;
        track.weight = 1.0;
        self.tracks.push(FxTrack {
            id,
            script: None,
            track,
        });
    }

    /// `FXTrack::FillCache` for each track still waiting for its script. A track whose script
    /// `lookup` cannot give is dropped.
    pub fn resolve(&mut self, mut lookup: impl FnMut(u32) -> Option<Arc<Ac2FxScript>>) {
        self.tracks.retain_mut(|node| {
            if node.script.is_none() {
                node.script = lookup(node.id);
            }
            node.script.is_some()
        });
    }

    /// `FXScriptTracker::Update(quantum)`. Each resolved track: its time advances by
    /// `quantum / period` (the `float` store rounds it); the effect and impulse triggers reached
    /// by that time join the track (`FxScript::GetTriggers`); a time at 1 or past it rests at
    /// -4 (`REST_TIME`); the track's triggers go to `triggers` (`TriggerUpdate::AddTrackTriggers`).
    /// A track whose time is no longer a number ends (retail's removal test fails for NaN).
    /// Tracks still waiting for a script are kept.
    pub fn update(&mut self, quantum: f32, triggers: &mut Ac2TriggerUpdate) {
        self.tracks.retain_mut(|node| {
            let Some(script) = node.script.as_deref() else {
                return true;
            };
            let c_time = (f64::from(quantum) / f64::from(script.period)
                + f64::from(node.track.ctime)) as f32;
            node.track.ctime = c_time;
            add_effects_and_impulses(
                &script.effects,
                &script.impulses,
                c_time,
                &mut node.track.triggers,
            );
            if 1.0 <= node.track.ctime {
                node.track.ctime = REST_TIME;
            }
            triggers.add_track_triggers(&mut node.track);
            node.track.ctime <= 1.0
        });
    }

    /// The tracks playing (resolved or waiting).
    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    /// Whether no track is playing.
    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    /// The script ids of the tracks, in the order they were started.
    pub fn script_ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.tracks.iter().map(|node| node.id)
    }
}

#[cfg(test)]
#[path = "fxscript_tracker_tests.rs"]
mod tests;
