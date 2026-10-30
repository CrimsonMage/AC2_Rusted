//! A creature standing idle the way an ACE server and the AC1 client show it: the motion table's
//! default cycle (`CMotionTable::SetDefaultState`) looping, and every heartbeat
//! (`PropertyFloat.HeartbeatInterval`) a roll against the weenie's HeartBeat emote sets for its
//! stance (`EmoteManager.OnHeartBeat`, `GetEmoteSet`): the set it picks plays its Motion actions,
//! each the motion table's link from the idle substate into that motion
//! (`CMotionTable::GetObjectSequence` for an action: `get_link`, then the cycle again), one after
//! another with its delay, and the cycle resumes after the last.

use std::collections::{HashMap, VecDeque};

use ac2_physics::PartPose;

use crate::dats::Ac1Dats;
use crate::formats::{Ac1Result, AnimData, Animation, Frame, MotionTable};
use crate::weenie::AceWeenie;

/// `MotionCommand.Ready`.
const READY: u32 = 0x4100_0003;

/// A run of animation frames as part poses, each held for its frame time (`CSequence` steps
/// whole frames).
#[derive(Debug, Clone, PartialEq)]
pub struct MotionClip {
    frames: Vec<(Vec<PartPose>, f64)>,
    length: f64,
}

impl MotionClip {
    /// The poses `time` seconds in, looping.
    pub fn looped(&self, time: f64) -> &[PartPose] {
        self.at(time.rem_euclid(self.length))
    }

    /// The poses `time` seconds in, held on the last frame past the end.
    pub fn at(&self, time: f64) -> &[PartPose] {
        let mut t = time.max(0.0);
        for (pose, duration) in &self.frames {
            if t < *duration {
                return pose;
            }
            t -= duration;
        }
        &self.frames[self.frames.len() - 1].0
    }

    /// Seconds from the first frame to the end of the last.
    pub fn length(&self) -> f64 {
        self.length
    }

    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }
}

/// One emote set the heartbeat can pick: its roll threshold and its motions.
#[derive(Debug, Clone, PartialEq)]
struct HeartbeatSet {
    probability: f64,
    /// (delay, motion) in order; motions with no link are left out.
    motions: Vec<(f64, u32)>,
}

/// What a creature does standing idle.
#[derive(Debug, Clone, PartialEq)]
pub struct IdleBehaviour {
    pub cycle: MotionClip,
    /// Each emote motion's link from the idle substate.
    actions: HashMap<u32, MotionClip>,
    /// By ascending probability, as `GetEmoteSet` orders them.
    sets: Vec<HeartbeatSet>,
    pub heartbeat_interval: f64,
}

impl IdleBehaviour {
    /// The motions an emote set can play, for reports.
    pub fn action_count(&self) -> usize {
        self.actions.len()
    }

    /// `GetEmoteSet` for a roll in `[0, 1)`: the first set whose probability is above it.
    fn pick(&self, roll: f64) -> Option<&HeartbeatSet> {
        self.sets.iter().find(|set| roll < set.probability)
    }
}

/// One creature's idle state over time.
#[derive(Debug, Clone, PartialEq)]
pub struct IdlePlayer {
    next_heartbeat: f64,
    /// Motions waiting their turn: (start time, motion).
    queue: VecDeque<(f64, u32)>,
    playing: Option<(f64, u32)>,
    cycle_start: f64,
    rng: u64,
}

impl IdlePlayer {
    /// A player starting at `time`; `seed` keeps creatures out of step (the first heartbeat
    /// falls somewhere in the first interval).
    pub fn new(behaviour: &IdleBehaviour, seed: u64, time: f64) -> Self {
        let mut player = Self {
            next_heartbeat: 0.0,
            queue: VecDeque::new(),
            playing: None,
            cycle_start: time,
            rng: seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1,
        };
        player.next_heartbeat = time + player.roll() * behaviour.heartbeat_interval;
        player
    }

