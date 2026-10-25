//! The music queue: retail's `DXMusicMessageThread` (`portal/client/src/sound/directx`). It plays
//! the DirectMusic segments the sound manager asks for: one primary segment (the music) at a time,
//! where a primary asked for while one plays waits and takes over at that segment's end, and
//! secondary segments that play alongside. Each segment is a [`Playback`] of its sequence and
//! style performances; their events go to the caller's [`EventSink`], stamped with the segment's
//! sound handle (the synthesizer plays them, see `crate::synth`).
//!
//! DirectMusic, as the decompile calls it (`IDirectMusicPerformance8` vtable offsets, with the
//! argument lists that identify each method):
//!
//! - `PlayInternal`, `+0xB4`: `PlaySegmentEx(segment, NULL, NULL, dwFlags, 0, &state, pAfter,
//!   audioPath)`. [`MusicThread::play`] starts the segment in the queue's time; the timing flags
//!   and the audio path are not modelled (a secondary segment starts at once).
//! - `Stop`, `+0xB8`: `StopEx(state, 0, dwStopFlags)` for a secondary; for the current primary
//!   retail only clears `m_bLoopPrimary` (`DXMusicMessageThread::Stop`, admin 0x005A9540), so its
//!   pass plays out to its end whatever the boundary. [`MusicThread::stop`] follows that. A
//!   secondary segment ends at once (the port's gap: retail's boundary flags are not modelled).
//!   A forced-immediate stop of the voice is the sound's own (`DirectXPlayingSound::StopInternal`
//!   calls `Stop(handle, 0)` and then ends the voice), not the queue's.
//! - `IsPlaying`, `+0x38`: [`MusicThread::is_playing`].
//! - `GetNotificationPMsg` / `FreePMsg`, `+0x54` / `+0x44`, read by `Startup`: the notifications a
//!   playback posts (start, almost end, end, abort) are read by [`MusicThread::pump`]. In the
//!   port they are read inside [`MusicThread::use_time`], not on a thread of their own.
//! - `SetGlobalParam(GUID_PerfMasterGrooveLevel, &level, 1)`, `+0x88`:
//!   [`MusicThread::set_groove_level`]. DirectMusic reads one byte, so the level is an `i8`.
//! - `SetNotificationHandle` / `AddNotificationType`, `+0x50` / `+0x58`: the notifications are on
//!   from the start; there is no event handle.
//!
//! The port's clock is the one [`MusicThread::use_time`] is given. Segment boundaries inside a
//! span are honoured: a primary that queued behind another starts at the instant the other ends.
//! So are band changes: the performance band is the last band segment started or band item
//! reached, and a band-less segment's notes map by it from that instant ([`BandChange`]).

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use crate::music::library::LoadedMusic;
use crate::music::performance::{EventSink, TimedEvent};
use crate::music::script::{BoundSegment, Instance};
use crate::music::style_performance::PatternChoice;
use crate::sink::Boundary;
use crate::synth::{ChannelSources, PerformanceBand};

#[cfg(test)]
mod band_tests;
#[cfg(test)]
mod boundary_tests;
#[cfg(test)]
mod dat_tests;
mod playback;
#[cfg(test)]
mod script_tests;
mod scripts;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod wave_tests;

use playback::Playback;
pub use scripts::SCRIPTED_HANDLE_BASE;

/// A seconds tolerance: a segment within it of its end has ended.
const EPS: f64 = 1e-9;

/// `DXMusicMessageThread::SegmentInfo::m_type`. A primary segment is the music: it loops until
/// stopped, and the next primary replaces it at its end.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentKind {
    Primary,
    Secondary,
}

/// A segment the sound manager asks the queue to play (`DXMusicMessageThread::SegmentInfo`): its
/// kind, the sound handle that names it, the DirectMusic segment with its closure, and its band
/// track's items as (seconds from the segment's start, band), in the track's order (none for a
/// segment with no band track; its notes then play on the performance's band).
#[derive(Debug, Clone)]
pub struct SegmentInfo {
    pub kind: SegmentKind,
    pub handle: u32,
    pub segment: Arc<LoadedMusic>,
    pub bands: Arc<[(f64, Arc<PerformanceBand>)]>,
    /// Its cues: the script calls of its script track and its segment triggers (none for a wave,
    /// or a segment with neither).
    pub cues: Arc<BoundSegment>,
}

