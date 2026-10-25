//! The chat options page's tables (`gmChatOptionsUI`, Legions 7160). Legions' static initialiser
//! (`0x0070CBD0`) fills the text-type rows of `s_chatTextTypeInfoTable` (stride 16: the text type
//! at +0, the name's string id at +4, a second id or flag at +8, the hidden flag at +0xC); the
//! window table (`0x0070CE80`) is the same rows at its own address, with the flag at +0xC saying
//! whether the text type is a window's choice. The font sizes and the colours are the
//! `ChatFontSizeChoices` and `ChatFontColorChoices` of the initialiser at `0x0070D130` and
//! `0x0070D1A0`.

/// The text type of the windows' section (`ID_Option_DevotedChatWindows`): its expander holds the
/// four windows' text-type choices instead of the font and filter options.
pub const WINDOW_SECTION: u32 = 0x0080_0000;
/// `TextType` of the entry line (`ID_Option_ChatEntryMessages`): its expander has no filters or
/// popup flag.
pub const ENTRY_TEXT_TYPE: u32 = 0x0004_0000;
/// The text type a window shows when its setting is unset (`FUN_005bfd50`'s default).
pub const DEFAULT_WINDOW_TEXT_TYPE: u32 = 0x10;
/// The window choices of [`WINDOW_SECTION`]: one per row of [`ROWS`].
pub const WINDOW_CHOICE_COUNT: u32 = 17;
/// The font size choices (`ChatFontSizeChoices`): the name's string id, and the size the
/// profile stores.
pub const FONT_SIZES: [(&str, u32); 4] = [
    ("ID_Option_ChatSmallerFont", 3),
    ("ID_Option_ChatSmallFont", 0),
    ("ID_Option_ChatMediumFont", 1),
    ("ID_Option_ChatLargeFont", 2),
];
/// The font colour choices (`ID_Option_ChatColor1` to `ID_Option_ChatColor10`): choice `n` is
/// the colour `n - 1`.
pub const FONT_COLOR_COUNT: u32 = 10;
/// The window labels (`ID_Option_Chat1` to `ID_Option_Chat4`); window `n` is `WINDOW_NAMES[n - 1]`.
pub const WINDOW_NAMES: [&str; 4] = [
    "ID_Option_Chat1",
    "ID_Option_Chat2",
    "ID_Option_Chat3",
    "ID_Option_Chat4",
];
/// The font size and colour labels, and the popup flag's.
pub const FONT_SIZE_NAME: &str = "ID_Option_ChatFontSize";
pub const FONT_COLOR_NAME: &str = "ID_Option_ChatFontColor";
pub const POPUP_NAME: &str = "ID_Option_ChatPopup";

/// One text type of the page: its expander's title (and the window choice's label), whether it is
/// hidden from the page (`+0xC` of the text-type table) and whether a window can choose it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextTypeRow {
    pub text_type: u32,
    pub name: &'static str,
    pub hidden: bool,
    pub window_choice: bool,
}

const fn row(text_type: u32, name: &'static str, hidden: bool, window_choice: bool) -> TextTypeRow {
    TextTypeRow {
        text_type,
        name,
        hidden,
        window_choice,
    }
}

/// The text-type rows in Legions' order (the page's expanders follow it, the hidden ones
/// excepted).
pub const ROWS: [TextTypeRow; 17] = [
    row(0x0000_0001, "ID_Option_ChatErrorMessages", false, false),
    row(0x0000_0002, "ID_Option_ChatCombatMessages", false, false),
    row(0x0000_0004, "ID_Option_ChatAdminMessages", true, false),
    row(0x0000_0008, "ID_Option_ChatStandardMessages", false, false),
    row(0x0000_0010, "ID_Option_ChatSayMessages", false, true),
    row(0x0000_0020, "ID_Option_ChatTellMessages", false, false),
    row(0x0000_0040, "ID_Option_ChatEmoteMessages", false, false),
    row(0x0000_0080, "ID_Option_ChatLogMessages", false, false),
    row(0x0001_0000, "ID_Option_ChatFellowshipMessages", false, true),
    row(0x0002_0000, "ID_Option_ChatAllegianceMessages", false, true),
    row(
        0x0040_0000,
        "ID_Option_ChatFactionChannelMessages",
        false,
        true,
    ),
    row(
        0x0008_0000,
        "ID_Option_ChatGeneralChannelMessages",
        false,
        true,
    ),
    row(
        0x0010_0000,
        "ID_Option_ChatTradeChannelMessages",
        false,
        true,
    ),
    row(
        0x0020_0000,
        "ID_Option_ChatRegionChannelMessages",
        false,
        true,
    ),
    row(0x0000_0100, "ID_Option_ChatBroadcastMessages", false, false),
    row(WINDOW_SECTION, "ID_Option_DevotedChatWindows", false, false),
    row(ENTRY_TEXT_TYPE, "ID_Option_ChatEntryMessages", false, false),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rows_are_legions_order_and_the_admin_row_is_hidden() {
        let types: Vec<u32> = ROWS.iter().map(|r| r.text_type).collect();
        assert_eq!(types.len(), 17);
        assert_eq!(types[0], 0x1);
        assert_eq!(types[15], WINDOW_SECTION);
        assert_eq!(types[16], ENTRY_TEXT_TYPE);
        let hidden: Vec<&str> = ROWS.iter().filter(|r| r.hidden).map(|r| r.name).collect();
        assert_eq!(hidden, ["ID_Option_ChatAdminMessages"]);
    }

    #[test]
    fn the_window_choices_are_the_channels_and_the_say_type() {
        let choices: Vec<u32> = ROWS
            .iter()
            .filter(|r| r.window_choice)
            .map(|r| r.text_type)
            .collect();
        assert_eq!(
            choices,
            [0x10, 0x1_0000, 0x2_0000, 0x40_0000, 0x8_0000, 0x10_0000, 0x20_0000]
        );
        assert!(choices.contains(&DEFAULT_WINDOW_TEXT_TYPE));
    }
}
