//! The chat options page's option specs and their values (`PopulateExpander`, `Refresh`):
//! which options a text type's expander has, their labels and defaults, and the value each reads
//! from the player's [`ChatSettings`].

use crate::character::Labels;
use crate::character_options::Kind;
use crate::chat::ChatSettings;
use crate::chat_options_table::{
    DEFAULT_WINDOW_TEXT_TYPE, ENTRY_TEXT_TYPE, FONT_COLOR_COUNT, FONT_COLOR_NAME, FONT_SIZES,
    FONT_SIZE_NAME, POPUP_NAME, ROWS, WINDOW_CHOICE_COUNT, WINDOW_NAMES, WINDOW_SECTION,
};

/// What an option edits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Field {
    FontSize,
    FontColor,
    /// The flag of window `n` (1 to 4) for the expander's text type.
    WindowFilter(u32),
    Popup,
    /// The text type of window `n`.
    WindowType(u32),
}

/// An option before its element is made.
pub(crate) struct Spec {
    pub(crate) field: Field,
    pub(crate) text_type: u32,
    pub(crate) name: &'static str,
    pub(crate) kind: Kind,
    pub(crate) labels: Vec<String>,
    pub(crate) default: f64,
}

pub(crate) fn flag(on: bool) -> f64 {
    f64::from(u8::from(on))
}

/// The choice of a text type's font size: its `SetDefault` value is the size itself (the
/// retail default is the size's index), so the default is the choice number of that index.
fn font_size_choice(size: u32) -> f64 {
    f64::from(size) + 1.0
}

/// The option specs of a text type's expander (`PopulateExpander`'s order).
pub(crate) fn specs(row: usize, defaults: &ChatSettings, labels: Labels) -> Vec<Spec> {
    let row = ROWS[row];
    if row.text_type == WINDOW_SECTION {
        let labels_of_rows: Vec<String> = ROWS
            .iter()
            .map(|r| {
                if r.window_choice {
                    labels.get(r.name)
                } else {
                    String::new()
                }
            })
            .collect();
        let default = ROWS
            .iter()
            .position(|r| r.text_type == DEFAULT_WINDOW_TEXT_TYPE)
            .map_or(1.0, |i| (i + 1) as f64);
        return (1..=4)
            .map(|window| Spec {
                field: Field::WindowType(window),
                text_type: row.text_type,
                name: WINDOW_NAMES[window as usize - 1],
                kind: Kind::Choice {
                    count: WINDOW_CHOICE_COUNT,
                },
                labels: labels_of_rows.clone(),
                default,
            })
            .collect();
    }
    let text_type = row.text_type;
    let entry = text_type == ENTRY_TEXT_TYPE;
    let size_count = if entry { 3 } else { 4 };
    let mut out = vec![Spec {
        field: Field::FontSize,
        text_type,
        name: FONT_SIZE_NAME,
        kind: Kind::Choice { count: size_count },
        labels: FONT_SIZES[..size_count as usize]
            .iter()
            .map(|(name, _)| labels.get(name))
            .collect(),
        default: font_size_choice(defaults.font_sizes.get(&text_type).copied().unwrap_or(0)),
    }];
    let colour = defaults.font_colors.get(&text_type).copied().unwrap_or(0);
    out.push(Spec {
        field: Field::FontColor,
        text_type,
        name: FONT_COLOR_NAME,
        kind: Kind::Choice {
            count: FONT_COLOR_COUNT,
        },
        labels: (1..=FONT_COLOR_COUNT)
            .map(|n| labels.get(&format!("ID_Option_ChatColor{n}")))
            .collect(),
        default: f64::from(colour) + 1.0,
    });
    if !entry {
        for window in 1..=4 {
            out.push(Spec {
                field: Field::WindowFilter(window),
                text_type,
                name: WINDOW_NAMES[window as usize - 1],
                kind: Kind::Flag,
                labels: Vec::new(),
                default: flag(defaults.filter(window) & text_type != 0),
            });
        }
        out.push(Spec {
            field: Field::Popup,
            text_type,
            name: POPUP_NAME,
            kind: Kind::Flag,
            labels: Vec::new(),
            default: flag(defaults.popups.get(&text_type).copied().unwrap_or(false)),
        });
    }
    out
}

/// The value an option shows for the player's settings (`Refresh`), or `None` when the setting
/// names no choice the control has (the control keeps its value).
pub(crate) fn read(field: Field, text_type: u32, settings: &ChatSettings) -> Option<f64> {
    match field {
        Field::FontSize => {
            let size = settings.font_sizes.get(&text_type).copied().unwrap_or(0);
            FONT_SIZES
                .iter()
                .position(|&(_, s)| s == size)
                .map(|i| (i + 1) as f64)
        }
        Field::FontColor => {
            let colour = settings.font_colors.get(&text_type).copied().unwrap_or(0);
            Some(f64::from(colour) + 1.0)
        }
        Field::WindowFilter(window) => Some(flag(settings.filter(window) & text_type != 0)),
        Field::Popup => Some(flag(
            settings.popups.get(&text_type).copied().unwrap_or(false),
        )),
        Field::WindowType(window) => {
            let wanted = settings.window_text_type(window);
            ROWS.iter()
                .rposition(|r| r.text_type == wanted)
                .map(|i| (i + 1) as f64)
        }
    }
}