/// A segment a script call or a segment trigger started: the sound manager gives it a voice, as it
/// does a music sound's, with the segment's channel sources and band.
#[derive(Debug, Clone)]
pub struct ScriptedPlay {
    pub kind: SegmentKind,
    pub handle: u32,
    pub sources: ChannelSources,
    pub bands: Arc<[(f64, Arc<PerformanceBand>)]>,
    pub segment: Arc<LoadedMusic>,
}

/// A voice's channel sources changing with the band its notes play on: a band-less segment when
/// the performance's band changes, or a segment whose band track reaches an item. The sound
/// manager sends it to the voice of `handle`.
#[derive(Debug, Clone)]
pub struct BandChange {
    pub handle: u32,
    pub sources: ChannelSources,
}

/// `DXMusicMessageThread::MusicEventType`: what UseTime hands the handler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MusicEventKind {
    SegmentStart,
    SegmentAlmostEnd,
    SegmentEnd,
    SegmentAbort,
}

/// A queued event: its kind, and the sound handle of the segment (0 when none is associated).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SegmentEvent {
    pub kind: MusicEventKind,
    pub handle: u32,
}

/// The callbacks UseTime makes (`DXMusicMessageHandler`: slot 0 `OnSegmentStart`, slot 1 the
/// almost-end, slot 2 `OnSegmentEnded`, which an abort also gets).
pub trait MusicHandler {
    fn on_segment_start(&mut self, handle: u32);
    fn on_segment_almost_end(&mut self, _handle: u32) {}
    fn on_segment_ended(&mut self, handle: u32);
}

/// The notifications a playback posts (`DMUS_NOTIFICATION_SEGSTART`, `SEGEND`, `SEGALMOST`), by
/// the segment state they are about. An abort is queued directly as an event instead: it is for
/// a segment that never played.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Notice {
    Start,
    End,
    AlmostEnd,
}

/// One entry of `m_aPlayingSounds`: a sound handle and the segment state playing for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Entry {
    handle: u32,
    state: u32,
}

/// `DXMusicMessageThread`: the queue of primary and secondary segments and their playbacks.
pub struct MusicThread {
    /// Whether a performance is attached (`SetHandler` with one; none after `shutdown`).
    attached: bool,
    /// The queue's time: the last `use_time`, or a segment boundary inside one.
    clock: f64,
    /// Whether `use_time` has set the clock's start.
    started: bool,
    /// The master groove level, as DirectMusic reads it (one byte).
    groove: i8,
    next_state: u32,
    playbacks: Vec<Playback>,
    /// `m_aPlayingSounds`: the handles playing, in the order they started.
    playing: Vec<Entry>,
    /// `m_pCurrentPrimary` and `m_segInfoCurrentPrimary`.
    current: Option<u32>,
    current_info: Option<SegmentInfo>,
    /// `m_bPrimaryHasStarted`, `m_bLoopPrimary`.
    primary_started: bool,
    loop_primary: bool,
    /// `m_segInfoNextPrimary`: a primary queued behind the current one (`m_bNextPrimaryQueued`).
    next_info: Option<SegmentInfo>,
    /// The primaries scripts queued behind the current one (`Segment.play` with `AtFinish`). They
    /// are the performance's own queue in retail, so they play in order before `next_info` and
    /// a request for a copy of the music never aborts them.
    scripted_primaries: VecDeque<SegmentInfo>,
    /// The notifications not yet read by `Startup` (the performance's queue).
    notices: VecDeque<(u32, Notice)>,
    /// `m_aEvents`: the events for UseTime to hand the handler.
    events: VecDeque<SegmentEvent>,
    /// The events the playbacks sent, not yet given to a sink.
    out: Vec<TimedEvent>,
    /// The performance's band: the band of the band segment that started last, or whose band
    /// track reached an item last (`DirectMusic`'s performance-wide band).
    performance: Option<Arc<PerformanceBand>>,
    /// The band changes not yet taken (see [`BandChange`]).
    changes: Vec<BandChange>,
    /// Each script's module variables, by the script's address (see `scripts`).
    instances: HashMap<usize, Instance>,
    /// The segments scripts started, not yet taken by the sound manager.
    scripted: Vec<ScriptedPlay>,
    /// The next handle for a scripted segment.
    next_scripted: u32,
}

