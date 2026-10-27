//! The option widgets of `gmCharacterUI`'s page: `OptionFlag` (a checkbox), `OptionValue` (a
//! slider over a range), `OptionChoice` (a drop-down) and `OptionExpander` (a header that shows
//! its options in a list), and the tree `gmCharacterUI::Setup` builds from them.
//!
//! Each option edits one setting of the player's options profile (`gmCWInterface::Get*` reads it,
//! `Set*` writes it through the player's script). Its widget keeps three values, as
//! `PreferenceBool` / `PreferenceFloat` / `PreferenceEnum` do: the current one the checkbox or
//! slider shows, the saved one the last apply or refresh committed (`mPrevious`), and the default
//! `SetDefault` gave it. The page reads the profile through [`OptionsSettings`].

use crate::element::UiTree;
use crate::layout::AttributeValue;
use crate::token::{tokenize, BUTTON_HIGHLIGHTED};

/// The attribute a drop-down's item holds its choice number in (`OptionChoice::AddChoice`'s `Key`).
const CHOICE_KEY: u32 = tokenize("Key");

/// The options profile's bit field (`GameplayOptionsProfile::m_bitField`, read by the `Get*`
/// accessors of `gmCWInterface`; the bit values are the profile's own).
pub const PROFILE_SHOW_SHORTCUT_NUMBERS: u32 = 0x1;
pub const PROFILE_SHOW_DAMAGE_TEXT: u32 = 0x4;
pub const PROFILE_SHOW_DAMAGE_TEXT_OTHER: u32 = 0x8;
pub const PROFILE_SHOW_RADAR_COORDS: u32 = 0x10;
pub const PROFILE_AUTO_TARGET: u32 = 0x20;
pub const PROFILE_ACCEPT_ALLEGIANCE: u32 = 0x40;
pub const PROFILE_ACCEPT_FELLOWSHIP: u32 = 0x80;
pub const PROFILE_ACCEPT_TRADE: u32 = 0x100;
pub const PROFILE_BYPASS_GOLD_MELTING: u32 = 0x400;
pub const PROFILE_SHORTCUT_LOCKED: u32 = 0x800;
pub const PROFILE_SHOW_PK_DAMAGE: u32 = 0x1000;
pub const PROFILE_SHORTCUT_INSTA_SCROLL: u32 = 0x2000;
pub const PROFILE_LATENCY_INDICATOR_GRAPH: u32 = 0x4000;
/// Set means hidden (`GetShowFellowsOnMap` inverts it).
pub const PROFILE_HIDE_FELLOWS_ON_MAP: u32 = 0x8000;
/// Set means hidden (`GetShowPetUI` inverts it).
pub const PROFILE_HIDE_PET_UI: u32 = 0x10000;

/// The profile values the page shows (`gmCWInterface::GetRadarMask`, `GetShortcutHeight`,
/// `GetDamageTextRangeOther`, and the bit field).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OptionsSettings {
    pub bits: u32,
    pub radar_mask: u32,
    pub shortcut_height: u32,
    pub damage_text_range_other: f32,
}

/// The `gmCWInterface` setter an option's value goes to (`ggmCWInterface::CallIFun_Set*`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Setter {
    AutoTarget,
    AcceptAllegiance,
    AcceptFellowship,
    AcceptTrade,
    ShowDamageText,
    ShowDamageTextOther,
    ShowShortcutNumbers,
    ShowPkDamage,
    ShortcutLocked,
    ShortcutInstaScroll,
    BypassGoldMeltingConfirmation,
    LatencyIndicatorGraph,
    ShowFellowsOnMap,
    ShowPetUi,
    ShowRadarCoords,
    RadarMask,
    DamageTextRangeOther,
    ShortcutHeight,
}

/// One setter call the page asks for: an int argument (flags, the mask, the height) or the float
/// range (`SetDamageTextRangeOther`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OptionSetting {
    Int { setter: Setter, value: u32 },
    Float { setter: Setter, value: f32 },
}

/// How an option's widget reads and writes its setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// The checkbox is the bit (`GetAutoTarget` and the like).
    Bit(u32),
    /// The checkbox shows the bit clear (`GetShowFellowsOnMap`, `GetShowPetUI`); the setter
    /// takes the checkbox as it is.
    Hidden(u32),
    /// The checkbox shows the bit clear (`RefreshFlag` for the gold-melting confirmation), and
    /// the setter takes the bypass, the checkbox's opposite (`ApplyFlag`).
    Bypass(u32),
    /// The checkbox is one bit of the radar mask (`RefreshFlag` / `ApplyFlag`'s mask).
    Radar(u32),
    /// The slider is `GetDamageTextRangeOther`.
    DamageRange,
    /// The drop-down is `GetShortcutHeight`.
    ShortcutHeight,
}

