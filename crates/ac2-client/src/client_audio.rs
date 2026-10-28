//! The client's sound under `Turbine::Client::UseTime`: after `CPhysics::UpdateAllAnimation`,
//! the effects the objects' animations started and stopped run through each object's
//! `FXManager` sound part, and `SoundManager::UseTime` hears them from the render camera
//! (`SmartBox::mRenderCamera.mCurrentPosition`, zero velocity). Every DAT read happened on the
//! loader thread with the object's body ([`FxSoundData`]). The sound behaviour (the manager's
//! emitters, ambient, rain, portal and the screens' sounds) is `ac2-audio`'s; this file starts
//! the FX's sounds and places the emitters through [`PhysicsSpace`].

use std::collections::hash_map::Entry;
use std::collections::{BTreeMap, HashMap, HashSet};

use ac2_anim::Ac2AnimEvent;
use ac2_assets::{Ac2FxSoundNode, Ac2VisualFxSounds};
use ac2_audio::{
    AmbientDriver, Emitter, PlaySoundData, PortalCue, PortalSounds, RainSounds, ScreenSounds,
    SoundManager,
};
use ac2_core::Random;
use ac2_physics::{CellSource, PhysicsBody, Position};
use ac2_render::portal::PortalSound;

use crate::client::Client;
use crate::sound_space::PhysicsSpace;
use ac2_render::fx_particles::{
    light_starts, particle_starts, ObjectLightStart, ObjectParticleStart,
};

/// A visual's FX sounds, read on the loader thread: `FXTable::GetFXData` per FX id, and which of
/// their sounds loop (`FXManager::SoundIsLooping`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FxSoundData {
    pub table: Ac2VisualFxSounds,
    pub looping: HashSet<u32>,
}

/// `FXState`, the sound part: the FX's scalar ramping to the desired one, the looping sounds
/// it keeps (`m_sound_contexts`) and when it ends (`m_end_time`, negative for never).
#[derive(Debug, Clone, PartialEq)]
struct FxState {
    desired: f32,
    current: f32,
    contexts: Vec<u32>,
    end_time: f64,
}

/// `FXManager`'s sound part for one object (`m_current_fx`).
#[derive(Debug, Default)]
struct FxSounds {
    states: BTreeMap<u32, FxState>,
    /// The FX ids whose particle systems started and have not been stopped (`StopFx`), with the
    /// handles of those systems (`DoParticleFX`'s particle ids).
    particles: HashMap<u32, Vec<u64>>,
}

/// What one effect call needs besides the FX.
struct FxCall<'a> {
    data: &'a FxSoundData,
    emitter: u64,
    is_player: bool,
    now: f64,
}

/// Whether a node plays here (`FXManager::ChannelMissmatch` with no channel preferences set,
/// the player-only flag, and the time of day, taken as day).
fn node_plays(node: &Ac2FxSoundNode, is_player: bool) -> bool {
    (!node.player_only || is_player) && node.day_time
}

/// `FXNode::GetSoundID`: one of the node's sounds (`Random::RollDice`).
pub(crate) fn sound_id(node: &Ac2FxSoundNode, random: &mut Random) -> u32 {
    match node.sounds.len() {
        0 => 0,
        1 => node.sounds[0],
        n => {
            let pick = random.roll_dice(0, n as i32 - 1);
            node.sounds.get(pick as usize).copied().unwrap_or(0)
        }
    }
}

/// The last `LockActions` (true) or `UnlockActions` (false) in a body's animation events.
fn lock_change(events: &[Ac2AnimEvent]) -> Option<bool> {
    events.iter().rev().find_map(|event| match event {
        Ac2AnimEvent::LockActions => Some(true),
        Ac2AnimEvent::UnlockActions => Some(false),
        _ => None,
    })
}

