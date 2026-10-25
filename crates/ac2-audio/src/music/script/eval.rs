//! The evaluator: runs a routine of a parsed [`Program`] against an [`Instance`] (the script's
//! module variables for one performance) and a [`ScriptHost`] (the built-ins and the objects the
//! script names).
//!
//! Semantics the subset follows (AudioVBScript, as far as the corpus shows; see [`super`]):
//! - a name is a constant when the host knows it, else a module variable; a variable never set
//!   reads as zero (VBScript's Empty in numeric use);
//! - `If` takes a comparison (a boolean) or a number (non-zero is true); `ElseIf` tests in order,
//!   the first true block runs;
//! - arithmetic takes numbers only (a boolean there is an error, not VBScript's coercion);
//! - a bare name runs the routine of that name in the same script; a missing routine is an error;
//! - a runaway script (a routine calling itself, or too many statements) is an error.

use std::collections::HashMap;

use super::parser::{BinOp, Expr, Program, Stmt, StmtKind};
use super::ScriptError;

/// Nested routine calls allowed.
const MAX_DEPTH: usize = 16;
/// Statements one call may run.
const MAX_STEPS: usize = 10_000;

/// A value: a number, or the result of a comparison.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    Number(f64),
    Bool(bool),
}

impl Value {
    fn number(self) -> Result<f64, String> {
        match self {
            Value::Number(n) => Ok(n),
            Value::Bool(_) => Err("a boolean is not a number in arithmetic".into()),
        }
    }

    fn truth(self) -> bool {
        match self {
            Value::Number(n) => n != 0.0,
            Value::Bool(b) => b,
        }
    }
}

/// What a script calls outside itself: its constants, its built-in functions and the methods of
/// the objects it names. Errors are messages; the interpreter adds the line.
pub trait ScriptHost {
    /// A named constant (`AtFinish`), if the host has one.
    fn constant(&self, name: &str) -> Option<f64>;
    /// A built-in function (`GetMasterGrooveLevel`) with its arguments.
    fn function(&mut self, name: &str, args: &[f64]) -> Result<f64, String>;
    /// A method of an object the script names (`MI_Hum.play(flags)`).
    fn method(&mut self, object: &str, method: &str, args: &[f64]) -> Result<(), String>;
}

/// One script's module variables (per performance: each performance runs its own copy).
#[derive(Debug, Clone, Default)]
pub struct Instance {
    variables: HashMap<String, Value>,
}

/// The state of one call.
struct Run<'a> {
    program: &'a Program,
    variables: &'a mut HashMap<String, Value>,
    host: &'a mut dyn ScriptHost,
    depth: usize,
    steps: usize,
}

impl Instance {
    /// Runs `routine` (case-insensitive) with `host`. The module's `Dim` names are created on
    /// first use, reading as zero.
    pub fn call(
        &mut self,
        program: &Program,
        routine: &str,
        host: &mut dyn ScriptHost,
    ) -> Result<(), ScriptError> {
        let mut run = Run {
            program,
            variables: &mut self.variables,
            host,
            depth: 0,
            steps: 0,
        };
        run.routine(routine, 0)
    }

    /// The value of a module variable (`None` until it is set or declared).
    pub fn variable(&self, name: &str) -> Option<Value> {
        self.variables.get(&name.to_ascii_lowercase()).copied()
    }
}

impl Run<'_> {
    fn routine(&mut self, name: &str, line: usize) -> Result<(), ScriptError> {
        let program = self.program;
        let routine = program
            .routine(name)
            .ok_or_else(|| fail(line, format!("routine {name} is not in the script")))?;
        if self.depth >= MAX_DEPTH {
            return Err(fail(line, "routines nested too deeply"));
        }
        self.depth += 1;
        let result = self.block(&routine.body);
        self.depth -= 1;
        result
    }

    fn block(&mut self, body: &[Stmt]) -> Result<(), ScriptError> {
        for stmt in body {
            self.steps += 1;
            if self.steps > MAX_STEPS {
                return Err(fail(stmt.line, "the routine runs too many statements"));
            }
            self.statement(stmt)?;
        }
        Ok(())
    }

    fn statement(&mut self, stmt: &Stmt) -> Result<(), ScriptError> {
        let line = stmt.line;
        match &stmt.kind {
            StmtKind::Assign { name, value } => {
                let value = self.eval(value).map_err(|m| fail(line, m))?;
                self.variables.insert(name.clone(), value);
            }
            StmtKind::Method {
                object,
                method,
                args,
            } => {
                let args = self.numbers(args).map_err(|m| fail(line, m))?;
                self.host
                    .method(object, method, &args)
                    .map_err(|m| fail(line, m))?;
            }
            StmtKind::Routine(name) => self.routine(name, line)?,
            StmtKind::If(branches) => {
                for (condition, body) in branches {
                    if self.eval(condition).map_err(|m| fail(line, m))?.truth() {
                        self.block(body)?;
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    fn numbers(&mut self, args: &[Expr]) -> Result<Vec<f64>, String> {
        args.iter().map(|a| self.eval(a)?.number()).collect()
    }

    fn eval(&mut self, expr: &Expr) -> Result<Value, String> {
        Ok(match expr {
            Expr::Number(n) => Value::Number(*n),
            Expr::Name(name) => {
                if let Some(constant) = self.host.constant(name) {
                    Value::Number(constant)
                } else {
                    self.variables
                        .get(name)
                        .copied()
                        .unwrap_or(Value::Number(0.0))
                }
            }
            Expr::Call { name, args } => {
                let args = self.numbers(args)?;
                Value::Number(self.host.function(name, &args)?)
            }
            Expr::Negate(inner) => Value::Number(-self.eval(inner)?.number()?),
            Expr::Binary { op, left, right } => {
                let left = self.eval(left)?;
                let right = self.eval(right)?;
                match op {
                    BinOp::Add => Value::Number(left.number()? + right.number()?),
                    BinOp::Sub => Value::Number(left.number()? - right.number()?),
                    BinOp::Mul => Value::Number(left.number()? * right.number()?),
                    BinOp::Div => {
                        let divisor = right.number()?;
                        if divisor == 0.0 {
                            return Err("division by zero".into());
                        }
                        Value::Number(left.number()? / divisor)
                    }
                    comparison => Value::Bool(compare(*comparison, left, right)?),
                }
            }
        })
    }
}

fn fail(line: usize, message: impl Into<String>) -> ScriptError {
    ScriptError {
        line,
        message: message.into(),
    }
}

fn compare(op: BinOp, left: Value, right: Value) -> Result<bool, String> {
    let (a, b) = (left.number()?, right.number()?);
    Ok(match op {
        BinOp::Eq => a == b,
        BinOp::Ne => a != b,
        BinOp::Lt => a < b,
        BinOp::Gt => a > b,
        BinOp::Le => a <= b,
        BinOp::Ge => a >= b,
        _ => return Err("not a comparison".into()),
    })
}