    /// xorshift64*: a roll in `[0, 1)`.
    fn roll(&mut self) -> f64 {
        self.rng ^= self.rng >> 12;
        self.rng ^= self.rng << 25;
        self.rng ^= self.rng >> 27;
        (self.rng.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Steps to `time` and returns the poses to draw. `interval` overrides the weenie's
    /// heartbeat interval when set.
    pub fn pose<'a>(
        &mut self,
        behaviour: &'a IdleBehaviour,
        time: f64,
        interval: Option<f64>,
    ) -> &'a [PartPose] {
        let interval = interval.unwrap_or(behaviour.heartbeat_interval).max(0.1);
        while time >= self.next_heartbeat {
            let beat = self.next_heartbeat;
            self.next_heartbeat += interval;
            if self.playing.is_some() || !self.queue.is_empty() {
                continue;
            }
            let roll = self.roll();
            if let Some(set) = behaviour.pick(roll) {
                let mut at = beat.max(time);
                for (delay, motion) in &set.motions {
                    at += delay;
                    self.queue.push_back((at, *motion));
                    at += behaviour
                        .actions
                        .get(motion)
                        .map_or(0.0, MotionClip::length);
                }
            }
        }
        if let Some((start, motion)) = self.playing {
            let clip = &behaviour.actions[&motion];
            if time < start + clip.length() {
                return clip.at(time - start);
            }
            self.playing = None;
            self.cycle_start = start + clip.length();
        }
        if let Some(&(start, motion)) = self.queue.front() {
            if time >= start {
                self.queue.pop_front();
                if let Some(clip) = behaviour.actions.get(&motion) {
                    self.playing = Some((start, motion));
                    return clip.at(time - start);
                }
            }
        }
        behaviour.cycle.looped(time - self.cycle_start)
    }

    /// The motion playing now, if any.
    pub fn action(&self) -> Option<u32> {
        self.playing.map(|(_, motion)| motion)
    }
}

impl Ac1Dats {
    /// The cycle motion table `table` plays for `motion` in its default style
    /// (`(style << 16) | (motion & 0xFFFFFF)`), as poses for `parts` parts; `None` when the
    /// table has no such cycle.
    pub fn motion_cycle(
        &self,
        table: u32,
        motion: u32,
        parts: usize,
    ) -> Ac1Result<Option<MotionClip>> {
        let table = self.decode(table, MotionTable::decode)?;
        let key = (table.default_style << 16) | (motion & 0x00FF_FFFF);
        match table.cycles.get(&key) {
            Some(anims) => self.clip(anims, parts),
            None => Ok(None),
        }
    }

    /// `weenie`'s idle behaviour over `motion_table` for a Setup of `parts` parts; `None` when
    /// the table has no default cycle.
    pub(crate) fn idle_behaviour(
        &self,
        weenie: &AceWeenie,
        motion_table: u32,
        parts: usize,
    ) -> Ac1Result<Option<IdleBehaviour>> {
        let table = self.decode(motion_table, MotionTable::decode)?;
        let Some((style, substate)) = table.default_state() else {
            return Ok(None);
        };
        let key = (style << 16) | (substate & 0x00FF_FFFF);
        let Some(cycle) = table.cycles.get(&key) else {
            return Ok(None);
        };
        let Some(cycle) = self.clip(cycle, parts)? else {
            return Ok(None);
        };
        // The sets for the creature's stance; a weenie with none for it (some list only their
        // combat stance) takes every set, so it still acts.
        let stance = |set: &&crate::weenie::AceEmoteSet| {
            set.style.is_none_or(|s| s == style)
                && set.substyle.is_none_or(|s| s == READY || s == substate)
        };
        let mut chosen: Vec<_> = weenie.heartbeat_emotes.iter().filter(stance).collect();
        if chosen.is_empty() {
            chosen = weenie.heartbeat_emotes.iter().collect();
        }
        let mut actions = HashMap::new();
        let mut sets = Vec::new();
        for set in chosen {
            let mut motions = Vec::new();
            for &(delay, motion) in &set.motions {
                if let std::collections::hash_map::Entry::Vacant(slot) = actions.entry(motion) {
                    let clip = match table.link(style, substate, motion) {
                        Some(anims) => self.clip(anims, parts)?,
                        None => None,
                    };
                    let Some(clip) = clip else {
                        continue;
                    };
                    slot.insert(clip);
                }
                motions.push((delay, motion));
            }
            sets.push(HeartbeatSet {
                probability: set.probability,
                motions,
            });
        }
        sets.sort_by(|a, b| a.probability.total_cmp(&b.probability));
        Ok(Some(IdleBehaviour {
            cycle,
            actions,
            sets,
            heartbeat_interval: weenie.heartbeat_interval,
        }))
    }

