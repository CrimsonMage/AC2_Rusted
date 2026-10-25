//! A headless output: the mixer behind a [`SoundSink`] that applies each command on the calling
//! thread and renders when asked, so tests can hear what the voices mix without an audio device.
//! The speakers ([`crate::SpeakerSink`]) apply the same commands and render in their callback.

use std::sync::{Arc, Mutex};

use crate::mixer::Mixer;
use crate::sink::{SinkCommand, SoundSink};

/// A mixer kept in memory: clones share it.
#[derive(Clone)]
pub struct MixingSink {
    mixer: Arc<Mutex<Mixer>>,
}

impl MixingSink {
    /// A mixer writing `rate` frames per second.
    pub fn new(rate: u32) -> Self {
        Self {
            mixer: Arc::new(Mutex::new(Mixer::new(rate))),
        }
    }

    /// `frames` frames of the mix (`channels` per frame, interleaved), rendered now.
    pub fn render(&self, frames: usize, channels: usize) -> Vec<f32> {
        let channels = channels.max(1);
        let mut out = vec![0.0; frames * channels];
        if let Ok(mut mixer) = self.mixer.lock() {
            mixer.render(&mut out, channels);
        }
        out
    }

    /// Reads the mixer's state.
    pub fn with_mixer<T>(&self, read: impl FnOnce(&Mixer) -> T) -> Option<T> {
        self.mixer.lock().ok().map(|mixer| read(&mixer))
    }
}

impl SoundSink for MixingSink {
    fn send(&mut self, command: SinkCommand) {
        if let Ok(mut mixer) = self.mixer.lock() {
            mixer.apply(command);
        }
    }
}
