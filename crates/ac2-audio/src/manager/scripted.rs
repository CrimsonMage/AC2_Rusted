//! The segments scripts start (`Segment.play`): the queue plays them, and each gets a playing
//! sound here so that it has a music voice and its notes route by its handle, as a `MusicInfo`'s
//! segment does (`DirectXPlayingSound`). A scripted segment has no `MusicInfo`, so its terms are
//! the defaults: a music sound, not looping (the queue repeats a primary), on the primary or
//! secondary track its play asked for, at full volume.

use crate::data::MusicInfo;
use crate::music_thread::SegmentKind;
use crate::play_data::{Channel, LoopingMode, PlaySoundData, SoundType, Track};
use crate::playing::PlayingSound;

use super::SoundManager;

impl SoundManager {
    /// Gives each segment a script started since the last call its playing sound, at `now`.
    pub(super) fn start_scripted(&mut self, now: f64) {
        for play in self.music.take_scripted() {
            let info = MusicInfo {
                did: 0,
                bit_field: 0,
                priority: 0.0,
                min_distance: 0.0,
                kind: 0,
                segment: 0,
            };
            let data = PlaySoundData {
                sound: 0,
                emitter: None,
                volume: 1.0,
                frequency: 1.0,
                looping: LoopingMode::ForceOff,
                kind: SoundType::ForceNonExclusive,
                track: match play.kind {
                    SegmentKind::Primary => Track::ForcePrimary,
                    SegmentKind::Secondary => Track::ForceSecondary,
                },
                channel: Channel::Music,
            };
            let band = self.band_part(play.bands.first().map(|(_, own)| own));
            let playing =
                PlayingSound::from_music(play.handle, &data, &info, play.sources, band, now);
            self.playing.push(playing);
        }
    }
}
