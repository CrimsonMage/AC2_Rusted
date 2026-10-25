//! `StringTableMetaLanguage`'s text utilities (`metalanguage_utils.cpp`) and the `PStringBase`
//! / `PSUtils` helpers they use, over UTF-16 code units without the terminator.
//!
//! Retail's `PStringBase::operator[]` returns the terminator for an index past the end; [`at`]
//! does the same with 0. The client never calls `setlocale`, so the CRT's wide character
//! classes are the "C" locale's: `towupper` maps only `a`-`z`.

/// `PStringBase<unsigned short>::whitespace_string`: space, LF, CR, tab, form feed.
pub const WHITESPACE: [u16; 5] = [0x20, 0x0A, 0x0D, 0x09, 0x0C];

/// The meta-language's special characters (`IsEscapeChar`, `EscapeString`).
const META_CHARS: [u16; 9] = [
    b'[' as u16,
    b']' as u16,
    b'!' as u16,
    b'{' as u16,
    b'}' as u16,
    b'#' as u16,
    b'\\' as u16,
    b'|' as u16,
    b'^' as u16,
];
const BACKSLASH: u16 = b'\\' as u16;
const QUOTE_LETTER: u16 = b'q' as u16;
const QUOTE: u16 = b'"' as u16;
const OPEN: u16 = b'[' as u16;
const CLOSE: u16 = b']' as u16;

/// `PStringBase::operator[]`: the unit at `i`, or the terminator (0) past the end.
pub(crate) fn at(s: &[u16], i: usize) -> u16 {
    s.get(i).copied().unwrap_or(0)
}

/// UTF-16 of an ASCII literal.
pub(crate) fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

/// `StringTableMetaLanguage::IsEscapeChar`: one of `[]!{}#\|^`.
pub fn is_escape_char(c: u16) -> bool {
    c != 0 && META_CHARS.contains(&c)
}

/// `PSUtils::wcschr_skipescapes` (`search_skipescapes`): the first `c` not directly after an
/// `escape` unit. Retail only looks at the previous unit, so in `\\[` the `[` still counts as
/// escaped. Stops at an embedded terminator, as the C string search does.
pub(crate) fn find_skip_escapes(s: &[u16], c: u16, escape: u16) -> Option<usize> {
    let mut escaped = false;
    for (i, &unit) in s.iter().enumerate() {
        if unit == 0 {
            return None;
        }
        if unit == c && !escaped {
            return Some(i);
        }
        escaped = unit == escape;
    }
    None
}

/// `PStringBase<unsigned short>::trim(bLeading, bTrailing, filter)`. The trailing scan uses
/// `wcschr(filter, c)`, which also matches an embedded terminator.
pub(crate) fn trim(s: &mut Vec<u16>, leading: bool, trailing: bool, filter: &[u16]) {
    let mut start = 0;
    if leading {
        while start < s.len() && s[start] != 0 && filter.contains(&s[start]) {
            start += 1;
        }
    }
    let mut end = s.len();
    if trailing {
        while end > start && (s[end - 1] == 0 || filter.contains(&s[end - 1])) {
            end -= 1;
        }
    }
    s.truncate(end);
    s.drain(..start);
}

/// `StringTableMetaLanguage::EscapeString`: a backslash before each special character.
pub fn escape_string(s: &[u16]) -> Vec<u16> {
    let mut out = Vec::with_capacity(s.len());
    for &c in s {
        if is_escape_char(c) {
            out.push(BACKSLASH);
        }
        out.push(c);
    }
    out
}

/// `StringTableMetaLanguage::UnescapeString`: `\q` becomes `"`, `\x` becomes `x`. A trailing
/// backslash takes the terminator (a 0 unit), as retail's indexing does.
pub fn unescape_string(s: &[u16]) -> Vec<u16> {
    let mut out = Vec::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        if s[i] == BACKSLASH {
            let next = at(s, i + 1);
            out.push(if next == QUOTE_LETTER { QUOTE } else { next });
            i += 2;
        } else {
            out.push(s[i]);
            i += 1;
        }
    }
    out
}

