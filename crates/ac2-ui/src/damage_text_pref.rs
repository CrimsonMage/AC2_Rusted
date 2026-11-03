//! The damage text size setting (a modern addition, AGENTS.md). Retail drew a floating damage
//! number at its `DamageTextBlob`'s scale alone (`gm3DTextUI::RecvNotice_Do3DDamage`: the
//! `impact_24` text shown at its measured size times the scale the script chose), so on a large
//! display the numbers were small. The player may draw them larger: the setting multiplies the
//! blob's scale, and the text is composed at that size from the font's coverage
//! ([`crate::DamageTexts::set_size`]).
//!
//! A client-only preference, `DamageTextSize`, in the tooltip keys' section (`Misc`) beside
//! `ClassicHud` and `UiScale`. The value is the size in hundredths (`100` is retail's size, `150`
//! is 1.5x). The choice is a drop-down on the options page; the default is 1.5x.

use ac2_prefs::{Choice, NameTable, Pref, PreferenceBank, Value};

use crate::tooltip_prefs::TOOLTIP_SECTION;

/// The client-only key (outside the DAT's `0x400000xx` range, after `ClassicHud` and `UiScale`).
pub const DAMAGE_TEXT_SIZE: u32 = 0x7E00_0003;
/// The key's name in `Preferences.ini`.
pub const DAMAGE_TEXT_SIZE_NAME: &str = "DamageTextSize";
/// The section the key is filed under (`Misc`).
pub const DAMAGE_TEXT_SIZE_SECTION: u32 = TOOLTIP_SECTION;
/// The owner the bank records for the key.
pub const OWNER: &str = "HudView";
/// Retail's size.
pub const RETAIL: i32 = 100;
/// The size a new profile starts at.
pub const DEFAULT: i32 = 150;
/// The choices in the order the drop-down lists them (hundredths).
pub const CHOICES: [i32; 6] = [RETAIL, 125, 150, 200, 250, 300];

/// The size a stored value names, as a factor of the blob's scale; a value that is no choice is
/// the default.
pub fn factor(value: i32) -> f64 {
    let value = if CHOICES.contains(&value) {
        value
    } else {
        DEFAULT
    };
    f64::from(value) / 100.0
}

/// The text of a choice on the options page.
pub fn label(value: i32) -> String {
    match value {
        RETAIL => "1x (retail size)".to_owned(),
        v => format!("{}x", f64::from(v) / 100.0),
    }
}

/// The key's name for the client's name table.
pub fn key_name(key: u32) -> Option<&'static str> {
    (key == DAMAGE_TEXT_SIZE).then_some(DAMAGE_TEXT_SIZE_NAME)
}

/// Registers the setting at its default. Whether it was registered.
pub fn register(bank: &mut PreferenceBank, names: &dyn NameTable) -> bool {
    let choices = CHOICES
        .iter()
        .map(|&value| Choice {
            value,
            label: value as u32,
        })
        .collect();
    let Some(def) = Pref::enumeration(
        DAMAGE_TEXT_SIZE_SECTION,
        DAMAGE_TEXT_SIZE,
        None,
        choices,
        DEFAULT,
    ) else {
        return false;
    };
    if bank.register(def, names).is_some() {
        bank.link(DAMAGE_TEXT_SIZE, OWNER);
        return true;
    }
    false
}

/// The size the bank holds, as a factor of the blob's scale (the default when the key is
/// missing).
pub fn damage_text_size(bank: &PreferenceBank) -> f64 {
    match bank.find(DAMAGE_TEXT_SIZE).map(|p| &p.value) {
        Some(Value::Int(v)) => factor(*v),
        _ => factor(DEFAULT),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_read_as_factors_and_unknown_ones_are_the_default() {
        assert_eq!(factor(RETAIL), 1.0);
        assert_eq!(factor(250), 2.5);
        assert_eq!(factor(175), 1.5);
        assert_eq!(factor(DEFAULT), 1.5);
    }

    #[test]
    fn choices_read_as_sizes() {
        let labels: Vec<String> = CHOICES.iter().map(|&v| label(v)).collect();
        assert_eq!(
            labels,
            ["1x (retail size)", "1.25x", "1.5x", "2x", "2.5x", "3x"]
        );
    }
}
