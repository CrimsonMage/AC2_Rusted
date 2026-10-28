//! The tooltip preferences (`UIElementManager::Init`, admin 0x005E6F70; `HandleApply`, admin
//! 0x005E4D70, Legions 0x005A5C00). `Init` registers two keys in section `0x4000000C`: the delay
//! before a tooltip shows (`ID_Misc_TooltipDelay`, a float from 0 to 10, defaulting to the
//! manager's 0.25 seconds) and whether tooltips show at all (`ID_Misc_TooltipEnable`, a bool,
//! on), both linked to the manager.
//!
//! An applied delay takes effect at once: a tooltip not yet shown starts when the pointer has
//! already rested that long, and a shown one goes when its duration has run (the checks of
//! `UseTime`). An applied switch clears the tooltip and starts it again, which shows nothing
//! while tooltips are off.

use ac2_prefs::{NameTable, Pref, PreferenceBank, Value};

use crate::element::UiTree;

/// The section of the tooltip keys.
pub const TOOLTIP_SECTION: u32 = 0x4000_000C;
/// `ID_Misc_TooltipDelay`.
pub const TOOLTIP_DELAY: u32 = 0x4000_005B;
/// `ID_Misc_TooltipEnable`.
pub const TOOLTIP_ENABLE: u32 = 0x4000_005C;
/// The owner the bank records for the tooltip keys.
pub const OWNER: &str = "UIElementManager";

/// `UIElementManager::Init`'s registration: the two keys, linked to the manager. Returns how many
/// were registered.
pub fn register(bank: &mut PreferenceBank, names: &dyn NameTable) -> usize {
    let defs = [
        Pref::float(TOOLTIP_SECTION, TOOLTIP_DELAY, 0.25, 0.0, 10.0),
        Some(Pref::boolean(TOOLTIP_SECTION, TOOLTIP_ENABLE, true)),
    ];
    let mut registered = 0;
    for def in defs.into_iter().flatten() {
        let key = def.key;
        if bank.register(def, names).is_some() {
            bank.link(key, OWNER);
            registered += 1;
        }
    }
    registered
}

/// The delay and the switch in `bank` (a missing key keeps the manager's default).
pub fn values(bank: &PreferenceBank) -> (f32, bool) {
    let delay = match bank.find(TOOLTIP_DELAY).map(|p| &p.value) {
        Some(Value::Float(v)) => *v,
        _ => 0.25,
    };
    let enable = match bank.find(TOOLTIP_ENABLE).map(|p| &p.value) {
        Some(Value::Bool(v)) => *v,
        _ => true,
    };
    (delay, enable)
}

impl UiTree {
    /// `UIElementManager::HandleApply` for the tooltip keys.
    pub fn apply_tooltip_prefs(&mut self, delay: f32, enabled: bool) {
        if f64::from(delay) != self.tooltip.delay {
            self.tooltip.delay = f64::from(delay);
            self.tooltip_use_time(self.time);
        }
        if enabled != self.tooltip.enabled {
            self.tooltip.enabled = enabled;
            self.clear_tooltip();
            self.start_tooltip();
        }
    }
}
