//! The music queue's callbacks into the sound manager: the queue's events go to the sink as notes
//! (`NoteRouter`), and its segment starts and ends reach the playing sounds (`MusicCallbacks`,
//! `DXMusicMessageHandler` for `DirectXSoundDevice::OnSegmentStart` and `OnSegmentEnded`).

use crate::music::performance::{EventSink, TimedEvent};
use crate::music_thread::MusicHandler;
use crate::playing::{DeviceState, PlayingSound};
use crate::sink::{SinkCommand, SoundSink};

/// Sends the music queue's events to the sink as notes, the queue's clock first.
pub(super) struct NoteRouter<'a> {
    pub(super) sink: &'a mut dyn SoundSink,
    pub(super) now: f64,
    pub(super) clock_sent: bool,
}

impl EventSink for NoteRouter<'_> {
    fn event(&mut self, event: &TimedEvent) {
        if !self.clock_sent {
            self.sink
                .send(SinkCommand::MusicClock { seconds: self.now });
            self.clock_sent = true;
        }
        self.sink.send(SinkCommand::Note(*event));
    }
}

/// `DXMusicMessageHandler` for the sound manager (`DirectXSoundDevice::OnSegmentStart` and
/// `OnSegmentEnded`): a segment's sound hears its start and end. The start has no effect on a
/// music voice (`DirectXPlayingSound::OnSegmentStarted` sets a flag the admin decompile never
/// reads); a wave segment's voice starts with its pass.
pub(super) struct MusicCallbacks<'a> {
    pub(super) playing: &'a mut [PlayingSound],
    pub(super) sink: &'a mut dyn SoundSink,
    pub(super) device: &'a DeviceState,
}

impl MusicHandler for MusicCallbacks<'_> {
    fn on_segment_start(&mut self, handle: u32) {
        if handle == 0 {
            return;
        }
        if let Some(playing) = self
            .playing
            .iter_mut()
            .find(|p| p.handle == handle && p.segment_kind().is_some())
        {
            playing.on_segment_started(self.sink, self.device);
        }
    }

    fn on_segment_ended(&mut self, handle: u32) {
        // `DirectXSoundDevice::OnSegmentEnded`: a handle of 0 names no sound.
        if handle == 0 {
            return;
        }
        if let Some(playing) = self
            .playing
            .iter_mut()
            .find(|p| p.handle == handle && p.segment_kind().is_some())
        {
            playing.on_segment_ended(self.sink);
        }
    }
}