    /// `anims` played in order, each over its frame range at its rate (a negative rate plays it
    /// backwards), as poses for `parts` parts.
    fn clip(&self, anims: &[AnimData], parts: usize) -> Ac1Result<Option<MotionClip>> {
        let mut frames = Vec::new();
        for data in anims {
            let animation = self.decode(data.animation, Animation::decode)?;
            let last = animation.frames.len() as i32 - 1;
            if last < 0 {
                continue;
            }
            let high = if data.high_frame < 0 {
                last
            } else {
                data.high_frame.min(last)
            };
            let low = data.low_frame.clamp(0, high);
            let rate = f64::from(data.framerate.abs()).max(1.0);
            let mut range: Vec<i32> = (low..=high).collect();
            if data.framerate < 0.0 {
                range.reverse();
            }
            for index in range {
                let pose = (0..parts)
                    .map(|part| {
                        let frame = animation.frames[index as usize]
                            .get(part)
                            .copied()
                            .unwrap_or(Frame::IDENTITY);
                        crate::creature::part_pose(frame)
                    })
                    .collect();
                frames.push((pose, 1.0 / rate));
            }
        }
        let length = frames.iter().map(|(_, d)| d).sum::<f64>();
        Ok((!frames.is_empty() && length > 0.0).then_some(MotionClip { frames, length }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clip(offsets: &[f32], frame: f64) -> MotionClip {
        MotionClip {
            frames: offsets
                .iter()
                .map(|x| {
                    (
                        vec![PartPose {
                            offset: [*x, 0.0, 0.0],
                            rotation: [1.0, 0.0, 0.0, 0.0],
                            scale: [1.0; 3],
                        }],
                        frame,
                    )
                })
                .collect(),
            length: offsets.len() as f64 * frame,
        }
    }

    #[test]
    fn a_clip_holds_each_frame_loops_or_holds_the_last() {
        let c = clip(&[0.0, 1.0], 0.1);
        assert_eq!(c.looped(0.05)[0].offset[0], 0.0);
        assert_eq!(c.looped(0.15)[0].offset[0], 1.0);
        assert_eq!(c.looped(0.25)[0].offset[0], 0.0);
        assert_eq!(c.at(5.0)[0].offset[0], 1.0);
    }

    #[test]
    fn a_heartbeat_plays_its_action_then_the_cycle_resumes() {
        let behaviour = IdleBehaviour {
            cycle: clip(&[0.0], 1.0),
            actions: HashMap::from([(0x1000_0051, clip(&[7.0, 8.0], 0.5))]),
            // A set every roll answers.
            sets: vec![HeartbeatSet {
                probability: 1.0,
                motions: vec![(0.0, 0x1000_0051)],
            }],
            heartbeat_interval: 5.0,
        };
        let mut player = IdlePlayer::new(&behaviour, 3, 0.0);
        let first = player.next_heartbeat;
        assert!((0.0..5.0).contains(&first));
        assert_eq!(
            player.pose(&behaviour, first - 0.01, None)[0].offset[0],
            0.0
        );
        assert_eq!(player.pose(&behaviour, first + 0.1, None)[0].offset[0], 7.0);
        assert_eq!(player.action(), Some(0x1000_0051));
        assert_eq!(player.pose(&behaviour, first + 0.6, None)[0].offset[0], 8.0);
        assert_eq!(player.pose(&behaviour, first + 1.1, None)[0].offset[0], 0.0);
        assert_eq!(player.action(), None);
    }

    #[test]
    fn the_heartbeat_takes_the_first_set_above_the_roll() {
        let set = |probability| HeartbeatSet {
            probability,
            motions: Vec::new(),
        };
        let behaviour = IdleBehaviour {
            cycle: clip(&[0.0], 1.0),
            actions: HashMap::new(),
            sets: vec![set(0.05), set(0.1)],
            heartbeat_interval: 5.0,
        };
        assert_eq!(behaviour.pick(0.01).map(|s| s.probability), Some(0.05));
        assert_eq!(behaviour.pick(0.07).map(|s| s.probability), Some(0.1));
        assert!(behaviour.pick(0.5).is_none());
    }
}
