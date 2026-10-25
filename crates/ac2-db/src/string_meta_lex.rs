//! `StringTableMetaLanguage`'s tokenizer (`metalanguage_tokenizer.cpp`): `LoadTokenizer`
//! joins a string's pieces with each variable's value between two 0x01 units, and
//! `_stringmeta_lex` cuts that into the grammar's tokens. Which characters are special depends
//! on whether the parser is inside a choice block (`IsInChoiceBlock`), so the parser passes
//! that state with every call.

use crate::string_meta_text::{at, is_escape_char};

/// The unit `LoadTokenizer` puts around each variable's value.
const VAR_MARK: u16 = 1;
const SPACE: u16 = b' ' as u16;
const TAB: u16 = b'\t' as u16;
const BACKSLASH: u16 = b'\\' as u16;
const HASH: u16 = b'#' as u16;
const MINUS: u16 = b'-' as u16;
const COLON: u16 = b':' as u16;
const OPEN: u16 = b'[' as u16;
const CLOSE: u16 = b']' as u16;
pub(crate) const BRACE_OPEN: u16 = b'{' as u16;
pub(crate) const BRACE_CLOSE: u16 = b'}' as u16;
pub(crate) const BAR: u16 = b'|' as u16;
pub(crate) const CARET: u16 = b'^' as u16;

/// A token (`stringmeta_lex`'s return value and `stringmeta_lval`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Token {
    /// End of input (-1, read as 0).
    End,
    /// 0x101: plain text, escapes kept except `\q`, which becomes `"`.
    Text(Vec<u16>),
    /// 0x102: a variable's value.
    Var(Vec<u16>),
    /// 0x103: a `#n:` label.
    Number(i32),
    /// 0x104: a choice's `[...]` letters, spaces removed.
    Flags(Vec<u16>),
    /// 0x105: an `{{...}}` block outside choice blocks.
    Info(Vec<u16>),
    /// `{` or `^` outside a choice block, `}` or `|` inside one.
    Char(u16),
}

/// `g_input` and `g_inputidx`.
pub(crate) struct Lexer {
    input: Vec<u16>,
    idx: usize,
}

/// `wcstol(s, &end, 10)` on a 32-bit `long`: leading whitespace, a sign, digits; saturates.
/// Returns the value and where parsing stopped (0 when nothing converted).
pub(crate) fn wcstol(s: &[u16]) -> (i32, usize) {
    let mut i = 0;
    while matches!(at(s, i), 0x09..=0x0D | 0x20) {
        i += 1;
    }
    let negative = at(s, i) == MINUS;
    if negative || at(s, i) == u16::from(b'+') {
        i += 1;
    }
    let first = i;
    let mut value: i64 = 0;
    while is_digit(at(s, i)) {
        value = (value * 10 + i64::from(at(s, i) - u16::from(b'0'))).min(1 << 32);
        i += 1;
    }
    if i == first {
        return (0, 0);
    }
    let value = if negative { -value } else { value };
    (
        value.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32,
        i,
    )
}

/// `iswdigit` in the "C" locale, for the digits `wcstol` reads.
fn is_digit(c: u16) -> bool {
    (u16::from(b'0')..=u16::from(b'9')).contains(&c)
}

impl Lexer {
    /// `StringTableMetaLanguage::LoadTokenizer(textparts, varparts)`: each piece, then (while
    /// there are values) 0x01, the value, 0x01.
    pub(crate) fn load(pieces: &[Option<Vec<u16>>], values: &[Vec<u16>]) -> Self {
        let mut input = Vec::new();
        for (i, piece) in pieces.iter().enumerate() {
            if let Some(piece) = piece {
                input.extend_from_slice(piece);
            }
            if let Some(value) = values.get(i) {
                input.push(VAR_MARK);
                input.extend_from_slice(value);
                input.push(VAR_MARK);
            }
        }
        Self { input, idx: 0 }
    }

    /// `StringTableMetaLanguage::GetCurrentStreamPosition`.
    pub(crate) fn position(&self) -> usize {
        self.idx
    }

    fn at(&self, i: usize) -> u16 {
        at(&self.input, i)
    }

    fn skip_blanks(&self, mut i: usize) -> usize {
        while matches!(self.at(i), SPACE | TAB) {
            i += 1;
        }
        i
    }

    /// `IsSpecialChar`.
    fn is_special(c: u16, in_choice: bool) -> bool {
        match c {
            BRACE_CLOSE | BAR => in_choice,
            BRACE_OPEN | CARET => !in_choice,
            _ => false,
        }
    }

    /// `IsNumberLabel`: outside choice blocks, `#`, an optionally negative number, `:`, and
    /// then a variable, `^` or `{`. Returns the label and where its token ends.
    fn number_label(&self, idx: usize, in_choice: bool) -> Option<(i32, usize)> {
        if in_choice || self.at(idx) != HASH {
            return None;
        }
        let mut e = self.skip_blanks(idx + 1);
        if self.at(e) != MINUS && !is_digit(self.at(e)) {
            return None;
        }
        let mut digits = Vec::new();
        if self.at(e) == MINUS {
            digits.push(MINUS);
            e += 1;
        }
        e = self.skip_blanks(e);
        if !is_digit(self.at(e)) {
            return None;
        }
        while is_digit(self.at(e)) && digits.len() < 1023 {
            digits.push(self.at(e));
            e += 1;
        }
        let (value, _) = wcstol(&digits);
        e = self.skip_blanks(e);
        if self.at(e) != COLON {
            return None;
        }
        e = self.skip_blanks(e + 1);
        matches!(self.at(e), VAR_MARK | CARET | BRACE_OPEN).then_some((value, e))
    }

