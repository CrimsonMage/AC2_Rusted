//! `gmCharNameCharacterValidator` (ac2/shared/languagespecific/gmcharnames.cpp): which characters a
//! character's name may hold in the client's language. Retail kept one validator, the global
//! `CharNameCharacterValidator::g_validator` (the first constructed: `gmClient::m_NameValidator`,
//! language `0x80000001` until set), and `CLCache`'s language setup set its language
//! (`g_validator->SetLanguage`, Legions 0x004B43E2) to the language's id in enum mapper
//! `0x4000002A` (`Turbine::Client::GetLanguage`'s `EnumMapper::GetEnum`). The creation dialog's
//! name field filters typing through it (`NameInputFilter`). Here the language is a process-wide
//! value set once the local DAT's language is known ([`set_name_language`]).

use std::sync::atomic::{AtomicU32, Ordering};

/// `gmClient::gmClient`'s `m_NameValidator.m_lang`.
pub const DEFAULT_NAME_LANGUAGE: u32 = 0x8000_0001;
/// The language id `IsCharacterValidForTransitions` takes the jamo for (`-0x7efffffb`), and
/// `gmClient::Init` sets the `Korean` C locale for.
pub const KOREAN: u32 = 0x8100_0005;

/// One row of `LanguageLetters` (Legions 0x00A3B7D0, six rows of 0x58 bytes): the language id,
/// its letters beyond ASCII (`wcschr` of a zero-ended UTF-16 string at +4), and an inclusive
/// range of letters at +0x54 (none while its low end is 0).
struct LanguageLetters {
    language: u32,
    letters: &'static [u16],
    range: Option<(u16, u16)>,
}

/// The table as the Legions 7160 executable holds it, by the languages' names in mapper
/// `0x4000002A`: English, Francais, Deutsch, Korean (the Hangul syllables), Japanese (no letters
/// beyond ASCII) and Chinese (CJK 0x3400..=0x9FB0). The French row has `Ú` (0xDA) where `Ù`
/// (0xD9) would pair with `ù`; kept as retail.
const LANGUAGE_LETTERS: [LanguageLetters; 6] = [
    LanguageLetters {
        language: 0x8000_0001,
        letters: &[],
        range: None,
    },
    LanguageLetters {
        language: 0x8100_0001,
        letters: &[0xE8, 0xC8, 0xE9, 0xC9, 0xE0, 0xC0, 0xE7, 0xC7, 0xF9, 0xDA],
        range: None,
    },
    LanguageLetters {
        language: 0x8100_0003,
        letters: &[0xFC, 0xDC, 0xF6, 0xD6, 0xE4, 0xC4],
        range: None,
    },
    LanguageLetters {
        language: KOREAN,
        letters: &[],
        range: Some((0xAC00, 0xD7A3)),
    },
    LanguageLetters {
        language: 0x8100_0004,
        letters: &[],
        range: None,
    },
    LanguageLetters {
        language: 0x4100_0001,
        letters: &[],
        range: Some((0x3400, 0x9FB0)),
    },
];

/// `gmCharNameCharacterValidator::IsCharacterValid` (Legions 0x005CD990): an ASCII letter
/// (`isalpha` under the C locale), `'`, `-` or space, or one of the language's letters or in its
/// range; a language not in the table takes none of those.
pub fn is_character_valid(language: u32, ch: u16) -> bool {
    if ch < 0x80 && (ch as u8).is_ascii_alphabetic() {
        return true;
    }
    // `wcschr(L"'- ", ch)`, which also finds the terminating zero.
    if matches!(ch, 0x27 | 0x2D | 0x20 | 0) {
        return true;
    }
    let Some(row) = LANGUAGE_LETTERS.iter().find(|r| r.language == language) else {
        return false;
    };
    row.letters.contains(&ch)
        || row
            .range
            .is_some_and(|(low, high)| (low..=high).contains(&ch))
}

/// `gmCharNameCharacterValidator::IsCharacterValidForTransitions` (Legions 0x005CDA50): the
/// Hangul compatibility jamo 0x3131..=0x3163, in Korean only (the letters an IME composes a
/// syllable from).
pub fn is_character_valid_for_transitions(language: u32, ch: u16) -> bool {
    language == KOREAN && (0x3131..0x3164).contains(&ch)
}

static NAME_LANGUAGE: AtomicU32 = AtomicU32::new(DEFAULT_NAME_LANGUAGE);

/// `g_validator->SetLanguage(id)`: the language the name checks follow.
pub fn set_name_language(language: u32) {
    NAME_LANGUAGE.store(language, Ordering::Relaxed);
}

/// `gmCharNameCharacterValidator::GetLanguage`.
pub fn name_language() -> u32 {
    NAME_LANGUAGE.load(Ordering::Relaxed)
}

/// `NameInputFilter` (admin 0x0042CB60): a character the name field takes, by
/// [`is_character_valid`] or [`is_character_valid_for_transitions`] in the set language.
pub fn name_input_filter(ch: u16) -> bool {
    let language = name_language();
    is_character_valid(language, ch) || is_character_valid_for_transitions(language, ch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_language_takes_ascii_and_its_own_letters() {
        for language in [DEFAULT_NAME_LANGUAGE, 0x8100_0001, KOREAN, 0x1234_5678] {
            assert!(is_character_valid(language, u16::from(b'Q')));
            assert!(is_character_valid(language, u16::from(b'\'')));
            assert!(!is_character_valid(language, u16::from(b'7')));
        }
        assert!(
            !is_character_valid(DEFAULT_NAME_LANGUAGE, 0xE9),
            "é is not English"
        );
        assert!(is_character_valid(0x8100_0001, 0xE9), "é is French");
        assert!(is_character_valid(0x8100_0001, 0xDA), "retail's Ú");
        assert!(!is_character_valid(0x8100_0001, 0xD9), "not Ù");
        assert!(is_character_valid(0x8100_0003, 0xDC), "Ü is German");
        assert!(!is_character_valid(0x8100_0003, 0xE9));
        assert!(is_character_valid(KOREAN, 0xAC00) && is_character_valid(KOREAN, 0xD7A3));
        assert!(!is_character_valid(KOREAN, 0xD7A4));
        assert!(is_character_valid(0x4100_0001, 0x4E00));
        assert!(!is_character_valid(0x8100_0004, 0x4E00));
    }

    #[test]
    fn only_korean_takes_the_jamo_and_the_filter_follows_the_language() {
        assert!(is_character_valid_for_transitions(KOREAN, 0x3131));
        assert!(is_character_valid_for_transitions(KOREAN, 0x3163));
        assert!(!is_character_valid_for_transitions(KOREAN, 0x3164));
        assert!(!is_character_valid_for_transitions(
            DEFAULT_NAME_LANGUAGE,
            0x3131
        ));
        assert_eq!(name_language(), DEFAULT_NAME_LANGUAGE);
        assert!(!name_input_filter(0x3131));
        set_name_language(KOREAN);
        assert!(name_input_filter(0x3131) && name_input_filter(0xAC00));
        set_name_language(DEFAULT_NAME_LANGUAGE);
    }
}
