//! UI names: element types, element ids, state ids, attribute ids and message ids are
//! `Tokenize` hashes of their names (`portal/shared/src/stringtoken`), held in `Token_*` globals.

/// `Tokenize(char *)`: each (signed) byte, shifted to the top seven bits, rotated right by a
/// shift that advances 7 bits per byte, xor-folded.
pub const fn tokenize(name: &str) -> u32 {
    let bytes = name.as_bytes();
    let mut hash = 0_u32;
    let mut shift = 0_u32;
    let mut i = 0;
    while i < bytes.len() {
        let value = ((bytes[i] as i8 as i32) << 25) as u32;
        hash ^= value >> shift;
        if shift > 25 {
            hash ^= value << ((32 - shift) & 31);
        }
        shift = (shift + 7) & 31;
        i += 1;
    }
    hash
}

/// `Token_reference`: an element that stands for another top-level element of its layout.
pub const REFERENCE: u32 = tokenize("reference");
/// `Token_stateless`: the state every element has besides its current one.
pub const STATELESS: u32 = tokenize("stateless");
/// `Token_referent`: the referenced element's id, an `ID` attribute of a reference's stateless
/// state.
pub const REFERENT: u32 = tokenize("referent");
/// `Token_Element_hide`.
pub const HIDE: u32 = tokenize("hide");
/// `Token_Element_max_width` / `max_height` / `min_width` / `min_height`.
pub const MAX_WIDTH: u32 = tokenize("max_width");
pub const MAX_HEIGHT: u32 = tokenize("max_height");
pub const MIN_WIDTH: u32 = tokenize("min_width");
pub const MIN_HEIGHT: u32 = tokenize("min_height");
/// `Token_Element_root_mover`.
pub const ROOT_MOVER: u32 = tokenize("root_mover");
