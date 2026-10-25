//! The parser for the AudioVBScript subset: `Dim` at module level, `Sub ... End Sub` routines,
//! and inside them assignments, `object.method(args)` calls, bare routine calls and
//! `If ... ElseIf ... End If` blocks. Expressions have comparisons, `+ - * /`, unary minus, numbers,
//! names, function calls and parentheses. Anything else is a parse error (not a silent skip).

use super::lexer::{lex, Spanned, Symbol, Token};
use super::ScriptError;

/// A binary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    Add,
    Sub,
    Mul,
    Div,
}

/// An expression. Names are lowercased (AudioVBScript names are case-insensitive).
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),
    Name(String),
    /// `name(args)`: a built-in function (`GetMasterGrooveLevel()`).
    Call {
        name: String,
        args: Vec<Expr>,
    },
    Negate(Box<Expr>),
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

/// A statement and the line it starts on.
#[derive(Debug, Clone, PartialEq)]
pub struct Stmt {
    pub line: usize,
    pub kind: StmtKind,
}

/// The statements of the subset.
#[derive(Debug, Clone, PartialEq)]
pub enum StmtKind {
    Assign {
        name: String,
        value: Expr,
    },
    /// `object.method(args)`, e.g. `MI_Hum.play(AtFinish)`.
    Method {
        object: String,
        method: String,
        args: Vec<Expr>,
    },
    /// A bare name: a routine of the same script is called.
    Routine(String),
    /// `If` / `ElseIf` branches, each a condition and its block. No `Else` (not in the corpus).
    If(Vec<(Expr, Vec<Stmt>)>),
}

/// A `Sub`: its name (as written) and body.
#[derive(Debug, Clone, PartialEq)]
pub struct Routine {
    pub name: String,
    pub body: Vec<Stmt>,
}

/// A parsed script: its module variables (`Dim`) and its routines.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Program {
    pub variables: Vec<String>,
    pub routines: Vec<Routine>,
}

impl Program {
    /// The routine named `name` (case-insensitive).
    pub fn routine(&self, name: &str) -> Option<&Routine> {
        self.routines
            .iter()
            .find(|r| r.name.eq_ignore_ascii_case(name))
    }
}

/// Parses a script's source.
pub fn parse(source: &str) -> Result<Program, ScriptError> {
    let mut parser = Parser {
        tokens: lex(source)?,
        at: 0,
    };
    let mut program = Program::default();
    loop {
        parser.skip_eols();
        let Some(word) = parser.peek_word() else {
            break;
        };
        match word.as_str() {
            "dim" => {
                parser.bump();
                loop {
                    let name = parser.word("a variable name")?;
                    program.variables.push(name);
                    if !parser.eat(&Token::Symbol(Symbol::Comma)) {
                        break;
                    }
                }
                parser.end_of_statement()?;
            }
            "sub" => {
                parser.bump();
                let name = parser.word("a routine name")?;
                parser.end_of_statement()?;
                let body = parser.block(&["end"])?;
                parser.expect_word("end")?;
                parser.expect_word("sub")?;
                parser.end_of_statement()?;
                program.routines.push(Routine { name, body });
            }
            _ => return Err(parser.error("only Dim and Sub at module level")),
        }
    }
    Ok(program)
}

struct Parser {
    tokens: Vec<Spanned>,
    at: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Spanned> {
        self.tokens.get(self.at)
    }

    fn token(&self) -> Option<&Token> {
        self.peek().map(|s| &s.token)
    }

    fn line(&self) -> usize {
        self.peek()
            .or_else(|| self.tokens.last())
            .map_or(0, |s| s.line)
    }

    fn bump(&mut self) {
        self.at += 1;
    }

    fn error(&self, message: &str) -> ScriptError {
        ScriptError {
            line: self.line(),
            message: message.to_string(),
        }
    }

    fn eat(&mut self, token: &Token) -> bool {
        if self.token() == Some(token) {
            self.bump();
            true
        } else {
            false
        }
    }

    /// The lowercased word at the cursor, without consuming it.
    fn peek_word(&self) -> Option<String> {
        match self.token()? {
            Token::Word(w) => Some(w.to_ascii_lowercase()),
            _ => None,
        }
    }

    fn word(&mut self, what: &str) -> Result<String, ScriptError> {
        match self.token() {
            Some(Token::Word(w)) => {
                let w = w.clone();
                self.bump();
                Ok(w)
            }
            _ => Err(self.error(&format!("expected {what}"))),
        }
    }

    fn expect_word(&mut self, keyword: &str) -> Result<(), ScriptError> {
        if self.peek_word().as_deref() == Some(keyword) {
            self.bump();
            Ok(())
        } else {
            Err(self.error(&format!("expected {keyword}")))
        }
    }

