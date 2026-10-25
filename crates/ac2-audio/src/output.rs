//! The speakers: the system's default output through cpal (WASAPI on Windows, ALSA on Linux,
//! CoreAudio on macOS), fed by the [`Mixer`]. cpal is the thin cross-platform layer; retail's
//! own mixing rules (channel volumes, DirectSound attenuation, per-voice rate, loops) stay in
//! this crate rather than in a playback library's mixer.
//!
//! The stream lives on its own thread (some backends' streams cannot leave the thread that
//! made them); the frame thread only sends commands.

use std::sync::mpsc::{channel, sync_channel, Receiver, Sender};
use std::thread::JoinHandle;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SampleFormat, SizedSample, StreamConfig};

use crate::mixer::Mixer;
use crate::sink::{SinkCommand, SoundSink};

/// The default output device as a sink.
pub struct SpeakerSink {
    commands: Sender<SinkCommand>,
    stop: Option<Sender<()>>,
    thread: Option<JoinHandle<()>>,
}

impl SpeakerSink {
    /// Opens the default output device; an error when there is none or it will not play.
    pub fn open() -> Result<Self, String> {
        let (commands, receive) = channel();
        let (stop, stopped) = channel::<()>();
        let (ready, opened) = sync_channel(1);
        let thread = std::thread::Builder::new()
            .name("ac2-audio-output".into())
            .spawn(move || match start(receive) {
                Ok(stream) => {
                    let _ = ready.send(Ok(()));
                    let _ = stopped.recv();
                    drop(stream);
                }
                Err(error) => {
                    let _ = ready.send(Err(error));
                }
            })
            .map_err(|e| e.to_string())?;
        opened.recv().map_err(|e| e.to_string()).and_then(|r| r)?;
        Ok(Self {
            commands,
            stop: Some(stop),
            thread: Some(thread),
        })
    }
}

impl SoundSink for SpeakerSink {
    fn send(&mut self, command: SinkCommand) {
        let _ = self.commands.send(command);
    }
}

impl Drop for SpeakerSink {
    fn drop(&mut self) {
        self.stop = None;
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn start(commands: Receiver<SinkCommand>) -> Result<cpal::Stream, String> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or("no audio output device")?;
    let supported = device.default_output_config().map_err(|e| e.to_string())?;
    let format = supported.sample_format();
    let config = supported.config();
    let stream = match format {
        SampleFormat::F32 => build::<f32>(&device, &config, commands),
        SampleFormat::I16 => build::<i16>(&device, &config, commands),
        SampleFormat::U16 => build::<u16>(&device, &config, commands),
        SampleFormat::I32 => build::<i32>(&device, &config, commands),
        other => Err(format!("unsupported output sample format {other:?}")),
    }?;
    stream.play().map_err(|e| e.to_string())?;
    Ok(stream)
}

fn build<T>(
    device: &cpal::Device,
    config: &StreamConfig,
    commands: Receiver<SinkCommand>,
) -> Result<cpal::Stream, String>
where
    T: SizedSample + FromSample<f32>,
{
    let channels = usize::from(config.channels);
    let mut mixer = Mixer::new(config.sample_rate);
    let mut scratch: Vec<f32> = Vec::new();
    device
        .build_output_stream(
            config,
            move |data: &mut [T], _| {
                while let Ok(command) = commands.try_recv() {
                    mixer.apply(command);
                }
                scratch.resize(data.len(), 0.0);
                mixer.render(&mut scratch, channels);
                for (out, sample) in data.iter_mut().zip(&scratch) {
                    *out = T::from_sample(*sample);
                }
            },
            |error| log::warn!("audio output: {error}"),
            None,
        )
        .map_err(|e| e.to_string())
}
