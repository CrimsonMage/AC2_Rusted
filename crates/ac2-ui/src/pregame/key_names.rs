//! The names of keys on the controls screen (`gmKeyboardUI::InqKeyStringInfo` reads them through
//! `ICIDM`'s name lookup).
//!
//! A keyboard semantic is `0x81000400 | scan code`. The names here are the DirectInput scan code
//! names (the public `DIK_*` constants), written out for this port. The retail names are the
//! strings `CKeyMap::LoadSemantics` and `CInputManager_WIN32::GetNameFromKey` hold in the client,
//! which this port does not copy, so a key's name can differ from retail's.

/// The keyboard semantics' base (`0x81000400 | DIK scan code`).
const KEY_BASE: u32 = 0x8100_0400;

/// The name of a key semantic, or a hex fallback for one this table does not name.
pub fn key_name(semantic: u32) -> String {
    if semantic & !0xFF == KEY_BASE {
        if let Some(name) = scan_code_name(semantic & 0xFF) {
            return name.to_owned();
        }
    }
    format!("Key {semantic:#010x}")
}

/// The semantic a key name stands for (`CKeyMap::GetSemanticByName`): the inverse of [`key_name`]
/// over the named keys, and the hex form `key_name` gives an unnamed semantic.
pub fn semantic_from_name(name: &str) -> Option<u32> {
    if let Some(hex) = name.strip_prefix("Key 0x") {
        return u32::from_str_radix(hex, 16).ok();
    }
    (0..=0xFF)
        .map(|code| KEY_BASE | code)
        .find(|&semantic| key_name(semantic) == name)
}

/// The DirectInput scan code names.
fn scan_code_name(code: u32) -> Option<&'static str> {
    let name = match code {
        0x01 => "Escape",
        0x02..=0x0A => return Some(DIGITS[(code - 0x02) as usize + 1]),
        0x0B => "0",
        0x0C => "-",
        0x0D => "=",
        0x0E => "Backspace",
        0x0F => "Tab",
        0x10 => "Q",
        0x11 => "W",
        0x12 => "E",
        0x13 => "R",
        0x14 => "T",
        0x15 => "Y",
        0x16 => "U",
        0x17 => "I",
        0x18 => "O",
        0x19 => "P",
        0x1A => "[",
        0x1B => "]",
        0x1C => "Enter",
        0x1D => "Left Control",
        0x1E => "A",
        0x1F => "S",
        0x20 => "D",
        0x21 => "F",
        0x22 => "G",
        0x23 => "H",
        0x24 => "J",
        0x25 => "K",
        0x26 => "L",
        0x27 => ";",
        0x28 => "'",
        0x29 => "`",
        0x2A => "Left Shift",
        0x2B => "\\",
        0x2C => "Z",
        0x2D => "X",
        0x2E => "C",
        0x2F => "V",
        0x30 => "B",
        0x31 => "N",
        0x32 => "M",
        0x33 => ",",
        0x34 => ".",
        0x35 => "/",
        0x36 => "Right Shift",
        0x37 => "Numpad *",
        0x38 => "Left Alt",
        0x39 => "Space",
        0x3A => "Caps Lock",
        0x3B..=0x44 => return Some(FUNCTION_KEYS[(code - 0x3B) as usize]),
        0x45 => "Num Lock",
        0x46 => "Scroll Lock",
        0x47 => "Numpad 7",
        0x48 => "Numpad 8",
        0x49 => "Numpad 9",
        0x4A => "Numpad -",
        0x4B => "Numpad 4",
        0x4C => "Numpad 5",
        0x4D => "Numpad 6",
        0x4E => "Numpad +",
        0x4F => "Numpad 1",
        0x50 => "Numpad 2",
        0x51 => "Numpad 3",
        0x52 => "Numpad 0",
        0x53 => "Numpad .",
        0x57 => "F11",
        0x58 => "F12",
        0x9C => "Numpad Enter",
        0x9D => "Right Control",
        0xB5 => "Numpad /",
        0xB7 => "Print Screen",
        0xB8 => "Right Alt",
        0xC5 => "Pause",
        0xC7 => "Home",
        0xC8 => "Up",
        0xC9 => "Page Up",
        0xCB => "Left",
        0xCD => "Right",
        0xCF => "End",
        0xD0 => "Down",
        0xD1 => "Page Down",
        0xD2 => "Insert",
        0xD3 => "Delete",
        0xDB => "Left Windows",
        0xDC => "Right Windows",
        0xDD => "Apps",
        _ => return None,
    };
    Some(name)
}

/// The digit keys `1` to `9` (index 1 to 9); index 0 is unused.
const DIGITS: [&str; 10] = ["", "1", "2", "3", "4", "5", "6", "7", "8", "9"];

/// F1 to F10.
const FUNCTION_KEYS: [&str; 10] = ["F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10"];

#[cfg(test)]
mod tests {
    use super::key_name;

    #[test]
    fn keys_are_named_by_scan_code() {
        assert_eq!(key_name(0x8100_042A), "Left Shift");
        assert_eq!(key_name(0x8100_041D), "Left Control");
        assert_eq!(key_name(0x8100_0439), "Space");
        assert_eq!(key_name(0x8100_0402), "1");
        assert_eq!(key_name(0x8100_040A), "9");
        assert_eq!(key_name(0x8100_0440), "F6");
    }

    #[test]
    fn unnamed_semantics_show_their_value() {
        assert_eq!(key_name(0x8200_0300), "Key 0x82000300");
        assert_eq!(key_name(0x8100_04EE), "Key 0x810004ee");
    }
}