/// The widget an option is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    /// `OptionFlag` (`OptionCheckbox`).
    Flag,
    /// `OptionValue` (`OptionSlider`) over `lower..=upper`.
    Value { lower: f64, upper: f64 },
    /// `OptionChoice` (`OptionDropDown`) with the choices `1..=count`.
    Choice { count: u32 },
}

/// One option of the page (`gmCharacterUI::AddOption`'s `OptionBase`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OptionDesc {
    /// The label's string id name (`ID_Option_*Text`); the tooltip is `<name>_Help`.
    pub name: &'static str,
    pub kind: Kind,
    pub target: Target,
    pub setter: Setter,
    /// `SetDefault` in `gmCharacterUI::Setup`.
    pub default: f64,
}

/// One expander of the page (`OptionExpander`): its title and its options, in `Setup`'s order.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TreeDesc {
    pub title: &'static str,
    pub options: &'static [OptionDesc],
}

const fn flag(name: &'static str, target: Target, setter: Setter, default: f64) -> OptionDesc {
    OptionDesc {
        name,
        kind: Kind::Flag,
        target,
        setter,
        default,
    }
}

/// `gmCharacterUI::Setup`'s tree, in its order: the combat options, the radar filter, the social
/// options and the miscellaneous gameplay options.
pub const SETUP: [TreeDesc; 4] = [
    TreeDesc {
        title: "ID_Options_Combat",
        options: &[
            flag(
                "ID_Option_AutoTargetText",
                Target::Bit(PROFILE_AUTO_TARGET),
                Setter::AutoTarget,
                1.0,
            ),
            flag(
                "ID_Option_DamageNumbersText",
                Target::Bit(PROFILE_SHOW_DAMAGE_TEXT),
                Setter::ShowDamageText,
                1.0,
            ),
            flag(
                "ID_Option_ThirdPartyDamageText",
                Target::Bit(PROFILE_SHOW_DAMAGE_TEXT_OTHER),
                Setter::ShowDamageTextOther,
                0.0,
            ),
            flag(
                "ID_Option_ShowPKDamage",
                Target::Bit(PROFILE_SHOW_PK_DAMAGE),
                Setter::ShowPkDamage,
                1.0,
            ),
            OptionDesc {
                name: "ID_Option_ThirdPartyDamageRange",
                kind: Kind::Value {
                    lower: 0.0,
                    upper: 1.0,
                },
                target: Target::DamageRange,
                setter: Setter::DamageTextRangeOther,
                default: 1.0,
            },
        ],
    },
    TreeDesc {
        title: "ID_Options_RadarFilter",
        options: &[
            flag(
                "ID_Option_RadarShowMonsters",
                Target::Radar(0x1),
                Setter::RadarMask,
                1.0,
            ),
            flag(
                "ID_Option_RadarShowNPCs",
                Target::Radar(0x2),
                Setter::RadarMask,
                1.0,
            ),
            flag(
                "ID_Option_RadarShowPCs",
                Target::Radar(0x4),
                Setter::RadarMask,
                1.0,
            ),
            flag(
                "ID_Option_RadarShowPortals",
                Target::Radar(0x8),
                Setter::RadarMask,
                1.0,
            ),
            flag(
                "ID_Option_RadarShowLifestones",
                Target::Radar(0x10),
                Setter::RadarMask,
                1.0,
            ),
            flag(
                "ID_Option_RadarShowItems",
                Target::Radar(0x20),
                Setter::RadarMask,
                0.0,
            ),
            flag(
                "ID_Option_RadarShowCoordinates",
                Target::Bit(PROFILE_SHOW_RADAR_COORDS),
                Setter::ShowRadarCoords,
                1.0,
            ),
        ],
    },
    TreeDesc {
        title: "ID_Options_Social",
        options: &[
            flag(
                "ID_Option_AcceptAllegianceText",
                Target::Bit(PROFILE_ACCEPT_ALLEGIANCE),
                Setter::AcceptAllegiance,
                1.0,
            ),
            flag(
                "ID_Option_AcceptFellowshipText",
                Target::Bit(PROFILE_ACCEPT_FELLOWSHIP),
                Setter::AcceptFellowship,
                1.0,
            ),
            flag(
                "ID_Option_AcceptTradeText",
                Target::Bit(PROFILE_ACCEPT_TRADE),
                Setter::AcceptTrade,
                1.0,
            ),
        ],
    },
    TreeDesc {
        title: "ID_Options_MiscGameplay",
        options: &[
            flag(
                "ID_Option_ShortcutNumbersText",
                Target::Bit(PROFILE_SHOW_SHORTCUT_NUMBERS),
                Setter::ShowShortcutNumbers,
                1.0,
            ),
            flag(
                "ID_Option_ShortcutLocked",
                Target::Bit(PROFILE_SHORTCUT_LOCKED),
                Setter::ShortcutLocked,
                1.0,
            ),
            flag(
                "ID_Option_ShortcutInstaScroll",
                Target::Bit(PROFILE_SHORTCUT_INSTA_SCROLL),
                Setter::ShortcutInstaScroll,
                1.0,
            ),
            flag(
                "ID_Option_GoldMeltingConfirmation",
                Target::Bypass(PROFILE_BYPASS_GOLD_MELTING),
                Setter::BypassGoldMeltingConfirmation,
                0.0,
            ),
            flag(
                "ID_Option_LatencyIndicatorGraph",
                Target::Bit(PROFILE_LATENCY_INDICATOR_GRAPH),
                Setter::LatencyIndicatorGraph,
                1.0,
            ),
            flag(
                "ID_Option_ShowFellowsOnMap",
                Target::Hidden(PROFILE_HIDE_FELLOWS_ON_MAP),
                Setter::ShowFellowsOnMap,
                1.0,
            ),
            flag(
                "ID_Option_ShowPetUI",
                Target::Hidden(PROFILE_HIDE_PET_UI),
                Setter::ShowPetUi,
                1.0,
            ),
            OptionDesc {
                name: "ID_Option_ShortcutHeight",
                kind: Kind::Choice { count: 5 },
                target: Target::ShortcutHeight,
                setter: Setter::ShortcutHeight,
                default: 1.0,
            },
        ],
    },
];