impl FxSounds {
    /// `FXManager::DoTargettedFX`'s sound branch (`DoFX`, no target): a new FX plays one sound
    /// per node through `DoSoundFX` with the object as emitter, keeping the looping ones; an FX
    /// already running takes the new scalar.
    fn do_fx(
        &mut self,
        fx: u32,
        scalar: f32,
        call: &FxCall<'_>,
        manager: &mut SoundManager,
        random: &mut Random,
    ) -> Vec<u32> {
        let mut played_dids = Vec::new();
        let Some(nodes) = call.data.table.get(fx).filter(|nodes| !nodes.is_empty()) else {
            return played_dids;
        };
        if let Some(state) = self.states.get_mut(&fx) {
            state.desired = scalar;
            for node in nodes.iter().filter(|n| node_plays(n, call.is_player)) {
                if node.ramp_time < 0.0002 {
                    state.current = scalar;
                }
            }
            for &handle in &state.contexts {
                manager.set_context_volume(handle, state.current);
            }
            return played_dids;
        }
        let mut state = FxState {
            desired: scalar,
            current: 0.0,
            contexts: Vec::new(),
            end_time: 0.0,
        };
        let mut played = false;
        for node in nodes.iter().filter(|n| node_plays(n, call.is_player)) {
            played = true;
            state.current = if node.ramp_time >= 0.0002 {
                0.0
            } else {
                scalar
            };
            let sound = sound_id(node, random);
            let looping = sound != 0 && call.data.looping.contains(&sound);
            if sound != 0 {
                // `DoSoundFX`.
                let mut data = PlaySoundData::new(sound);
                data.emitter = Some(call.emitter);
                data.volume = state.current;
                let handle = manager.play(&data).unwrap_or(0);
                played_dids.push(sound);
                if looping && !node.fire_and_forget && handle != 0 {
                    state.contexts.push(handle);
                }
            }
            // `CalcDuration`: a looping sound keeps the FX until it is stopped.
            state.end_time = if looping { -1.0 } else { call.now };
        }
        if played {
            self.states.insert(fx, state);
        }
        played_dids
    }

    /// `FXManager::StopFX`: the FX's looping sounds stop.
    fn stop_fx(&mut self, fx: u32, manager: &mut SoundManager) {
        if let Some(state) = self.states.remove(&fx) {
            for handle in state.contexts {
                manager.stop_context(handle, 0.0);
            }
        }
    }

    /// `FXManager::UseTime`: each running FX's scalar steps toward the desired one (once per
    /// node, `quantum / ramp_time`), its sounds follow, and an FX past its end is dropped.
    fn use_time(
        &mut self,
        quantum: f32,
        now: f64,
        data: &FxSoundData,
        is_player: bool,
        manager: &mut SoundManager,
    ) {
        self.states.retain(|fx, state| {
            let nodes = data.table.get(*fx).cloned().unwrap_or_default();
            if state.current != state.desired {
                for node in nodes.iter().filter(|n| node_plays(n, is_player)) {
                    if node.ramp_time >= 0.0002 {
                        let step = quantum / node.ramp_time;
                        if state.desired <= state.current {
                            state.current = (state.current - step).max(state.desired);
                        } else {
                            state.current = (state.current + step).min(state.desired);
                        }
                    } else {
                        state.current = state.desired;
                    }
                    for &handle in &state.contexts {
                        manager.set_context_volume(handle, state.current);
                    }
                }
            }
            !(0.0 <= state.end_time && state.end_time <= now)
        });
    }
}

