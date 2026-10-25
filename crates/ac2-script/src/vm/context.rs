//! `ContextState`: one script thread's stack, tags, frame pointer, instruction pointer and call
//! stack.
//!
//! Retail keeps raw pointers; here the stack pointer and frame pointer are slot indices into
//! `m_data` (0x1800 bytes, 1,536 slots, each with a tag byte in `m_tags`), the instruction
//! pointer is a word index into [`super::Program::code`], and `m_callStack` is 192
//! `StackFrame { m_ip, m_fp }` records with `m_sf` pointing at the current one.

use super::tag;

/// Stack slots (`m_data` is 0x1800 bytes).
pub const STACK_SLOTS: usize = 0x600;
/// `StackFrame` records in `m_callStack` (0x600 bytes of 8-byte frames).
pub const CALL_FRAMES: usize = 192;

/// One `StackFrame`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Frame {
    /// The return instruction (the frame's `m_ip` while a callee runs).
    pub ip: usize,
    pub fp: usize,
}

/// `ContextState`.
#[derive(Debug, Clone)]
pub struct Context {
    pub data: Vec<u32>,
    pub tags: Vec<u8>,
    /// `m_sp`: the next free slot.
    pub sp: usize,
    /// `m_fp`: the current frame's base slot (locals at `fp..`, arguments and `this` below).
    pub fp: usize,
    /// `m_ip`: the next instruction word.
    pub ip: usize,
    /// `m_callStack`; the last element is `*m_sf`.
    pub frames: Vec<Frame>,
    /// `m_cid`.
    pub cid: u32,
    /// The owning process's instance state reference, if any.
    pub instance: Option<u32>,
    /// `m_efun`: the efun an interrupted context stopped in (its frame still open); resuming
    /// calls it again before the code continues.
    pub efun: u32,
    /// Whether an efun may stop this context to wait for data the host is loading (contexts
    /// the host resumes; nested initializer and system-call contexts read through).
    pub can_yield: bool,
    /// The context stopped in `efun` waiting for data.
    pub data_wait: bool,
}

/// A stack access outside `m_data` or the call stack: retail corrupts memory, the port stops.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StackFault;

pub type StackResult<T> = Result<T, StackFault>;

impl Context {
    /// `ContextState::Init`.
    pub fn new(cid: u32, instance: Option<u32>) -> Self {
        Self {
            data: vec![0; STACK_SLOTS],
            tags: vec![0; STACK_SLOTS],
            sp: 0,
            fp: 0,
            ip: 0,
            frames: vec![Frame::default()],
            cid,
            instance,
            efun: 0,
            can_yield: false,
            data_wait: false,
        }
    }

    /// Whether the call stack is back at its base (`m_sf == m_callStack`).
    pub fn at_base(&self) -> bool {
        self.frames.len() == 1
    }

    pub fn push(&mut self, value: u32, tag: u8) -> StackResult<()> {
        if self.sp >= STACK_SLOTS {
            return Err(StackFault);
        }
        self.data[self.sp] = value;
        self.tags[self.sp] = tag;
        self.sp += 1;
        Ok(())
    }

    /// Pushes a 64-bit value low word first, both slots tagged `tag`.
    pub fn push64(&mut self, value: u64, tag: u8) -> StackResult<()> {
        self.push(value as u32, tag)?;
        self.push((value >> 32) as u32, tag)
    }

    pub fn pop(&mut self) -> StackResult<u32> {
        self.sp = self.sp.checked_sub(1).ok_or(StackFault)?;
        Ok(self.data[self.sp])
    }

    pub fn pop64(&mut self) -> StackResult<u64> {
        let high = self.pop()?;
        let low = self.pop()?;
        Ok(u64::from(low) | u64::from(high) << 32)
    }

    /// `ContextState::PushDFloat`.
    pub fn push_double(&mut self, value: f64) -> StackResult<()> {
        self.push64(value.to_bits(), tag::DOUBLE)
    }

    /// `ContextState::PushDInt`.
    pub fn push_long(&mut self, value: i64) -> StackResult<()> {
        self.push64(value as u64, tag::LONG_INT)
    }

    /// `ContextState::PushTimetype`.
    pub fn push_time(&mut self, value: f64) -> StackResult<()> {
        self.push64(value.to_bits(), tag::TIME)
    }

    pub fn push_float(&mut self, value: f32) -> StackResult<()> {
        self.push(value.to_bits(), tag::FLOAT)
    }

    pub fn push_int(&mut self, value: i32) -> StackResult<()> {
        self.push(value as u32, tag::INT)
    }

    /// The slot `offset` words from the frame pointer (negative: arguments and `this`).
    pub fn frame_slot(&self, offset: i32) -> StackResult<usize> {
        let slot = self.fp as i64 + i64::from(offset);
        if slot < 0 || slot as usize >= STACK_SLOTS {
            return Err(StackFault);
        }
        Ok(slot as usize)
    }

    /// A slot by index, bounds-checked.
    pub fn slot(&self, index: usize) -> StackResult<(u32, u8)> {
        if index >= STACK_SLOTS {
            return Err(StackFault);
        }
        Ok((self.data[index], self.tags[index]))
    }

    pub fn set_slot(&mut self, index: usize, value: u32, tag: u8) -> StackResult<()> {
        if index >= STACK_SLOTS {
            return Err(StackFault);
        }
        self.data[index] = value;
        self.tags[index] = tag;
        Ok(())
    }

    /// The word at `sp - 1 - depth` without popping.
    pub fn peek(&self, depth: usize) -> StackResult<u32> {
        let index = self.sp.checked_sub(1 + depth).ok_or(StackFault)?;
        Ok(self.data[index])
    }

    /// Enters a function at `entry`: the current frame remembers the return instruction, a
    /// new frame starts with a null frame pointer (`HandleCall`, `VM::CallIFun`).
    pub fn enter(&mut self, entry: usize) -> StackResult<()> {
        if self.frames.len() >= CALL_FRAMES {
            return Err(StackFault);
        }
        if let Some(current) = self.frames.last_mut() {
            current.ip = self.ip;
        }
        self.frames.push(Frame { ip: entry, fp: 0 });
        self.ip = entry;
        self.fp = 0;
        Ok(())
    }

    /// `Interp::HandlePopFrame`: drops the stack to the frame pointer, pops the frame and
    /// resumes the caller's frame pointer and instruction.
    pub fn leave(&mut self) -> StackResult<()> {
        self.sp = self.fp;
        if self.frames.len() <= 1 {
            return Err(StackFault);
        }
        self.frames.pop();
        let caller = *self.frames.last().ok_or(StackFault)?;
        self.fp = caller.fp;
        self.ip = caller.ip;
        Ok(())
    }
}