impl Default for MusicThread {
    fn default() -> Self {
        Self::new()
    }
}

impl MusicThread {
    /// `DXMusicMessageThread::DXMusicMessageThread` and `SetHandler`: a queue with a performance
    /// attached and nothing playing. The groove starts at 0.
    pub fn new() -> Self {
        Self {
            attached: true,
            clock: 0.0,
            started: false,
            groove: 0,
            next_state: 1,
            playbacks: Vec::new(),
            playing: Vec::new(),
            current: None,
            current_info: None,
            primary_started: false,
            loop_primary: true,
            next_info: None,
            scripted_primaries: VecDeque::new(),
            notices: VecDeque::new(),
            events: VecDeque::new(),
            out: Vec::new(),
            performance: None,
            changes: Vec::new(),
            instances: HashMap::new(),
            scripted: Vec::new(),
            next_scripted: SCRIPTED_HANDLE_BASE,
        }
    }

    /// `DXMusicMessageThread::Play`: starts a secondary segment now, or queues a primary (it
    /// starts when the current primary ends). `false` when the segment cannot play (no handle,
    /// no performance, or a secondary whose object is not a segment).
    pub fn play(&mut self, info: SegmentInfo) -> bool {
        if info.handle == 0 || !self.attached {
            return false;
        }
        match info.kind {
            SegmentKind::Primary => {
                if let Some(state) = self.queue_primary(&info) {
                    self.associate(state, info.handle);
                }
                true
            }
            SegmentKind::Secondary => match self.play_internal(&info) {
                Some(state) => {
                    self.associate(state, info.handle);
                    true
                }
                None => false,
            },
        }
    }

    /// `DXMusicMessageThread::Stop`: stops the segment for `handle`. The current primary is not
    /// stopped: retail clears its loop (`m_bLoopPrimary = 0`) and returns, so its pass plays out
    /// to its end whatever the boundary (the stop's `dwStopFlags` are not read for it). A
    /// secondary segment is stopped now (`StopEx` with the boundary; the port's gap: retail's
    /// boundary-timed stop of a secondary is not modelled). A handle that is not playing (a
    /// primary waiting behind the current one, or none) gets an abort event.
    pub fn stop(&mut self, handle: u32, _boundary: Boundary) {
        if !self.attached {
            self.abort(handle);
            return;
        }
        let Some(index) = self.playing.iter().position(|e| e.handle == handle) else {
            if self
                .next_info
                .as_ref()
                .is_some_and(|next| next.handle == handle)
            {
                self.next_info = None;
            }
            self.scripted_primaries.retain(|next| next.handle != handle);
            self.abort(handle);
            return;
        };
        let state = self.playing[index].state;
        if self.current == Some(state) {
            self.loop_primary = false;
            if let Some(playback) = self.playbacks.iter_mut().find(|p| p.id() == state) {
                playback.end_pass_after_stop();
            }
            return;
        }
        self.end_playback(state);
    }

    /// Ends the playback `state` now (its notes release here), and posts its end.
    fn end_playback(&mut self, state: u32) {
        let stop = match self.playbacks.iter_mut().find(|p| p.id() == state) {
            Some(playback) if playback.ended() => false,
            Some(playback) => {
                playback.stop(&mut self.out);
                true
            }
            None => true,
        };
        if stop {
            self.notices.push_back((state, Notice::End));
        }
    }

    /// Queues an abort for `handle` (`DXMusicMessageThread::Stop`'s `SegmentAbort` events).
    fn abort(&mut self, handle: u32) {
        self.events.push_back(SegmentEvent {
            kind: MusicEventKind::SegmentAbort,
            handle,
        });
    }

    /// Whether the queue holds `handle`: its segment is in the playing list, or waits behind the
    /// current primary.
    pub fn holds(&self, handle: u32) -> bool {
        self.attached
            && (self.playing.iter().any(|e| e.handle == handle)
                || self
                    .next_info
                    .as_ref()
                    .is_some_and(|next| next.handle == handle)
                || self
                    .scripted_primaries
                    .iter()
                    .any(|next| next.handle == handle))
    }