    fn expect(&mut self, symbol: Symbol) -> Result<(), ScriptError> {
        if self.eat(&Token::Symbol(symbol)) {
            Ok(())
        } else {
            Err(self.error(&format!("expected {symbol:?}")))
        }
    }

    fn skip_eols(&mut self) {
        while self.token() == Some(&Token::Eol) {
            self.bump();
        }
    }

    /// The end of a statement: a line break (or the end of the source).
    fn end_of_statement(&mut self) -> Result<(), ScriptError> {
        match self.token() {
            None => Ok(()),
            Some(Token::Eol) => {
                self.bump();
                Ok(())
            }
            Some(_) => Err(self.error("expected the end of the line")),
        }
    }

    /// Statements up to a line starting with one of `stops` (not consumed).
    fn block(&mut self, stops: &[&str]) -> Result<Vec<Stmt>, ScriptError> {
        let mut body = Vec::new();
        loop {
            self.skip_eols();
            match self.peek_word() {
                Some(word) if stops.contains(&word.as_str()) => return Ok(body),
                _ if self.peek().is_none() => {
                    return Err(self.error("the block is not closed"));
                }
                _ => body.push(self.statement()?),
            }
        }
    }

    fn statement(&mut self) -> Result<Stmt, ScriptError> {
        let line = self.line();
        let kind = match self.peek_word().as_deref() {
            Some("if") => self.if_statement()?,
            Some(other @ ("dim" | "sub" | "else" | "elseif" | "end")) => {
                return Err(self.error(&format!("{other} is not allowed here")));
            }
            _ => self.simple_statement()?,
        };
        self.end_of_statement()?;
        Ok(Stmt { line, kind })
    }

    fn simple_statement(&mut self) -> Result<StmtKind, ScriptError> {
        let name = self.word("a statement")?;
        match self.token() {
            Some(Token::Symbol(Symbol::Eq)) => {
                self.bump();
                let value = self.expr()?;
                Ok(StmtKind::Assign { name, value })
            }
            Some(Token::Symbol(Symbol::Dot)) => {
                self.bump();
                let method = self.word("a method name")?;
                self.expect(Symbol::LParen)?;
                let args = self.arguments()?;
                Ok(StmtKind::Method {
                    object: name,
                    method,
                    args,
                })
            }
            Some(Token::Eol) | None => Ok(StmtKind::Routine(name)),
            Some(_) => Err(self.error("unsupported statement")),
        }
    }

    fn if_statement(&mut self) -> Result<StmtKind, ScriptError> {
        self.expect_word("if")?;
        let mut branches = Vec::new();
        let condition = self.expr()?;
        self.expect_word("then")?;
        self.end_of_statement()?;
        branches.push((condition, self.block(&["elseif", "else", "end"])?));
        loop {
            match self.peek_word().as_deref() {
                Some("elseif") => {
                    self.bump();
                    let condition = self.expr()?;
                    self.expect_word("then")?;
                    self.end_of_statement()?;
                    branches.push((condition, self.block(&["elseif", "else", "end"])?));
                }
                Some("else") => return Err(self.error("Else is not in the supported subset")),
                _ => break,
            }
        }
        self.expect_word("end")?;
        self.expect_word("if")?;
        Ok(StmtKind::If(branches))
    }

    /// The arguments of a call, after its `(`, up to and including the `)`.
    fn arguments(&mut self) -> Result<Vec<Expr>, ScriptError> {
        let mut args = Vec::new();
        if self.eat(&Token::Symbol(Symbol::RParen)) {
            return Ok(args);
        }
        loop {
            args.push(self.expr()?);
            if self.eat(&Token::Symbol(Symbol::RParen)) {
                return Ok(args);
            }
            self.expect(Symbol::Comma)?;
        }
    }