    /// `IsFlagsBlock`: inside a choice block, `[` up to a closing `]`.
    fn flags_block(&self, idx: usize, in_choice: bool) -> Option<(Vec<u16>, usize)> {
        if !in_choice || self.at(idx) != OPEN {
            return None;
        }
        let mut e = idx + 1;
        let mut content = Vec::new();
        while self.at(e) != CLOSE && e < self.input.len() {
            content.push(self.at(e));
            e += 1;
        }
        (self.at(e) == CLOSE).then_some((content, e + 1))
    }

    /// `_stringmeta_lex`.
    pub(crate) fn next(&mut self, in_choice: bool) -> Token {
        let len = self.input.len();
        if self.idx >= len {
            return Token::End;
        }
        let c = self.at(self.idx);
        if c == VAR_MARK {
            let start = self.idx + 1;
            let mut end = start;
            while end < len && self.at(end) != VAR_MARK {
                end += 1;
            }
            self.idx = end + 1;
            return Token::Var(self.input[start.min(len)..end.min(len)].to_vec());
        }
        if c == BRACE_OPEN && self.at(self.idx + 1) == BRACE_OPEN && !in_choice {
            let start = self.idx + 2;
            let mut end = start;
            while !(self.at(end) == BRACE_CLOSE && self.at(end + 1) == BRACE_CLOSE)
                && self.at(end) != 0
            {
                end += 1;
            }
            self.idx = end;
            if self.at(end) == BRACE_CLOSE && self.at(end + 1) == BRACE_CLOSE {
                self.idx = end + 2;
            }
            return Token::Info(self.input[start.min(len)..end.min(len)].to_vec());
        }
        if Self::is_special(c, in_choice) {
            self.idx += 1;
            return Token::Char(c);
        }
        if let Some((value, end)) = self.number_label(self.idx, in_choice) {
            self.idx = end;
            return Token::Number(value);
        }
        if let Some((content, end)) = self.flags_block(self.idx, in_choice) {
            self.idx = self.skip_blanks(end);
            return Token::Flags(content.into_iter().filter(|&c| c != SPACE).collect());
        }
        Token::Text(self.text(in_choice))
    }

    /// The plain-text branch of `_stringmeta_lex`: up to a variable, a special character, a
    /// number label or a flags block.
    fn text(&mut self, in_choice: bool) -> Vec<u16> {
        let len = self.input.len();
        let mut text = Vec::new();
        while self.idx < len {
            let c = self.at(self.idx);
            if c == VAR_MARK
                || Self::is_special(c, in_choice)
                || self.number_label(self.idx, in_choice).is_some()
                || self.flags_block(self.idx, in_choice).is_some()
            {
                break;
            }
            let next = self.at(self.idx + 1);
            if c == BACKSLASH && is_escape_char(next) {
                text.extend_from_slice(&[BACKSLASH, next]);
                self.idx += 2;
            } else if c == BACKSLASH && next == u16::from(b'q') {
                text.push(u16::from(b'"'));
                self.idx += 2;
            } else {
                text.push(c);
                self.idx += 1;
            }
        }
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::string_meta_text::wide;

    fn tokens(pieces: &[&str], values: &[&str], in_choice_after_brace: bool) -> Vec<Token> {
        let pieces: Vec<_> = pieces.iter().map(|p| Some(wide(p))).collect();
        let values: Vec<_> = values.iter().map(|v| wide(v)).collect();
        let mut lexer = Lexer::load(&pieces, &values);
        let mut in_choice = false;
        let mut out = Vec::new();
        loop {
            let token = lexer.next(in_choice);
            if token == Token::End {
                return out;
            }
            match token {
                Token::Char(BRACE_OPEN) if in_choice_after_brace => in_choice = true,
                Token::Char(BRACE_CLOSE) => in_choice = false,
                _ => {}
            }
            out.push(token);
        }
    }

    #[test]
    fn labels_variables_choices_and_flags() {
        let got = tokens(
            &["#1:{The[!n]} #1:", " says, \\q#2:", "\\q"],
            &["Bob", "hi"],
            true,
        );
        assert_eq!(
            got,
            vec![
                Token::Number(1),
                Token::Char(BRACE_OPEN),
                Token::Text(wide("The")),
                Token::Flags(wide("!n")),
                Token::Char(BRACE_CLOSE),
                Token::Text(wide(" ")),
                Token::Number(1),
                Token::Var(wide("Bob")),
                Token::Text(wide(" says, \"")),
                Token::Number(2),
                Token::Var(wide("hi")),
                Token::Text(wide("\"")),
            ]
        );
    }

    #[test]
    fn specials_depend_on_the_choice_state_and_escapes_stay() {
        // Outside a block `|` and `}` are text; `\[` stays escaped.
        assert_eq!(
            tokens(&["a|b} \\[x\\]"], &[], false),
            vec![Token::Text(wide("a|b} \\[x\\]"))]
        );
        // A `#` that is not a label is text.
        assert_eq!(
            tokens(&["We're #"], &["3"], false),
            vec![Token::Text(wide("We're #")), Token::Var(wide("3"))]
        );
        assert_eq!(
            tokens(&["{{keepspaces}}x"], &[], false),
            vec![Token::Info(wide("keepspaces")), Token::Text(wide("x"))]
        );
        assert_eq!(
            tokens(&["# -2 :^"], &["v"], false),
            vec![Token::Number(-2), Token::Char(CARET), Token::Var(wide("v"))]
        );
    }

    #[test]
    fn wcstol_saturates_and_reports_its_end() {
        assert_eq!(wcstol(&wide("  42x")), (42, 4));
        assert_eq!(wcstol(&wide("-7")), (-7, 2));
        assert_eq!(wcstol(&wide("x")), (0, 0));
        assert_eq!(wcstol(&wide("99999999999")).0, i32::MAX);
    }
}
