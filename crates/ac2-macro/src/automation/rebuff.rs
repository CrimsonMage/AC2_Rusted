//! Auto-rebuff: the effects the player chose are kept up by using the skill that casts each, from
//! its shortcut bar slot, as pressing the slot does (`gmCWInterface::UseShortcut`): the player's
//! script makes the request. A skill not on the bar is not used (there is no other gesture that
//! casts it). An effect is recast when it is missing or ends within the lead time.
//!
//! The effects come from the status bar strip's list (`gmCWInterface::InqEffectList`, the server's
//! `WM_Effect` 0x17 and 0x16): a name and, for a timed effect, its cast time and duration. A cast
//! that does not renew the effect within [`CONFIRM_SECONDS`] (refused, out of vigor, interrupted)
//! is not tried again for [`BACKOFF_SECONDS`]; a skill still recharging (`InqSkillTimeLeft`) is
//! not used.

use std::collections::BTreeMap;

/// The least time between two casts.
pub const MIN_GAP_SECONDS: f64 = 2.0;
/// A cast must renew its effect within this long.
pub const CONFIRM_SECONDS: f64 = 5.0;
/// How long a cast that did not renew its effect waits before the next try.
pub const BACKOFF_SECONDS: f64 = 60.0;

/// One effect kept up, and the skill that casts it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RebuffEntry {
    /// The effect's name as the strip shows it.
    pub effect: String,
    /// The skill's enum (the shortcut bar's `ShortcutInfo` value).
    pub skill: u32,
    pub enabled: bool,
}

/// The player's choices.
#[derive(Debug, Clone, PartialEq)]
pub struct RebuffSettings {
    pub enabled: bool,
    pub entries: Vec<RebuffEntry>,
    /// Seconds before the end at which an effect is recast.
    pub lead: f32,
    pub out_of_combat_only: bool,
    /// The least vigor, as a fraction of the maximum, to cast with.
    pub min_vigor: f32,
    /// Wait for the current action or cast to finish.
    pub wait_for_action: bool,
}

impl Default for RebuffSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            entries: Vec::new(),
            lead: 30.0,
            out_of_combat_only: true,
            min_vigor: 0.2,
            wait_for_action: true,
        }
    }
}

/// One of the player's effects.
#[derive(Debug, Clone, PartialEq)]
pub struct ActiveEffect {
    pub name: String,
    /// Server time it ends; `None` for an effect without a duration.
    pub ends: Option<f64>,
}

/// A skill the player could use: where it is on the bar and how long until it is ready.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkillState {
    pub skill: u32,
    pub slot: Option<u32>,
    /// Seconds until it can be used again (0 when ready).
    pub recharge: f32,
}

/// What the client knows this frame.
#[derive(Debug, Clone, Copy, Default)]
pub struct RebuffView<'a> {
    pub now: f64,
    /// In the world, alive, and not in portal space.
    pub ready: bool,
    pub in_combat: bool,
    /// Vigor as a fraction of its maximum.
    pub vigor: f32,
    /// An action or cast is under way (a special attack started and not ended, a use pending).
    pub busy: bool,
    pub effects: &'a [ActiveEffect],
    pub skills: &'a [SkillState],
}

/// The player's own gesture: press shortcut bar slot `slot` (it holds `skill`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UseShortcut {
    pub slot: u32,
    pub skill: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Attempt {
    at: f64,
    /// The effect's end when the cast went out.
    before: Option<f64>,
}

/// The auto-rebuff's state.
#[derive(Debug, Clone)]
pub struct AutoRebuff {
    pub settings: RebuffSettings,
    last_cast: f64,
    attempts: BTreeMap<u32, Attempt>,
    backoff: BTreeMap<u32, f64>,
}

impl Default for AutoRebuff {
    fn default() -> Self {
        Self {
            settings: RebuffSettings::default(),
            last_cast: f64::NEG_INFINITY,
            attempts: BTreeMap::new(),
            backoff: BTreeMap::new(),
        }
    }
}

/// The effect named `name`, as (present, end).
fn find<'a>(effects: &'a [ActiveEffect], name: &str) -> Option<&'a ActiveEffect> {
    effects.iter().find(|e| e.name == name)
}

impl AutoRebuff {
    pub fn with_settings(settings: RebuffSettings) -> Self {
        Self {
            settings,
            ..Self::default()
        }
    }

    /// Whether the entry's effect needs casting now (missing or within the lead time).
    pub fn due(&self, entry: &RebuffEntry, view: &RebuffView<'_>) -> bool {
        match find(view.effects, &entry.effect) {
            None => true,
            Some(effect) => effect
                .ends
                .is_some_and(|ends| ends - view.now <= f64::from(self.settings.lead)),
        }
    }

    /// One frame: at most one skill used.
    pub fn tick(&mut self, view: &RebuffView<'_>) -> Option<UseShortcut> {
        // Casts that went out: renewed, or given up on and backed off.
        let entries = self.settings.entries.clone();
        self.attempts.retain(|skill, attempt| {
            let renewed = entries.iter().filter(|e| e.skill == *skill).any(|e| {
                find(view.effects, &e.effect).is_some_and(|now| match (attempt.before, now.ends) {
                    (Some(before), Some(after)) => after > before,
                    (None, _) => true,
                    (Some(_), None) => true,
                })
            });
            if renewed {
                return false;
            }
            if view.now - attempt.at > CONFIRM_SECONDS {
                self.backoff.insert(*skill, view.now + BACKOFF_SECONDS);
                return false;
            }
            true
        });
        self.backoff.retain(|_, until| *until > view.now);
        let s = &self.settings;
        if !s.enabled
            || !view.ready
            || (s.out_of_combat_only && view.in_combat)
            || (s.wait_for_action && view.busy)
            || view.vigor < s.min_vigor
            || view.now - self.last_cast < MIN_GAP_SECONDS
        {
            return None;
        }
        let (entry, slot) = self
            .settings
            .entries
            .iter()
            .filter(|e| e.enabled)
            .find_map(|entry| {
                if self.attempts.contains_key(&entry.skill)
                    || self.backoff.contains_key(&entry.skill)
                    || !self.due(entry, view)
                {
                    return None;
                }
                let skill = view.skills.iter().find(|k| k.skill == entry.skill)?;
                if skill.recharge > 0.0 {
                    return None;
                }
                Some((entry.clone(), skill.slot?))
            })?;
        let before = find(view.effects, &entry.effect).and_then(|e| e.ends);
        self.attempts.insert(
            entry.skill,
            Attempt {
                at: view.now,
                before,
            },
        );
        self.last_cast = view.now;
        Some(UseShortcut {
            slot,
            skill: entry.skill,
        })
    }
}
