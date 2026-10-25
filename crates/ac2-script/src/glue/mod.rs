//! Native glue: the efuns scripts call, keyed by retail efun id (`native type << 16 | n`).
//!
//! [`register_pure`] binds the glue that needs nothing but the VM (strings, the VM queries, ...).
//! Engine-facing glue (UI, weenie, physics, gameplay: `gCWInterface`, `ggmCWInterface`,
//! `gCWPhysicsObject`, `gCM_*`, ...) stays unbound until the client binds it through an
//! [`crate::vm::EFunHost`]; calling it crashes the context with
//! [`crate::vm::Fault::UnboundEFun`], and [`efun_name`] names it.
//!
//! Calling convention (`Interp::HandleCallEFun`): the caller pushes the return slot(s), the
//! arguments last to first, then `this`; the efun sees `this` at `fp - 1`, argument `k` at
//! `fp - 1 - k` (a 64-bit argument takes two slots, low word lower) and writes the return
//! value below the last argument. Int and bool results are tagged `BOOL` (4), references `REF`.

mod behavior;
mod cparse;
mod enums;
mod format;
mod hashes;
mod lists;
mod math;
mod missile;
mod multi;
mod random;
mod string_info;
mod string_text;
mod vm_glue;
mod wcc;
mod wlog;
mod wpstring;

use std::sync::OnceLock;

use crate::native::{ntype, NativeObject};
use crate::vm::{tag, Context, EFuns, Fault, InterpStatus, Vm};

pub use format::{msvc_double, msvc_float};

/// Binds every pure glue function.
pub fn register_pure(efuns: &mut EFuns) {
    vm_glue::register(efuns);
    wcc::register(efuns);
    wpstring::register(efuns);
    string_info::register(efuns);
    string_text::register(efuns);
    enums::register(efuns);
    lists::register(efuns);
    hashes::register(efuns);
    wlog::register(efuns);
    random::register(efuns);
    behavior::register(efuns);
    missile::register(efuns);
    math::register(efuns);
    multi::register(efuns);
}

/// The retail glue function behind efun `id` (`gAAHash::CallEFun_add`).
pub fn efun_name(id: u32) -> Option<&'static str> {
    static TABLE: OnceLock<Vec<(u32, &'static str)>> = OnceLock::new();
    let table = TABLE.get_or_init(|| {
        include_str!("efun_names.txt")
            .lines()
            .filter_map(|line| {
                let (id, name) = line.split_once(' ')?;
                Some((u32::from_str_radix(id, 16).ok()?, name))
            })
            .collect()
    });
    table
        .binary_search_by_key(&id, |(i, _)| *i)
        .ok()
        .map(|i| table[i].1)
}

/// Every retail (efun id, glue name).
pub(crate) fn efun_table() -> impl Iterator<Item = (u32, &'static str)> {
    include_str!("efun_names.txt").lines().filter_map(|line| {
        let (id, name) = line.split_once(' ')?;
        Some((u32::from_str_radix(id, 16).ok()?, name))
    })
}

/// The word `n` slots below the frame pointer of an efun frame (1 is `this`).
pub(crate) fn word(ctx: &Context, n: usize) -> u32 {
    ctx.arg(n).unwrap_or(u32::MAX)
}

/// A 64-bit argument whose low word is `n` slots down.
pub(crate) fn word64(ctx: &Context, n: usize) -> u64 {
    u64::from(word(ctx, n)) | u64::from(word(ctx, n - 1)) << 32
}

/// Every retail efun id.
pub fn efun_ids() -> impl Iterator<Item = u32> {
    include_str!("efun_names.txt")
        .lines()
        .filter_map(|line| u32::from_str_radix(line.split_once(' ')?.0, 16).ok())
}

/// The efun frame: `this`, arguments and the return slot.
pub(crate) struct Call<'a> {
    pub ctx: &'a mut Context,
}

impl<'a> Call<'a> {
    pub fn new(ctx: &'a mut Context) -> Self {
        Self { ctx }
    }

    /// The word `n` slots below the frame pointer (1 is `this`).
    pub fn word(&self, n: usize) -> u32 {
        self.ctx.arg(n).unwrap_or(u32::MAX)
    }

    pub fn this(&self) -> u32 {
        self.word(1)
    }

    /// A 64-bit argument whose low word is `n` slots down (its high word is `n - 1`).
    pub fn word64(&self, n: usize) -> u64 {
        u64::from(self.word(n)) | u64::from(self.word(n - 1)) << 32
    }

    pub fn ret(&mut self, n: usize, value: u32, tag: u8) {
        self.ctx.set_arg(n, value, tag);
    }
}

/// Native-object access for glue.
impl Vm {
    pub fn native(&self, reference: u32) -> Option<&NativeObject> {
        self.refs.heap(reference).ok()?.native.as_ref()
    }

    pub fn native_mut(&mut self, reference: u32) -> Option<&mut NativeObject> {
        self.refs.heap_mut(reference).ok()?.native.as_mut()
    }

    /// A `wpstring`'s characters.
    pub fn wstring(&self, reference: u32) -> Option<&[u16]> {
        match self.native(reference)? {
            NativeObject::WString(s) => Some(s),
            _ => None,
        }
    }

    /// `ReferenceTable::CreateNativeObject(wpstring, ...)`.
    pub fn new_wstring(&mut self, text: Vec<u16>) -> u32 {
        self.refs
            .create_native(ntype::WPSTRING, NativeObject::WString(text))
    }

    /// The crash a glue function returns for a null or foreign `this`
    /// (`index != Undef_ReferenceID`).
    pub(crate) fn bad_this(&mut self, reference: u32) -> InterpStatus {
        self.fault(Fault::BadReference(reference))
    }
}

/// Narrows UTF-16 to bytes the way `PStringBase<unsigned short>::to_spstring` does for ASCII.
pub(crate) fn narrow(text: &[u16]) -> Vec<u8> {
    text.iter().map(|&c| c as u8).collect()
}

/// The tag glue writes on int results.
pub(crate) const INT_RESULT: u8 = tag::BOOL;

#[cfg(test)]
mod tests;
