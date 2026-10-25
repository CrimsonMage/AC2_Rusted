//! The resolved library the interpreter runs: `Resolver::ProcessByteStream` over a
//! [`ByteStream`].
//!
//! The opcode stream is 32-bit words; an instruction is an op word (opcode in the low byte, a
//! tag byte, a signed 16-bit immediate in the high half) plus the operand words
//! `OpcodeStream::WalkStream` counts. At load, `ProcessByteStream` rewrites every `Call` (0x0B)
//! whose operand has the top bit set into a `CallStatic` (0x56) to the virtual function's
//! resolved offset (`Resolver::GetVirtualFunctionOffset`); `CallEFun` operands stay efun ids
//! here (retail swaps them for function pointers, [`crate::vm::EFuns`] looks them up per call).

use crate::bytestream::ByteStream;

/// Opcodes (`Interp::SetupHandlers`). `| LONG` selects the 64-bit form where one exists.
pub mod op {
    pub const NOP: u32 = 0x01;
    pub const PUSH: u32 = 0x02;
    pub const PUSHV: u32 = 0x03;
    pub const POP: u32 = 0x04;
    pub const POPN: u32 = 0x05;
    pub const NEW: u32 = 0x06;
    pub const NEW_NATIVE: u32 = 0x07;
    pub const NEW_STRING: u32 = 0x08;
    pub const PUSH_FRAME: u32 = 0x09;
    pub const POP_FRAME: u32 = 0x0A;
    pub const CALL: u32 = 0x0B;
    pub const CALL_EFUN: u32 = 0x0C;
    pub const ADD: u32 = 0x0D;
    pub const SUB: u32 = 0x0E;
    pub const MULT: u32 = 0x0F;
    pub const DIV: u32 = 0x10;
    pub const MOD: u32 = 0x11;
    pub const NEGATE: u32 = 0x12;
    pub const INC: u32 = 0x13;
    pub const DEC: u32 = 0x14;
    pub const FADD: u32 = 0x15;
    pub const FSUB: u32 = 0x16;
    pub const FMULT: u32 = 0x17;
    pub const FDIV: u32 = 0x18;
    pub const FNEGATE: u32 = 0x19;
    pub const FINC: u32 = 0x1A;
    pub const FDEC: u32 = 0x1B;
    pub const I2F: u32 = 0x1C;
    pub const F2I: u32 = 0x1D;
    pub const I2I: u32 = 0x1E;
    pub const F2F: u32 = 0x1F;
    pub const F2T: u32 = 0x20;
    pub const LOAD: u32 = 0x22;
    pub const LOAD_THIS: u32 = 0x23;
    pub const STORE: u32 = 0x24;
    pub const RLOAD: u32 = 0x25;
    pub const RSTORE: u32 = 0x26;
    pub const SADDR: u32 = 0x27;
    pub const RADDR: u32 = 0x28;
    pub const IF: u32 = 0x29;
    pub const RJMP: u32 = 0x2A;
    pub const SWITCH: u32 = 0x2B;
    pub const L_NOT: u32 = 0x2C;
    pub const L_OR: u32 = 0x2D;
    pub const L_AND: u32 = 0x2E;
    pub const L_EQ: u32 = 0x2F;
    pub const L_NEQ: u32 = 0x30;
    pub const L_SLT: u32 = 0x31;
    pub const L_SGT: u32 = 0x32;
    pub const L_SLTE: u32 = 0x33;
    pub const L_SGTE: u32 = 0x34;
    pub const L_ULT: u32 = 0x35;
    pub const L_UGT: u32 = 0x36;
    pub const L_ULTE: u32 = 0x37;
    pub const L_UGTE: u32 = 0x38;
    pub const F_L_NOT: u32 = 0x39;
    pub const L_FOR: u32 = 0x3A;
    pub const L_FAND: u32 = 0x3B;
    pub const L_FEQ: u32 = 0x3C;
    pub const L_FNEQ: u32 = 0x3D;
    pub const L_FLT: u32 = 0x3E;
    pub const L_FGT: u32 = 0x3F;
    pub const L_FLTE: u32 = 0x40;
    pub const L_FGTE: u32 = 0x41;
    pub const B_NEGATE: u32 = 0x42;
    pub const B_OR: u32 = 0x43;
    pub const B_XOR: u32 = 0x44;
    pub const B_AND: u32 = 0x45;
    pub const B_LSHIFT: u32 = 0x46;
    pub const B_RSHIFT: u32 = 0x47;
    /// Reference equality (the `O_L_EQ` handler).
    pub const R_EQ: u32 = 0x48;
    pub const R_NEQ: u32 = 0x49;
    /// The end of a program: what `Interp::Exec` runs once the call stack is empty.
    pub const TERMINATE: u32 = 0x4A;
    pub const CRASH_4B: u32 = 0x4B;
    pub const CRASH_4C: u32 = 0x4C;
    pub const STORE_POPN: u32 = 0x4D;
    pub const THIS_CALL: u32 = 0x4F;
    pub const THIS_CALL_EFUN: u32 = 0x50;
    pub const LOAD_CALL: u32 = 0x51;
    pub const LOAD_CALL_EFUN: u32 = 0x52;
    pub const RLOAD_CALL: u32 = 0x53;
    pub const RLOAD_CALL_EFUN: u32 = 0x54;
    pub const THIS_RLOAD: u32 = 0x55;
    pub const CALL_STATIC: u32 = 0x56;
    pub const U2U: u32 = 0x60;
    pub const B_URSHIFT: u32 = 0x61;
    pub const U2F: u32 = 0x62;
    /// The 64-bit form bit (`Interp::AddLongHandler`).
    pub const LONG: u32 = 0x80;
}