/// The client's sound manager and every object's FX sounds.
pub(crate) struct ClientAudio {
    pub(crate) manager: SoundManager,
    objects: HashMap<u64, FxSounds>,
    /// The particle systems the objects' FX started and have not been taken by the renderer
    /// (`FXManager::DoParticleFX`), and the handles of those FX stopped or whose objects left.
    pub(crate) object_particles: Vec<ObjectParticleStart>,
    pub(crate) object_particle_stops: Vec<u64>,
    /// The lights the objects' FX placed and have not been taken (`FXManager::DoLightSourceFX`),
    /// and the objects that left the world (their lights go with them).
    pub(crate) object_lights: Vec<ObjectLightStart>,
    pub(crate) object_lights_gone: Vec<u64>,
    /// `Render::m_Prefs.LightSourceEffects`: while off, an FX places no light (its particle
    /// systems still start; `DoLightSourceFX` returns without a light).
    pub(crate) light_effects: bool,
    /// The last handle handed out (`ObjectParticleStart::handle`).
    particle_handles: u64,
    /// `WorldRender`'s rain loop and thunder.
    pub(crate) rain: RainSounds,
    /// The ambient sound and its driver (`WorldRender::UpdateAmbientSounds`).
    pub(crate) ambient: AmbientDriver<Position>,
    /// The screens' music (pregame, story) and the credits' sound.
    pub(crate) screen: ScreenSounds,
    /// The portal space's sounds and its tunnel loops (`PortalSpace::Effect`).
    pub(crate) portal: PortalSounds,
}

/// `DirectXPlayingSound::UpdatePosition`'s view of an object from the listener, through the
/// physics space: the body's place and velocity.
fn body_emitter(
    space: &PhysicsSpace<'_>,
    listener: &Position,
    body: &PhysicsBody,
    in_world: bool,
) -> Emitter {
    ac2_audio::emitter(
        space,
        *listener,
        body.position(),
        body.movement_state().velocity,
        in_world,
    )
}

impl Client {
    /// Gives the client its sound manager (`SoundManager::s_pSoundMgr`): the objects'
    /// animation effects play through it, and it runs every frame.
    /// Gives the sound manager the sound preferences the bank holds (`SoundManager::HandleApply`).
    pub fn apply_sound_preferences(&mut self, bank: &ac2_prefs::PreferenceBank) {
        if let Some(audio) = self.audio.as_mut() {
            ac2_audio::prefs::apply(&mut audio.manager, bank);
        }
    }

    /// `Render::m_Prefs.LightSourceEffects`: the FX lights follow the saved preference.
    pub fn apply_light_preferences(&mut self, bank: &ac2_prefs::PreferenceBank) {
        if let Some(audio) = self.audio.as_mut() {
            audio.light_effects = ac2_render::prefs::light_source_effects(bank);
        }
    }

    pub fn attach_audio(&mut self, manager: SoundManager) {
        self.audio = Some(ClientAudio {
            manager,
            objects: HashMap::new(),
            object_particles: Vec::new(),
            object_particle_stops: Vec::new(),
            object_lights: Vec::new(),
            object_lights_gone: Vec::new(),
            light_effects: true,
            particle_handles: 0,
            ambient: Default::default(),
            screen: Default::default(),
            portal: Default::default(),
            rain: Default::default(),
        });
    }

    /// The portal space's sound DIDs (see [`crate::portal_sound_ids`]); set once the DAT is open.
    pub fn set_portal_sound_ids(&mut self, ids: ac2_audio::PortalSoundIds) {
        if let Some(audio) = self.audio.as_mut() {
            audio.portal.set_ids(ids);
        }
    }

    /// The rain sound DIDs (see [`crate::rain_sound_ids`]); set once the DAT is open.
    pub fn set_rain_sound_ids(&mut self, ids: ac2_audio::RainSoundIds) {
        if let Some(audio) = self.audio.as_mut() {
            audio.rain.set_ids(ids);
        }
    }

    /// The FX scripts the bodies' `DoFXScript` plays (see [`ac2_anim::load_fx_scripts`]).
    pub fn set_fx_scripts(&mut self, scripts: ac2_anim::FxScripts) {
        self.fx_scripts = scripts;
    }

    /// The particle systems the objects' FX started since the renderer last took them, in the
    /// order they started; the renderer starts them on the objects.
    pub fn take_object_particle_starts(&mut self) -> Vec<ObjectParticleStart> {
        self.audio
            .as_mut()
            .map(|audio| std::mem::take(&mut audio.object_particles))
            .unwrap_or_default()
    }