/// `StringTableMetaLanguage::StripMetaLetters`: cut at the first unescaped `[`, then trim
/// trailing whitespace.
pub fn strip_meta_letters(s: &[u16]) -> Vec<u16> {
    let mut out = s.to_vec();
    if let Some(cut) = find_skip_escapes(s, OPEN, BACKSLASH) {
        out.truncate(cut);
        trim(&mut out, false, true, &WHITESPACE);
    }
    out
}

/// `StringTableMetaLanguage::RetrieveMetaLetters`: `(body, letters)`; the body is the text
/// before the first `[` (trailing whitespace trimmed), the letters what follows it up to `]`.
pub fn retrieve_meta_letters(s: &[u16]) -> (Vec<u16>, Vec<u16>) {
    let Some(open) = s.iter().position(|&c| c == OPEN) else {
        return (s.to_vec(), Vec::new());
    };
    let mut body = s[..open].to_vec();
    let rest = &s[open + 1..];
    let letters = match rest.iter().position(|&c| c == CLOSE) {
        Some(close) => rest[..close].to_vec(),
        None => rest.to_vec(),
    };
    trim(&mut body, false, true, &WHITESPACE);
    (body, letters)
}

/// `PSUtils::TrimExcessSpaces`: each run of spaces and tabs keeps only its first unit.
fn collapse_spaces(s: &[u16]) -> Vec<u16> {
    let mut out = Vec::with_capacity(s.len());
    let mut in_run = false;
    for &c in s {
        let blank = c == 0x20 || c == 0x09;
        if !(blank && in_run) {
            out.push(c);
        }
        in_run = blank;
    }
    out
}

/// `StringTableMetaLanguage::TrimExcessSpaces`: leading whitespace trimmed, then runs of
/// spaces and tabs collapsed. Trailing whitespace stays.
pub fn trim_excess_spaces(s: &mut Vec<u16>) {
    trim(s, true, false, &WHITESPACE);
    *s = collapse_spaces(s);
}

/// The CRT's `iswspace` in the "C" locale (the `_wctype` table below 256).
fn is_wspace(c: u16) -> bool {
    match c {
        0x09..=0x0D | 0x20 | 0xA0 => true,
        0..=0xFF => false,
        _ => char::from_u32(u32::from(c)).is_some_and(char::is_whitespace),
    }
}

/// The CRT's `iswpunct` in the "C" locale: ASCII punctuation, the Latin-1 signs, and the
/// General Punctuation block above 256.
fn is_wpunct(c: u16) -> bool {
    match c {
        0x21..=0x2F | 0x3A..=0x40 | 0x5B..=0x60 | 0x7B..=0x7E => true,
        0xAA | 0xB2 | 0xB3 | 0xB5 | 0xB9 | 0xBA => false,
        0xA1..=0xBF | 0xD7 | 0xF7 => true,
        0x2010..=0x2027 | 0x2030..=0x205E | 0x3001..=0x3003 | 0x3008..=0x3011 => true,
        _ => false,
    }
}

/// `towupper` in the "C" locale: only `a`-`z` change.
fn to_upper(c: u16) -> u16 {
    if (u16::from(b'a')..=u16::from(b'z')).contains(&c) {
        c - 0x20
    } else {
        c
    }
}

/// `StringTableMetaLanguage::CapitalizeFirstLetter`: the first unit that is not whitespace
/// or punctuation (`¿` and `¡` count as punctuation; `\`, `/` and `@` stop the search) goes
/// through `towupper`.
pub fn capitalize_first_letter(s: &[u16]) -> Vec<u16> {
    let mut out = s.to_vec();
    for c in out.iter_mut() {
        let letter = !is_wspace(*c) && !is_wpunct(*c) && *c != 0xBF && *c != 0xA1;
        if letter || matches!(*c, 0x5C | 0x2F | 0x40) {
            *c = to_upper(*c);
            break;
        }
    }
    out
}

