//! The hit chance settings (a modern addition, AGENTS.md: hit chance on name plates). Retail
//! showed a creature's hit chance only in its examination ("Chance to hit this target: ", the last
//! row of the profile the server sent). The player may show the latest one after the creature's
//! name on its name plate, and may let the client ask for the examinations of nearby creatures on
//! its own so the plates have a value.
//!
//! Two client-only preferences in the tooltip keys' section (`Misc`) beside `ClassicHud`,
//! `UiScale` and `DamageTextSize`, both off by default: `ShowHitChanceOnNameplates` and
//! `ExamineNearbyForHitChance`.

use ac2_prefs::{NameTable, Pref, PreferenceBank, Value};

use crate::tooltip_prefs::TOOLTIP_SECTION;

/// Show the hit chance on name plates (outside the DAT's `0x400000xx` range, after
/// `DamageTextSize`).
pub const SHOW_HIT_CHANCE: u32 = 0x7E00_0004;
/// Examine nearby creatures for their hit chance.
pub const EXAMINE_NEARBY: u32 = 0x7E00_0005;
/// The keys' names in `Preferences.ini` (the options page spaces them as its row labels).
pub const SHOW_HIT_CHANCE_NAME: &str = "ShowHitChanceOnNameplates";
pub const EXAMINE_NEARBY_NAME: &str = "ExamineNearbyForHitChance";
/// The section the keys are filed under (`Misc`).
pub const HIT_CHANCE_SECTION: u32 = TOOLTIP_SECTION;
/// The owner the bank records for the keys.
pub const OWNER: &str = "HudView";

/// The keys' names for the client's name table.
pub fn key_name(key: u32) -> Option<&'static str> {
    match key {
        SHOW_HIT_CHANCE => Some(SHOW_HIT_CHANCE_NAME),
        EXAMINE_NEARBY => Some(EXAMINE_NEARBY_NAME),
        _ => None,
    }
}

/// Registers both switches, off. Whether both were registered.
pub fn register(bank: &mut PreferenceBank, names: &dyn NameTable) -> bool {
    let mut all = true;
    for key in [SHOW_HIT_CHANCE, EXAMINE_NEARBY] {
        let def = Pref::boolean(HIT_CHANCE_SECTION, key, false);
        if bank.register(def, names).is_some() {
            bank.link(key, OWNER);
        } else {
            all = false;
        }
    }
    all
}

fn switch(bank: &PreferenceBank, key: u32) -> bool {
    matches!(bank.find(key).map(|p| &p.value), Some(Value::Bool(true)))
}

/// Whether name plates show the hit chance (off when the key is missing).
pub fn show_hit_chance(bank: &PreferenceBank) -> bool {
    switch(bank, SHOW_HIT_CHANCE)
}

/// Whether the client examines nearby creatures for it (off when the key is missing).
pub fn examine_nearby(bank: &PreferenceBank) -> bool {
    switch(bank, EXAMINE_NEARBY)
}

/// A name plate's text: the name, then the hit chance when there is one.
pub fn plate_text(name: &str, hit_chance: Option<u32>) -> String {
    match hit_chance {
        Some(percent) => format!("{name} {percent}%"),
        None => name.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Names;
    impl NameTable for Names {
        fn section_name(&self, id: u32) -> Option<String> {
            (id == HIT_CHANCE_SECTION).then(|| "Misc".to_owned())
        }
        fn key_name(&self, id: u32) -> Option<String> {
            key_name(id).map(str::to_owned)
        }
        fn enum_name(&self, _: u32, _: u32) -> Option<String> {
            None
        }
        fn enum_label(&self, _: u32, _: &str) -> Option<u32> {
            None
        }
    }

    #[test]
    fn both_switches_register_off_and_read_from_the_bank() {
        let mut bank = PreferenceBank::default();
        assert!(!show_hit_chance(&bank) && !examine_nearby(&bank));
        assert!(register(&mut bank, &Names));
        assert!(!show_hit_chance(&bank) && !examine_nearby(&bank));
        bank.find_mut(SHOW_HIT_CHANCE).unwrap().value = Value::Bool(true);
        assert!(show_hit_chance(&bank) && !examine_nearby(&bank));
        bank.find_mut(EXAMINE_NEARBY).unwrap().value = Value::Bool(true);
        assert!(examine_nearby(&bank));
    }

    #[test]
    fn the_plate_shows_the_percentage_after_the_name() {
        assert_eq!(plate_text("Drudge Robber", Some(64)), "Drudge Robber 64%");
        assert_eq!(plate_text("Drudge Robber", None), "Drudge Robber");
    }
}
