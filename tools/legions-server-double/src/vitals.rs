//! The player's vitals as the double's server keeps them: health and vigor, each a pair of int
//! qualities (`Health_CurrentLevel` 0x102 / `Health_CachedMax` 0x107, `Vigor_CurrentLevel` 0x103 /
//! `Vigor_CachedMax` 0x108, `gmIntStats`) the client's `VitalSystem` 0x1 / 0x4 read, and the float
//! regeneration rates (`Health_RegenRate` 0x106, `Vigor_RegenRate` 0x107,
//! `Health_CombatRegenRate` 0x10B, `Vigor_CombatRegenRate` 0x10C, `gmFloatStats`).
//!
//! What the client shows is `Cached_Max` plus the attribute terms its own `VitalSystem` adds
//! (`SkillSystem` 0x29 / 0x35); the server holds the sum in the current quality, so a character at
//! full vitals has `current == shown max`. The shipped wlib's server-side vital functions are
//! stubs (`VitalSystem` 0x3, 0x6-0x8 and 0xA-0xD are crash opcodes), so the starting values, the
//! costs and the regeneration below are the double's own test choices; only the qualities, the
//! private update messages and the failure message are retail's.

use std::collections::BTreeMap;

use ac2_net::protocol::{CombatMessage, QualityValue};

use crate::{State, PLAYER};

/// `Vigor_CurrentLevel` and `Vigor_CachedMax` (`gmIntStats`).
pub const VIGOR: u32 = 0x103;
pub const VIGOR_MAX: u32 = 0x108;
/// The regeneration rates, in points a second (`gmFloatStats`): resting and in combat.
pub const HEALTH_REGEN_RATE: u32 = 0x106;
pub const VIGOR_REGEN_RATE: u32 = 0x107;
pub const HEALTH_COMBAT_REGEN_RATE: u32 = 0x10B;
pub const VIGOR_COMBAT_REGEN_RATE: u32 = 0x10C;
/// The player's starting vigor (and its max): the double's choice.
pub const PLAYER_VIGOR_MAX: u32 = 60;
/// Points a second the double regenerates, resting and in combat (the double's choice).
pub const HEALTH_REGEN: f32 = 1.0;
pub const VIGOR_REGEN: f32 = 4.0;
pub const HEALTH_COMBAT_REGEN: f32 = 0.0;
pub const VIGOR_COMBAT_REGEN: f32 = 1.0;
/// Seconds between regeneration steps.
pub const REGEN_INTERVAL: f64 = 1.0;
/// `gmWErrorType::Combat_NotEnoughVigor`: the failure of a skill used without the vigor.
pub const COMBAT_NOT_ENOUGH_VIGOR: u32 = 0x4E20;

/// The vital state beyond health (which `Fight` keeps): vigor, what skills cost, and the
/// regeneration clock.
#[derive(Debug, Clone, Default)]
pub struct Vitals {
    /// Vigor (current, max).
    pub vigor: (u32, u32),
    /// The vigor each skill costs a use (skill id -> vigor); a skill without an entry is free.
    pub vigor_costs: BTreeMap<u32, u32>,
    /// When the next regeneration step is due (0: not started).
    pub regen_due: f64,
    /// Regeneration is off (a test of exact vital arithmetic pauses it).
    pub regen_paused: bool,
}

/// The player's vital qualities a login describes (`CBaseQualities` ints): health, vigor and
/// their maxima.
pub(crate) fn login_ints(health: (u32, u32), vigor: (u32, u32)) -> [(u32, i32); 4] {
    [
        (crate::HEALTH, health.0 as i32),
        (VIGOR, vigor.0 as i32),
        (crate::HEALTH_MAX, health.1 as i32),
        (VIGOR_MAX, vigor.1 as i32),
    ]
}

/// The regeneration rates a login describes (`CBaseQualities` floats).
pub(crate) fn login_floats() -> [(u32, f32); 4] {
    [
        (HEALTH_REGEN_RATE, HEALTH_REGEN),
        (VIGOR_REGEN_RATE, VIGOR_REGEN),
        (HEALTH_COMBAT_REGEN_RATE, HEALTH_COMBAT_REGEN),
        (VIGOR_COMBAT_REGEN_RATE, VIGOR_COMBAT_REGEN),
    ]
}

impl State {
    /// The player's vigor and its max (private qualities, as health's).
    pub fn send_player_vigor(&mut self) {
        let (vigor, max) = self.vitals.vigor;
        self.send_private(VIGOR, QualityValue::Int(vigor as i32));
        self.send_private(VIGOR_MAX, QualityValue::Int(max as i32));
    }

    /// Sets what `skill` costs a use in vigor.
    pub fn set_vigor_cost(&mut self, skill: u32, cost: u32) {
        self.vitals.vigor_costs.insert(skill, cost);
    }

    /// The player used `skill` on `target`: its vigor cost comes off and the vigor quality goes
    /// out, or, without the vigor, the use fails with `Combat_NotEnoughVigor` (`WM_Combat` 0x1A)
    /// and nothing is spent. Returns whether the use goes on.
    pub(crate) fn charge_vigor(&mut self, skill: u32) -> bool {
        let cost = self.vitals.vigor_costs.get(&skill).copied().unwrap_or(0);
        if cost == 0 {
            return true;
        }
        if self.vitals.vigor.0 < cost {
            self.send_combat(&CombatMessage::ActionFailed {
                error: COMBAT_NOT_ENOUGH_VIGOR,
                subject: PLAYER,
                skill,
            });
            return false;
        }
        self.vitals.vigor.0 -= cost;
        self.send_private(VIGOR, QualityValue::Int(self.vitals.vigor.0 as i32));
        true
    }

    /// Regeneration: every [`REGEN_INTERVAL`] the living player gains the rate for resting or
    /// for fighting (an attack under way) of each vital, up to its max, each change a private
    /// quality update.
    pub(crate) fn regenerate(&mut self) {
        if self.vitals.regen_due == 0.0 {
            self.vitals.regen_due = self.elapsed + REGEN_INTERVAL;
        }
        if self.vitals.regen_paused
            || self.elapsed < self.vitals.regen_due
            || self.fight.player_dead
        {
            return;
        }
        self.vitals.regen_due = self.elapsed + REGEN_INTERVAL;
        let fighting = self.fight.attack.is_some();
        let (health_rate, vigor_rate) = if fighting {
            (HEALTH_COMBAT_REGEN, VIGOR_COMBAT_REGEN)
        } else {
            (HEALTH_REGEN, VIGOR_REGEN)
        };
        let (health, max) = self.fight.player;
        let healed = (health + health_rate as u32).min(max);
        if max > 0 && healed != health {
            self.fight.player.0 = healed;
            self.send_private(crate::HEALTH, QualityValue::Int(healed as i32));
        }
        let (vigor, max) = self.vitals.vigor;
        let rested = (vigor + vigor_rate as u32).min(max);
        if max > 0 && rested != vigor {
            self.vitals.vigor.0 = rested;
            self.send_private(VIGOR, QualityValue::Int(rested as i32));
        }
    }
}