    /// `DXMusicMessageThread::IsPlaying`: the segment for `handle` is playing (not ended).
    pub fn is_playing(&self, handle: u32) -> bool {
        self.attached
            && self
                .playing
                .iter()
                .find(|e| e.handle == handle)
                .is_some_and(|e| self.is_state_playing(e.state))
    }

    /// `DirectXSoundDevice::SetGrooveLevel` (`SoundManager::SetGrooveLevel` passes the level plus
    /// its offset): the master groove of every style, and of the styles started later. The
    /// style's next measure chooses its pattern at the new level.
    pub fn set_groove_level(&mut self, level: i32) {
        self.groove = level as i8;
        let groove = self.groove;
        for playback in &mut self.playbacks {
            playback.set_groove(groove);
        }
    }

    /// The master groove level DirectMusic reads (the low byte of the level).
    pub fn groove_level(&self) -> i8 {
        self.groove
    }

    /// The patterns the style of the segment for `handle` has chosen, in measure order.
    pub fn choices(&self, handle: u32) -> Option<&[PatternChoice]> {
        let state = self.playing.iter().find(|e| e.handle == handle)?.state;
        self.playbacks.iter().find(|p| p.id() == state)?.choices()
    }

    /// `DXMusicMessageThread::UseTime` with the `Startup` loop folded in: the playing segments
    /// run for the time since the last call, a segment starting or ending inside it is handled
    /// at its instant, and the events the segments sent go to the buffer ([`Self::send_events`]).
    /// A band track's item reached inside the span changes the band at its instant
    /// ([`Self::take_band_changes`]). The queued events (start, almost end, end, abort) are then
    /// handed to `handler` in order; without a handler they wait for the next call that has one.
    pub fn use_time(&mut self, now: f64, handler: Option<&mut dyn MusicHandler>) {
        if !self.started {
            self.started = true;
            self.clock = now;
        }
        let mut remaining = (now - self.clock).max(0.0);
        self.pump();
        loop {
            self.apply_due_bands();
            self.run_due_cues();
            let to_end = self
                .playbacks
                .iter()
                .filter(|p| !p.ended())
                .map(|p| p.remaining().min(p.next_band_in()).min(p.next_cue_in()))
                .fold(f64::INFINITY, f64::min);
            let step = remaining.min(to_end);
            for playback in self.playbacks.iter_mut().filter(|p| !p.ended()) {
                playback.advance(step, &mut self.out);
            }
            self.clock += step;
            remaining -= step;
            let mut ended = false;
            for playback in self
                .playbacks
                .iter_mut()
                .filter(|p| !p.ended() && p.remaining() <= EPS)
            {
                playback.finish(&mut self.out);
                self.notices.push_back((playback.id(), Notice::AlmostEnd));
                self.notices.push_back((playback.id(), Notice::End));
                ended = true;
            }
            self.pump();
            if remaining <= 0.0 || (step <= 0.0 && !ended) {
                break;
            }
        }
        self.clock = now;
        let Some(handler) = handler else {
            return;
        };
        while let Some(event) = self.events.pop_front() {
            match event.kind {
                MusicEventKind::SegmentStart => handler.on_segment_start(event.handle),
                MusicEventKind::SegmentAlmostEnd => handler.on_segment_almost_end(event.handle),
                MusicEventKind::SegmentEnd | MusicEventKind::SegmentAbort => {
                    handler.on_segment_ended(event.handle);
                    self.remove_entry(event.handle);
                }
            }
        }
    }

    /// Gives the events the playbacks sent since the last call to `sink`, in order.
    pub fn send_events(&mut self, sink: &mut dyn EventSink) {
        for event in self.out.drain(..) {
            sink.event(&event);
        }
    }

    /// `DXMusicMessageThread::~DXMusicMessageThread` (and `SetHandler(NULL)`): every segment is
    /// dropped, the queue detaches, and later plays fail.
    pub fn shutdown(&mut self) {
        self.attached = false;
        self.playbacks.clear();
        self.playing.clear();
        self.current = None;
        self.current_info = None;
        self.next_info = None;
        self.scripted_primaries.clear();
        self.primary_started = false;
        self.loop_primary = true;
        self.notices.clear();
        self.events.clear();
        self.out.clear();
        self.performance = None;
        self.changes.clear();
        self.instances.clear();
        self.scripted.clear();
    }

