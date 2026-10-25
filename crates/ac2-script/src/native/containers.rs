//! The script containers' run-time list form; the hashes and multi-containers are in
//! `hash_table` and `multi`. Letters name the key and value kinds, `A` a 32-bit atom, `L` a
//! 64-bit long, `R` a reference id, `N` a reference id hashed by its native value
//! (`ReferenceIDWrapper`, [`RefKey::Ref`]).

/// A hash key: a 32/64-bit value, or a `ReferenceIDWrapper` (id, cached bucket, use-cache).
pub use ac2_net::protocol::HashKey as RefKey;

/// A list or array: elements widened to u64 (`wide` says they are 64-bit).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScriptList {
    pub wide: bool,
    pub items: Vec<u64>,
}