    fn expr(&mut self) -> Result<Expr, ScriptError> {
        let mut left = self.additive()?;
        loop {
            let op = match self.token() {
                Some(Token::Symbol(Symbol::Eq)) => BinOp::Eq,
                Some(Token::Symbol(Symbol::Ne)) => BinOp::Ne,
                Some(Token::Symbol(Symbol::Lt)) => BinOp::Lt,
                Some(Token::Symbol(Symbol::Gt)) => BinOp::Gt,
                Some(Token::Symbol(Symbol::Le)) => BinOp::Le,
                Some(Token::Symbol(Symbol::Ge)) => BinOp::Ge,
                _ => return Ok(left),
            };
            self.bump();
            let right = self.additive()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
    }

    fn additive(&mut self) -> Result<Expr, ScriptError> {
        let mut left = self.multiplicative()?;
        loop {
            let op = match self.token() {
                Some(Token::Symbol(Symbol::Plus)) => BinOp::Add,
                Some(Token::Symbol(Symbol::Minus)) => BinOp::Sub,
                _ => return Ok(left),
            };
            self.bump();
            let right = self.multiplicative()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
    }

    fn multiplicative(&mut self) -> Result<Expr, ScriptError> {
        let mut left = self.unary()?;
        loop {
            let op = match self.token() {
                Some(Token::Symbol(Symbol::Star)) => BinOp::Mul,
                Some(Token::Symbol(Symbol::Slash)) => BinOp::Div,
                _ => return Ok(left),
            };
            self.bump();
            let right = self.unary()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
    }

    fn unary(&mut self) -> Result<Expr, ScriptError> {
        if self.eat(&Token::Symbol(Symbol::Minus)) {
            return Ok(Expr::Negate(Box::new(self.unary()?)));
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, ScriptError> {
        match self.token().cloned() {
            Some(Token::Number(n)) => {
                self.bump();
                Ok(Expr::Number(n))
            }
            Some(Token::Word(w)) => {
                self.bump();
                if self.eat(&Token::Symbol(Symbol::LParen)) {
                    let args = self.arguments()?;
                    Ok(Expr::Call {
                        name: w.to_ascii_lowercase(),
                        args,
                    })
                } else {
                    Ok(Expr::Name(w.to_ascii_lowercase()))
                }
            }
            Some(Token::Symbol(Symbol::LParen)) => {
                self.bump();
                let inner = self.expr()?;
                self.expect(Symbol::RParen)?;
                Ok(inner)
            }
            _ => Err(self.error("expected an expression")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_routine_with_an_if_chain_parses_to_its_statements() {
        let source = "Dim level\n\
            Sub ChooseGroove\n\
            level = GetMasterGrooveLevel()\n\
            If level < 5 Then\n\
            Seg_A.play(AtFinish)\n\
            ElseIf level > 4 Then\n\
            Seg_B.play(AtFinish)\n\
            End If\n\
            End Sub\n";
        let program = parse(source).unwrap();
        assert_eq!(program.variables, vec!["level".to_string()]);
        let routine = program.routine("choosegroove").expect("the routine");
        assert_eq!(routine.body.len(), 2);
        assert_eq!(
            routine.body[0].kind,
            StmtKind::Assign {
                name: "level".into(),
                value: Expr::Call {
                    name: "getmastergroovelevel".into(),
                    args: vec![],
                },
            }
        );
        let StmtKind::If(branches) = &routine.body[1].kind else {
            panic!("an If");
        };
        assert_eq!(branches.len(), 2);
        assert!(matches!(
            branches[1].1[0].kind,
            StmtKind::Method { ref method, .. } if method == "play"
        ));
    }

    #[test]
    fn precedence_puts_multiplication_under_addition_and_comparison_last() {
        let program = parse("Sub R\nx = 1 + 2 * 3 < 9\nEnd Sub\n").unwrap();
        let StmtKind::Assign { value, .. } = &program.routines[0].body[0].kind else {
            panic!("an assignment");
        };
        let Expr::Binary {
            op: BinOp::Lt,
            left,
            ..
        } = value
        else {
            panic!("a comparison at the top");
        };
        let Expr::Binary {
            op: BinOp::Add,
            right,
            ..
        } = left.as_ref()
        else {
            panic!("an addition under it");
        };
        assert!(matches!(
            right.as_ref(),
            Expr::Binary { op: BinOp::Mul, .. }
        ));
    }

    #[test]
    fn a_bare_name_is_a_routine_call_and_names_are_case_insensitive() {
        let program = parse("Sub A\nchoosegroove\nEnd Sub\nSub ChooseGroove\nEnd Sub\n").unwrap();
        assert_eq!(
            program.routines[0].body[0].kind,
            StmtKind::Routine("choosegroove".into())
        );
        assert!(program.routine("CHOOSEGROOVE").is_some());
    }

    #[test]
    fn an_else_is_rejected_as_outside_the_subset() {
        let err = parse("Sub R\nIf x > 1 Then\ny = 1\nElse\ny = 2\nEnd If\nEnd Sub\n").unwrap_err();
        assert!(err.message.contains("Else"), "{err}");
    }

    #[test]
    fn a_statement_outside_a_routine_is_rejected() {
        let err = parse("x = 1\n").unwrap_err();
        assert_eq!(err.line, 1);
    }

    #[test]
    fn an_unclosed_routine_is_rejected() {
        assert!(parse("Sub R\ny = 1\n").is_err());
    }
}
