//! Auto-butcher: when a butcherable corpse lies within reach and a butchering knife is in the
//! pack, the knife is used on it as the player uses it. The knife's use (select it, use the
//! selection) arms the targeting pointer (`CUsageSystem` 0x9 keeps the knife in the usage list
//! and raises `SetTargetingPointer`); the targeting click (select the corpse, use the selection)
//! completes it and the player's script sends `WM_Usage` itself. Nothing else is sent.
//!
//! A corpse is butcherable for a knife when the corpse's `Usage_TargetType` (721) shares a bit
//! with the knife's `Usage_ValidTargetTypes` (722, 8 on the butchering knives): the test the
//! player's script makes. A corpse eight or more levels below the player is passed over (the
//! servers refuse it: AC2Hero `butcher.html`, Pyrealium `butchery::too_far_below`). Each corpse is
//! tried once. With "only my kills" (the default) a corpse is used only when its examination
//! names the player as its killer: the corpse's profile has "Killed by" and the killer's name
//! (`examination.stt` 0x32A6B06F, in every one of the 5,302 corpse profiles in the 2017 captures),
//! asked for quietly as the hit chance's is (`OnlineSession::examine_quietly`); a corpse whose
//! profile names no killer within the result time is passed over. A refusal (`WM_Usage` 0x23, or no item reaching the pack in time) switches the tool
//! off with a line saying why.

use std::collections::BTreeSet;

/// The knife's use must arm the pointer within this many seconds.
pub const ARM_SECONDS: f64 = 3.0;
/// The butchered items must reach the pack within this many seconds of the targeting click.
pub const RESULT_SECONDS: f64 = 5.0;
/// AC2Hero `butcher.html`: a corpse eight or more levels below the butcher is refused.
pub const LEVEL_GAP: u32 = 8;
/// How many corpses tried are remembered.
const REMEMBERED: usize = 512;

/// The player's choices.
#[derive(Debug, Clone, PartialEq)]
pub struct ButcherSettings {
    pub enabled: bool,
    /// The farthest corpse, in metres: the client does not walk to a corpse, so only corpses
    /// already this close are used (6 m is the servers' butchering reach: Pyrealium's
    /// `combat_distance <= 6.0`).
    pub max_distance: f32,
    pub pause_in_combat: bool,
    /// Seconds between two uses.
    pub cooldown: f32,
    /// Only corpses whose examination names the player as their killer.
    pub only_my_kills: bool,
}

impl Default for ButcherSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            max_distance: 6.0,
            pause_in_combat: true,
            cooldown: 2.0,
            only_my_kills: true,
        }
    }
}

/// A knife in the player's pack and its `Usage_ValidTargetTypes` (722).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Knife {
    pub id: u64,
    pub valid_targets: u32,
}

/// Whose kill a corpse was, as far as its examinations tell.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum KilledBy {
    /// No profile of it named a killer yet.
    #[default]
    Unknown,
    /// The player.
    Player,
    /// Someone or something else.
    Other,
}

/// A world object that could be butchered: its `Usage_TargetType` (721), its distance from the
/// player, its level, when known, and its killer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Corpse {
    pub id: u64,
    pub target_type: u32,
    pub distance: f32,
    pub level: Option<u32>,
    pub killed_by: KilledBy,
}

/// What the client knows this frame.
#[derive(Debug, Clone, Copy, Default)]
pub struct ButcherView<'a> {
    /// Server time, in seconds.
    pub now: f64,
    /// In the world, alive, and not in portal space.
    pub ready: bool,
    /// The player is attacking.
    pub in_combat: bool,
    pub player_level: Option<u32>,
    pub knives: &'a [Knife],
    pub corpses: &'a [Corpse],
}

/// What happened since the last frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButcherEvent {
    /// `SetTargetingPointer`: the pointer was armed or put away.
    TargetingPointer(bool),
    /// `CUsageSystem` 0x2: a use failed (`WM_Usage` 0x23).
    UsageRefused,
    /// Something new reached the pack.
    PackGained,
}

