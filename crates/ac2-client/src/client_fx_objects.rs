//! The FX objects' pass under `FXManager`, which needs no sound device. Each animation step the
//! objects' started and stopped effects run their object parts (`DoTargettedFX`'s and `StopFX`'s,
//! in [`crate::fx_objects`]), each object's startup FX make their objects once per object
//! (`DoStartupFX`), and the objects the FX faded are removed (`CObjectMaint`'s pending deletions).
//! The sound and particle parts of the same events run in `client_audio` when a sound device is
//! attached; the objects do not wait for it.

use ac2_anim::Ac2AnimEvent;
use ac2_assets::{Ac2FxObject, Ac2FxObjectAttach};
use ac2_math::Vec3;
use ac2_physics::placed_at;
use ac2_smartbox::fx_objects::{create_alpha_moves, fx_nodes};
use ac2_world::{land_block_size, GameClock};

use crate::client::Client;
use crate::frame::FxFollow;
use crate::remote::{Context, RemoteCalls};
use ac2_render::fx_particles::startup_light_starts;

impl Client {
    /// `FXManager::DoTargettedFX`'s and `FXManager::StopFX`'s object parts for each object's
    /// events this step, in event order.
    pub(crate) fn fx_object_events(
        &mut self,
        events: &[(u64, Option<u32>, Vec<Ac2AnimEvent>)],
        now: f64,
    ) {
        // The holding-point children (`CHILDLIST`'s holding entries) and the time of day, as the
        // streak part of `DoTargettedFX` reads them.
        let holds: Vec<(u64, u64, u32)> = self
            .session
            .objects()
            .iter()
            .filter_map(|o| o.parent.map(|(parent, location)| (parent, o.id, location)))
            .collect();
        let streak_ctx = ac2_render::fx_streaks::StreakContext {
            player: self.player.as_ref().map(|p| p.id),
            night: self.game_clock.as_ref().is_some_and(GameClock::is_night),
            holds: &holds,
        };
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
                            self.fx_streaks.start_streaks(*id, fx, table, &streak_ctx);
                            let at = self.emitter_place(*id);
                            self.fx_decals.start(
                                *id,
                                fx,
                                table,
                                at,
                                &streak_ctx,
                                land_block_size(),
                                now,
                                &mut self.random,
                            );
                        }
                    }
                    Ac2AnimEvent::StopFx { fx } => {
                        let nodes = particles
                            .map(|table| fx_nodes(table, fx))
                            .unwrap_or_default();
                        self.fx_objects.stop(*id, fx, &nodes, now);
                        self.fx_streaks.stop_streaks(*id, fx);
                        self.fx_decals.stop(*id, fx);
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
    pub(crate) fn start_startup_fx(&mut self, now: f64) {
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
                if let Some(nodes) = table.get(fx) {
                    let at = self.emitter_place(id);
                    self.fx_decals.start_startup(
                        nodes,
                        at,
                        land_block_size(),
                        now,
                        &mut self.random,
                    );
                }
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
        let is_gone = |id: u64| {
            Some(id) != player_id
                && remotes.get(id).is_none()
                && id & 0xFF00_0000_0000_0000 != ac2_audio::AMBIENT_EMITTERS
        };
        for id in self.fx_objects.emitters() {
            if is_gone(id) {
                self.fx_objects.remove_emitter(id, now);
                self.fx_streaks.forget_emitter(id);
                self.fx_decals.forget_emitter(id);
            }
        }
        // A streak's emitter may have no FX left running: its streaks go with it all the same.
        for id in self.fx_streaks.streaked_emitters() {
            if is_gone(id) {
                self.fx_streaks.remove_streaks(id);
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
            let Some(position) = placed_at(at, object.attach) else {
                continue;
            };
            let follow = match object.attach {
                Ac2FxObjectAttach::Attached(attach) => Some(FxFollow {
                    emitter: object.emitter,
                    attach,
                }),
                Ac2FxObjectAttach::Emitter | Ac2FxObjectAttach::Unplaced => None,
            };
            let mut cx = Context {
                session: &mut self.session,
                terrain: &mut self.terrain,
                player,
            };
            self.remotes.add_local(
                object.id,
                object.visual,
                position,
                follow,
                create_alpha_moves(object.ramp_time),
                &mut cx,
            );
        }
        for id in self.fx_objects.step(now) {
            if self.remotes.is_local(id) {
                self.remotes.removed(id, &mut self.terrain);
            }
        }
        self.request_remote_bodies();
    }
}

impl Client {
    /// The emitter's place for an FX decal: its cell and block-local origin, `None` while it has no
    /// body (`DoDecalFX` needs the physics rep).
    fn emitter_place(&self, id: u64) -> Option<(u32, Vec3)> {
        let is_player = self.player.as_ref().is_some_and(|p| p.id == id);
        let at = if is_player {
            self.player
                .as_ref()
                .filter(|p| p.body.is_some())
                .map(|p| p.position())
        } else {
            self.remotes.get(id).and_then(|r| r.position())
        };
        at.map(|p| {
            let o = p.frame.origin;
            (p.cell_id, Vec3::new(o.x, o.y, o.z))
        })
    }

    /// The decal calls the FX made since the renderer last took them, in order.
    pub fn take_object_decal_calls(&mut self) -> Vec<ac2_render::decal::DecalSpec> {
        self.fx_decals.take_calls()
    }

    /// `Render::m_Prefs.DecalEffects`: the decal preference the bank holds.
    pub fn apply_decal_preferences(&mut self, bank: &ac2_prefs::PreferenceBank) {
        self.fx_decals
            .set_enabled(ac2_render::prefs::decal_effects(bank));
    }

    /// The streak calls the FX started since the renderer last took them, in order.
    pub fn take_object_streak_calls(&mut self) -> Vec<ac2_render::fx_streaks::ObjectStreakCall> {
        self.fx_streaks.take_calls()
    }

    /// `Render::m_Prefs.StreakEffects`: the streak preference the bank holds.
    pub fn apply_streak_preferences(&mut self, bank: &ac2_prefs::PreferenceBank) {
        self.fx_streaks
            .set_enabled(ac2_render::prefs::streak_effects(bank));
    }

    /// Whether the FX make and draw streaks (`StreakEffects` is on).
    pub fn streaks_enabled(&self) -> bool {
        self.fx_streaks.enabled()
    }
}