    /// The handles of the particle systems stopped since the renderer last took them (their FX
    /// stopped, or their object left the world).
    pub fn take_object_particle_stops(&mut self) -> Vec<u64> {
        self.audio
            .as_mut()
            .map(|audio| std::mem::take(&mut audio.object_particle_stops))
            .unwrap_or_default()
    }

    /// The lights the objects' FX placed since the renderer last took them, in the order they
    /// started (`FXManager::DoLightSourceFX`).
    pub fn take_object_light_starts(&mut self) -> Vec<ObjectLightStart> {
        self.audio
            .as_mut()
            .map(|audio| std::mem::take(&mut audio.object_lights))
            .unwrap_or_default()
    }

    /// The objects that left the world since the renderer last took them; their FX lights go.
    pub fn take_object_lights_gone(&mut self) -> Vec<u64> {
        self.audio
            .as_mut()
            .map(|audio| std::mem::take(&mut audio.object_lights_gone))
            .unwrap_or_default()
    }

    /// `WorldRender::UpdateRainSounds` for the frame: the rain amount at the server time (the
    /// region's sky, `CDayDesc::GetRain`), zero where the camera's cell cannot see outside.
    pub(crate) fn rain_use_time(&mut self, now: f64) {
        let amount = self
            .region
            .as_ref()
            .and_then(|region| region.sample_sky_environment(self.session.cur_time()))
            .map_or(0.0, |sample| sample.rain().amount());
        let can_see_outside = self
            .player
            .as_ref()
            .filter(|p| p.in_world)
            .is_some_and(|p| {
                let cell = p.position().cell_id;
                ac2_audio::is_outside(cell)
                    || self
                        .terrain
                        .env_cell(cell)
                        .is_some_and(|env| env.seen_outside)
            });
        let Some(audio) = self.audio.as_mut() else {
            return;
        };
        audio.rain.update(
            amount,
            can_see_outside,
            now,
            &mut self.random,
            &mut audio.manager,
        );
    }

    /// `PortalSpace::Effect::StartNow` and `StopNow`'s sounds, from the effect's events since
    /// the last frame.
    pub(crate) fn play_portal_sounds(&mut self) {
        let events = self.portal.take_sounds();
        if events.is_empty() {
            return;
        }
        let cues: Vec<PortalCue> = events
            .iter()
            .map(|event| match event {
                PortalSound::Start => PortalCue::Start,
                PortalSound::Stop => PortalCue::Stop,
            })
            .collect();
        if let Some(audio) = self.audio.as_mut() {
            audio.portal.play(&mut audio.manager, &cues);
        }
    }

    /// `gmCreditsUI::UpdateSound`'s `SoundManager::Play` of `did` on `Channel_Main`, its handle
    /// kept for the next check.
    pub fn play_credit_sound(&mut self, did: u32) {
        let Some(audio) = self.audio.as_mut() else {
            return;
        };
        audio.screen.play_credit(&mut audio.manager, did);
    }

    /// Whether the credits' sound still plays (its handle, and the sound has not finished).
    pub fn credit_sound_busy(&self) -> bool {
        self.audio
            .as_ref()
            .is_some_and(|audio| audio.screen.credit_busy(&audio.manager))
    }

    /// `gmStoryUI::Play`'s sounds for a keyframe: its music and its voice, each replacing the
    /// story's last one. Without a sound manager, nothing plays.
    pub(crate) fn play_story_sounds(&mut self, music: Option<u32>, voice: Option<u32>) {
        let Some(audio) = self.audio.as_mut() else {
            return;
        };
        audio.screen.play_story(&mut audio.manager, music, voice);
    }

    /// `gmStoryUI::Finish`: the story's music and voice fade out over two seconds and forget
    /// their handles.
    pub(crate) fn stop_story_sounds(&mut self) {
        let Some(audio) = self.audio.as_mut() else {
            return;
        };
        audio.screen.stop_story(&mut audio.manager);
    }

