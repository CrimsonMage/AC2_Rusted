//! The client's sound under `Turbine::Client::UseTime`: after `CPhysics::UpdateAllAnimation`,
//! the effects the objects' animations started and stopped run through each object's
//! `FXManager` sound part, and `SoundManager::UseTime` hears them from the render camera
//! (`SmartBox::mRenderCamera.mCurrentPosition`, zero velocity). Every DAT read happened on the
//! loader thread with the object's body ([`FxSoundData`]).

use std::collections::{BTreeMap, HashMap, HashSet};

use ac2_anim::Ac2AnimEvent;
use ac2_assets::{Ac2FxSoundNode, Ac2VisualFxSounds};
use ac2_audio::{Emitter, PlaySoundData, SoundManager};
use ac2_core::Random;
use ac2_physics::{PhysicsBody, Position, Vector};

use crate::client::Client;

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
fn sound_id(node: &Ac2FxSoundNode, random: &mut Random) -> u32 {
    match node.sounds.len() {
        0 => 0,
        1 => node.sounds[0],
        n => {
            let pick = random.roll_dice(0, n as i32 - 1);
            node.sounds.get(pick as usize).copied().unwrap_or(0)
        }
    }
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
    ) {
        let Some(nodes) = call.data.table.get(fx).filter(|nodes| !nodes.is_empty()) else {
            return;
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
            return;
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
    pub(crate) ambient: crate::client_ambient::ClientAmbient,
}

/// `DirectXPlayingSound::UpdatePosition`'s view of an object from the listener:
/// `globaltolocalvec(listener, PositionOffset::GetOffset(listener, object))`, the velocity the
/// same way, muted when it is not in the world (`CPhysicsObj::ShouldMuteSound`'s last case).
fn emitter(
    listener: &Position,
    position: &Position,
    velocity: [f32; 3],
    in_world: bool,
) -> Emitter {
    let offset = listener
        .frame
        .global_to_local_vec(listener.offset_to(position));
    let [vx, vy, vz] = velocity;
    let velocity = listener.frame.global_to_local_vec(Vector::new(vx, vy, vz));
    Emitter {
        offset: [offset.x, offset.y, offset.z],
        velocity: [velocity.x, velocity.y, velocity.z],
        muted: !in_world,
    }
}

fn body_emitter(listener: &Position, body: &PhysicsBody, in_world: bool) -> Emitter {
    emitter(
        listener,
        &body.position(),
        body.movement_state().velocity,
        in_world,
    )
}

impl Client {
    /// Gives the client its sound manager (`SoundManager::s_pSoundMgr`): the objects'
    /// animation effects play through it, and it runs every frame.
    pub fn attach_audio(&mut self, manager: SoundManager) {
        self.audio = Some(ClientAudio {
            manager,
            objects: HashMap::new(),
            ambient: Default::default(),
        });
    }

    /// The attached sound manager (the UI plays its sounds through it).
    pub fn audio_mut(&mut self) -> Option<&mut SoundManager> {
        self.audio.as_mut().map(|audio| &mut audio.manager)
    }

    /// After `CPhysics::UpdateAllAnimation`: each object's started and stopped effects, then
    /// its `FXManager::UseTime` for the step.
    pub(crate) fn animation_sounds(&mut self, step: f32, now: f64) {
        let Some(audio) = self.audio.as_mut() else {
            return;
        };
        let mut events = self.remotes.take_animation_events();
        let player = self.player.as_mut().filter(|p| p.in_world);
        let player_id = player.as_ref().map(|p| p.id);
        if let Some(player) = player {
            if let Some(body) = player.body.as_mut() {
                events.push((
                    player.id,
                    player.visual,
                    body.physics.take_animation_events(),
                ));
            }
        }
        for (id, visual, events) in events {
            let Some(data) = self
                .remotes
                .body_data(visual)
                .and_then(|data| data.fx_sounds.as_deref())
            else {
                continue;
            };
            let call = FxCall {
                data,
                emitter: id,
                is_player: Some(id) == player_id,
                now,
            };
            let fx = audio.objects.entry(id).or_default();
            for event in events {
                match event {
                    Ac2AnimEvent::StartFx {
                        fx: effect,
                        intensity,
                    } => {
                        fx.do_fx(
                            effect,
                            intensity,
                            &call,
                            &mut audio.manager,
                            &mut self.random,
                        );
                    }
                    Ac2AnimEvent::StopFx { fx: effect } => fx.stop_fx(effect, &mut audio.manager),
                    _ => {}
                }
            }
            fx.use_time(step, now, data, call.is_player, &mut audio.manager);
        }
    }

    /// `SoundManager::UseTime` from the render camera: objects that left stop their sounds
    /// (`StopSoundsForEmitter`), and every emitter is placed relative to the camera.
    pub(crate) fn audio_use_time(&mut self, now: f64) {
        self.ambient_use_time(now);
        let ambient = self.ambient_emitters(&self.scene.current);
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
            audio.objects.remove(&id);
            audio.manager.stop_sounds_for_emitter(id);
        }
        let listener = self.scene.current;
        let lookup = |id: u64| -> Option<Emitter> {
            if let Some(emitter) = ambient.get(&id) {
                return Some(*emitter);
            }
            if Some(id) == player_id {
                let body = player?.body.as_ref()?;
                return Some(body_emitter(&listener, &body.physics, true));
            }
            let remote = remotes.get(id)?;
            match remote.body() {
                Some(body) => Some(body_emitter(&listener, body, remote.in_world())),
                None => Some(emitter(
                    &listener,
                    &remote.position()?,
                    [0.0; 3],
                    remote.in_world(),
                )),
            }
        };
        audio.manager.use_time_with(now, &lookup);
    }
}
