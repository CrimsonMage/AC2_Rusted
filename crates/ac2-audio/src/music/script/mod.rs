//! AudioVBScript: the scripts a segment's Script track calls (`RIFF 'DMSC'`, `scla` = "AudioVBScript",
//! source in `scsr`). The DATs use one subset, which this module implements: a lexer ([`lexer`]),
//! a parser ([`parser`]), an evaluator ([`eval`]) and the binding of a segment's script calls to
//! the objects they name ([`bind`]). The queue runs the routines (`music_thread::scripts`).
//!
//! The corpus (14 scripts, checked on the DAT) uses: `Dim` at module level; one `Sub ChooseGroove`
//! per script; `x = GetMasterGrooveLevel()`; `If`/`ElseIf`/`End If` on `<` and `>` against integer
//! literals; and `<embedded segment>.play(AtFinish)` inside the blocks. No `Else`, `Select Case`,
//! `Rand`, `SetMasterGrooveLevel`, `Stop`, `Function` or tempo calls appear.
//!
//! Semantics, marked by source:
//! - DirectMusic documented (`DMUS_SEGF_FLAGS`, MSDN "DMUS_SEGF_FLAGS", the archived
//!   `IDirectMusicPerformance::PlaySegment` page): `DMUS_SEGF_QUEUE` (256) puts a primary segment at
//!   the end of the primary queue; `DMUS_SEGF_SECONDARY` (128) and `DMUS_SEGF_CONTROL` (512) are
//!   secondary segments; `DMUS_SEGF_GRID` (1<<11), `BEAT` (1<<12), `MEASURE` (1<<13) are boundaries.
//! - Inference: the constant `AtFinish` is `DMUS_SEGF_QUEUE`, that is "play after the current
//!   primary". The corpus needs no other value. The AudioVBScript constant table was not reachable
//!   from the archived docs, so this is the reading that fits the call and the `ChooseGroove`
//!   routine's intent (choose the next primary by the groove level).
//! - Inference: a primary segment played from a script with no `QUEUE` flag would stop the
//!   current primary in DirectMusic; the port's queue has no such path, so the subset rejects it.
//! - Inference: `GetMasterGrooveLevel()` is the performance's `GUID_PerfMasterGrooveLevel` (the
//!   queue's groove, one byte; `SoundManager::SetGrooveLevel` sets it).
//! - Port choice: a script call's time is the segment's logical time; its `flags` (0x2 in the
//!   corpus; `DMUS_IO_SCRIPTTRACKF_*` in DirectMusic, value unverified) are not used because every
//!   corpus call is at time 0.
//! - Port choice: variables are per performance, per script. A routine's `Dim` names are module
//!   variables; VBScript locals are not in the subset.

pub mod bind;
pub mod eval;
pub mod lexer;
pub mod parser;

pub use bind::{bind_segment, BoundCall, BoundScript, BoundSegment, BoundTarget, BoundTrigger};
pub use eval::{Instance, ScriptHost, Value};
pub use parser::{parse, Program};

use std::fmt;

/// A script that does not parse, or a routine that fails: the line and what went wrong.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptError {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for ScriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for ScriptError {}

#[cfg(test)]
mod dat_tests;
#[cfg(test)]
mod tests;
