//! Binding a segment's cues on the loader thread (`DirectXMusicLoader::LoadFromMusicInfo` loads the
//! segment's closure, scripts included). A segment has two kinds of cue: its script track's calls
//! (a routine runs at a time) and its segment trigger track's items (a segment starts as a
//! secondary at a time, `DMUS_SEGF_SECONDARY`). Each cue is bound here: a call's script is parsed
//! once, and each segment a script's container holds, or a trigger names, carries its closure,
//! its band schedule, its channel sources and its own cues (bound the same way, to a depth limit).
//! The queue then starts them with no file access and no parsing (`music_thread::scripts`).

use std::collections::HashMap;
use std::sync::Arc;

use super::parse;
use super::parser::Program;
use crate::music::clock::MusicClock;
use crate::music::container::Container;
use crate::music::library::LoadedMusic;
use crate::music::segment::{Segment, TrackData};
use crate::music::MusicObject;
use crate::synth::{performance_bands, style_sources, ChannelSources, PerformanceBand};

/// Segments nested in segments (a trigger's child with its own triggers) bound this deep.
const MAX_DEPTH: usize = 8;

/// A segment that a script or a trigger starts: its closure, its band schedule, the channel
/// sources of its style, and its own cues.
#[derive(Debug, Clone, PartialEq)]
pub struct BoundTarget {
    pub music: Arc<LoadedMusic>,
    pub bands: Arc<[(f64, Arc<PerformanceBand>)]>,
    pub sources: ChannelSources,
    pub cues: Arc<BoundSegment>,
}

/// A script with its parsed program and the segments its container holds (keyed by lowercase
/// alias).
#[derive(Debug, PartialEq)]
pub struct BoundScript {
    pub program: Program,
    pub targets: HashMap<String, BoundTarget>,
}

/// A call the script track makes: at `seconds` from the segment's start, run `routine` of
/// `script`.
#[derive(Debug, Clone, PartialEq)]
pub struct BoundCall {
    pub seconds: f64,
    pub script: Arc<BoundScript>,
    pub routine: String,
}

/// A segment trigger: at `seconds` from the segment's start, `target` starts as a secondary.
#[derive(Debug, Clone, PartialEq)]
pub struct BoundTrigger {
    pub seconds: f64,
    pub target: BoundTarget,
}

/// The cues of one segment, in track order.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BoundSegment {
    pub calls: Vec<BoundCall>,
    pub triggers: Vec<BoundTrigger>,
}

/// The cues of a loaded segment (empty for a wave or an object that is not a segment).
pub fn bind_segment(music: &LoadedMusic) -> Arc<BoundSegment> {
    let mut scripts = HashMap::new();
    Arc::new(bind_at(music, 0, &mut scripts))
}

/// Cached parsed scripts, by the script's address in the closure.
type ScriptCache = HashMap<usize, Option<Arc<BoundScript>>>;

fn bind_at(music: &LoadedMusic, depth: usize, scripts: &mut ScriptCache) -> BoundSegment {
    let MusicObject::Segment(segment) = &music.segment else {
        return BoundSegment::default();
    };
    if depth > MAX_DEPTH {
        return BoundSegment::default();
    }
    let clock = MusicClock::from_segment(segment);
    BoundSegment {
        calls: calls(music, segment, &clock, depth, scripts),
        triggers: triggers(music, segment, &clock, depth, scripts),
    }
}

fn calls(
    music: &LoadedMusic,
    segment: &Segment,
    clock: &MusicClock,
    depth: usize,
    scripts: &mut ScriptCache,
) -> Vec<BoundCall> {
    let mut out = Vec::new();
    for track in &segment.tracks {
        let TrackData::Scripts(list) = &track.data else {
            continue;
        };
        for call in list {
            let Some(MusicObject::Script(script)) =
                call.script.as_ref().and_then(|r| music.resolve(r))
            else {
                log::warn!("script call {}: its script is missing", call.routine);
                continue;
            };
            let key = Arc::as_ptr(script) as usize;
            let bound = match scripts.get(&key) {
                Some(cached) => cached.clone(),
                None => {
                    let built = match parse(&script.source) {
                        Ok(program) => Some(Arc::new(BoundScript {
                            targets: targets(music, script.container.as_ref(), depth, scripts),
                            program,
                        })),
                        Err(error) => {
                            log::warn!("script {:?}: {error}", script.name);
                            None
                        }
                    };
                    scripts.insert(key, built.clone());
                    built
                }
            };
            let Some(bound) = bound else {
                continue;
            };
            if bound.program.routine(&call.routine).is_none() {
                log::warn!("script {:?} has no routine {}", script.name, call.routine);
                continue;
            }
            out.push(BoundCall {
                seconds: clock.seconds_at(call.time),
                script: bound,
                routine: call.routine.clone(),
            });
        }
    }
    out
}

fn triggers(
    music: &LoadedMusic,
    segment: &Segment,
    clock: &MusicClock,
    depth: usize,
    scripts: &mut ScriptCache,
) -> Vec<BoundTrigger> {
    let mut out = Vec::new();
    for track in &segment.tracks {
        let TrackData::Triggers(list) = &track.data else {
            continue;
        };
        for trigger in list {
            let Some(child) = trigger.segment.as_ref().and_then(|r| music.resolve(r)) else {
                log::warn!(
                    "segment trigger at {}: its segment is missing",
                    trigger.time
                );
                continue;
            };
            let MusicObject::Segment(_) = child else {
                continue;
            };
            let target = target(music, child.clone(), depth, scripts);
            out.push(BoundTrigger {
                seconds: clock.seconds_at(trigger.time),
                target,
            });
        }
    }
    out
}

/// A target: the object with the closure of `music` (its objects are the ones the segment
/// reaches), its bands, its style's channel sources and its own cues.
fn target(
    music: &LoadedMusic,
    object: MusicObject,
    depth: usize,
    scripts: &mut ScriptCache,
) -> BoundTarget {
    let bound = LoadedMusic {
        segment: object,
        objects: music.objects.clone(),
        missing: 0,
    };
    BoundTarget {
        bands: performance_bands(&bound).into(),
        sources: style_sources(&bound).into(),
        cues: Arc::new(bind_at(&bound, depth + 1, scripts)),
        music: Arc::new(bound),
    }
}

/// The segments a script's container holds, each as a target.
fn targets(
    music: &LoadedMusic,
    container: Option<&Container>,
    depth: usize,
    scripts: &mut ScriptCache,
) -> HashMap<String, BoundTarget> {
    let Some(container) = container else {
        return HashMap::new();
    };
    container
        .objects
        .iter()
        .filter_map(|item| {
            let object = match (&item.object, &item.reference) {
                (Some(object), _) => object.clone(),
                (None, Some(reference)) => music.resolve(reference)?.clone(),
                (None, None) => return None,
            };
            let MusicObject::Segment(_) = object else {
                return None;
            };
            Some((
                item.alias.to_ascii_lowercase(),
                target(music, object, depth, scripts),
            ))
        })
        .collect()
}
