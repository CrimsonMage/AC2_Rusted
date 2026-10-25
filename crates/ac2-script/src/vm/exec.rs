//! `Interp::Exec` and the stack, frame, reference and call handlers (`interp_core.cpp`,
//! `interp_optimized.cpp`); the arithmetic families live in `arith.rs`.

use super::context::StackFault;
use super::refs::{flags, Slot, Target};
use super::{op, tag, Context, Fault, FunctionId, InitState, InterpStatus, Vm};
use crate::native::NativeObject;

/// A handler's early exit: a status to return from `Exec`.
pub(super) type Step = Result<(), InterpStatus>;

impl From<StackFault> for Fault {
    fn from(_: StackFault) -> Self {
        Fault::Stack
    }
}

impl Vm {
    /// `Interp::Exec`: fetch and dispatch until a handler returns anything but `Success`. With
    /// the call stack at its base the "instruction" is `TERMINATE`.
    pub(crate) fn exec(&mut self, ctx: &mut Context) -> InterpStatus {
        loop {
            let word = if ctx.at_base() {
                op::TERMINATE
            } else {
                match self.program.code.get(ctx.ip) {
                    Some(&w) => {
                        ctx.ip += 1;
                        w
                    }
                    None => return self.fault(Fault::BadJump),
                }
            };
            match self.step(word, ctx) {
                Ok(()) => {}
                Err(status) => return status,
            }
        }
    }

    /// Converts a fault into the `Crash` exit.
    pub(super) fn crash(&mut self, fault: Fault) -> InterpStatus {
        self.fault(fault)
    }

    pub(super) fn operand(&mut self, ctx: &mut Context) -> Result<u32, InterpStatus> {
        match self.program.code.get(ctx.ip) {
            Some(&w) => {
                ctx.ip += 1;
                Ok(w)
            }
            None => Err(self.crash(Fault::BadJump)),
        }
    }

