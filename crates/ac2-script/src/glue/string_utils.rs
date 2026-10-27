//! `gStringUtils`: the script's word test over a string (`portal/shared/src/interp/glue/
//! gstringutils.cpp`, `IsWordInString`).

use super::cparse::wcsicmp;
use super::{Call, INT_RESULT};
use crate::vm::{Context, EFuns, InterpStatus, Vm};

/// The separators `PSUtils::FindAllWords` splits the word list at (`" ,;"`).
const SEPARATORS: [u16; 3] = [b' ' as u16, b',' as u16, b';' as u16];

pub(super) fn register(efuns: &mut EFuns) {
    efuns.register(0x25BB_0001, is_word_in_string);
}

/// `gStringUtils::CallEFun_IsWordInString(str, words)`: 1 when `str` is one of the words in
/// `words`, compared without case (`_wcsicmp`), 0 otherwise. The words are split at the
/// separators, and an empty piece is no word.
fn is_word_in_string(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (str_ref, words_ref) = (call.word(2), call.word(3));
    let found = match (vm.wstring(str_ref), vm.wstring(words_ref)) {
        (Some(text), Some(words)) => words
            .split(|c| SEPARATORS.contains(c))
            .filter(|word| word.first().is_some_and(|&c| c != 0))
            .any(|word| wcsicmp(word, text) == 0),
        _ => false,
    };
    Call::new(ctx).ret(4, u32::from(found), INT_RESULT);
    InterpStatus::Success
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain([0]).collect()
    }

    fn found(text: &str, words: &str) -> bool {
        let text = wide(text);
        let words = wide(words);
        words
            .split(|c| SEPARATORS.contains(c))
            .filter(|word| word.first().is_some_and(|&c| c != 0))
            .any(|word| wcsicmp(word, &text) == 0)
    }

    #[test]
    fn a_word_is_one_of_the_separated_words_without_case() {
        assert!(found("afk", "Yaraq afk, busy;away"));
        assert!(found("AFK", "afk"));
        assert!(found("busy", "afk, busy"));
        assert!(!found("bus", "afk, busy"));
        assert!(!found("afk", ""));
        assert!(!found("afk", " ,; "));
    }
}