/// The string id a name is in the UI table (`tokenize`, as the string tables key them).
pub fn string_id(name: &str) -> u32 {
    tokenize(name)
}

/// The label of choice `n` of a drop-down (`ID_Option_ShortcutHeight<n>`).
pub fn choice_name(name: &str, n: u32) -> String {
    format!("{name}{n}")
}

/// The element a widget's template is (`OptionBase::Create`'s element id).
pub fn template_name(kind: Kind) -> &'static str {
    match kind {
        Kind::Flag => "OptionCheckbox",
        Kind::Value { .. } => "OptionSlider",
        Kind::Choice { .. } => "OptionDropDown",
    }
}

/// `gmCWInterface::Get*`'s value of `target` in the profile, as the widget shows it: 0 or 1 for a
/// checkbox (`RefreshFlag`'s `SetCurrent`), the range's value for a slider, the height for a
/// drop-down.
pub fn read(target: Target, settings: &OptionsSettings) -> f64 {
    let bit_set = |bit: u32| settings.bits & bit != 0;
    let flag = |on: bool| f64::from(u8::from(on));
    match target {
        Target::Bit(bit) => flag(bit_set(bit)),
        Target::Hidden(bit) | Target::Bypass(bit) => flag(!bit_set(bit)),
        Target::Radar(bit) => flag(settings.radar_mask & bit != 0),
        // `RefreshFlag`'s `OptionValue` is `GetDamageTextRangeOther`, a float at the store.
        Target::DamageRange => f64::from(settings.damage_text_range_other),
        Target::ShortcutHeight => f64::from(settings.shortcut_height),
    }
}

/// The setting `value` of a widget with `target` and `setter` writes (`ApplyFlag`, `ApplyValue`,
/// `ApplyChoice`): the checkbox's value for a flag (the bypass inverted), the float range, the
/// height.
pub fn write(setter: Setter, target: Target, value: f64) -> OptionSetting {
    match target {
        Target::Bypass(_) => OptionSetting::Int {
            setter,
            value: u32::from(value == 0.0),
        },
        Target::DamageRange => OptionSetting::Float {
            setter,
            value: value as f32,
        },
        Target::ShortcutHeight => OptionSetting::Int {
            setter,
            value: value as u32,
        },
        Target::Bit(_) | Target::Hidden(_) | Target::Radar(_) => OptionSetting::Int {
            setter,
            value: u32::from(value != 0.0),
        },
    }
}

/// `OptionBase::IsChanged`: a flag or a choice differs from its saved value; a slider's
/// difference is over a hundredth.
pub(crate) fn is_changed_of(kind: Kind, current: f64, saved: f64) -> bool {
    match kind {
        Kind::Value { .. } => {
            let d = current - saved;
            d * d >= 0.0001
        }
        Kind::Flag | Kind::Choice { .. } => current != saved,
    }
}