    /// One instruction.
    fn step(&mut self, word: u32, ctx: &mut Context) -> Step {
        let code = word & 0xFF;
        let tag_byte = (word >> 8) as u8;
        let imm = (word >> 16) as i16 as i32;
        let immu = word >> 16;
        let s = |r: Result<(), StackFault>, vm: &mut Vm| r.map_err(|_| vm.crash(Fault::Stack));
        match code {
            0x01 | 0x81 => Ok(()),
            op::TERMINATE | 0xCA => Err(InterpStatus::Terminated),
            op::CRASH_4B | op::CRASH_4C | 0xCB | 0xCC => Err(InterpStatus::Crash),
            op::PUSH => {
                let v = self.operand(ctx)?;
                s(ctx.push(v, tag_byte), self)
            }
            0x82 => {
                let high_first = self.operand(ctx)?;
                let low = self.operand(ctx)?;
                let t = tag_byte | tag::LONG;
                s(ctx.push(low, t).and_then(|_| ctx.push(high_first, t)), self)
            }
            op::PUSHV => s(ctx.push(imm as u32, tag_byte), self),
            0x83 => {
                let t = tag_byte | tag::LONG;
                s(
                    ctx.push(imm as u32, t)
                        .and_then(|_| ctx.push((imm >> 31) as u32, t)),
                    self,
                )
            }
            op::POP | 0x84 => s(ctx.pop().map(|_| ()), self),
            op::POPN | 0x85 => self.pop_n(ctx, immu as usize),
            op::NEW | 0x86 => {
                let pid = self.operand(ctx)?;
                let r = self
                    .allocate_package(pid, InitState::UseWState, false)
                    .unwrap_or(u32::MAX);
                s(ctx.push(r, tag::REF), self)
            }
            op::NEW_NATIVE | 0x87 => {
                let native_type = self.operand(ctx)?;
                let object = NativeObject::new_empty(native_type);
                let r = self.refs.create_native(native_type, object);
                s(ctx.push(r, tag::REF), self)
            }
            op::NEW_STRING | 0x88 => {
                let n = self.operand(ctx)?;
                let r = self.string_literal(n).unwrap_or(u32::MAX);
                s(ctx.push(r, tag::REF), self)
            }
            op::PUSH_FRAME | 0x89 => self.push_frame(ctx, immu as usize),
            op::POP_FRAME | 0x8A => s(ctx.leave(), self),
            op::CALL | 0x8B => {
                let fid = self.operand(ctx)?;
                self.call(ctx, FunctionId(fid))
            }
            op::CALL_STATIC | 0xD6 => {
                let offset = self.operand(ctx)?;
                self.enter(ctx, offset)
            }
            op::CALL_EFUN | 0x8C => {
                let efun = self.operand(ctx)?;
                self.efun(ctx, efun)
            }
            op::LOAD => self.load(ctx, imm),
            0xA2 => self.load_long(ctx, imm),
            op::LOAD_THIS | 0xA3 => self.load_this(ctx),
            op::STORE => self.store(ctx),
            0xA4 => self.store_long(ctx),
            op::RLOAD => {
                let r = ctx.pop().map_err(|_| self.crash(Fault::Stack))?;
                self.rload(ctx, r, imm, false)
            }
            0xA5 => {
                let r = ctx.pop().map_err(|_| self.crash(Fault::Stack))?;
                self.rload(ctx, r, imm, true)
            }
            op::RSTORE => self.rstore(ctx, imm, false),
            0xA6 => self.rstore(ctx, imm, true),
            op::SADDR | 0xA7 => {
                let offset = (ctx.fp as i64 + i64::from(imm)) as u32 & 0xFFFF;
                let blob = offset << flags::OFFSET_SHIFT | flags::KIND_VALUE;
                let r = self.refs.add(blob, Target::Ref(u32::MAX));
                s(ctx.push(r, tag::REF), self)
            }
            op::RADDR | 0xA8 => {
                let base = ctx.pop().map_err(|_| self.crash(Fault::Stack))?;
                let blob = (immu << flags::OFFSET_SHIFT) | flags::KIND_VALUE;
                let r = self.refs.add(blob, Target::Ref(base));
                s(ctx.push(r, tag::REF), self)
            }
            op::IF | 0xA9 => {
                let v = ctx.pop().map_err(|_| self.crash(Fault::Stack))?;
                if v == 0 {
                    self.jump(ctx, imm)?;
                }
                Ok(())
            }
            op::RJMP | 0xAA => self.jump(ctx, imm),
            op::SWITCH | 0xAB => self.switch(ctx),
            op::STORE_POPN => {
                self.store(ctx)?;
                self.pop_n(ctx, immu as usize)
            }
            op::THIS_CALL => self.this_call(ctx),
            op::THIS_CALL_EFUN => {
                self.load_this(ctx)?;
                let efun = self.operand(ctx)?;
                self.efun(ctx, efun)
            }
            op::LOAD_CALL => {
                self.load(ctx, imm)?;
                let fid = self.operand(ctx)?;
                self.call(ctx, FunctionId(fid))
            }
            op::LOAD_CALL_EFUN => {
                self.load(ctx, imm)?;
                let efun = self.operand(ctx)?;
                self.efun(ctx, efun)
            }
            op::RLOAD_CALL | op::RLOAD_CALL_EFUN => {
                let r = ctx.pop().map_err(|_| self.crash(Fault::Stack))?;
                self.rload(ctx, r, imm, false)?;
                let operand = self.operand(ctx)?;
                if code == op::RLOAD_CALL {
                    self.call(ctx, FunctionId(operand))
                } else {
                    self.efun(ctx, operand)
                }
            }
            op::THIS_RLOAD => {
                let this = ctx
                    .slot(ctx.fp.wrapping_sub(1))
                    .map_err(|_| self.crash(Fault::Stack))?
                    .0;
                self.rload(ctx, this, imm, false)
            }
            _ => self.arith(code, ctx),
        }
    }

    fn pop_n(&mut self, ctx: &mut Context, n: usize) -> Step {
        ctx.sp = ctx
            .sp
            .checked_sub(n)
            .ok_or_else(|| self.crash(Fault::Stack))?;
        Ok(())
    }

    /// `Interp::HandlePushFrame`: the frame pointer becomes the stack top and `n` local slots
    /// follow, tagged uninitialised (their data is left as it was).
    fn push_frame(&mut self, ctx: &mut Context, n: usize) -> Step {
        if ctx.sp + n > super::STACK_SLOTS {
            return Err(self.crash(Fault::Stack));
        }
        if let Some(frame) = ctx.frames.last_mut() {
            frame.fp = ctx.sp;
        }
        ctx.fp = ctx.sp;
        ctx.tags[ctx.sp..ctx.sp + n].fill(tag::UNINIT);
        ctx.sp += n;
        Ok(())
    }