    /// A pregame screen's music (`gmIntroUI`, `gmCharacterSelectionUI` and
    /// `gmAvatarModificationUI` constructors): the previous screen's music fades out over two
    /// seconds, then the screen's plays on `Channel_Main` at full volume, its handle kept for
    /// the next screen.
    pub fn play_pregame_music(&mut self, did: u32) {
        let Some(audio) = self.audio.as_mut() else {
            return;
        };
        audio.screen.play_pregame(&mut audio.manager, did);
    }

    /// The screen music's playing sound, if one plays (its handle).
    pub fn pregame_music_handle(&self) -> Option<u32> {
        self.audio.as_ref()?.screen.pregame_handle()
    }

    /// The attached sound manager (the UI plays its sounds through it).
    pub fn audio_mut(&mut self) -> Option<&mut SoundManager> {
        self.audio.as_mut().map(|audio| &mut audio.manager)
    }

    /// The height of the water surface over the cell an object stands in (`None` without water,
    /// or when the object is not in the world).
    fn water_surface_under(&self, id: u64, player_id: Option<u64>) -> Option<f32> {
        let cell = if Some(id) == player_id {
            self.player.as_ref()?.position().cell_id
        } else {
            self.remotes.get(id)?.position()?.cell_id
        };
        self.terrain.water_surface_z(cell)
    }

    /// After `CPhysics::UpdateAllAnimation`: each object's started and stopped effects, then
    /// its `FXManager::UseTime` for the step. The objects' part runs first and needs no sound
    /// device (`client_fx_objects`); the sound and particle parts run only with one.
    pub(crate) fn animation_sounds(&mut self, step: f32, now: f64) {
        self.start_startup_fx();
        // The player's events come first: its latent script physics finish on them.
        let player_taken = self.player.as_mut().filter(|p| p.in_world).and_then(|p| {
            let (id, visual) = (p.id, p.visual);
            p.body
                .as_mut()
                .map(|body| (id, visual, body.physics.take_animation_events()))
        });
        let player_id = player_taken.as_ref().map(|(id, _, _)| *id);
        if let Some((id, _, player_events)) = &player_taken {
            let completions = crate::client_latent::completions(player_events);
            self.finish_latent(*id, completions);
        }
        let mut events = self.remotes.take_animation_events();
        if let Some(taken) = player_taken {
            events.push(taken);
        }
        // The player's `LockActions` / `UnlockActions` events (`PlayerMovementModule::LockActions`)
        // count whatever the visual's sounds are.
        let lock = events
            .iter()
            .filter(|(id, _, _)| Some(*id) == player_id)
            .filter_map(|(_, _, events)| lock_change(events))
            .next_back();
        self.fx_object_events(&events, now);
        // The water surface under each emitter at this step (`FireParticleFX` reads it when a
        // fire-and-forget node starts).
        let water: HashMap<u64, Option<f32>> = events
            .iter()
            .map(|(id, _, _)| (*id, self.water_surface_under(*id, player_id)))
            .collect();
        if let Some(audio) = self.audio.as_mut() {
            for (id, visual, events) in events {
                let visual_data = self.remotes.body_data(visual);
                let sounds = visual_data.and_then(|data| data.fx_sounds.as_deref());
                let particles = visual_data.and_then(|data| data.fx_particles.as_deref());
                if sounds.is_none() && particles.is_none() {
                    continue;
                }
                let is_player = Some(id) == player_id;
                let fx = audio.objects.entry(id).or_default();
                for event in events {
                    match event {
                        Ac2AnimEvent::StartFx {
                            fx: effect,
                            intensity,
                        } => {
                            if let Some(data) = sounds {
                                let call = FxCall {
                                    data,
                                    emitter: id,
                                    is_player,
                                    now,
                                };
                                fx.do_fx(
                                    effect,
                                    intensity,
                                    &call,
                                    &mut audio.manager,
                                    &mut self.random,
                                );
                            }
                            if let Some(table) = particles {
                                // `FXManager::DoTargettedFX`: an FX already running keeps its
                                // systems and lights.
                                if let Entry::Vacant(slot) = fx.particles.entry(effect) {
                                    let mut starts = particle_starts(
                                        id,
                                        effect,
                                        table,
                                        &mut self.random,
                                        water.get(&id).copied().flatten(),
                                    );
                                    let mut handles = Vec::with_capacity(starts.len());
                                    for start in &mut starts {
                                        audio.particle_handles += 1;
                                        start.handle = audio.particle_handles;
                                        // `FXManager::DoParticleFX`: only a looping system keeps
                                        // its handle, so `StopFX` destroys it and no other.
                                        if start.looping {
                                            handles.push(start.handle);
                                        }
                                    }
                                    slot.insert(handles);
                                    audio.object_particles.extend(starts);
                                    if audio.light_effects {
                                        audio.object_lights.extend(light_starts(id, effect, table));
                                    }
                                }
                            }
                        }
                        Ac2AnimEvent::StopFx { fx: effect } => {
                            // A startup FX runs on until forced (`FXManager::StopFX`'s
                            // `m_bStartupFX` check, with no force flag from the animation).
                            if self
                                .fx_objects
                                .record(id, effect)
                                .is_some_and(|r| r.startup)
                            {
                                continue;
                            }
                            if sounds.is_some() {
                                fx.stop_fx(effect, &mut audio.manager);
                            }
                            // `FXManager::StopFX` destroys the FX's particle systems by their ids.
                            if let Some(handles) = fx.particles.remove(&effect) {
                                audio.object_particle_stops.extend(handles);
                            }
                        }
                        _ => {}
                    }
                }
                if let Some(data) = sounds {
                    let call = FxCall {
                        data,
                        emitter: id,
                        is_player,
                        now,
                    };
                    fx.use_time(step, now, data, call.is_player, &mut audio.manager);
                }
            }
        }
        // `FXManager::UseTime`'s expired FX stop.
        self.fx_objects.expire(now);
        if let Some(locked) = lock {
            self.lock_player_actions(locked);
        }
        self.apply_fx_objects(now);
    }

