//! The lexer for the AudioVBScript subset (see [`super`]). Words are case-insensitive (the parser
//! compares them lowercased); a `'` starts a comment to the end of the line; a line break ends a
//! statement.

use super::ScriptError;

/// One token. Words are kept as written; keywords are recognised by the parser.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Word(String),
    Number(f64),
    Symbol(Symbol),
    /// The end of a line (a statement's end).
    Eol,
}

/// The operators and punctuation the subset uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Symbol {
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    Dot,
    Comma,
}

/// A token and the line it is on (1-based).
#[derive(Debug, Clone, PartialEq)]
pub struct Spanned {
    pub token: Token,
    pub line: usize,
}

/// Splits `source` into tokens. A character outside the subset is an error.
pub fn lex(source: &str) -> Result<Vec<Spanned>, ScriptError> {
    let mut out = Vec::new();
    for (index, text) in source.lines().enumerate() {
        let line = index + 1;
        lex_line(text, line, &mut out)?;
        out.push(Spanned {
            token: Token::Eol,
            line,
        });
    }
    Ok(out)
}

fn lex_line(text: &str, line: usize, out: &mut Vec<Spanned>) -> Result<(), ScriptError> {
    let chars: Vec<char> = text.chars().collect();
    let mut at = 0;
    while at < chars.len() {
        let c = chars[at];
        if c.is_whitespace() {
            at += 1;
            continue;
        }
        if c == '\'' {
            break;
        }
        let (token, next) = if c.is_ascii_alphabetic() || c == '_' {
            let end = scan(&chars, at, |c| c.is_ascii_alphanumeric() || *c == '_');
            (Token::Word(chars[at..end].iter().collect()), end)
        } else if c.is_ascii_digit() {
            let mut end = scan(&chars, at, |c| c.is_ascii_digit());
            if chars.get(end) == Some(&'.') && chars.get(end + 1).is_some_and(char::is_ascii_digit)
            {
                end = scan(&chars, end + 1, |c| c.is_ascii_digit());
            }
            let text: String = chars[at..end].iter().collect();
            let value = text.parse::<f64>().map_err(|_| ScriptError {
                line,
                message: format!("bad number {text}"),
            })?;
            (Token::Number(value), end)
        } else {
            let two: String = chars[at..(at + 2).min(chars.len())].iter().collect();
            let (symbol, width) = match two.as_str() {
                "<=" => (Symbol::Le, 2),
                ">=" => (Symbol::Ge, 2),
                "<>" => (Symbol::Ne, 2),
                _ => match c {
                    '=' => (Symbol::Eq, 1),
                    '<' => (Symbol::Lt, 1),
                    '>' => (Symbol::Gt, 1),
                    '+' => (Symbol::Plus, 1),
                    '-' => (Symbol::Minus, 1),
                    '*' => (Symbol::Star, 1),
                    '/' => (Symbol::Slash, 1),
                    '(' => (Symbol::LParen, 1),
                    ')' => (Symbol::RParen, 1),
                    '.' => (Symbol::Dot, 1),
                    ',' => (Symbol::Comma, 1),
                    other => {
                        return Err(ScriptError {
                            line,
                            message: format!("unsupported character {other:?}"),
                        })
                    }
                },
            };
            (Token::Symbol(symbol), at + width)
        };
        out.push(Spanned { token, line });
        at = next;
    }
    Ok(())
}

fn scan(chars: &[char], from: usize, keep: impl Fn(&char) -> bool) -> usize {
    let mut end = from;
    while end < chars.len() && keep(&chars[end]) {
        end += 1;
    }
    end
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens(source: &str) -> Vec<Token> {
        lex(source).unwrap().into_iter().map(|s| s.token).collect()
    }

    #[test]
    fn words_numbers_and_symbols_split_and_lines_end_statements() {
        assert_eq!(
            tokens("If level <= 4.5 Then"),
            vec![
                Token::Word("If".into()),
                Token::Word("level".into()),
                Token::Symbol(Symbol::Le),
                Token::Number(4.5),
                Token::Word("Then".into()),
                Token::Eol,
            ]
        );
    }

    #[test]
    fn a_method_call_is_word_dot_word_parens() {
        assert_eq!(
            tokens("MI_Hum.play(AtFinish)"),
            vec![
                Token::Word("MI_Hum".into()),
                Token::Symbol(Symbol::Dot),
                Token::Word("play".into()),
                Token::Symbol(Symbol::LParen),
                Token::Word("AtFinish".into()),
                Token::Symbol(Symbol::RParen),
                Token::Eol,
            ]
        );
    }

    #[test]
    fn a_quote_starts_a_comment_to_the_end_of_the_line() {
        assert_eq!(tokens("' a comment"), vec![Token::Eol]);
        assert_eq!(
            tokens("x = 1 ' trailing"),
            vec![
                Token::Word("x".into()),
                Token::Symbol(Symbol::Eq),
                Token::Number(1.0),
                Token::Eol,
            ]
        );
    }

    #[test]
    fn a_character_outside_the_subset_is_an_error() {
        let err = lex("x = \"text\"").unwrap_err();
        assert_eq!(err.line, 1);
    }
}
