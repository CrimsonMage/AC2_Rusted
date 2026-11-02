//! The UI scale setting (a modern addition, AGENTS.md: the window resizes freely). Retail drew its
//! UI pixel for pixel on the device's display (`UIElementManager::RefreshEvent` and
//! `UIElement::UpdateSizeAndPosition` anchor every root to `RenderDevice::GetDisplayWidth` and
//! `GetDisplayHeight`), so on a bigger display the UI covered less of the screen. The default
//! here is that size (1x); the player may draw it larger, or let it fill the window as the
//! first modern client did ("Fill": `min(width / 1024, height / 768)`).
//!
//! A client-only preference, `UiScale`, in the tooltip keys' section (`Misc`) beside `ClassicHud`.
//! The value is the scale in hundredths (`100` is 1x, `250` is 2.5x) or `0` for Fill, so the
//! file reads as a number. The choice is a drop-down on the options page.

use ac2_prefs::{Choice, NameTable, Pref, PreferenceBank, Value};

use crate::tooltip_prefs::TOOLTIP_SECTION;

/// The client-only key (outside the DAT's `0x400000xx` range, beside `ClassicHud`).
pub const UI_SCALE: u32 = 0x7E00_0002;
/// The key's name in `Preferences.ini`.
pub const UI_SCALE_NAME: &str = "UiScale";
/// The section the key is filed under (`Misc`).
pub const UI_SCALE_SECTION: u32 = TOOLTIP_SECTION;
/// The owner the bank records for the key.
pub const OWNER: &str = "HudView";
/// The value that means Fill.
pub const FILL: i32 = 0;
/// The choices in the order the drop-down lists them (hundredths; [`FILL`] last).
pub const CHOICES: [i32; 8] = [100, 125, 150, 175, 200, 250, 300, FILL];

/// How large the UI is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum UiScaleSetting {
    /// Retail's size: one UI pixel per window pixel (the default, 100).
    #[default]
    Retail,
    /// A fixed scale, in hundredths (above 100).
    Percent(u32),
    /// The largest scale that keeps a 1024 x 768 display inside the window.
    Fill,
}

impl UiScaleSetting {
    /// The setting a stored value names; a value that is no choice is the default.
    pub fn from_value(value: i32) -> Self {
        match value {
            FILL => Self::Fill,
            100 => Self::Retail,
            v if CHOICES.contains(&v) => Self::Percent(v as u32),
            _ => Self::Retail,
        }
    }

    /// The stored value.
    pub fn value(self) -> i32 {
        match self {
            Self::Retail => 100,
            Self::Percent(p) => p as i32,
            Self::Fill => FILL,
        }
    }

    /// The scale asked for, `None` for Fill.
    pub fn factor(self) -> Option<f64> {
        match self {
            Self::Retail => Some(1.0),
            Self::Percent(p) => Some(f64::from(p) / 100.0),
            Self::Fill => None,
        }
    }

    /// The text of a choice on the options page.
    pub fn label(value: i32) -> String {
        match value {
            FILL => "Fill the window".to_owned(),
            100 => "1x (retail size)".to_owned(),
            v => format!("{}x", f64::from(v) / 100.0),
        }
    }
}

/// The key's name for the client's name table.
pub fn key_name(key: u32) -> Option<&'static str> {
    (key == UI_SCALE).then_some(UI_SCALE_NAME)
}

/// Registers the setting at 1x. Whether it was registered.
pub fn register(bank: &mut PreferenceBank, names: &dyn NameTable) -> bool {
    let choices = CHOICES
        .iter()
        .map(|&value| Choice {
            value,
            label: value as u32,
        })
        .collect();
    let Some(def) = Pref::enumeration(UI_SCALE_SECTION, UI_SCALE, None, choices, 100) else {
        return false;
    };
    if bank.register(def, names).is_some() {
        bank.link(UI_SCALE, OWNER);
        return true;
    }
    false
}

/// The setting the bank holds (1x when the key is missing).
pub fn ui_scale(bank: &PreferenceBank) -> UiScaleSetting {
    match bank.find(UI_SCALE).map(|p| &p.value) {
        Some(Value::Int(v)) => UiScaleSetting::from_value(*v),
        _ => UiScaleSetting::Retail,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_round_trip_and_unknown_ones_are_retail() {
        for value in CHOICES {
            assert_eq!(UiScaleSetting::from_value(value).value(), value);
        }
        assert_eq!(UiScaleSetting::from_value(333), UiScaleSetting::Retail);
        assert_eq!(UiScaleSetting::default().factor(), Some(1.0));
        assert_eq!(UiScaleSetting::from_value(FILL).factor(), None);
        assert_eq!(UiScaleSetting::from_value(175).factor(), Some(1.75));
    }

    #[test]
    fn choices_read_as_scales() {
        let labels: Vec<String> = CHOICES.iter().map(|&v| UiScaleSetting::label(v)).collect();
        assert_eq!(
            labels,
            [
                "1x (retail size)",
                "1.25x",
                "1.5x",
                "1.75x",
                "2x",
                "2.5x",
                "3x",
                "Fill the window"
            ]
        );
    }
}