/// The choice number an item of a drop-down holds (its `Key`).
fn choice_of(tree: &UiTree, item: usize) -> Option<i32> {
    match tree.get(item)?.attribute(CHOICE_KEY)? {
        AttributeValue::Int(value) => Some(*value),
        _ => None,
    }
}

/// `OptionChoice::AddChoice`: each `(number, label)` is a text item of the drop-down's menu,
/// keyed by its number and inserted in number order. Nothing is selected.
pub(crate) fn fill_choices(tree: &mut UiTree, control: Option<usize>, choices: &[(u32, String)]) {
    let Some(menu) = control else {
        return;
    };
    tree.menu_flush(menu);
    for (number, label) in choices {
        let items = tree.menu_items(menu).to_vec();
        let here = items
            .iter()
            .position(|&item| choice_of(tree, item).is_some_and(|key| key > *number as i32))
            .unwrap_or(items.len());
        if let Some(item) = tree.menu_insert_text(menu, label, here) {
            tree.set_int(item, CHOICE_KEY, *number as i32);
        }
    }
}

/// `OptionChoice::GetCurrent`: the number of the menu's selected item (`None` with no selection).
pub(crate) fn selected_choice(tree: &UiTree, control: Option<usize>) -> Option<f64> {
    let menu = control?;
    let item = tree.menu_selected_item(menu)?;
    choice_of(tree, item).map(f64::from)
}

/// `OptionChoice::SetCurrent`: the item keyed `choice` is selected. With no such item the
/// selection stays where it is.
fn select_choice(tree: &mut UiTree, control: Option<usize>, choice: f64) {
    let Some(menu) = control else {
        return;
    };
    let wanted = choice as i32;
    let item = tree
        .menu_items(menu)
        .iter()
        .copied()
        .find(|&item| choice_of(tree, item) == Some(wanted));
    if item.is_some() {
        tree.menu_set_selected_item(menu, item);
    }
}

/// Shows an option's value on its control when the drop-down is a menu: a choice selects its
/// item (`OptionChoice::SetCurrent`), the other kinds as [`sync_control_of`].
pub(crate) fn sync_option(tree: &mut UiTree, control: Option<usize>, kind: Kind, current: f64) {
    match kind {
        Kind::Choice { .. } => select_choice(tree, control, current),
        _ => sync_control_of(tree, control, kind, current, &[]),
    }
}

/// Shows a value on a control: the checkbox's highlight, the slider's position, the drop-down's
/// label (choice `n` is `labels[n - 1]`).
pub(crate) fn sync_control_of(
    tree: &mut UiTree,
    control: Option<usize>,
    kind: Kind,
    current: f64,
    labels: &[String],
) {
    let Some(control) = control else {
        return;
    };
    match kind {
        Kind::Flag => tree.set_bool(control, BUTTON_HIGHLIGHTED, current != 0.0),
        Kind::Value { lower, upper } => {
            let span = upper - lower;
            let position = if span == 0.0 {
                0.0
            } else {
                (current - lower) / span
            };
            tree.scrollbar_set_position(control, position as f32);
        }
        Kind::Choice { .. } => {
            let choice = (current as usize).checked_sub(1);
            if let Some(label) = choice.and_then(|c| labels.get(c)) {
                tree.set_text(control, label);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_setup_tree_is_retail_order() {
        let titles: Vec<&str> = SETUP.iter().map(|t| t.title).collect();
        assert_eq!(
            titles,
            [
                "ID_Options_Combat",
                "ID_Options_RadarFilter",
                "ID_Options_Social",
                "ID_Options_MiscGameplay"
            ]
        );
        let counts: Vec<usize> = SETUP.iter().map(|t| t.options.len()).collect();
        assert_eq!(counts, [5, 7, 3, 8]);
    }

    #[test]
    fn the_bypass_is_read_and_written_as_the_checkbox_opposite() {
        let on_bypass = OptionsSettings {
            bits: PROFILE_BYPASS_GOLD_MELTING,
            radar_mask: 0,
            shortcut_height: 1,
            damage_text_range_other: 1.0,
        };
        // The checkbox asks for the confirmation: unticked while the bypass is set.
        assert_eq!(
            read(Target::Bypass(PROFILE_BYPASS_GOLD_MELTING), &on_bypass),
            0.0
        );
        assert_eq!(
            write(
                Setter::BypassGoldMeltingConfirmation,
                Target::Bypass(PROFILE_BYPASS_GOLD_MELTING),
                0.0
            ),
            OptionSetting::Int {
                setter: Setter::BypassGoldMeltingConfirmation,
                value: 1
            }
        );
    }
}
