//! The FX objects' pass under `FXManager`, which needs no sound device. Each animation step the
//! objects' started and stopped effects run their object parts (`DoTargettedFX`'s and `StopFX`'s,
//! in [`crate::fx_objects`]), each object's startup FX make their objects once per object
//! (`DoStartupFX`), and the objects the FX faded are removed (`CObjectMaint`'s pending deletions).
//! The sound and particle parts of the same events run in `client_audio` when a sound device is
//! attached; the objects do not wait for it.

use ac2_anim::Ac2AnimEvent;
use ac2_assets::{Ac2FxObject, Ac2VisualFxParticles};

use crate::client::Client;
use crate::fx_particles::startup_light_starts;
use crate::remote::Context;

/// The FX nodes that create or hide objects for `fx` (`FXManager::DoTargettedFX`'s node list).
fn fx_nodes(table: &Ac2VisualFxParticles, fx: u32) -> Vec<Ac2FxObject> {
    table
        .get(fx)
        .map(|list| list.iter().map(|a| a.object).collect())
        .unwrap_or_default()
}

impl Client {
    /// `FXManager::DoTargettedFX`'s and `FXManager::StopFX`'s object parts for each object's
    /// events this step, in event order.
    pub(crate) fn fx_object_events(
        &mut self,
        events: &[(u64, Option<u32>, Vec<Ac2AnimEvent>)],
        now: f64,
    ) {
        for (id, visual, events) in events {
            let particles = self
                .remotes
                .body_data(*visual)
                .and_then(|data| data.fx_particles.as_deref());
            for event in events {
                match *event {
                    Ac2AnimEvent::StartFx { fx, .. } => {
                        if let Some(table) = particles {
                            let nodes = fx_nodes(table, fx);
                            self.fx_objects.start(*id, fx, &nodes, now);
                        }
                    }
                    Ac2AnimEvent::StopFx { fx } => {
                        let nodes = particles
                            .map(|table| fx_nodes(table, fx))
                            .unwrap_or_default();
                        self.fx_objects.stop(*id, fx, &nodes, now);
                    }
                    _ => {}
                }
            }
        }
    }

    /// `CPhysicsObj::InitObjectEnd` -> `ApplyStartupFX(rep, object)`: each object whose body
    /// exists runs its visual's startup FX once. The objects they create are made here; their
    /// lights (`DoLightSourceFX`, gated by `LightSourceEffects`) and sounds (`DoSoundFX`) need
    /// the sound pass's state, so they start only with a sound device.
    pub(crate) fn start_startup_fx(&mut self) {
        let player = self
            .player
            .as_ref()
            .filter(|p| p.body.is_some())
            .map(|p| (p.id, p.visual));
        for (id, visual) in player.into_iter().chain(self.remotes.bodied()) {
            if !self.fx_objects.mark_startup(id) {
                continue;
            }
            let Some(data) = self.remotes.body_data(visual) else {
                continue;
            };
            let Some(table) = data.fx_particles.as_deref() else {
                continue;
            };
            let sounds = data.fx_sounds.as_deref();
            if let Some(audio) = self.audio.as_mut() {
                if audio.light_effects {
                    audio.object_lights.extend(startup_light_starts(id, table));
                }
            }
            for &fx in table.startup() {
                if let (Some(audio), Some(nodes)) =
                    (self.audio.as_mut(), sounds.and_then(|s| s.table.get(fx)))
                {
                    for node in nodes.iter() {
                        let sound = crate::client_audio::sound_id(node, &mut self.random);
                        if sound != 0 {
                            let mut play = ac2_audio::PlaySoundData::new(sound);
                            play.emitter = Some(id);
                            audio.manager.play(&play);
                        }
                    }
                }
                let objects: Vec<Ac2FxObject> = table
                    .get(fx)
                    .map(|list| list.iter().map(|a| a.object).collect())
                    .unwrap_or_default();
                self.fx_objects.start_startup(id, fx, &objects);
            }
        }
    }

    /// The emitters that left the world stop their FX (`DestroyAllTransientFX`), and the objects
    /// that left forget their startup FX, so they run it again when they return.
    pub(crate) fn fx_objects_use_time(&mut self, now: f64) {
        let player_id = self.player.as_ref().filter(|p| p.in_world).map(|p| p.id);
        let remotes = &self.remotes;
        self.fx_objects
            .retain_startup(|id| Some(id) == player_id || remotes.tracks(id));
        for id in self.fx_objects.emitters() {
            let gone = Some(id) != player_id
                && remotes.get(id).is_none()
                && id & 0xFF00_0000_0000_0000 != ac2_audio::AMBIENT_EMITTERS;
            if gone {
                self.fx_objects.remove_emitter(id, now);
            }
        }
    }

    /// The FX objects the FX asked for are created at their emitter, and the faded ones whose
    /// destruction has come are removed (`CObjectMaint`'s pending deletions).
    pub(crate) fn apply_fx_objects(&mut self, now: f64) {
        let pending = self.fx_objects.take_pending();
        let player_id = self.player.as_ref().map(|p| p.id);
        let player = self
            .player
            .as_ref()
            .filter(|p| p.body.is_some())
            .map(|p| p.position());
        for object in pending {
            let at = if Some(object.emitter) == player_id {
                player
            } else {
                self.remotes.get(object.emitter).and_then(|r| r.position())
            };
            let Some(position) = at else {
                continue;
            };
            let mut cx = Context {
                session: &mut self.session,
                terrain: &self.terrain,
                player,
            };
            self.remotes
                .add_local(object.id, object.visual, position, &mut cx);
        }
        for id in self.fx_objects.step(now) {
            if self.remotes.is_local(id) {
                self.remotes.removed(id);
            }
        }
        self.request_remote_bodies();
    }
}
