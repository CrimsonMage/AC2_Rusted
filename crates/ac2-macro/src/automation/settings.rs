//! The tools' settings: one text file per character (`key=value` lines, unknown keys skipped),
//! and the changes the settings window asks for.

use super::butcher::ButcherSettings;
use super::rebuff::{RebuffEntry, RebuffSettings};

/// The settings' directory under the client's config directory.
pub const DIRECTORY: &str = "automation";

/// The file name for character `character`.
pub fn file_name(character: u64) -> String {
    format!("{character:016x}.txt")
}

/// Both tools' settings.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Settings {
    pub butcher: ButcherSettings,
    pub rebuff: RebuffSettings,
}

fn flag(value: bool) -> &'static str {
    if value {
        "1"
    } else {
        "0"
    }
}

impl Settings {
    pub fn encode(&self) -> Vec<u8> {
        let b = &self.butcher;
        let r = &self.rebuff;
        let mut text = String::new();
        let mut line = |key: &str, value: String| {
            text.push_str(key);
            text.push('=');
            text.push_str(&value);
            text.push('\n');
        };
        line("butcher.enabled", flag(b.enabled).into());
        line("butcher.max_distance", b.max_distance.to_string());
        line("butcher.pause_in_combat", flag(b.pause_in_combat).into());
        line("butcher.cooldown", b.cooldown.to_string());
        line("butcher.only_my_kills", flag(b.only_my_kills).into());
        line("rebuff.enabled", flag(r.enabled).into());
        line("rebuff.lead", r.lead.to_string());
        line(
            "rebuff.out_of_combat_only",
            flag(r.out_of_combat_only).into(),
        );
        line("rebuff.min_vigor", r.min_vigor.to_string());
        line("rebuff.wait_for_action", flag(r.wait_for_action).into());
        for entry in &r.entries {
            let name: String = entry
                .effect
                .chars()
                .filter(|c| *c != '\n' && *c != '\r')
                .collect();
            line(
                "rebuff.entry",
                format!("{}|{:x}|{}", flag(entry.enabled), entry.skill, name),
            );
        }
        text.into_bytes()
    }