    /// Relative jumps count bytes from the next instruction.
    fn jump(&mut self, ctx: &mut Context, bytes: i32) -> Step {
        if bytes % 4 != 0 {
            return Err(self.crash(Fault::BadJump));
        }
        let target = ctx.ip as i64 + i64::from(bytes / 4);
        if target < 0 {
            return Err(self.crash(Fault::BadJump));
        }
        ctx.ip = target as usize;
        Ok(())
    }

    /// `Interp::HandleSwitch`: default offset, case count, (value, offset) pairs; every
    /// offset counts from the end of the table.
    fn switch(&mut self, ctx: &mut Context) -> Step {
        let value = ctx.pop().map_err(|_| self.crash(Fault::Stack))?;
        let default = self.operand(ctx)? as i32;
        let n = self.operand(ctx)? as i32;
        let table = ctx.ip;
        let end = table + 2 * n.max(0) as usize;
        let mut target = default;
        if end > self.program.code.len() {
            return Err(self.crash(Fault::BadJump));
        }
        let program = self.program.clone();
        for pair in program.code[table..end].chunks_exact(2) {
            if pair[0] == value {
                target = pair[1] as i32;
                break;
            }
        }
        ctx.ip = end;
        self.jump(ctx, target)
    }

    fn load(&mut self, ctx: &mut Context, offset: i32) -> Step {
        let slot = ctx
            .frame_slot(offset)
            .map_err(|_| self.crash(Fault::Stack))?;
        let (v, t) = (ctx.data[slot], ctx.tags[slot]);
        ctx.push(v, t).map_err(|_| self.crash(Fault::Stack))
    }

    fn load_long(&mut self, ctx: &mut Context, offset: i32) -> Step {
        let slot = ctx
            .frame_slot(offset)
            .map_err(|_| self.crash(Fault::Stack))?;
        let (low, t) = (ctx.data[slot], ctx.tags[slot] | tag::LONG);
        let high = ctx.slot(slot + 1).map_err(|_| self.crash(Fault::Stack))?.0;
        ctx.push(low, t)
            .and_then(|_| ctx.push(high, t))
            .map_err(|_| self.crash(Fault::Stack))
    }

    fn load_this(&mut self, ctx: &mut Context) -> Step {
        let this = ctx
            .slot(ctx.fp.wrapping_sub(1))
            .map_err(|_| self.crash(Fault::Stack))?
            .0;
        ctx.push(this, tag::REF)
            .map_err(|_| self.crash(Fault::Stack))
    }

    /// `Interp::HandleStore`: pops the local index; the value stays on the stack.
    fn store(&mut self, ctx: &mut Context) -> Step {
        let index = ctx.pop().map_err(|_| self.crash(Fault::Stack))? as i32;
        let top = ctx
            .sp
            .checked_sub(1)
            .ok_or_else(|| self.crash(Fault::Stack))?;
        let (v, t) = (ctx.data[top], ctx.tags[top]);
        let slot = ctx
            .frame_slot(index)
            .map_err(|_| self.crash(Fault::Stack))?;
        ctx.set_slot(slot, v, t)
            .map_err(|_| self.crash(Fault::Stack))
    }

    /// `Interp::HandleStore_Long`: both words, but only the first slot's tag.
    fn store_long(&mut self, ctx: &mut Context) -> Step {
        let index = ctx.pop().map_err(|_| self.crash(Fault::Stack))? as i32;
        let low_at = ctx
            .sp
            .checked_sub(2)
            .ok_or_else(|| self.crash(Fault::Stack))?;
        let (low, high, t) = (ctx.data[low_at], ctx.data[low_at + 1], ctx.tags[low_at]);
        let slot = ctx
            .frame_slot(index)
            .map_err(|_| self.crash(Fault::Stack))?;
        if slot + 1 >= super::STACK_SLOTS {
            return Err(self.crash(Fault::Stack));
        }
        ctx.data[slot] = low;
        ctx.data[slot + 1] = high;
        ctx.tags[slot] = t;
        Ok(())
    }