/// `FunctionID` as packed: `funcNum` in the low 16 bits, `pkgNum` in bits 16-30, `absFlag` in
/// bit 31.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FunctionId(pub u32);

impl FunctionId {
    pub fn new(pkg: u32, func: u32, absolute: bool) -> Self {
        Self((pkg & 0x7FFF) << 16 | (func & 0xFFFF) | if absolute { 0x8000_0000 } else { 0 })
    }

    pub fn func_num(self) -> u32 {
        self.0 & 0xFFFF
    }

    pub fn pkg_num(self) -> u32 {
        (self.0 >> 16) & 0x7FFF
    }

    pub fn is_absolute(self) -> bool {
        self.0 & 0x8000_0000 != 0
    }
}

/// The library as the interpreter sees it.
#[derive(Debug, Clone, Default)]
pub struct Program {
    /// The opcode stream as words, with `ProcessByteStream`'s rewrites applied.
    pub code: Vec<u32>,
    /// `m_funcIDMap` (`VTableSection::m_funcMapper`), by package id.
    pub func_ids: Vec<Vec<u32>>,
    /// `m_vtbl`, by vtable index: byte offsets into the opcode stream.
    pub vtables: Vec<Vec<u32>>,
    /// `m_pkgInfo` slot counts, by package id.
    pub package_sizes: Vec<u32>,
    /// `m_pkgIdMap`: package id -> vtable index.
    pub package_vtable: Vec<u32>,
    /// The string literals (`StringLitTable`), UTF-16.
    pub string_literals: Vec<Vec<u16>>,
    /// `m_pkgIdStrMap`.
    pub package_names: Vec<(Vec<u8>, u32)>,
}

impl Program {
    /// `Resolver::ProcessByteStream`.
    pub fn from_stream(stream: &ByteStream) -> Self {
        let mut program = Self {
            code: stream
                .opcodes
                .chunks_exact(4)
                .map(|w| u32::from_le_bytes([w[0], w[1], w[2], w[3]]))
                .collect(),
            func_ids: stream
                .vtables
                .func_mapper
                .iter()
                .map(|ids| ids.iter().map(|id| id.0).collect())
                .collect(),
            vtables: stream.vtables.vtables.clone(),
            package_sizes: stream.vtables.package_info.iter().map(|i| i.size).collect(),
            package_vtable: stream.vtables.package_id_map.clone(),
            string_literals: stream.string_literals.clone(),
            package_names: stream.vtables.package_names.clone(),
        };
        program.resolve_virtual_calls();
        program
    }

