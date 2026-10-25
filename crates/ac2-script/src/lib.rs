#![forbid(unsafe_code)]

//! The WSL script system (index pack 10, `portal/shared/src/interp` and the `interp/glue`
//! directories): much of AC2's UI and gameplay logic is compiled WSL running on this VM.
//!
//! - [`wlib`]: the compiled library (`WLib`, one `ByteStream`: opcodes, string literals,
//!   package exports and vtables).
//! - [`wstate`]: saved reference graphs (`WState`: package singletons, weenie states), with
//!   [`reference`] (`ReferenceEntry`/`HeapObject`) and [`native`] (the engine objects heap
//!   objects can hold).
//!
//! DAT strings are scrambled; [`pstring`] undoes `crypto_uint32_munge`/`crypto_nibble_munge`.

pub mod bytestream;
pub mod data;
pub mod glue;
pub mod native;
pub mod pstring;
pub mod reference;
pub mod unpack;
pub mod vm;
pub mod wlib;
pub mod wstate;

pub use bytestream::ByteStream;
pub use reference::{HeapObject, PackedEntry, PackedReference};
pub use unpack::UnpackError;
pub use wlib::{load_wlib, WLib, LEGIONS_WLIB_DID};
pub use wstate::{load_wstate, WState};

#[cfg(test)]
mod real_data_tests;
#[cfg(test)]
mod real_data_vm_tests;
