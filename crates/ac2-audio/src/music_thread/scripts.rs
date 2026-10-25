//! The scripts of the queue. A segment's script track calls its routine at the call's time on the
//! queue's clock ([`MusicThread::run_due_calls`]); the routine runs here, so it runs where the
//! queue runs and never reads a file. The queue is the script's host: `GetMasterGrooveLevel()` is
//! the queue's groove, and `<object>.play(flags)` starts the object's segment on the queue, as a
//! primary (`DMUS_SEGF_QUEUE`, behind the current primary) or a secondary
//! (`DMUS_SEGF_SECONDARY` / `CONTROL`). Each script keeps its module variables for this
//! performance. The semantics and their sources are in [`crate::music::script`].

use std::sync::Arc;

use super::playback::Cue;
use super::{MusicThread, ScriptedPlay, SegmentInfo, SegmentKind};
use crate::music::script::{BoundCall, BoundScript, BoundTrigger, ScriptHost};

/// `DMUS_SEGF_QUEUE` (256): a primary segment at the end of the primary queue.
const SEGF_QUEUE: u32 = 256;
/// `DMUS_SEGF_SECONDARY` (128) and `DMUS_SEGF_CONTROL` (512, a secondary control segment).
const SEGF_SECONDARY: u32 = 128;
const SEGF_CONTROL: u32 = 512;

/// The first handle a scripted segment gets. The sound manager counts its handles up from 1, so
/// these never collide with them.
pub const SCRIPTED_HANDLE_BASE: u32 = 0x8000_0000;

impl MusicThread {
    /// Runs every cue that is due on the playing segments (their time has come): a script call
    /// runs its routine, a trigger starts its segment as a secondary.
    pub(super) fn run_due_cues(&mut self) {
        let mut due = Vec::new();
        for playback in self.playbacks.iter_mut().filter(|p| !p.ended()) {
            due.extend(playback.take_due_cues());
        }
        for cue in due {
            match cue {
                Cue::Call(call) => self.run_call(&call),
                Cue::Trigger(trigger) => self.start_trigger(&trigger),
            }
        }
    }

    /// A segment trigger (`DMUS_IO_SEGMENT_ITEM`, played as `DMUS_SEGF_SECONDARY`): its segment
    /// starts now as a secondary, with its own cues.
    fn start_trigger(&mut self, trigger: &BoundTrigger) {
        let target = &trigger.target;
        let handle = self.next_scripted_handle();
        let info = SegmentInfo {
            kind: SegmentKind::Secondary,
            handle,
            segment: Arc::clone(&target.music),
            bands: Arc::clone(&target.bands),
            cues: Arc::clone(&target.cues),
        };
        if self.play(info) {
            self.scripted.push(ScriptedPlay {
                kind: SegmentKind::Secondary,
                handle,
                sources: Arc::clone(&target.sources),
                bands: Arc::clone(&target.bands),
                segment: Arc::clone(&target.music),
            });
        }
    }

    /// Runs one call: the routine of the call's script, with that script's variables for this
    /// performance. An error is logged; the rest of the queue carries on.
    fn run_call(&mut self, call: &BoundCall) {
        let key = Arc::as_ptr(&call.script) as usize;
        let mut instance = self.instances.remove(&key).unwrap_or_default();
        let result = {
            let mut host = QueueHost {
                queue: self,
                script: &call.script,
            };
            instance.call(&call.script.program, &call.routine, &mut host)
        };
        if let Err(error) = result {
            log::warn!("script routine {}: {error}", call.routine);
        }
        self.instances.insert(key, instance);
    }

    /// The handle for the next scripted segment.
    fn next_scripted_handle(&mut self) -> u32 {
        let handle = self.next_scripted;
        self.next_scripted = self.next_scripted.wrapping_add(1).max(SCRIPTED_HANDLE_BASE);
        handle
    }
}

/// The queue as a script's host, for one call.
struct QueueHost<'a> {
    queue: &'a mut MusicThread,
    script: &'a BoundScript,
}

impl ScriptHost for QueueHost<'_> {
    fn constant(&self, name: &str) -> Option<f64> {
        // `AtFinish` is `DMUS_SEGF_QUEUE` (see `crate::music::script`): after the current primary.
        name.eq_ignore_ascii_case("atfinish")
            .then_some(f64::from(SEGF_QUEUE))
    }

    fn function(&mut self, name: &str, args: &[f64]) -> Result<f64, String> {
        if !args.is_empty() {
            return Err(format!("{name} takes no arguments"));
        }
        if name.eq_ignore_ascii_case("getmastergroovelevel") {
            return Ok(f64::from(self.queue.groove));
        }
        Err(format!("{name} is not a built-in of the subset"))
    }

    fn method(&mut self, object: &str, method: &str, args: &[f64]) -> Result<(), String> {
        if !method.eq_ignore_ascii_case("play") {
            return Err(format!("{object}.{method} is not in the subset"));
        }
        let [flags] = args else {
            return Err("play takes one flags argument".into());
        };
        if flags.fract() != 0.0 || *flags < 0.0 || *flags > f64::from(u32::MAX) {
            return Err(format!("play flags {flags} are not DMUS_SEGF_* bits"));
        }
        let flags = *flags as u32;
        let target = self
            .script
            .targets
            .get(&object.to_ascii_lowercase())
            .ok_or_else(|| format!("{object} is not a segment in the script's container"))?;
        let kind = if flags & (SEGF_SECONDARY | SEGF_CONTROL) != 0 {
            SegmentKind::Secondary
        } else if flags & SEGF_QUEUE != 0 {
            SegmentKind::Primary
        } else {
            return Err(format!(
                "play of a primary without DMUS_SEGF_QUEUE (flags {flags}) is not in the subset"
            ));
        };
        let handle = self.queue.next_scripted_handle();
        let info = SegmentInfo {
            kind,
            handle,
            segment: Arc::clone(&target.music),
            bands: Arc::clone(&target.bands),
            cues: Arc::clone(&target.cues),
        };
        let started = match kind {
            SegmentKind::Primary => self.queue.queue_scripted_primary(info),
            SegmentKind::Secondary => self.queue.play(info),
        };
        if !started {
            return Err(format!("{object} could not play"));
        }
        self.queue.scripted.push(ScriptedPlay {
            kind,
            handle,
            sources: Arc::clone(&target.sources),
            bands: Arc::clone(&target.bands),
            segment: Arc::clone(&target.music),
        });
        Ok(())
    }
}