    /// `SoundManager::UseTime` from the render camera: objects that left stop their sounds
    /// (`StopSoundsForEmitter`), and every emitter is placed relative to the camera.
    pub(crate) fn audio_use_time(&mut self, now: f64) {
        self.ambient_use_time(now);
        self.groove_use_time(now);
        let ambient = self.ambient_emitters(&self.scene.current);
        let space = PhysicsSpace {
            terrain: &self.terrain,
        };
        let Some(audio) = self.audio.as_mut() else {
            return;
        };
        let player = self.player.as_ref().filter(|p| p.in_world);
        let player_id = player.map(|p| p.id);
        let remotes = &self.remotes;
        let gone: Vec<u64> = audio
            .objects
            .keys()
            .copied()
            .filter(|id| Some(*id) != player_id && remotes.get(*id).is_none())
            .filter(|id| *id & 0xFF00_0000_0000_0000 != ac2_audio::AMBIENT_EMITTERS)
            .collect();
        for id in gone {
            if let Some(fx) = audio.objects.remove(&id) {
                // The object's part array goes with it, and so its particle systems and lights.
                audio
                    .object_particle_stops
                    .extend(fx.particles.into_values().flatten());
                audio.object_lights_gone.push(id);
            }
            audio.manager.stop_sounds_for_emitter(id);
        }
        let listener = self.scene.current;
        let lookup = |id: u64| -> Option<Emitter> {
            if let Some(emitter) = ambient.get(&id) {
                return Some(*emitter);
            }
            if Some(id) == player_id {
                let body = player?.body.as_ref()?;
                return Some(body_emitter(&space, &listener, &body.physics, true));
            }
            let remote = remotes.get(id)?;
            match remote.body() {
                Some(body) => Some(body_emitter(&space, &listener, body, remote.in_world())),
                None => Some(ac2_audio::emitter(
                    &space,
                    listener,
                    remote.position()?,
                    [0.0; 3],
                    remote.in_world(),
                )),
            }
        };
        audio.manager.use_time_with(now, &lookup);
    }
}

#[cfg(test)]
#[path = "fx_effects_tests.rs"]
mod tests;
