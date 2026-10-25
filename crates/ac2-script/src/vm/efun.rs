//! External functions: script -> native calls.
//!
//! An efun id is `native type << 16 | function number` (`CEFunDispatcher::GetEFunAddress`,
//! `gmCEFunDispatcher::GetEFunAddress`: 0x10001 is `gAAHash::CallEFun_add`). `Interp::
//! HandleCallEFun` opens a frame whose frame pointer is the stack top, so an efun reads `this`
//! at `fp - 1`, its arguments below that and writes its return value into the slot the caller
//! reserved under the arguments; the frame is dropped when the efun returns anything but
//! `Interrupted`.

use std::collections::HashMap;

use super::{Context, Fault, InterpStatus, Vm};

/// A native efun implementation.
pub type EFunFn = fn(&mut Vm, &mut Context) -> InterpStatus;

/// Efuns the host binds at run time (UI, weenie, gameplay glue). Return `None` for an efun the
/// host does not implement.
pub trait EFunHost {
    fn call(&mut self, efun: u32, vm: &mut Vm, ctx: &mut Context) -> Option<InterpStatus>;
}

/// The efun table: built-in glue by id, then the host.
#[derive(Default)]
pub struct EFuns {
    table: HashMap<u32, EFunFn>,
    host: Option<Box<dyn EFunHost>>,
}

impl EFuns {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, efun: u32, function: EFunFn) {
        self.table.insert(efun, function);
    }

    pub fn set_host(&mut self, host: Box<dyn EFunHost>) {
        self.host = Some(host);
    }

    pub fn is_bound(&self, efun: u32) -> bool {
        self.table.contains_key(&efun)
    }
}

impl Vm {
    /// Calls efun `efun` with the frame already opened.
    pub(crate) fn call_efun(&mut self, efun: u32, ctx: &mut Context) -> InterpStatus {
        if let Some(function) = self.efuns.table.get(&efun).copied() {
            return function(self, ctx);
        }
        if let Some(mut host) = self.efuns.host.take() {
            let status = host.call(efun, self, ctx);
            self.efuns.host = Some(host);
            if let Some(status) = status {
                return status;
            }
        }
        self.fault(Fault::UnboundEFun(efun))
    }

    /// `Interp::HandleCallEFun` after its operand was read.
    pub(crate) fn run_efun(&mut self, efun: u32, ctx: &mut Context) -> InterpStatus {
        if ctx.frames.len() >= super::context::CALL_FRAMES {
            return self.fault(Fault::Stack);
        }
        if let Some(current) = ctx.frames.last_mut() {
            current.ip = ctx.ip;
        }
        ctx.frames.push(super::Frame { ip: 0, fp: ctx.sp });
        ctx.ip = 0;
        ctx.fp = ctx.sp;
        let status = self.call_efun(efun, ctx);
        if status != InterpStatus::Interrupted && ctx.leave().is_err() {
            return self.fault(Fault::Stack);
        }
        status
    }
}

/// Argument and return slot helpers for efun implementations (`fp - 1` is `this`).
impl Context {
    /// The word `n` slots below the frame pointer (`arg(1)` is `this`).
    pub fn arg(&self, n: usize) -> Option<u32> {
        self.fp.checked_sub(n).map(|i| self.data[i])
    }

    /// The tag of the slot `n` below the frame pointer.
    pub fn arg_tag(&self, n: usize) -> Option<u8> {
        self.fp.checked_sub(n).map(|i| self.tags[i])
    }

    /// Writes the slot `n` below the frame pointer.
    pub fn set_arg(&mut self, n: usize, value: u32, tag: u8) -> bool {
        match self.fp.checked_sub(n) {
            Some(i) => {
                self.data[i] = value;
                self.tags[i] = tag;
                true
            }
            None => false,
        }
    }
}
