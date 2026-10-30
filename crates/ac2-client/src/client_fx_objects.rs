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
use ac2_smartbox::fx_objects::{
    camera_fx_starts, create_alpha_moves, fx_nodes, FxTarget, TargetCall,
};
use ac2_smartbox::remotes::LocalStart;
use ac2_world::{land_block_size, GameClock};

use crate::client::Client;
use crate::frame::FxFollow;
use crate::remote::{Context, RemoteCalls};
use ac2_render::fx_decals::EmitterPlace;
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
            terrain: 0,
        };
        for (id, on) in std::mem::take(&mut self.pending_streaks) {
            self.fx_streaks.set_object_streaks(id, on, &streak_ctx);
        }
        for (id, visual, events) in events {
            // `FXManager::m_terrain` of this emitter: the FX nodes keyed by terrain follow it.
            let terrain = self.fx_terrain(*id);
            let streak_ctx = ac2_render::fx_streaks::StreakContext {
                terrain,
                ..streak_ctx
            };
            let particles = self
                .remotes
                .body_data(*visual)
                .and_then(|data| data.fx_particles.clone());
            let particles = particles.as_deref();
            for event in events {
                match *event {
                    Ac2AnimEvent::StartFx {
                        fx,
                        intensity,
                        target,
                    } => {
                        if let Some(table) = particles {
                            let nodes = fx_nodes(table, fx, terrain);
                            // A new FX plays its nodes' camera effects (an FX already running
                            // only takes the new scalar).
                            if self.fx_objects.record(*id, fx).is_none() {
                                let is_player = streak_ctx.player == Some(*id);
                                for (camera_fx, scalar) in
                                    camera_fx_starts(&nodes, intensity, is_player, streak_ctx.night)
                                {
                                    self.request_camera_fx(camera_fx, scalar, *id);
                                }
                            }
                            let mut nodes = ac2_smartbox::fx_objects::gated_nodes(
                                &nodes,
                                streak_ctx.player == Some(*id),
                                streak_ctx.night,
                            );
                            // `FXManager::SoundIsLooping` of each node's sound
                            // (`CalcDuration`).
                            if let Some(sounds) = self
                                .remotes
                                .body_data(*visual)
                                .and_then(|data| data.fx_sounds.as_ref())
                            {
                                for node in &mut nodes {
                                    node.sound_looping = sounds.looping.contains(&node.sound);
                                }
                            }
                            let fx_target = self.fx_target_of(*id, target);
                            let remotes = &mut self.remotes;
                            let scripts = &self.fx_scripts;
                            self.fx_objects.start_targeted(
                                *id,
                                fx,
                                &nodes,
                                now,
                                intensity,
                                &fx_target,
                                &|script| scripts.get(script).map(|s| s.period),
                                &mut || remotes.allocate_temporary_id(),
                            );
                            self.apply_fx_targets();
                            self.apply_slider_fx();
                            self.fx_streaks.start_streaks(*id, fx, table, &streak_ctx);
                            self.fx_repulsors.start(*id, fx, table, &streak_ctx);
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
                            .map(|table| fx_nodes(table, fx, terrain))
                            .unwrap_or_default();
                        self.fx_objects.stop(*id, fx, &nodes, now);
                        self.apply_fx_targets();
                        self.fx_streaks.stop_streaks(*id, fx);
                        self.fx_repulsors.stop(*id, fx);
                        self.fx_decals.stop(*id, fx);
                    }
                    // `CPhysicsObj::DeleteSelf(fade, delay)` of an FX object whose behaviour
                    // ended (`mDestroyOnCompletion`): its opacity slider moves and
                    // `CObjectMaint` deletes it after the delay.
                    Ac2AnimEvent::DeleteSelf { fade, delay } => {
                        let seconds = self.remotes.delete_self(*id, f64::from(fade), delay);
                        self.fx_objects.delete_after(*id, now, seconds);
                    }
                    _ => {}
                }
            }
        }
    }

    /// The target an FX starts on `emitter` with: the event's object and the description the
    /// emitter's behaviour asked to clone (`PHYSICSOBJREP::GetVDescToClone` and the cloned
    /// appearance accessors).
    fn fx_target_of(&self, emitter: u64, target: u64) -> FxTarget {
        let body = if self.player.as_ref().is_some_and(|p| p.id == emitter) {
            self.player
                .as_ref()
                .and_then(|p| p.body.as_ref().map(|b| &b.physics))
        } else {
            self.remotes.get(emitter).and_then(|r| r.body())
        };
        let (vdesc_to_clone, apr, keys) = body.map_or((0, 0, &[][..]), |b| b.fx_clone());
        FxTarget {
            id: target,
            vdesc_to_clone,
            apr,
            apr_keys: keys.to_vec(),
        }
    }

    /// What the FX just started did to objects beyond creating them: the emitters take the objects
    /// they made as their FX target (`PHYSICSOBJREP::SetFXTarget`), and the targets take their
    /// calls (`FXManager::DoTargetAndCloningFXs`): the node's target FX (`DoFX`), the holding
    /// point they hang from (`CPhysicsObj::FXSetParent`) or leave.
    pub(crate) fn apply_fx_targets(&mut self) {
        for (emitter, object) in self.fx_objects.take_new_targets() {
            if let Some(body) = self.script_body_mut(emitter) {
                body.set_fx_target(object);
            }
        }
        for call in self.fx_objects.take_target_calls() {
            match call {
                TargetCall::DoFx { object, fx, scalar } => {
                    if let Some(body) = self.script_body_mut(object) {
                        body.do_fx(fx, scalar);
                    }
                }
                TargetCall::StopFx { object, fx } => {
                    if let Some(body) = self.script_body_mut(object) {
                        body.stop_fx(fx);
                    }
                }
                TargetCall::Parent {
                    object,
                    emitter,
                    attach: Ac2FxObjectAttach::Attached(attach),
                } => {
                    if let Some(remote) = self.remotes.get_mut(object) {
                        remote.fx_follow = Some(FxFollow { emitter, attach });
                    }
                }
                TargetCall::Parent { .. } => {}
                TargetCall::Unparent { object } => {
                    if let Some(remote) = self.remotes.get_mut(object) {
                        remote.fx_follow = None;
                    }
                }
            }
        }
    }

    /// `FXManager::DoSliderFX` for the slider nodes the FX just started: the emitter's body
    /// (`PHYSICSOBJREP::DoSlider(0, ...)`), the player's or a remote's, and with `m_prop_color`
    /// each object it holds (`PHYSICSOBJREP::DoSliderOnChild`).
    fn apply_slider_fx(&mut self) {
        for slider in self.fx_objects.take_sliders() {
            if let Some(body) = self.script_body_mut(slider.emitter) {
                body.do_slider(0, slider.slider, slider.scalar, slider.ramp_time);
            }
            if slider.propagate {
                let children = self.session.objects().children(slider.emitter);
                self.remotes.do_slider_on_child(
                    slider.slider,
                    slider.scalar,
                    slider.ramp_time,
                    &children,
                );
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
        let bodied: Vec<_> = player.into_iter().chain(self.remotes.bodied()).collect();
        for (id, visual) in bodied {
            if !self.fx_objects.mark_startup(id) {
                continue;
            }
            // The tables are shared: held apart from the remotes, which make the FX's objects.
            let Some((table, sounds)) = self
                .remotes
                .body_data(visual)
                .and_then(|data| Some((data.fx_particles.clone()?, data.fx_sounds.clone())))
            else {
                continue;
            };
            let (table, sounds) = (table.as_ref(), sounds.as_deref());
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
                let remotes = &mut self.remotes;
                self.fx_objects
                    .start_startup(id, fx, &objects, &mut || remotes.allocate_temporary_id());
                if let Some(nodes) = table.get(fx) {
                    // `DoStartupFX`: the streak and repulsor nodes make theirs as well.
                    self.fx_streaks.start_startup(id, nodes);
                    self.fx_repulsors.start_startup(id, nodes);
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
            Some(id) != player_id && remotes.get(id).is_none() && !ac2_audio::is_ambient_emitter(id)
        };
        for id in self.fx_objects.emitters() {
            if is_gone(id) {
                self.fx_objects.remove_emitter(id, now);
                self.fx_streaks.forget_emitter(id);
                self.fx_repulsors.forget_emitter(id);
                self.fx_decals.forget_emitter(id);
            }
        }
        // A streak's emitter may have no FX left running: its streaks go with it all the same.
        for id in self.fx_streaks.streaked_emitters() {
            if is_gone(id) {
                self.fx_streaks.remove_streaks(id);
            }
        }
        // The same for a repulsor's emitter (`CPartArray`'s destructor).
        for id in self.fx_repulsors.repulsed_emitters() {
            if is_gone(id) {
                self.fx_repulsors.remove_repulsors(id);
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
                // Never made: its id goes back (`CreateTemporaryObject`'s failure path).
                self.remotes.deallocate_temporary_id(object.id);
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
                LocalStart {
                    alpha_moves: create_alpha_moves(object.ramp_time),
                    behavior: object.behavior,
                },
                &mut cx,
            );
        }
        // `DoCreateObjectFX`'s `DoBehavior` results for the objects whose bodies came in.
        for (id, started) in self.remotes.take_behavior_results() {
            self.fx_objects.behavior_result(id, started, now);
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
    /// `FXManager::m_terrain` of an object: the terrain its body last reported
    /// (`PHYSICSOBJREP::SetTerrainType`), zero before that or without a body.
    pub(crate) fn fx_terrain(&self, id: u64) -> u32 {
        let body = if self.player.as_ref().is_some_and(|p| p.id == id) {
            self.player
                .as_ref()
                .and_then(|p| p.body.as_ref())
                .map(|b| &b.physics)
        } else {
            self.remotes.get(id).and_then(|r| r.body())
        };
        body.map_or(0, |b| b.fx_terrain_type())
    }

    /// The emitter's place for an FX decal: its cell, block-local origin and heading, `None` while it has no
    /// body (`DoDecalFX` needs the physics rep).
    fn emitter_place(&self, id: u64) -> Option<EmitterPlace> {
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
            EmitterPlace {
                cell: p.cell_id,
                origin: Vec3::new(o.x, o.y, o.z),
                // `Frame::fl2gv`'s second row: the local y axis in global axes.
                forward: {
                    let f = p
                        .frame
                        .local_to_global_vec(ac2_physics::Vector::new(0.0, 1.0, 0.0));
                    Vec3::new(f.x, f.y, f.z)
                },
            }
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

    /// The repulsor calls the FX made since the renderer last took them, in order.
    pub fn take_object_repulsor_calls(
        &mut self,
    ) -> Vec<ac2_render::fx_repulsors::ObjectRepulsorCall> {
        self.fx_repulsors.take_calls()
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

impl Client {
    /// The local description `id` is drawn with: its own, with the appearances its FX applied
    /// merged on (`CPhysicsObj::DoAppearanceFX`: `m_effects_vdesc` applied additively onto
    /// `m_local_vdesc`).
    pub(crate) fn drawn_appearance(
        &self,
        id: u64,
        local: std::sync::Arc<ac2_weenie::Appearance>,
    ) -> std::sync::Arc<ac2_weenie::Appearance> {
        match self.fx_objects.appearance_effects(id) {
            Some(effects) => {
                let mut drawn = (*local).clone();
                drawn.apply_additive_appearance(effects);
                std::sync::Arc::new(drawn)
            }
            None => local,
        }
    }
}

#[cfg(test)]
#[path = "fx_object_behavior_tests.rs"]
mod behavior_tests;