    /// The segments scripts started since the last call (see [`ScriptedPlay`]).
    pub fn take_scripted(&mut self) -> Vec<ScriptedPlay> {
        std::mem::take(&mut self.scripted)
    }

    /// `DXMusicMessageThread::PlayInternal`: starts `info`'s segment now as a new playback and
    /// posts its start notification. Retail passes the segment it follows (`pAfter`); a primary
    /// is only started here at the end of that segment, so the start time is the boundary.
    fn play_internal(&mut self, info: &SegmentInfo) -> Option<u32> {
        if !self.attached {
            return None;
        }
        let id = self.next_state;
        self.next_state = self.next_state.wrapping_add(1).max(1);
        let mut playback = Playback::new(
            id,
            info.handle,
            self.clock,
            &info.segment,
            self.groove,
            id,
            &info.bands,
            self.performance.as_ref(),
        )?;
        playback.schedule_cues(&info.cues);
        let own = playback.own_band();
        self.playbacks.push(playback);
        if own {
            // A segment with a band track makes its first band the performance's.
            if let Some((_, first)) = info.bands.first() {
                self.set_performance(Arc::clone(first));
            }
        }
        self.notices.push_back((id, Notice::Start));
        Some(id)
    }

    /// The performance's band changes to `band`: the segments whose notes play on it (those with
    /// no band track of their own) map their notes by it from now on, and their voices take its
    /// sources (a [`BandChange`]).
    fn set_performance(&mut self, band: Arc<PerformanceBand>) {
        for playback in self
            .playbacks
            .iter_mut()
            .filter(|p| !p.own_band() && !p.ended())
        {
            playback.set_band(Some(&band.band));
            self.changes.push(BandChange {
                handle: playback.owner(),
                sources: band.sources.clone(),
            });
        }
        self.performance = Some(band);
    }

    /// The band items reached by now: each segment's own band changes (its notes and its voice),
    /// and the performance's band with it.
    fn apply_due_bands(&mut self) {
        for index in 0..self.playbacks.len() {
            if self.playbacks[index].ended() {
                continue;
            }
            for band in self.playbacks[index].take_due_bands() {
                let playback = &mut self.playbacks[index];
                playback.set_band(Some(&band.band));
                self.changes.push(BandChange {
                    handle: playback.owner(),
                    sources: band.sources.clone(),
                });
                self.set_performance(band);
            }
        }
    }

    /// The performance's band (none until a segment with a band track plays).
    pub fn performance(&self) -> Option<Arc<PerformanceBand>> {
        self.performance.clone()
    }

    /// The band changes since the last call, in the order they happened.
    pub fn take_band_changes(&mut self) -> Vec<BandChange> {
        std::mem::take(&mut self.changes)
    }

    /// `DXMusicMessageThread::QueuePrimaryInternal`: with no primary playing, starts `info` as
    /// the primary; otherwise it replaces the primary waiting behind the current one (that one
    /// gets an abort) and waits for the current primary's end.
    fn queue_primary(&mut self, info: &SegmentInfo) -> Option<u32> {
        if self.current.is_some() {
            if let Some(waiting) = self.next_info.take() {
                self.events.push_back(SegmentEvent {
                    kind: MusicEventKind::SegmentAbort,
                    handle: waiting.handle,
                });
            }
            self.next_info = Some(info.clone());
            return None;
        }
        self.current_info = None;
        self.primary_started = false;
        self.loop_primary = true;
        let state = self.play_internal(info)?;
        self.current = Some(state);
        self.current_info = Some(info.clone());
        Some(state)
    }

    /// `DXMusicMessageThread::OnMusicStopping`: the current primary's end (or its abort). A
    /// queued primary takes over; otherwise a looping primary plays again and a finished one
    /// is released.
    fn on_music_stopping(&mut self) {
        if let Some(next) = self.scripted_primaries.pop_front() {
            self.start_queued_primary(next);
            return;
        }
        if self.next_info.is_some() {
            self.play_queued_primary();
            return;
        }
        let Some(current) = self.current else {
            return;
        };
        let replay = match self.current_info.clone() {
            Some(info) if self.loop_primary && self.length_of(current) > EPS => Some(info),
            _ => None,
        };
        match replay {
            Some(info) => {
                let state = self.play_internal(&info);
                self.current = state;
                self.primary_started = false;
                if let Some(state) = state {
                    self.associate(state, info.handle);
                }
            }
            None => {
                self.current = None;
                self.current_info = None;
            }
        }
    }

