//! `StringTableMetaLanguage`'s grammar (`stringmeta_parse`, a bison LALR(1) parser in
//! `metalanguage_grammar.obj`) and its rule actions (`metalanguage_rules.cpp`).
//!
//! The bison tables accept exactly this grammar (checked by running them):
//!
//! ```text
//! start   : ε | items          items   : item | items item
//! item    : text | var | block | INFO                     (INFO: r_OutsideInfoBlock)
//! var     : VAR | '^' VAR | NUMBER VAR | NUMBER '^' VAR   (r_Var(value, label, capitalize))
//! block   : '{' choices '}' | NUMBER '{' choices '}'      (r_ChoiceBlockBegin / End)
//! choices : choice | choices '|' choice
//! choice  : text | text FLAGS                              (r_Flags on the text)
//! text    : TEXT                                           (r_Text)
//! ```
//!
//! The block's begin action is a default reduction right after `{`, and its end one right
//! after `}`, so no lookahead is read across a change of the tokenizer's choice-block state;
//! this recursive-descent form reads tokens at the same points. Any syntax error, rule error
//! or duplicate default choice fails the whole render (`curError`), so bison's error recovery
//! has no visible effect and is not reproduced.

use crate::string_meta_lex::{wcstol, Lexer, Token, BAR, BRACE_CLOSE, BRACE_OPEN, CARET};
use crate::string_meta_text::{find_skip_escapes, trim, WHITESPACE};
use crate::string_state::{
    female_meta_letters, is_number_singular, male_meta_letters, StringState,
};

const BACKSLASH: u16 = b'\\' as u16;

/// `StringTableMetaLanguage::NodeType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NodeKind {
    Text,
    Var,
    ChoiceBlock,
    /// `ntOutsideVar`: an `{{...}}` block (`item`, `keepspaces`).
    OutsideVar,
}

/// `StringTableMetaLanguage::Node`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Node {
    pub kind: NodeKind,
    pub text: Vec<u16>,
    pub children: Vec<Node>,
    /// The `#n:` label (`ID`; 0 unlabelled).
    pub id: i32,
    /// `bForceCapital` (`^`).
    pub force_capital: bool,
    /// A variable's meta letters or a choice's `[...]` letters.
    pub flags: Vec<u16>,
    /// The tokenizer position when the node was added, less one (error reports).
    pub pos: usize,
}

impl Node {
    fn new(kind: NodeKind) -> Self {
        Self {
            kind,
            text: Vec::new(),
            children: Vec::new(),
            id: 0,
            force_capital: false,
            flags: Vec::new(),
            pos: 0,
        }
    }
}

/// `RenderErrorCode` (`recSuccess` is the `Ok` side).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderErrorCode {
    SyntaxError,
    DuplicateIds,
    DuplicateChoices,
    InternalError,
}

/// `RenderErr`: what failed and where.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderErr {
    pub code: RenderErrorCode,
    pub pos: usize,
    pub message: &'static str,
}

/// The parser state: `g_Root`, `g_curNode` (the open choice block) and the tokenizer.
struct Parser<'a> {
    lexer: Lexer,
    root: Vec<Node>,
    block: Option<Node>,
    state: Option<&'a StringState>,
    lookahead: Option<Token>,
}

/// `stringmeta_parse` with `InitRules`: the root's children, or the first error.
pub(crate) fn parse(lexer: Lexer, state: Option<&StringState>) -> Result<Vec<Node>, RenderErr> {
    let mut parser = Parser {
        lexer,
        root: Vec::new(),
        block: None,
        state,
        lookahead: None,
    };
    parser.items()?;
    Ok(parser.root)
}

