//! Vitae, the death penalty: `CurrentVitae` (float 0x11E, a percentage, 100 when unset) scales the
//! player's maximum health and vigor (`VitalSystem` 0x1 / 0x4 multiply the cached maxima by
//! `DeathSystem` 0x8, `CurrentVitae` times 0.01), `XPToRaiseVitae` (long 0x13C) is what
//! `CPlayer` 0x127 hands `CM_Character::SendNotice_UpdateVitae` as the cost, and the client's
//! `gmVitaeStatusUI` shows both. The player's script reads the two qualities and raises the notice
//! when `CurrentVitae` changes (`CPlayer` 0x85); the penalty itself, how much a death costs and
//! how experience restores it, is server code the shipped wlib stubs (`DeathSystem` 0x1, 0x5), so
//! the amounts below are the double's own test choices.

use ac2_net::protocol::QualityValue;

use crate::State;

/// `CurrentVitae` (float) and `XPToRaiseVitae` (long) (`gmFloatStats`, `gmLongIntStats`).
pub const CURRENT_VITAE: u32 = 0x11E;
pub const XP_TO_RAISE_VITAE: u32 = 0x13C;
/// What a death takes off the vitae percentage, and the lowest it goes (the double's choice).
pub const VITAE_PENALTY: f32 = 5.0;
pub const VITAE_FLOOR: f32 = 60.0;
/// The experience that raises the vitae by one point (the double's choice).
pub const XP_PER_VITAE_POINT: u64 = 100;

/// The player's vitae.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vitae {
    /// `CurrentVitae`.
    pub percent: f32,
    /// `XPToRaiseVitae`: the experience still needed for the next point (0 at full vitae).
    pub xp_needed: u64,
}

impl Default for Vitae {
    fn default() -> Self {
        Self {
            percent: 100.0,
            xp_needed: 0,
        }
    }
}

impl State {
    /// A maximum as the client shows it under the vitae (`VitalSystem` 0x1 / 0x4): the cached
    /// maximum times `DeathSystem` 0x8 (a float product, truncated), at least 1.
    pub fn shown_max(&self, cached: u32) -> u32 {
        let factor = (f64::from(self.vitals.vitae.percent) * f64::from(0.01f32)) as f32;
        let scaled = (f64::from(cached as f32) * f64::from(factor)) as f32;
        (scaled as i32).max(1) as u32
    }

    /// Sends the vitae qualities (private updates: the float, then the long).
    pub fn send_vitae(&mut self) {
        let Vitae { percent, xp_needed } = self.vitals.vitae;
        self.send_private(CURRENT_VITAE, QualityValue::Float(percent));
        self.send_private(XP_TO_RAISE_VITAE, QualityValue::LongInt(xp_needed as i64));
    }

    /// A death costs [`VITAE_PENALTY`] points (to [`VITAE_FLOOR`]); the next point costs
    /// [`XP_PER_VITAE_POINT`].
    pub(crate) fn apply_vitae_penalty(&mut self) {
        let vitae = &mut self.vitals.vitae;
        vitae.percent = (vitae.percent - VITAE_PENALTY).max(VITAE_FLOOR);
        if vitae.percent < 100.0 && vitae.xp_needed == 0 {
            vitae.xp_needed = XP_PER_VITAE_POINT;
        }
        self.send_vitae();
    }

    /// Experience earned pays the vitae back: each [`XP_PER_VITAE_POINT`] raises it a point, until
    /// it is full.
    pub(crate) fn restore_vitae(&mut self, mut xp: u64) {
        let before = self.vitals.vitae;
        let vitae = &mut self.vitals.vitae;
        while vitae.percent < 100.0 && xp > 0 {
            let paid = xp.min(vitae.xp_needed);
            xp -= paid;
            vitae.xp_needed -= paid;
            if vitae.xp_needed == 0 {
                vitae.percent = (vitae.percent + 1.0).min(100.0);
                if vitae.percent < 100.0 {
                    vitae.xp_needed = XP_PER_VITAE_POINT;
                }
            }
        }
        if self.vitals.vitae != before {
            // The float first when it changed, as a death sends it; the long always follows.
            self.send_vitae();
        }
    }
}
