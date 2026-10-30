//! The HUD's style (a modern addition, AGENTS.md: the HUD's two styles). The modern style is the
//! client's own HUD built over the same frameworks and elements; the classic style is the retail
//! DAT HUD as the layouts author it. The choice is a client-only preference, `ClassicHud`, in the
//! tooltip keys' section (`0x4000000C`) so the options page lists it there. Its key has no name in
//! the DAT's key mapper, so the client's name table names it ([`CLASSIC_HUD_NAME`]).

use ac2_prefs::{NameTable, Pref, PreferenceBank, Value};

use crate::tooltip_prefs::TOOLTIP_SECTION;

/// The HUD's style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum HudStyle {
    /// The client's own HUD over the retail frameworks.
    Modern,
    /// The retail DAT HUD.
    #[default]
    Classic,
}

impl HudStyle {
    /// The other style.
    pub fn toggled(self) -> Self {
        match self {
            Self::Modern => Self::Classic,
            Self::Classic => Self::Modern,
        }
    }

    /// The style a command-line word names (`modern` or `classic`, any case).
    pub fn parse(word: &str) -> Option<Self> {
        match word.to_ascii_lowercase().as_str() {
            "modern" => Some(Self::Modern),
            "classic" => Some(Self::Classic),
            _ => None,
        }
    }
}

/// The client-only key of the classic switch. Retail's keys are `0x400000xx`; this one is outside
/// their range so it can never collide with a DAT key.
pub const CLASSIC_HUD: u32 = 0x7E00_0001;
/// The key's name in `Preferences.ini` and on the options page.
pub const CLASSIC_HUD_NAME: &str = "ClassicHud";
/// The section the key is filed under (`Misc`, the tooltip keys').
pub const HUD_STYLE_SECTION: u32 = TOOLTIP_SECTION;
/// The owner the bank records for the key.
pub const OWNER: &str = "HudView";

/// The client's own key names, for the keys the DAT's mapper does not name.
pub fn client_key_name(key: u32) -> Option<&'static str> {
    (key == CLASSIC_HUD).then_some(CLASSIC_HUD_NAME)
}

/// Registers the switch, on by default until the modern HUD is complete. Whether it was
/// registered.
pub fn register(bank: &mut PreferenceBank, names: &dyn NameTable) -> bool {
    let def = Pref::boolean(HUD_STYLE_SECTION, CLASSIC_HUD, true);
    if bank.register(def, names).is_some() {
        bank.link(CLASSIC_HUD, OWNER);
        return true;
    }
    false
}

/// The style the bank holds (classic when the key is missing).
pub fn hud_style(bank: &PreferenceBank) -> HudStyle {
    match bank.find(CLASSIC_HUD).map(|p| &p.value) {
        Some(Value::Bool(false)) => HudStyle::Modern,
        _ => HudStyle::Classic,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn styles_parse_and_toggle() {
        assert_eq!(HudStyle::parse("Modern"), Some(HudStyle::Modern));
        assert_eq!(HudStyle::parse("classic"), Some(HudStyle::Classic));
        assert_eq!(HudStyle::parse("retail"), None);
        assert_eq!(HudStyle::Modern.toggled(), HudStyle::Classic);
        assert_eq!(HudStyle::default(), HudStyle::Classic);
    }

    /// The client's table over a DAT that names only the section.
    struct Names;
    impl NameTable for Names {
        fn section_name(&self, id: u32) -> Option<String> {
            (id == HUD_STYLE_SECTION).then(|| "Misc".to_owned())
        }
        fn key_name(&self, id: u32) -> Option<String> {
            client_key_name(id).map(str::to_owned)
        }
        fn enum_name(&self, _: u32, _: u32) -> Option<String> {
            None
        }
        fn enum_label(&self, _: u32, _: &str) -> Option<u32> {
            None
        }
    }

    #[test]
    fn the_switch_reads_from_the_bank() {
        let names = Names;
        let mut bank = PreferenceBank::default();
        assert_eq!(hud_style(&bank), HudStyle::Classic);
        assert!(register(&mut bank, &names));
        assert_eq!(hud_style(&bank), HudStyle::Classic);
        bank.find_mut(CLASSIC_HUD).unwrap().value = Value::Bool(false);
        assert_eq!(hud_style(&bank), HudStyle::Modern);
        assert_eq!(client_key_name(CLASSIC_HUD), Some("ClassicHud"));
        assert_eq!(client_key_name(0x4000_005B), None);
    }
}