/// What the client does: the player's own gestures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ButcherAction {
    /// Select the knife and use the selection (the knife's double-click).
    UseItem(u64),
    /// Select the corpse and use the selection (the targeting click).
    UseOn(u64),
    /// Ask quietly for the corpse's profile, to learn its killer.
    Examine(u64),
    /// The tool switched itself off: a line for the chat.
    Stopped(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Stage {
    Idle,
    Arming { corpse: u64, since: f64 },
    Using { since: f64 },
}

/// The auto-butcher's state.
#[derive(Debug, Clone)]
pub struct AutoButcher {
    pub settings: ButcherSettings,
    stage: Stage,
    tried: BTreeSet<u64>,
    last_use: f64,
    /// The corpse whose profile was asked for, and when.
    asked: Option<(u64, f64)>,
}

impl Default for AutoButcher {
    fn default() -> Self {
        Self {
            settings: ButcherSettings::default(),
            stage: Stage::Idle,
            tried: BTreeSet::new(),
            last_use: f64::NEG_INFINITY,
            asked: None,
        }
    }
}

fn remember(set: &mut BTreeSet<u64>, id: u64) {
    if set.len() >= REMEMBERED {
        if let Some(&first) = set.iter().next() {
            set.remove(&first);
        }
    }
    set.insert(id);
}

impl AutoButcher {
    pub fn with_settings(settings: ButcherSettings) -> Self {
        Self {
            settings,
            ..Self::default()
        }
    }

    /// Whether a use is under way (the knife armed or the corpse clicked).
    pub fn busy(&self) -> bool {
        self.stage != Stage::Idle
    }

    fn stop(&mut self, why: &str) -> Vec<ButcherAction> {
        self.settings.enabled = false;
        self.stage = Stage::Idle;
        vec![ButcherAction::Stopped(format!(
            "[Auto-butcher] {why}; switched off."
        ))]
    }

    /// One frame: the events since the last, then at most one gesture.
    pub fn tick(&mut self, view: &ButcherView<'_>, events: &[ButcherEvent]) -> Vec<ButcherAction> {
        let mut out = Vec::new();
        for event in events {
            match (*event, self.stage) {
                (ButcherEvent::TargetingPointer(true), Stage::Arming { corpse, .. }) => {
                    self.stage = Stage::Using { since: view.now };
                    out.push(ButcherAction::UseOn(corpse));
                }
                (ButcherEvent::UsageRefused, Stage::Arming { .. } | Stage::Using { .. }) => {
                    return self.stop("The server refused the butchering");
                }
                (ButcherEvent::PackGained, Stage::Using { .. }) => {
                    self.stage = Stage::Idle;
                    self.last_use = view.now;
                }
                _ => {}
            }
        }
        match self.stage {
            Stage::Arming { since, .. } if view.now - since > ARM_SECONDS => {
                return self.stop("The knife did not ask for a target");
            }
            Stage::Using { since } if view.now - since > RESULT_SECONDS => {
                return self.stop("Nothing was butchered");
            }
            Stage::Idle => {}
            _ => return out,
        }
        if !self.settings.enabled || !view.ready {
            return out;
        }
        if (self.settings.pause_in_combat && view.in_combat)
            || view.now - self.last_use < f64::from(self.settings.cooldown)
        {
            return out;
        }
        let Some(knife) = view.knives.iter().find(|k| k.valid_targets != 0) else {
            return out;
        };
        let only_mine = self.settings.only_my_kills;
        let candidate = view
            .corpses
            .iter()
            .filter(|c| self.eligible(c, knife, view.player_level))
            .filter(|c| !only_mine || c.killed_by != KilledBy::Other)
            .min_by(|a, b| a.distance.total_cmp(&b.distance));
        if let Some(corpse) = candidate.filter(|c| only_mine && c.killed_by == KilledBy::Unknown) {
            match self.asked {
                Some((id, at)) if id == corpse.id && view.now - at > RESULT_SECONDS => {
                    // No killer named in time: not known to be the player's.
                    remember(&mut self.tried, corpse.id);
                    self.asked = None;
                }
                Some((id, _)) if id == corpse.id => {}
                _ => {
                    self.asked = Some((corpse.id, view.now));
                    out.push(ButcherAction::Examine(corpse.id));
                }
            }
            return out;
        }
        if let Some(corpse) = candidate {
            remember(&mut self.tried, corpse.id);
            self.stage = Stage::Arming {
                corpse: corpse.id,
                since: view.now,
            };
            self.last_use = view.now;
            out.push(ButcherAction::UseItem(knife.id));
        }
        out
    }

    /// The knife may be used on `corpse` now.
    pub fn eligible(&self, corpse: &Corpse, knife: &Knife, player_level: Option<u32>) -> bool {
        let too_low =
            matches!((player_level, corpse.level), (Some(p), Some(c)) if p >= c + LEVEL_GAP);
        corpse.target_type & knife.valid_targets != 0
            && !self.tried.contains(&corpse.id)
            && corpse.distance <= self.settings.max_distance
            && !too_low
    }
}