    /// The settings a file holds; what it lacks keeps its default, a bad value is skipped.
    pub fn decode(bytes: &[u8]) -> Self {
        let mut settings = Self::default();
        let text = String::from_utf8_lossy(bytes);
        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let on = || value == "1";
            let number = |min: f32, max: f32| {
                value
                    .parse::<f32>()
                    .ok()
                    .filter(|v| v.is_finite())
                    .map(|v| v.clamp(min, max))
            };
            let b = &mut settings.butcher;
            let r = &mut settings.rebuff;
            match key {
                "butcher.enabled" => b.enabled = on(),
                "butcher.max_distance" => {
                    b.max_distance = number(1.0, MAX_DISTANCE).unwrap_or(b.max_distance)
                }
                "butcher.pause_in_combat" => b.pause_in_combat = on(),
                "butcher.only_my_kills" => b.only_my_kills = on(),
                "butcher.cooldown" => b.cooldown = number(0.0, MAX_COOLDOWN).unwrap_or(b.cooldown),
                "rebuff.enabled" => r.enabled = on(),
                "rebuff.lead" => r.lead = number(0.0, MAX_LEAD).unwrap_or(r.lead),
                "rebuff.out_of_combat_only" => r.out_of_combat_only = on(),
                "rebuff.min_vigor" => r.min_vigor = number(0.0, 1.0).unwrap_or(r.min_vigor),
                "rebuff.wait_for_action" => r.wait_for_action = on(),
                "rebuff.entry" => {
                    let mut parts = value.splitn(3, '|');
                    let (Some(enabled), Some(skill), Some(effect)) =
                        (parts.next(), parts.next(), parts.next())
                    else {
                        continue;
                    };
                    let Ok(skill) = u32::from_str_radix(skill, 16) else {
                        continue;
                    };
                    if !effect.is_empty() && r.entries.len() < MAX_ENTRIES {
                        r.entries.push(RebuffEntry {
                            effect: effect.to_owned(),
                            skill,
                            enabled: enabled == "1",
                        });
                    }
                }
                _ => {}
            }
        }
        settings
    }

    /// Applies one change from the settings window; returns whether anything changed.
    pub fn apply(&mut self, change: &Change) -> bool {
        let before = self.clone();
        let b = &mut self.butcher;
        let r = &mut self.rebuff;
        let step = |value: &mut f32, by: f32, min: f32, max: f32| {
            *value = ((*value + by) * 100.0).round() / 100.0;
            *value = value.clamp(min, max);
        };
        match change {
            Change::Toggle(Flag::Butcher) => b.enabled = !b.enabled,
            Change::Toggle(Flag::ButcherPauseInCombat) => b.pause_in_combat = !b.pause_in_combat,
            Change::Toggle(Flag::ButcherOnlyMyKills) => b.only_my_kills = !b.only_my_kills,
            Change::Toggle(Flag::Rebuff) => r.enabled = !r.enabled,
            Change::Toggle(Flag::OutOfCombat) => r.out_of_combat_only = !r.out_of_combat_only,
            Change::Toggle(Flag::WaitForAction) => r.wait_for_action = !r.wait_for_action,
            Change::Step(Value::MaxDistance, up) => {
                step(&mut b.max_distance, sign(*up), 1.0, MAX_DISTANCE)
            }
            Change::Step(Value::Cooldown, up) => {
                step(&mut b.cooldown, sign(*up), 0.0, MAX_COOLDOWN)
            }
            Change::Step(Value::Lead, up) => step(&mut r.lead, 5.0 * sign(*up), 0.0, MAX_LEAD),
            Change::Step(Value::MinVigor, up) => step(&mut r.min_vigor, 0.05 * sign(*up), 0.0, 1.0),
            Change::Maintain { effect, skill } => {
                if r.entries.len() < MAX_ENTRIES && !r.entries.iter().any(|e| e.effect == *effect) {
                    r.entries.push(RebuffEntry {
                        effect: effect.clone(),
                        skill: *skill,
                        enabled: true,
                    });
                }
            }
            Change::Forget(index) => {
                if *index < r.entries.len() {
                    r.entries.remove(*index);
                }
            }
            Change::ToggleEntry(index) => {
                if let Some(entry) = r.entries.get_mut(*index) {
                    entry.enabled = !entry.enabled;
                }
            }
            Change::SetSkill { index, skill } => {
                if let Some(entry) = r.entries.get_mut(*index) {
                    entry.skill = *skill;
                }
            }
        }
        *self != before
    }
}

fn sign(up: bool) -> f32 {
    if up {
        1.0
    } else {
        -1.0
    }
}

pub const MAX_DISTANCE: f32 = 30.0;
pub const MAX_COOLDOWN: f32 = 30.0;
pub const MAX_LEAD: f32 = 300.0;
pub const MAX_ENTRIES: usize = 16;

/// An on/off choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flag {
    Butcher,
    ButcherPauseInCombat,
    ButcherOnlyMyKills,
    Rebuff,
    OutOfCombat,
    WaitForAction,
}

/// A number stepped up or down.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Value {
    /// Metres, by 1.
    MaxDistance,
    /// Seconds, by 1.
    Cooldown,
    /// Seconds, by 5.
    Lead,
    /// Fraction of vigor, by 0.05.
    MinVigor,
}

/// What a press in the settings window asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    Toggle(Flag),
    /// `true`: up.
    Step(Value, bool),
    /// Keep `effect` up, cast by `skill`.
    Maintain {
        effect: String,
        skill: u32,
    },
    Forget(usize),
    ToggleEntry(usize),
    SetSkill {
        index: usize,
        skill: u32,
    },
}