/// `CharUtils::IsVowel`.
fn is_vowel(c: u16) -> bool {
    matches!(
        char::from_u32(u32::from(c)),
        Some(
            'A' | 'E'
                | 'I'
                | 'O'
                | 'U'
                | 'a'
                | 'e'
                | 'i'
                | 'o'
                | 'u'
                | 'À'
                | 'Á'
                | 'Ä'
                | 'È'
                | 'É'
                | 'Ö'
                | 'Ú'
                | 'Ü'
                | 'à'
                | 'á'
                | 'ä'
                | 'è'
                | 'é'
                | 'ö'
                | 'ù'
                | 'ü'
        )
    )
}

/// `StringTableMetaLanguage::ApplyMetaTagsToName` (`StringInfo::ApplyMetasToPlayerName`):
/// strip any meta letters, trim, then append ` [M]` or ` [F]` with `v` for a leading vowel.
/// Retail's plural test reads the terminator instead of the last letter
/// (`name[m_len - 1]`), so it never adds `s`; that is kept.
pub fn apply_meta_tags_to_name(name: &[u16], is_male: bool) -> Vec<u16> {
    let mut out = strip_meta_letters(name);
    trim(&mut out, true, true, &WHITESPACE);
    let mut letters = wide(if is_male { "M" } else { "F" });
    if is_vowel(at(&out, 0)) {
        letters.push(u16::from(b'v'));
    }
    out.extend(wide(" ["));
    out.extend(letters);
    out.push(CLOSE);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(w: &[u16]) -> String {
        String::from_utf16_lossy(w)
    }

    #[test]
    fn escapes_round_trip_and_quote_letters_unescape() {
        let raw = wide("a[b]{c|d}^#!\\");
        let escaped = escape_string(&raw);
        assert_eq!(s(&escaped), "a\\[b\\]\\{c\\|d\\}\\^\\#\\!\\\\");
        assert_eq!(unescape_string(&escaped), raw);
        assert_eq!(s(&unescape_string(&wide("say \\qhi\\q"))), "say \"hi\"");
        assert_eq!(unescape_string(&wide("x\\")), vec![u16::from(b'x'), 0]);
    }

    #[test]
    fn meta_letters_strip_at_the_first_unescaped_bracket() {
        assert_eq!(s(&strip_meta_letters(&wide("Bob  [M]"))), "Bob");
        assert_eq!(s(&strip_meta_letters(&wide("a\\[b\\] [n]"))), "a\\[b\\]");
        // Retail only checks the previous unit: `\\[` still reads as escaped.
        assert_eq!(s(&strip_meta_letters(&wide("a\\\\[b"))), "a\\\\[b");
        let (body, letters) = retrieve_meta_letters(&wide("Sword  [mv]"));
        assert_eq!((s(&body), s(&letters)), ("Sword".into(), "mv".into()));
    }

    #[test]
    fn spaces_collapse_after_leading_trim() {
        let mut t = wide("  a  b\t\tc  ");
        trim_excess_spaces(&mut t);
        assert_eq!(s(&t), "a b\tc ");
    }

    #[test]
    fn the_first_letter_capitalizes_past_punctuation_in_the_c_locale() {
        assert_eq!(s(&capitalize_first_letter(&wide("  \"the"))), "  \"The");
        assert_eq!(s(&capitalize_first_letter(&wide("¿que"))), "¿Que");
        assert_eq!(s(&capitalize_first_letter(&wide("/the"))), "/the");
        assert_eq!(s(&capitalize_first_letter(&wide("élan"))), "élan");
    }

    #[test]
    fn names_get_gender_and_vowel_letters_but_never_plural() {
        assert_eq!(
            s(&apply_meta_tags_to_name(&wide(" Explorer [x] "), true)),
            "Explorer [Mv]"
        );
        assert_eq!(
            s(&apply_meta_tags_to_name(&wide("Jess"), false)),
            "Jess [F]"
        );
    }
}
