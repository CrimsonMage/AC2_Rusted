//! The client's side of the ambient sound (`WorldRender::UpdateAmbientSounds`). The loader reads
//! the region's sound data and each landblock's points on its thread ([`AmbientLoaded`]); the
//! driver that weighs the environments around the player is `ac2_audio::AmbientDriver`, and the
//! world's answers come from [`PhysicsSpace`].

use std::collections::HashMap;
use std::sync::Arc;

use ac2_audio::{AmbientData, AmbientPoint, AmbientRequest, Emitter};
use ac2_physics::Position;
use ac2_world::Ac2AmbientSoundPoint;

use crate::client::Client;
use crate::loader::Request;
use crate::sound_space::PhysicsSpace;

/// What the loader read for the ambient sound.
#[derive(Debug, Clone)]
pub enum AmbientLoaded {
    Data(Option<Arc<AmbientData>>),
    Points(u32, Vec<Ac2AmbientSoundPoint>),
}

impl Client {
    pub(crate) fn ambient_loaded(&mut self, loaded: AmbientLoaded) {
        let Some(audio) = self.audio.as_mut() else {
            return;
        };
        match loaded {
            AmbientLoaded::Data(data) => audio.ambient.set_data(data),
            AmbientLoaded::Points(block, points) => audio.ambient.set_points(
                block,
                points
                    .into_iter()
                    .map(|point| AmbientPoint {
                        sound_type: point.sound_type,
                        origin: point.origin,
                    })
                    .collect(),
            ),
        }
    }

    /// `WorldRender::UpdateAmbientSounds` and `AmbientSound::UseTime` for the frame, with the
    /// loader reads the driver asks for.
    pub(crate) fn ambient_use_time(&mut self, now: f64) {
        let mut requests = Vec::new();
        {
            let Some(audio) = self.audio.as_mut() else {
                return;
            };
            let listener = self
                .player
                .as_ref()
                .filter(|p| p.in_world)
                .map(|p| p.position());
            let space = PhysicsSpace {
                terrain: &self.terrain,
            };
            audio
                .ambient
                .use_time(&space, listener, now, &mut audio.manager, &mut requests);
        }
        for request in requests {
            self.loader.request(match request {
                AmbientRequest::Data => Request::AmbientData,
                AmbientRequest::Points(block) => Request::AmbientPoints(block),
            });
        }
    }

    /// The ambient sound types the cells have weighed (`AmbientSound::sound_types`).
    pub fn ambient_sound_types(&self) -> Vec<u32> {
        self.audio
            .as_ref()
            .map_or_else(Vec::new, |audio| audio.ambient.sound.sound_types())
    }

    /// Every ambient emitter's sound id and its place from the listener, global axes
    /// (`AmbientSound::emitters`).
    pub fn ambient_sound_emitters(&self) -> Vec<(u64, [f32; 3])> {
        self.audio
            .as_ref()
            .map_or_else(Vec::new, |audio| audio.ambient.sound.emitters())
    }

    /// The dominant ambient environment's `EnvInfo` (its reverb), once the cells were weighed.
    pub fn ambient_environment(&self) -> Option<u32> {
        self.audio.as_ref()?.ambient.sound.environment()
    }

    /// The ambient emitters from the listener (the viewer the ambient sound places around).
    pub(crate) fn ambient_emitters(&self, listener: &Position) -> HashMap<u64, Emitter> {
        let Some(audio) = self.audio.as_ref() else {
            return HashMap::new();
        };
        let space = PhysicsSpace {
            terrain: &self.terrain,
        };
        audio.ambient.emitters_from(&space, *listener)
    }
}