impl Parser<'_> {
    fn syntax_error(&self) -> RenderErr {
        RenderErr {
            code: RenderErrorCode::SyntaxError,
            pos: self.lexer.position(),
            message: "parse error",
        }
    }

    fn peek(&mut self) -> &Token {
        if self.lookahead.is_none() {
            let in_choice = self.block.is_some();
            self.lookahead = Some(self.lexer.next(in_choice));
        }
        self.lookahead.as_ref().expect("lookahead read")
    }

    fn take(&mut self) -> Token {
        self.peek();
        self.lookahead.take().expect("lookahead read")
    }

    /// `StringTableMetaLanguage::AddNode`: into the open block, or (for the block itself or
    /// outside one) into the root.
    fn add_node(&mut self, mut node: Node) {
        node.pos = self.lexer.position().saturating_sub(1);
        match &mut self.block {
            Some(block) => block.children.push(node),
            None => self.root.push(node),
        }
    }

    fn items(&mut self) -> Result<(), RenderErr> {
        loop {
            match self.take() {
                Token::End => return Ok(()),
                Token::Text(text) => self.r_text(text),
                Token::Info(text) => {
                    let mut node = Node::new(NodeKind::OutsideVar);
                    node.text = text;
                    self.add_node(node);
                }
                Token::Var(value) => self.r_var(&value, 0, false)?,
                Token::Char(CARET) => self.caret_var(0)?,
                Token::Char(BRACE_OPEN) => self.block(0)?,
                Token::Number(label) => match self.take() {
                    Token::Var(value) => self.r_var(&value, label, false)?,
                    Token::Char(CARET) => self.caret_var(label)?,
                    Token::Char(BRACE_OPEN) => self.block(label)?,
                    _ => return Err(self.syntax_error()),
                },
                _ => return Err(self.syntax_error()),
            }
        }
    }

    fn caret_var(&mut self, label: i32) -> Result<(), RenderErr> {
        match self.take() {
            Token::Var(value) => self.r_var(&value, label, true),
            _ => Err(self.syntax_error()),
        }
    }

    /// `r_Text`.
    fn r_text(&mut self, text: Vec<u16>) {
        let mut node = Node::new(NodeKind::Text);
        node.text = text;
        self.add_node(node);
    }

    /// A choice block after its `{`: `r_ChoiceBlockBegin`, the choices, `}`,
    /// `r_ChoiceBlockEnd`.
    fn block(&mut self, label: i32) -> Result<(), RenderErr> {
        let mut block = Node::new(NodeKind::ChoiceBlock);
        block.id = label;
        self.block = Some(block);
        loop {
            let Token::Text(text) = self.take() else {
                return Err(self.syntax_error());
            };
            self.r_text(text);
            if let Token::Flags(_) = self.peek() {
                let Token::Flags(flags) = self.take() else {
                    unreachable!("peeked flags")
                };
                // `r_Flags`: the last added node is the choice's text.
                if let Some(choice) = self.block.as_mut().and_then(|b| b.children.last_mut()) {
                    choice.flags = flags;
                }
            }
            match self.take() {
                Token::Char(BAR) => continue,
                Token::Char(BRACE_CLOSE) => break,
                _ => return Err(self.syntax_error()),
            }
        }
        let block = self.block.take().expect("open block");
        // `CheckForErrorsInChoiceBlock`: at most one choice without letters.
        let error = block
            .children
            .iter()
            .filter(|c| c.flags.is_empty())
            .nth(1)
            .map(|second| RenderErr {
                code: RenderErrorCode::DuplicateChoices,
                pos: second.pos,
                message: "multiple default choices in this block",
            });
        self.add_node(block);
        error.map_or(Ok(()), Err)
    }

    /// `StringTableMetaLanguage::r_Var(value, labelID, bForceCapitalize)`: the value before
    /// an unescaped `[` is the text and the bracket's letters its flags (`M` and `F` expand
    /// to the state's player letters plus the letter); a value without brackets that is a
    /// whole number gets the flag `1` when singular, then (Legions 7160 only, not in admin
    /// 7148) `+` when not negative.
    fn r_var(&mut self, value: &[u16], label: i32, capitalize: bool) -> Result<(), RenderErr> {
        let mut node = Node::new(NodeKind::Var);
        node.id = label;
        node.force_capital = capitalize;
        let Some(open) = find_skip_escapes(value, u16::from(b'['), BACKSLASH) else {
            node.text = value.to_vec();
            let (num, end) = wcstol(value);
            let whole = if end == 0 {
                value.is_empty()
            } else {
                end == value.len()
            };
            if whole && is_number_singular(self.state, num) {
                node.flags.push(u16::from(b'1'));
            }
            if whole && num >= 0 {
                node.flags.push(u16::from(b'+'));
            }
            self.add_node(node);
            return Ok(());
        };
        node.text = value[..open].to_vec();
        trim(&mut node.text, false, true, &WHITESPACE);
        let Some(close) = find_skip_escapes(value, u16::from(b']'), BACKSLASH) else {
            return Err(RenderErr {
                code: RenderErrorCode::SyntaxError,
                pos: self.lexer.position(),
                message: "variable-substitution string is missing ']'",
            });
        };
        for &c in value.get(open + 1..close).unwrap_or(&[]) {
            match c {
                0x4D => {
                    node.flags.extend(male_meta_letters(self.state));
                    node.flags.push(c);
                }
                0x46 => {
                    node.flags.extend(female_meta_letters(self.state));
                    node.flags.push(c);
                }
                0x20 => {}
                _ => node.flags.push(c),
            }
        }
        let trailing = value[close + 1..]
            .iter()
            .take_while(|&&c| c != 0)
            .any(|&c| c != 0x20 && c != 0x09);
        self.add_node(node);
        if trailing {
            return Err(RenderErr {
                code: RenderErrorCode::SyntaxError,
                pos: self.lexer.position(),
                message: "there is text after the ']' in a variable-substitution string",
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::string_meta_text::wide;

    fn parse_str(pieces: &[&str], values: &[&str]) -> Result<Vec<Node>, RenderErr> {
        let pieces: Vec<_> = pieces.iter().map(|p| Some(wide(p))).collect();
        let values: Vec<_> = values.iter().map(|v| wide(v)).collect();
        parse(Lexer::load(&pieces, &values), None)
    }

    #[test]
    fn blocks_collect_choices_with_flags() {
        let nodes = parse_str(&["#1:{he[m]|she[f]|it} ^", ""], &["Bob [M]"]).unwrap();
        let kinds: Vec<_> = nodes.iter().map(|n| n.kind).collect();
        assert_eq!(
            kinds,
            [NodeKind::ChoiceBlock, NodeKind::Text, NodeKind::Var]
        );
        assert_eq!(nodes[0].id, 1);
        let flags: Vec<_> = nodes[0].children.iter().map(|c| c.flags.clone()).collect();
        assert_eq!(flags, [wide("m"), wide("f"), vec![]]);
        assert_eq!(nodes[2].text, wide("Bob"));
        assert_eq!(nodes[2].flags, wide("mnM"));
        assert!(nodes[2].force_capital);
    }

    #[test]
    fn singular_numbers_flag_one_and_non_negative_ones_plus() {
        let nodes = parse_str(&["", ""], &["1"]).unwrap();
        assert_eq!(nodes[0].flags, wide("1+"));
        let nodes = parse_str(&["", ""], &["12"]).unwrap();
        assert_eq!(nodes[0].flags, wide("+"));
        let nodes = parse_str(&["", ""], &["-3"]).unwrap();
        assert!(nodes[0].flags.is_empty());
        let nodes = parse_str(&["", ""], &["1,000"]).unwrap();
        assert!(nodes[0].flags.is_empty());
    }

    #[test]
    fn grammar_and_rule_errors_fail() {
        for (pieces, values) in [
            (vec!["{}"], vec![]),
            (vec!["{a||b}"], vec![]),
            (vec!["{a[x]b}"], vec![]),
            (vec!["^x"], vec![]),
            (vec!["{a|b|c}"], vec![]),
            (vec!["", ""], vec!["Bob [M"]),
            (vec!["", ""], vec!["Bob [M] x"]),
            (vec!["{", "}"], vec!["v"]),
        ] {
            assert!(
                parse_str(&pieces, &values).is_err(),
                "{pieces:?} {values:?}"
            );
        }
        assert!(parse_str(&["{a|b[x]}"], &[]).is_ok());
        assert!(parse_str(&[""], &[]).unwrap().is_empty());
    }
}
