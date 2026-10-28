//! The options pages' setters as IFun calls (`ggmCWInterface::CallIFun_Set*`): the game-options and
//! chat-options settings the UI produces, turned into the IFun id and its arguments for the script VM.

use ac2_net::protocol::PackValue;

use ac2_script::host::ifun_ids::*;

use crate::ChatOptionSetting;
use crate::{OptionSetting, Setter};

/// The IFun and argument of an options page setting (`ggmCWInterface::CallIFun_Set*`).
pub fn option_call(setting: &OptionSetting) -> (u32, PackValue) {
    let setter = match setting {
        OptionSetting::Int { setter, .. } | OptionSetting::Float { setter, .. } => *setter,
    };
    let ifun = match setter {
        Setter::AutoTarget => IFUN_SET_AUTO_TARGET,
        Setter::AcceptAllegiance => IFUN_SET_ACCEPT_ALLEGIANCE,
        Setter::AcceptFellowship => IFUN_SET_ACCEPT_FELLOWSHIP,
        Setter::AcceptTrade => IFUN_SET_ACCEPT_TRADE,
        Setter::ShowDamageText => IFUN_SET_SHOW_DAMAGE_TEXT,
        Setter::ShowDamageTextOther => IFUN_SET_SHOW_DAMAGE_TEXT_OTHER,
        Setter::ShowShortcutNumbers => IFUN_SET_SHOW_SHORTCUT_NUMBERS,
        Setter::ShowPkDamage => IFUN_SET_SHOW_PK_DAMAGE,
        Setter::ShortcutLocked => IFUN_SET_SHORTCUT_LOCKED,
        Setter::ShortcutInstaScroll => IFUN_SET_SHORTCUT_INSTA_SCROLL,
        Setter::BypassGoldMeltingConfirmation => IFUN_SET_BYPASS_GOLD_MELTING,
        Setter::LatencyIndicatorGraph => IFUN_SET_LATENCY_INDICATOR_GRAPH,
        Setter::ShowFellowsOnMap => IFUN_SET_SHOW_FELLOWS_ON_MAP,
        Setter::ShowPetUi => IFUN_SET_SHOW_PET_UI,
        Setter::ShowRadarCoords => IFUN_SET_SHOW_RADAR_COORDS,
        Setter::RadarMask => IFUN_SET_RADAR_MASK,
        Setter::DamageTextRangeOther => IFUN_SET_DAMAGE_TEXT_RANGE_OTHER,
        Setter::ShortcutHeight => IFUN_SET_SHORTCUT_HEIGHT,
    };
    let arg = match setting {
        OptionSetting::Int { value, .. } => PackValue::Int(*value),
        OptionSetting::Float { value, .. } => PackValue::Float(*value),
    };
    (ifun, arg)
}

/// The IFun and arguments of a chat options setting (`ggmCWInterface::CallIFun_SetChat*`).
pub fn chat_option_call(setting: &ChatOptionSetting) -> (u32, Vec<PackValue>) {
    use crate::ChatOptionSetting as S;
    match *setting {
        S::FontSize { text_type, size } => (
            IFUN_SET_CHAT_FONT_SIZE,
            vec![PackValue::Int(text_type), PackValue::Int(size)],
        ),
        S::FontColor { text_type, color } => (
            IFUN_SET_CHAT_FONT_COLOR,
            vec![PackValue::Int(text_type), PackValue::Int(color)],
        ),
        S::PopupFlag { text_type, flag } => (
            IFUN_SET_CHAT_POPUP_FLAG,
            vec![PackValue::Int(text_type), PackValue::Int(flag)],
        ),
        S::WindowFilter { window, filter } => (
            IFUN_SET_CHAT_WINDOW_FILTER,
            vec![PackValue::Int(window), PackValue::Int(filter)],
        ),
        S::WindowTextType { window, text_type } => (
            IFUN_SET_CHAT_WINDOW_TEXT_TYPE,
            vec![PackValue::Int(window), PackValue::Int(text_type)],
        ),
    }
}