    /// A program over hand-built code (tests and tools).
    pub fn from_parts(code: Vec<u32>, func_ids: Vec<Vec<u32>>, vtables: Vec<Vec<u32>>) -> Self {
        let n = func_ids.len();
        let mut program = Self {
            code,
            package_vtable: (0..n as u32).map(|i| i.min(vtables.len() as u32)).collect(),
            func_ids,
            vtables,
            package_sizes: vec![0; n],
            ..Self::default()
        };
        program.resolve_virtual_calls();
        program
    }

    fn resolve_virtual_calls(&mut self) {
        let mut pc = 0;
        while pc < self.code.len() {
            let word = self.code[pc];
            if word & 0xFF == op::CALL {
                if let Some(&target) = self.code.get(pc + 1) {
                    if target & 0x8000_0000 != 0 {
                        let fid = FunctionId(target & 0x7FFF_FFFF);
                        let offset = self
                            .virtual_function_offset(fid.pkg_num(), fid)
                            .unwrap_or(u32::MAX);
                        self.code[pc] = op::CALL_STATIC;
                        self.code[pc + 1] = offset;
                    }
                }
            }
            pc = self.walk(pc);
        }
    }

    /// `OpcodeStream::WalkStream`: the word index of the instruction after the one at `pc`.
    pub fn walk(&self, pc: usize) -> usize {
        match self.code[pc] & 0xFF {
            0x02 | 0x06 | 0x07 | 0x08 | 0x0B | 0x0C | 0x4F..=0x54 | 0x56 => pc + 2,
            0x2B => pc + 3 + 2 * self.code.get(pc + 2).copied().unwrap_or(0) as usize,
            0x82 => pc + 3,
            _ => pc + 1,
        }
    }

    /// The vtable slot `fid` names in package `pkg`'s vtable (shared by
    /// `GetVirtualFunctionOffset`, `GetFunctionOffset` and `Interp::HandleCall`).
    fn slot(&self, pkg: u32, fid: FunctionId) -> Option<u32> {
        let vtable = *self.package_vtable.get(pkg as usize)?;
        let slot = *self
            .func_ids
            .get(fid.pkg_num() as usize)?
            .get(fid.func_num() as usize)?
            & 0xFFFF;
        let offset = *self.vtables.get(vtable as usize)?.get(slot as usize)?;
        (offset != u32::MAX).then_some(offset)
    }

    /// `Resolver::GetVirtualFunctionOffset`: a byte offset into the opcode stream.
    pub fn virtual_function_offset(&self, vtbl_pkg: u32, fid: FunctionId) -> Option<u32> {
        self.slot(vtbl_pkg, fid)
    }

    /// `Resolver::GetFunctionOffset`: an absolute id names its own package.
    pub fn function_offset(&self, inst_pkg: u32, fid: FunctionId) -> Option<u32> {
        if fid.is_absolute() {
            self.slot(fid.pkg_num(), FunctionId(fid.0 & 0x7FFF_FFFF))
        } else {
            self.slot(inst_pkg, fid)
        }
    }

    /// `PackageInfo::size` of package `pkg` (0 when out of range).
    pub fn package_size(&self, pkg: u32) -> u32 {
        self.package_sizes.get(pkg as usize).copied().unwrap_or(0)
    }

    /// `VM::GetPackageID(name)`.
    pub fn package_id(&self, name: &str) -> Option<u32> {
        self.package_names
            .iter()
            .find(|(n, _)| n.as_slice() == name.as_bytes())
            .map(|(_, id)| *id)
    }
}