    /// Locates `offset` words into `reference`, filling a lazy singleton on the way.
    pub(crate) fn locate(&mut self, reference: u32, offset: i32) -> Result<Slot, InterpStatus> {
        for _ in 0..2 {
            match self.refs.locate(reference, i64::from(offset)) {
                Ok(Some(slot)) => return Ok(slot),
                Ok(None) => self.fill_package(reference),
                Err(e) => return Err(self.crash(Fault::BadReference(e.0))),
            }
        }
        Err(self.crash(Fault::BadReference(reference)))
    }

    fn rload(&mut self, ctx: &mut Context, reference: u32, offset: i32, long: bool) -> Step {
        let slot = self.locate(reference, offset)?;
        let (v, t) = self
            .refs
            .read(ctx, slot)
            .map_err(|e| self.crash(Fault::BadReference(e.0)))?;
        if !long {
            return ctx.push(v, t).map_err(|_| self.crash(Fault::Stack));
        }
        let next = match slot {
            Slot::Stack(i) => Slot::Stack(i + 1),
            Slot::Heap { reference, index } => Slot::Heap {
                reference,
                index: index + 1,
            },
        };
        let (high, _) = self
            .refs
            .read(ctx, next)
            .map_err(|e| self.crash(Fault::BadReference(e.0)))?;
        let t = t | tag::LONG;
        ctx.push(v, t)
            .and_then(|_| ctx.push(high, t))
            .map_err(|_| self.crash(Fault::Stack))
    }

    /// `Interp::HandleRStore(_Long)`: pops the reference; the value stays on the stack.
    fn rstore(&mut self, ctx: &mut Context, offset: i32, long: bool) -> Step {
        let reference = ctx.pop().map_err(|_| self.crash(Fault::Stack))?;
        let words = if long { 2 } else { 1 };
        let at = ctx
            .sp
            .checked_sub(words)
            .ok_or_else(|| self.crash(Fault::Stack))?;
        let (low, t) = (ctx.data[at], ctx.tags[at]);
        let slot = self.locate(reference, offset)?;
        self.refs
            .write(ctx, slot, low, t)
            .map_err(|e| self.crash(Fault::BadReference(e.0)))?;
        if long {
            let high = ctx.data[at + 1];
            let next = match slot {
                Slot::Stack(i) => Slot::Stack(i + 1),
                Slot::Heap { reference, index } => Slot::Heap {
                    reference,
                    index: index + 1,
                },
            };
            // Only the first word's tag is written; keep the second slot's tag.
            let (_, old_tag) = self
                .refs
                .read(ctx, next)
                .map_err(|e| self.crash(Fault::BadReference(e.0)))?;
            self.refs
                .write(ctx, next, high, old_tag)
                .map_err(|e| self.crash(Fault::BadReference(e.0)))?;
        }
        Ok(())
    }

    /// Enters the function at byte `offset` unless it is `u32::MAX` (no such function: retail
    /// skips the call and carries on).
    fn enter(&mut self, ctx: &mut Context, offset: u32) -> Step {
        if offset == u32::MAX {
            return Ok(());
        }
        ctx.enter((offset / 4) as usize)
            .map_err(|_| self.crash(Fault::Stack))
    }

    /// `Interp::HandleCall`: a virtual call on the `this` at the stack top.
    fn call(&mut self, ctx: &mut Context, fid: FunctionId) -> Step {
        let this = ctx.peek(0).map_err(|_| self.crash(Fault::Stack))?;
        let pkg = self.package_id(this).map_err(|f| self.crash(f))?;
        let offset = self
            .program
            .virtual_function_offset(pkg, fid)
            .unwrap_or(u32::MAX);
        self.enter(ctx, offset)
    }

    /// `Interp::HandleOptimized_This_Call`: push `this`, then call through
    /// `Resolver::GetFunctionOffset` (an absolute id names its package).
    fn this_call(&mut self, ctx: &mut Context) -> Step {
        self.load_this(ctx)?;
        let this = ctx.peek(0).map_err(|_| self.crash(Fault::Stack))?;
        let fid = FunctionId(self.operand(ctx)?);
        let pkg = if fid.is_absolute() {
            fid.pkg_num()
        } else {
            self.package_id(this).map_err(|f| self.crash(f))?
        };
        let offset = self.program.function_offset(pkg, fid).unwrap_or(u32::MAX);
        self.enter(ctx, offset)
    }

    fn efun(&mut self, ctx: &mut Context, efun: u32) -> Step {
        match self.run_efun(efun, ctx) {
            InterpStatus::Success => Ok(()),
            status => Err(status),
        }
    }
}