    /// `DXMusicMessageThread::PlayQueuedPrimarySegment`: the queued primary starts; it becomes
    /// the current primary, and the old one is released (it plays out its own notifications).
    fn play_queued_primary(&mut self) {
        let Some(next) = self.next_info.take() else {
            return;
        };
        self.start_queued_primary(next);
    }

    /// Starts `next` as the current primary (it was queued behind the one that just ended).
    fn start_queued_primary(&mut self, next: SegmentInfo) {
        let handle = next.handle;
        let state = self.play_internal(&next);
        self.current = state;
        self.current_info = state.map(|_| next);
        self.primary_started = false;
        self.loop_primary = true;
        if let Some(state) = state {
            self.associate(state, handle);
        }
    }

    /// A primary a script started (`Segment.play` with `AtFinish`): it starts now when no primary
    /// plays, else it waits in the performance's queue behind the current one.
    pub(super) fn queue_scripted_primary(&mut self, info: SegmentInfo) -> bool {
        if !self.attached || info.handle == 0 {
            return false;
        }
        if self.current.is_none() {
            return self.play(info);
        }
        self.scripted_primaries.push_back(info);
        true
    }

    /// `DXMusicMessageThread::AssociateSoundHandle`: `handle` plays `state`. A state replacing
    /// the one a handle had (a looping primary's pass) drops the old playback.
    fn associate(&mut self, state: u32, handle: u32) {
        match self.playing.iter_mut().find(|e| e.handle == handle) {
            Some(entry) => {
                let old = std::mem::replace(&mut entry.state, state);
                if self.current != Some(old) {
                    self.playbacks.retain(|p| p.id() != old);
                }
            }
            None => self.playing.push(Entry { handle, state }),
        }
    }

    /// `DXMusicMessageThread::GetAssociatedSoundHandle`: the handle a segment state plays for, or
    /// 0 once its handle has been replaced.
    fn handle_of(&self, state: u32) -> u32 {
        self.playing
            .iter()
            .find(|e| e.state == state)
            .map_or(0, |e| e.handle)
    }

    /// `DXMusicMessageThread::Startup`'s notification switch: each notification becomes an
    /// event, and an end or abort (or almost end) of the current primary runs
    /// [`Self::on_music_stopping`]. Last, the `WAIT_TIMEOUT` check: a started primary that no
    /// longer plays runs it too.
    fn pump(&mut self) {
        while let Some((state, notice)) = self.notices.pop_front() {
            let handle = self.handle_of(state);
            let kind = match notice {
                Notice::Start => {
                    if self.current == Some(state) {
                        self.primary_started = true;
                    }
                    MusicEventKind::SegmentStart
                }
                Notice::AlmostEnd => MusicEventKind::SegmentAlmostEnd,
                Notice::End => MusicEventKind::SegmentEnd,
            };
            self.events.push_back(SegmentEvent { kind, handle });
            if notice != Notice::Start && self.current == Some(state) {
                self.on_music_stopping();
            }
        }
        if let Some(current) = self.current {
            if self.primary_started && !self.is_state_playing(current) {
                self.on_music_stopping();
            }
        }
    }

    /// The removal `UseTime` makes for an end or abort (`m_aPlayingSounds` swap-remove): the
    /// segment for `handle` leaves the playing list, and its playback is dropped unless it is
    /// the current primary.
    fn remove_entry(&mut self, handle: u32) {
        let Some(index) = self.playing.iter().position(|e| e.handle == handle) else {
            return;
        };
        let state = self.playing.swap_remove(index).state;
        if self.current != Some(state) {
            self.playbacks.retain(|p| p.id() != state);
        }
    }

    fn is_state_playing(&self, state: u32) -> bool {
        self.playbacks
            .iter()
            .find(|p| p.id() == state)
            .is_some_and(|p| !p.ended())
    }

    fn length_of(&self, state: u32) -> f64 {
        self.playbacks
            .iter()
            .find(|p| p.id() == state)
            .map_or(0.0, |p| p.length())
    }
}
