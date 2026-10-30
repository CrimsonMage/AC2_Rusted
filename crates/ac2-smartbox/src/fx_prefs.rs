//! The FX channel preferences (`ChannelPrefs::LoadChannelPrefs`, Legions 0x004F3730), registered
//! when the first `FXManager` is made: section `0x40000009` holds the gore amount (`0x40000035`, an
//! enum), the gore hue (`0x40000036`, an enum) and the foot step particles switch (`0x40000039`,
//! a bool). The function also links `0x40000037` (foot step sounds) but registers nothing for it
//! in 7160. Their handler, `ChannelPrefs::HandleApply` / `HandleChange`, writes each preference's
//! value into the table `FXManager::ChannelMissmatch` compares an FX node's channel keys with
//! ([`apply`], which sets the shared table in `ac2-assets`).
//!
//! The gore and hue choices run up to the top levels the language's `StringState` allows
//! (`StringStateInfo::GetTopGoreLevel` and `GetTopBloodLevel`: `m_topGoreLevel` and
//! `m_topBloodColor`), and the defaults are those levels. Without a `StringState` retail
//! registers none of the three.

use ac2_assets::{set_fx_channel_prefs, Ac2FxChannelPrefs, CHANNEL_KEYS, CHANNEL_PREFERENCES};
use ac2_prefs::{Choice, NameTable, Pref, PreferenceBank, Value};

/// The FX section (`ID_FX_FXSection`).
pub const FX_SECTION: u32 = 0x4000_0009;
/// `ID_FX_Gore_Amount`: 1 fun, 2 none, 3 low, 4 normal.
pub const GORE_AMOUNT: u32 = CHANNEL_PREFERENCES[0];
/// `ID_FX_Gore_Hue`: 1 green, 2 red.
pub const GORE_HUE: u32 = CHANNEL_PREFERENCES[1];
/// `ID_FX_Foot_Step_Particles`.
pub const FOOT_STEP_PARTICLES: u32 = CHANNEL_PREFERENCES[3];
/// The owner the bank records for the keys (`PreferenceBank::Link(key, ChannelPrefs)`).
pub const OWNER: &str = "ChannelPrefs";

/// The gore amount's mapper (`PreferenceEnum::Map(0x40000044)`) and the hue's (`0x40000045`).
const GORE_MAP: u32 = 0x4000_0044;
const HUE_MAP: u32 = 0x4000_0045;
/// The keys `LoadChannelPrefs` links (the handler also hears the foot step sounds key).
const LINKED: [u32; 4] = [0x4000_0035, 0x4000_0036, 0x4000_0037, 0x4000_0039];

/// An enum's choices as `LoadChannelPrefs` assigns them: choices `1..=always` unconditionally,
/// then each further choice up to `last` only while the top level is above the one before it.
fn levels(top: u32, always: u32, last: u32) -> Vec<Choice> {
    (1..=last)
        .filter(|n| *n <= always || top >= *n)
        .map(|n| Choice {
            value: n as i32,
            label: n,
        })
        .collect()
}

/// Registers the channel preferences in `bank` and links them to `ChannelPrefs`, as
/// `LoadChannelPrefs` does: the three registered keys need the `StringState`'s top gore level and
/// top blood level (`None`: no state, nothing is registered). Returns how many were registered.
pub fn register(
    bank: &mut PreferenceBank,
    names: &dyn NameTable,
    top_levels: Option<(u32, u32)>,
) -> usize {
    for key in LINKED {
        bank.link(key, OWNER);
    }
    let Some((top_gore, top_blood)) = top_levels else {
        return 0;
    };
    let defs = [
        Pref::enumeration(
            FX_SECTION,
            GORE_AMOUNT,
            Some(GORE_MAP),
            levels(top_gore, 2, 4),
            top_gore as i32,
        ),
        Pref::enumeration(
            FX_SECTION,
            GORE_HUE,
            Some(HUE_MAP),
            levels(top_blood, 1, 2),
            top_blood as i32,
        ),
        Some(Pref::boolean(FX_SECTION, FOOT_STEP_PARTICLES, true)),
    ];
    let mut registered = 0;
    for def in defs.into_iter().flatten() {
        if bank.register(def, names).is_some() {
            registered += 1;
        }
    }
    registered
}

/// The channel table the bank's values give (`ChannelPrefs::HandleApply`): each registered
/// preference's value under its channel key; a bool is 1 or 0.
pub fn channel_prefs(bank: &PreferenceBank) -> Ac2FxChannelPrefs {
    let mut table = Ac2FxChannelPrefs::default();
    for (key, pref) in CHANNEL_KEYS.iter().zip(CHANNEL_PREFERENCES) {
        let value = match bank.find(pref).map(|p| &p.value) {
            Some(Value::Int(v)) => u32::try_from(*v).ok(),
            Some(Value::Bool(v)) => Some(u32::from(*v)),
            _ => None,
        };
        if let Some(value) = value {
            table.set(*key, value);
        }
    }
    table
}

/// `ChannelPrefs::HandleApply`: the bank's values go into the table every `FXManager` reads.
pub fn apply(bank: &PreferenceBank) {
    set_fx_channel_prefs(channel_prefs(bank));
}

#[cfg(test)]
#[path = "fx_prefs_tests.rs"]
mod tests;
